// copied from https://github.com/Plume-org/Plume/blob/main/plume-common/src/activity_pub/sign.rs
use super::digest::Digest;
use base64::{engine::general_purpose, Engine as _};
use rocket::http::HeaderMap;
use time::format_description::well_known::Rfc2822;
use time::{Duration, OffsetDateTime, PrimitiveDateTime};
use tracing::{event, Level};

pub struct SignatureHeader {
    pub algorithm: Option<String>,
    pub headers: Option<String>,
    pub signature: Option<String>,
}

#[must_use]
pub fn parse_header(signature_header: &str) -> SignatureHeader {
    let mut result: SignatureHeader = SignatureHeader {
        algorithm: None,
        headers: None,
        signature: None,
    };
    for part in signature_header.split(',') {
        match part {
            part if part.starts_with("algorithm=") => {
                result.algorithm = Some(part[11..part.len() - 1].to_owned());
            }
            part if part.starts_with("headers=") => {
                result.headers = Some(part[9..part.len() - 1].to_owned());
            }
            part if part.starts_with("signature=") => {
                result.signature = Some(part[11..part.len() - 1].to_owned());
            }
            _ => {}
        }
    }

    result
}

/// # Panics
///
/// Will panic if it can´t parse the date in the header.
pub fn is_valid<S: super::verifier::Verifier + ::std::fmt::Debug>(
    sender: &S,
    all_headers: &HeaderMap<'_>,
    content: &Digest,
) -> bool {
    event!(Level::DEBUG, "verify_http_headers");
    let signature_header = all_headers.get_one("Signature");
    if signature_header.is_none() {
        event!(Level::DEBUG, "missing signature header");
        return false;
    }
    let signature_header = signature_header.expect("sign::verify_http_headers: unreachable");
    let signature_header = parse_header(signature_header);
    if signature_header.signature.is_none() || signature_header.headers.is_none() {
        event!(Level::DEBUG, "missing part of headers");
        return false;
    }
    let signature = signature_header
        .signature
        .expect("sign::verify_http_headers: unreachable");
    event!(Level::DEBUG, signature = signature);
    let signature = general_purpose::STANDARD
        .decode(signature)
        .expect("sign::verify_http_headers: can't decode signature");
    let headers = signature_header.headers.unwrap();
    let select_headers = super::headers::select(all_headers, headers.as_str());
    event!(Level::DEBUG, select_headers = select_headers);

    if !sender.verify(&select_headers, &signature).unwrap_or(false) {
        event!(Level::DEBUG, "invalid signature");
        return false;
    }

    if !headers.contains("digest") {
        event!(Level::DEBUG, "valid no digest");
        return false;
    }
    let digest_validity = verify_digest(all_headers, content);
    if !digest_validity {
        return false;
    }

    if !headers.contains("date") {
        event!(Level::DEBUG, "valid no date");
        return false;
    }

    verify_date_header(all_headers)
}

pub fn verify_digest(all_headers: &HeaderMap<'_>, content: &Digest) -> bool {
    let digest = all_headers.get_one("digest").unwrap_or("");
    let digest = Digest::from_header(digest);
    if !digest.map(|d| d.verify_header(content)).unwrap_or(false) {
        event!(Level::DEBUG, "valid but digest doesn't match");
        return false;
    }

    true
}

pub fn verify_date_header(all_headers: &HeaderMap<'_>) -> bool {
    let date = all_headers.get_one("date");
    if date.is_none() {
        event!(Level::DEBUG, "missing date header");
        return false;
    }
    let date = PrimitiveDateTime::parse(date.unwrap(), &Rfc2822);
    let date = if let Ok(date) = date {
        date.assume_utc()
    } else {
        event!(Level::DEBUG, "invalid date header");
        return false;
    };
    verify_date(date)
}

pub fn verify_date(date: OffsetDateTime) -> bool {
    let diff = OffsetDateTime::now_utc() - date;
    let future = Duration::hours(12);
    let past = Duration::hours(-12);
    if diff < future && diff > past {
        event!(Level::DEBUG, "valid");
        true
    } else {
        event!(Level::DEBUG, "valid but date is in the past");
        false
    }
}
