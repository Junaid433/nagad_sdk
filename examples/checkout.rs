use nagad_api::model::{BaseConfig, PaymentParams};
use nagad_api::nagad::NagadBase;
use nagad_api::utils::generate_fake_invoice;

fn main() {
    let config = BaseConfig {
        nagad_app_env: "development".to_string(),
        nagad_app_account: Some("YOUR_ACCOUNT".to_string()),
        nagad_app_merchantid: "YOUR_MERCHANT_ID".to_string(),
        nagad_app_merchant_private_key: "BASE64_PRIVATE_KEY".to_string(),
        nagad_app_merchant_pg_public_key: "BASE64_PUBLIC_KEY".to_string(),
        nagad_app_timezone: Some("Asia/Dhaka".to_string()),
        nagad_app_currency_code: Some("050".to_string()),
    };

    let params = PaymentParams {
        amount: "100.00".to_string(),
        invoice: generate_fake_invoice(20, true, Some("INV-"), None),
        merchant_callback: "https://merchant.example.com/callback".to_string(),
    };

    match NagadBase::new(config, params) {
        Ok(base) => match base.pay_now_without_redirection() {
            Ok(callback_url) => println!("Callback URL: {}", callback_url),
            Err(err) => eprintln!("Request error: {}", err),
        },
        Err(err) => eprintln!("Configuration error: {}", err),
    }
}
