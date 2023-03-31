// copied from https://github.com/Plume-org/Plume/blob/main/plume-common/src/activity_pub/request.rs
use base64::{engine::general_purpose, Engine as _};
use openssl::hash::{Hasher, MessageDigest};
use tracing::{event, Level};

pub struct Digest(pub String);

impl Digest {
    #[must_use]
    pub fn from_body(body: &str) -> String {
        let mut hasher =
            Hasher::new(MessageDigest::sha256()).expect("Digest::digest: initialization error");
        hasher
            .update(body.as_bytes())
            .expect("Digest::digest: content insertion error");
        let res = general_purpose::STANDARD
            .encode(hasher.finish().expect("Digest::digest: finalizing error"));
        format!("SHA-256={res}")
    }

    #[must_use]
    pub fn verify(&self, body: &str) -> bool {
        event!(Level::DEBUG, "verify digest");
        if self.algorithm() == "SHA-256" {
            let mut hasher =
                Hasher::new(MessageDigest::sha256()).expect("Digest::digest: initialization error");
            hasher
                .update(body.as_bytes())
                .expect("Digest::digest: content insertion error");
            self.value()
                == hasher
                    .finish()
                    .expect("Digest::digest: finalizing error")
                    .as_ref()
        } else {
            false //algorithm not supported
        }
    }

    #[must_use]
    pub fn verify_header(&self, other: &Self) -> bool {
        event!(Level::DEBUG, "verify_header digest");
        self.value() == other.value()
    }

    #[must_use]
    pub fn algorithm(&self) -> &str {
        let pos = self
            .0
            .find('=')
            .expect("Digest::algorithm: invalid header error");
        &self.0[..pos]
    }

    #[must_use]
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

    /// # Errors
    ///
    /// Will return `Err` if an invalid header is provided or if it uses an invalid algorithm.
    pub fn from_header(dig: &str) -> Result<Self, String> {
        dig.find('=').map_or_else(
            || Err("Digest::from_header: invalid header".to_owned()),
            |pos| {
                let pos = pos + 1;
                if general_purpose::STANDARD.decode(&dig[pos..]).is_ok() {
                    Ok(Self(dig.to_owned()))
                } else {
                    Err("Digest::from_header: invalid algorithm".to_owned())
                }
            },
        )
    }
}
