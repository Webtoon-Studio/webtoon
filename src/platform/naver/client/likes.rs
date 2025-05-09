use serde::Deserialize;

#[derive(Deserialize)]
pub struct Likes {
    contents: Vec<Content>,
}

impl Likes {
    pub fn count(&self) -> u32 {
        let Some(content) = self.contents.first() else {
            return 0;
        };
        let Some(reaction) = content.reactions.first() else {
            return 0;
        };

        reaction.count
    }
}

#[derive(Deserialize)]
struct Content {
    reactions: Vec<Reaction>,
}

#[derive(Deserialize)]
struct Reaction {
    count: u32,
}
