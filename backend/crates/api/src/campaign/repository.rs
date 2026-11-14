use crate::game::{Error, Identity};
use postgres::{Client, NoTls, Transaction};
use serde_json::{Value, json};
use wonderwilds_domain::campaign::{self, CampaignDefinition, CampaignState};

#[derive(Clone)]
pub struct Repository {
    connection: Option<String>,
    #[cfg(test)]
    fail_before_commit: bool,
}
impl Repository {
    pub fn new(connection: Option<String>) -> Self {
        Self {
            connection,
            #[cfg(test)]
            fail_before_commit: false,
        }
    }
    pub fn readiness(&self) -> Result<(), Error> {
        let mut client =
            Client::connect(self.connection.as_deref().ok_or(Error::Unavailable)?, NoTls)
                .map_err(|_| Error::Unavailable)?;
        let ready:bool=client.query_one("SELECT to_regclass('wonderwilds.campaign_progress') IS NOT NULL AND to_regclass('wonderwilds.campaign_command_receipts') IS NOT NULL",&[]).map_err(|_|Error::Unavailable)?.get(0);
        definition()?;
        if ready {
            Ok(())
        } else {
            Err(Error::Unavailable)
        }
    }
    pub fn run(&self, identity: &Identity, request: Option<&Value>) -> Result<Value, Error> {
        if let Some(r) = request {
            wonderwilds_contracts::validate("CampaignCommand", r)
                .map_err(|_| Error::InvalidInput)?;
        }
        let mut c = Client::connect(self.connection.as_deref().ok_or(Error::Unavailable)?, NoTls)
            .map_err(|_| Error::Unavailable)?;
        let mut tx = c.transaction().map_err(|_| Error::Unavailable)?;
        own(&mut tx, identity)?;
        if let Some(r) = request
            && let Some(row)=tx.query_opt("SELECT request,response,digest FROM wonderwilds.campaign_command_receipts WHERE profile_id=$1::text::uuid AND command_id=$2",&[&identity.profile_id,&r["commandId"].as_str().ok_or(Error::InvalidInput)?]).map_err(|_|Error::Unavailable)? {
            if row.get::<_,Value>(0)!=*r {return Err(Error::RevisionConflict)}
            let response:Value=row.get(1);
            if row.get::<_,String>(2)!=crate::persistence::receipt(r,&response,&json!({})) {return Err(Error::Unavailable)}
            wonderwilds_contracts::validate("CampaignSnapshot",&response).map_err(|_|Error::Unavailable)?;
            tx.commit().map_err(|_|Error::Unavailable)?;
            return Ok(response)
        }
        let (mut revision, mut state) = load(&mut tx, identity)?;
        let def = match &state {
            Some(s) => definition_version(&s.content_version)?,
            None => definition()?,
        };
        let owned = crate::game::campaign_companions(&mut tx, identity)?;
        if let Some(r) = request {
            if r["expectedRevision"].as_u64() != Some(revision) {
                return Err(Error::RevisionConflict);
            }
            let payload = &r["payload"];
            if payload["type"] == "adopt" {
                if state.is_some() || blocked(&mut tx, identity)? {
                    return Err(Error::RevisionConflict);
                }
                state = Some(campaign::adopt(&def));
            } else {
                let current = state.as_ref().ok_or(Error::NotFound)?;
                let command =
                    serde_json::from_value(payload.clone()).map_err(|_| Error::InvalidInput)?;
                let out = campaign::apply(&def, current, &command, &owned, false)
                    .map_err(domain_error)?;
                for id in out.recruit {
                    crate::game::campaign_grant(
                        &mut tx,
                        identity,
                        &id,
                        &format!("campaign:recruit:{id}"),
                        0,
                    )?;
                }
                if let campaign::CampaignCommand::Care { companion_id, kind } = &command
                    && matches!(kind.as_str(), "appearance_wayfinder" | "appearance_base")
                {
                    crate::game::campaign_appearance(
                        &mut tx,
                        identity,
                        companion_id,
                        kind == "appearance_wayfinder",
                        r["commandId"].as_str().ok_or(Error::InvalidInput)?,
                    )?;
                }
                state = Some(out.state);
            }
            revision += 1;
            save(
                &mut tx,
                identity,
                revision,
                state.as_ref().ok_or(Error::Unavailable)?,
            )?;
        }
        let response = projection(&mut tx, identity, &def, revision, state.as_ref())?;
        if let Some(r) = request {
            tx.execute("INSERT INTO wonderwilds.campaign_command_receipts(profile_id,command_id,revision,request,response,digest) VALUES($1::text::uuid,$2,$3,$4,$5,$6)",&[&identity.profile_id,&r["commandId"].as_str().ok_or(Error::InvalidInput)?,&(revision as i64),r,&response,&crate::persistence::receipt(r,&response,&json!({}))]).map_err(|_|Error::Unavailable)?;
        }
        #[cfg(test)]
        if self.fail_before_commit {
            return Err(Error::Unavailable);
        }
        tx.commit().map_err(|_| Error::Unavailable)?;
        Ok(response)
    }
}
pub(crate) fn definition() -> Result<CampaignDefinition, Error> {
    wonderwilds_content::campaign().map_err(|_| Error::Unavailable)
}
fn domain_error(e: campaign::CampaignError) -> Error {
    match e {
        campaign::CampaignError::VersionMismatch | campaign::CampaignError::InvalidContent(_) => {
            Error::Unavailable
        }
        campaign::CampaignError::LearningRequired => Error::Forbidden,
        _ => Error::InvalidInput,
    }
}
fn own(tx: &mut Transaction<'_>, identity: &Identity) -> Result<(), Error> {
    tx.batch_execute("SET LOCAL statement_timeout='2000ms';SET LOCAL lock_timeout='1000ms';")
        .map_err(|_| Error::Unavailable)?;
    tx.query_one(
        "SELECT set_config('app.parent_id',$1,true)",
        &[&identity.parent_id],
    )
    .map_err(|_| Error::Unavailable)?;
    if tx.query_opt("SELECT id FROM wonderwilds.profiles WHERE id=$1::text::uuid AND parent_id=$2::text::uuid FOR UPDATE",&[&identity.profile_id,&identity.parent_id]).map_err(|_|Error::Unavailable)?.is_none(){return Err(Error::Forbidden)}
    Ok(())
}
fn load(
    tx: &mut Transaction<'_>,
    identity: &Identity,
) -> Result<(u64, Option<CampaignState>), Error> {
    let row=tx.query_opt("SELECT revision,checkpoint FROM wonderwilds.campaign_progress WHERE profile_id=$1::text::uuid",&[&identity.profile_id]).map_err(|_|Error::Unavailable)?;
    match row {
        None => Ok((0, None)),
        Some(row) => {
            let value: Value = row.get(1);
            let stored = crate::persistence::unwrap(&value, "campaign-storage-1")?;
            if stored["contentFingerprint"]
                != crate::persistence::digest(
                    &serde_json::to_value(definition_version(
                        stored["campaign"]["content_version"]
                            .as_str()
                            .ok_or(Error::Unavailable)?,
                    )?)
                    .map_err(|_| Error::Unavailable)?,
                )
            {
                return Err(Error::Unavailable);
            }
            let state = serde_json::from_value(stored["campaign"].clone())
                .map_err(|_| Error::Unavailable)?;
            let revision = row.get::<_, i64>(0) as u64;
            let receipt=tx.query_opt("SELECT revision,request,response,digest FROM wonderwilds.campaign_command_receipts WHERE profile_id=$1::text::uuid ORDER BY revision DESC LIMIT 1",&[&identity.profile_id]).map_err(|_|Error::Unavailable)?.ok_or(Error::Unavailable)?;
            let response: Value = receipt.get(2);
            if receipt.get::<_, i64>(0) as u64 != revision
                || response["campaign"]["state"] != stored["campaign"]
                || receipt.get::<_, String>(3)
                    != crate::persistence::receipt(&receipt.get(1), &response, &json!({}))
            {
                return Err(Error::Unavailable);
            }
            Ok((revision, Some(state)))
        }
    }
}
fn save(
    tx: &mut Transaction<'_>,
    identity: &Identity,
    revision: u64,
    state: &CampaignState,
) -> Result<(), Error> {
    let value = crate::persistence::envelope(
        "campaign-storage-1",
        json!({"contentFingerprint":crate::persistence::digest(&serde_json::to_value(definition_version(&state.content_version)?).map_err(|_|Error::Unavailable)?),"campaign":state}),
    );
    tx.execute("INSERT INTO wonderwilds.campaign_progress(profile_id,revision,checkpoint) VALUES($1::text::uuid,$2,$3) ON CONFLICT(profile_id) DO UPDATE SET revision=EXCLUDED.revision,checkpoint=EXCLUDED.checkpoint",&[&identity.profile_id,&(revision as i64),&value]).map_err(|_|Error::Unavailable)?;
    Ok(())
}
fn blocked(tx: &mut Transaction<'_>, identity: &Identity) -> Result<bool, Error> {
    Ok(crate::game::campaign_pending(tx, identity)?
        || crate::mission_learning::repository::pending_activity(tx, identity)?)
}
fn projection(
    tx: &mut Transaction<'_>,
    identity: &Identity,
    def: &CampaignDefinition,
    revision: u64,
    state: Option<&CampaignState>,
) -> Result<Value, Error> {
    let mut value = json!({"schemaVersion":"1.0","revision":revision,"adopted":state.is_some(),"adoptionBlocked":if state.is_none(){blocked(tx,identity)?}else{false},"locations":def.locations.iter().map(|l|json!({"id":l.id,"name":l.name,"camp":l.camp})).collect::<Vec<_>>(),"starters":def.companions.iter().filter(|c|def.starters.contains(&c.id)).map(|c|json!({"id":c.id,"name":c.name,"affinity":c.affinity})).collect::<Vec<_>>()});
    if let Some(s) = state {
        value["campaign"] = serde_json::to_value(
            campaign::snapshot(def, s, &crate::game::campaign_companions(tx, identity)?)
                .map_err(domain_error)?,
        )
        .map_err(|_| Error::Unavailable)?;
    }
    if let Some(companions) = value
        .get_mut("campaign")
        .and_then(|campaign| campaign.get_mut("companions"))
        .and_then(Value::as_array_mut)
    {
        let appearances = crate::game::campaign_appearances(tx, identity)?;
        for companion in companions {
            if let Some(affinity) = companion["id"]
                .as_str()
                .and_then(|id| def.companions.iter().find(|c| c.id == id))
                .map(|c| &c.affinity)
            {
                companion["affinity"] = json!(affinity);
            }
            if let Some(appearance) = companion["id"].as_str().and_then(|id| appearances.get(id)) {
                companion["appearance"] = json!(appearance);
            }
        }
    }
    wonderwilds_contracts::validate("CampaignSnapshot", &value).map_err(|_| Error::Unavailable)?;
    Ok(value)
}

/// Frozen server-resolved join; the learner cannot redirect its recipient.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct LearningStep {
    pub action_id: String,
    pub campaign_revision: u64,
    pub content_version: String,
    pub rules_version: String,
    pub recipient: String,
    #[serde(default)]
    pub milestone: Option<String>,
    pub location: String,
    pub mission: String,
    pub label: String,
    pub introduction: String,
}
pub(crate) fn learning_step(
    tx: &mut Transaction<'_>,
    identity: &Identity,
    action_id: &str,
    revision: u64,
) -> Result<LearningStep, Error> {
    let (rev, state) = load(tx, identity)?;
    if rev != revision {
        return Err(Error::RevisionConflict);
    }
    let state = state.ok_or(Error::NotFound)?;
    let def = definition_version(&state.content_version)?;
    let owned = crate::game::campaign_companions(tx, identity)?;
    let projection = campaign::snapshot(&def, &state, &owned).map_err(domain_error)?;
    let action = projection
        .legal_actions
        .iter()
        .find(|a| a.id == action_id && a.learning)
        .ok_or(Error::InvalidInput)?;
    let recipient = action
        .recipient
        .as_ref()
        .or(state.team.first())
        .ok_or(Error::NotFound)?
        .clone();
    if !owned.iter().any(|c| c.id == recipient) {
        return Err(Error::Forbidden);
    }
    Ok(LearningStep {
        action_id: action_id.to_owned(),
        campaign_revision: revision,
        content_version: def.version,
        rules_version: def.rules_version,
        recipient,
        milestone: action.milestone.clone(),
        location: state.location,
        mission: action.mission.clone(),
        label: action.label.clone(),
        introduction: action
            .learning_context
            .clone()
            .unwrap_or_else(|| "Let's help our companion with this discovery.".into()),
    })
}
pub(crate) fn complete_learning(
    tx: &mut Transaction<'_>,
    identity: &Identity,
    step: &LearningStep,
    command_id: &str,
) -> Result<(), Error> {
    let def = definition_version(&step.content_version)?;
    if step.content_version != def.version || step.rules_version != def.rules_version {
        return Err(Error::Unavailable);
    }
    let (rev, state) = load(tx, identity)?;
    let current = state.ok_or(Error::Unavailable)?;
    if current.location != step.location {
        return Ok(());
    }
    let owned = crate::game::campaign_companions(tx, identity)?;
    if !owned.iter().any(|c| c.id == step.recipient) {
        return Err(Error::Unavailable);
    }
    let mut bound_definition = def.clone();
    if let Some(action) = bound_definition
        .actions
        .iter_mut()
        .find(|a| a.id == step.action_id)
    {
        action.recipient = Some(step.recipient.clone());
    }
    let out = match campaign::apply(
        &bound_definition,
        &current,
        &campaign::CampaignCommand::Action {
            action_id: step.action_id.clone(),
        },
        &owned,
        true,
    ) {
        Ok(out) => out,
        Err(campaign::CampaignError::VersionMismatch) => return Err(Error::Unavailable),
        Err(_) => return Ok(()),
    };
    for id in out.recruit {
        crate::game::campaign_grant(tx, identity, &id, &format!("campaign:recruit:{id}"), 0)?;
    }
    if def
        .actions
        .iter()
        .any(|a| a.id == step.action_id && a.kind == "training")
    {
        crate::game::campaign_grant(
            tx,
            identity,
            &step.recipient,
            &format!(
                "campaign:{}:{}",
                step.milestone
                    .as_deref()
                    .map_or("first-help", |m| if m == "first_help" {
                        "first-help"
                    } else {
                        m
                    }),
                step.recipient
            ),
            10,
        )?;
    }
    let revision = rev + 1;
    save(tx, identity, revision, &out.state)?;
    let response = projection(tx, identity, &def, revision, Some(&out.state))?;
    let request = json!({"schemaVersion":"1.0","commandId":format!("learning-{}",crate::game::digest_id(command_id)),"expectedRevision":rev,"payload":{"type":"action","action_id":step.action_id}});
    tx.execute("INSERT INTO wonderwilds.campaign_command_receipts(profile_id,command_id,revision,request,response,digest) VALUES($1::text::uuid,$2,$3,$4,$5,$6)",&[&identity.profile_id,&request["commandId"].as_str().ok_or(Error::Unavailable)?,&(revision as i64),&request,&response,&crate::persistence::receipt(&request,&response,&json!({}))]).map_err(|_|Error::Unavailable)?;
    Ok(())
}

pub(crate) fn definition_version(version: &str) -> Result<CampaignDefinition, Error> {
    wonderwilds_content::campaign_version(version).map_err(|_| Error::Unavailable)
}
/// Only existing authored, nonquest collectibles can be shared as copies.
pub(crate) fn owned_keepsakes(
    tx: &mut Transaction<'_>,
    identity: &Identity,
) -> Result<std::collections::BTreeSet<String>, Error> {
    let (_, state) = load(tx, identity)?;
    Ok(state
        .map(|s| {
            s.items
                .into_iter()
                .filter(|(id, n)| *n > 0 && wonderwilds_domain::paired::whitelisted_keepsake(id))
                .map(|(id, _)| id)
                .collect()
        })
        .unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn cmd(id: &str, rev: u64, payload: Value) -> Value {
        json!({"schemaVersion":"1.0","commandId":id,"expectedRevision":rev,"payload":payload})
    }
    /// Upper-bound authored completion bookkeeping, including alternative choices.
    fn full_authored_state(def: &CampaignDefinition) -> CampaignState {
        let mut state = campaign::adopt(def);
        state.starter = def.starters.first().cloned();
        state.team = def
            .companions
            .iter()
            .take(2)
            .map(|c| c.id.clone())
            .collect();
        for action in &def.actions {
            state.flags.extend(
                action
                    .requires_flags
                    .iter()
                    .chain(&action.forbids_flags)
                    .chain(&action.grants_flags)
                    .cloned(),
            );
            state.grants.insert(format!("action:{}", action.id));
            if !action.grant_id.is_empty() {
                state.grants.insert(action.grant_id.clone());
            }
            for key in action.costs.keys().chain(action.rewards.keys()) {
                state.items.insert(key.clone(), 100);
            }
        }
        for encounter in &def.encounters {
            state.grants.insert(format!("win:{}", encounter.id));
            state.flags.extend(encounter.win_flags.iter().cloned());
        }
        for companion in &def.companions {
            state.energy.insert(companion.id.clone(), 5);
            for milestone in [
                "first_help",
                "field_partnership",
                "personal_project",
                "wayfinder",
            ] {
                state
                    .flags
                    .insert(format!("milestone:{}:{milestone}", companion.id));
            }
            for care in ["play", "groom", "quiet_rest", "favourite_place"] {
                state
                    .flags
                    .insert(format!("memory:care:{}:{care}", companion.id));
            }
            state
                .flags
                .insert(format!("technique:{}:free_signature", companion.id));
            state
                .flags
                .insert(format!("free_signature_used:{}", companion.id));
            state
                .flags
                .insert(format!("recruit_declined:{}", companion.id));
        }
        for location in &def.locations {
            state.flags.insert(format!("last_camp:{}", location.id));
        }
        state
    }
    fn full_projection(def: &CampaignDefinition, state: &CampaignState) -> Value {
        let owned = def
            .companions
            .iter()
            .map(|c| campaign::CompanionProjection {
                id: c.id.clone(),
                level: 5,
            })
            .collect::<Vec<_>>();
        let mut value = json!({"schemaVersion":"1.0","revision":1000,"adopted":true,"adoptionBlocked":false,"campaign":campaign::snapshot(def,state,&owned).unwrap(),"locations":def.locations.iter().map(|l|json!({"id":l.id,"name":l.name,"camp":l.camp})).collect::<Vec<_>>(),"starters":def.companions.iter().filter(|c|def.starters.contains(&c.id)).map(|c|json!({"id":c.id,"name":c.name,"affinity":c.affinity})).collect::<Vec<_>>()});
        for companion in value["campaign"]["companions"].as_array_mut().unwrap() {
            companion["appearance"] = json!("wayfinder");
            companion["affinity"] = json!(
                def.companions
                    .iter()
                    .find(|c| companion["id"] == c.id)
                    .unwrap()
                    .affinity
            );
        }
        value
    }
    #[test]
    fn complete_campaign_public_snapshots_fit_contract_and_receipt_budget() {
        let def = definition().unwrap();
        let mut state = full_authored_state(&def);
        assert!(state.flags.len() > 256);
        assert!(state.grants.len() >= def.actions.len());
        let mut maximum = 0;
        for location in &def.locations {
            state.location = location.id.clone();
            let value = full_projection(&def, &state);
            wonderwilds_contracts::validate("CampaignSnapshot", &value).unwrap();
            let bytes = value.to_string().len();
            maximum = maximum.max(bytes);
            assert!(bytes < 65_536, "{} needs {bytes} bytes", location.id);
        }
        eprintln!(
            "Full authored campaign: {} actions, {} milestones, {} flags, {} grants; max compact snapshot {maximum} bytes",
            def.actions.len(),
            def.companions.len() * 4,
            state.flags.len(),
            state.grants.len()
        );
        state.flags = (0..4096).map(|i| format!("finite-{i}")).collect();
        state.grants = state.flags.clone();
        wonderwilds_contracts::validate("CampaignSnapshot", &full_projection(&def, &state))
            .unwrap();
    }
    /// Explicit test-only synthetic UI setup. Never an HTTP or maintenance route.
    #[test]
    #[ignore = "guarded fresh disposable browser-QA Wayfinder fixture only"]
    fn postgres_seed_wayfinder_ui_fixture() {
        assert_eq!(
            std::env::var("WW_QA_ALLOW_WAYFINDER_FIXTURE").as_deref(),
            Ok("1")
        );
        let cluster = std::env::var("WW_QA_CLUSTER_DIR").expect("disposable QA wrapper required");
        assert!(
            std::path::Path::new(&cluster)
                .join("data/postmaster.pid")
                .is_file()
        );
        let expected = format!("host={cluster}/socket port=55432 user=ww_qa_admin dbname=ww_qa");
        let connection = std::env::var("WW_STORY_TEST_DATABASE_URL").unwrap();
        assert_eq!(connection, expected);
        assert_eq!(std::env::var("WW_QA_DATABASE_URL").unwrap(), expected);
        let profile =
            std::env::var("WW_QA_SEED_PROFILE_ID").expect("explicit synthetic profile required");
        let parent =
            std::env::var("WW_QA_SEED_PARENT_ID").expect("explicit synthetic parent required");
        let mut client = Client::connect(&connection, NoTls).unwrap();
        let mut tx = client.transaction().unwrap();
        let identity = Identity {
            profile_id: profile.clone(),
            parent_id: parent.clone(),
            session_id: "guarded-browser-fixture".into(),
        };
        own(&mut tx, &identity).unwrap();
        let issuer:String=tx.query_one("SELECT a.identity_issuer FROM wonderwilds.parents a JOIN wonderwilds.profiles p ON p.parent_id=a.id WHERE p.id=$1::text::uuid AND a.id=$2::text::uuid",&[&profile,&parent]).unwrap().get(0);
        assert_eq!(issuer, "synthetic-training-qa");
        assert!(
            tx.query_opt(
                "SELECT 1 FROM wonderwilds.campaign_progress WHERE profile_id=$1::text::uuid",
                &[&profile]
            )
            .unwrap()
            .is_none(),
            "fixture requires fresh campaign"
        );
        assert!(tx.query_opt("SELECT 1 FROM wonderwilds.mission_learning_progress WHERE profile_id=$1::text::uuid",&[&profile]).unwrap().is_none(),"fixture cannot replace learning");
        assert!(
            crate::game::campaign_companions(&mut tx, &identity)
                .unwrap()
                .is_empty(),
            "fixture cannot replace owned companions"
        );
        crate::game::campaign_grant(&mut tx, &identity, "pip", "campaign:recruit:pip", 0).unwrap();
        for milestone in ["first-help", "field_partnership", "personal_project"] {
            crate::game::campaign_grant(
                &mut tx,
                &identity,
                "pip",
                &format!("campaign:{milestone}:pip"),
                10,
            )
            .unwrap();
        }
        let def = definition().unwrap();
        let mut state = campaign::adopt(&def);
        state.starter = Some("pip".into());
        state.team = vec!["pip".into()];
        state.energy.insert("pip".into(), 5);
        state.flags.insert("starter_chosen".into());
        state.flags.insert("beacon_complete".into());
        for action in def.actions.iter().filter(|a| {
            a.recipient.as_deref() == Some("pip")
                && matches!(
                    a.milestone.as_deref(),
                    Some("first_help" | "field_partnership" | "personal_project")
                )
        }) {
            state.flags.extend(
                action
                    .requires_flags
                    .iter()
                    .chain(&action.grants_flags)
                    .cloned(),
            );
            state.grants.insert(format!("action:{}", action.id));
            if !action.grant_id.is_empty() {
                state.grants.insert(action.grant_id.clone());
            }
        }
        save(&mut tx, &identity, 1, &state).unwrap();
        let response = projection(&mut tx, &identity, &def, 1, Some(&state)).unwrap();
        assert_eq!(response["campaign"]["companions"][0]["level"], 4);
        assert!(
            response["campaign"]["legal_actions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|a| a["id"] == "train_pip_wayfinder")
        );
        let request = cmd("qa-wayfinder-fixture", 0, json!({"type":"adopt"}));
        tx.execute("INSERT INTO wonderwilds.campaign_command_receipts(profile_id,command_id,revision,request,response,digest) VALUES($1::text::uuid,'qa-wayfinder-fixture',1,$2,$3,$4)",&[&profile,&request,&response,&crate::persistence::receipt(&request,&response,&json!({}))]).unwrap();
        tx.commit().unwrap();
        println!(
            "Guarded synthetic Wayfinder UI fixture: canonical Pip level4, authored Wayfinder action ready, no academic evidence created."
        );
    }

    #[test]
    #[ignore = "requires disposable WW_STORY_TEST_DATABASE_URL"]
    fn postgres_campaign_pinned_initial_definition_and_learning_resume() {
        use crate::mission_learning::repository::Repository as LearningRepository;
        let connection = std::env::var("WW_STORY_TEST_DATABASE_URL").unwrap();
        let mut admin = Client::connect(&connection, NoTls).unwrap();
        admin.batch_execute("CREATE ROLE ww_pin_test LOGIN NOSUPERUSER NOBYPASSRLS;GRANT USAGE ON SCHEMA wonderwilds TO ww_pin_test;GRANT SELECT,INSERT,UPDATE,DELETE ON ALL TABLES IN SCHEMA wonderwilds TO ww_pin_test;GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA wonderwilds TO ww_pin_test;").unwrap();
        let parent = "93000000-0000-0000-0000-000000000001";
        let profile = "94000000-0000-0000-0000-000000000001";
        admin.execute("INSERT INTO wonderwilds.parents(id,identity_issuer,identity_subject) VALUES($1::text::uuid,'synthetic-pinned','pin')",&[&parent]).unwrap();
        admin.execute("INSERT INTO wonderwilds.profiles(id,parent_id,ability_band) VALUES($1::text::uuid,$2::text::uuid,'beginning')",&[&profile,&parent]).unwrap();
        let config: postgres::Config = connection.parse().unwrap();
        let host = match &config.get_hosts()[0] {
            postgres::config::Host::Unix(path) => path.to_str().unwrap(),
            _ => panic!(),
        };
        let restricted = format!(
            "host={host} port={} user=ww_pin_test dbname={}",
            config.get_ports()[0],
            config.get_dbname().unwrap()
        );
        let identity = Identity {
            parent_id: parent.into(),
            profile_id: profile.into(),
            session_id: "synthetic-pin".into(),
        };
        let old = definition_version("1.0.0").unwrap();
        assert_eq!(old.actions.len(), 260);
        assert_eq!(definition().unwrap().version, "1.1.0");
        let state = campaign::adopt(&old);
        let mut client = Client::connect(&restricted, NoTls).unwrap();
        let mut tx = client.transaction().unwrap();
        own(&mut tx, &identity).unwrap();
        save(&mut tx, &identity, 1, &state).unwrap();
        let response = projection(&mut tx, &identity, &old, 1, Some(&state)).unwrap();
        let request = cmd("pinned-adopt", 0, json!({"type":"adopt"}));
        tx.execute("INSERT INTO wonderwilds.campaign_command_receipts(profile_id,command_id,revision,request,response,digest) VALUES($1::text::uuid,'pinned-adopt',1,$2,$3,$4)",&[&profile,&request,&response,&crate::persistence::receipt(&request,&response,&json!({}))]).unwrap();
        tx.commit().unwrap();
        let repo = Repository::new(Some(restricted.clone()));
        assert_eq!(repo.run(&identity, None).unwrap(), response);
        let chosen = repo
            .run(
                &identity,
                Some(&cmd(
                    "pinned-starter",
                    1,
                    json!({"type":"choose_starter","companion_id":"pip"}),
                )),
            )
            .unwrap();
        assert_eq!(chosen["campaign"]["state"]["content_version"], "1.0.0");
        let met = repo
            .run(
                &identity,
                Some(&cmd(
                    "pinned-welcome",
                    2,
                    json!({"type":"action","action_id":"welcome_meet"}),
                )),
            )
            .unwrap();
        let learning =
            crate::mission_learning::repository::PostgresRepository::new(restricted.clone());
        let begun=learning.execute(&identity,&cmd("pinned-begin",0,json!({"type":"begin_campaign_step","actionId":"welcome_supplies","campaignRevision":met["revision"]}))).unwrap();
        // Reopen adapters as after an API restart with the newer default pack.
        let restarted =
            crate::mission_learning::repository::PostgresRepository::new(restricted.clone());
        let done=restarted.execute(&identity,&cmd("pinned-finish",begun["revision"].as_u64().unwrap(),json!({"type":"finish_together","encounterId":begun["activeActivity"]["id"],"variantId":begun["activeActivity"]["variantId"]}))).unwrap();
        assert!(done.get("activeActivity").is_none());
        let final_state = Repository::new(Some(restricted))
            .run(&identity, None)
            .unwrap();
        assert_eq!(final_state["campaign"]["state"]["content_version"], "1.0.0");
        assert!(
            final_state["campaign"]["state"]["flags"]
                .as_array()
                .unwrap()
                .contains(&json!("welcome_supplies_done"))
        );
    }
    #[test]
    #[ignore = "requires disposable migrated WW_STORY_TEST_DATABASE_URL"]
    fn postgres_campaign_adoption_retry_race_canonical_learning_and_lifecycle() {
        let connection = std::env::var("WW_STORY_TEST_DATABASE_URL").unwrap();
        let mut admin = Client::connect(&connection, NoTls).unwrap();
        admin.batch_execute("CREATE ROLE ww_campaign_test LOGIN NOSUPERUSER NOBYPASSRLS;GRANT USAGE ON SCHEMA wonderwilds TO ww_campaign_test;GRANT SELECT,INSERT,UPDATE,DELETE ON ALL TABLES IN SCHEMA wonderwilds TO ww_campaign_test;GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA wonderwilds TO ww_campaign_test;").unwrap();
        let parent = "81000000-0000-0000-0000-000000000001";
        let profile = "82000000-0000-0000-0000-000000000001";
        admin.execute("INSERT INTO wonderwilds.parents(id,identity_issuer,identity_subject) VALUES($1::text::uuid,'synthetic-campaign','campaign')",&[&parent]).unwrap();
        admin.execute("INSERT INTO wonderwilds.profiles(id,parent_id,ability_band) VALUES($1::text::uuid,$2::text::uuid,'beginning')",&[&profile,&parent]).unwrap();
        let mut config: postgres::Config = connection.parse().unwrap();
        config.user("ww_campaign_test");
        let host = match &config.get_hosts()[0] {
            postgres::config::Host::Unix(path) => path.to_str().unwrap(),
            _ => panic!(),
        };
        let restricted = format!(
            "host={host} port={} user=ww_campaign_test dbname={}",
            config.get_ports()[0],
            config.get_dbname().unwrap()
        );
        let repo = Repository::new(Some(restricted.clone()));
        let identity = Identity {
            parent_id: parent.into(),
            profile_id: profile.into(),
            session_id: "synthetic-campaign".into(),
        };
        // Real app startup warms immutable caches before concurrent cold reads.
        let _app = crate::game::http::AppState::development(
            Some(restricted.clone()),
            "https://wonderwilds.local".into(),
        )
        .unwrap();
        std::thread::scope(|scope| {
            let campaign = scope.spawn(|| repo.run(&identity, None));
            let story = scope.spawn(|| {
                crate::story::repository::Repository::read(
                    &crate::story::repository::PostgresRepository::new(restricted.clone()),
                    &identity,
                )
            });
            let learning = scope.spawn(|| {
                crate::mission_learning::repository::Repository::read(
                    &crate::mission_learning::repository::PostgresRepository::new(
                        restricted.clone(),
                    ),
                    &identity,
                )
            });
            assert!(campaign.join().unwrap().is_ok());
            assert!(story.join().unwrap().is_ok());
            assert!(learning.join().unwrap().is_ok());
        });
        let initial = repo.run(&identity, None).unwrap();
        assert_eq!(initial["adopted"], false);
        let adopt = cmd("adopt", 0, json!({"type":"adopt"}));
        let first = repo.run(&identity, Some(&adopt)).unwrap();
        assert_eq!(first["revision"], 1);
        assert_eq!(repo.run(&identity, Some(&adopt)).unwrap(), first);
        assert_eq!(
            repo.run(&identity, Some(&cmd("stale", 0, json!({"type":"adopt"})))),
            Err(Error::RevisionConflict)
        );
        let starter = cmd(
            "starter",
            1,
            json!({"type":"choose_starter","companion_id":"pip"}),
        );
        let mut faulty = repo.clone();
        faulty.fail_before_commit = true;
        assert_eq!(
            faulty.run(&identity, Some(&starter)),
            Err(Error::Unavailable)
        );
        assert_eq!(repo.run(&identity, None).unwrap(), first);
        let threads = (0..2)
            .map(|_| {
                let r = repo.clone();
                let i = identity.clone();
                let c = starter.clone();
                std::thread::spawn(move || r.run(&i, Some(&c)))
            })
            .collect::<Vec<_>>();
        for thread in threads {
            assert_eq!(thread.join().unwrap().unwrap()["revision"], 2)
        }
        let chosen = repo.run(&identity, None).unwrap();
        assert_eq!(chosen["campaign"]["state"]["starter"], "pip");
        assert_eq!(
            chosen["campaign"]["companions"].as_array().unwrap().len(),
            1
        );
        let game = crate::game::repository::PostgresRepository::new(
            restricted.clone(),
            crate::game::development_content().unwrap(),
        );
        assert!(game.verify_history(&identity).is_ok());
        assert_eq!(repo.run(&identity, Some(&adopt)).unwrap(), first);
        let mut other = identity.clone();
        other.parent_id = "81000000-0000-0000-0000-000000000099".into();
        assert_eq!(repo.run(&other, None), Err(Error::Forbidden));
        let mut restricted_client = Client::connect(&restricted, NoTls).unwrap();
        restricted_client
            .execute(
                "SELECT set_config('app.parent_id',$1,false)",
                &[&other.parent_id],
            )
            .unwrap();
        assert_eq!(
            restricted_client
                .query_one("SELECT count(*) FROM wonderwilds.campaign_progress", &[])
                .unwrap()
                .get::<_, i64>(0),
            0
        );
        assert!(restricted_client.execute("INSERT INTO wonderwilds.campaign_progress(profile_id,revision,checkpoint) VALUES($1::text::uuid,1,'{}'::jsonb)",&[&profile]).is_err());
        admin
            .execute("SELECT set_config('app.parent_id',$1,false)", &[&parent])
            .unwrap();
        let exported: Value = admin
            .query_one(
                "SELECT wonderwilds.export_profile($1::text::uuid)",
                &[&profile],
            )
            .unwrap()
            .get(0);
        assert_eq!(exported["campaignReceipts"].as_array().unwrap().len(), 2);
        // A failure after canonical grant creation must roll back every table.
        let mut client = Client::connect(&restricted, NoTls).unwrap();
        let mut tx = client.transaction().unwrap();
        own(&mut tx, &identity).unwrap();
        crate::game::campaign_grant(&mut tx, &identity, "tansy", "campaign:recruit:tansy", 0)
            .unwrap();
        drop(tx);
        assert_eq!(
            repo.run(&identity, None).unwrap()["campaign"]["companions"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        // Joined curriculum completion saves campaign effects and evidence together.
        use crate::mission_learning::repository::Repository as LearningRepository;
        let learning =
            crate::mission_learning::repository::PostgresRepository::new(restricted.clone());
        let mut learning_completions = 0;
        let mut tested_recipient = false;
        for index in 0..100 {
            let current = repo.run(&identity, None).unwrap();
            if current["campaign"]["state"]["encounter"].is_object() {
                repo.run(&identity,Some(&cmd(&format!("guided-{index}"),current["revision"].as_u64().unwrap(),json!({"type":"encounter_turn","companion_id":current["campaign"]["state"]["team"][0],"movement":"guided"})))).unwrap();
                continue;
            }
            let actions = current["campaign"]["legal_actions"].as_array().unwrap();
            let Some(action) = actions.iter().find(|a| {
                a["id"] != "bridge_trial"
                    && !current["campaign"]["state"]["grants"]
                        .as_array()
                        .unwrap()
                        .contains(&json!(format!("action:{}", a["id"].as_str().unwrap())))
            }) else {
                break;
            };
            let action_id = action["id"].as_str().unwrap();
            if action["learning"] == true {
                let prior = learning.read(&identity).unwrap();
                let begin = cmd(
                    &format!("campaign-begin-{index}"),
                    prior["revision"].as_u64().unwrap(),
                    json!({"type":"begin_campaign_step","actionId":action_id,"campaignRevision":current["revision"]}),
                );
                let active = learning.execute(&identity, &begin).unwrap();
                let pin_test = action["kind"] == "training" && !tested_recipient;
                if pin_test {
                    let mut client = Client::connect(&restricted, NoTls).unwrap();
                    let mut tx = client.transaction().unwrap();
                    own(&mut tx, &identity).unwrap();
                    crate::game::campaign_grant(
                        &mut tx,
                        &identity,
                        "tansy",
                        "campaign:recruit:tansy",
                        0,
                    )
                    .unwrap();
                    tx.commit().unwrap();
                    repo.run(
                        &identity,
                        Some(&cmd(
                            "switch-bound-team",
                            current["revision"].as_u64().unwrap(),
                            json!({"type":"set_team","companion_ids":["tansy"]}),
                        )),
                    )
                    .unwrap();
                }
                let finish = cmd(
                    &format!("campaign-finish-{index}"),
                    active["revision"].as_u64().unwrap(),
                    json!({"type":"finish_together","encounterId":active["activeActivity"]["id"],"variantId":active["activeActivity"]["variantId"]}),
                );
                let done = learning.execute(&identity, &finish).unwrap();
                // Consumers wait for the write response before refreshing. All
                // concurrent refreshes must then see the joined committed state.
                std::thread::scope(|scope| {
                    let a = scope.spawn(|| repo.run(&identity, None));
                    let b = scope.spawn(|| repo.run(&identity, None));
                    let l = scope.spawn(|| learning.read(&identity));
                    assert_eq!(a.join().unwrap().unwrap(), b.join().unwrap().unwrap());
                    assert_eq!(l.join().unwrap().unwrap(), done);
                });
                assert_eq!(learning.execute(&identity, &finish).unwrap(), done);
                assert!(done.get("activeActivity").is_none());
                learning_completions += 1;
                if pin_test {
                    tested_recipient = true;
                    let after = repo.run(&identity, None).unwrap();
                    assert!(
                        after["campaign"]["companions"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|c| c["id"] == "pip" && c["level"] == 2)
                    );
                    assert!(
                        after["campaign"]["companions"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|c| c["id"] == "tansy" && c["level"] == 1)
                    );
                    assert!(
                        after["campaign"]["state"]["flags"]
                            .as_array()
                            .unwrap()
                            .contains(&json!("milestone:pip:first_help"))
                    );
                    break;
                }
            } else {
                repo.run(
                    &identity,
                    Some(&cmd(
                        &format!("campaign-action-{index}"),
                        current["revision"].as_u64().unwrap(),
                        json!({"type":"action","action_id":action_id}),
                    )),
                )
                .unwrap();
            }
        }
        // Existing companions, active legacy stories and pending activities survive adoption.
        let legacy_profile = "82000000-0000-0000-0000-000000000002";
        admin.execute("INSERT INTO wonderwilds.profiles(id,parent_id,ability_band) VALUES($1::text::uuid,$2::text::uuid,'beginning')",&[&legacy_profile,&parent]).unwrap();
        let legacy = Identity {
            profile_id: legacy_profile.into(),
            ..identity.clone()
        };
        let mut client = Client::connect(&restricted, NoTls).unwrap();
        let mut tx = client.transaction().unwrap();
        own(&mut tx, &legacy).unwrap();
        crate::game::campaign_grant(&mut tx, &legacy, "pip", "campaign:recruit:pip", 0).unwrap();
        tx.commit().unwrap();
        use crate::story::repository::Repository as StoryRepository;
        let story = crate::story::repository::PostgresRepository::new(restricted.clone());
        let original_story = story.read(&legacy).unwrap();
        let accept = original_story["sceneInput"]["availableActions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["action"]["type"] == "accept_mission")
            .unwrap();
        let saved_story = story
            .execute(
                &legacy,
                &cmd(
                    "legacy-accept",
                    0,
                    json!({"type":"choose","actionId":accept["id"]}),
                ),
            )
            .unwrap();
        let begun = learning
            .execute(&legacy, &cmd("legacy-begin", 0, json!({"type":"begin"})))
            .unwrap();
        assert_eq!(repo.run(&legacy, None).unwrap()["adoptionBlocked"], true);
        assert_eq!(
            repo.run(
                &legacy,
                Some(&cmd("defer-adopt", 0, json!({"type":"adopt"})))
            ),
            Err(Error::RevisionConflict)
        );
        learning
            .execute(
                &legacy,
                &cmd(
                    "legacy-abandon",
                    1,
                    json!({"type":"abandon","encounterId":begun["activeActivity"]["id"]}),
                ),
            )
            .unwrap();
        repo.run(
            &legacy,
            Some(&cmd("legacy-adopt", 0, json!({"type":"adopt"}))),
        )
        .unwrap();
        let legacy_chosen = repo
            .run(
                &legacy,
                Some(&cmd(
                    "legacy-starter",
                    1,
                    json!({"type":"choose_starter","companion_id":"pip"}),
                )),
            )
            .unwrap();
        assert_eq!(
            legacy_chosen["campaign"]["companions"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(story.read(&legacy).unwrap(), saved_story);
        assert_eq!(admin.query_one("SELECT count(*) FROM wonderwilds.command_receipts WHERE profile_id=$1::text::uuid",&[&legacy_profile]).unwrap().get::<_,i64>(0),1);
        assert!(learning_completions > 0);
        assert!(tested_recipient);
        let mut client = Client::connect(&restricted, NoTls).unwrap();
        let mut tx = client.transaction().unwrap();
        own(&mut tx, &identity).unwrap();
        for milestone in ["field_partnership", "personal_project", "wayfinder"] {
            crate::game::campaign_grant(
                &mut tx,
                &identity,
                "pip",
                &format!("campaign:{milestone}:pip"),
                10,
            )
            .unwrap();
        }
        tx.commit().unwrap();
        for (index, kind) in ["appearance_wayfinder", "appearance_base"]
            .iter()
            .enumerate()
        {
            let current = repo.run(&identity, None).unwrap();
            let result = repo
                .run(
                    &identity,
                    Some(&cmd(
                        &format!("form-{index}"),
                        current["revision"].as_u64().unwrap(),
                        json!({"type":"care","companion_id":"pip","kind":kind}),
                    )),
                )
                .unwrap();
            assert!(
                result["campaign"]["companions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|c| c["id"] == "pip"
                        && c["appearance"] == if index == 0 { "wayfinder" } else { "base" })
            );
        }
        assert!(game.verify_history(&identity).is_ok());
        // Verify actual JSONB storage after all canonical companion grants.
        let mut sizing_client = Client::connect(&restricted, NoTls).unwrap();
        let mut sizing = sizing_client.transaction().unwrap();
        own(&mut sizing, &identity).unwrap();
        let definition = definition().unwrap();
        for companion in &definition.companions {
            crate::game::campaign_grant(
                &mut sizing,
                &identity,
                &companion.id,
                &format!("campaign:recruit:{}", companion.id),
                0,
            )
            .unwrap();
            for milestone in [
                "first-help",
                "field_partnership",
                "personal_project",
                "wayfinder",
            ] {
                crate::game::campaign_grant(
                    &mut sizing,
                    &identity,
                    &companion.id,
                    &format!("campaign:{milestone}:{}", companion.id),
                    10,
                )
                .unwrap();
            }
        }
        let canonical = crate::game::campaign_companions(&mut sizing, &identity).unwrap();
        assert_eq!(canonical.len(), 12);
        assert!(canonical.iter().all(|c| c.level == 5));
        let mut completed = full_authored_state(&definition);
        let mut max_bytes = 0;
        let mut largest = Value::Null;
        for location in &definition.locations {
            completed.location = location.id.clone();
            let response =
                projection(&mut sizing, &identity, &definition, 1000, Some(&completed)).unwrap();
            let bytes: i32 = sizing
                .query_one("SELECT octet_length($1::jsonb::text)", &[&response])
                .unwrap()
                .get(0);
            assert!(
                bytes < 65_536,
                "{} JSONB response needs {bytes} bytes",
                location.id
            );
            if bytes > max_bytes {
                max_bytes = bytes;
                largest = response;
            }
        }
        let synthetic_request = cmd("sizing-complete-campaign", 999, json!({"type":"camp"}));
        sizing.execute("INSERT INTO wonderwilds.campaign_command_receipts(profile_id,command_id,revision,request,response,digest) VALUES($1::text::uuid,'sizing-complete-campaign',1000,$2,$3,$4)",&[&identity.profile_id,&synthetic_request,&largest,&crate::persistence::receipt(&synthetic_request,&largest,&json!({}))]).unwrap();
        eprintln!(
            "Full authored campaign with 12 canonical recruit + 48 milestone receipts: maximum PostgreSQL JSONB snapshot {max_bytes} bytes"
        );
        sizing.rollback().unwrap();
        let checkpoint:Value=admin.query_one("SELECT checkpoint FROM wonderwilds.campaign_progress WHERE profile_id=$1::text::uuid",&[&profile]).unwrap().get(0);
        let mut unsupported = checkpoint.clone();
        unsupported["format"] = json!("campaign-storage-unknown");
        admin.execute("UPDATE wonderwilds.campaign_progress SET checkpoint=$2 WHERE profile_id=$1::text::uuid",&[&profile,&unsupported]).unwrap();
        assert_eq!(repo.run(&identity, None), Err(Error::Unavailable));
        admin.execute("UPDATE wonderwilds.campaign_progress SET checkpoint=$2 WHERE profile_id=$1::text::uuid",&[&profile,&checkpoint]).unwrap();

        admin
            .execute(
                "DELETE FROM wonderwilds.profiles WHERE id=$1::text::uuid",
                &[&profile],
            )
            .unwrap();
        for table in ["campaign_progress", "campaign_command_receipts"] {
            assert_eq!(admin.query_one(&format!("SELECT count(*) FROM wonderwilds.{table} WHERE profile_id=$1::text::uuid"),&[&profile]).unwrap().get::<_,i64>(0),0)
        }
    }
}
