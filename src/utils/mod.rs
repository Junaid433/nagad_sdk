use crate::error::{NagadError, Result};
use base64::{Engine, engine::general_purpose::STANDARD};
use rand::{Rng, distributions::Alphanumeric};
use rsa::signature::{RandomizedSigner, SignatureEncoding};
use rsa::{
    Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey, pkcs1::DecodeRsaPrivateKey, pkcs1v15::SigningKey,
    pkcs8::DecodePublicKey,
};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

pub fn generate_random_string(length: usize, prefix: Option<&str>, suffix: Option<&str>) -> String {
    let mut rng = rand::thread_rng();
    let random: String = (0..length)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect();
    let mut result = String::new();
    if let Some(pre) = prefix {
        result.push_str(pre);
    }
    result.push_str(&random);
    if let Some(suf) = suffix {
        result.push_str(suf);
    }
    result
}

pub fn generate_fake_invoice(
    length: usize,
    capitalize: bool,
    prefix: Option<&str>,
    suffix: Option<&str>,
) -> String {
    let invoice = generate_random_string(length, prefix, suffix);
    if capitalize {
        invoice.to_uppercase()
    } else {
        invoice
    }
}

pub fn encrypt_with_public_key(public_key_base64: &str, data: &[u8]) -> Result<String> {
    let pem = format!(
        "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----",
        public_key_base64
    );
    let public_key = RsaPublicKey::from_public_key_pem(&pem)
        .map_err(|e| NagadError::PublicKey(e.to_string()))?;
    let padding = Pkcs1v15Encrypt;
    let mut rng = rand::thread_rng();
    let encrypted = public_key
        .encrypt(&mut rng, padding, data)
        .map_err(|e| NagadError::PublicKey(e.to_string()))?;
    Ok(STANDARD.encode(encrypted))
}

pub fn decrypt_with_private_key(private_key_base64: &str, data: &str) -> Result<String> {
    let pem = format!(
        "-----BEGIN RSA PRIVATE KEY-----\n{}\n-----END RSA PRIVATE KEY-----",
        private_key_base64
    );
    let private_key =
        RsaPrivateKey::from_pkcs1_pem(&pem).map_err(|e| NagadError::PrivateKey(e.to_string()))?;
    let cipher = STANDARD
        .decode(data)
        .map_err(|e| NagadError::PrivateKey(e.to_string()))?;
    let padding = Pkcs1v15Encrypt;
    let decrypted = private_key
        .decrypt(padding, &cipher)
        .map_err(|e| NagadError::PrivateKey(e.to_string()))?;
    String::from_utf8(decrypted).map_err(|e| NagadError::PrivateKey(e.to_string()))
}

pub fn sign_with_private_key(private_key_base64: &str, data: &[u8]) -> Result<String> {
    use sha2::Sha256;

    let pem = format!(
        "-----BEGIN RSA PRIVATE KEY-----\n{}\n-----END RSA PRIVATE KEY-----",
        private_key_base64
    );
    let private_key =
        RsaPrivateKey::from_pkcs1_pem(&pem).map_err(|e| NagadError::PrivateKey(e.to_string()))?;
    let signing_key = SigningKey::<Sha256>::new(private_key);
    let mut rng = rand::thread_rng();
    let signature = signing_key.sign_with_rng(&mut rng, data).to_bytes();
    Ok(STANDARD.encode(signature))
}

pub fn http_post(
    url: &str,
    payload: &Value,
    headers: Option<&HashMap<String, String>>,
) -> Result<Value> {
    let client = reqwest::blocking::Client::new();
    let mut req = client.post(url).json(payload);
    if let Some(hdrs) = headers {
        for (key, value) in hdrs {
            req = req.header(key, value);
        }
    }
    let resp = req.send().map_err(|e| NagadError::Http(e.to_string()))?;
    let status = resp.status();
    let text = resp.text().map_err(|e| NagadError::Http(e.to_string()))?;
    if !status.is_success() {
        return Err(NagadError::Http(format!("{}: {}", status, text)));
    }
    serde_json::from_str(&text).map_err(|e| NagadError::Json(e.to_string()))
}

pub fn http_get(url: &str) -> Result<String> {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(url)
        .send()
        .map_err(|e| NagadError::Http(e.to_string()))?;
    let status = resp.status();
    let text = resp.text().map_err(|e| NagadError::Http(e.to_string()))?;
    if !status.is_success() {
        return Err(NagadError::Http(format!("{}: {}", status, text)));
    }
    Ok(text)
}

pub fn server_details() -> HashMap<String, String> {
    let mut details = HashMap::new();
    details.insert(
        "base".to_string(),
        env::var("SERVER_ADDR").unwrap_or_default(),
    );
    details.insert(
        "ip".to_string(),
        env::var("REMOTE_ADDR").unwrap_or_default(),
    );
    details.insert(
        "port".to_string(),
        env::var("REMOTE_PORT").unwrap_or_default(),
    );
    details.insert(
        "request_url".to_string(),
        env::var("REQUEST_URI").unwrap_or_default(),
    );
    details.insert(
        "user agent".to_string(),
        env::var("HTTP_USER_AGENT").unwrap_or_default(),
    );
    details
}

pub fn success_response(url: &str) -> Result<HashMap<String, String>> {
    let parsed = Url::parse(url).map_err(|e| NagadError::UrlParse(e.to_string()))?;
    let mut map = HashMap::new();
    if let Some(query) = parsed.query() {
        let parsed_query: HashMap<String, String> =
            serde_urlencoded::from_str(query).map_err(|e| NagadError::UrlParse(e.to_string()))?;
        map.extend(parsed_query);
    }
    Ok(map)
}

pub fn current_timestamp_ymdhis() -> Result<String> {
    let now = chrono::Local::now();
    Ok(now.format("%Y%m%d%H%M%S").to_string())
}

pub fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}

pub fn json_value<T: Serialize>(value: &T) -> Result<Value> {
    serde_json::to_value(value).map_err(|e| NagadError::Json(e.to_string()))
}

pub fn client_ip() -> String {
    if let Ok(ip) = env::var("HTTP_CLIENT_IP") {
        if !ip.is_empty() {
            return ip;
        }
    }
    if let Ok(ip) = env::var("HTTP_X_FORWARDED_FOR") {
        if !ip.is_empty() {
            return ip;
        }
    }
    if let Ok(ip) = env::var("HTTP_X_FORWARDED") {
        if !ip.is_empty() {
            return ip;
        }
    }
    if let Ok(ip) = env::var("HTTP_FORWARDED_FOR") {
        if !ip.is_empty() {
            return ip;
        }
    }
    if let Ok(ip) = env::var("HTTP_FORWARDED") {
        if !ip.is_empty() {
            return ip;
        }
    }
    if let Ok(ip) = env::var("REMOTE_ADDR") {
        if !ip.is_empty() {
            return ip;
        }
    }
    "UNKNOWN IP".to_string()
}
