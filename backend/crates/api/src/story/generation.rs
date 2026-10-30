//! Arrival-time generation outside database locks. No identity or learner records
//! enter the prompt; validated results are committed with the travel receipt.
use super::*;
use wonderwilds_domain::story::exploration::place;
pub trait ExplorationProvider: Send + Sync {
    fn propose(&self, input: &Value) -> Result<Value, director::Error>;
}
pub fn input(state: &State) -> Result<Value, Error> {
    let p = place(state.location()).ok_or(Error::InvalidInput)?;
    let previous = state.current_neighbour();
    Ok(
        json!({"area":p.name,"visitStoryNumber":previous.map_or(1, |q| q.generation + 1),
        "previousStory":previous.and_then(|q| q.story.as_ref()).map(|s| json!({"name":s.content.name,"title":s.content.title,"payoff":s.content.payoff})),
        "capabilities":"One new character. One to three sequential look/listen/investigate tasks at reachable local objects, then return and share with the character. Each step may include learningBridge: an invitation to organise discovery markers for that investigation. The server supplies and assesses the preparation activity; do not invent its questions, quantities or outcome. No inventory, combat, currency, travel tasks or invented game commands.",
        "editorialBrief":"Ages 6–8, short warm sentences, concrete discoveries, gentle variety. No personal data, external links, threats, manipulation or unsafe activities."}),
    )
}
pub fn fallback(state: &State) -> Result<Value, Error> {
    let p = place(state.location()).ok_or(Error::InvalidInput)?;
    let n = state.current_neighbour().map_or(1, |q| q.generation + 1);
    let themes = [
        "a rustling leaf",
        "a hollow branch",
        "a smooth pebble",
        "a winding trail",
    ];
    let theme = themes[(n as usize - 1) % themes.len()];
    Ok(
        json!({"name":format!("{}{}",p.character[..1].to_uppercase(),&p.character[1..]),"personality":"A friendly neighbour who enjoys quiet discoveries.","sprite":"patter",
        "greeting":format!("Hello! Shall we investigate {theme} together?"),
        "title":format!("A little discovery · {n}"),"premise":format!("Explore {theme} in {} and share what you notice.",p.name),
        "steps":[{"label":format!("Look closely at {theme}."),"shape":"hollow"}],
        "payoff":"You and your neighbour shared a quiet discovery."}),
    )
}
