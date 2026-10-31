//! Finite, server-owned keepsakes. Art generation never grants inventory.
use super::Location;
pub struct Find {
    pub id: &'static str,
    pub source: &'static str,
    pub name: &'static str,
    pub location: Location,
}
pub const FINDS: &[Find] = &[
    Find {
        id: "lucky_clover",
        source: "find_clover",
        name: "a lucky clover",
        location: Location::BridgeBank,
    },
    Find {
        id: "ribbon",
        source: "find_ribbon",
        name: "a golden ribbon",
        location: Location::Workshop,
    },
    Find {
        id: "river_crystal",
        source: "find_crystal",
        name: "a river crystal",
        location: Location::FarBank,
    },
    Find {
        id: "seashell",
        source: "find_shell",
        name: "a pearly seashell",
        location: Location::Cove,
    },
    Find {
        id: "fern_leaf",
        source: "find_fern",
        name: "a curled fern leaf",
        location: Location::FernHollow,
    },
    Find {
        id: "moon_mushroom",
        source: "find_mushroom",
        name: "a moon mushroom",
        location: Location::MossyClearing,
    },
    Find {
        id: "feather",
        source: "find_feather",
        name: "an owl feather",
        location: Location::OwlGrove,
    },
    Find {
        id: "acorn",
        source: "find_acorn",
        name: "a golden acorn",
        location: Location::AcornTrail,
    },
];
