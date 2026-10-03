use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailClaims {
    pub sub: i64,
    pub email: String,
    pub exp: usize,
}

pub(crate) fn sign_email_token(config: &crate::config::Config, user_id: i64, email: &str) -> String {
    let exp = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        + 7 * 86400) as usize;
    let claims = EmailClaims {
        sub: user_id,
        email: email.to_string(),
        exp,
    };
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .unwrap_or_default()
}

pub fn verify_email_token(config: &crate::config::Config, token: &str) -> Option<EmailClaims> {
    jsonwebtoken::decode::<EmailClaims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    )
    .ok()
    .map(|d| d.claims)
}
