//! Email from a mailbox file (.mbox), such as Gmail's Google Takeout export.
//!
//! Only mail you sent is imported: Gmail marks it with an "X-Gmail-Labels: …Sent…" header,
//! and a file whose name says "sent" is taken as sent mail. The mailbox is streamed, never
//! loaded whole, because Takeout files can be many gigabytes.
//! Quoted replies and forwards (lines starting with ">", "On … wrote:", "Op … schreef:", an Outlook
//! "From:/Sent:" block, "Forwarded message") are someone else's words. They're kept after
//! [`QUOTED_MARK`] so Trace can show what they contain apart from what you wrote yourself.

use super::{file_name_lower, ImportProgress, ImportSummary, Source, Writer};
use anyhow::Result;
use mail_parser::MessageParser;
use std::io::{BufRead, BufReader, Read};

/// Parsed sent email, ready to store.
#[derive(Debug, Clone)]
pub struct Mail {
    pub id: String,
    pub ts: i64,
    pub text: String,
    /// The conversation it belongs to: Gmail's conversation ID, or the first email it replies to.
    pub thread: String,
    /// The sender's address (yours, for sent mail).
    pub from: String,
    /// Gmail's own conversation ID, in exports from Google Takeout.
    pub gmail_thread: Option<u64>,
}

/// Separates your own words from the quoted or forwarded email under them. Everything after
/// this line in a stored email was written by someone else.
pub const QUOTED_MARK: &str = "\n\n\u{2014}\u{2014} Earlier email, written by someone else \u{2014}\u{2014}\n";

/// The most of a quoted email that's kept, so long threads don't fill the database.
const MAX_QUOTED: usize = 20_000;

/// The verb in the "On <date>, <name> wrote:" line, in the languages mail apps use most. Some put
/// the name after it ("Op … schreef Anna <anna@…>:").
const WROTE: [&str; 11] = [
    " wrote", " schreef", " schrieb", " a écrit", " escribió", " ha scritto", " escreveu", " skrev", " kirjoitti",
    " napisał", " a scris",
];

/// Lines that start a forwarded email, or Outlook's header block above a reply.
const FORWARD: [&str; 8] = [
    "-----original message", "---------- forwarded message", "-------- forwarded message", "begin forwarded message",
    "-------- doorgestuurd bericht", "---------- doorgestuurd bericht", "-----oorspronkelijk bericht", "-------- weitergeleitete nachricht",
];

/// Whether `t` (trimmed) starts the email being replied to or forwarded. `next` is the line after it,
/// for Outlook's "From: … / Sent: …" block.
fn starts_quote(t: &str, next: Option<&str>) -> bool {
    let l = t.to_lowercase();
    // The date or the address in it keeps "Here's what my teacher wrote:" from counting.
    if l.ends_with(':') && WROTE.iter().any(|w| l.contains(w)) && t.chars().count() < 300 && t.chars().any(|c| c.is_ascii_digit() || c == '@') {
        return true;
    }
    if t == "________________________________" || FORWARD.iter().any(|f| l.starts_with(f)) {
        return true;
    }
    let header = |s: &str, names: &[&str]| {
        let s = s.trim().to_lowercase();
        names.iter().any(|n| s.starts_with(n))
    };
    header(t, &["from:", "van:", "von:", "de :", "de:"])
        && next.is_some_and(|n| header(n, &["sent:", "verzonden:", "gesendet:", "envoyé :", "envoyé:", "date:", "datum:", "to:", "aan:"]))
}

/// Splits a plain-text body into your own words and the quoted or forwarded part under them.
pub(crate) fn split_quoted(body: &str) -> (String, String) {
    let lines: Vec<&str> = body.lines().collect();
    let mut own = Vec::new();
    let mut quoted = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let t = lines[i].trim();
        // Gmail wraps long "On …, Name <address>" lines, putting "wrote:" on the next line.
        let wrapped = (t.starts_with("On ") || t.starts_with("Op ") || t.starts_with("Am ") || t.starts_with("Le "))
            && lines.get(i + 1).is_some_and(|n| starts_quote(n.trim(), None) && !n.trim().starts_with('-'));
        if wrapped || starts_quote(t, lines.get(i + 1).copied()) {
            quoted.extend(lines[i..].iter().map(|l| l.trim_start_matches('>').trim_start()));
            break;
        }
        if let Some(q) = t.strip_prefix('>') {
            quoted.push(q.trim_start());
        } else {
            own.push(lines[i]);
        }
        i += 1;
    }
    let mut quoted = quoted.join("\n").trim().to_string();
    if quoted.len() > MAX_QUOTED {
        let mut cut = MAX_QUOTED;
        while !quoted.is_char_boundary(cut) {
            cut -= 1;
        }
        quoted.truncate(cut);
    }
    (own.join("\n").trim().to_string(), quoted)
}

/// Turns raw email bytes into the text Sompio scans: recipients, subject, and your own words.
pub(crate) fn parse_mail(raw: &[u8]) -> Option<Mail> {
    let msg = MessageParser::default().parse(raw)?;
    let to: Vec<String> = msg
        .to()
        .map(|a| a.iter().filter_map(|x| x.address().map(str::to_string)).collect())
        .unwrap_or_default();
    let subject = msg.subject().unwrap_or("").to_string();
    let body = msg.body_text(0).map(|b| b.to_string()).unwrap_or_default();
    let (own, quoted) = split_quoted(&body);
    let mut text = String::new();
    if !to.is_empty() {
        text.push_str(&format!("To: {}\n", to.join(", ")));
    }
    if !subject.is_empty() {
        text.push_str(&format!("Subject: {subject}\n"));
    }
    text.push('\n');
    text.push_str(&own);
    if text.len() > 100_000 {
        let mut cut = 100_000;
        while !text.is_char_boundary(cut) {
            cut -= 1;
        }
        text.truncate(cut);
    }
    if !quoted.is_empty() {
        text.push_str(QUOTED_MARK);
        text.push_str(&quoted);
    }
    let ts = msg.date().map(|d| d.to_timestamp() * 1000).unwrap_or(0);
    let id = msg.message_id().map(str::to_string).unwrap_or_else(|| format!("{ts}-{}", subject.len()));
    let from = msg.from().and_then(|a| a.first()).and_then(|a| a.address()).unwrap_or("").to_string();
    let gmail_thread = msg.header_raw("X-GM-THRID").and_then(|v| v.trim().parse::<u64>().ok());
    // The oldest email in the conversation: the first one listed under References, else the one it
    // answers, else this one.
    let first_of = |v: &mail_parser::HeaderValue| -> Option<String> {
        v.as_text_list().and_then(|l| l.first().map(|s| s.to_string())).or_else(|| v.as_text().map(str::to_string))
    };
    let root = first_of(msg.references()).or_else(|| first_of(msg.in_reply_to())).unwrap_or_else(|| id.clone());
    let thread = match gmail_thread {
        Some(t) => format!("gm:{t}"),
        None => root.trim().trim_matches(['<', '>']).to_string(),
    };
    Some(Mail { id, ts, text, thread, from, gmail_thread })
}

/// Splits an mbox stream into messages and calls `f` with each one's raw bytes.
fn each_message(r: &mut dyn Read, mut f: impl FnMut(&[u8]) -> Result<()>) -> Result<()> {
    let mut reader = BufReader::with_capacity(1 << 20, r);
    let mut current: Vec<u8> = Vec::new();
    let mut line = Vec::new();
    let mut prev_blank = true;
    loop {
        line.clear();
        if reader.read_until(b'\n', &mut line)? == 0 {
            break;
        }
        if prev_blank && line.starts_with(b"From ") {
            if !current.is_empty() {
                f(&current)?;
                current.clear();
            }
            prev_blank = false;
            continue; // the separator line isn't part of the message
        }
        prev_blank = line == b"\n" || line == b"\r\n";
        // mboxrd escapes ">From " inside messages
        if line.starts_with(b">From ") {
            current.extend_from_slice(&line[1..]);
        } else {
            current.extend_from_slice(&line);
        }
    }
    if !current.is_empty() {
        f(&current)?;
    }
    Ok(())
}

fn is_sent(raw: &[u8]) -> bool {
    // Look only at the headers.
    let end = raw.windows(4).position(|w| w == b"\r\n\r\n").or_else(|| raw.windows(2).position(|w| w == b"\n\n")).unwrap_or(raw.len());
    let headers = String::from_utf8_lossy(&raw[..end]);
    headers.lines().any(|l| {
        let l = l.to_ascii_lowercase();
        l.starts_with("x-gmail-labels:") && l.split(|c: char| c == ',' || c == ':').any(|p| p.trim() == "sent")
    })
}

pub(super) fn import(
    src: &mut Source,
    names: &[String],
    w: &mut Writer,
    progress: &mut dyn FnMut(ImportProgress),
) -> Result<ImportSummary> {
    let mut summary = ImportSummary::default();
    let files: Vec<String> = names.iter().filter(|n| n.to_ascii_lowercase().ends_with(".mbox")).cloned().collect();
    let cid = w.channel("sent", "Sent mail", "posts", None, None)?;
    let mut total = 0usize;
    for f in &files {
        let whole_file_sent = file_name_lower(f).contains("sent");
        src.with_reader(f, |r| {
            each_message(r, |raw| {
                total += 1;
                if total % 2000 == 0 {
                    progress(ImportProgress { phase: "Reading mailbox", done: total, total: 0 });
                }
                if !(whole_file_sent || is_sent(raw)) {
                    return Ok(());
                }
                if let Some(m) = super::parse_sent_mail(raw) {
                    // Google Takeout says which Gmail conversation each email is in: open that directly.
                    let link = match m.gmail_thread {
                        Some(t) => super::gmail_open_link(&m.from, t),
                        None => super::gmail_link(&m.id),
                    };
                    w.message_in(&m.id, cid, m.ts, &m.text, "", Some(&link), Some(&m.thread))?;
                }
                Ok(())
            })
        })?;
    }
    import_eml(src, names, w, cid, &mut summary)?;
    if w.messages == 0 && total > 0 {
        summary.warnings.push(format!("Read {total} emails, but none were marked as sent by you."));
    }
    Ok(summary)
}

/// Folder names mail apps give sent mail, in the languages they're most often set to.
const SENT_FOLDERS: [&str; 12] = [
    "sent", "gesendet", "verzonden", "envoy", "enviad", "inviat", "skickat", "sendt", "wysłane", "lähetetyt", "odeslan", "verstuurd",
];

/// Whether a file sits in a folder for sent mail, like "Sent/2024-01-02 Re: rent.eml".
fn in_sent_folder(path: &str) -> bool {
    let p = path.to_lowercase().replace('\\', "/");
    let dir = p.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
    dir.split('/').any(|part| SENT_FOLDERS.iter().any(|s| part.contains(s)))
}

/// The sender's address, lower-cased.
fn from_address(raw: &[u8]) -> Option<String> {
    let msg = MessageParser::default().parse_headers(raw)?;
    msg.from()?.first()?.address().map(|a| a.trim().to_lowercase())
}

/// Emails exported one per file (.eml), as Tuta and Proton Mail export them. Sent mail is what sits
/// in a "Sent" folder (or is marked sent). If no folder says so, the export is taken to be a Sent
/// folder exported on its own when every email in it comes from the same address.
fn import_eml(src: &mut Source, names: &[String], w: &mut Writer, cid: i64, summary: &mut ImportSummary) -> Result<()> {
    let files: Vec<&String> = names.iter().filter(|n| n.to_ascii_lowercase().ends_with(".eml")).collect();
    if files.is_empty() {
        return Ok(());
    }
    let mut read = Vec::with_capacity(files.len());
    for f in &files {
        let raw = src.read(f)?;
        let marked = in_sent_folder(f) || is_sent(&raw);
        read.push((marked, from_address(&raw), raw));
    }
    let any_marked = read.iter().any(|(m, _, _)| *m);
    let senders: std::collections::HashSet<&String> = read.iter().filter_map(|(_, f, _)| f.as_ref()).collect();
    let one_sender = senders.len() == 1;
    for (marked, _, raw) in &read {
        if !(*marked || (!any_marked && one_sender)) {
            continue;
        }
        if let Some(m) = super::parse_sent_mail(raw) {
            w.message(&m.id, cid, m.ts, &m.text, "", None)?;
        }
    }
    if !any_marked && !one_sender {
        summary.warnings.push(
            "These emails come from different senders, so Sompio can't tell which ones you sent. Export only your Sent folder, then open that folder in Sompio.".into(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn splits_and_keeps_only_your_words() {
        let mbox = "From 1@xxx Mon Jan 1 00:00:00 2024\nX-Gmail-Labels: Sent,Important\nMessage-ID: <a@b>\nTo: bank@example.org\nSubject: Account\nDate: Mon, 1 Jan 2024 10:00:00 +0000\n\nMy IBAN is NL91 ABNA 0417 1643 00\n>From the archives\n\nOn Sun, someone wrote:\n> their private stuff\n\nFrom 2@xxx Mon Jan 1 00:00:00 2024\nX-Gmail-Labels: Inbox\nSubject: Newsletter\n\nhello\n";
        let mut got = vec![];
        each_message(&mut mbox.as_bytes(), |raw| {
            got.push((is_sent(raw), raw.to_vec()));
            Ok(())
        })
        .unwrap();
        assert_eq!(got.len(), 2);
        assert!(got[0].0 && !got[1].0);
        let m = parse_mail(&got[0].1).unwrap();
        assert!(m.text.contains("To: bank@example.org") && m.text.contains("NL91"));
        assert!(m.text.contains("From the archives"), "unescaped mboxrd line");
        let (own, quoted) = m.text.split_once(QUOTED_MARK).unwrap();
        assert!(own.contains("NL91") && !own.contains("their private stuff"));
        assert!(quoted.contains("their private stuff") && !quoted.contains("> "));
        assert_eq!(m.id, "a@b");
        assert_eq!(m.ts, 1_704_103_200_000);
    }

    #[test]
    fn finds_quotes_in_other_languages_and_outlook() {
        let cases = [
            "Thanks!\n\nOp ma 1 jan. 2024 om 10:00 schreef Anna <anna@example.org>:\nMijn BSN is 111222333",
            "Thanks!\n\nOn Mon, 1 Jan 2024 at 10:00, Anna Example\n<anna@example.org> wrote:\nsecret",
            "Thanks!\n\nFrom: Anna Example <anna@example.org>\nSent: Monday, 1 January 2024 10:00\nTo: me\nsecret",
            "Thanks!\n\n---------- Forwarded message ---------\nFrom: Anna\nsecret",
            "Thanks!\n\nAm Mo., 1. Jan. 2024 um 10:00 Uhr schrieb Anna <anna@example.org>:\nsecret",
        ];
        for c in cases {
            let (own, quoted) = split_quoted(c);
            assert_eq!(own, "Thanks!", "{c}");
            assert!(!quoted.is_empty(), "{c}");
        }
        // A sentence that merely mentions sending something isn't a quote.
        let (own, quoted) = split_quoted("From: the shop\nI ordered it yesterday.\nHere's what my teacher wrote:\nwell done");
        assert!(quoted.is_empty() && own.contains("the shop") && own.contains("well done"));
    }

    #[test]
    fn sent_folders_in_exports() {
        assert!(in_sent_folder("Tuta export/Sent/2024 Re rent.eml"));
        assert!(in_sent_folder("Verzonden items\\a.eml"));
        assert!(!in_sent_folder("Inbox/sent you a gift.eml"));
        assert!(!in_sent_folder("a.eml"));
        assert_eq!(from_address(b"From: Sam <Sam@Example.org>\r\nSubject: x\r\n\r\nhi").as_deref(), Some("sam@example.org"));
    }
}
