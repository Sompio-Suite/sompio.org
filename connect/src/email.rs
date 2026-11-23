//! Live email over IMAP: reads your Sent folder and can move a sent email to Trash.
//!
//! Signing in uses an app password (Gmail, Yahoo, iCloud and others hand these out in their
//! security settings), never the account's real password. It is kept in the OS credential store.
//! Sompio only opens the Sent folder read-only ("EXAMINE") while reading, so nothing gets marked
//! as read, and it only touches a message when the person asks it to delete one.
//!
//! Records use the email's Message-ID, the same ID a Google Takeout import uses, so review
//! decisions carry over between the two.

use crate::SecretStore;
use anyhow::{anyhow, bail, Context, Result};
use imap::{ClientBuilder, ConnectionMode, Connection, Session};
use imap_proto::NameAttribute;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use sompio_clean::{gmail_link, parse_sent_mail, LiveChannel, LiveMessage};

const KEY_ACCOUNT: &str = "email.account";
/// Only the first part of each email is downloaded: the text comes first, attachments after.
const MAX_BYTES: usize = 512 * 1024;
const BATCH: usize = 100;

#[derive(Debug, Clone, Serialize)]
pub struct Provider {
    pub id: &'static str,
    pub name: &'static str,
    pub host: &'static str,
    /// 993 for IMAP over TLS; Proton Mail Bridge listens on 1143 with STARTTLS.
    pub port: u16,
    /// Where the person makes an app password.
    pub help_url: &'static str,
}

pub const PROVIDERS: &[Provider] = &[
    Provider { id: "gmail", name: "Gmail", host: "imap.gmail.com", port: 993, help_url: "https://myaccount.google.com/apppasswords" },
    Provider { id: "proton", name: "Proton Mail", host: "127.0.0.1", port: 1143, help_url: "https://proton.me/mail/bridge" },
    Provider { id: "yahoo", name: "Yahoo Mail", host: "imap.mail.yahoo.com", port: 993, help_url: "https://login.yahoo.com/account/security" },
    Provider { id: "icloud", name: "iCloud Mail", host: "imap.mail.me.com", port: 993, help_url: "https://account.apple.com" },
    Provider { id: "gmx", name: "GMX", host: "imap.gmx.net", port: 993, help_url: "https://www.gmx.net" },
    Provider { id: "fastmail", name: "Fastmail", host: "imap.fastmail.com", port: 993, help_url: "https://app.fastmail.com/settings/security/apps" },
    Provider { id: "mailboxorg", name: "mailbox.org", host: "imap.mailbox.org", port: 993, help_url: "" },
    Provider { id: "posteo", name: "Posteo", host: "posteo.de", port: 993, help_url: "" },
    Provider { id: "runbox", name: "Runbox", host: "mail.runbox.com", port: 993, help_url: "https://help.runbox.com/imap/" },
    Provider { id: "startmail", name: "StartMail", host: "imap.startmail.com", port: 993, help_url: "https://support.startmail.com/hc/en-us/articles/360006596718-Server-addresses" },
    Provider { id: "other", name: "Other (IMAP)", host: "", port: 993, help_url: "" },
];

#[derive(Serialize, Deserialize, Clone)]
struct Account {
    provider: String,
    host: String,
    /// Saved accounts from before ports could change used 993.
    #[serde(default = "imaps_port")]
    port: u16,
    address: String,
    password: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Status {
    pub connected: bool,
    pub address: Option<String>,
    pub provider: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncProgress {
    pub done: usize,
    pub total: usize,
}

/// Turns IMAP and network errors into something a person can act on.
fn friendly(e: impl std::fmt::Display) -> anyhow::Error {
    let s = e.to_string();
    let l = s.to_ascii_lowercase();
    if l.contains("authenticat") || l.contains("invalid credentials") || l.contains("login failed") || l.contains("[auth") {
        anyhow!("The email address or app password wasn't accepted. Use an app password, not your normal password.")
    } else if l.contains("failed to lookup") || l.contains("no such host") || l.contains("dns") || l.contains("not known") {
        anyhow!("Couldn't find that mail server. Check the server name and your internet connection.")
    } else if l.contains("timed out") || l.contains("refused") || l.contains("unreachable") {
        anyhow!("Couldn't reach the mail server. Check your internet connection and try again.")
    } else if l.contains("certificate") || l.contains("tls") {
        anyhow!("The mail server's security certificate couldn't be verified, so Sompio didn't connect.")
    } else {
        anyhow!("Email: {s}")
    }
}

type Imap = Session<Connection>;

fn imaps_port() -> u16 {
    993
}

/// A mail server on this computer, like Proton Mail Bridge.
fn is_local(host: &str) -> bool {
    matches!(host, "127.0.0.1" | "localhost" | "::1" | "[::1]")
}

/// "host" or "host:port" from the IMAP server field.
fn split_host(input: &str, default_port: u16) -> Result<(String, u16)> {
    let h = input.trim().trim_start_matches("imaps://").trim_start_matches("imap://").trim_end_matches('/');
    let (host, port) = match h.rsplit_once(':') {
        Some((host, port)) if !host.contains(':') || host.ends_with(']') => {
            let port = port.parse::<u16>().map_err(|_| anyhow!("The port after the colon should be a number, like 993."))?;
            (host.to_string(), port)
        }
        _ => (h.to_string(), default_port),
    };
    if host.is_empty() || host.contains(' ') {
        bail!("Enter your provider's IMAP server, like imap.example.com.");
    }
    Ok((host, port))
}

fn login(a: &Account) -> Result<Imap> {
    let mut b = ClientBuilder::new(a.host.as_str(), a.port);
    b = if a.port == 993 { b.mode(ConnectionMode::Tls) } else { b.mode(ConnectionMode::StartTls) };
    if is_local(&a.host) {
        // Proton Mail Bridge runs on this computer and makes its own certificate, which no
        // authority signs. The connection never leaves the computer, so it's accepted, but only here.
        b = b.danger_skip_tls_verify(true);
    }
    let client = b.connect().map_err(|e| friendly_for(&a.host, e))?;
    client.login(&a.address, &a.password).map_err(|(e, _)| friendly_for(&a.host, e))
}

/// Like `friendly`, but explains Bridge when the server is on this computer.
fn friendly_for(host: &str, e: impl std::fmt::Display) -> anyhow::Error {
    let s = e.to_string().to_ascii_lowercase();
    if is_local(host) && (s.contains("refused") || s.contains("timed out") || s.contains("unreachable")) {
        anyhow!("Couldn't reach Proton Mail Bridge on this computer. Open Bridge, sign in, and check the IMAP port it shows.")
    } else if is_local(host) && (s.contains("authenticat") || s.contains("login failed") || s.contains("[auth") || s.contains("no such user")) {
        anyhow!("Bridge didn't accept that. Use the password Bridge shows under your account's mailbox details, not your Proton password.")
    } else {
        friendly(e)
    }
}

/// The folder with the given special use, falling back to common names.
fn find_folder(s: &mut Imap, want: NameAttribute<'static>, names: &[&str]) -> Result<String> {
    let list = s.list(Some(""), Some("*")).map_err(friendly)?;
    if let Some(n) = list.iter().find(|n| n.attributes().contains(&want)) {
        return Ok(n.name().to_string());
    }
    for guess in names {
        if let Some(n) = list.iter().find(|n| n.name().eq_ignore_ascii_case(guess)) {
            return Ok(n.name().to_string());
        }
    }
    bail!("Couldn't find the {} folder in this mailbox.", names[0])
}

fn sent_folder(s: &mut Imap) -> Result<String> {
    find_folder(s, NameAttribute::Sent, &["Sent", "Sent Items", "Sent Messages", "Sent Mail", "[Gmail]/Sent Mail", "INBOX.Sent"])
}

fn trash_folder(s: &mut Imap) -> Result<String> {
    find_folder(s, NameAttribute::Trash, &["Trash", "Deleted Items", "Deleted Messages", "[Gmail]/Trash", "[Gmail]/Bin", "INBOX.Trash"])
}

fn quote(s: &str) -> Result<String> {
    if s.contains(['"', '\\', '\r', '\n']) {
        bail!("That email can't be looked up on the server.");
    }
    Ok(format!("\"{s}\""))
}

pub struct Email {
    store: Arc<dyn SecretStore>,
}

impl Email {
    pub fn new(store: Arc<dyn SecretStore>) -> Self {
        Self { store }
    }

    fn account(&self) -> Result<Option<Account>> {
        Ok(self.store.get(KEY_ACCOUNT)?.and_then(|j| serde_json::from_str(&j).ok()))
    }

    fn need_account(&self) -> Result<Account> {
        self.account()?.ok_or_else(|| anyhow!("Connect an email account first."))
    }

    /// Doesn't go online: says what's saved.
    pub fn status(&self) -> Result<Status> {
        Ok(match self.account()? {
            Some(a) => Status { connected: true, address: Some(a.address), provider: Some(a.provider) },
            None => Status { connected: false, address: None, provider: None },
        })
    }

    /// Checks the details by signing in and finding the Sent folder, then saves them.
    pub fn connect(&self, provider: &str, host: &str, address: &str, password: &str) -> Result<Status> {
        let address = address.trim();
        if !address.contains('@') {
            bail!("Enter your full email address.");
        }
        let password: String = password.chars().filter(|c| !c.is_whitespace()).collect(); // Google shows them in groups of four
        if password.is_empty() {
            bail!("Enter the app password.");
        }
        let p = PROVIDERS.iter().find(|p| p.id == provider);
        let (host, port) = match p {
            // Bridge can be set to another port, so the field may override it.
            Some(p) if p.id == "proton" && !host.trim().is_empty() => split_host(host, p.port)?,
            Some(p) if !p.host.is_empty() => (p.host.to_string(), p.port),
            _ => split_host(host, 993)?,
        };
        let a = Account { provider: provider.to_string(), host, port, address: address.to_string(), password };
        let mut s = login(&a)?;
        sent_folder(&mut s)?;
        let _ = s.logout();
        self.store.set(KEY_ACCOUNT, &serde_json::to_string(&a)?)?;
        self.status()
    }

    /// Forgets the account. Nothing on the server changes.
    pub fn disconnect(&self) -> Result<()> {
        self.store.delete(KEY_ACCOUNT)
    }

    /// Reads every email in the Sent folder.
    pub fn fetch_sent(&self, progress: &mut dyn FnMut(SyncProgress)) -> Result<Vec<LiveChannel>> {
        let a = self.need_account()?;
        let mut s = login(&a)?;
        let folder = sent_folder(&mut s)?;
        let mbox = s.examine(&folder).map_err(friendly)?;
        let validity = mbox.uid_validity.unwrap_or(0);
        let mut uids: Vec<u32> = s.uid_search("ALL").map_err(friendly)?.into_iter().collect();
        uids.sort_unstable();
        let total = uids.len();
        progress(SyncProgress { done: 0, total });
        let gmail = a.host == "imap.gmail.com";
        let mut messages = Vec::with_capacity(total);
        for (i, chunk) in uids.chunks(BATCH).enumerate() {
            let set = chunk.iter().map(u32::to_string).collect::<Vec<_>>().join(",");
            let fetches = s.uid_fetch(&set, format!("(UID BODY.PEEK[]<0.{MAX_BYTES}>)")).map_err(friendly)?;
            for f in fetches.iter() {
                let (Some(uid), Some(raw)) = (f.uid, f.body()) else { continue };
                let Some(mut m) = parse_sent_mail(raw) else { continue };
                if !m.id.contains('@') {
                    m.id = format!("imap:{validity}:{uid}");
                }
                let link = (gmail && !m.id.starts_with("imap:")).then(|| gmail_link(&m.id));
                messages.push(LiveMessage { ext_id: m.id, ts: m.ts, content: m.text, link });
            }
            progress(SyncProgress { done: ((i + 1) * BATCH).min(total), total });
        }
        let _ = s.logout();
        Ok(vec![LiveChannel { ext_id: "sent".into(), name: "Sent mail".into(), kind: "posts".into(), messages }])
    }

    /// Moves one sent email to Trash. `ext_id` is the Message-ID (or imap:validity:uid).
    pub fn delete_message(&self, ext_id: &str) -> Result<()> {
        let a = self.need_account()?;
        let mut s = login(&a)?;
        let sent = sent_folder(&mut s)?;
        let mbox = s.select(&sent).map_err(friendly)?;
        let uids: Vec<u32> = if let Some(rest) = ext_id.strip_prefix("imap:") {
            let (v, uid) = rest.split_once(':').context("bad id")?;
            if v.parse::<u32>().ok() != mbox.uid_validity {
                bail!("The mailbox changed since the last sync. Sync again, then delete.");
            }
            vec![uid.parse()?]
        } else {
            let q = format!("HEADER Message-ID {}", quote(&format!("<{ext_id}>"))?);
            s.uid_search(q).map_err(friendly)?.into_iter().collect()
        };
        if uids.is_empty() {
            bail!("That email is no longer in your Sent folder.");
        }
        let set = uids.iter().map(u32::to_string).collect::<Vec<_>>().join(",");
        let trash = trash_folder(&mut s)?;
        let caps = s.capabilities().map_err(friendly)?;
        if caps.has_str("MOVE") {
            s.uid_mv(&set, &trash).map_err(friendly)?;
        } else {
            s.uid_copy(&set, &trash).map_err(friendly)?;
            s.uid_store(&set, "+FLAGS.SILENT (\\Deleted)").map_err(friendly)?;
            if caps.has_str("UIDPLUS") {
                s.uid_expunge(&set).map_err(friendly)?;
            } else {
                s.expunge().map_err(friendly)?;
            }
        }
        let _ = s.logout();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MemoryStore;

    #[test]
    fn host_and_port() {
        assert_eq!(split_host("imap.example.com", 993).unwrap(), ("imap.example.com".into(), 993));
        assert_eq!(split_host(" 127.0.0.1:1144 ", 1143).unwrap(), ("127.0.0.1".into(), 1144));
        assert_eq!(split_host("imaps://mail.example.org:143/", 993).unwrap(), ("mail.example.org".into(), 143));
        assert!(split_host("127.0.0.1:abc", 1143).is_err());
        assert!(split_host("", 993).is_err());
        assert!(is_local("127.0.0.1") && is_local("localhost") && !is_local("imap.gmail.com"));
        // Accounts saved by earlier versions have no port: they used 993.
        let old: Account = serde_json::from_str(r#"{"provider":"gmail","host":"imap.gmail.com","address":"a@b","password":"x"}"#).unwrap();
        assert_eq!(old.port, 993);
    }

    #[test]
    fn checks_input_before_going_online() {
        let e = Email::new(Arc::new(MemoryStore::default()));
        assert!(!e.status().unwrap().connected);
        assert!(e.connect("gmail", "", "not-an-address", "x").unwrap_err().to_string().contains("full email"));
        assert!(e.connect("gmail", "", "a@b.c", "   ").unwrap_err().to_string().contains("app password"));
        assert!(e.connect("other", "", "a@b.c", "pw").unwrap_err().to_string().contains("IMAP server"));
        assert!(e.delete_message("x@y").unwrap_err().to_string().contains("Connect"));
    }

    #[test]
    fn quoting_refuses_injection() {
        assert!(quote("<a@b>").is_ok());
        assert!(quote("<a@b>\" OR ALL").is_err());
    }

    fn _send_check() {
        fn is_send<T: Send>() {}
        is_send::<Email>();
    }
}
