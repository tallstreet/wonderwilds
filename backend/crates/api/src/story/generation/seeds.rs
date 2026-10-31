//! Narrative ingredients and playable offline arcs, not new game mechanics.
pub(super) struct Seed {
    pub kind: &'static str,
    pub wish: &'static str,
    pub structure: &'static str,
    pub title: &'static str,
    pub premise: &'static str,
    // Artwork, objective, discovery. The route supplies each destination.
    pub mechanics: &'static [&'static str],
    pub steps: &'static [(&'static str, &'static str, &'static str)],
    pub payoff: &'static str,
}

pub(super) const SEEDS: &[Seed] = &[
    Seed {
        kind: "An unexpected musician",
        wish: "A neighbour wants to identify a strange sound before describing it to a friend",
        structure: "Hear a puzzling clue, then discover a surprising natural source",
        title: "The whistling stem",
        premise: "A little whistle answers the breeze. Follow the sound into the neighbouring area and discover who our musician is.",
        mechanics: &[],
        steps: &[
            (
                "instrument",
                "Listen for the whistle",
                "The whistle rises when the breeze blows. An answering note drifts from the neighbouring area.",
            ),
            (
                "hollow",
                "Look for the answering note",
                "Wind whistles through a hollow stem. Our musician is the breeze, playing a tiny natural flute!",
            ),
        ],
        payoff: "We can tell our friend about the breeze's tiny flute and listen for its song together.",
    },
    Seed {
        kind: "A mistaken first impression",
        wish: "A neighbour thinks a leaf is damaged and wants to understand what happened",
        structure: "One close observation overturns an ordinary assumption",
        title: "The folded leaf",
        premise: "I thought this leaf was broken, but its edges are neatly folded. Could a closer look explain the fold?",
        mechanics: &[],
        steps: &[(
            "leaf",
            "Notice the folded edges",
            "Fine silk holds the leaf edges together. It is a tiny insect's shelter, so we leave its roof undisturbed.",
        )],
        payoff: "The leaf is a little home, not a broken thing. We can admire its clever fold without opening it.",
    },
    Seed {
        kind: "A picture that changes with viewpoint",
        wish: "A neighbour wants to understand why a familiar landmark looks different",
        structure: "Observe an odd shape, compare another viewpoint, then reinterpret the first clue",
        title: "The disappearing stripe",
        premise: "A bright stripe on a stone seems to vanish. Compare how light falls here and nearby, then look at our stone again.",
        mechanics: &[],
        steps: &[
            (
                "stones",
                "Look at the bright stripe",
                "The stripe shines on the stone's wet side. A similar glimmer catches the light along the next path.",
            ),
            (
                "trail",
                "Compare the glimmer beside the path",
                "A damp patch glimmers only where sunlight reaches it. Shade makes the same patch look dark.",
            ),
            (
                "stones",
                "Look again at the stone's shaded side",
                "The stripe is still there in the shade, but no longer sparkles. Light made our stripe seem to disappear!",
            ),
        ],
        payoff: "Our stone kept its stripe all along. We discovered how light can change a familiar sight.",
    },
    Seed {
        kind: "A tiny visitor's trail",
        wish: "A neighbour is curious about a silvery mark with no visible maker",
        structure: "Follow a physical trace to an explanation without disturbing its maker",
        title: "The silver ribbon",
        premise: "A silver ribbon crosses the path without any cloth. Follow its gentle curve and see where it leads.",
        mechanics: &[],
        steps: &[
            (
                "trail",
                "Notice the silvery curve",
                "The ribbon glistens along the path towards a broad leaf in the neighbouring area.",
            ),
            (
                "leaf",
                "Look beside the broad leaf",
                "A snail rests beneath the leaf. Its glistening trail explains our silver ribbon; we let it rest.",
            ),
        ],
        payoff: "Our ribbon was a snail's travelling trace. We can describe its journey without waking our tiny visitor.",
    },
    Seed {
        kind: "A quiet place for a shy listener",
        wish: "A neighbour enjoys birdsong but finds rustling leaves distracting",
        structure: "Compare two listening places to answer a friend's specific preference",
        title: "Room for a small song",
        premise: "My friend loves small bird calls. Could we compare listening places and describe somewhere sheltered from the rustling wind?",
        mechanics: &[],
        steps: &[
            (
                "hollow",
                "Listen beside the sheltered hollow",
                "The hollow blocks the wind. A small bird call is clear here; compare it with the open path nearby.",
            ),
            (
                "instrument",
                "Listen where the path is open",
                "Wind rustles the open branches and covers the small calls. The sheltered hollow suits our friend's quiet listening.",
            ),
        ],
        payoff: "We can recommend the sheltered hollow and explain why small songs are easier to hear there.",
    },
    Seed {
        kind: "A miniature weather mystery",
        wish: "A neighbour wonders why seedlings stayed dry after rain",
        structure: "One observation reveals how an overlooked detail helps something small",
        title: "A tiny umbrella",
        premise: "Rain dotted the path, yet a little patch stayed dry. Look closely for something that could explain this tiny weather mystery.",
        mechanics: &[],
        steps: &[(
            "leaf",
            "Look beneath the broad leaf",
            "Raindrops cling to a broad leaf above dry seedlings. Its sloping surface carried the rain around their little patch.",
        )],
        payoff: "The seedlings have a leafy umbrella. We can share the discovery and leave their shelter just as it is.",
    },
    Seed {
        kind: "An echo mistaken for an answer",
        wish: "A neighbour wants to know why a sound seems to reply",
        structure: "Notice a puzzling reply, test an alternative explanation, resolve the misunderstanding",
        title: "Who answered the tapping?",
        premise: "A tapping sound seems to get an answer. Compare the nearby listening spots, then return to the first sound with a new idea.",
        mechanics: &[],
        steps: &[
            (
                "hollow",
                "Listen for the answering tap",
                "A loose twig taps a hollow trunk in the wind. A softer tap follows it; listen from the nearby open path.",
            ),
            (
                "trail",
                "Listen away from the hollow trunk",
                "The first tap reaches the open path, but its soft reply fades. The reply is strongest near the hollow.",
            ),
            (
                "instrument",
                "Listen for the delay beside the trunk",
                "The softer tap follows the twig's tap each time. The hollow carries an echo, rather than another musician.",
            ),
        ],
        payoff: "The answering voice was an echo. Our neighbour can enjoy the twig's little duet with its own sound.",
    },
    Seed {
        kind: "A woodland pattern puzzle",
        wish: "A neighbour thinks the marks on stones are tiny painted trees",
        structure: "Compare a puzzling pattern with a living example to discover a connection",
        title: "Trees on a stone",
        premise: "Branching marks cover a stone. Compare them with the nearby leaves and help me describe what they resemble.",
        mechanics: &[],
        steps: &[
            (
                "stones",
                "Notice the branching marks",
                "Thin marks branch across the stone. Their shapes resemble the veins of a leaf along the neighbouring path.",
            ),
            (
                "leaf",
                "Compare the leaf's branching veins",
                "The leaf veins branch into smaller lines too. Different things can share a branching pattern without being tiny trees.",
            ),
        ],
        payoff: "We found a shared pattern in a stone and a leaf. Our neighbour has a new way to describe the marks.",
    },
    Seed {
        kind: "A familiar object with a surprising detail",
        wish: "A neighbour wonders why fallen acorn cups have different edges",
        structure: "A single careful comparison reveals a previously unnoticed difference",
        title: "Little cups, different rims",
        premise: "These fallen acorn cups look alike from far away. Could a closer look reveal something my friend has missed?",
        mechanics: &[],
        steps: &[(
            "stones",
            "Compare the fallen cups' rims",
            "One cup has a smooth worn rim; another still has its little scales. Weathering made similar cups look different.",
        )],
        payoff: "Our friend can spot smooth and scaly rims now. Familiar little cups held a detail we had overlooked.",
    },
    Seed {
        kind: "A story told by the breeze",
        wish: "A neighbour wants a true observation to inspire a short woodland tale",
        structure: "Discover a movement, compare its cause, then connect the clues into a tellable story",
        title: "The travelling leaf's tale",
        premise: "A leaf turns beside the path. Notice its movement, compare the sheltered place nearby, then see what gives our little tale its ending.",
        mechanics: &[],
        steps: &[
            (
                "trail",
                "Watch the leaf turn beside the path",
                "A dry leaf turns whenever the wind reaches the path. The sheltered place nearby may show why it pauses.",
            ),
            (
                "hollow",
                "Notice the stillness in the sheltered place",
                "Leaves lie still behind the hollow trunk. It blocks the gusts that move leaves along the open path.",
            ),
            (
                "leaf",
                "Notice where the turning leaf pauses",
                "The turning leaf settles in the trunk's shelter. The breeze began its journey and the quiet shelter gave it an ending.",
            ),
        ],
        payoff: "We have a little tale grounded in what we saw: a breezy journey ending in a sheltered resting place.",
    },
    Seed {
        kind: "A colour that depends on the weather",
        wish: "A neighbour thinks the rain has changed a stone into a different kind",
        structure: "Compare wet and dry surfaces to resolve an apparent transformation",
        title: "The rain-dark stone",
        premise: "Rain made a familiar stone look much darker. Compare its surface with a sheltered stone and help me understand the change.",
        mechanics: &[],
        steps: &[
            (
                "stones",
                "Notice the wet stone's colour",
                "The wet stone looks dark, but a dry patch on its underside is pale. Compare a sheltered stone nearby.",
            ),
            (
                "hollow",
                "Compare the stone beneath the shelter",
                "The sheltered stone is dry and pale, like the first stone's underside. Water changed the appearance, not the kind of stone.",
            ),
        ],
        payoff: "Our familiar stone only wears a rain-dark coat. We can explain the difference between a wet surface and a new stone.",
    },
    Seed {
        kind: "A rhythm hiding in ordinary movement",
        wish: "A neighbour wants inspiration for a greeting rhyme without moving or making anything",
        structure: "One thoughtful listening pause connects an ordinary sound to a creative idea",
        title: "A greeting in the rustle",
        premise: "I want an idea for a gentle greeting rhyme. Could the leaves' changing rustle suggest how it might sound?",
        mechanics: &[],
        steps: &[(
            "instrument",
            "Listen for pauses in the rustle",
            "A gust makes the leaves rustle, then leaves a quiet pause. A greeting could have a lively beginning and a soft, roomy ending.",
        )],
        payoff: "The rustle and its pause gave our neighbour an idea for a greeting, without picking leaves or disturbing the grove.",
    },
];

// These are complete server-supported recipes, not model-invented effects.
pub(super) const PROJECTS: &[Seed] = &[
    Seed {
        kind: "Build a resting seat",
        wish: "A neighbour wants a comfortable stopping place on the next path",
        structure: "Gather fallen wood, prepare planks, then build a seat where it is needed",
        title: "A seat along the way",
        premise: "Let's make a resting seat for the neighbouring path. Gather loose wood, prepare planks here, then build the seat next door.",
        mechanics: &["gather_wood", "craft_planks", "build_seat"],
        steps: &[
            (
                "hollow",
                "Gather loose project wood",
                "The fallen wood is in your bag, reserved for this seat.",
            ),
            (
                "stones",
                "Prepare the seat planks",
                "The wood has become sturdy planks. Carry them to the neighbouring path.",
            ),
            (
                "trail",
                "Build the resting seat",
                "The planks are now a seat beside the path.",
            ),
        ],
        payoff: "The new seat is ready for a peaceful pause. You finished right where your help was needed.",
    },
    Seed {
        kind: "Prepare and deliver a picnic",
        wish: "A neighbour wants to send a welcome picnic to the next area",
        structure: "Pick mission berries, pack a basket, deliver it to the picnic spot",
        title: "A picnic for next door",
        premise: "Our neighbours have a picnic spot but no basket. Pick berries, pack them here and take the picnic to the neighbouring area.",
        mechanics: &["gather_berries", "pack_basket", "deliver_basket"],
        steps: &[
            (
                "leaf",
                "Pick berries for the basket",
                "The picnic berries are in your bag, ready to pack.",
            ),
            (
                "stones",
                "Pack the picnic basket",
                "The berries are packed in a basket. Its next stop is the neighbouring picnic spot.",
            ),
            (
                "trail",
                "Deliver the picnic basket",
                "The basket has been delivered and the picnic is laid out.",
            ),
        ],
        payoff: "A picnic is ready for our neighbours. The delivery is complete; there is no return errand.",
    },
    Seed {
        kind: "Make and place a path lantern",
        wish: "A neighbour would enjoy a glowing landmark on the next path",
        structure: "Gather glow pebbles, make a lantern, place it at the destination",
        title: "A glow along the path",
        premise: "Let's add a glowing landmark next door. Gather glow pebbles, make a lantern here, then place it beside the neighbouring path.",
        mechanics: &["gather_glow_pebbles", "craft_lantern", "place_lantern"],
        steps: &[
            (
                "stones",
                "Gather project glow pebbles",
                "The glow pebbles are safely in your bag for this lantern.",
            ),
            (
                "instrument",
                "Make the path lantern",
                "A lantern is ready. Carry its gentle glow to the neighbouring path.",
            ),
            (
                "trail",
                "Place the path lantern",
                "The lantern now glows beside the neighbouring path.",
            ),
        ],
        payoff: "The path has a new glowing landmark. Your lantern stays here to brighten future visits.",
    },
    Seed {
        kind: "Choose a design and build it",
        wish: "A neighbour invites the player to choose the look of a new resting corner",
        structure: "Choose leafy or starry decoration, gather wood, make planks and build the chosen corner",
        title: "Our kind of cosy",
        premise: "Would a leafy or starry resting corner feel cosy? Choose a design, gather wood and prepare planks, then build your choice next door.",
        mechanics: &["choose_style", "gather_wood", "craft_planks", "build_seat"],
        steps: &[
            (
                "leaf",
                "Choose the corner's design",
                "Your chosen design is saved for the new resting corner.",
            ),
            (
                "hollow",
                "Gather the corner's wood",
                "The loose wood is reserved for your chosen corner.",
            ),
            (
                "stones",
                "Prepare the corner's planks",
                "The planks are ready. Take them to the neighbouring area to build your design.",
            ),
            (
                "trail",
                "Build your chosen corner",
                "Your chosen resting corner is built beside the path.",
            ),
        ],
        payoff: "Your resting corner is ready, with the design you chose. You can enjoy it on future visits.",
    },
    Seed {
        kind: "Prepare a useful supply delivery",
        wish: "A neighbour wants spare planks delivered to a nearby project store",
        structure: "Collect project wood, prepare planks, leave them at a destination supply stack",
        title: "Planks for our neighbours",
        premise: "A project store next door needs planks. Gather fallen wood and prepare it here, then deliver the stack where our neighbours can use it.",
        mechanics: &["gather_wood", "craft_planks", "deliver_planks"],
        steps: &[
            (
                "hollow",
                "Gather wood for the delivery",
                "The delivery wood is in your bag.",
            ),
            (
                "stones",
                "Prepare the delivery planks",
                "The delivery planks are ready for the neighbouring project store.",
            ),
            (
                "trail",
                "Deliver the plank stack",
                "The planks are stacked in the project store, ready for our neighbours.",
            ),
        ],
        payoff: "The supply delivery is complete. A useful stack of planks now waits at the neighbouring project store.",
    },
    Seed {
        kind: "Design a glowing welcome",
        wish: "A neighbour wants the player to choose a lantern decoration for a path landmark",
        structure: "Choose a leafy or starry decoration, gather glow pebbles, make and place the lantern",
        title: "A welcome with your own glow",
        premise: "Choose a leafy or starry decoration for a welcome lantern. Gather glow pebbles, make the lantern and place your design on the next path.",
        mechanics: &[
            "choose_style",
            "gather_glow_pebbles",
            "craft_lantern",
            "place_lantern",
        ],
        steps: &[
            (
                "leaf",
                "Choose the lantern decoration",
                "Your lantern decoration is saved.",
            ),
            (
                "stones",
                "Gather the lantern's glow pebbles",
                "The pebbles are in your bag for your decorated lantern.",
            ),
            (
                "instrument",
                "Make your decorated lantern",
                "Your lantern is ready to carry to the neighbouring path.",
            ),
            (
                "trail",
                "Place your decorated lantern",
                "Your chosen lantern now lights the path.",
            ),
        ],
        payoff: "A lantern with your chosen decoration welcomes visitors. Your design remains here for future walks.",
    },
];
