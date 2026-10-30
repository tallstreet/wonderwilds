//! Read-only, draft curriculum graph bundled with the learning engine.
//! The data remains pending review; loading it never activates content or grants mastery.
use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillNode {
    pub id: String,
    pub subject: String,
    pub objective: String,
    pub band: String,
    pub prerequisites: Vec<String>,
    pub allowed_representations: Vec<String>,
    pub review_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphError {
    InvalidPack,
    DuplicateSkill,
    MissingPrerequisite,
    Cycle,
}

#[derive(Debug, Clone)]
pub struct CurriculumGraph {
    nodes: BTreeMap<String, SkillNode>,
}

#[derive(Deserialize)]
struct Pack {
    nodes: Vec<RawNode>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawNode {
    id: String,
    subject: String,
    objective: String,
    band: String,
    prerequisites: Vec<String>,
    allowed_representations: Vec<String>,
    review_status: String,
}

impl CurriculumGraph {
    /// Load and validate the frozen Foundation and Years 1–6 draft packs.
    pub fn load() -> Result<Self, GraphError> {
        let foundation: Pack =
            serde_json::from_str(include_str!("../../../../content/curriculum/seed-v1.json"))
                .map_err(|_| GraphError::InvalidPack)?;
        let years: Pack = serde_json::from_str(include_str!(
            "../../../../content/curriculum/years/seed-years1to6-v1.json"
        ))
        .map_err(|_| GraphError::InvalidPack)?;

        let mut nodes = BTreeMap::new();
        for raw in foundation.nodes.into_iter().chain(years.nodes) {
            let node = SkillNode {
                id: raw.id,
                subject: raw.subject,
                objective: raw.objective,
                band: raw.band,
                prerequisites: raw.prerequisites,
                allowed_representations: raw.allowed_representations,
                review_status: raw.review_status,
            };
            if node.id.is_empty()
                || node.objective.is_empty()
                || node.subject.is_empty()
                || node.allowed_representations.is_empty()
                || nodes.insert(node.id.clone(), node).is_some()
            {
                return Err(GraphError::DuplicateSkill);
            }
        }
        for node in nodes.values() {
            if node.prerequisites.iter().any(|p| !nodes.contains_key(p)) {
                return Err(GraphError::MissingPrerequisite);
            }
        }
        let graph = Self { nodes };
        graph.topological_order()?;
        Ok(graph)
    }

    pub fn get(&self, skill_id: &str) -> Option<&SkillNode> {
        self.nodes.get(skill_id)
    }

    pub fn nodes(&self) -> impl Iterator<Item = &SkillNode> {
        self.nodes.values()
    }

    pub fn topological_order(&self) -> Result<Vec<&SkillNode>, GraphError> {
        let mut indegree: BTreeMap<&str, usize> = self
            .nodes
            .values()
            .map(|n| (n.id.as_str(), n.prerequisites.len()))
            .collect();
        let mut children: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for node in self.nodes.values() {
            for parent in &node.prerequisites {
                children
                    .entry(parent.as_str())
                    .or_default()
                    .push(node.id.as_str());
            }
        }
        let mut ready: BTreeSet<&str> = indegree
            .iter()
            .filter_map(|(id, count)| (*count == 0).then_some(*id))
            .collect();
        let mut order = Vec::with_capacity(self.nodes.len());
        while let Some(id) = ready.pop_first() {
            order.push(self.nodes.get(id).ok_or(GraphError::InvalidPack)?);
            for child in children.get(id).into_iter().flatten() {
                let count = indegree.get_mut(child).ok_or(GraphError::InvalidPack)?;
                *count -= 1;
                if *count == 0 {
                    ready.insert(child);
                }
            }
        }
        if order.len() != self.nodes.len() {
            return Err(GraphError::Cycle);
        }
        Ok(order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_packs_load_as_one_complete_acyclic_graph_without_approvals() {
        let graph = CurriculumGraph::load().unwrap();
        assert_eq!(graph.nodes().count(), 120);
        assert_eq!(graph.topological_order().unwrap().len(), 120);
        assert_eq!(graph.get("M01").unwrap().review_status, "pending");
        assert_eq!(graph.get("MY101").unwrap().prerequisites, ["M05"]);
    }
}
