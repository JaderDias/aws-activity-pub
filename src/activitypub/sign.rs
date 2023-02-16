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

pub trait Signer {
    fn get_key_id(&self) -> String;

    /// Sign some data with the signer keypair
    fn sign(&self, to_sign: &str) -> Result<Vec<u8>, String>;
    /// Verify if the signature is valid
    fn verify(&self, data: &str, signature: &[u8]) -> Result<bool, String>;
}

pub fn verify_http_headers<S: Signer + ::std::fmt::Debug>(
    sender: &S,
    all_headers: &HeaderMap<'_>,
    data: &Digest,
) -> SignatureValidity {
    let sig_header = all_headers.get_one("Signature");
    if sig_header.is_none() {
        return SignatureValidity::Absent;
    }
    let sig_header = sig_header.expect("sign::verify_http_headers: unreachable");

    let mut _key_id = None;
    let mut _algorithm = None;
    let mut headers = None;
    let mut signature = None;
    for part in sig_header.split(',') {
        match part {
            part if part.starts_with("keyId=") => _key_id = Some(&part[7..part.len() - 1]),
            part if part.starts_with("algorithm=") => _algorithm = Some(&part[11..part.len() - 1]),
            part if part.starts_with("headers=") => headers = Some(&part[9..part.len() - 1]),
            part if part.starts_with("signature=") => signature = Some(&part[11..part.len() - 1]),
            _ => {}
        }
    }

    if signature.is_none() || headers.is_none() {
        //missing part of the header
        return SignatureValidity::Invalid;
    }
    let headers = headers
        .expect("sign::verify_http_headers: unreachable")
        .split_whitespace()
        .collect::<Vec<_>>();
    let signature = signature.expect("sign::verify_http_headers: unreachable");
    let h = headers
        .iter()
        .map(|header| (header, all_headers.get_one(header)))
        .map(|(header, value)| format!("{}: {}", header.to_lowercase(), value.unwrap_or("")))
        .collect::<Vec<_>>()
        .join("\n");

    if !sender
        .verify(
            &h,
            general_purpose::STANDARD
                .decode(signature)
                .unwrap_or_default()
                .as_ref(),
        )
        .unwrap_or(false)
    {
        return SignatureValidity::Invalid;
    }
    if !headers.contains(&"digest") {
        // signature is valid, but body content is not verified
        return SignatureValidity::ValidNoDigest;
    }
    let digest = all_headers.get_one("digest").unwrap_or("");
    let digest = Digest::from_header(digest);
    if !digest.map(|d| d.verify_header(data)).unwrap_or(false) {
        // signature was valid, but body content does not match its digest
        return SignatureValidity::Invalid;
    }
    if !headers.contains(&"date") {
        return SignatureValidity::Valid; //maybe we shouldn't trust a request without date?
    }

    let date = all_headers.get_one("date");
    if date.is_none() {
        return SignatureValidity::Outdated;
    }
    let date = NaiveDateTime::parse_from_str(date.unwrap(), "%a, %d %h %Y %T GMT");
    if date.is_err() {
        return SignatureValidity::Outdated;
    }
    let diff = Utc::now().naive_utc() - date.unwrap();
    let future = Duration::hours(12);
    let past = Duration::hours(-12);
    if diff < future && diff > past {
        SignatureValidity::Valid
    } else {
        SignatureValidity::Outdated
    }
}
