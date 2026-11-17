//! Versioned authenticated backup envelope. The header also separates its key
//! and ciphertext from other uses of the application's encryption primitives.
use super::*;

const MAGIC: &[u8] = b"DDNSUPD-BACKUP\x01";
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 24;

fn key(passphrase: &str, salt: &[u8]) -> Result<[u8; 32], SecretError> {
    let params = argon2::Params::new(65536, 3, 1, Some(32)).map_err(|_| SecretError)?;
    let mut key = [0; 32];
    Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params)
        .hash_password_into(passphrase.as_bytes(), salt, &mut key)
        .map_err(|_| SecretError)?;
    Ok(key)
}

pub fn seal(passphrase: &str, plaintext: &[u8]) -> Result<Vec<u8>, SecretError> {
    let mut salt = [0; SALT_LEN];
    let mut nonce = [0; NONCE_LEN];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut nonce);
    let ciphertext = XChaCha20Poly1305::new((&key(passphrase, &salt)?).into())
        .encrypt(
            XNonce::from_slice(&nonce),
            Payload {
                msg: plaintext,
                aad: MAGIC,
            },
        )
        .map_err(|_| SecretError)?;
    Ok([MAGIC, &salt, &nonce, &ciphertext].concat())
}

pub fn open(passphrase: &str, archive: &[u8]) -> Result<Vec<u8>, SecretError> {
    let body = archive.strip_prefix(MAGIC).ok_or(SecretError)?;
    if body.len() < SALT_LEN + NONCE_LEN + 16 {
        return Err(SecretError);
    }
    let (salt, body) = body.split_at(SALT_LEN);
    let (nonce, ciphertext) = body.split_at(NONCE_LEN);
    XChaCha20Poly1305::new((&key(passphrase, salt)?).into())
        .decrypt(
            XNonce::from_slice(nonce),
            Payload {
                msg: ciphertext,
                aad: MAGIC,
            },
        )
        .map_err(|_| SecretError)
}
