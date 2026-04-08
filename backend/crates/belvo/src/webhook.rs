//! Webhook verification and handling for Belvo webhooks.

use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::error::{BelvoError, Result};
use crate::models::WebhookPayload;

type HmacSha256 = Hmac<Sha256>;

/// Verify a Belvo webhook signature.
///
/// # Arguments
/// * `secret` - The webhook secret from Belvo
/// * `payload` - The raw request body
/// * `signature` - The signature from the `belvo-signature` header
///
/// # Returns
/// * `Ok(true)` if the signature is valid
/// * `Err(BelvoError::InvalidWebhookSignature)` if the signature is invalid
pub fn verify_webhook_signature(secret: &str, payload: &[u8], signature: &str) -> Result<bool> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| BelvoError::InvalidWebhookSignature)?;

    mac.update(payload);

    let expected = hex::encode(mac.finalize().into_bytes());

    if expected == signature {
        Ok(true)
    } else {
        Err(BelvoError::InvalidWebhookSignature)
    }
}

/// Parse a webhook payload from JSON.
pub fn parse_webhook_payload(payload: &[u8]) -> Result<WebhookPayload> {
    serde_json::from_slice(payload).map_err(BelvoError::Json)
}

/// Verify and parse a webhook in one step.
///
/// # Arguments
/// * `secret` - The webhook secret from Belvo
/// * `payload` - The raw request body
/// * `signature` - The signature from the `belvo-signature` header
///
/// # Returns
/// * `Ok(WebhookPayload)` if valid
/// * `Err(BelvoError)` if verification or parsing fails
pub fn verify_and_parse_webhook(
    secret: &str,
    payload: &[u8],
    signature: &str,
) -> Result<WebhookPayload> {
    verify_webhook_signature(secret, payload, signature)?;
    parse_webhook_payload(payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_signature_verification() {
        let secret = "test_secret";
        let payload = b"{\"webhook_code\":\"link_created\",\"link_id\":\"123\"}";

        // Generate expected signature
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload);
        let signature = hex::encode(mac.finalize().into_bytes());

        // Verify
        let result = verify_webhook_signature(secret, payload, &signature);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_webhook_signature_invalid() {
        let secret = "test_secret";
        let payload = b"{\"webhook_code\":\"link_created\",\"link_id\":\"123\"}";
        let wrong_signature = "invalid_signature";

        let result = verify_webhook_signature(secret, payload, wrong_signature);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_webhook_payload_valid() {
        use crate::models::WebhookEventType;

        let payload = br#"{"webhook_code":"link_created","link_id":"550e8400-e29b-41d4-a716-446655440000","webhook_type":"LINK"}"#;

        let result = parse_webhook_payload(payload);
        assert!(result.is_ok());
        let webhook = result.unwrap();
        assert!(matches!(
            webhook.webhook_code,
            WebhookEventType::LinkCreated
        ));
    }

    #[test]
    fn test_parse_webhook_payload_invalid_json() {
        let payload = b"not valid json";

        let result = parse_webhook_payload(payload);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_and_parse_webhook_success() {
        use crate::models::WebhookEventType;

        let secret = "test_secret";
        let payload = br#"{"webhook_code":"link_created","link_id":"550e8400-e29b-41d4-a716-446655440000","webhook_type":"LINK"}"#;

        // Generate signature
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload);
        let signature = hex::encode(mac.finalize().into_bytes());

        let result = verify_and_parse_webhook(secret, payload, &signature);
        assert!(result.is_ok());
        let webhook = result.unwrap();
        assert!(matches!(
            webhook.webhook_code,
            WebhookEventType::LinkCreated
        ));
    }

    #[test]
    fn test_verify_and_parse_webhook_bad_signature() {
        let secret = "test_secret";
        let payload = br#"{"webhook_code":"link_created","link_id":"550e8400-e29b-41d4-a716-446655440000","webhook_type":"LINK"}"#;
        let wrong_signature = "bad_signature";

        let result = verify_and_parse_webhook(secret, payload, wrong_signature);
        assert!(result.is_err());
    }
}
