use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct RawLikesResponse {
    #[serde(alias = "contents")]
    pub result: Contents,
}

#[derive(Deserialize, Debug)]
pub struct Contents {
    pub contents: Vec<Reaction>,
}

#[derive(Deserialize, Debug)]
pub struct Reaction {
    #[serde(alias = "reactions")]
    pub reactions: Vec<Count>,
}

#[derive(Deserialize, Debug)]
pub struct Count {
    #[serde(alias = "count")]
    pub count: u32,
}
