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

        event!(Level::DEBUG, body = body);
        hasher
            .update(body.as_bytes())
            .expect("Digest::digest: content insertion error");
        let res = general_purpose::STANDARD
            .encode(hasher.finish().expect("Digest::digest: finalizing error"));
        event!(Level::DEBUG, digest = res);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_body() {
        // Arrange
        let body = r#"{"content":"test content","@context":null,"conversation":"tag:g4rxofniv3ethj5t4dgyma7dmy0wnukp.lambda-url.eu-central-1.on.aws,2019-04-28:objectId=1754000:objectType=Conversation","discoverable":false,"id":"https://g4rxofniv3ethj5t4dgyma7dmy0wnukp.lambda-url.eu-central-1.on.aws/users/test_username/statuses/7049093257877979136","published":"2023-01-19T00:00:00Z","type":"Note","sensitive":false,"url":"https://g4rxofniv3ethj5t4dgyma7dmy0wnukp.lambda-url.eu-central-1.on.aws/@test_username","partition_key":"users/sample_user8/statuses"}"#;
        let expected = "SHA-256=r5I/bD57JCWgwOaiBiS/RKDbaifG5qdlJNdTZfTWR1Q=";

        // Act
        let actual = Digest::from_body(body);

        // Assert
        assert_eq!(expected, actual);
    }
}
