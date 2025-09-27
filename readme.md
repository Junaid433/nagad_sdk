# Nagad API (Rust Port)

A complete Rust translation of the original PHP Nagad payment gateway client. The crate exposes the same feature set (configuration management, checkout orchestration, helper utilities, and verification helpers) in an idiomatic Rust API.

## Overview
- Mirrors the PHP SDK flow: initialize checkout, submit order, receive callback URL, and verify payments.
- Uses `reqwest` for HTTP, `rsa` plus `sha2` for signing and encryption, and `chrono` for timestamp handling.
- Provides helper utilities for random invoice generation, callback parsing, and basic server metadata.

## Requirements
- Rust 1.70+ (edition 2021 or newer; crate is tested on 2024 edition toolchain).
- Access to Nagad sandbox or production credentials (merchant ID, account, private key, gateway public key, callback URL).
- The private and public keys must be supplied as Base64 encoded strings exactly as delivered by Nagad.

## Installation
Add the crate to your project. When developing locally you can depend on the checked-out path:

```bash
cargo add nagad_api --path /absolute/path/to/nagad_api
```

Or point Cargo to the Git repository directly:

```toml
# Cargo.toml
[dependencies]
nagad_api = { git = "https://github.com/your-org/nagad-api-rust.git", tag = "v0.1.0" }
```

## Configuration Model
Instantiate `BaseConfig` with the same values you would place in the original PHP `.env` file. All string values must be provided exactly (no surrounding quotes, already Base64 encoded keys).

| Field | Required | Description |
| ----- | -------- | ----------- |
| `nagad_app_env` | yes | `development` or `production` |
| `nagad_app_account` | optional | Merchant account number (optional in API) |
| `nagad_app_merchantid` | yes | Merchant ID assigned by Nagad |
| `nagad_app_merchant_private_key` | yes | Base64 RSA private key string |
| `nagad_app_merchant_pg_public_key` | yes | Base64 RSA public key string from Nagad |
| `nagad_app_timezone` | optional | Defaults to `Asia/Dhaka` |
| `nagad_app_currency_code` | optional | Defaults to `050` |

Parameters for a single payment request are modeled with `PaymentParams`:

| Field | Description |
| ----- | ----------- |
| `amount` | Amount as string formatted the way the Nagad API expects (for example `100.00`) |
| `invoice` | Unique invoice or order identifier |
| `merchant_callback` | HTTPS endpoint that receives Nagad redirect notifications |

## Typical Checkout Flow
1. Build `BaseConfig` and `PaymentParams` (optionally using helper utilities for invoices).
2. Create `NagadBase` with those values. The constructor validates required fields.
3. Call `pay_now_without_redirection()` to obtain the callback URL (or `pay_now()` if you plan to handle browser redirection in your environment).
4. Redirect the customer or embed the callback URL according to your client logic.
5. After the customer completes payment, call `verify_payment()` using the `paymentReferenceId` returned by Nagad to confirm status.

### Quick Start Example
```rust
use nagad_api::model::{BaseConfig, PaymentParams};
use nagad_api::nagad::NagadBase;
use nagad_api::utils::generate_fake_invoice;

fn main() -> nagad_api::error::Result<()> {
    let config = BaseConfig {
        nagad_app_env: "development".into(),
        nagad_app_account: Some("ACCOUNT".into()),
        nagad_app_merchantid: "MERCHANT_ID".into(),
        nagad_app_merchant_private_key: "BASE64_PRIVATE".into(),
        nagad_app_merchant_pg_public_key: "BASE64_PUBLIC".into(),
        nagad_app_timezone: Some("Asia/Dhaka".into()),
        nagad_app_currency_code: Some("050".into()),
    };

    let params = PaymentParams {
        amount: "100.00".into(),
        invoice: generate_fake_invoice(20, true, Some("INV-"), None),
        merchant_callback: "https://merchant.example.com/callback".into(),
    };

    let base = NagadBase::new(config, params)?;
    let callback_url = base.pay_now_without_redirection()?;
    println!("Present this URL to the customer: {}", callback_url);
    Ok(())
}
```

### Handling the Callback
When Nagad redirects the customer back to your callback endpoint, the redirect URL will include query parameters such as `payment_ref_id`, `status`, and `status_code`. Use `utils::success_response` to parse a full URL into a `HashMap<String, String>`:

```rust
use nagad_api::utils::success_response;

fn parse_callback(url: &str) {
    match success_response(url) {
        Ok(data) => println!("Payment status: {}", data.get("status").unwrap_or(&"unknown".into())),
        Err(err) => eprintln!("Failed to parse callback: {}", err),
    }
}
```

### Verifying a Payment
After you store the `paymentReferenceId` from the initial checkout response, verify it before marking an order as paid:

```rust
fn verify(base: &NagadBase, payment_reference_id: &str) {
    match base.verify_payment(payment_reference_id) {
        Ok(raw_response) => println!("Verification payload: {}", raw_response),
        Err(err) => eprintln!("Verification failed: {}", err),
    }
}
```

## Examples
`cargo run --example checkout` executes the complete checkout initialization showcased above. Duplicate the example and inject real credentials to perform an end-to-end sandbox transaction.

## Helper Utilities
The `utils` module includes several helpers:
- `generate_random_string` and `generate_fake_invoice` for testing or lightweight invoice management.
- `encrypt_with_public_key`, `decrypt_with_private_key`, and `sign_with_private_key` encapsulate RSA operations.
- `http_post` and `http_get` wrap `reqwest::blocking` requests with consistent error handling.
- `success_response` parses callback URLs.
- `client_ip` returns the best-effort client IP sourced from environment variables, mirroring the PHP implementation.

## Error Handling
Most functions return `nagad_api::error::Result<T>`, which aliases `Result<T, NagadError>`. Key variants include:
- `InvalidConfiguration` and `MissingParam` for configuration/parameter validation failures.
- `PublicKey` and `PrivateKey` when RSA operations fail.
- `Http`, `Json`, and `UrlParse` for transport and serialization problems.
- `Verification` for unexpected or failed status responses from Nagad.

Ensure you bubble these errors up to logs or monitoring systems; they surface the same conditions that were previously emitted by the PHP library.

## Differences from the PHP Library
- `pay_now` returns the redirect URL instead of emitting script-based browser redirects. Integrators should handle redirection manually.
- Logging is not automatically written to disk. Host applications can capture errors and log them using their preferred logger.
- `Helper::serverDetails` equivalents provide best-effort environment data but do not require server globals.

## Testing Strategy
- Use sandbox credentials supplied by Nagad for integration tests. The included example demonstrates how to stage requests.
- Consider wrapping calls in your own facade that can be mocked during unit testing; the library focuses on direct Nagad communication.

## Security Reminders
- Keep merchant private keys out of version control. Load them from secret storage or environment variables.
- Always use HTTPS callback endpoints.
- Nagad requires whitelisted IPs for production; verify whitelisting before switching `nagad_app_env` to `production`.

For additional scenarios (refunds, transaction queries) extend the crate following the same patterns as the PHP source or submit pull requests.
