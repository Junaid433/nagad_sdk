use crate::error::{NagadError, Result};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct KeyConfig {
    pub app_env: Option<String>,
    pub app_account: Option<String>,
    pub app_merchant_id: Option<String>,
    pub merchant_private_key: Option<String>,
    pub pg_public_key: Option<String>,
    pub time_zone: Option<String>,
    pub currency_code: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Key {
    app_env: String,
    app_account: Option<String>,
    app_merchant_id: String,
    merchant_private_key: String,
    pg_public_key: String,
    time_zone: String,
    currency_code: String,
}

impl Key {
    pub fn from_map(config: &HashMap<String, String>) -> Result<Self> {
        let app_env = config
            .get("NAGAD_APP_ENV")
            .cloned()
            .ok_or(NagadError::InvalidConfiguration)?;
        let app_merchant_id = config
            .get("NAGAD_APP_MERCHANTID")
            .cloned()
            .ok_or(NagadError::InvalidConfiguration)?;
        let merchant_private_key = config
            .get("NAGAD_APP_MERCHANT_PRIVATE_KEY")
            .cloned()
            .ok_or(NagadError::InvalidConfiguration)?;
        let pg_public_key = config
            .get("NAGAD_APP_MERCHANT_PG_PUBLIC_KEY")
            .cloned()
            .ok_or(NagadError::InvalidConfiguration)?;

        let app_account = config.get("NAGAD_APP_ACCOUNT").cloned();
        let time_zone = config
            .get("NAGAD_APP_TIMEZONE")
            .cloned()
            .unwrap_or_else(|| "Asia/Dhaka".to_string());
        let currency_code = config
            .get("NAGAD_APP_CURRENCY_CODE")
            .cloned()
            .unwrap_or_else(|| "050".to_string());

        Ok(Self {
            app_env,
            app_account,
            app_merchant_id,
            merchant_private_key,
            pg_public_key,
            time_zone,
            currency_code,
        })
    }

    pub fn from_config(config: &KeyConfig) -> Result<Self> {
        let app_env = config
            .app_env
            .clone()
            .ok_or(NagadError::InvalidConfiguration)?;
        let app_merchant_id = config
            .app_merchant_id
            .clone()
            .ok_or(NagadError::InvalidConfiguration)?;
        let merchant_private_key = config
            .merchant_private_key
            .clone()
            .ok_or(NagadError::InvalidConfiguration)?;
        let pg_public_key = config
            .pg_public_key
            .clone()
            .ok_or(NagadError::InvalidConfiguration)?;

        let app_account = config.app_account.clone();
        let time_zone = config
            .time_zone
            .clone()
            .unwrap_or_else(|| "Asia/Dhaka".to_string());
        let currency_code = config
            .currency_code
            .clone()
            .unwrap_or_else(|| "050".to_string());

        Ok(Self {
            app_env,
            app_account,
            app_merchant_id,
            merchant_private_key,
            pg_public_key,
            time_zone,
            currency_code,
        })
    }

    pub fn app_env(&self) -> &str {
        &self.app_env
    }

    pub fn app_account(&self) -> Option<&str> {
        self.app_account.as_deref()
    }

    pub fn merchant_id(&self) -> &str {
        &self.app_merchant_id
    }

    pub fn merchant_private_key(&self) -> &str {
        &self.merchant_private_key
    }

    pub fn pg_public_key(&self) -> &str {
        &self.pg_public_key
    }

    pub fn time_zone(&self) -> &str {
        &self.time_zone
    }

    pub fn currency_code(&self) -> &str {
        &self.currency_code
    }
}
