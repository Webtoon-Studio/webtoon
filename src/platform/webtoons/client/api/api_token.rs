use serde::Deserialize;

#[allow(unused)]
#[derive(Deserialize, Debug)]
pub struct ApiToken {
    pub status: String,
    pub result: Token,
}

#[derive(Deserialize, Debug)]
pub struct Token {
    pub token: String,
}
