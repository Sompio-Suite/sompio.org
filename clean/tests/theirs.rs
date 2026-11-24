//! In sent email, what you wrote yourself is kept apart from the quoted email under it.

use sompio_clean::{import_live, parse_sent_mail, run_scan, FindingsFilter, LiveChannel, LiveMessage, ScanOptions, Store};

#[test]
fn quoted_email_and_recipients_count_as_someone_elses() {
    let raw = "Message-ID: <r1@mail.example>\r\nTo: anna@example.org\r\nSubject: Re: forms\r\nDate: Mon, 1 Jan 2024 10:00:00 +0000\r\n\r\n\
My IBAN is NL91 ABNA 0417 1643 00, my phone +31 6 12345678.\r\n\r\n\
Op ma 1 jan. 2024 om 09:00 schreef Anna <anna@example.org>:\r\n> Mijn telefoon is +31 6 87654321\r\n";
    let mail = parse_sent_mail(raw.as_bytes()).unwrap();
    let mut store = Store::open_in_memory().unwrap();
    let chan = LiveChannel {
        ext_id: "sent".into(),
        name: "Sent mail".into(),
        kind: "posts".into(),
        messages: vec![LiveMessage { ext_id: mail.id, ts: mail.ts, content: mail.text, link: None }],
    };
    import_live(&mut store, "email", "e", vec![chan]).unwrap();
    let summary = run_scan(&mut store, &ScanOptions::default(), &mut |_, _| {}).unwrap();

    let all = store.findings_filtered(&FindingsFilter::default(), 0, 50, true).unwrap().items;
    let mine: Vec<_> = all.iter().filter(|f| !f.theirs).map(|f| f.matched.replace(' ', "")).collect();
    let theirs: Vec<_> = all.iter().filter(|f| f.theirs).map(|f| f.matched.replace(' ', "")).collect();
    assert!(mine.iter().any(|m| m.contains("NL91")), "{mine:?}");
    assert!(mine.iter().any(|m| m.contains("12345678")), "{mine:?}");
    assert!(theirs.iter().any(|m| m.contains("87654321")), "{theirs:?}");
    assert!(theirs.iter().any(|m| m.contains("anna@example.org")), "{theirs:?}");
    assert!(!mine.iter().any(|m| m.contains("87654321") || m.contains("anna@")), "{mine:?}");
    // Your own come first, and someone else's don't count as open.
    assert!(!all[0].theirs);
    assert_eq!(summary.theirs as usize, theirs.len());
    let mine_open: i64 = summary.by_kind.iter().map(|k| k.mine_open).sum();
    let open: i64 = summary.by_kind.iter().map(|k| k.open).sum();
    assert_eq!(mine_open as usize, mine.len());
    assert_eq!(open as usize, all.len());
}

#[test]
fn sample_data_shows_someone_elses_words() {
    let mut store = Store::open_in_memory().unwrap();
    sompio_clean::load_demo(&mut store).unwrap();
    let summary = run_scan(&mut store, &ScanOptions::default(), &mut |_, _| {}).unwrap();
    assert!(summary.theirs >= 3, "{}", summary.theirs);
}

#[test]
fn stored_data_is_checked_again_after_detection_changes() {
    use sompio_clean::{recheck_if_outdated, DETECTION_VERSION};
    let mut store = Store::open_in_memory().unwrap();
    // Nothing checked yet: nothing to redo.
    assert!(!recheck_if_outdated(&mut store).unwrap());
    sompio_clean::load_demo(&mut store).unwrap();
    run_scan(&mut store, &ScanOptions::default(), &mut |_, _| {}).unwrap();
    assert!(!recheck_if_outdated(&mut store).unwrap(), "fresh scan is current");
    // An install that last checked with an older version (or before versions were stored).
    store.set_meta("detection_version", &(DETECTION_VERSION - 1).to_string()).unwrap();
    assert!(recheck_if_outdated(&mut store).unwrap());
    assert!(!recheck_if_outdated(&mut store).unwrap(), "only once");
}
