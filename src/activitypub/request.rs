use http::header::{HeaderMap, HeaderValue};
use time::format_description::well_known::Rfc2822;
use time::OffsetDateTime;

use base64::{engine::general_purpose, Engine as _};
use tracing::{event, Level};

const SELECT_HEADERS: &'static str = "(request-target) host date digest content-type";

pub fn request<T: super::signer::Signer>(
    method: &str,
    path: &str,
    all_headers: &mut HeaderMap,
    request_body: &str,
    signer: &T,
    signature_key_id: &str,
) {
    insert_digest(all_headers, &request_body);
    insert_date(all_headers);
    insert_signature(method, path, all_headers, signer, signature_key_id);
}

fn insert_digest(all_headers: &mut HeaderMap, request_body: &str) {
    let digest = super::digest::Digest::from_body(request_body);
    event!(Level::DEBUG, digest = digest);
    let digest = HeaderValue::from_str(&digest).unwrap();
    all_headers.insert("digest", digest);
}

fn insert_date(all_headers: &mut HeaderMap) {
    let date: OffsetDateTime = OffsetDateTime::now_utc();
    let date = date.format(&Rfc2822).unwrap();
    let date = HeaderValue::from_str(&date).unwrap();
    all_headers.insert("date", date);
}

fn insert_signature<T: super::signer::Signer>(
    method: &str,
    path: &str,
    all_headers: &mut HeaderMap,
    signer: &T,
    signature_key_id: &str,
) {
    let select_headers = select_headers(method, path, all_headers, SELECT_HEADERS);
    let signature = signer.sign(&select_headers).unwrap();
    let signature = general_purpose::STANDARD.encode(signature);
    event!(Level::DEBUG, signature = signature);
    let signature_header = format!("keyId=\"{signature_key_id}\",algorithm=\"rsa-sha256\",headers=\"{SELECT_HEADERS}\",signature=\"{signature}\"");
    let signature_header = HeaderValue::from_str(signature_header.as_str()).unwrap();
    all_headers.insert("signature", signature_header);
    event!(Level::DEBUG, all_headers = format!("{:?}", all_headers));
}

fn select_headers(method: &str, path: &str, all_headers: &HeaderMap, query: &str) -> String {
    event!(
        Level::DEBUG,
        query = query,
        method = method,
        path = path,
        all_headers = format!("{all_headers:?}"),
    );
    query
        .split_whitespace()
        .map(|header| {
            if header == "(request-target)" {
                return format!("(request-target): {} {}", method.to_lowercase(), path);
            }
            format!(
                "{}: {}",
                header.to_lowercase(),
                all_headers.get(header).unwrap().to_str().unwrap_or("")
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
