use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentParams {
    pub amount: String,
    pub invoice: String,
    pub merchant_callback: String,
}
