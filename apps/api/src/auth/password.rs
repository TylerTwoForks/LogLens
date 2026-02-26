use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use sha2::{Digest, Sha256};

use crate::error::ApiError;

pub fn hash_password(plain: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(plain.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| ApiError::internal(format!("failed to hash password: {e}")))
}

pub fn verify_password(plain: &str, hash: &str) -> Result<bool, ApiError> {
    let parsed = PasswordHash::new(hash)
        .map_err(|e| ApiError::internal(format!("failed to parse password hash: {e}")))?;
    Ok(Argon2::default()
        .verify_password(plain.as_bytes(), &parsed)
        .is_ok())
}

/// Generate auth_subject from email, matching the Next.js `hashEmailToSubject`
/// implementation: `user_${SHA256(email).hex().slice(0, 24)}`.
///
/// IMPORTANT: email must be trimmed + lowercased before calling this,
/// matching the `createSessionFromEmail` behavior in apps/web/lib/auth.ts.
pub fn hash_email_to_subject(email: &str) -> String {
    let digest = Sha256::digest(email.as_bytes());
    let hex_str = hex::encode(digest);
    format!("user_{}", &hex_str[..24])
}

pub fn validate_email(email: &str) -> Result<(), ApiError> {
    if email.len() > 254 {
        return Err(ApiError::bad_request("email too long"));
    }
    let parts: Vec<&str> = email.splitn(2, '@').collect();
    if parts.len() != 2 || parts[0].is_empty() || !parts[1].contains('.') {
        return Err(ApiError::bad_request("invalid email format"));
    }
    Ok(())
}

pub fn validate_password(password: &str) -> Result<(), ApiError> {
    if password.len() < 8 {
        return Err(ApiError::bad_request(
            "password must be at least 8 characters",
        ));
    }
    if password.len() > 128 {
        return Err(ApiError::bad_request(
            "password must be at most 128 characters",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify_roundtrip() {
        let hash = hash_password("testpass123").unwrap();
        assert!(verify_password("testpass123", &hash).unwrap());
        assert!(!verify_password("wrongpass", &hash).unwrap());
    }

    #[test]
    fn hash_email_matches_typescript_implementation() {
        // Verified against Node.js:
        // createHash("sha256").update("test@example.com").digest("hex").slice(0, 24)
        // = "973dfe463ec85785f5f95af5"
        let subject = hash_email_to_subject("test@example.com");
        assert_eq!(subject, "user_973dfe463ec85785f5f95af5");
    }

    #[test]
    fn validate_email_rejects_invalid() {
        assert!(validate_email("good@example.com").is_ok());
        assert!(validate_email("noatsign").is_err());
        assert!(validate_email("@nodomain.com").is_err());
        assert!(validate_email("no@dotindomain").is_err());
    }

    #[test]
    fn validate_password_enforces_length() {
        assert!(validate_password("short").is_err());
        assert!(validate_password("longenough").is_ok());
        assert!(validate_password(&"x".repeat(129)).is_err());
        assert!(validate_password(&"x".repeat(128)).is_ok());
    }
}
