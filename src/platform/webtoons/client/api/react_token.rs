use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ReactToken {
    pub result: ReactResult,
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactResult {
    pub guest_token: Option<String>,
    pub timestamp: Option<i64>,
    pub status_code: Option<u16>,
}
