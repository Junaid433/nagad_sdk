use crate::error::{NagadError, Result};
use crate::nagad::NagadBase;
use crate::utils::{
    client_ip, current_timestamp_ymdhis, decrypt_with_private_key, encrypt_with_public_key,
    http_post, sign_with_private_key,
};
use serde_json::{Value, json};
use std::collections::HashMap;

pub struct RequestHandler {
    base: NagadBase,
    api_url: String,
}

impl RequestHandler {
    pub fn new(base: NagadBase) -> Self {
        Self {
            base,
            api_url: "check-out/initialize/".to_string(),
        }
    }

    pub fn send_request(&self, redirection: bool) -> Result<String> {
        let post_url = format!(
            "{}{}{}/{}",
            self.base.base_url(),
            self.api_url,
            self.base.merchant_id(),
            self.base.invoice()
        );

        let sensitive_data = json!({
            "merchantId": self.base.key().merchant_id(),
            "datetime": current_timestamp_ymdhis()?,
            "orderId": self.base.invoice(),
            "challenge": crate::utils::generate_random_string(40, Some("you"), Some("me")),
        });

        let sensitive_json =
            serde_json::to_string(&sensitive_data).map_err(|e| NagadError::Json(e.to_string()))?;

        let public_signature =
            encrypt_with_public_key(self.base.key().pg_public_key(), sensitive_json.as_bytes())?;

        let signature = sign_with_private_key(
            self.base.key().merchant_private_key(),
            sensitive_json.as_bytes(),
        )?;

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("X-KM-Api-Version".to_string(), "v-0.2.0".to_string());
        headers.insert("X-KM-IP-V4".to_string(), client_ip());
        headers.insert("X-KM-Client-Type".to_string(), "PC_WEB".to_string());

        let post_body = json!({
            "accountNumber": self.base.key().app_account(),
            "dateTime": current_timestamp_ymdhis()?,
            "sensitiveData": public_signature,
            "signature": signature,
        });

        let response = http_post(&post_url, &post_body, Some(&headers))?;

        if !response.is_object() {
            return Err(NagadError::Http("null response".to_string()));
        }

        if let Some(error) = response.get("error") {
            return Err(NagadError::Http(error.to_string()));
        }

        if let Some(reason) = response.get("reason") {
            return Err(NagadError::Http(reason.to_string()));
        }

        self.handle_response(response, redirection)
    }

    fn handle_response(&self, response: Value, _redirection: bool) -> Result<String> {
        let sensitive_data = response
            .get("sensitiveData")
            .and_then(|v| v.as_str())
            .ok_or_else(|| NagadError::Verification("missing sensitiveData".to_string()))?;
        let _signature = response
            .get("signature")
            .and_then(|v| v.as_str())
            .ok_or_else(|| NagadError::Verification("missing signature".to_string()))?;

        let plain_response =
            decrypt_with_private_key(self.base.key().merchant_private_key(), sensitive_data)?;
        let plain_json: Value =
            serde_json::from_str(&plain_response).map_err(|e| NagadError::Json(e.to_string()))?;

        let payment_reference_id = plain_json
            .get("paymentReferenceId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| NagadError::Verification("missing paymentReferenceId".to_string()))?;
        let challenge = plain_json
            .get("challenge")
            .and_then(|v| v.as_str())
            .ok_or_else(|| NagadError::Verification("missing challenge".to_string()))?;

        let order_sensitive = json!({
            "merchantId": self.base.merchant_id(),
            "orderId": self.base.invoice(),
            "currencyCode": self.base.key().currency_code(),
            "amount": self.base.amount(),
            "challenge": challenge,
        });

        let order_json =
            serde_json::to_string(&order_sensitive).map_err(|e| NagadError::Json(e.to_string()))?;

        let order_post = json!({
            "sensitiveData": encrypt_with_public_key(self.base.key().pg_public_key(), order_json.as_bytes())?,
            "signature": sign_with_private_key(self.base.key().merchant_private_key(), order_json.as_bytes())?,
            "merchantCallbackURL": self.base.merchant_callback(),
        });

        let order_url = format!(
            "{}check-out/complete/{}",
            self.base.base_url(),
            payment_reference_id
        );

        let order_response = http_post(&order_url, &order_post, Some(&default_headers()))?;

        if let Some(status) = order_response.get("status").and_then(|v| v.as_str()) {
            if status == "Success" {
                if let Some(url) = order_response.get("callBackUrl").and_then(|v| v.as_str()) {
                    return Ok(url.to_string());
                }
            }
            return Err(NagadError::Verification(format!("status: {}", status)));
        }

        if order_response.get("message").is_some() {
            return Err(NagadError::Verification(
                order_response
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown error")
                    .to_string(),
            ));
        }

        Err(NagadError::Verification("unexpected response".to_string()))
    }
}

fn default_headers() -> HashMap<String, String> {
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("X-KM-Api-Version".to_string(), "v-0.2.0".to_string());
    headers.insert("X-KM-IP-V4".to_string(), client_ip());
    headers.insert("X-KM-Client-Type".to_string(), "PC_WEB".to_string());
    headers
}
