//! Original, deterministic practice starters. These sample a skill, not its entire assessment rubric.
//! Productive language tasks deliberately require adult review instead of a tile-based certificate.
use super::*;

pub(super) struct Item {
    pub stimulus: String,
    pub prompt: String,
    pub answers: Vec<String>,
    pub hint: String,
    pub review: bool,
}

fn cards(stimulus: impl ToString, prompt: &str, answers: &[&str], hint: &str) -> Item {
    Item {
        stimulus: stimulus.to_string(),
        prompt: prompt.to_owned(),
        answers: answers.iter().map(|s| (*s).to_owned()).collect(),
        hint: hint.to_owned(),
        review: false,
    }
}
fn quantity(stimulus: impl ToString, prompt: &str, answer: i64, hint: &str) -> Item {
    let mut item = cards(stimulus, prompt, &[], hint);
    item.answers = [
        answer,
        answer + 1,
        if answer > 0 { answer - 1 } else { answer + 2 },
    ]
    .iter()
    .map(ToString::to_string)
    .collect();
    item
}
fn workshop(stimulus: &str, prompt: &str, hint: &str) -> Item {
    let mut item = cards(stimulus, prompt, &[], hint);
    item.review = true;
    item
}

pub(super) fn item(skill: &str, seed: u64) -> Option<Item> {
    let n = (seed % 12 + 1) as i64;
    let n = match skill {
        "M07" => n % 8 + 1,
        "M08" | "MY204" => n % 6 + 1,
        "M15" => n % 4 + 1,
        "M17" => n % 4 + 2,
        "MY203" => n % 9 + 2,
        "MY302" | "MY401" => n % 8 + 1,
        "MY403" | "MY506" => n % 5 + 1,
        "MY507" => n % 8 + 5,
        "MY508" => n % 10 + 1,
        _ => n,
    };
    let word_slot = (seed % 8) as usize;
    let item = match skill {
        "M02" => quantity(
            "●".repeat((n % 4 + 1) as usize),
            "Look at the small group. How many do you see?",
            n % 4 + 1,
            "Look for a pair, or two pairs. You can count if you need to.",
        ),
        "M07" => quantity(
            format!(
                "Pip has {n} crystals. Fern brings 2 more.\n{} + ●●",
                "●".repeat(n as usize)
            ),
            "How many crystals are there now?",
            n + 2,
            "Start with Pip's group, then count on two more.",
        ),
        "M08" => quantity(
            format!("{} berries → 2 picnic plates", 2 * n),
            "Share equally. How many berries go on each plate?",
            n,
            "Give one berry to each plate in turn until none are left.",
        ),
        "M09" => {
            let (a, b) = [("●", "▲"), ("★", "■"), ("▲", "◆")][word_slot % 3];
            cards(
                format!("{a} {b} {a} {b} {a} ?"),
                "Which piece comes next?",
                &[b, a, "✿"],
                "Find the small part that repeats, then keep it going.",
            )
        }
        "M10" => cards(
            [
                "▲ has 3 straight sides.",
                "■ has 4 equal straight sides.",
                "● has a curved edge.",
            ][word_slot % 3],
            "Pick the shape name.",
            match word_slot % 3 {
                0 => &["triangle", "square", "circle"],
                1 => &["square", "circle", "triangle"],
                _ => &["circle", "triangle", "square"],
            },
            "Look at the sides and corners.",
        ),
        "M11" => cards(
            "      ★\n      ●",
            "The star is where, compared with the circle?",
            &["above", "below", "beside"],
            "Look from the circle towards the star.",
        ),
        "M12" => cards(
            format!("Fern's ribbon: {} units\nPip's ribbon: {} units", n + 2, n),
            "Which ribbon is longer?",
            &["Fern's ribbon", "Pip's ribbon", "They are the same length"],
            "Line up one end of each ribbon. Compare the other ends.",
        ),
        "M13" => cards(
            "Pip wakes up → eats breakfast → brushes teeth → leaves home",
            "What happens just after breakfast?",
            &["brushes teeth", "wakes up", "leaves home"],
            "Follow the arrows one step at a time.",
        ),
        "M14" => cards(
            "Our basket is for shapes with three sides: ▲ △",
            "Which shape belongs in this basket?",
            &["triangle", "circle", "square"],
            "Check the sorting rule before you add a shape.",
        ),
        "M15" => quantity(
            format!(
                "Our fruit vote\nApples: {}\nPears: {}",
                "●".repeat(n as usize),
                "●".repeat((n + 1) as usize)
            ),
            "How many votes did apples get?",
            n,
            "Each dot stands for one vote. Count only the apple row.",
        ),
        "M16" => quantity(
            format!("{} bundles of ten seeds + 3 loose seeds", n % 2 + 1),
            "How many seeds altogether?",
            (n % 2 + 1) * 10 + 3,
            "Count each bundle as ten, then add the loose seeds.",
        ),
        "M17" => quantity(
            format!("{n} planting rows, with 2 seeds in every row"),
            "How many seeds altogether?",
            n * 2,
            "Count in twos, once for each equal row.",
        ),
        "M18" => cards(
            "A round picnic cake is cut into 2 equal pieces.",
            "What is one equal piece called?",
            &["one half", "one quarter", "one whole"],
            "The name tells how many equal parts make the whole.",
        ),
        "MY101" => quantity(
            format!("{}, {}, {}, ?", n, n + 5, n + 10),
            "Count on in fives. Which number is next?",
            n + 15,
            "Add five each time. You can use a number line.",
        ),
        "MY102" => quantity(
            format!("{} tens and {} ones", n % 8 + 1, n % 9),
            "Build the number. How much is it?",
            (n % 8 + 1) * 10 + n % 9,
            "Each ten is worth ten ones.",
        ),
        "MY103" => cards(
            format!("{}  ?  {}", n + 20, n + 23),
            "Which comparison symbol fits?",
            &["<", "=", ">"],
            "The open end faces the larger number.",
        ),
        "MY104" => quantity(
            format!("{} + 7 = ?", n % 10 + 1),
            "How many altogether?",
            n % 10 + 8,
            "Try making ten first, then add what is left.",
        ),
        "MY105" => quantity(
            format!("{} − 6 = ?", n % 10 + 10),
            "How many are left?",
            n % 10 + 4,
            "Count back six, or think: six plus what makes the starting number?",
        ),
        "MY106" => cards(
            "A box has 6 flat square faces, 12 edges and 8 corners.",
            "Which solid is it?",
            &["cube", "sphere", "cone"],
            "Imagine a dice. Look at its flat faces.",
        ),
        "MY107" => quantity(
            format!("Pip's path is {n} same-sized blocks long. Fern's path is 2 blocks longer."),
            "How many blocks long is Fern's path?",
            n + 2,
            "Use the same unit for both paths.",
        ),
        "MY108" => quantity(
            format!("{}, {}, {}, ?", n, n + 2, n + 4),
            "Continue the number pattern.",
            n + 6,
            "Find the amount added each time.",
        ),
        "MY109" => quantity(
            format!("Bird count\nRobins: {n}\nFinches: {}", n + 2),
            "How many birds were counted in all?",
            2 * n + 2,
            "Read both rows and combine their counts.",
        ),
        "MY110" => cards(
            "Clock clue: the long hand points to 6 and the short hand is between 3 and 4.",
            "What time is it?",
            &["3:30", "3:00", "6:30"],
            "The long hand at 6 shows half past the hour.",
        ),
        "MY201" => quantity(
            format!("{} hundreds, 4 tens, 2 ones", n % 8 + 1),
            "What number is represented?",
            (n % 8 + 1) * 100 + 42,
            "Combine the hundreds, tens and ones.",
        ),
        "MY202" => quantity(
            format!("{} + 18 = ?", n + 29),
            "Find the total.",
            n + 47,
            "Add tens, then ones. Regroup ten ones when needed.",
        ),
        "MY203" => quantity(
            format!("{n} rows × 3 lanterns in each row"),
            "How many lanterns altogether?",
            n * 3,
            "Add three for each row or use a multiplication fact.",
        ),
        "MY204" => quantity(
            format!("{} seeds shared equally among 3 pots", n * 3),
            "How many seeds in each pot?",
            n,
            "Deal one seed to each pot in turn.",
        ),
        "MY205" => quantity(
            format!("{} berries shared into 4 equal groups", n * 4),
            "How many berries are one quarter of the collection?",
            n,
            "A quarter is one of four equal parts.",
        ),
        "MY206" => quantity(
            format!(
                "A ribbon starts at the 2 cm mark and ends at the {} cm mark.",
                n + 5
            ),
            "How long is the ribbon in centimetres?",
            n + 3,
            "Find the distance between the two marks, rather than the ending mark alone.",
        ),
        "MY207" => cards(
            "It is quarter past 7 in the morning.",
            "Which clock label matches?",
            &["7:15 AM", "7:45 AM", "7:15 PM"],
            "Quarter past is fifteen minutes after the hour. Morning uses AM.",
        ),
        "MY208" => cards(
            "Fold a square exactly in half so the two sides match.",
            "How many lines of symmetry does a square have?",
            &["4", "2", "1"],
            "Try vertical, horizontal and diagonal folds.",
        ),
        "MY209" => quantity(
            format!(
                "Library visits\nMonday: {n}\nTuesday: {}\nWednesday: {}",
                n + 3,
                n + 1
            ),
            "How many more visits were there on Tuesday than Monday?",
            3,
            "Compare the Tuesday and Monday counts by subtraction.",
        ),
        "MY301" => quantity(
            format!("Round {} to the nearest hundred.", n * 100 + 47),
            "Which rounded number is closest?",
            n * 100,
            "Look at the tens digit to decide whether to round up.",
        ),
        "MY302" => quantity(
            format!("{} + 125 = ?", n * 100 + 87),
            "Find the sum.",
            n * 100 + 212,
            "Align ones under ones, tens under tens, and hundreds under hundreds.",
        ),
        "MY303" => quantity(
            format!("{} × 7 = ?", n % 9 + 2),
            "Use your multiplication facts.",
            (n % 9 + 2) * 7,
            "Think of equal groups of seven.",
        ),
        "MY304" => quantity(
            format!("{} shells packed into bags of 4", n * 4 + 1),
            "How many shells are left after filling all the complete bags?",
            1,
            "Make full groups of four. The objects left over are the remainder.",
        ),
        "MY305" => cards(
            "Compare 2/7 and 5/7.",
            "Which fraction is larger?",
            &["5/7", "2/7", "They are equal"],
            "The parts are the same size, so compare how many parts there are.",
        ),
        "MY306" => quantity(
            format!("A rectangle is {n} cm long and 3 cm wide."),
            "What is its area in square centimetres?",
            n * 3,
            "Area counts the squares inside: length times width.",
        ),
        "MY307" => quantity(
            format!(
                "Bird graph\nEach ★ means 2 birds.\nRobins: {}",
                "★".repeat(n as usize)
            ),
            "How many robins does the graph show?",
            n * 2,
            "Use the key: count two birds for every star.",
        ),
        "MY308" => cards(
            "A bag contains only blue counters. Pip takes one without looking.",
            "How likely is a blue counter?",
            &["certain", "unlikely", "impossible"],
            "Consider every possible counter in the bag.",
        ),
        "MY309" => cards(
            "North ↑\nPip is on square B2. He walks one square east.",
            "Where is Pip now?",
            &["C2", "A2", "B3"],
            "East goes right. Change the column letter, keeping the row.",
        ),
        "MY401" => quantity(
            format!(
                "In {}, what is the value of the 3 in the ten-thousands place?",
                n * 100000 + 30000 + 421
            ),
            "Choose the value, not just the digit.",
            30000,
            "Read the place occupied by the digit from right to left.",
        ),
        "MY402" => quantity(
            format!("{} × 4 = ?", n + 20),
            "Find the product.",
            (n + 20) * 4,
            "Split the two-digit number into tens and ones. Multiply both parts.",
        ),
        "MY403" => quantity(
            format!("1/2 = ?/{}", (n + 1) * 2),
            "Which numerator makes these fractions equivalent?",
            n + 1,
            "Multiply numerator and denominator by the same number.",
        ),
        "MY404" => cards(
            "A hundredths grid has 37 of its 100 squares shaded.",
            "Which decimal describes the shaded amount?",
            &["0.37", "3.7", "0.037"],
            "Thirty-seven hundredths has two digits after the decimal point.",
        ),
        "MY405" => quantity(
            format!("A {n} cm by 4 cm rectangle has a 1 cm by 2 cm corner removed."),
            "What is the remaining area in square centimetres?",
            n * 4 - 2,
            "Find the large rectangle's area, then subtract the removed corner.",
        ),
        "MY406" => cards(
            "Pond temperature\n9 AM: 12°C\n10 AM: 14°C\n11 AM: 16°C",
            "If the same rise continues, what temperature would you predict at noon?",
            &["18°C", "16°C", "14°C"],
            "Compare the change each hour. A prediction assumes the trend continues.",
        ),
        "MY407" => cards(
            "A fair spinner has 4 equal sections: red, red, red, blue.",
            "What is the probability of blue?",
            &["1/4", "3/4", "1/2"],
            "Count blue sections, then count all equal sections.",
        ),
        "MY408" => quantity(
            format!("The walk begins at 9:10 AM and lasts {} minutes.", n + 15),
            "How many minutes after 9:00 AM does it end?",
            n + 25,
            "Start ten minutes past nine, then add the walking time.",
        ),
        "MY501" => quantity(
            format!("{} ÷ 12 = ?", (n + 20) * 12),
            "Find how many groups of twelve.",
            n + 20,
            "Use the related multiplication fact. Split the dividend if useful.",
        ),
        "MY502" => cards(
            "Compare 2/3 and 3/4.",
            "Which fraction is larger?",
            &["3/4", "2/3", "They are equal"],
            "Rewrite both fractions in twelfths, then compare.",
        ),
        "MY503" => cards(
            "1/2 + 1/4 = ?",
            "Choose the total.",
            &["3/4", "2/6", "1/8"],
            "Change one half into two quarters before adding.",
        ),
        "MY504" => cards(
            "0.375 + 0.200 = ?",
            "Choose the total.",
            &["0.575", "0.395", "0.577"],
            "Line up decimal places and add the thousandths, hundredths and tenths.",
        ),
        "MY505" => quantity(
            format!("A tray has 100 seeds. {} of them sprout.", n * 5),
            "What percentage sprouted? Choose the number before the % sign.",
            n * 5,
            "Percent means out of one hundred.",
        ),
        "MY506" => quantity(
            format!("A graph follows y = 2 × x.\nx = {n}"),
            "What is the y-coordinate?",
            n * 2,
            "Substitute the x-value into the rule.",
        ),
        "MY507" => quantity(
            format!("Daily visitors: {n}, {}, {}", n + 1, n + 2),
            "What is the mean number of visitors?",
            n + 1,
            "Add all three values and divide by three.",
        ),
        "MY508" => quantity(
            format!("A rectangular tank measures {n} cm × 3 cm × 2 cm."),
            "What is its volume in cubic centimetres?",
            n * 6,
            "Multiply length, width and height.",
        ),
        "MY601" => quantity(
            format!("({n} + 3) × 2 = ?"),
            "Follow the order of operations.",
            (n + 3) * 2,
            "Work inside the brackets before multiplying.",
        ),
        "MY602" => quantity(
            format!("A ${} backpack is 25% off.", n * 4),
            "How many dollars is the discount?",
            n,
            "Twenty-five percent is one quarter. Divide the price by four.",
        ),
        "MY603" => cards(
            ["11", "13", "17", "19", "23", "29", "31", "37"][word_slot],
            "Is this number prime or composite?",
            &["prime", "composite"],
            "Check whether it has factors other than one and itself.",
        ),
        "MY604" => quantity(
            format!(
                "Paint uses 2 cups blue for every 3 cups yellow.\nBlue: {} cups",
                n * 2
            ),
            "How many cups of yellow keep the same ratio?",
            n * 3,
            "Scale both parts of the ratio by the same amount.",
        ),
        "MY605" => quantity(
            format!("x + 7 = {}", n + 7),
            "What value of x balances the equation?",
            n,
            "Undo adding seven by subtracting seven from both sides.",
        ),
        "MY606" => quantity(
            format!("A triangle has angles 60° and {}°.", n + 40),
            "How many degrees is the third angle?",
            80 - n,
            "The three interior angles in a triangle add to 180 degrees.",
        ),
        "MY607" => cards(
            "Two bars show 99 and 100 votes. The vertical axis starts at 98, making the second bar look twice as tall.",
            "What should you check before comparing the bars?",
            &[
                "the axis starting point",
                "the bar colours",
                "the chart title's font",
            ],
            "Compare the actual values as well as the visible bar lengths.",
        ),
        "MY608" => cards(
            "The probability of rain is 3/10.",
            "What is the probability of no rain?",
            &["7/10", "3/10", "1/10"],
            "An event and its complement have probabilities that add to one.",
        ),
        _ => return english(skill, word_slot),
    };
    Some(item)
}

fn english(skill: &str, slot: usize) -> Option<Item> {
    // Each tuple is authored content, never a provider prompt or generated question.
    let words = ["cat", "dog", "sun", "map", "hen", "pig", "cup", "fox"];
    let word = words[slot];
    let item = match skill {
        "E01" => workshop(
            "Take turns with a grown-up. Put a pencil beside a book. Then put the pencil on the book.",
            "Listen as your grown-up says the directions, then try them. Swap turns!",
            "Give one direction at a time if you need help. A grown-up observes listening and turn-taking.",
        ),
        "E02" => cards(
            "Pip scoops soil into a pot.",
            "Which word names what Pip is doing?",
            &["scooping", "sleeping", "flying"],
            "Picture the action. You can act it out.",
        ),
        "E03" => cards(
            format!(
                "{word} · {}",
                ["hat", "log", "fun", "cap", "pen", "wig", "pup", "box"][slot]
            ),
            "Say the words together. What do you notice?",
            &[
                "They rhyme",
                "They start with the same sound",
                "They have two syllables each",
            ],
            "Listen to the ends of the words.",
        ),
        "E04" => workshop(
            &format!("Say “{word}” slowly. Move one counter for each sound you hear."),
            "Tell a grown-up the first sound. What changes if you remove it?",
            "Use sounds rather than letter names. A grown-up checks the spoken sounds; a transcript cannot assess pronunciation.",
        ),
        "E05" => workshop(
            &format!(
                "Look at the first letter in “{word}”. Find something nearby that begins with its sound."
            ),
            "Say the letter's sound to a grown-up, then swap examples.",
            "A grown-up models the taught sound. Speech-to-text is not a pronunciation score.",
        ),
        "E06" | "EY101" => workshop(
            &format!("Read this word aloud: {word}.\nNow read: The {word} is here."),
            "Slide the sounds together, then read the whole word to a grown-up.",
            "A grown-up checks decoding. Reading the displayed word into speech-to-text would not establish independent decoding.",
        ),
        "E07" => cards(
            format!("Pip wants to label a picture of a {word}."),
            "Pick the correctly spelt label.",
            &[
                word,
                ["cot", "dig", "san", "mop", "hin", "peg", "cap", "fix"][slot],
                "zzz",
            ],
            "Say the word slowly. Check each sound against the letters.",
        ),
        "E08" | "EY102" => cards(
            [
                "I can ___ Pip.",
                "Pip is ___ the tree.",
                "We ___ a picnic.",
                "Come ___ me.",
                "The bird ___ small.",
                "I ___ a shell.",
                "This is ___ bag.",
                "We go ___ home.",
            ][slot],
            "Choose the word that completes the sign.",
            match slot {
                0 => &["see", "sea", "bee"],
                1 => &["under", "after", "happy"],
                2 => &["have", "has", "of"],
                3 => &["with", "wiv", "whit"],
                4 => &["is", "it", "if"],
                5 => &["have", "of", "has"],
                6 => &["my", "me", "by"],
                _ => &["to", "two", "too"],
            },
            "Read the whole sentence with your word and check that it makes sense.",
        ),
        "E09" => cards(
            "Pip hops. Fern hops?",
            "How many sentences are on this strip?",
            &["2", "1", "6"],
            "Look for capitals and the end punctuation of each sentence.",
        ),
        "E10" => cards(
            "The cup is red.",
            "What colour is the cup?",
            &["red", "blue", "green"],
            "Read the sentence yourself and look for the describing word.",
        ),
        "E11" | "EY107" => workshop(
            "Grown-up story: Pip found a seed. He planted it in a pot. A shoot grew. Pip smiled.",
            "Listen while a grown-up reads. Retell what happened first, next and last. What might happen next?",
            "Keep the story out of view during listening. Ask a question about it. A grown-up checks listening and retelling.",
        ),
        "E12" => cards(
            "Fern’s dog sleeps in bed.",
            "Where does Fern's dog sleep?",
            &["in bed", "in a tree", "in a pond"],
            "Read the short text yourself, then find the words that answer the question.",
        ),
        "E13" => workshop(
            "Pip is coming to your picnic.",
            "Make a short invitation for Pip. Tell him where to come. Draw, dictate to a grown-up, or write.",
            "Try: Dear Pip, please come to … Read your message to a grown-up and check whether Pip will know where to go.",
        ),
        "E14" => workshop(
            "Pip found one warm blanket. He shared it with cold Fern instead of keeping it all.",
            "Do you agree with Pip's choice? Tell a grown-up your reason.",
            "Try: I think … because … There can be several thoughtful answers. A grown-up listens to your reasoning.",
        ),
        "E15" => workshop(
            "A bird sat on a branch.",
            "Add detail to the sentence. Tell what the bird looks like and how it moves. Write or dictate it.",
            "Read your sentence back with a grown-up. Check spelling patterns you have been taught.",
        ),
        "EY103" => cards(
            "Pip gives his seedlings water each morning. He puts them near a sunny window. He checks their leaves every day.",
            "What is this text mostly about?",
            &[
                "how Pip cares for seedlings",
                "how to build a window",
                "why Pip likes rain",
            ],
            "Look for the idea that connects all the sentences.",
        ),
        "EY104" => workshop(
            "pip found a shell",
            "Write a complete sentence about the shell. Use a capital letter and end punctuation.",
            "Say your sentence first, then write it. A grown-up checks meaning, grammar and punctuation.",
        ),
        "EY105" => cards(
            "The tiny bird sings softly.",
            "Which word is the verb?",
            &["sings", "tiny", "bird"],
            "Find the word that tells what the bird does.",
        ),
        "EY106" => cards(
            format!(
                "Spell {word}: {}",
                [
                    "c _ t", "d _ g", "s _ n", "m _ p", "h _ n", "p _ g", "c _ p", "f _ x"
                ][slot]
            ),
            "Complete the word: choose the missing vowel.",
            match slot {
                0 | 3 => &["a", "e", "i"],
                1 | 7 => &["o", "a", "e"],
                2 | 6 => &["u", "a", "o"],
                4 => &["e", "i", "a"],
                _ => &["i", "e", "u"],
            },
            "Say the word slowly and listen for the middle vowel.",
        ),
        "EY201" => workshop(
            "The little robin landed beside the garden. It lifted a twig and carried it to a tall tree.",
            "Read this to a grown-up. Pause at full stops and make it sound like a story.",
            "Try unfamiliar words in parts, then reread the sentence smoothly. A grown-up checks fluency and expression.",
        ),
        "EY202" => cards(
            "Fern pulled on her raincoat and picked up an umbrella before opening the door.",
            "What can you infer about the weather?",
            &[
                "Fern expects rain",
                "It must be snowing",
                "Fern is going swimming",
            ],
            "Use the clues in what Fern takes with her.",
        ),
        "EY203" => workshop(
            "Think of a real or imaginary visit to the garden.",
            "Write or dictate a short recount with a beginning, middle and end.",
            "Plan with First … Next … Finally … Tell a grown-up who was there and what happened, then review your recount together.",
        ),
        "EY204" => cards(
            format!("One {word}. Two ___ ."),
            "Choose the plural label.",
            &[
                &format!("{word}{}", if word == "fox" { "es" } else { "s" }),
                word,
                &format!("{word}ing"),
            ],
            "Most of these words add -s. Words ending in x add -es.",
        ),
        "EY205" => cards(
            "Pip packed apples pears and plums.",
            "Which punctuation separates items in the list?",
            &["commas", "question marks", "apostrophes"],
            "Read each item and pause between them.",
        ),
        "EY206" => cards(
            "GARDEN BIRDS\nRobins\nRobins often search the soil for food.\nPicture caption: A robin carrying a worm.",
            "Which part tells you what the picture shows?",
            &["the caption", "the main heading", "the subheading"],
            "A caption is a short label connected to an illustration.",
        ),
        "EY301" => cards(
            "Pip gripped the rope. The bridge creaked under his feet, but Fern waited safely on the far bank. Pip took another careful step.",
            "Which detail best shows Pip is being brave?",
            &[
                "He continues despite being frightened",
                "The rope is long",
                "Fern is standing on a bank",
            ],
            "Connect what the character does with the difficulty he faces.",
        ),
        "EY302" => workshop(
            "Explain how to grow a seed in a pot.",
            "Make a short information text with a topic sentence, useful details and a closing sentence.",
            "Plan what the reader needs to know. A grown-up checks that your details support the topic and that the text ends clearly.",
        ),
        "EY303" => cards(
            "Pip needs to pack the basket again.",
            "Which word means pack again?",
            &["repack", "unpack", "packing"],
            "The prefix re- can mean again.",
        ),
        "EY304" => cards(
            "Please close the garden gate.",
            "Which sentence type is this?",
            &["command", "question", "statement"],
            "Think about what the sentence asks its reader to do.",
        ),
        "EY305" => cards(
            "After the long walk, Pip was exhausted and lay down to rest.",
            "What does exhausted mean here?",
            &["very tired", "very hungry", "very excited"],
            "Use the nearby action as a clue to the word's meaning.",
        ),
        "EY306" => workshop(
            "Your audience wants to learn how to care for a garden animal.",
            "Plan and give a short talk to a grown-up: introduce your topic, explain three ideas and finish clearly.",
            "Face your listener, speak clearly and invite a question. A grown-up gives feedback on clarity, audience and purpose.",
        ),
        "EY401" => cards(
            "Text A: A signed ranger report measured 12 nesting birds this morning.\nText B: An unsigned post says there are thousands, without describing any count.",
            "Which is the stronger source for today's bird count?",
            &[
                "Text A, because it describes a measured count",
                "Text B, because its number is bigger",
                "Both are equally reliable just because they are written",
            ],
            "Compare the evidence, author and method behind each claim.",
        ),
        "EY402" => workshop(
            "Should our neighbourhood plant a shared garden?",
            "Write a persuasive piece with your position, two reasons, supporting evidence and a conclusion.",
            "Consider who will read it. A grown-up helps check whether your evidence supports your reasons and whether other views deserve a response.",
        ),
        "EY403" => cards(
            "Please put the basket over ___.",
            "Choose the correct homophone.",
            &["there", "their", "they're"],
            "Decide whether you need a place, possession or a shortened form of they are.",
        ),
        "EY404" => workshop(
            "Pip watered the seedlings. The soil was dry.",
            "Combine these ideas into a complex sentence. Choose a conjunction that explains the relationship.",
            "Try because, although or when, then check how each changes the meaning. A grown-up checks your independent and subordinate clauses.",
        ),
        "EY405" => cards(
            "The willow danced in the wind.",
            "Which language device gives the tree a human action?",
            &["personification", "simile", "literal description"],
            "Ask whether the writer gives a non-human thing a human quality.",
        ),
        "EY406" => workshop(
            "Draft: We went there. It was nice. We did stuff.",
            "Revise this into a clear, interesting account. Add precise details and organise your ideas.",
            "Ask a grown-up what is unclear. Revise, reread aloud and explain why your changes help the reader.",
        ),
        "EY501" => cards(
            "I held my breath as the bridge swayed. From here, I could not see whether Fern had reached safety.",
            "How does the first-person viewpoint build tension?",
            &[
                "We know only what the worried narrator knows",
                "It tells us every character's thoughts",
                "It removes uncertainty about Fern",
            ],
            "Consider what information the narrator can and cannot give the reader.",
        ),
        "EY502" => workshop(
            "Write an information or persuasive text about protecting a local habitat.",
            "Plan the structure, write a draft and refine it for a chosen audience.",
            "Use evidence, precise vocabulary and varied sentences. A grown-up reviews structure and clarity; revise in response to feedback.",
        ),
        "EY503" => cards(
            "The seedlings must be watered today.",
            "What does must suggest?",
            &[
                "a strong obligation",
                "a small possibility",
                "an action completed yesterday",
            ],
            "Compare must with might and could.",
        ),
        "EY504" => cards(
            "An advertisement claims a new seed grows twice as fast. It quotes one gardener and gives no growing conditions or comparison data.",
            "Which question best tests the argument?",
            &[
                "Were comparable seeds tested under the same conditions?",
                "Is the packet colourful?",
                "Does the gardener like flowers?",
            ],
            "Look for missing evidence and a fair comparison, rather than accepting the claim.",
        ),
        "EY505" => cards(
            "Dictionary: fragile (adjective) — easily broken or damaged.\nThesaurus: delicate, breakable.",
            "Which word best replaces fragile in 'Carry the fragile glass carefully'?",
            &["breakable", "heavy", "bright"],
            "Use the reference entry, then check the word in the sentence.",
        ),
        "EY601" => cards(
            "Report A: The visitors walked through the gate.\nReport B: A mob stormed through the gate.",
            "How does Report B position the reader?",
            &[
                "It presents the visitors as threatening",
                "It uses neutral wording",
                "It proves the visitors damaged the gate",
            ],
            "Compare loaded words with neutral descriptions. Wording alone is not evidence of damage.",
        ),
        "EY602" => workshop(
            "Choose an audience and a purpose for an extended story, explanation or persuasive text.",
            "Plan, draft, revise and edit a complete text. Share your polished version with a grown-up.",
            "Review structure, paragraph links, sentence choices, vocabulary, spelling and punctuation. Explain your revisions; a grown-up reviews the finished work.",
        ),
        "EY603" => cards(
            "Message to a friend: Hey, can you give me a hand?\nFormal request: Could you please assist me?",
            "Why do the messages use different language?",
            &[
                "They use different registers for different audiences",
                "Only the formal request has meaning",
                "Informal language is always incorrect",
            ],
            "Consider audience, purpose and context when choosing a register.",
        ),
        "EY604" => workshop(
            "Inquiry: Which plants would best support wildlife in our neighbourhood?",
            "Write a research question, compare at least two sources, take notes and present a supported answer.",
            "Record author, title, date and where each source was found. Distinguish source evidence from your conclusions. A grown-up reviews reliability and referencing.",
        ),
        "EY605" => workshop(
            "Discuss with a partner: How should we use a small shared garden space?",
            "Take turns, paraphrase your partner's idea, ask a question and build a shared proposal.",
            "Try: I heard you say … I see it differently because … Could we combine …? A grown-up observes listening, respectful disagreement and collaboration.",
        ),
        _ => return None,
    };
    Some(item)
}

pub(super) fn activity(skill: &str, seed: u64, id: &str) -> Result<MissionActivity, MissionError> {
    let item = item(skill, seed).ok_or(MissionError::UnsupportedSkill)?;
    let node = CurriculumGraph::load()
        .map_err(|_| MissionError::InvalidState)?
        .get(skill)
        .cloned()
        .ok_or(MissionError::UnsupportedSkill)?;
    // Card IDs encode position only. The key and authored correct answer stay server-side.
    let mut labels = item.answers.clone();
    if !labels.is_empty() {
        let len = labels.len();
        labels.rotate_left((seed % len as u64) as usize);
    }
    let choices: Vec<NamedItem> = labels
        .into_iter()
        .enumerate()
        .map(|(i, label)| NamedItem {
            id: format!("card-{i}"),
            label,
        })
        .collect();
    let key = if item.review {
        AssessmentKey::Review
    } else {
        AssessmentKey::Choice {
            correct_id: choices
                .iter()
                .find(|c| c.label == item.answers[0])
                .ok_or(MissionError::InvalidState)?
                .id
                .clone(),
            choice_ids: choices.iter().map(|c| c.id.clone()).collect(),
        }
    };
    let representation = node.allowed_representations[0].clone();
    let task = PublicTask::PracticeCards {
        stimulus: item.stimulus,
        choices,
        adult_review: item.review,
    };
    let fingerprint = canonical_fingerprint(skill, &task);
    Ok(MissionActivity {
        view: MissionActivityView {
            schema_version: "1.0".into(),
            id: id.into(),
            skill_id: skill.into(),
            content_version: "mission-curriculum-draft-2".into(),
            variant_id: format!("{skill}-practice-v2-{:02}", seed % 24),
            context_id: format!("{skill}-practice-context-{}", seed % 3),
            representation: representation.clone(),
            presentation: MissionPresentation {
                title: if item.review {
                    "Create and share together".into()
                } else {
                    "Try a little challenge".into()
                },
                objective: node.objective,
                lesson_text: item.hint,
                prompt: item.prompt,
                context: "A little practice for our next adventure.".into(),
                representation,
            },
            task,
        },
        key,
        canonical_fingerprint: fingerprint,
    })
}
