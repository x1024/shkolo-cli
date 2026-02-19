use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleAuthRequest {
    #[serde(rename = "idToken")]
    pub id_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub cached: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_at: Option<String>,
    pub data: T,
}

impl<T> ApiResponse<T> {
    pub fn new(data: T, cached: bool, cached_at: Option<String>) -> Self {
        Self {
            success: true,
            cached,
            cached_at,
            data,
        }
    }
}

