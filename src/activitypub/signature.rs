// copied from https://github.com/Plume-org/Plume/blob/main/plume-common/src/activity_pub/sign.rs
use super::digest::Digest;
use base64::{engine::general_purpose, Engine as _};
use chrono::{offset::Utc, Duration, NaiveDateTime};
use rocket::http::HeaderMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SignatureValidity {
    Invalid,
    ValidNoDigest,
    Valid,
    Absent,
    Outdated,
}

pub struct SignatureHeader {
    pub algorithm: Option<String>,
    pub headers: Option<String>,
    pub signature: Option<String>,
}

pub fn parse_header(signature_header: &str) -> SignatureHeader {
    let mut result: SignatureHeader = SignatureHeader {
        algorithm: None,
        headers: None,
        signature: None,
    };
    for part in signature_header.split(',') {
        match part {
            part if part.starts_with("algorithm=") => {
                result.algorithm = Some((&part[11..part.len() - 1]).to_owned())
            }
            part if part.starts_with("headers=") => {
                result.headers = Some((&part[9..part.len() - 1]).to_owned())
            }
            part if part.starts_with("signature=") => {
                result.signature = Some((&part[11..part.len() - 1]).to_owned())
            }
            _ => {}
        }
    }

    result
}

fn select_headers(all_headers: &HeaderMap<'_>, query: &str) -> String {
    query
        .split_whitespace()
        .map(|header| (header, all_headers.get_one(header)))
        .map(|(header, value)| format!("{}: {}", header.to_lowercase(), value.unwrap_or("")))
        .collect::<Vec<_>>()
        .join("\n")
}

/// # Panics
///
/// Will panic if it can´t parse the date in the header.
pub fn verify_http_headers<S: super::verifier::Verifier + ::std::fmt::Debug>(
    sender: &S,
    all_headers: &HeaderMap<'_>,
    data: &Digest,
) -> SignatureValidity {
    println!("verify_http_headers");
    let signature_header = all_headers.get_one("Signature");
    if signature_header.is_none() {
        println!("missing signature header");
        return SignatureValidity::Absent;
    }
    let signature_header = signature_header.expect("sign::verify_http_headers: unreachable");
    let signature_header = parse_header(signature_header);
    if signature_header.signature.is_none() || signature_header.headers.is_none() {
        println!("missing part of headers");
        //missing part of the header
        return SignatureValidity::Invalid;
    }
    let signature = signature_header
        .signature
        .expect("sign::verify_http_headers: unreachable");
    println!("signature: {:?}", signature);
    let signature = general_purpose::STANDARD
        .decode(signature)
        .expect("sign::verify_http_headers: can't decode signature");
    let headers = signature_header.headers.unwrap();
    let select_headers = select_headers(all_headers, headers.as_str());
    println!("verify_http_headers: {}", select_headers);

    if !sender.verify(&select_headers, &signature).unwrap_or(false) {
        println!("invalid signature");
        return SignatureValidity::Invalid;
    }
    if !headers.contains(&"digest") {
        println!("valid no digest");
        // signature is valid, but body content is not verified
        return SignatureValidity::ValidNoDigest;
    }
    let digest = all_headers.get_one("digest").unwrap_or("");
    let digest = Digest::from_header(digest);
    if !digest.map(|d| d.verify_header(data)).unwrap_or(false) {
        println!("valid but digest doesn't match");
        // signature was valid, but body content does not match its digest
        return SignatureValidity::Invalid;
    }
    if !headers.contains(&"date") {
        println!("valid no date");
        return SignatureValidity::Valid; //maybe we shouldn't trust a request without date?
    }

    let date = all_headers.get_one("date");
    if date.is_none() {
        println!("missing date header");
        return SignatureValidity::Outdated;
    }
    let date = NaiveDateTime::parse_from_str(date.unwrap(), "%a, %d %h %Y %T GMT");
    if date.is_err() {
        println!("invalid date header");
        return SignatureValidity::Outdated;
    }
    let diff = Utc::now().naive_utc() - date.unwrap();
    let future = Duration::hours(12);
    let past = Duration::hours(-12);
    if diff < future && diff > past {
        println!("valid");
        SignatureValidity::Valid
    } else {
        println!("valid but date is in the past");
        SignatureValidity::Outdated
    }
}
