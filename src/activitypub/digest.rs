// copied from https://github.com/Plume-org/Plume/blob/main/plume-common/src/activity_pub/request.rs
use base64::{engine::general_purpose, Engine as _};
use openssl::hash::{Hasher, MessageDigest};
use reqwest::header::HeaderValue;

pub struct Digest(pub String);

impl Digest {
    pub fn digest(body: &str) -> HeaderValue {
        let mut hasher =
            Hasher::new(MessageDigest::sha256()).expect("Digest::digest: initialization error");
        hasher
            .update(body.as_bytes())
            .expect("Digest::digest: content insertion error");
        let res = general_purpose::STANDARD
            .encode(&hasher.finish().expect("Digest::digest: finalizing error"));
        HeaderValue::from_str(&format!("SHA-256={}", res))
            .expect("Digest::digest: header creation error")
    }

    pub fn verify(&self, body: &str) -> bool {
        if self.algorithm() == "SHA-256" {
            let mut hasher =
                Hasher::new(MessageDigest::sha256()).expect("Digest::digest: initialization error");
            hasher
                .update(body.as_bytes())
                .expect("Digest::digest: contfent insertion error");
            self.value()
                == hasher
                    .finish()
                    .expect("Digest::digest: finalizing error")
                    .as_ref()
        } else {
            false //algorithm not supported
        }
    }

    pub fn verify_header(&self, other: &Digest) -> bool {
        self.value() == other.value()
    }

    pub fn algorithm(&self) -> &str {
        let pos = self
            .0
            .find('=')
            .expect("Digest::algorithm: invalid header error");
        &self.0[..pos]
    }

    pub fn value(&self) -> Vec<u8> {
        let pos = self
            .0
            .find('=')
            .expect("Digest::value: invalid header error")
            + 1;
        general_purpose::STANDARD
            .decode(&self.0[pos..])
            .expect("Digest::value: invalid encoding error")
    }

    pub fn from_header(dig: &str) -> Result<Self, String> {
        if let Some(pos) = dig.find('=') {
            let pos = pos + 1;
            if general_purpose::STANDARD.decode(&dig[pos..]).is_ok() {
                Ok(Digest(dig.to_owned()))
            } else {
                Err("Digest::from_header: invalid algorithm".to_owned())
            }
        } else {
            Err("Digest::from_header: invalid header".to_owned())
        }
    }

    pub fn from_body(body: &str) -> Self {
        let mut hasher =
            Hasher::new(MessageDigest::sha256()).expect("Digest::digest: initialization error");
        hasher
            .update(body.as_bytes())
            .expect("Digest::digest: content insertion error");
        let res = general_purpose::STANDARD
            .encode(&hasher.finish().expect("Digest::digest: finalizing error"));
        Digest(format!("SHA-256={}", res))
    }
}
