use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutResponse {
    pub status: String,
    pub call_back_url: Option<String>,
    pub message: Option<String>,
    pub sensitive_data: Option<String>,
    pub signature: Option<String>,
    pub payment_reference_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub status: String,
    pub response: String,
    pub request: HashMap<String, serde_json::Value>,
    pub server: HashMap<String, String>,
}
