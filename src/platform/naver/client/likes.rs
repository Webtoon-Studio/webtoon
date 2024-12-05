use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::platform::naver) struct Response {
    pub contents: Vec<Reactions>,
    pub parent_contents: Vec<ParentReactions>,
}

#[derive(Deserialize)]
pub(in crate::platform::naver) struct Reactions {
    pub reactions: Vec<Count>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::platform::naver) struct ParentReactions {
    pub parent_reactions: Vec<Count>,
}

#[derive(Deserialize)]
pub(in crate::platform::naver) struct Count {
    pub count: u32,
}
