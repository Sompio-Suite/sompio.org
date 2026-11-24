use sompio_clean::{Kind, ScanOptions, Scanner};

fn kinds(text: &str) -> Vec<Kind> {
    let s = Scanner::new(&ScanOptions { watch_terms: vec!["Jan de Vries".into(), "Acme BV".into()] }).unwrap();
    s.detect(text).into_iter().map(|d| d.kind).collect()
}

fn has(text: &str, k: Kind) -> bool {
    kinds(text).contains(&k)
}

#[test]
fn finds_real_personal_info() {
    assert!(has("mail me at jan.devries@gmail.com", Kind::Email));
    assert!(has("bel me: +31 6 12345678", Kind::Phone));
    assert!(has("mijn nummer is 06-12345678", Kind::Phone));
    assert!(has("call (415) 555-0132 tomorrow", Kind::Phone));
    assert!(has("IBAN NL91 ABNA 0417 1643 00 thanks", Kind::Iban));
    assert!(has("NL91ABNA0417164300", Kind::Iban));
    assert!(has("card 4111 1111 1111 1111 exp 12/29", Kind::Card));
    assert!(has("I live at Prinsengracht 263, 1016 GV Amsterdam", Kind::Address));
    assert!(has("I live at Prinsengracht 263, 1016 GV Amsterdam", Kind::Postcode));
    assert!(has("ship it to 221B Baker Street please", Kind::Address));
    assert!(has("stau pe Strada Lipscani nr. 12", Kind::Address));
    assert!(has("my server is 81.204.12.7", Kind::IpAddress));
    assert!(has("password: hunter2abc", Kind::Credential));
    assert!(has("token ghp_aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789", Kind::Credential));
    assert!(has("I'm Jan de Vries btw", Kind::WatchTerm));
    assert!(has("working at ACME BV now", Kind::WatchTerm));
}

#[test]
fn ignores_common_false_alarms() {
    // Discord mentions, channel links and IDs are long numbers, not cards or phones
    assert!(kinds("hey <@123456789012345678> look").is_empty());
    assert!(kinds("https://discord.com/channels/111111111111111111/222222222222222222").is_empty());
    assert!(kinds("4111111111111112").is_empty(), "fails Luhn");
    assert!(!has("IBAN NL91 ABNA 0417 1643 01", Kind::Iban), "bad checksum");
    // private / local IPs and version numbers
    assert!(!has("router at 192.168.1.1", Kind::IpAddress));
    assert!(!has("updated to v1.2.3.4", Kind::IpAddress));
    // casual sentences
    assert!(kinds("it's 5 minutes down the road").is_empty());
    assert!(kinds("meet at 2024 AM? lol, 1080 HZ monitor").is_empty());
    assert!(kinds("gg wp, 1v1 me at 10pm").is_empty());
    // watch terms match whole words only
    assert!(!has("janitor duty", Kind::WatchTerm));
}

#[test]
fn iban_digits_are_not_also_a_phone_number() {
    let k = kinds("NL91 ABNA 0417 1643 00");
    assert_eq!(k, vec![Kind::Iban]);
}

#[test]
fn finds_identity_numbers_codes_dates_and_plates() {
    // BSN 111222333 passes the eleven test; only flagged next to its name
    assert!(has("mijn bsn is 111222333", Kind::IdNumber));
    assert!(has("BSN: 1112.22.333", Kind::IdNumber));
    assert!(!has("order 111222333 shipped", Kind::IdNumber));
    assert!(!has("bsn 111222334", Kind::IdNumber), "fails the eleven test");
    assert!(has("my ssn is 123-45-6789", Kind::IdNumber));
    assert!(!has("ssn 000-12-3456", Kind::IdNumber));
    // A valid Romanian CNP (check digit 6)
    assert!(has("CNP 1800101221144 pentru formular", Kind::IdNumber));
    assert!(!has("CNP 1800101221145", Kind::IdNumber), "bad check digit");
    assert!(has("passport number: NXC4R9P12", Kind::Passport));
    assert!(has("paspoort nr. BN12FG345", Kind::Passport));
    assert!(!has("my passport is EXPIRED", Kind::Passport));
    assert!(has("discord backup codes: 8f3k-2m9x", Kind::BackupCode));
    assert!(!has("my recovery codes are stored safely", Kind::BackupCode));
    assert!(has(
        "seed phrase: abandon ability able about above absent absorb abstract absurd abuse access accident",
        Kind::BackupCode
    ));
    assert!(!has(
        "recovery phrase is something you should never share with the people you meet online okay",
        Kind::BackupCode
    ));
    assert!(has("born on 14-03-1994 in Utrecht", Kind::BirthDate));
    assert!(has("date of birth: March 14, 1994", Kind::BirthDate));
    assert!(has("geboren op 14 maart 1994", Kind::BirthDate));
    assert!(!has("born to raid, 1994 was a good year", Kind::BirthDate));
    assert!(has("my car is GZ-123-K, parked outside", Kind::NumberPlate));
    assert!(has("kenteken: 12-ABC-3", Kind::NumberPlate));
    assert!(has("license plate is AB 123 CD", Kind::NumberPlate));
    assert!(!has("GG-WP-ME lol", Kind::NumberPlate));
}

#[test]
fn finds_exact_locations() {
    assert!(has("[Photo location: 52.37312, 4.89034]", Kind::Location));
    assert!(has("come over https://www.google.com/maps/@52.3731234,4.8903456,17z", Kind::Location));
    assert!(has("https://www.openstreetmap.org/#map=18/52.37312/4.89034", Kind::Location));
    assert!(!has("pi is 3.14159, e is 2.71828", Kind::Location));
}

#[test]
fn finds_numbers_from_other_countries() {
    // Bank accounts
    for iban in ["DE89 3704 0044 0532 0130 00", "FR14 2004 1010 0505 0001 3M02 606", "ES91 2100 0418 4502 0005 1332", "IT60 X054 2811 1010 0000 0123 456", "GB29 NWBK 6016 1331 9268 19", "BE68 5390 0754 7034", "PL61 1090 1014 0000 0712 1981 2874", "RO49 AAAA 1B31 0075 9384 0000"] {
        assert!(has(&format!("send it to {iban} please"), Kind::Iban), "{iban}");
    }
    assert!(has("sort code 20-00-00, account number 12345678", Kind::Iban));
    assert!(has("routing number 021000021 account 1234567890", Kind::Iban));
    assert!(!has("routing number 021000022 account 1234567890", Kind::Iban), "bad check digit");
    // ID numbers
    assert!(has("my NI number is AB 12 34 56 C", Kind::IdNumber));
    assert!(!has("GB 12 34 56 A", Kind::IdNumber));
    assert!(has("DNI 12345678Z para la reserva", Kind::IdNumber));
    assert!(!has("DNI 12345678A", Kind::IdNumber), "wrong check letter");
    assert!(has("NIE X1234567L", Kind::IdNumber));
    assert!(has("codice fiscale RSSMRA85T10A562S", Kind::IdNumber));
    assert!(has("PESEL 44051401359", Kind::IdNumber));
    assert!(!has("PESEL 44051401358", Kind::IdNumber));
    assert!(has("Steuer-ID: 12 345 678 901", Kind::IdNumber));
    assert!(has("numéro de sécurité sociale 1 85 05 78 006 084 91", Kind::IdNumber));
    assert!(!has("numéro de sécurité sociale 1 85 05 78 006 084 36", Kind::IdNumber));
    // Addresses
    assert!(has("j'habite au 12 rue de Rivoli, Paris", Kind::Address));
    assert!(has("vivo en Calle Mayor 5, Madrid", Kind::Address));
    assert!(has("abito in Via Roma, 10 a Milano", Kind::Address));
    assert!(has("mieszkam na ul. Długa 7", Kind::Address));
    assert!(!has("send it via email 3 times", Kind::Address));
}

#[test]
fn postcodes_in_dates_codes_and_company_footers_are_skipped() {
    // Real ones still count.
    assert!(has("I live at Prinsengracht 263, 1016 GV Amsterdam", Kind::Postcode));
    assert!(has("postcode: 3811 AB", Kind::Postcode));
    assert!(has("ons nieuwe adres is Dorpsstraat 4\n2011 VA Haarlem", Kind::Postcode));
    // A booking: the year of a date followed by a code.
    assert!(!has("Payment in selected currency 07/06/2018 MC 159082262 confirmed", Kind::Postcode));
    assert!(!has("Payment ID Status Payment amount 28/02/2017 VI 49.98 EUR", Kind::Postcode));
    assert!(!has("ref 2018 MC 159082262", Kind::Postcode));
    // A company's own address in an email footer.
    assert!(!has("Booking.com B.V. - Nieuwezijds Voorburgwal 162, 1012 SJ Amsterdam", Kind::Postcode));
    assert!(!has("Example Ltd, registered office: Keizersgracht 1, 1015 CJ Amsterdam", Kind::Postcode));
}

#[test]
fn postcodes_after_a_date_split_over_two_lines_are_skipped() {
    // Booking emails often wrap right after the day and month.
    assert!(!has("Payment in selected currency 07/06/\n2018 MC 159082262 confirmed", Kind::Postcode));
    assert!(!has("Payment amount 28/02/\r\n2017 VI 49.98 EUR", Kind::Postcode));
    // A real address on its own line still counts.
    assert!(has("Dorpsstraat 4\n2011 VA Haarlem", Kind::Postcode));
}

#[test]
fn postcodes_after_a_date_with_anything_in_between_are_skipped() {
    let cases = [
        ("nbsp", "currency 07/06/\u{a0}2018 MC 159082262"),
        ("thin", "currency 07/06/\u{2009}2018 MC 159082262"),
        ("zwsp", "currency 07/06/\u{200b}2018 MC 159082262"),
        ("nl-space", "currency 07/06/\n 2018 MC 159082262"),
        ("nbsp-after", "currency 07/06/2018\u{a0}MC\u{a0}159082262"),
        ("tab-cells", "currency\t07/06/\t2018 MC\t159082262"),
        ("pipe", "07/06/ | 2018 MC | 159082262"),
        ("ellipsis-row", "Payment amount 28/02/2017 VI 49.98 EUR"),
        ("date-dash", "2017-02-28 2017 VI"),
        ("dots", "Datum 28.02. 2017 VI"),
        ("html-cells", "07/06/ </td><td> 2018 MC"),
    ];
    let mut hits = vec![];
    for (n, t) in cases {
        if has(t, Kind::Postcode) { hits.push(n); }
    }
    assert!(hits.is_empty(), "still matched: {hits:?}");
}

#[test]
fn real_postcodes_after_numbers_still_count() {
    // House numbers, phone numbers and house-number ranges right before a postcode.
    assert!(has("Kerkstraat 12-14, 1017 GL Amsterdam", Kind::Postcode));
    assert!(has("Tel 020-1234567, Damrak 1\n1012 LG Amsterdam", Kind::Postcode));
    assert!(has("Laan 3/2\n3511 AB Utrecht", Kind::Postcode));
}

#[test]
fn postcodes_in_a_company_signature_block_are_skipped() {
    // The tester's Booking.com email: name, street and postcode on separate lines.
    assert!(!has("Thanks for booking!\n\nBooking.com\nOosterdokskade 163\n1011 DL Amsterdam\nThe Netherlands", Kind::Postcode));
    assert!(!has("Kind regards,\nExample B.V.\nKeizersgracht 1\n1015 CJ Amsterdam", Kind::Postcode));
    // Your own address under your name still counts, and so does a sentence with a website in it.
    assert!(has("Groeten,\nSam Rivers\nPrinsengracht 263\n1016 GV Amsterdam", Kind::Postcode));
    assert!(has("I found the flat on funda.nl last week.\nThe address is Damrak 1\n1012 LG Amsterdam", Kind::Postcode));
}

#[test]
fn booking_signature_on_one_line_is_skipped() {
    // How the tester's email looks after the HTML is turned into text: a link, then the signature on one line.
    assert!(!has("<https://www.booking.com/content/help.html?aid=2311236#tns_travellers_gonewrong> Booking.com Oosterdokskade 163 1011 DL Amsterdam The Netherlands ----", Kind::Postcode));
    assert!(!has("<https://www.booking.com/help?aid=2311236#tns_travellers_gonewrong>\nBooking.com Oosterdokskade 163 1011 DL Amsterdam The Netherlands", Kind::Postcode));
}

#[test]
fn addresses_and_postcodes_from_other_countries() {
    let yes = [
        ("italy-booking", "Location 305 Via Vittorio Emanuele II, 95124 Catania, Italy", Kind::Postcode),
        ("italy-street", "Location 305 Via Vittorio Emanuele II, 95124 Catania, Italy", Kind::Address),
        ("germany", "Hauptstraße 5\n10115 Berlin", Kind::Postcode),
        ("germany-prefix", "Anschrift: D-10115 Berlin", Kind::Postcode),
        ("france", "12 rue de Rivoli, 75001 Paris", Kind::Postcode),
        ("poland", "ul. Długa 7, 00-238 Warszawa", Kind::Postcode),
        ("portugal", "Rua Augusta 24, 1100-048 Lisboa, Portugal", Kind::Postcode),
        ("brazil", "Avenida Paulista 1578, 01310-200 São Paulo, Brasil", Kind::Postcode),
        ("sweden", "Drottninggatan 5, 111 51 Stockholm", Kind::Postcode),
        ("sweden-street", "Drottninggatan 5, 111 51 Stockholm", Kind::Address),
        ("spain-caps", "Calle Mayor 5\n28013 MADRID", Kind::Postcode),
        ("country-only", "Hotel Bellevue\n8001 Zürich, Switzerland", Kind::Postcode),
        ("us", "1600 Amphitheatre Parkway, Mountain View, CA 94043", Kind::Postcode),
        ("us-street", "1600 Amphitheatre Parkway, Mountain View, CA 94043", Kind::Address),
        ("uk", "10 Downing Street, London SW1A 2AA", Kind::Postcode),
        ("canada", "Toronto, ON M5V 3L9", Kind::Postcode),
        ("australia", "Sydney NSW 2000", Kind::Postcode),
        ("ireland", "Grafton Street 12, Dublin D02 X285", Kind::Postcode),
        ("japan", "〒100-0001 東京都千代田区", Kind::Postcode),
        ("hungary", "Andrássy út 12, 1061 Budapest", Kind::Postcode),
        ("finland", "Aleksanterinkatu 52, 00100 Helsinki", Kind::Postcode),
    ];
    let missed: Vec<_> = yes.iter().filter(|(_, t, k)| !has(t, *k)).map(|(n, _, _)| *n).collect();
    assert!(missed.is_empty(), "missed: {missed:?}");

    // Numbers that are not postcodes.
    let no = [
        ("price", "Total, 2500 Euros"),
        ("year", "In 2019 Rome was hot"),
        ("year-country", "Released in 2024 Paris, France"),
        ("order", "Order 12345 Shipped today"),
        ("hours", "Meeting at 1500 Hours"),
        ("house-number", "1600 Amphitheatre Parkway, Mountain View, CA 94043"),
        ("no-comma-zip", "call me at TX 78701 later"),
        ("tracking", "Tracking 482910 Delivered"),
        ("phone", "Tel 020 555 12 Amsterdam"),
        ("pages", "See pages 1200 Onwards"),
    ];
    let wrong: Vec<_> = no
        .iter()
        .filter(|(n, t)| {
            let s = Scanner::new(&ScanOptions { watch_terms: vec![] }).unwrap();
            s.detect(t).iter().any(|d| d.kind == Kind::Postcode && (*n != "house-number" || &t[d.start..d.end] == "1600"))
        })
        .map(|(n, _)| *n)
        .collect();
    assert!(wrong.is_empty(), "wrongly flagged: {wrong:?}");
}

#[test]
fn addresses_and_postcodes_around_the_world() {
    use Kind::{Address as A, Postcode as P};
    let yes: &[(&str, &str, Kind)] = &[
        // Africa
        ("south-africa", "12 Long Street, Cape Town, 8001", P),
        ("south-africa-street", "12 Long Street, Cape Town, 8001", A),
        ("kenya-box", "P.O. Box 30126, Nairobi", A),
        ("kenya-postcode", "Kenyatta Avenue 14\n00100 Nairobi, Kenya", P),
        ("nigeria", "15 Allen Avenue, Ikeja, Lagos 100271, Nigeria", P),
        ("egypt", "١٢ شارع التحرير، القاهرة", A),
        ("morocco", "24 Rue Mohammed V, 20250 Casablanca, Maroc", P),
        ("senegal-bp", "B.P. 3010 Dakar, Sénégal", A),
        ("ghana-gps", "GA-183-8164, Accra, Ghana", P),
        ("ethiopia", "Bole Road 12\n1000 Addis Ababa, Ethiopia", P),
        ("tunisia", "Avenue Habib Bourguiba 5, 1001 Tunis", P),
        // Americas
        ("us", "350 Fifth Avenue, New York, NY 10118", P),
        ("us-zip4", "Austin, TX 78701-1234", P),
        ("canada", "290 Bremner Blvd, Toronto, ON M5V 3L9", P),
        ("mexico", "Av. Insurgentes Sur 1602, Col. Crédito Constructor, 03940 Ciudad de México", P),
        ("mexico-street", "Av. Insurgentes Sur 1602, Col. Crédito Constructor, 03940 Ciudad de México", A),
        ("mexico-cp", "C.P. 03940", P),
        ("brazil", "Avenida Paulista 1578, 01310-200 São Paulo, Brasil", P),
        ("brazil-cep", "CEP: 01310-200", P),
        ("argentina", "Av. Corrientes 1234, C1043AAZ Buenos Aires", P),
        ("colombia", "Calle 93 # 11-26, Bogotá", A),
        ("chile", "Avenida Providencia 1234, 7500000 Santiago, Chile", P),
        ("peru", "Av. Larco 345, Miraflores 15074, Peru", P),
        // Europe
        ("russia", "ул. Тверская, д. 7, Москва, 125009", P),
        ("russia-street", "ул. Тверская, д. 7, Москва, 125009", A),
        ("russia-start", "125009, Москва, ул. Тверская, 7", P),
        ("ukraine", "вул. Хрещатик, 22, Київ, 01001", P),
        ("greece", "Οδός Ερμού 12, 105 63 Αθήνα", P),
        ("greece-street", "Οδός Ερμού 12, 105 63 Αθήνα", A),
        ("czech", "Vodičkova 12\n110 00 Praha 1, Czech Republic", P),
        ("latvia", "Brīvības iela 1, LV-1010 Rīga", P),
        ("malta", "Republic Street 12, Valletta VLT 1117, Malta", P),
        ("plz", "PLZ 10115", P),
        ("uk", "221B Baker Street, London NW1 6XE", P),
        // Middle East
        ("israel", "רחוב הרצל 12, תל אביב", A),
        ("iran", "خیابان ولیعصر، پلاک ۱۲", A),
        ("saudi", "King Fahd Road 25\nRiyadh 12271, Saudi Arabia", P),
        ("turkey", "İstiklal Caddesi No: 5, 34433 Beyoğlu, İstanbul", P),
        ("uae-box", "PO Box 12345, Dubai, United Arab Emirates", A),
        // Asia
        ("india", "12 MG Road, Bengaluru 560001", P),
        ("india-space", "Flat No. 4, Andheri West, Mumbai - 400 053, India", P),
        ("india-flat", "Flat No. 4, Andheri West, Mumbai - 400 053, India", A),
        ("india-pin", "PIN code: 110001", P),
        ("pakistan", "House No. 12, Street 5, F-7/2, Islamabad 44000, Pakistan", P),
        ("bangladesh", "House 12, Road 5, Dhanmondi, Dhaka 1205, Bangladesh", P),
        ("china", "北京市朝阳区建国路88号", A),
        ("china-postcode", "邮编：100022", P),
        ("japan", "東京都千代田区丸の内1丁目9番1号", A),
        ("japan-postcode", "〒100-0005 東京都千代田区", P),
        ("korea", "서울특별시 강남구 테헤란로 152", A),
        ("korea-postcode", "우편번호 06236", P),
        ("singapore", "10 Bayfront Avenue, Singapore 018956", P),
        ("malaysia", "Jalan Sultan Ismail 12, 50250 Kuala Lumpur", P),
        ("malaysia-street", "Jalan Sultan Ismail 12, 50250 Kuala Lumpur", A),
        ("indonesia", "Jl. M.H. Thamrin No. 1, Jakarta 10310, Indonesia", P),
        ("indonesia-street", "Jl. M.H. Thamrin No. 1, Jakarta 10310, Indonesia", A),
        ("thailand", "123/45 ถนนสุขุมวิท กรุงเทพฯ 10110", A),
        ("thailand-postcode", "Sukhumvit Road 123, Bangkok 10110, Thailand", P),
        ("vietnam", "45 Đường Lê Lợi, Quận 1, Hồ Chí Minh", A),
        ("philippines", "6750 Ayala Avenue, Makati City 1226, Philippines", P),
        // Oceania
        ("australia", "1 Macquarie Street, Sydney NSW 2000", P),
        ("new-zealand", "12 Queen Street, Auckland 1010, New Zealand", P),
    ];
    let missed: Vec<_> = yes.iter().filter(|(_, t, k)| !has(t, *k)).map(|(n, _, _)| *n).collect();
    assert!(missed.is_empty(), "missed: {missed:?}");

    // Everyday numbers that must not become postcodes.
    let no = [
        "Order 12345, Germany",
        "Invoice 2500, India",
        "Ticket 482910 for the match",
        "your PIN: 1234",
        "zip the 2024 files",
        "We met in 2019, Singapore was hot",
        "Flight KL 1234 to Lagos",
        "Room 1205, Hilton",
        "It costs 1500 rupees in Mumbai",
        "the cap is 100 users",
        "Page 1200 of 3000",
        "ABC 1234 is the model",
        "There were 50000 people in Nairobi",
        "Call 0800 1234 today",
        "Kenya 2023 report",
    ];
    let s = Scanner::new(&ScanOptions { watch_terms: vec![] }).unwrap();
    let wrong: Vec<_> = no
        .iter()
        .filter_map(|t| s.detect(t).into_iter().find(|d| d.kind == Kind::Postcode).map(|d| format!("{t:?} -> {:?}", &t[d.start..d.end])))
        .collect();
    assert!(wrong.is_empty(), "wrongly flagged: {wrong:#?}");
}

#[test]
fn postcode_word_is_not_part_of_the_find() {
    let s = Scanner::new(&ScanOptions { watch_terms: vec![] }).unwrap();
    let t = "PLZ 10115";
    let d = s.detect(t).into_iter().find(|d| d.kind == Kind::Postcode).unwrap();
    assert_eq!(&t[d.start..d.end], "10115");
}

#[test]
fn everyday_numbers_in_receipts_and_newsletters_are_not_addresses() {
    let s = Scanner::new(&ScanOptions { watch_terms: vec![] }).unwrap();
    let texts = [
        "Your order #482913 has shipped. Estimated delivery: Tuesday, 3 December 2024.",
        "Subtotal: 1299 SEK\nShipping: 49 SEK\nTotal: 1348 SEK",
        "Flight AF 1234 departs Paris CDG at 10:45, arrives 12:30 Madrid, Spain",
        "Your verification code is 482913. Do not share it.",
        "We have 2500 Members in Germany and 1200 Members in France",
        "Room 1402, Floor 14, Marriott Hotel",
        "Netflix: your plan costs 1399 INR per month in India",
        "Episode 1204 Recap, United States edition",
        "Model XPS 9310 Laptop, Japan import",
        "In 2023, Nigeria grew 3%",
        "Call us on 1800 123 456, Australia wide",
        "Reference: 55012, Kenya Airways booking",
        "Seat 12A, Gate 23, Terminal 3, Singapore",
        "Lot 4521 Sold, Canada auction",
        "Battery 5000 mAh, India version",
        "Steps today: 10234, Great job!",
        "Score: 1500 Points, Brazil server",
        "Season 2024 Tickets, Mexico City",
        "Unit price 1250, Quantity 3",
        "CVE-2024-12345 fixed in version 2.3.4",
        "The 10000 Steps challenge, Philippines",
        "Download 4500 Songs, Ireland only",
        "Price: 25000 KES, Nairobi",
        "Hi all,\n1500 Tickets left\nGermany",
        "Invoice no. 20240012\nAmsterdam, Netherlands",
        "Mumbai Indians won by 45 runs, 200 Runs chased",
    ];
    let wrong: Vec<_> = texts
        .iter()
        .flat_map(|t| s.detect(t).into_iter().filter(|d| matches!(d.kind, Kind::Postcode | Kind::Address)).map(move |d| format!("{t:?} -> {:?}", &t[d.start..d.end])))
        .collect();
    assert!(wrong.is_empty(), "wrongly flagged: {wrong:#?}");
}

#[test]
fn sample_addresses_from_around_the_world_are_found() {
    for t in [
        "my London flat: 18 Elm Grove, London N7 9QP",
        "entrega em Rua das Flores 210, 01310-930 São Paulo",
        "post it to P.O. Box 48213, Nairobi, Kenya",
        "Tokyo apartment: 〒150-0042 渋谷区宇田川町3丁目12番5号",
    ] {
        assert!(has(t, Kind::Address), "no street address in {t:?}");
    }
    assert!(has("PLZ 10115", Kind::Postcode));
}
