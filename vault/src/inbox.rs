//! Saving logins while Sompio is locked. The vault holds a secret "inbox" key; only its public half
//! is kept outside. With the public half, a new login can be locked away (X25519 + HKDF-SHA256 +
//! ChaCha20-Poly1305, a fresh key pair for every login), but nothing can be read: not the vault,
//! and not even the logins just added. They move into the vault the next time it's opened.

use crate::browser::{host_of, match_score};
use crate::store::Vault;
use anyhow::{anyhow, bail, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use chacha20poly1305::{aead::Aead, ChaCha20Poly1305, KeyInit, Nonce};
use curve25519_dalek::montgomery::MontgomeryPoint;
use hkdf::Hkdf;
use keepass::db::{CustomDataItem, CustomDataValue};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use zeroize::{Zeroize, Zeroizing};

pub const INBOX_KEY: &str = "Sompio.InboxKey";
const INFO: &[u8] = b"sompio-inbox-v1";

/// A login saved while the vault was locked.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InboxItem {
    pub url: String,
    pub title: String,
    pub username: String,
    pub password: String,
    /// Unix ms
    pub at: i64,
}

impl Drop for InboxItem {
    fn drop(&mut self) {
        self.password.zeroize();
    }
}

/// What moving the inbox into the vault did.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct InboxMerge {
    pub added: usize,
    pub updated: usize,
    /// Lines that couldn't be opened (damaged, or locked for another vault)
    pub skipped: usize,
}

#[derive(Serialize, Deserialize)]
struct Sealed {
    v: u8,
    /// The one-time public key
    e: String,
    n: String,
    d: String,
}

fn cipher(shared: &[u8; 32], epk: &[u8; 32], pk: &[u8; 32]) -> ChaCha20Poly1305 {
    let mut salt = [0u8; 64];
    salt[..32].copy_from_slice(epk);
    salt[32..].copy_from_slice(pk);
    let mut key = Zeroizing::new([0u8; 32]);
    Hkdf::<Sha256>::new(Some(&salt), shared).expand(INFO, key.as_mut()).expect("32 bytes is a valid length");
    ChaCha20Poly1305::new(key.as_ref().into())
}

/// Locks a login away for the vault with this public key. One line of text.
pub fn seal(public: &[u8; 32], item: &InboxItem) -> Result<String> {
    let mut esk = Zeroizing::new([0u8; 32]);
    rand::thread_rng().fill_bytes(esk.as_mut());
    let epk = MontgomeryPoint::mul_base_clamped(*esk).to_bytes();
    let shared = Zeroizing::new(MontgomeryPoint(*public).mul_clamped(*esk).to_bytes());
    if shared.iter().all(|b| *b == 0) {
        bail!("That inbox key isn't usable.");
    }
    let mut nonce = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce);
    let plain = Zeroizing::new(serde_json::to_vec(item)?);
    let data = cipher(&shared, &epk, public)
        .encrypt(Nonce::from_slice(&nonce), chacha20poly1305::aead::Payload { msg: &plain, aad: INFO })
        .map_err(|_| anyhow!("Couldn't lock the login away."))?;
    Ok(serde_json::to_string(&Sealed { v: 1, e: B64.encode(epk), n: B64.encode(nonce), d: B64.encode(data) })?)
}

fn open(secret: &[u8; 32], line: &str) -> Option<InboxItem> {
    let s: Sealed = serde_json::from_str(line).ok()?;
    if s.v != 1 {
        return None;
    }
    let epk: [u8; 32] = B64.decode(&s.e).ok()?.try_into().ok()?;
    let nonce: [u8; 12] = B64.decode(&s.n).ok()?.try_into().ok()?;
    let data = B64.decode(&s.d).ok()?;
    let pk = MontgomeryPoint::mul_base_clamped(*secret).to_bytes();
    let shared = Zeroizing::new(MontgomeryPoint(epk).mul_clamped(*secret).to_bytes());
    let plain = Zeroizing::new(
        cipher(&shared, &epk, &pk)
            .decrypt(Nonce::from_slice(&nonce), chacha20poly1305::aead::Payload { msg: &data, aad: INFO })
            .ok()?,
    );
    serde_json::from_slice(&plain).ok()
}

impl Vault {
    fn inbox_secret(&self) -> Option<Zeroizing<[u8; 32]>> {
        let item = self.db.meta.custom_data.get(INBOX_KEY)?;
        let Some(CustomDataValue::String(s)) = &item.value else { return None };
        let bytes = Zeroizing::new(B64.decode(s).ok()?);
        let arr: [u8; 32] = bytes.as_slice().try_into().ok()?;
        Some(Zeroizing::new(arr))
    }

    /// The inbox's public key, made (and saved in the vault) the first time it's needed.
    pub fn inbox_public_key(&mut self) -> Result<[u8; 32]> {
        if let Some(sk) = self.inbox_secret() {
            return Ok(MontgomeryPoint::mul_base_clamped(*sk).to_bytes());
        }
        let mut sk = Zeroizing::new([0u8; 32]);
        rand::thread_rng().fill_bytes(sk.as_mut());
        self.db.meta.custom_data.insert(
            INBOX_KEY.to_string(),
            CustomDataItem { value: Some(CustomDataValue::String(B64.encode(*sk))), last_modification_time: None },
        );
        self.save()?;
        Ok(MontgomeryPoint::mul_base_clamped(*sk).to_bytes())
    }

    /// Moves logins saved while locked into the vault: a new password for a login already saved
    /// for that website and username, else a new login. The caller deletes the inbox afterwards.
    pub fn merge_inbox(&mut self, lines: &str) -> Result<InboxMerge> {
        let Some(sk) = self.inbox_secret() else {
            return Ok(InboxMerge { skipped: lines.lines().filter(|l| !l.trim().is_empty()).count(), ..Default::default() });
        };
        let mut out = InboxMerge::default();
        let mut items: Vec<InboxItem> = Vec::new();
        for line in lines.lines().filter(|l| !l.trim().is_empty()) {
            match open(&sk, line.trim()) {
                Some(i) if host_of(&i.url).is_some() && !i.password.is_empty() => items.push(i),
                _ => out.skipped += 1,
            }
        }
        // Oldest first, so the newest password for a login wins.
        items.sort_by_key(|i| i.at);
        for i in &items {
            let existing = self
                .logins_for(&i.url)
                .into_iter()
                .find(|s| s.username == i.username && match_score(&s.url, &i.url) == Some(3))
                .map(|s| s.id);
            self.save_from_browser(existing.as_deref(), &i.url, &i.title, &i.username, &i.password)?;
            if existing.is_some() {
                out.updated += 1;
            } else {
                out.added += 1;
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(url: &str, user: &str, pw: &str, at: i64) -> InboxItem {
        InboxItem { url: url.into(), title: String::new(), username: user.into(), password: pw.into(), at }
    }

    fn vault() -> (tempfile::TempDir, Vault) {
        let dir = tempfile::tempdir().unwrap();
        let v = Vault::open(&dir.path().join("v.kdbx"), &[7u8; 32]).unwrap();
        (dir, v)
    }

    #[test]
    fn logins_saved_while_locked_move_in_on_unlock() {
        let (_dir, mut v) = vault();
        let pk = v.inbox_public_key().unwrap();
        assert_eq!(v.inbox_public_key().unwrap(), pk, "the same key every time");
        let lines = [
            seal(&pk, &item("https://shop.example/signup", "me@example.org", "first", 1)).unwrap(),
            seal(&pk, &item("https://shop.example/account", "me@example.org", "second", 2)).unwrap(),
            seal(&pk, &item("https://mail.example/", "", "pw", 3)).unwrap(),
            "not a sealed login".to_string(),
        ]
        .join("\n");
        assert!(!lines.contains("first") && !lines.contains("me@example.org"), "nothing readable in the inbox");
        let r = v.merge_inbox(&lines).unwrap();
        assert_eq!(r, InboxMerge { added: 2, updated: 1, skipped: 1 });
        let shop = v.logins_for("https://shop.example/login");
        assert_eq!(shop.len(), 1);
        assert_eq!(v.reveal(&shop[0].id, "Password").unwrap().as_str(), "second", "the newest password wins");
    }

    #[test]
    fn another_vaults_inbox_cant_be_opened() {
        let (_a, mut a) = vault();
        let (_b, mut b) = vault();
        let line = seal(&a.inbox_public_key().unwrap(), &item("https://x.example", "u", "p", 1)).unwrap();
        b.inbox_public_key().unwrap();
        assert_eq!(b.merge_inbox(&line).unwrap(), InboxMerge { skipped: 1, ..Default::default() });
        assert_eq!(a.merge_inbox(&line).unwrap().added, 1);
    }

    #[test]
    fn the_inbox_key_stays_out_of_exports() {
        let (dir, mut v) = vault();
        v.inbox_public_key().unwrap();
        let out = dir.path().join("e.kdbx");
        v.export_kdbx(&out, "export-password", None).unwrap();
        let e = keepass::Database::open(&mut std::fs::File::open(&out).unwrap(), keepass::DatabaseKey::new().with_password("export-password")).unwrap();
        assert!(!e.meta.custom_data.contains_key(INBOX_KEY));
    }
}
