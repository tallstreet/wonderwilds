use crate::game::{Error, Identity};
use postgres::{Client, NoTls, Transaction};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use wonderwilds_domain::paired::{self, State};
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
    fn connect(&self) -> Result<Client, Error> {
        Client::connect(self.connection.as_deref().ok_or(Error::Unavailable)?, NoTls)
            .map_err(|_| Error::Unavailable)
    }
    pub fn readiness(&self) -> Result<(), Error> {
        let mut client = self.connect()?;
        let ready:bool=client.query_one("SELECT to_regclass('wonderwilds.paired_progress') IS NOT NULL AND to_regclass('wonderwilds.paired_command_receipts') IS NOT NULL AND to_regclass('wonderwilds.paired_keepsakes') IS NOT NULL AND to_regclass('wonderwilds.paired_household_clock') IS NOT NULL",&[]).map_err(|_|Error::Unavailable)?.get(0);
        if ready {
            Ok(())
        } else {
            Err(Error::Unavailable)
        }
    }
    pub fn profiles(&self, identity: &Identity) -> Result<Value, Error> {
        let mut client = self.connect()?;
        let mut tx = client.transaction().map_err(|_| Error::Unavailable)?;
        context(&mut tx, identity)?;
        let profiles=tx.query("SELECT id::text,ability_band FROM wonderwilds.profiles WHERE parent_id=$1::text::uuid ORDER BY created_at,id",&[&identity.parent_id]).map_err(|_|Error::Unavailable)?.iter().map(|r|json!({"id":r.get::<_,String>(0),"abilityBand":r.get::<_,String>(1)})).collect::<Vec<_>>();
        if !profiles.iter().any(|p| p["id"] == identity.profile_id) {
            return Err(Error::Forbidden);
        }
        let value = json!({"schemaVersion":"1.0","profiles":profiles});
        wonderwilds_contracts::validate("HouseholdProfilesSnapshot", &value)
            .map_err(|_| Error::Unavailable)?;
        tx.commit().map_err(|_| Error::Unavailable)?;
        Ok(value)
    }
    pub fn run(&self, identity: &Identity, request: Option<&Value>) -> Result<Value, Error> {
        if let Some(r) = request {
            wonderwilds_contracts::validate("PairedCommand", r).map_err(|_| Error::InvalidInput)?;
        }
        let mut client = self.connect()?;
        let mut tx = client.transaction().map_err(|_| Error::Unavailable)?;
        context(&mut tx, identity)?;
        // One household coordinator, then participant profile locks in lexical order.
        if tx
            .query_opt(
                "SELECT id FROM wonderwilds.parents WHERE id=$1::text::uuid FOR UPDATE",
                &[&identity.parent_id],
            )
            .map_err(|_| Error::Unavailable)?
            .is_none()
        {
            return Err(Error::Forbidden);
        }
        tx.execute("INSERT INTO wonderwilds.paired_household_clock(parent_id,revision) VALUES($1::text::uuid,0) ON CONFLICT(parent_id) DO NOTHING",&[&identity.parent_id]).map_err(|_|Error::Unavailable)?;
        let household_revision=tx.query_one("SELECT revision FROM wonderwilds.paired_household_clock WHERE parent_id=$1::text::uuid FOR UPDATE",&[&identity.parent_id]).map_err(|_|Error::Unavailable)?.get::<_,i64>(0) as u64;
        let row=tx.query_opt("SELECT revision,checkpoint,navigator_profile_id::text,keeper_profile_id::text FROM wonderwilds.paired_progress WHERE parent_id=$1::text::uuid",&[&identity.parent_id]).map_err(|_|Error::Unavailable)?;
        let has_progress = row.is_some();
        let (mut revision, mut state): (u64, Option<State>) = match row {
            None => (household_revision, None),
            Some(row) => {
                if row.get::<_, i64>(0) as u64 != household_revision {
                    return Err(Error::Unavailable);
                }
                let stored: Value = row.get(1);
                let value = crate::persistence::unwrap(&stored, "paired-storage-1")?;
                (row.get::<_, i64>(0) as u64, {
                    if value["rulesVersion"] != paired::RULES_VERSION {
                        return Err(Error::Unavailable);
                    }
                    let state: Option<State> = serde_json::from_value(value["pair"].clone())
                        .map_err(|_| Error::Unavailable)?;
                    let navigator: Option<String> =
                        serde_json::from_value(value["navigatorProfileId"].clone())
                            .map_err(|_| Error::Unavailable)?;
                    let keeper: Option<String> =
                        serde_json::from_value(value["keeperProfileId"].clone())
                            .map_err(|_| Error::Unavailable)?;
                    if row.get::<_, Option<String>>(2) != navigator
                        || row.get::<_, Option<String>>(3) != keeper
                    {
                        return Err(Error::Unavailable);
                    }
                    if let Some(s) = &state {
                        paired::validate(s).map_err(|_| Error::Unavailable)?;
                        if navigator.as_ref() != Some(&s.navigator_profile_id)
                            || keeper.as_ref() != Some(&s.keeper_profile_id)
                        {
                            return Err(Error::Unavailable);
                        }
                    }
                    state
                })
            }
        };
        let mut participants = BTreeSet::from([identity.profile_id.clone()]);
        if let Some(s) = &state {
            participants.extend(s.participants().into_iter().map(str::to_owned));
        }
        let parsed = request
            .map(|r| {
                serde_json::from_value::<paired::Command>(r["payload"].clone())
                    .map_err(|_| Error::InvalidInput)
            })
            .transpose()?;
        if let Some(paired::Command::Start { partner_profile_id }) = &parsed {
            participants.insert(partner_profile_id.clone());
        }
        for id in &participants {
            if tx.query_opt("SELECT id FROM wonderwilds.profiles WHERE id=$1::text::uuid AND parent_id=$2::text::uuid FOR UPDATE",&[id,&identity.parent_id]).map_err(|_|Error::Unavailable)?.is_none(){return Err(Error::Forbidden)}
        }
        if let Some(r)=request&&let Some(row)=tx.query_opt("SELECT actor_profile_id::text,request,response,digest,domain_state FROM wonderwilds.paired_command_receipts WHERE parent_id=$1::text::uuid AND command_id=$2",&[&identity.parent_id,&r["commandId"].as_str().ok_or(Error::InvalidInput)?]).map_err(|_|Error::Unavailable)?{if row.get::<_,String>(0)!=identity.profile_id||row.get::<_,Value>(1)!=*r{return Err(Error::RevisionConflict)}let response:Value=row.get(2);if row.get::<_,String>(3)!=crate::persistence::receipt(r,&response,&json!({"rulesVersion":paired::RULES_VERSION,"actor":identity.profile_id,"state":row.get::<_,Value>(4)})){return Err(Error::Unavailable)}wonderwilds_contracts::validate("PairedSnapshot",&response).map_err(|_|Error::Unavailable)?;tx.commit().map_err(|_|Error::Unavailable)?;return Ok(response)}
        if revision > 0 && has_progress {
            let row=tx.query_opt("SELECT revision,request,response,digest,actor_profile_id::text,domain_state FROM wonderwilds.paired_command_receipts WHERE parent_id=$1::text::uuid ORDER BY revision DESC LIMIT 1",&[&identity.parent_id]).map_err(|_|Error::Unavailable)?;
            // Older solo records may outlive deleted participant receipts. The
            // durable household clock, rather than child-linked receipts, owns
            // sequencing. Only an empty coordinator can recover this way.
            if let Some(row) =
                row.filter(|row| state.is_some() || row.get::<_, i64>(0) as u64 >= revision)
            {
                let stored: Value = row.get(5);
                let actor: String = row.get(4);
                if row.get::<_, i64>(0) as u64 != revision
                    || stored != json!(state)
                    || row.get::<_, String>(3)
                        != crate::persistence::receipt(
                            &row.get(1),
                            &row.get(2),
                            &json!({"rulesVersion":paired::RULES_VERSION,"actor":actor,"state":stored}),
                        )
                {
                    return Err(Error::Unavailable);
                }
            } else if state.is_some() {
                return Err(Error::Unavailable);
            }
        }
        if let (Some(r), Some(command)) = (request, parsed.as_ref()) {
            if r["expectedRevision"].as_u64() != Some(revision) {
                return Err(Error::RevisionConflict);
            }
            crate::persistence::take_budget(&mut tx, &identity.profile_id, "paired")?;
            let owned = crate::game::campaign_companions(&mut tx, identity)?
                .into_iter()
                .map(|c| c.id)
                .collect();
            let keepsakes = crate::campaign::repository::owned_keepsakes(&mut tx, identity)?;
            // Freeze both original offers at final confirmation; no resource is debited.
            if matches!(command, paired::Command::ConfirmExchange)
                && let Some(s) = &state
            {
                for (profile, item) in &s.offers {
                    let source = Identity {
                        profile_id: profile.clone(),
                        ..identity.clone()
                    };
                    if !crate::campaign::repository::owned_keepsakes(&mut tx, &source)?
                        .contains(item)
                    {
                        return Err(Error::RevisionConflict);
                    }
                }
            }
            let before = state.clone();
            let applied = paired::apply(
                state.as_ref(),
                &identity.profile_id,
                r["commandId"].as_str().ok_or(Error::InvalidInput)?,
                command,
                &owned,
                &keepsakes,
            )
            .map_err(|e| match e {
                paired::Error::Forbidden => Error::Forbidden,
                paired::Error::Conflict => Error::RevisionConflict,
                paired::Error::Invalid => Error::InvalidInput,
            })?;
            for copy in applied.copies {
                tx.execute("INSERT INTO wonderwilds.paired_keepsakes(profile_id,source_profile_id,keepsake_id) VALUES($1::text::uuid,$2::text::uuid,$3) ON CONFLICT(profile_id,source_profile_id,keepsake_id) DO NOTHING",&[&copy.recipient,&copy.source,&copy.keepsake_id]).map_err(|_|Error::Unavailable)?;
            }
            state = applied.state;
            revision = revision
                .checked_add(1)
                .filter(|r| *r <= 1_000_000_000)
                .ok_or(Error::RateLimited)?;
            if tx.execute("UPDATE wonderwilds.paired_household_clock SET revision=$2 WHERE parent_id=$1::text::uuid AND revision=$3",&[&identity.parent_id,&(revision as i64),&(household_revision as i64)]).map_err(|_|Error::Unavailable)?!=1{return Err(Error::Unavailable)}
            let ids = state
                .as_ref()
                .or(before.as_ref())
                .map(|s| {
                    (
                        Some(s.navigator_profile_id.clone()),
                        Some(s.keeper_profile_id.clone()),
                    )
                })
                .unwrap_or((None, None));
            let checkpoint = crate::persistence::envelope(
                "paired-storage-1",
                json!({"rulesVersion":paired::RULES_VERSION,"pair":state,"navigatorProfileId":ids.0,"keeperProfileId":ids.1}),
            );
            tx.execute("INSERT INTO wonderwilds.paired_progress(parent_id,revision,checkpoint,navigator_profile_id,keeper_profile_id) VALUES($1::text::uuid,$2,$3,$4::text::uuid,$5::text::uuid) ON CONFLICT(parent_id) DO UPDATE SET revision=EXCLUDED.revision,checkpoint=EXCLUDED.checkpoint,navigator_profile_id=EXCLUDED.navigator_profile_id,keeper_profile_id=EXCLUDED.keeper_profile_id",&[&identity.parent_id,&(revision as i64),&checkpoint,&ids.0,&ids.1]).map_err(|_|Error::Unavailable)?;
            let response = snapshot(&mut tx, identity, revision, state.as_ref())?;
            tx.execute("INSERT INTO wonderwilds.paired_command_receipts(parent_id,command_id,revision,actor_profile_id,navigator_profile_id,keeper_profile_id,request,response,digest,domain_state) VALUES($1::text::uuid,$2,$3,$4::text::uuid,$5::text::uuid,$6::text::uuid,$7,$8,$9,$10)",&[&identity.parent_id,&r["commandId"].as_str().ok_or(Error::InvalidInput)?,&(revision as i64),&identity.profile_id,&ids.0,&ids.1,r,&response,&crate::persistence::receipt(r,&response,&json!({"rulesVersion":paired::RULES_VERSION,"actor":identity.profile_id,"state":state})),&json!(state)]).map_err(|_|Error::Unavailable)?;
            #[cfg(test)]
            if self.fail_before_commit {
                return Err(Error::Unavailable);
            }
            tx.commit().map_err(|_| Error::Unavailable)?;
            return Ok(response);
        }
        let response = snapshot(&mut tx, identity, revision, state.as_ref())?;
        tx.commit().map_err(|_| Error::Unavailable)?;
        Ok(response)
    }
}
fn context(tx: &mut Transaction<'_>, identity: &Identity) -> Result<(), Error> {
    tx.batch_execute("SET LOCAL statement_timeout='2000ms';SET LOCAL lock_timeout='1000ms';")
        .map_err(|_| Error::Unavailable)?;
    tx.query_one(
        "SELECT set_config('app.parent_id',$1,true)",
        &[&identity.parent_id],
    )
    .map_err(|_| Error::Unavailable)?;
    Ok(())
}
fn snapshot(
    tx: &mut Transaction<'_>,
    identity: &Identity,
    revision: u64,
    state: Option<&State>,
) -> Result<Value, Error> {
    let mut value = json!({"schemaVersion":"1.0","revision":revision,"actorProfileId":identity.profile_id,"mode":if state.is_some(){"paired"}else{"solo"},"offers":[],"confirmedProfileIds":[],"exchangeComplete":false,"companions":[],"availableCompanions":crate::game::campaign_companions(tx,identity)?,"availableKeepsakeIds":crate::campaign::repository::owned_keepsakes(tx,identity)?,"completedPractices":0,"legalMovements":[],"practiceTarget":6,"practicePressureLimit":5,"practiceTurnLimit":8,"keepsakeCount":tx.query_one("SELECT count(*) FROM wonderwilds.paired_keepsakes WHERE profile_id=$1::text::uuid",&[&identity.profile_id]).map_err(|_|Error::Unavailable)?.get::<_,i64>(0),"keepsakes":tx.query("SELECT keepsake_id,source_profile_id::text FROM wonderwilds.paired_keepsakes WHERE profile_id=$1::text::uuid ORDER BY recorded_at DESC,id DESC LIMIT 128",&[&identity.profile_id]).map_err(|_|Error::Unavailable)?.iter().map(|r|{let mut k=json!({"keepsakeId":r.get::<_,String>(0)});if let Some(source)=r.get::<_,Option<String>>(1){k["sourceProfileId"]=json!(source)}k}).collect::<Vec<_>>()});
    if let Some(s) = state {
        value["navigatorProfileId"] = json!(s.navigator_profile_id);
        value["keeperProfileId"] = json!(s.keeper_profile_id);
        value["offers"] = json!(
            s.offers
                .iter()
                .map(|(profile, item)| json!({"profileId":profile,"keepsakeId":item}))
                .collect::<Vec<_>>()
        );
        value["confirmedProfileIds"] = json!(s.confirmed_profile_ids);
        value["exchangeComplete"] = json!(s.exchange_complete);
        value["companions"] = json!(
            s.companions
                .iter()
                .map(|(profile, friend)| json!({"profileId":profile,"companionId":friend}))
                .collect::<Vec<_>>()
        );
        value["completedPractices"] = json!(s.completed_practices);
        if let Some(p) = &s.practice {
            value["practice"] = json!(p);
            if p.outcome == paired::Outcome::Active
                && s.role(&identity.profile_id) == Some(p.next_role)
            {
                value["legalMovements"] = if p.next_role == paired::Role::Navigator {
                    json!(["scout", "rest"])
                } else {
                    json!(["brace", "rally"])
                }
            }
        }
    }
    wonderwilds_contracts::validate("PairedSnapshot", &value).map_err(|_| Error::Unavailable)?;
    Ok(value)
}

#[cfg(test)]
mod tests;
