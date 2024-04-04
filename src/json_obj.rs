use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenResponse {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    pub token: String,
    #[serde(rename = "subject")]
    pub puuid: String
}