//! The exposure scanner: finds personal information you have posted.
//!
//! Every detector is a regex plus, where possible, a real validity check
//! (IBAN checksum, Luhn for cards, digit counts for phone numbers) to keep
//! false alarms low. Results are stored as byte positions only.

use crate::Store;
use sha2::{Digest, Sha256};
use anyhow::Result;
use regex::Regex;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Credential,
    Card,
    Iban,
    WatchTerm,
    Address,
    Phone,
    Email,
    IpAddress,
    Postcode,
    CryptoWallet,
    SensitiveFile,
    IdNumber,
    Passport,
    BackupCode,
    BirthDate,
    NumberPlate,
    Location,
}

impl Kind {
    pub const ALL: [Kind; 17] = [
        Kind::Credential,
        Kind::BackupCode,
        Kind::IdNumber,
        Kind::Passport,
        Kind::Card,
        Kind::Iban,
        Kind::WatchTerm,
        Kind::Address,
        Kind::Phone,
        Kind::Email,
        Kind::Location,
        Kind::IpAddress,
        Kind::BirthDate,
        Kind::Postcode,
        Kind::SensitiveFile,
        Kind::NumberPlate,
        Kind::CryptoWallet,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Kind::Credential => "credential",
            Kind::Card => "card",
            Kind::Iban => "iban",
            Kind::WatchTerm => "watch_term",
            Kind::Address => "address",
            Kind::Phone => "phone",
            Kind::Email => "email",
            Kind::IpAddress => "ip_address",
            Kind::Postcode => "postcode",
            Kind::CryptoWallet => "crypto_wallet",
            Kind::SensitiveFile => "sensitive_file",
            Kind::IdNumber => "id_number",
            Kind::Passport => "passport",
            Kind::BackupCode => "backup_code",
            Kind::BirthDate => "birth_date",
            Kind::NumberPlate => "number_plate",
            Kind::Location => "location",
        }
    }

    pub fn from_key(s: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|k| k.key() == s)
    }

    pub fn label(self) -> &'static str {
        match self {
            Kind::Credential => "Passwords, tokens & keys",
            Kind::Card => "Payment cards",
            Kind::Iban => "Bank accounts",
            Kind::WatchTerm => "Your watch terms",
            Kind::Address => "Street addresses",
            Kind::Phone => "Phone numbers",
            Kind::Email => "Email addresses",
            Kind::IpAddress => "IP addresses",
            Kind::Postcode => "Postcodes",
            Kind::CryptoWallet => "Crypto wallets",
            Kind::SensitiveFile => "Sensitive files on this computer",
            Kind::IdNumber => "National ID numbers",
            Kind::Passport => "Passport & ID card numbers",
            Kind::BackupCode => "Backup codes & recovery phrases",
            Kind::BirthDate => "Dates of birth",
            Kind::NumberPlate => "Number plates",
            Kind::Location => "Exact locations",
        }
    }

    /// What the person should do about it, in plain words.
    pub fn advice(self) -> &'static str {
        match self {
            Kind::Credential => "Change this password or revoke this token now. Deleting the message doesn't undo the leak, because others may already have seen it.",
            Kind::Card => "Contact your bank to block or replace this card, then delete the message.",
            Kind::Iban => "A bank account number (an IBAN, or a sort code or routing number with an account number) can't empty your account on its own, but scammers use it to sound convincing, and in some countries it's enough to set up a direct debit. Delete the message unless you share this account publicly anyway.",
            Kind::WatchTerm => "You told Sompio this is sensitive. Decide whether this message needs to go.",
            Kind::Address => "Delete the message unless you're fine with everyone in this chat knowing this address.",
            Kind::Phone => "Delete the message if you don't want this number linked to your account there.",
            Kind::Email => "Check whether you want this address linked to your account there. If it's a public work address, mark it as not a problem everywhere.",
            Kind::IpAddress => "An IP address can show roughly where you are. Delete the message if this was your home connection.",
            Kind::Postcode => "A postcode narrows down where you live. Low risk on its own, more so next to a street name.",
            Kind::CryptoWallet => "Wallet addresses are public, but tying one to your account reveals its balance and history.",
            Kind::SensitiveFile => "This file's name suggests an identity document, money matters or secrets. Keep it only if you need it, ideally in an encrypted folder or password manager, and move it to the Recycle Bin if you don't.",
            Kind::IdNumber => "A national ID number (like a BSN, social security or National Insurance number, DNI, codice fiscale, PESEL or CNP) is what fraudsters need to open accounts in your name. Delete the message, and only give this number to official bodies, your employer, bank or doctor.",
            Kind::Passport => "With a passport or ID card number, someone can pose as you. Delete the message. If a photo of the document went out too, report it to your town hall or the police and consider replacing the document.",
            Kind::BackupCode => "Backup codes and recovery phrases get around your two-factor login or open a crypto wallet outright. Make new backup codes on that site now (that cancels the old ones), or move the coins to a new wallet, then delete the message.",
            Kind::BirthDate => "Your date of birth is a common security question and a key piece for identity fraud, together with your name. Delete the message if this is your own date of birth.",
            Kind::Location => "This photo or map link shows an exact spot. If it's your home, school or work, delete the post. Before sharing a photo again, remove its location. On Windows: right-click the file, Properties, Details, Remove Properties and Personal Information. On a Mac, in Photos: Image, Location, Hide Location.",
            Kind::NumberPlate => "A number plate can be tied to your car, and so to where you park and drive. Delete the message unless it's fine for this chat to know it.",
        }
    }

    /// 3 = high, 2 = medium, 1 = low
    pub fn severity(self) -> i64 {
        match self {
            Kind::Credential | Kind::Card | Kind::Iban | Kind::WatchTerm | Kind::Address => 3,
            Kind::IdNumber | Kind::Passport | Kind::BackupCode => 3,
            Kind::Phone | Kind::Email | Kind::IpAddress | Kind::SensitiveFile | Kind::BirthDate | Kind::Location => 2,
            Kind::Postcode | Kind::CryptoWallet | Kind::NumberPlate => 1,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScanOptions {
    /// Things only you know are sensitive: your real name, street, employer, school...
    pub watch_terms: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Detection {
    pub kind: Kind,
    pub start: usize,
    pub end: usize,
}

pub struct Scanner {
    /// Each detector, and whether it only matches text with letters outside plain Latin.
    detectors: Vec<(Kind, Regex, bool)>,
    watch: Option<Regex>,
}

impl Scanner {
    pub fn new(opts: &ScanOptions) -> Result<Self> {
        let d = |k: Kind, p: &str| -> Result<(Kind, Regex)> { Ok((k, Regex::new(p)?)) };
        let detectors = vec![
            d(Kind::Credential, r"\b[A-Za-z0-9_-]{24,28}\.[A-Za-z0-9_-]{6}\.[A-Za-z0-9_-]{27,40}\b")?, // Discord token / JWT-like
            d(Kind::Credential, r"\beyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}")?, // JWT
            d(Kind::Credential, r"\b(?:ghp|gho|ghu|ghs)_[A-Za-z0-9]{36}\b|\bgithub_pat_[A-Za-z0-9_]{50,}\b")?,
            d(Kind::Credential, r"\bAKIA[0-9A-Z]{16}\b|\bsk-[A-Za-z0-9_-]{20,}|\bxox[abprs]-[A-Za-z0-9-]{10,}")?,
            d(Kind::Credential, r"(?i)\b(?:password|passwd|pwd|pass|wachtwoord|passwort|parola|parolă)\s*(?:is\s*)?[:=]\s*\S{4,}")?,
            d(Kind::Card, r"\b[0-9](?:[ -]?[0-9]){12,18}\b")?,
            d(Kind::Iban, r"\b[A-Z]{2}[0-9]{2}(?: ?[A-Z0-9]{4}){2,7}(?: ?[A-Z0-9]{1,3})?\b")?,
            // UK: sort code and account number together
            d(Kind::Iban, r"(?i)\bsort\s*code\W{0,4}[0-9]{2}[- ]?[0-9]{2}[- ]?[0-9]{2}\b[^\n]{0,25}?\b(?:account|acc|a/c)(?:\s*(?:no|number|nr))?\.?\W{0,4}[0-9]{8}\b")?,
            // US: routing number and account number together (either order)
            d(Kind::Iban, r"(?i)\b(?:routing|aba|rtn)(?:\s*(?:no|number|#))?\.?\W{0,4}[0-9]{9}\b[^\n]{0,25}?\b(?:account|acct|acc)(?:\s*(?:no|number|#))?\.?\W{0,4}[0-9]{6,17}\b")?,
            d(Kind::Iban, r"(?i)\b(?:account|acct)(?:\s*(?:no|number|#))?\.?\W{0,4}[0-9]{6,17}\b[^\n]{0,25}?\b(?:routing|aba|rtn)(?:\s*(?:no|number|#))?\.?\W{0,4}[0-9]{9}\b")?,
            // Dutch / German / Belgian compound street names: "Prinsengracht 263", "Hauptstraße 5"
            d(Kind::Address, r"(?i)\b\p{L}{2}[\p{L}'-]*(?:straat|laan|weg|gracht|plein|kade|dijk|singel|steeg|strasse|straße|gasse|allee|platz|damm|ufer|gatan|gata|gade|vej|vägen|veien|katu)\s+[0-9]{1,4}[a-z]?\b")?,
            // English: "221B Baker Street" (capitalised name words, to skip "5 minutes down the road")
            d(Kind::Address, r"\b[0-9]{1,5}[A-Za-z]?\s+(?:[A-Z][\p{L}'.-]*\s+){1,3}(?i:street|st|avenue|ave|road|rd|boulevard|blvd|lane|ln|drive|dr|way|court|ct|place|pl|square|sq|crescent|terrace|parkway|pkwy|highway|hwy|grove|gardens|mews|row)\b\.?")?,
            // Romanian: "Strada Lipscani nr. 12", "Bd. Unirii 5"
            d(Kind::Address, r"(?i)\b(?:strada|str\.|bulevardul|bd\.|calea|aleea|[șşs]oseaua|splaiul)\s+\p{L}[\p{L} .-]{1,40}?,?\s*(?:nr\.?\s*)?[0-9]{1,4}\b")?,
            // French: "12 rue de Rivoli", "5 bis avenue Foch"
            d(Kind::Address, r"(?i:\b[0-9]{1,4}(?:\s?(?:bis|ter))?,?\s+(?:rue|avenue|av\.|boulevard|bd|place|chemin|all[ée]e|impasse|quai|cours)\s+(?:de la |de l'|du |des |de |d')?)\p{Lu}[\p{L}'-]+")?,
            // Spanish, Italian, Portuguese, Polish: "Calle Mayor 5", "Via Roma, 10", "Rua Augusta 24", "ul. Długa 7"
            d(Kind::Address, r"(?i:\b(?:calle|c/|avenida|avda\.|plaza|paseo|carrer|via|viale|piazza|corso|rua|avenida|pra[çc]a|ulica|ul\.|aleja|al\.)\s+(?:de la |del |de |da |das |do |dos |della |dei |di )?)\p{Lu}[\p{L}'.-]*(?:\s\p{Lu}[\p{L}'.-]*){0,3},?\s*(?:n[º°o.]?\s*)?[0-9]{1,4}[a-zA-Z]?\b")?,
            // House number first, as booking sites write it: "305 Via Vittorio Emanuele II", "12 Calle Mayor"
            d(Kind::Address, r"\b[0-9]{1,4}[a-zA-Z]?,?\s+(?:Via|Viale|Piazza|Corso|Largo|Calle|Avenida|Avda\.|Plaza|Paseo|Carrer|Rua|Travessa|Pra[çc]a)\s+(?:de la |del |de |da |do |dos |della |dei |di )?\p{Lu}[\p{L}'.-]*(?:\s\p{Lu}[\p{L}'.-]*){0,3}")?,
            // Spanish and Portuguese name-first with "Av.": "Av. Insurgentes Sur 1602", "Av. Paulista, 1578"
            d(Kind::Address, r"\bAv\.\s+\p{Lu}[\p{L}'.-]*(?:\s\p{Lu}[\p{L}'.-]*){0,3},?\s*(?:n[º°o.]?\s*)?[0-9]{1,5}\b")?,
            // Colombia: "Calle 93 # 11-26", "Carrera 7 # 71-21"
            d(Kind::Address, r"(?i)\b(?:calle|carrera|cra\.?|cl\.?|kr\.?|avenida|transversal|diagonal)\s+[0-9]{1,3}\s?[a-z]?\s*(?:bis\s*)?(?:#|no\.?)\s*[0-9]{1,3}\s?[a-z]?\s*-\s*[0-9]{1,3}\b")?,
            // P.O. boxes, the usual address in much of Africa and the Middle East
            d(Kind::Address, r"(?i)\b(?:p\.?\s?o\.?\s?box|post office box|private bag|postfach|postbus|apartado(?: de correos| postal)?|caixa postal|casilla(?: de correo)?|bo[iî]te postale|skrytka pocztowa)\s*:?\s*(?:no\.?\s*)?[a-z]?[0-9]{1,6}\b|\bB\.P\.\s?[0-9]{1,6}\b|а/я\s?[0-9]{1,6}|ص\.\s?ب\.?\s?[0-9٠-٩]{1,6}")?,
            // House and plot numbers, as in South Asia and Africa: "House No. 12", "Plot 45", "Flat 4B"
            d(Kind::Address, r"(?i)\b(?:house\s?(?:no\.?|number|#)|plot\s?(?:no\.?|#)?|flat\s?(?:no\.?|#)|door\s?no\.?)\s*[0-9]{1,5}[a-z]?\b")?,
            // Malaysia and Indonesia: "Jalan Sudirman No. 5", "Jl. Thamrin 10"
            d(Kind::Address, r"\b(?:Jalan|Jl\.|Jln\.?)\s+\p{Lu}[\p{L}.']*(?:\s\p{Lu}[\p{L}.']*){0,3},?\s*(?:No\.?\s*)?[0-9]{1,4}\b")?,
            // Vietnam: "123 Đường Lê Lợi", "45 Phố Huế"
            d(Kind::Address, r"\b[0-9]{1,4}[a-zA-Z]?\s+(?:Đường|đường|Phố|phố)\s+\p{Lu}[\p{L}]*(?:\s\p{Lu}[\p{L}]*){0,3}")?,
            // Russian and Ukrainian: "ул. Тверская, д. 7", "вул. Хрещатик, 22"
            d(Kind::Address, r"(?i:\b(?:ул\.|улица|проспект|пр-т|пер\.|переулок|шоссе|бульвар|б-р|наб\.|набережная|вул\.|вулиця)\s*)\p{Lu}[\p{L}.-]*(?:\s\p{L}[\p{L}.-]*){0,2},?\s*(?:д\.|дом|буд\.)?\s*[0-9]{1,4}")?,
            // Greek: "Οδός Ερμού 12", "Λεωφ. Κηφισίας 44"
            d(Kind::Address, r"(?i:\b(?:οδός|οδ\.|λεωφόρος|λεωφ\.)\s+)\p{Greek}+(?:\s\p{Greek}+){0,2}\s+[0-9]{1,4}\b")?,
            // Arabic and Persian: "١٢ شارع التحرير", "شارع الملك فهد 25", "خیابان ولیعصر، پلاک ۱۲"
            d(Kind::Address, r"[0-9٠-٩]{1,4}\s+شارع\s+\p{Arabic}+(?:\s\p{Arabic}+){0,3}|(?:شارع|خیابان|کوچه)\s+\p{Arabic}+(?:\s\p{Arabic}+){0,3}[،,]?\s*(?:پلاک\s*|رقم\s*)?[0-9٠-٩۰-۹]{1,4}")?,
            // Hebrew: "רחוב הרצל 12"
            d(Kind::Address, r"(?:רחוב|רח')\s+\p{Hebrew}+(?:\s\p{Hebrew}+){0,2}\s+[0-9]{1,4}")?,
            // Chinese, Japanese and Korean: "建国路88号", "1丁目2番地3", "테헤란로 123"
            d(Kind::Address, r"\p{Han}{1,12}(?:路|街|大道|道|巷|弄)[0-9０-９一二三四五六七八九十百]{1,6}号")?,
            d(Kind::Address, r"[0-9０-９一二三四五六七八九十]{1,3}丁目[0-9０-９]{1,4}(?:番地?|[-－])[0-9０-９]{0,4}")?,
            d(Kind::Address, r"\p{Hangul}{1,10}(?:로|길|대로)\s?[0-9]{1,4}(?:번길\s?[0-9]{1,4})?")?,
            // Thai: "123/45 ถนนสุขุมวิท", "ซอยสุขุมวิท 11"
            d(Kind::Address, r"[0-9]{1,5}(?:/[0-9]{1,4})?\s*(?:ถนน|ซอย)\p{Thai}+|(?:ถนน|ซอย)\p{Thai}+\s?[0-9]{1,4}")?,
            // Hungarian and Turkish: "Andrássy út 12", "Atatürk Caddesi No: 5"
            d(Kind::Address, r"\b\p{Lu}[\p{L}.-]+\s+(?:utca|út|tér|körút)\s+[0-9]{1,4}\b")?,
            d(Kind::Address, r"\b\p{Lu}[\p{L}]+\s+(?i:caddesi|cad\.|sokak|sokağı|sk\.|bulvarı|blv\.)\s*(?i:no)\.?\s*:?\s*[0-9]{1,4}\b")?,
            d(Kind::Phone, r"(?:\+|\b00)[1-9][0-9 ().-]{7,18}[0-9]")?,
            d(Kind::Phone, r"\b0[1-9][0-9 -]{7,12}[0-9]\b")?,
            d(Kind::Phone, r"\(?\b[2-9][0-9]{2}\)?[ .-][0-9]{3}[ .-][0-9]{4}\b")?,
            d(Kind::Email, r"(?i)\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b")?,
            d(Kind::IpAddress, r"\b(?:(?:25[0-5]|2[0-4][0-9]|1?[0-9]?[0-9])\.){3}(?:25[0-5]|2[0-4][0-9]|1?[0-9]?[0-9])\b")?,
            d(Kind::Postcode, r"\b[1-9][0-9]{3} ?[A-Z]{2}\b")?, // Dutch postcode
            // Postcodes written only in numbers, with an optional country prefix: 95124, 75001, 00-950,
            // 1100-148, 01310-100, 114 55, D-10115. Only counted next to a city, street or country.
            d(Kind::Postcode, r"\b(?:[A-Z]{1,2}-)?(?:[0-9]{5}-[0-9]{5}|[0-9]{5}-[0-9]{3}|[0-9]{4}-[0-9]{3}|[0-9]{3}-[0-9]{4}|[0-9]{2}-[0-9]{3}|[0-9]{3} [0-9]{3}|[0-9]{3} [0-9]{2}|[0-9]{4,7})\b")?,
            // After the word for it, in any of many languages: "Postcode: 3811 AB", "PLZ 10115", "邮编 100022"
            (Kind::Postcode, Regex::new(&format!(r"{POSTCODE_WORDS}[A-Z0-9](?:[A-Z0-9 -]{{1,8}}[A-Z0-9])?\b"))?),
            // Malta (VLT 1117), Argentina (C1425DKF), Ghana's digital address (GA-123-4567)
            d(Kind::Postcode, r"\b[A-Z]{3} [0-9]{4}\b")?,
            d(Kind::Postcode, r"\b[A-HJ-NP-Z][0-9]{4}[A-Z]{3}\b")?,
            d(Kind::Postcode, r"\b[A-Z]{2}-[0-9]{3,4}-[0-9]{4}\b")?,
            // UK (SW1A 2AA), Canada (M5V 3L9), Ireland (D02 X285)
            d(Kind::Postcode, r"\b[A-Z]{1,2}[0-9][A-Z0-9]? [0-9][ABD-HJLNP-UW-Z]{2}\b")?,
            d(Kind::Postcode, r"\b[ABCEGHJ-NPRSTVXY][0-9][ABCEGHJ-NPRSTV-Z] [0-9][ABCEGHJ-NPRSTV-Z][0-9]\b")?,
            d(Kind::Postcode, r"\b(?:[AC-FHKNPRTV-Y][0-9]{2}|D6W) [0-9AC-FHKNPRTV-Y]{4}\b")?,
            // US state + ZIP after the city ("Austin, TX 78701"), Australian state + postcode ("Sydney NSW 2000")
            d(Kind::Postcode, r"\b(?:AL|AK|AZ|AR|CA|CO|CT|DE|DC|FL|GA|HI|ID|IL|IN|IA|KS|KY|LA|ME|MD|MA|MI|MN|MS|MO|MT|NE|NV|NH|NJ|NM|NY|NC|ND|OH|OK|OR|PA|RI|SC|SD|TN|TX|UT|VT|VA|WA|WV|WI|WY|PR) [0-9]{5}(?:-[0-9]{4})?\b")?,
            d(Kind::Postcode, r"\b(?:NSW|VIC|QLD|TAS|ACT) [0-9]{4}\b")?,
            // Japan: 〒100-0001
            d(Kind::Postcode, r"〒 ?[0-9]{3}-[0-9]{4}")?,
            d(Kind::CryptoWallet, r"\bbc1[a-z0-9]{25,59}\b|\b0x[a-fA-F0-9]{40}\b")?,
            // Dutch BSN, only next to its name (nine-digit numbers are everywhere); checked with the eleven test
            d(Kind::IdNumber, r"(?i)\b(?:bsn|burgerservicenummer|sofi-?nummer|citizen service number)\b\W{0,5}(?:(?:is|nr|nummer|number)\.?\s*:?\s*)?[0-9](?:[ .]?[0-9]){8}\b")?,
            // Other countries' numbers next to their name: Polish PESEL, French social security (NIR),
            // German tax ID, Belgian national number
            d(Kind::IdNumber, r"(?i)\b(?:pesel|num[ée]ro de s[ée]curit[ée] sociale|s[ée]curit[ée] sociale|num[ée]ro de s[ée]cu|nir|steuer-?id|steueridentifikationsnummer|steuer-identifikationsnummer|rijksregisternummer|num[ée]ro national|national number)\b\W{0,5}(?:(?:is|nr|no|number|nummer|num[ée]ro)\.?\s*:?\s*)?[0-9](?:[ .-]?[0-9]){10,14}\b")?,
            // US social security number, 123-45-6789
            d(Kind::IdNumber, r"\b[0-8][0-9]{2}-[0-9]{2}-[0-9]{4}\b")?,
            // UK National Insurance number, AB 12 34 56 C
            d(Kind::IdNumber, r"\b[A-CEGHJ-PR-TW-Z][A-CEGHJ-NPR-TW-Z] ?[0-9]{2} ?[0-9]{2} ?[0-9]{2} ?[A-D]\b")?,
            // Spanish DNI (12345678Z) and NIE (X1234567L), with their check letter
            d(Kind::IdNumber, r"\b(?:[0-9]{8}|[XYZ]-?[0-9]{7})-?[A-HJ-NP-TV-Z]\b")?,
            // Italian codice fiscale: RSSMRA85T10A562S
            d(Kind::IdNumber, r"\b[A-Z]{6}[0-9LMNPQRSTUV]{2}[ABCDEHLMPRST][0-9LMNPQRSTUV]{2}[A-Z][0-9LMNPQRSTUV]{3}[A-Z]\b")?,
            // Romanian CNP: 13 digits with a check digit and a real birth date inside
            d(Kind::IdNumber, r"\b[1-8][0-9]{12}\b")?,
            d(Kind::Passport, r"(?i:\b(?:passport|paspoort|pa[sș]aport|reisepass|id[ -]?card|identity card|identiteitskaart|id-kaart)(?:\s+(?:no|nr|number|nummer|num[aă]r))?\.?\s*(?:is\s+)?[:#]?\s*)[A-Z0-9]{6,9}\b")?,
            d(Kind::BackupCode, r"(?i:\b(?:backup|recovery|herstel|reserve|2fa|two[- ]factor)[ -]?(?:codes?|keys?)\b[^\n]{0,20}?)[A-Za-z0-9]{4,6}[- ]?[A-Za-z0-9]{4,6}\b")?,
            d(Kind::BackupCode, r"(?i)\b(?:seed|recovery|secret|mnemonic|herstel)\s*(?:phrase|words|zin)\b\W{0,10}(?:is\s*)?:?\s*(?:[a-z]{3,8}\s+){11,23}[a-z]{3,8}\b")?,
            d(Kind::BirthDate, r"(?i:\b(?:born(?:\s+on)?|date of birth|dob|d\.o\.b\.?|birthday(?:\s+is)?|geboren(?:\s+op|\s+am)?|geboortedatum|geb\.|n[ăa]scut[ăa]?(?:\s+(?:la|pe|[îi]n))?|data na[șs]terii|geburtsdatum)\s*:?\s*)(?:[0-3]?[0-9][-/. ][01]?[0-9][-/. ](?:19|20)[0-9]{2}|(?:19|20)[0-9]{2}-[01][0-9]-[0-3][0-9]|[0-3]?[0-9](?:st|nd|rd|th)?\s+(?i:jan|feb|maart|mar|mrt|apr|may|mei|mai|jun|iun|jul|iul|aug|sep|oct|okt|nov|dec|ian)[a-zăâîșț]*\.?,?\s+(?:19|20)[0-9]{2}|(?i:jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec)[a-z]*\.?\s+[0-3]?[0-9](?:st|nd|rd|th)?,?\s+(?:19|20)[0-9]{2})\b")?,
            // The place a photo was taken (written by the computer scan), or a map link to an exact spot
            d(Kind::Location, r"\[Photo location: -?[0-9]{1,2}\.[0-9]{3,6}, -?[0-9]{1,3}\.[0-9]{3,6}\]")?,
            d(Kind::Location, r"(?i)\b(?:maps\.google\.[a-z.]+|google\.[a-z.]+/maps|maps\.apple\.com|openstreetmap\.org)/?\S*?[@=/#](?:map=[0-9]{1,2}/)?-?[0-9]{1,2}\.[0-9]{4,}[,/]-?[0-9]{1,3}\.[0-9]{4,}")?,
            // Dutch number plates in their official dash patterns (no vowels since 1978), or after the word
            d(Kind::NumberPlate, r"\b(?:[A-Z]{2}-[0-9]{2}-[A-Z]{2}|[A-Z]{2}-[A-Z]{2}-[0-9]{2}|[0-9]{2}-[A-Z]{2}-[A-Z]{2}|[0-9]{2}-[A-Z]{3}-[0-9]|[0-9]-[A-Z]{3}-[0-9]{2}|[A-Z]{2}-[0-9]{3}-[A-Z]|[A-Z]-[0-9]{3}-[A-Z]{2}|[A-Z]{3}-[0-9]{2}-[A-Z]|[A-Z]-[0-9]{2}-[A-Z]{3}|[0-9]-[A-Z]{2}-[0-9]{3}|[0-9]{3}-[A-Z]{2}-[0-9])\b")?,
            d(Kind::NumberPlate, r"(?i:\b(?:kenteken|licen[cs]e plate|number plate|plate number|nummernschild|num[aă]r de [îi]nmatriculare)(?:\s+(?:is|nr|number))?\s*:?\s*)[A-Z0-9]{1,3}[- ]?[A-Z0-9]{2,4}[- ]?[A-Z0-9]{1,3}\b")?,
        ];

        let terms: Vec<String> = opts
            .watch_terms
            .iter()
            .map(|t| t.trim())
            .filter(|t| t.chars().count() >= 2)
            .map(|t| {
                let esc = regex::escape(t);
                let word = |c: Option<char>| c.is_some_and(|c| c.is_alphanumeric() || c == '_');
                let pre = if word(t.chars().next()) { r"\b" } else { "" };
                let post = if word(t.chars().last()) { r"\b" } else { "" };
                format!("{pre}{esc}{post}")
            })
            .collect();
        let watch = if terms.is_empty() {
            None
        } else {
            Some(Regex::new(&format!("(?i)(?:{})", terms.join("|")))?)
        };
        let detectors: Vec<(Kind, Regex, bool)> = detectors.into_iter().map(|(k, re)| {
            let script_only = SCRIPT_ONLY.iter().any(|w| re.as_str().contains(w));
            (k, re, script_only)
        }).collect();
        Ok(Self { detectors, watch })
    }

    pub fn detect(&self, text: &str) -> Vec<Detection> {
        let mut found = Vec::new();
        if let Some(w) = &self.watch {
            found.extend(w.find_iter(text).map(|m| Detection { kind: Kind::WatchTerm, start: m.start(), end: m.end() }));
        }
        let ascii = text.is_ascii();
        for (kind, re, script_only) in &self.detectors {
            if ascii && *script_only {
                continue; // No Cyrillic, Arabic, Chinese… in this text, so these can't match.
            }
            for m in re.find_iter(text) {
                if validate(*kind, text, m.start(), m.end()) {
                    // Keep only the postcode itself, not the word before it ("PLZ 10115" → "10115")
                    let skip = if *kind == Kind::Postcode { postcode_keyword(m.as_str()).map_or(0, |k| k.len()) } else { 0 };
                    found.push(Detection { kind: *kind, start: m.start() + skip, end: m.end() });
                }
            }
        }
        dedupe(found)
    }
}

/// Drops exact duplicates, and anything sitting inside a match that is at least as serious
/// (the digits inside an IBAN are not also a phone number).
fn dedupe(mut v: Vec<Detection>) -> Vec<Detection> {
    v.sort_by_key(|d| (d.start, std::cmp::Reverse(d.end), std::cmp::Reverse(d.kind.severity())));
    v.dedup_by(|a, b| a.kind == b.kind && a.start == b.start && a.end == b.end);
    let snapshot = v.clone();
    v.retain(|d| {
        !snapshot.iter().any(|o| {
            o != d
                && o.start <= d.start
                && o.end >= d.end
                && (o.end - o.start) > (d.end - d.start)
                && o.kind.severity() >= d.kind.severity()
        })
    });
    v
}

fn digits(s: &str) -> Vec<u32> {
    s.chars().filter_map(|c| c.to_digit(10)).collect()
}

fn luhn(ds: &[u32]) -> bool {
    let sum: u32 = ds
        .iter()
        .rev()
        .enumerate()
        .map(|(i, &d)| if i % 2 == 1 { let x = d * 2; if x > 9 { x - 9 } else { x } } else { d })
        .sum();
    sum % 10 == 0
}

fn iban_valid(s: &str) -> bool {
    let compact: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    if !(15..=34).contains(&compact.len()) {
        return false;
    }
    let rearranged = format!("{}{}", &compact[4..], &compact[..4]);
    let mut rem: u64 = 0;
    for c in rearranged.chars() {
        let v = match c {
            '0'..='9' => c as u64 - '0' as u64,
            'A'..='Z' => c as u64 - 'A' as u64 + 10,
            _ => return false,
        };
        rem = if v >= 10 { (rem * 100 + v) % 97 } else { (rem * 10 + v) % 97 };
    }
    rem == 1
}

fn validate(kind: Kind, text: &str, start: usize, end: usize) -> bool {
    let m = &text[start..end];
    let before = &text[..start];
    let after = &text[end..];
    match kind {
        Kind::Card => {
            // Discord mentions and links are full of long numbers; skip those.
            if before.ends_with("<@") || before.ends_with("<@!") || before.ends_with("<@&")
                || before.ends_with("<#") || before.ends_with(':') || before.ends_with('/')
                || after.starts_with('>')
            {
                return false;
            }
            let ds = digits(m);
            let separated = m.contains(' ') || m.contains('-');
            let len_ok = if separated { (13..=19).contains(&ds.len()) } else { ds.len() == 15 || ds.len() == 16 };
            let prefix: String = ds.iter().take(4).map(|d| char::from_digit(*d, 10).unwrap()).collect();
            let p2: u32 = prefix[..2].parse().unwrap_or(0);
            let p4: u32 = prefix.parse().unwrap_or(0);
            let brand = prefix.starts_with('4')
                || (51..=55).contains(&p2)
                || (2221..=2720).contains(&p4)
                || p2 == 34 || p2 == 37
                || p4 == 6011 || p2 == 65 || p2 == 35;
            len_ok && brand && luhn(&ds)
        }
        Kind::Iban => {
            let lower = m.to_lowercase();
            if lower.contains("routing") || lower.contains("aba") || lower.contains("rtn") {
                // The 9-digit US routing number has its own check digit
                let routing = m
                    .split(|c: char| !c.is_ascii_digit())
                    .find(|p| p.len() == 9)
                    .map(|p| p.chars().map(|c| c as u32 - '0' as u32).collect::<Vec<_>>());
                return routing.is_some_and(|d| {
                    (3 * (d[0] + d[3] + d[6]) + 7 * (d[1] + d[4] + d[7]) + (d[2] + d[5] + d[8])) % 10 == 0 && d.iter().any(|x| *x != 0)
                });
            }
            lower.contains("sort") || iban_valid(m)
        }
        Kind::Phone => {
            let n = digits(m).len();
            let next_is_digit = after.chars().next().is_some_and(|c| c.is_ascii_digit());
            (9..=15).contains(&n) && !next_is_digit && !before.ends_with(['/', '=', '#'])
        }
        Kind::IpAddress => {
            if before.ends_with(['v', 'V', '.']) || after.starts_with('.') {
                return false; // version numbers like v1.2.3.4
            }
            let o: Vec<u32> = m.split('.').filter_map(|p| p.parse().ok()).collect();
            !(o[0] == 0 || o[0] == 10 || o[0] == 127 || o[0] >= 224
                || (o[0] == 192 && o[1] == 168)
                || (o[0] == 172 && (16..=31).contains(&o[1]))
                || (o[0] == 169 && o[1] == 254))
        }
        Kind::Postcode => {
            let tail = m.rsplit(' ').next().unwrap_or("");
            let letters = if m.len() >= 2 && m.is_char_boundary(m.len() - 2) { &m[m.len() - 2..] } else { "" };
            let prev = before.trim_end_matches([' ', '\t']).chars().next_back();
            let next = after.trim_start_matches([' ', '\t']).chars().next();
            if !postcode_shape_ok(m, tail, before, after) {
                return false;
            }
            // The same line and the three lines above it: a company's signature puts its name a
            // line or two above the address ("Booking.com / Oosterdokskade 163 / 1011 DL Amsterdam").
            let lines: Vec<&str> = before.rsplit('\n').take(4).collect();
            let mut near = lines.join("\n");
            let mut cut = near.len().saturating_sub(240);
            while !near.is_char_boundary(cut) {
                cut += 1;
            }
            near = near[cut..].to_lowercase();
            let brand_line = lines.iter().skip(1).any(|l| is_brand_line(l));
            !matches!(letters, "SA" | "SD" | "SS" | "AM" | "PM" | "GB" | "MB" | "KB" | "TB" | "HZ")
                && !after.starts_with(|c: char| c.is_alphanumeric())
                // The year of a date ("07/06/2018 MC") or part of a longer number or code.
                && !matches!(prev, Some('/' | '.' | '#' | '_') | Some('0'..='9'))
                // A dash only on its own, as in "Mumbai - 400 053", never as in "12-1017"
                && !(prev == Some('-') && !before.trim_end_matches([' ', '\t']).trim_end_matches('-').ends_with([' ', '\t']))
                // Also when something sits between the day and month and the year, as in the
                // table cells of HTML emails ("07/06/ | 2018 MC", or a line break in between).
                && !date_part_before(before)
                && !next.is_some_and(|c| c.is_ascii_digit())
                // A company's own address, as in an email footer ("Booking.com B.V., ... 1012 SJ").
                && ![" b.v.", " bv,", " bv ", " n.v.", " ltd", " gmbh", " inc.", "kvk", "chamber of commerce", "registered office", "statutaire zetel"]
                    .iter()
                    .any(|w| near.contains(w))
                && !brand_line
        }
        Kind::IdNumber => id_valid(m, before, after),
        Kind::Passport => {
            let tail = m.rsplit(|c: char| !c.is_ascii_alphanumeric()).next().unwrap_or("");
            tail.chars().filter(|c| c.is_ascii_digit()).count() >= 2
        }
        Kind::BackupCode => {
            if m.to_lowercase().contains("phrase") || m.to_lowercase().contains("words") || m.to_lowercase().contains("zin") {
                // A real recovery phrase is only word-list words, never everyday ones like "the" or "you"
                const NOT_SEED: [&str; 14] = ["the", "and", "you", "your", "should", "with", "this", "that", "for", "are", "was", "have", "what", "don"];
                let words: Vec<&str> = m.split_whitespace().collect();
                let tail = &words[words.len().saturating_sub(12)..];
                return !tail.iter().any(|w| NOT_SEED.contains(&w.to_lowercase().as_str()));
            }
            let code = m.rsplit(|c: char| c.is_whitespace() || c == ':').next().unwrap_or("");
            code.chars().any(|c| c.is_ascii_digit()) && code.chars().any(|c| c.is_ascii_alphabetic())
                || code.chars().filter(|c| c.is_ascii_digit()).count() >= 8
        }
        Kind::NumberPlate => {
            // The plate itself: everything after the last lowercase letter or colon of the keyword
            let cut = m.char_indices().filter(|(_, c)| c.is_lowercase() || *c == ':').last().map(|(i, c)| i + c.len_utf8()).unwrap_or(0);
            let tail = &m[cut..];
            let letters = tail.chars().filter(|c| c.is_ascii_alphabetic()).count();
            let nums = tail.chars().filter(|c| c.is_ascii_digit()).count();
            let keyword = m.chars().any(|c| c.is_lowercase());
            letters >= 1 && nums >= 1 && (5..=8).contains(&(letters + nums))
                && (keyword || !tail.chars().any(|c| "AEIOU".contains(c)))
                && !before.ends_with(|c: char| c.is_ascii_alphanumeric() || c == '-')
                && !after.starts_with('-')
        }
        _ => true,
    }
}

fn id_valid(m: &str, before: &str, after: &str) -> bool {
    let ds = digits(m);
    let lower = m.to_lowercase();
    let has = |w: &str| lower.contains(w);
    // Numbers that came with their name
    if has("pesel") {
        const W: [u32; 10] = [1, 3, 7, 9, 1, 3, 7, 9, 1, 3];
        return ds.len() == 11 && (10 - ds[..10].iter().zip(W).map(|(d, w)| d * w).sum::<u32>() % 10) % 10 == ds[10];
    }
    if has("sécu") || has("secu") || has("nir") {
        return ds.len() == 15 && matches!(ds[0], 1 | 2) && {
            let n: u64 = ds[..13].iter().fold(0, |a, d| a * 10 + *d as u64);
            97 - n % 97 == (ds[13] * 10 + ds[14]) as u64
        };
    }
    if has("steuer") {
        return ds.len() == 11 && ds[0] != 0;
    }
    if has("rijksregister") || has("national") {
        return ds.len() == 11 && {
            let n: u64 = ds[..9].iter().fold(0, |a, d| a * 10 + *d as u64);
            let check = (ds[9] * 10 + ds[10]) as u64;
            97 - n % 97 == check || 97 - (2_000_000_000 + n) % 97 == check
        };
    }
    if has("bsn") || has("burgerservice") || has("sofi") || has("citizen") {
        return ds.len() == 9 && elfproef(&ds);
    }
    let alone = !before.ends_with(|c: char| c.is_ascii_alphanumeric() || c == '-') && !after.starts_with(|c: char| c == '-' || c.is_ascii_alphanumeric());
    if !alone {
        return false;
    }
    let compact: String = m.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    let letters = compact.chars().filter(|c| c.is_ascii_alphabetic()).count();
    // Italian codice fiscale: 16 characters, the last a check letter
    if compact.len() == 16 && letters >= 9 {
        return codice_fiscale_valid(&compact);
    }
    // Spanish DNI / NIE: the letter is the number modulo 23
    if (compact.len() == 9) && compact.ends_with(|c: char| c.is_ascii_alphabetic()) && letters <= 2 {
        let num: String = compact[..8].replace('X', "0").replace('Y', "1").replace('Z', "2");
        let Ok(n) = num.parse::<u32>() else { return false };
        return "TRWAGMYFPDXBNJZSQVHLCKE".as_bytes()[(n % 23) as usize] as char == compact.chars().last().unwrap();
    }
    // UK National Insurance number: two letters, six digits, one letter; some prefixes are never used
    if letters == 3 && compact.len() == 9 {
        return !["BG", "GB", "NK", "KN", "TN", "NT", "ZZ"].contains(&&compact[..2]);
    }
    if m.contains('-') && ds.len() == 9 {
        // US SSN: never 000, 666 or 9xx in the first part, 00 or 0000 in the others
        let area = ds[0] * 100 + ds[1] * 10 + ds[2];
        return area != 0 && area != 666 && !(ds[3] == 0 && ds[4] == 0) && ds[5..].iter().any(|d| *d != 0);
    }
    ds.len() == 13 && cnp_valid(&ds)
}

/// The Italian tax code's last letter is a check on the first 15 characters.
fn codice_fiscale_valid(cf: &str) -> bool {
    const ODD: [u32; 26] = [1, 0, 5, 7, 9, 13, 15, 17, 19, 21, 2, 4, 18, 20, 11, 3, 6, 8, 12, 14, 16, 10, 22, 25, 24, 23];
    let val = |c: char| -> (u32, u32) {
        let i = if c.is_ascii_digit() { c as u32 - '0' as u32 } else { c as u32 - 'A' as u32 };
        (i, ODD[i as usize])
    };
    let b: Vec<char> = cf.chars().collect();
    if b.len() != 16 {
        return false;
    }
    let sum: u32 = b[..15].iter().enumerate().map(|(i, c)| { let (even, odd) = val(*c); if i % 2 == 0 { odd } else { even } }).sum();
    (b'A' + (sum % 26) as u8) as char == b[15]
}

/// The Dutch eleven test for a BSN: 9·d1 + 8·d2 + … + 2·d8 − d9 is a multiple of 11.
fn elfproef(ds: &[u32]) -> bool {
    if ds.len() != 9 || ds.iter().all(|d| *d == ds[0]) {
        return false;
    }
    let sum: i64 = ds[..8].iter().enumerate().map(|(i, d)| (9 - i as i64) * *d as i64).sum::<i64>() - ds[8] as i64;
    sum % 11 == 0
}

/// Romanian personal numeric code: sex/century digit, a real date, and the check digit.
fn cnp_valid(ds: &[u32]) -> bool {
    if ds.len() != 13 {
        return false;
    }
    let month = ds[3] * 10 + ds[4];
    let day = ds[5] * 10 + ds[6];
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return false;
    }
    const W: [u32; 12] = [2, 7, 9, 1, 4, 6, 3, 5, 8, 2, 7, 9];
    let r = ds[..12].iter().zip(W).map(|(d, w)| d * w).sum::<u32>() % 11;
    ds[12] == if r == 10 { 1 } else { r }
}


// ---------------------------------------------------------------------------------
// Masking helpers (shared with query.rs)

/// Hides a sensitive span but keeps the first character of each word, so it stays recognisable:
/// "jan@mail.com" -> "j••@m•••.c••".
pub(crate) fn mask_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_alnum = false;
    for ch in s.chars() {
        out.push(if ch.is_alphanumeric() && prev_alnum { '•' } else { ch });
        prev_alnum = ch.is_alphanumeric();
    }
    out
}

/// Sorted, merged byte ranges that are safe to slice on.
pub(crate) fn merge_spans(content: &str, mut spans: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
    spans.retain(|&(s, e)| s < e && e <= content.len() && content.is_char_boundary(s) && content.is_char_boundary(e));
    spans.sort();
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (s, e) in spans {
        match merged.last_mut() {
            Some(last) if s <= last.1 => last.1 = last.1.max(e),
            _ => merged.push((s, e)),
        }
    }
    merged
}

/// Normalised form used for "never flag this again" fingerprints, so
/// "06-12345678" and "06 1234 5678" count as the same number.
fn normalize(kind: Kind, value: &str) -> String {
    match kind {
        Kind::Phone | Kind::Card | Kind::Iban | Kind::Postcode | Kind::IdNumber | Kind::NumberPlate => {
            value.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_lowercase()
        }
        _ => value.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase(),
    }
}

fn fingerprint(kind: Kind, value: &str) -> String {
    let mut h = Sha256::new();
    h.update(kind.key().as_bytes());
    h.update([0]);
    h.update(normalize(kind, value).as_bytes());
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

// ---------------------------------------------------------------------------------
// Files from the computer scan: the first line is the file's path, the rest its text.

fn word_at(hay: &str, i: usize, len: usize) -> bool {
    let before = hay[..i].chars().next_back();
    let after = hay[i + len..].chars().next();
    !before.is_some_and(|c| c.is_alphanumeric()) && !after.is_some_and(|c| c.is_alphanumeric())
}

/// Flags a file by its name: identity documents and money matters, or key files.
fn detect_file_name(path_line: &str) -> Vec<Detection> {
    let lower = path_line.to_lowercase();
    // Only the file's own name counts, not the folders it sits in.
    let name_start = path_line.rfind('/').map(|i| i + 1).unwrap_or(0);
    let name = path_line[name_start..].to_lowercase();
    if crate::pcscan::is_key_file(&lower) {
        return vec![Detection { kind: Kind::Credential, start: name_start, end: path_line.len() }];
    }
    // Treat _ - . as spaces so "Passport_scan.jpg" and "bank-statement.pdf" match.
    let spaced: String = name.chars().map(|c| if matches!(c, '_' | '-' | '.') { ' ' } else { c }).collect();
    for word in crate::pcscan::SENSITIVE_NAME_WORDS {
        let mut from = 0;
        while let Some(pos) = spaced[from..].find(word) {
            let i = from + pos;
            if word_at(&spaced, i, word.len()) {
                // The whole file name is the finding, so "never flag this again" applies
                // to this file only, not to every file with the same word in its name.
                return vec![Detection { kind: Kind::SensitiveFile, start: name_start, end: path_line.len() }];
            }
            from = i + word.len();
        }
    }
    vec![]
}

/// What to scan in a message: the whole text, or for PC files the name and the body separately.
fn detect_message(scanner: &Scanner, platform: &str, content: &str) -> Vec<Detection> {
    if platform != "pc" {
        return scanner.detect(content);
    }
    let first_end = content.find('\n').unwrap_or(content.len());
    let mut found = detect_file_name(&content[..first_end]);
    let body_start = content.find("\n\n").map(|i| i + 2).unwrap_or(content.len());
    found.extend(scanner.detect(&content[body_start..]).into_iter().map(|d| Detection {
        start: d.start + body_start,
        end: d.end + body_start,
        ..d
    }));
    found
}

/// Raised whenever detection changes in a way that alters findings. Stored data checked with an
/// older version is checked again once, automatically: the user already chose to check it.
pub const DETECTION_VERSION: u32 = 3;

/// Checks stored data again if it was last checked by an older version of the detection. Nothing
/// new is read from the user's files or accounts; their decisions on findings are kept. Returns
/// whether it ran.
pub fn recheck_if_outdated(store: &mut Store) -> Result<bool> {
    let checked_before = store.get_meta("scanned_at_ms")?.is_some();
    let version: u32 = store.get_meta("detection_version")?.and_then(|v| v.parse().ok()).unwrap_or(0);
    if !checked_before || version >= DETECTION_VERSION {
        return Ok(false);
    }
    let opts = ScanOptions { watch_terms: store.watch_terms()? };
    run_scan(store, &opts, &mut |_, _| {})?;
    Ok(true)
}

/// Parts of the patterns that only match non-Latin scripts (or Vietnamese and 〒), so those
/// detectors are skipped on plain-ASCII text.
const SCRIPT_ONLY: &[&str] = &["улица", "οδός", "شارع", "רחוב", "大道", "丁目", "\\p{Hangul}", "ถนน", "〒 ?[0-9]{3}", "Đường"];

/// The extra checks for each country's postcode shape. Dutch, UK, Canadian, US, Australian and
/// Japanese ones say what they are; Irish ones and number-only ones need an address around them.
fn postcode_shape_ok(m: &str, tail: &str, before: &str, after: &str) -> bool {
    // "Postcode: …", "PLZ …", "CEP …", "邮编 …": the word says what it is.
    if let Some(k) = postcode_keyword(m) {
        let code = &m[k.len()..];
        let n = code.chars().filter(|c| c.is_ascii_digit()).count();
        let key = k.to_lowercase();
        return n >= 3
            && (!key.starts_with("pin") || n == 6) // an Indian PIN code, not a card PIN
            && (!key.starts_with("zip") || n >= 5)
            && (!key.starts_with("cap") || n == 5)
            && (!(key.starts_with("c.p") || key.starts_with("cp")) || (4..=5).contains(&n));
    }
    if m.starts_with('〒') {
        return true;
    }
    let has_letter = m.chars().any(|c| c.is_ascii_alphabetic());
    let prefixed = m.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) && m.contains('-') && !m.starts_with(|c: char| c.is_ascii_digit());
    // Ghana's digital address: GA-123-4567
    if m.matches('-').count() == 2 && m.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) {
        return city_after(after) || country_after(after) || before.to_lowercase().contains("ghana") || before.to_lowercase().contains("digital address");
    }
    // US ZIP: the state comes right after "City,"
    if m.len() >= 8 && m.as_bytes()[2] == b' ' && tail.chars().all(|c| c.is_ascii_digit() || c == '-') && tail.len() >= 5 {
        return before.trim_end().ends_with(',');
    }
    // Australia
    if ["NSW ", "VIC ", "QLD ", "TAS ", "ACT "].iter().any(|p| m.starts_with(p)) {
        return true;
    }
    // Malta (VLT 1117) and Argentina (C1425DKF): letters and numbers that could be any code, so
    // only with an address around them.
    let malta = m.len() == 8 && m.as_bytes()[3] == b' ' && m[..3].chars().all(|c| c.is_ascii_uppercase()) && m[4..].chars().all(|c| c.is_ascii_digit());
    let argentina = m.len() == 8 && !m.contains(' ') && m[1..5].chars().all(|c| c.is_ascii_digit());
    if malta || argentina {
        let near = after.chars().take(80).collect::<String>().to_lowercase();
        return street_near(before) || (malta && near.contains("malta")) || (argentina && (city_after(after) || near.contains("argentina")));
    }
    if has_letter && !prefixed {
        if tail.len() == 4 && m.len() == 8 {
            // Irish Eircode: needs a street or the country
            return street_near(before) || country_after(after) || after.to_lowercase().contains("dublin") || before.to_lowercase().contains("dublin");
        }
        return true;
    }

    // Postcodes in numbers only. Cheapest checks first: most numbers in a mailbox have no
    // city or country next to them.
    let city = city_after(after);
    let country = || country_after(after);
    if prefixed {
        return city || country();
    }
    let rest = after.trim_start_matches([' ', '\t']);
    let end_after = matches!(rest.chars().next(), None | Some(',' | '\n' | '\r' | ')' | '.' | ';' | '|' | '<'));
    // City or country before it, as in Asia, India, Russia and South Africa:
    // "Mumbai 400001", "Singapore 238823", "Cape Town, 8001", "Москва, 125009".
    let city_b = end_after && city_before(before);
    let country_b = end_after && country_before(before);
    let prev = before.trim_end_matches([' ', '\t']).chars().next_back();
    let prev_ok = matches!(prev, None | Some(',' | ';' | '\n' | '\r' | '|' | '>' | '(' | ':'));
    // A number after a label ("Reference: 55012", "Score: 1500") is that thing, not a postcode.
    if matches!(prev, Some(':' | '#')) {
        let label = before.trim_end_matches([' ', '\t', ':', '#']);
        let word: String = label.chars().rev().take_while(|c| c.is_alphabetic()).collect::<Vec<_>>().into_iter().rev().collect();
        if NOT_A_PLACE.contains(&word.to_lowercase().as_str()) {
            return false;
        }
    }
    if !city && !prev_ok && !city_b && !country_b {
        return false;
    }
    let year = m.len() == 4 && (m.starts_with("19") || m.starts_with("20"));
    // The house number of a street that follows ("1600 Amphitheatre Parkway, …") is not a postcode.
    let segment: String = rest.split([',', '\n']).next().unwrap_or("").chars().take(40).collect();
    if street_word(&segment) {
        return false;
    }
    // A street on this line, before or after it ("125009, Москва, ул. Тверская, 7")
    let line_after: String = rest.lines().next().unwrap_or("").chars().take(80).collect();
    let street = street_near(before) || street_word(&line_after) && line_after.chars().any(|c| c.is_ascii_digit());
    if year && !street {
        return false;
    }
    if country_b {
        return true;
    }
    if street {
        return city || city_b || country();
    }
    // No street: a city and the country have to be there.
    (city && prev_ok || city_b) && country()
}

/// "Postcode:", "PLZ", "CEP", "邮编" and the like at the start of a match: returns that part.
fn postcode_keyword(m: &str) -> Option<&str> {
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| Regex::new(&format!("^(?:{POSTCODE_WORDS})")).expect("valid pattern")).find(m).map(|k| &m[..k.end()])
}

/// The words people put before a postcode, in many languages, with what follows them up to the code.
const POSTCODE_WORDS: &str = r"(?i:\b(?:post\s?code|postal\s?code|zip\s?code|zip|plz|postleitzahl|cap|c\.\s?p\.|cp|c[óo]digo postal|code postal|cep|pin\s?code|pincode|kod pocztowy|postnummer|postinumero|irányítószám|ir\.?\s?sz\.?|psč|po[šs]tovní směrovací číslo|почтовый индекс|поштовий індекс|индекс|індекс|τ\.\s?κ\.)\s*[:.#]?\s*|(?:邮编|邮政编码|郵便番号|우편번호|รหัสไปรษณีย์|الرمز البريدي|מיקוד)\s*[:：]?\s*〒?)";

fn street_word(s: &str) -> bool {
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:street|st|avenue|ave|road|rd|lane|ln|drive|dr|way|court|ct|place|pl|boulevard|blvd|highway|hwy|parkway|pkwy|square|sq|terrace|crescent|close|plot|house no|flat|apt|apartment|p\.?\s?o\.?\s?box|private bag|via|viale|piazza|piazzale|corso|largo|vicolo|calle|carrera|cra|avenida|avda|plaza|paseo|carrer|carretera|camino|colonia|col|rua|travessa|pra[çc]a|rue|av|chemin|quai|cours|impasse|all[ée]e|route|ulica|ul|aleja|al|plac|osiedle|strada|str|bulevardul|bd|calea|aleea|[șş]oseaua|splaiul|ulice|n[áa]m[ěe]st[íi]|utca|[úu]t|t[ée]r|k[öo]r[úu]t|odos|caddesi|cad|sokak|sk|mahallesi|mah|marg|nagar|jalan|jl|jln|soi|thanon|đường|phố|duong|barabara|mtaa|улица|ул|проспект|пр-т|переулок|пер|шоссе|бульвар|набережная|наб|вулиця|вул|οδός|οδ|λεωφόρος|λεωφ|شارع|خیابان|کوچه|רחוב|רח'|ถนน|ซอย|postfach|apartado|caixa postal|casilla|postbus)\b|\p{L}(?:straat|laan|weg|gracht|plein|kade|dijk|singel|steeg|strasse|straße|gasse|allee|platz|damm|ufer|gatan|gata|gade|vej|vägen|veien|katu|tie)\b|\p{Han}(?:路|街|大道|巷|弄|号|丁目|番地)|\p{Hangul}(?:로|길|대로)\s?[0-9]")
            .expect("valid pattern")
    })
    .is_match(s)
}

/// A street name with a house number on this line or the line above.
fn street_near(before: &str) -> bool {
    let lines: Vec<&str> = before.rsplit('\n').take(2).collect();
    let mut win = lines.into_iter().rev().collect::<Vec<_>>().join("\n");
    let mut cut = win.len().saturating_sub(200);
    while !win.is_char_boundary(cut) {
        cut += 1;
    }
    win = win[cut..].to_string();
    street_word(&win) && win.chars().any(|c| c.is_ascii_digit() || ('０'..='９').contains(&c) || ('٠'..='٩').contains(&c) || ('۰'..='۹').contains(&c))
}

/// A word that names a place: capitalised in scripts that have capitals, any word in those that don't.
fn place_word(word: &str) -> bool {
    let mut cs = word.chars();
    let Some(first) = cs.next() else { return false };
    let n = word.chars().count();
    if first.is_uppercase() {
        // "Catania", "São", or all capitals ("CATANIA")
        n >= 3 || word.chars().all(|c| c.is_uppercase())
    } else {
        // Scripts without capitals: Chinese, Japanese, Korean, Arabic, Hebrew, Thai, Hindi…
        first.is_alphabetic() && !first.is_lowercase() && n >= 2
    }
}

/// Words that come before a number but are not a city: "Order 12345", "Invoice 2500,".
const NOT_A_PLACE: &[&str] = &[
    "order", "invoice", "ticket", "room", "page", "pages", "flight", "booking", "reservation", "ref", "reference", "case", "item",
    "number", "no", "nr", "account", "customer", "member", "id", "code", "version", "model", "table", "seat", "gate", "bus",
    "line", "route", "tracking", "receipt", "transaction", "payment", "total", "amount", "price", "year", "step", "chapter",
    "bestelling", "factuur", "rechnung", "bestellung", "commande", "facture", "pedido", "factura", "ordine", "fattura",
    "zamówienie", "faktura", "comanda", "factura", "kundennummer", "klantnummer", "auftrag", "pin", "otp",
    "score", "points", "tickets", "members", "songs", "steps", "runs", "people", "users", "items", "units", "pieces",
    "copies", "words", "euros", "dollars", "miles", "votes", "views", "likes", "followers", "copies", "pages",
    "subscribers", "students", "employees", "visitors", "participants", "guests", "seats", "hours", "minutes",
    "days", "weeks", "months", "years", "times", "calories", "level", "levels", "episode", "season", "lot", "unit",
];

/// A city right before the postcode: "Mumbai 400001", "Cape Town, 8001", "Москва, 125009".
fn city_before(before: &str) -> bool {
    let line = before.rsplit('\n').next().unwrap_or("");
    let t = line.trim_end_matches([' ', '\t', '\u{a0}']);
    let t = t.strip_suffix([',', '-', '–']).unwrap_or(t).trim_end();
    let word: String = t.chars().rev().take_while(|c| c.is_alphabetic() || *c == '-' || *c == '\'' || *c == '.').collect::<Vec<_>>().into_iter().rev().collect();
    let word = word.trim_start_matches(['-', '\'', '.']);
    place_word(word) && !NOT_A_PLACE.contains(&word.to_lowercase().as_str())
}

/// A city name right after the postcode: "95124 Catania", "75001 Paris", "00-950 Warszawa",
/// "125009, Москва".
fn city_after(after: &str) -> bool {
    let rest = after.trim_start_matches([' ', '\t', '\u{a0}']);
    let rest = rest.strip_prefix(',').unwrap_or(rest).trim_start_matches([' ', '\t', '\u{a0}']);
    let word: String = rest.chars().take_while(|c| c.is_alphabetic() || *c == '-' || *c == '\'').collect();
    if !place_word(&word) {
        return false;
    }
    if word.chars().all(|c| c.is_uppercase() || c == '-') && word.chars().next().is_some_and(|c| c.is_uppercase()) {
        // An all-capitals city ("CATANIA") only at the end of the line or before a comma
        let next = rest[word.len()..].trim_start_matches([' ', '\t']).chars().next();
        return matches!(next, None | Some(',' | '\n' | '\r'));
    }
    if NOT_A_PLACE.contains(&word.to_lowercase().as_str()) {
        return false;
    }
    // A city isn't followed by an everyday word ("1500 Tickets left"), only by another part of its
    // name, a comma or the end of the line.
    let next: String = rest[word.len()..].trim_start_matches([' ', '\t']).chars().take_while(|c| c.is_alphabetic()).collect();
    let small = ["de", "del", "do", "da", "dos", "la", "le", "les", "di", "am", "an", "im", "sur", "upon", "on", "en", "aan", "y", "e", "i"];
    !next.chars().next().is_some_and(|c| c.is_lowercase()) || small.contains(&next.as_str())
}

const COUNTRIES: &str = r"(?i:afghanistan|albania|shqipëri|algeria|algérie|الجزائر|andorra|angola|argentina|armenia|australia|austria|österreich|osterreich|azerbaijan|bahrain|bangladesh|বাংলাদেশ|barbados|belarus|беларусь|belgium|belgië|belgie|belgique|belize|benin|bénin|bhutan|bolivia|bosnia and herzegovina|bosnia|botswana|brazil|brasil|brunei|bulgaria|българия|burkina faso|burundi|cambodia|cameroon|cameroun|canada|cape verde|cabo verde|chad|tchad|chile|china|中国|中國|colombia|congo|drc|costa rica|côte d'ivoire|cote d'ivoire|ivory coast|croatia|hrvatska|cuba|cyprus|czech republic|czechia|česko|denmark|danmark|djibouti|dominican republic|república dominicana|ecuador|egypt|مصر|el salvador|estonia|eesti|eswatini|ethiopia|fiji|finland|suomi|france|gabon|gambia|georgia|germany|deutschland|ghana|greece|ελλάδα|guatemala|guinea|haiti|haïti|honduras|hong kong|香港|hungary|magyarország|iceland|ísland|india|भारत|indonesia|iran|ایران|iraq|العراق|ireland|éire|israel|ישראל|italy|italia|jamaica|japan|日本|jordan|الأردن|kazakhstan|қазақстан|kenya|korea|south korea|대한민국|한국|kosovo|kuwait|الكويت|kyrgyzstan|laos|latvia|latvija|lebanon|لبنان|lesotho|liberia|libya|liechtenstein|lithuania|lietuva|luxembourg|macau|madagascar|malawi|malaysia|maldives|mali|malta|mauritania|mauritius|maurice|mexico|méxico|moldova|monaco|mongolia|montenegro|morocco|maroc|المغرب|mozambique|moçambique|myanmar|namibia|nepal|नेपाल|netherlands|nederland|new zealand|aotearoa|nicaragua|niger|nigeria|north macedonia|macedonia|norway|norge|oman|عمان|pakistan|پاکستان|palestine|فلسطين|panama|panamá|papua new guinea|paraguay|peru|perú|philippines|pilipinas|poland|polska|portugal|puerto rico|qatar|قطر|romania|românia|russia|россия|rwanda|saudi arabia|السعودية|senegal|sénégal|serbia|srbija|србија|seychelles|sierra leone|singapore|新加坡|slovakia|slovensko|slovenia|slovenija|somalia|south africa|south sudan|spain|españa|espana|sri lanka|sudan|السودان|suriname|sweden|sverige|switzerland|schweiz|suisse|svizzera|syria|سوريا|taiwan|台灣|台湾|tajikistan|tanzania|thailand|ประเทศไทย|togo|trinidad and tobago|tunisia|tunisie|تونس|turkey|türkiye|turkiye|turkmenistan|uganda|ukraine|україна|united arab emirates|uae|الإمارات|united kingdom|uk|great britain|england|scotland|wales|northern ireland|united states|united states of america|usa|u\.s\.a\.|us|uruguay|uzbekistan|o'zbekiston|venezuela|vietnam|viet nam|việt nam|yemen|اليمن|zambia|zimbabwe)";

/// A country name on the rest of this line or the next one.
fn country_after(after: &str) -> bool {
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(&format!(r"(?:^|[^\p{{L}}]){COUNTRIES}(?:[^\p{{L}}]|$)")).expect("valid pattern"));
    let mut win: String = after.lines().take(2).collect::<Vec<_>>().join(" ");
    let mut cut = 80.min(win.len());
    while !win.is_char_boundary(cut) {
        cut += 1;
    }
    win.truncate(cut);
    // "US" in capitals only, so the word "us" doesn't count
    re.find_iter(&win).any(|m| !m.as_str().trim_matches(|c: char| !c.is_alphabetic()).eq("us"))
}

/// A country name right before the postcode: "Singapore 238823", "Singapore, 238823".
fn country_before(before: &str) -> bool {
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(&format!(r"(?:^|[^\p{{L}}]){COUNTRIES}[\s,\-–]*$")).expect("valid pattern"));
    let line = before.rsplit('\n').next().unwrap_or("");
    let mut cut = line.len().saturating_sub(60);
    while !line.is_char_boundary(cut) {
        cut += 1;
    }
    re.find(&line[cut..]).is_some_and(|m| !m.as_str().trim_matches(|c: char| !c.is_alphabetic()).eq("us"))
}

/// A line that is just a company or website name, as at the top of a signature ("Booking.com",
/// "Wizz Air Hungary Ltd.", "Example B.V."). Not a line with a sentence on it.
fn is_brand_line(line: &str) -> bool {
    let l = line.trim();
    let lower = l.to_lowercase();
    let words = l.split_whitespace().count();
    let domain = words == 1
        && [".com", ".nl", ".net", ".org", ".eu", ".de", ".co.uk", ".io"].iter().any(|d| lower.ends_with(d))
        && !lower.contains('@');
    let company = words <= 5 && [" b.v.", " bv", " n.v.", " ltd", " ltd.", " gmbh", " inc.", " s.a.", " s.r.l.", " plc"].iter().any(|w| lower.ends_with(w));
    domain || company
}

/// Whether the text just before ends with the day and month of a date ("07/06/", "28-02-",
/// "28.02."), ignoring whatever separates them from what follows: spaces, line breaks,
/// invisible characters or table separators.
fn date_part_before(before: &str) -> bool {
    static DATE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = DATE.get_or_init(|| Regex::new(r"(?:^|[^0-9])[0-9]{1,2}[/.\-][0-9]{1,2}[/.\-]$").expect("valid pattern"));
    let mut cut = before.len().saturating_sub(24);
    while !before.is_char_boundary(cut) {
        cut += 1;
    }
    // Keep only digits and date separators, with a run of separators counted once ("07/06/ </td>").
    let mut kept = String::new();
    for c in before[cut..].chars().filter(|c| c.is_ascii_digit() || matches!(c, '/' | '.' | '-')) {
        let repeat = !c.is_ascii_digit() && kept.ends_with(|p: char| !p.is_ascii_digit());
        if !repeat {
            kept.push(c);
        }
    }
    re.is_match(&kept)
}

/// Whether a finding at `start` is in text someone else wrote or that's about someone else: in a sent
/// email, the recipients on the "To:" line and the quoted or forwarded email under your own words.
pub(crate) fn is_theirs(platform: &str, content: &str, start: usize) -> bool {
    if platform != "email" {
        return false;
    }
    if content.starts_with("To: ") && start < content.find('\n').unwrap_or(content.len()) {
        return true;
    }
    content.find(crate::import::QUOTED_MARK).is_some_and(|q| start >= q)
}

// ---------------------------------------------------------------------------------
// Running a scan and reading results

#[derive(Debug, Clone, Serialize)]
pub struct KindCount {
    pub kind: Kind,
    pub label: &'static str,
    pub severity: i64,
    pub advice: &'static str,
    pub count: i64,
    /// findings still to review (not decided, or marked for later)
    pub open: i64,
    /// the same, leaving out findings in someone else's words: what the score counts
    pub mine_open: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanSummary {
    pub scanned: i64,
    pub flagged_messages: i64,
    pub total_findings: i64,
    /// findings with no decision yet
    pub open: i64,
    pub later: i64,
    /// marked deleted or not a problem
    pub done: i64,
    pub ignored_rules: i64,
    /// findings in someone else's words (a quoted email, a recipient's address)
    pub theirs: i64,
    pub scanned_at_ms: Option<i64>,
    pub by_kind: Vec<KindCount>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FindingView {
    pub id: i64,
    pub message_id: String,
    pub channel_id: String,
    pub channel_name: String,
    pub platform: String,
    pub ts: i64,
    pub kind: Kind,
    pub label: &'static str,
    pub severity: i64,
    pub advice: &'static str,
    pub before: String,
    pub matched: String,
    pub after: String,
    pub masked: bool,
    /// deleted | ok | later, or None when not decided yet
    pub status: Option<String>,
    /// Where to open it on its platform, when it has a web address
    pub link: Option<String>,
    /// For files found on this computer: the full path
    pub file_path: Option<String>,
    /// How many findings in the same list have this exact value (1 = only here)
    pub repeats: i64,
    /// Found in someone else's words: an email you quoted or forwarded, or a recipient's address
    pub theirs: bool,
}

/// Every place one value was found, for acting on all of them at once.
#[derive(Debug, Clone, Serialize)]
pub struct GroupInfo {
    pub ids: Vec<i64>,
    /// The message each finding is in, so a message is only deleted once
    pub message_ids: Vec<String>,
    /// The platform of each finding, in the same order as `ids`
    pub member_platforms: Vec<String>,
    pub platforms: Vec<GroupPlatform>,
    pub first_ts: i64,
    pub last_ts: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct GroupPlatform {
    pub platform: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FindingsPage {
    pub total: i64,
    /// How many of `total` are someone else's (they come last in the list).
    pub theirs: i64,
    pub offset: i64,
    pub items: Vec<FindingView>,
}

/// Scans every imported message. Replaces the results of any previous scan;
/// decisions the user already made are kept.
pub fn run_scan(
    store: &mut Store,
    opts: &ScanOptions,
    progress: &mut dyn FnMut(i64, i64),
) -> Result<ScanSummary> {
    let scanner = Scanner::new(opts)?;
    store.set_meta("watch_terms", &serde_json::to_string(&opts.watch_terms)?)?;
    let total: i64 = store.conn.query_row("SELECT COUNT(*) FROM messages", [], |r| r.get(0))?;
    let ignored: std::collections::HashSet<(String, String)> = {
        let mut stmt = store.conn.prepare("SELECT kind, fingerprint FROM ignored")?;
        let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
        rows.collect::<rusqlite::Result<_>>()?
    };

    let tx = store.conn.transaction()?;
    tx.execute("DELETE FROM findings", [])?;
    {
        let mut read = tx.prepare(
            "SELECT m.id, m.content, c.platform FROM messages m JOIN channels c ON c.id = m.channel_id WHERE m.content <> ''",
        )?;
        let mut write = tx.prepare(
            "INSERT INTO findings(message_id, kind, severity, start_byte, end_byte, fp, theirs) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )?;
        let mut rows = read.query([])?;
        let mut done = 0i64;
        while let Some(row) = rows.next()? {
            let id: i64 = row.get(0)?;
            let content: String = row.get(1)?;
            let platform: String = row.get(2)?;
            for d in detect_message(&scanner, &platform, &content) {
                let fp = fingerprint(d.kind, &content[d.start..d.end]);
                if ignored.contains(&(d.kind.key().to_string(), fp.clone())) {
                    continue;
                }
                let theirs = is_theirs(&platform, &content, d.start);
                write.execute(params![id, d.kind.key(), d.kind.severity(), d.start as i64, d.end as i64, fp, theirs])?;
            }
            done += 1;
            if done % 5000 == 0 {
                progress(done, total);
            }
        }
        progress(total, total);
    }
    tx.commit()?;
    store.set_meta("scanned_at_ms", &crate::now_ms().to_string())?;
    store.set_meta("detection_version", &DETECTION_VERSION.to_string())?;
    store.scan_summary()
}

/// Builds a short preview around the match. If `reveal` is false, every sensitive
/// span in the preview is masked, not just this one, so nothing leaks on screen.
fn snippet(content: &str, start: usize, end: usize, spans: &[(usize, usize)], reveal: bool) -> (String, String, String, bool) {
    const CONTEXT: usize = 70;
    let render = |from: usize, to: usize| -> String {
        if reveal {
            return content[from..to].to_string();
        }
        let mut out = String::new();
        let mut prev_alnum = false;
        for (i, ch) in content[from..to].char_indices() {
            let pos = from + i;
            let sensitive = spans.iter().any(|&(s, e)| pos >= s && pos < e);
            out.push(if sensitive && ch.is_alphanumeric() && prev_alnum { '•' } else { ch });
            prev_alnum = ch.is_alphanumeric();
        }
        out
    };
    let mut from = start;
    for (n, (i, _)) in content[..start].char_indices().rev().enumerate() {
        if n >= CONTEXT {
            break;
        }
        from = i;
    }
    let mut to = end;
    for (n, (i, ch)) in content[end..].char_indices().enumerate() {
        if n >= CONTEXT {
            break;
        }
        to = end + i + ch.len_utf8();
    }
    let mut before = render(from, start);
    let mut after = render(end, to);
    if from > 0 {
        before.insert(0, '…');
    }
    if to < content.len() {
        after.push('…');
    }
    (before.replace('\n', " "), render(start, end), after.replace('\n', " "), !reveal)
}

/// What to list on the Review screen.
#[derive(Debug, Clone)]
pub struct FindingsFilter<'a> {
    pub kind: Option<&'a str>,
    pub platform: Option<&'a str>,
    pub status: StatusFilter,
    /// One row per distinct value (the newest place it appears)
    pub group: bool,
    /// Only findings with the same value as this one
    pub same_as: Option<i64>,
    pub only_id: Option<i64>,
}

impl Default for FindingsFilter<'_> {
    fn default() -> Self {
        Self { kind: None, platform: None, status: StatusFilter::Open, group: false, same_as: None, only_id: None }
    }
}

/// Which findings to list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusFilter {
    /// nothing decided yet
    Open,
    Later,
    /// deleted or not a problem
    Done,
    All,
}

impl StatusFilter {
    pub fn parse(s: &str) -> StatusFilter {
        match s {
            "later" => StatusFilter::Later,
            "done" => StatusFilter::Done,
            "all" => StatusFilter::All,
            _ => StatusFilter::Open,
        }
    }
    fn sql(self) -> &'static str {
        match self {
            StatusFilter::Open => "r.status IS NULL",
            StatusFilter::Later => "r.status = 'later'",
            StatusFilter::Done => "r.status IN ('deleted', 'ok')",
            StatusFilter::All => "1",
        }
    }
}

const FINDING_FROM: &str = "FROM findings f
     JOIN messages m ON m.id = f.message_id
     JOIN channels ch ON ch.id = m.channel_id
     LEFT JOIN reviews r ON r.uid = m.uid AND r.kind = f.kind AND r.start_byte = f.start_byte";

impl Store {
    pub fn watch_terms(&self) -> Result<Vec<String>> {
        Ok(self
            .get_meta("watch_terms")?
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default())
    }

    pub fn scan_summary(&self) -> Result<ScanSummary> {
        let c = &self.conn;
        let one = |sql: &str| -> Result<i64> { Ok(c.query_row(sql, [], |r| r.get(0))?) };
        let mut stmt = c.prepare(&format!(
            "SELECT f.kind, COUNT(*), SUM(CASE WHEN r.status IS NULL OR r.status = 'later' THEN 1 ELSE 0 END),
                    SUM(CASE WHEN (r.status IS NULL OR r.status = 'later') AND f.theirs = 0 THEN 1 ELSE 0 END)
             {FINDING_FROM} GROUP BY f.kind"
        ))?;
        let mut counts: HashMap<String, (i64, i64, i64)> = HashMap::new();
        for row in stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))? {
            let (k, n, open, mine_open) = row?;
            counts.insert(k, (n, open, mine_open));
        }
        let by_kind = Kind::ALL
            .into_iter()
            .map(|k| {
                let (count, open, mine_open) = counts.get(k.key()).copied().unwrap_or((0, 0, 0));
                KindCount { kind: k, label: k.label(), severity: k.severity(), advice: k.advice(), count, open, mine_open }
            })
            .collect();
        let status_count = |filter: StatusFilter| one(&format!("SELECT COUNT(*) {FINDING_FROM} WHERE {}", filter.sql()));
        Ok(ScanSummary {
            scanned: one("SELECT COUNT(*) FROM messages")?,
            flagged_messages: one("SELECT COUNT(DISTINCT message_id) FROM findings")?,
            total_findings: one("SELECT COUNT(*) FROM findings")?,
            open: status_count(StatusFilter::Open)?,
            later: status_count(StatusFilter::Later)?,
            done: status_count(StatusFilter::Done)?,
            ignored_rules: one("SELECT COUNT(*) FROM ignored")?,
            theirs: one("SELECT COUNT(*) FROM findings WHERE theirs = 1")?,
            scanned_at_ms: self.get_meta("scanned_at_ms")?.and_then(|s| s.parse().ok()),
            by_kind,
        })
    }

    /// Findings, most serious and newest first. `kind` filters by Kind::key().
    pub fn findings(&self, kind: Option<&str>, status: StatusFilter, offset: i64, limit: i64, reveal: bool) -> Result<FindingsPage> {
        self.findings_filtered(&FindingsFilter { kind, status, ..Default::default() }, offset, limit, reveal)
    }

    pub fn findings_for_platform(
        &self,
        kind: Option<&str>,
        platform: Option<&str>,
        status: StatusFilter,
        offset: i64,
        limit: i64,
        reveal: bool,
    ) -> Result<FindingsPage> {
        self.findings_filtered(&FindingsFilter { kind, platform, status, ..Default::default() }, offset, limit, reveal)
    }

    /// One finding with its text shown in full (for revealing a single item).
    pub fn reveal_finding(&self, id: i64) -> Result<Option<FindingView>> {
        let f = FindingsFilter { status: StatusFilter::All, only_id: Some(id), ..Default::default() };
        Ok(self.findings_filtered(&f, 0, 1, true)?.items.pop())
    }

    pub fn findings_filtered(&self, q: &FindingsFilter, offset: i64, limit: i64, reveal: bool) -> Result<FindingsPage> {
        let c = &self.conn;
        // `base` numbers each value's findings, newest first, and counts them, so grouping
        // can keep one row per value and every row knows how often its value appears.
        let base = format!(
            "WITH base AS (
               SELECT f.id, f.message_id, m.channel_id, ch.name AS cname, ch.platform, m.ts, f.kind, f.severity,
                      f.start_byte, f.end_byte, m.content, r.status, m.link, m.attachments, f.theirs,
                      ROW_NUMBER() OVER (PARTITION BY f.kind, f.fp ORDER BY m.ts DESC, f.id) AS rn,
                      COUNT(*) OVER (PARTITION BY f.kind, f.fp) AS repeats
               {FINDING_FROM}
               WHERE (?1 IS NULL OR f.kind = ?1) AND (?5 IS NULL OR ch.platform = ?5) AND {}
                 AND (?7 IS NULL OR (f.kind, f.fp) = (SELECT kind, fp FROM findings WHERE id = ?7))
             )",
            q.status.sql()
        );
        let keep = "(?2 IS NULL OR id = ?2) AND (?6 = 0 OR rn = 1)";
        let group = q.group as i64;
        let (lim, off) = (limit.clamp(1, 1000), offset.max(0));
        let (total, theirs): (i64, i64) = c.query_row(
            &format!("{base} SELECT COUNT(*), COALESCE(SUM(theirs), 0) FROM base WHERE {keep} AND ?3 = ?3 AND ?4 = ?4"),
            params![q.kind, q.only_id, 0, 0, q.platform, group, q.same_as],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let mut stmt = c.prepare_cached(&format!(
            "{base} SELECT id, message_id, channel_id, cname, platform, ts, kind, start_byte, end_byte, content, status, link,
                           attachments, repeats, theirs
             FROM base WHERE {keep}
             ORDER BY theirs, severity DESC, repeats * (?6) DESC, ts DESC, start_byte
             LIMIT ?3 OFFSET ?4"
        ))?;
        type Row = (i64, i64, i64, String, String, i64, String, usize, usize, String, Option<String>, Option<String>, String, i64, bool);
        let rows: Vec<Row> = stmt
            .query_map(params![q.kind, q.only_id, lim, off, q.platform, group, q.same_as], |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                    r.get::<_, i64>(7)? as usize,
                    r.get::<_, i64>(8)? as usize,
                    r.get(9)?,
                    r.get(10)?,
                    r.get(11)?,
                    r.get(12)?,
                    r.get(13)?,
                    r.get(14)?,
                ))
            })?
            .collect::<rusqlite::Result<_>>()?;

        let mut items = Vec::with_capacity(rows.len());
        for (id, mid, cid, cname, platform, ts, k, start, end, content, status, link, attachments, repeats, theirs) in rows {
            let Some(kind) = Kind::from_key(&k) else { continue };
            if end > content.len() || !content.is_char_boundary(start) || !content.is_char_boundary(end) {
                continue;
            }
            let spans = self.spans_for(mid)?;
            let (before, matched, after, masked) = snippet(&content, start, end, &spans, reveal);
            let file_path = (platform == "pc").then_some(attachments);
            items.push(FindingView {
                id,
                message_id: mid.to_string(),
                channel_id: cid.to_string(),
                channel_name: cname,
                platform,
                ts,
                kind,
                label: kind.label(),
                severity: kind.severity(),
                advice: kind.advice(),
                before,
                matched,
                after,
                masked,
                status,
                link,
                file_path,
                repeats,
                theirs,
            });
        }
        Ok(FindingsPage { total, theirs, offset, items })
    }

    /// Every finding with the same value as `finding_id`, within one status list.
    pub fn group_info(&self, finding_id: i64, status: StatusFilter) -> Result<GroupInfo> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT f.id, f.message_id, ch.platform, m.ts {FINDING_FROM}
             WHERE (f.kind, f.fp) = (SELECT kind, fp FROM findings WHERE id = ?1) AND {}
             ORDER BY m.ts",
            status.sql()
        ))?;
        let rows: Vec<(i64, i64, String, i64)> =
            stmt.query_map([finding_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?.collect::<rusqlite::Result<_>>()?;
        let mut platforms: Vec<GroupPlatform> = Vec::new();
        for (_, _, p, _) in &rows {
            match platforms.iter_mut().find(|x| &x.platform == p) {
                Some(x) => x.count += 1,
                None => platforms.push(GroupPlatform { platform: p.clone(), count: 1 }),
            }
        }
        platforms.sort_by(|a, b| b.count.cmp(&a.count));
        Ok(GroupInfo {
            first_ts: rows.first().map_or(0, |r| r.3),
            last_ts: rows.last().map_or(0, |r| r.3),
            ids: rows.iter().map(|r| r.0).collect(),
            message_ids: rows.iter().map(|r| r.1.to_string()).collect(),
            member_platforms: rows.iter().map(|r| r.2.clone()).collect(),
            platforms,
        })
    }

    /// Records one decision for many findings at once. Returns what each had before, for undo.
    pub fn set_status_many(&mut self, ids: &[i64], status: Option<&str>) -> Result<Vec<(i64, Option<String>)>> {
        if let Some(s) = status {
            anyhow::ensure!(matches!(s, "deleted" | "ok" | "later"), "unknown status {s}");
        }
        let tx = self.conn.transaction()?;
        let mut before = Vec::with_capacity(ids.len());
        {
            let mut get = tx.prepare_cached(
                "SELECT m.uid, f.kind, f.start_byte, r.status FROM findings f JOIN messages m ON m.id = f.message_id
                 LEFT JOIN reviews r ON r.uid = m.uid AND r.kind = f.kind AND r.start_byte = f.start_byte WHERE f.id = ?1",
            )?;
            let mut set = tx.prepare_cached(
                "INSERT INTO reviews(uid, kind, start_byte, status) VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(uid, kind, start_byte) DO UPDATE SET status = excluded.status",
            )?;
            let mut clear = tx.prepare_cached("DELETE FROM reviews WHERE uid = ?1 AND kind = ?2 AND start_byte = ?3")?;
            for &id in ids {
                let row: Option<(String, String, i64, Option<String>)> = get
                    .query_row([id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
                    .map(Some)
                    .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })?;
                let Some((uid, kind, start, prev)) = row else { continue };
                match status {
                    Some(s) => set.execute(params![uid, kind, start, s])?,
                    None => clear.execute(params![uid, kind, start])?,
                };
                before.push((id, prev));
            }
        }
        tx.commit()?;
        Ok(before)
    }

    /// Puts decisions back the way `set_status_many` found them.
    pub fn restore_statuses(&mut self, previous: &[(i64, Option<String>)]) -> Result<()> {
        for status in [None, Some("deleted"), Some("ok"), Some("later")] {
            let ids: Vec<i64> = previous.iter().filter(|(_, p)| p.as_deref() == status).map(|(id, _)| *id).collect();
            if !ids.is_empty() {
                self.set_status_many(&ids, status)?;
            }
        }
        Ok(())
    }

    /// Fills in value fingerprints for findings stored before they existed.
    pub(crate) fn backfill_fingerprints(&mut self) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut read = tx.prepare("SELECT f.id, f.kind, f.start_byte, f.end_byte, m.content FROM findings f JOIN messages m ON m.id = f.message_id")?;
            let mut write = tx.prepare("UPDATE findings SET fp = ?2 WHERE id = ?1")?;
            let mut rows = read.query([])?;
            while let Some(r) = rows.next()? {
                let (id, k, s, e, content): (i64, String, i64, i64, String) = (r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?);
                if let (Some(kind), Some(v)) = (Kind::from_key(&k), content.get(s as usize..e as usize)) {
                    write.execute(params![id, fingerprint(kind, v)])?;
                }
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub(crate) fn spans_for(&self, message_id: i64) -> Result<Vec<(usize, usize)>> {
        let mut stmt = self.conn.prepare_cached("SELECT start_byte, end_byte FROM findings WHERE message_id = ?1")?;
        let spans = stmt
            .query_map([message_id], |r| Ok((r.get::<_, i64>(0)? as usize, r.get::<_, i64>(1)? as usize)))?
            .collect::<rusqlite::Result<_>>()?;
        Ok(spans)
    }

    /// Records a decision for one finding: "deleted", "ok", "later", or None to undo.
    pub fn set_status(&self, finding_id: i64, status: Option<&str>) -> Result<()> {
        if let Some(s) = status {
            anyhow::ensure!(matches!(s, "deleted" | "ok" | "later"), "unknown status {s}");
        }
        let (uid, kind, start): (String, String, i64) = self.conn.query_row(
            "SELECT m.uid, f.kind, f.start_byte FROM findings f JOIN messages m ON m.id = f.message_id WHERE f.id = ?1",
            [finding_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        match status {
            Some(s) => self.conn.execute(
                "INSERT INTO reviews(uid, kind, start_byte, status) VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(uid, kind, start_byte) DO UPDATE SET status = excluded.status",
                params![uid, kind, start, s],
            )?,
            None => self.conn.execute(
                "DELETE FROM reviews WHERE uid = ?1 AND kind = ?2 AND start_byte = ?3",
                params![uid, kind, start],
            )?,
        };
        Ok(())
    }

    /// Moves a file found by the computer scan to the Recycle Bin, and marks everything
    /// found in it as deleted. Returns the path that was moved.
    pub fn recycle_file(&self, finding_id: i64) -> Result<String> {
        let (message_id, platform, path): (i64, String, String) = self.conn.query_row(
            "SELECT m.id, c.platform, m.attachments FROM findings f
             JOIN messages m ON m.id = f.message_id JOIN channels c ON c.id = m.channel_id WHERE f.id = ?1",
            [finding_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        anyhow::ensure!(platform == "pc", "only files on this computer can be moved to the Recycle Bin");
        crate::pcscan::move_to_recycle_bin(&path)?;
        self.conn.execute(
            "INSERT INTO reviews(uid, kind, start_byte, status)
             SELECT m.uid, f.kind, f.start_byte, 'deleted' FROM findings f JOIN messages m ON m.id = f.message_id
             WHERE f.message_id = ?1
             ON CONFLICT(uid, kind, start_byte) DO UPDATE SET status = 'deleted'",
            [message_id],
        )?;
        Ok(path)
    }

    /// What a finding points at, for deleting it on its platform: (platform, message uid).
    pub fn delete_target(&self, finding_id: i64) -> Result<(String, String)> {
        Ok(self.conn.query_row(
            "SELECT c.platform, m.uid FROM findings f
             JOIN messages m ON m.id = f.message_id JOIN channels c ON c.id = m.channel_id WHERE f.id = ?1",
            [finding_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?)
    }

    /// After a message was deleted on its platform: marks everything found in it as deleted.
    pub fn mark_deleted(&self, finding_id: i64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO reviews(uid, kind, start_byte, status)
             SELECT m.uid, f.kind, f.start_byte, 'deleted' FROM findings f JOIN messages m ON m.id = f.message_id
             WHERE f.message_id = (SELECT message_id FROM findings WHERE id = ?1)
             ON CONFLICT(uid, kind, start_byte) DO UPDATE SET status = 'deleted'",
            [finding_id],
        )?;
        Ok(())
    }

    /// The full path of a computer-scan file a finding is in, for "Show in folder".
    pub fn file_path_for(&self, finding_id: i64) -> Result<Option<String>> {
        Ok(self.conn.query_row(
            "SELECT CASE WHEN c.platform = 'pc' THEN m.attachments END FROM findings f
             JOIN messages m ON m.id = f.message_id JOIN channels c ON c.id = m.channel_id WHERE f.id = ?1",
            [finding_id],
            |r| r.get(0),
        )?)
    }

    /// "Never flag this again": stops flagging this exact value (say, a public work email)
    /// now and in future scans. Returns how many findings were cleared.
    pub fn ignore_everywhere(&mut self, finding_id: i64) -> Result<usize> {
        let (kind_key, start, end, content): (String, usize, usize, String) = self.conn.query_row(
            "SELECT f.kind, f.start_byte, f.end_byte, m.content FROM findings f JOIN messages m ON m.id = f.message_id WHERE f.id = ?1",
            [finding_id],
            |r| Ok((r.get(0)?, r.get::<_, i64>(1)? as usize, r.get::<_, i64>(2)? as usize, r.get(3)?)),
        )?;
        let kind = Kind::from_key(&kind_key).ok_or_else(|| anyhow::anyhow!("unknown kind"))?;
        let value = content.get(start..end).ok_or_else(|| anyhow::anyhow!("finding no longer matches its message"))?;
        let fp = fingerprint(kind, value);

        let tx = self.conn.transaction()?;
        tx.execute("INSERT OR IGNORE INTO ignored(kind, fingerprint) VALUES (?1, ?2)", params![kind_key, fp])?;
        let doomed: Vec<i64> = {
            let mut stmt = tx.prepare(
                "SELECT f.id, f.start_byte, f.end_byte, m.content FROM findings f JOIN messages m ON m.id = f.message_id WHERE f.kind = ?1",
            )?;
            let rows = stmt.query_map([&kind_key], |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)? as usize, r.get::<_, i64>(2)? as usize, r.get::<_, String>(3)?))
            })?;
            let mut ids = Vec::new();
            for row in rows {
                let (id, s, e, text) = row?;
                if text.get(s..e).is_some_and(|v| fingerprint(kind, v) == fp) {
                    ids.push(id);
                }
            }
            ids
        };
        {
            let mut del = tx.prepare("DELETE FROM findings WHERE id = ?1")?;
            for id in &doomed {
                del.execute([id])?;
            }
        }
        tx.commit()?;
        Ok(doomed.len())
    }

    /// Forgets every "never flag this again" rule. The items come back on the next scan.
    pub fn clear_ignored(&self) -> Result<usize> {
        Ok(self.conn.execute("DELETE FROM ignored", [])?)
    }

    /// Writes a CSV of flagged messages with no message text in it: IDs, dates, status
    /// and where to find each one (a web link, or a file path for files on this computer).
    pub fn export_flagged_csv(&self, path: &std::path::Path) -> Result<usize> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT ch.platform, ch.name, m.uid, m.ts, MAX(f.severity), GROUP_CONCAT(DISTINCT f.kind),
                    CASE WHEN SUM(r.status IS NULL) > 0 THEN 'to review'
                         WHEN SUM(r.status = 'later') > 0 THEN 'later' ELSE 'done' END,
                    COALESCE(m.link, CASE WHEN ch.platform = 'pc' THEN m.attachments END, '')
             {FINDING_FROM}
             GROUP BY f.message_id ORDER BY ch.platform, ch.name, m.ts"
        ))?;
        let mut w = csv::Writer::from_path(path)?;
        w.write_record(["platform", "where", "id", "sent_at_utc_ms", "severity", "kinds", "status", "link_or_path"])?;
        let mut n = 0;
        let rows = stmt.query_map([], |r| {
            Ok([
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?.to_string(),
                r.get::<_, i64>(4)?.to_string(),
                r.get::<_, String>(5)?,
                r.get::<_, String>(6)?,
                r.get::<_, String>(7)?,
            ])
        })?;
        for row in rows {
            w.write_record(row?)?;
            n += 1;
        }
        w.flush()?;
        Ok(n)
    }
}
