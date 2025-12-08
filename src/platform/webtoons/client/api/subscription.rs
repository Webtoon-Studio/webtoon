#[derive(Debug, serde::Deserialize)]
pub struct Response {
    pub success: bool,
    pub favorite: bool,
}
