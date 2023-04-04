use http::header::{HeaderMap, HeaderValue};
use openssl::{hash::MessageDigest, pkey::PKey, rsa::Rsa, sign::Signer};
use time::format_description::well_known::Rfc2822;
use time::OffsetDateTime;

use base64::{engine::general_purpose, Engine as _};
use tracing::{event, Level};

const SELECT_HEADERS: &str = "(request-target) host date digest content-type";

pub fn sign(
    method: &str,
    path: &str,
    all_headers: &mut HeaderMap,
    request_body: &str,
    private_key: &[u8],
    signature_key_id: &str,
) {
    insert_digest(all_headers, request_body);
    insert_date(all_headers);
    insert_signature(method, path, all_headers, private_key, signature_key_id);
}

fn insert_digest(all_headers: &mut HeaderMap, request_body: &str) {
    let digest = super::digest::Digest::from_body(request_body);
    let digest = HeaderValue::from_str(&digest).unwrap();
    all_headers.insert("digest", digest);
}

fn insert_date(all_headers: &mut HeaderMap) {
    let date: OffsetDateTime = OffsetDateTime::now_utc();
    let date = date.format(&Rfc2822).unwrap();
    let date = date.as_str().replace("+0000", "GMT");
    let date = HeaderValue::from_str(&date).unwrap();
    all_headers.insert("date", date);
}

fn insert_signature(
    method: &str,
    path: &str,
    all_headers: &mut HeaderMap,
    private_key: &[u8],
    signature_key_id: &str,
) {
    let select_headers = select_headers(method, path, all_headers, SELECT_HEADERS);
    let signature = get_signature(private_key, &select_headers).unwrap();
    let signature = general_purpose::STANDARD.encode(signature);
    event!(Level::DEBUG, signature = signature);
    let signature_header = format!("keyId=\"{signature_key_id}\",algorithm=\"rsa-sha256\",headers=\"{SELECT_HEADERS}\",signature=\"{signature}\"");
    let signature_header = HeaderValue::from_str(signature_header.as_str()).unwrap();
    all_headers.insert("signature", signature_header);
    event!(Level::DEBUG, all_headers = format!("{all_headers:?}"));
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

fn get_signature(private_key: &[u8], to_sign: &str) -> Result<Vec<u8>, String> {
    let key = PKey::from_rsa(
        Rsa::private_key_from_der(private_key)
            .map_err(|e| format!("Failed to private_key_from_der {e:?}"))?,
    )
    .map_err(|e| format!("Failed to from_rsa {e:?}"))?;
    let mut signer = Signer::new(MessageDigest::sha256(), &key)
        .map_err(|e| format!("Failed to create signer {e:?}"))?;
    event!(Level::DEBUG, to_sign = to_sign);
    signer
        .update(to_sign.as_bytes())
        .map_err(|e| format!("Failed to update signer {e:?}"))?;
    signer
        .sign_to_vec()
        .map_err(|e| format!("Failed to sign_to_vec {e:?}"))
}
