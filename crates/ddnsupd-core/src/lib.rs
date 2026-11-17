use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit, Payload},
};
use rand::{RngCore, rngs::OsRng};
use sha2::{Digest, Sha256};
use std::net::IpAddr;
use uuid::Uuid;

pub mod backup;

#[derive(Debug, thiserror::Error)]
#[error("secret operation failed")]
pub struct SecretError;

pub fn token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}
pub fn digest(value: &str) -> String {
    hex::encode(Sha256::digest(value.as_bytes()))
}
pub fn hash_password(value: &str) -> Result<String, SecretError> {
    Argon2::default()
        .hash_password(value.as_bytes(), &SaltString::generate(&mut OsRng))
        .map(|v| v.to_string())
        .map_err(|_| SecretError)
}
pub fn verify_password(hash: &str, value: &str) -> bool {
    PasswordHash::new(hash).is_ok_and(|hash| {
        Argon2::default()
            .verify_password(value.as_bytes(), &hash)
            .is_ok()
    })
}
pub fn encrypt(key: &[u8; 32], provider: Uuid, value: &str) -> Result<Vec<u8>, SecretError> {
    let mut nonce = [0u8; 24];
    OsRng.fill_bytes(&mut nonce);
    let cipher = XChaCha20Poly1305::new(key.into());
    let encrypted = cipher
        .encrypt(
            XNonce::from_slice(&nonce),
            Payload {
                msg: value.as_bytes(),
                aad: provider.as_bytes(),
            },
        )
        .map_err(|_| SecretError)?;
    Ok([nonce.to_vec(), encrypted].concat())
}
pub fn decrypt(key: &[u8; 32], provider: Uuid, value: &[u8]) -> Result<String, SecretError> {
    if value.len() < 40 {
        return Err(SecretError);
    }
    let bytes = XChaCha20Poly1305::new(key.into())
        .decrypt(
            XNonce::from_slice(&value[..24]),
            Payload {
                msg: &value[24..],
                aad: provider.as_bytes(),
            },
        )
        .map_err(|_| SecretError)?;
    String::from_utf8(bytes).map_err(|_| SecretError)
}
pub fn valid_name(value: &str, wildcard: bool) -> bool {
    let value = if wildcard {
        value.strip_prefix("*.").unwrap_or(value)
    } else {
        value
    };
    value.len() <= 253
        && !value.is_empty()
        && value.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        })
}
pub fn public_ip(value: &str, record_type: &str) -> bool {
    let Ok(ip) = value.parse::<IpAddr>() else {
        return false;
    };
    match ip {
        IpAddr::V4(ip) => {
            let n = u32::from(ip);
            let blocked = [
                (0, 8),
                (0x0a000000, 8),
                (0x64400000, 10),
                (0x7f000000, 8),
                (0xa9fe0000, 16),
                (0xac100000, 12),
                (0xc0000000, 24),
                (0xc0000200, 24),
                (0xc0a80000, 16),
                (0xc6120000, 15),
                (0xc6336400, 24),
                (0xcb007100, 24),
                (0xe0000000, 3),
            ];
            record_type == "A"
                && !blocked
                    .iter()
                    .any(|(base, bits)| n >> (32 - bits) == base >> (32 - bits))
        }
        IpAddr::V6(ip) => {
            let segments = ip.segments();
            record_type == "AAAA"
                && segments[0] & 0xe000 == 0x2000
                && !(segments[0] == 0x2001 && (segments[1] < 0x200 || segments[1] == 0xdb8))
                && !(segments[0] == 0x3fff && segments[1] & 0xf000 == 0)
                && segments[0] != 0x2002
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encryption_is_bound_and_authenticated() {
        let key = [7; 32];
        let id = Uuid::new_v4();
        let encrypted = encrypt(&key, id, "secret").unwrap();
        assert_eq!(decrypt(&key, id, &encrypted).unwrap(), "secret");
        assert!(decrypt(&[8; 32], id, &encrypted).is_err());
        assert!(decrypt(&key, Uuid::new_v4(), &encrypted).is_err());
        let mut bad = encrypted.clone();
        bad[30] ^= 1;
        assert!(decrypt(&key, id, &bad).is_err());
        assert_ne!(encrypt(&key, id, "secret").unwrap(), encrypted);
    }
    #[test]
    fn passwords_are_argon2id() {
        let hash = hash_password("long-test-password").unwrap();
        assert!(hash.starts_with("$argon2id$"));
        assert!(verify_password(&hash, "long-test-password"));
        assert!(!verify_password(&hash, "wrong"));
    }
    #[test]
    fn reject_nonpublic_and_wrong_family() {
        for ip in [
            "10.0.0.1",
            "127.0.0.1",
            "100.64.0.1",
            "192.0.2.1",
            "198.51.100.1",
            "203.0.113.1",
            "224.0.0.1",
            "0.0.0.0",
            "garbage",
            "::ffff:8.8.8.8",
        ] {
            assert!(!public_ip(ip, "A"), "{ip}");
        }
        assert!(public_ip("8.8.8.8", "A"));
        assert!(public_ip("2606:4700:4700::1111", "AAAA"));
        assert!(!public_ip("2001:db8::1", "AAAA"));
        assert!(!public_ip("8.8.8.8", "AAAA"));
    }
    #[test]
    fn names_cannot_escape_zones() {
        assert!(valid_name("home.example.org", false));
        assert!(valid_name("*.example.org", true));
        for name in [
            "../other",
            "evil/name",
            "EXAMPLE.org",
            "-bad.example",
            "bad..example",
        ] {
            assert!(!valid_name(name, true));
        }
    }
}
pub mod config_import;
