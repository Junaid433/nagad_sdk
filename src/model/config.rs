use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseConfig {
    pub nagad_app_env: String,
    pub nagad_app_account: Option<String>,
    pub nagad_app_merchantid: String,
    pub nagad_app_merchant_private_key: String,
    pub nagad_app_merchant_pg_public_key: String,
    pub nagad_app_timezone: Option<String>,
    pub nagad_app_currency_code: Option<String>,
}
