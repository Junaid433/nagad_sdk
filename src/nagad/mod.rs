use crate::error::{NagadError, Result};
use crate::key::Key;
use crate::model::{BaseConfig, PaymentParams};
use crate::request::RequestHandler;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct NagadBase {
    environment: String,
    base_url: String,
    timezone: String,
    amount: String,
    invoice: String,
    merchant_id: String,
    merchant_callback: String,
    key: Key,
}

impl NagadBase {
    pub fn new(config: BaseConfig, params: PaymentParams) -> Result<Self> {
        let map = Self::config_to_map(&config);
        let key = Key::from_map(&map)?;
        Self::validate_params(&params)?;

        let amount = params.amount.clone();
        let invoice = params.invoice.clone();
        let merchant_callback = params.merchant_callback.clone();
        let merchant_id = key.merchant_id().to_string();
        let timezone = key.time_zone().to_string();
        let environment = if key.app_env() == "production" {
            "production".to_string()
        } else {
            "development".to_string()
        };
        let base_url = if environment == "production" {
            "https://api.mynagad.com/api/dfs/".to_string()
        } else {
            "http://sandbox.mynagad.com:10080/remote-payment-gateway-1.0/api/dfs/".to_string()
        };

        Ok(Self {
            environment,
            base_url,
            timezone,
            amount,
            invoice,
            merchant_id,
            merchant_callback,
            key,
        })
    }

    fn config_to_map(config: &BaseConfig) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("NAGAD_APP_ENV".to_string(), config.nagad_app_env.clone());
        if let Some(account) = &config.nagad_app_account {
            map.insert("NAGAD_APP_ACCOUNT".to_string(), account.clone());
        }
        map.insert(
            "NAGAD_APP_MERCHANTID".to_string(),
            config.nagad_app_merchantid.clone(),
        );
        map.insert(
            "NAGAD_APP_MERCHANT_PRIVATE_KEY".to_string(),
            config.nagad_app_merchant_private_key.clone(),
        );
        map.insert(
            "NAGAD_APP_MERCHANT_PG_PUBLIC_KEY".to_string(),
            config.nagad_app_merchant_pg_public_key.clone(),
        );
        if let Some(tz) = &config.nagad_app_timezone {
            map.insert("NAGAD_APP_TIMEZONE".to_string(), tz.clone());
        }
        if let Some(currency) = &config.nagad_app_currency_code {
            map.insert("NAGAD_APP_CURRENCY_CODE".to_string(), currency.clone());
        }
        map
    }

    fn validate_params(params: &PaymentParams) -> Result<()> {
        if params.amount.is_empty() {
            return Err(NagadError::MissingParam("amount"));
        }
        if params.invoice.is_empty() {
            return Err(NagadError::MissingParam("invoice"));
        }
        if params.merchant_callback.is_empty() {
            return Err(NagadError::MissingParam("merchantCallback"));
        }
        Ok(())
    }

    pub fn timezone(&self) -> &str {
        &self.timezone
    }

    pub fn amount(&self) -> &str {
        &self.amount
    }

    pub fn invoice(&self) -> &str {
        &self.invoice
    }

    pub fn merchant_id(&self) -> &str {
        &self.merchant_id
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn environment(&self) -> &str {
        &self.environment
    }

    pub fn merchant_callback(&self) -> &str {
        &self.merchant_callback
    }

    pub fn key(&self) -> &Key {
        &self.key
    }

    pub fn pay_now(&self) -> Result<String> {
        let handler = RequestHandler::new(self.clone());
        handler.send_request(true)
    }

    pub fn pay_now_without_redirection(&self) -> Result<String> {
        let handler = RequestHandler::new(self.clone());
        handler.send_request(false)
    }

    pub fn verify_payment(&self, payment_ref_id: &str) -> Result<String> {
        let mut base_url =
            "http://sandbox.mynagad.com:10080/remote-payment-gateway-1.0/api/dfs/".to_string();
        if self.key.app_env() == "production" {
            base_url = "https://api.mynagad.com/api/dfs/".to_string();
        }
        let url = format!("{}verify/payment/{}", base_url, payment_ref_id);
        crate::utils::http_get(&url)
    }
}
