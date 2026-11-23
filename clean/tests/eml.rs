//! Emails exported one per file (.eml), the way Tuta and Proton Mail export them.

use sompio_clean::{import_package, run_scan, FindingsFilter, ScanOptions, Store};
use std::fs;

fn mail(from: &str, to: &str, id: &str, body: &str) -> String {
    format!("From: {from}\r\nTo: {to}\r\nMessage-ID: <{id}>\r\nSubject: test\r\nDate: Mon, 1 Jan 2024 10:00:00 +0000\r\n\r\n{body}\r\n")
}

#[test]
fn whole_mailbox_export_keeps_the_sent_folder() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("Inbox")).unwrap();
    fs::create_dir_all(dir.path().join("Sent")).unwrap();
    fs::write(dir.path().join("Sent/1.eml"), mail("me@tuta.com", "bank@example.org", "s1@x", "My IBAN is NL91 ABNA 0417 1643 00")).unwrap();
    fs::write(dir.path().join("Inbox/2.eml"), mail("shop@example.org", "me@tuta.com", "i1@x", "Your code is 4111 1111 1111 1111")).unwrap();
    let mut store = Store::open_in_memory().unwrap();
    let s = import_package(&mut store, dir.path(), &mut |_| {}).unwrap();
    assert_eq!(s.platform, "email");
    assert_eq!(s.messages, 1);
    run_scan(&mut store, &ScanOptions::default(), &mut |_, _| {}).unwrap();
    let all = store.findings_filtered(&FindingsFilter::default(), 0, 50, true).unwrap().items;
    assert!(all.iter().any(|f| f.matched.contains("NL91")));
    assert!(!all.iter().any(|f| f.matched.contains("4111")), "received mail isn't imported");
}

#[test]
fn a_sent_folder_exported_alone() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.eml"), mail("me@proton.me", "anna@example.org", "a@x", "Call me on +31 6 12345678")).unwrap();
    fs::write(dir.path().join("b.eml"), mail("Me <ME@proton.me>", "bob@example.org", "b@x", "hello")).unwrap();
    let mut store = Store::open_in_memory().unwrap();
    assert_eq!(import_package(&mut store, dir.path(), &mut |_| {}).unwrap().messages, 2);

    // Mixed senders and no Sent folder: nothing is guessed.
    let mixed = tempfile::tempdir().unwrap();
    fs::write(mixed.path().join("a.eml"), mail("me@proton.me", "x@example.org", "c@x", "hi")).unwrap();
    fs::write(mixed.path().join("b.eml"), mail("news@example.org", "me@proton.me", "d@x", "hi")).unwrap();
    let mut store = Store::open_in_memory().unwrap();
    let err = import_package(&mut store, mixed.path(), &mut |_| {}).unwrap_err().to_string();
    assert!(err.contains("no messages"), "{err}");
}
