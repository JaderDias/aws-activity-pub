use super::signer::Signer;
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Duration, Utc};
use openssl::sha::sha256;

pub trait Signable {
    fn sign<T>(&mut self, creator: &T) -> Result<&mut Self, String>
    where
        T: Signer;
    fn verify<T>(self, creator: &T) -> bool
    where
        T: Signer;

    fn hash(data: &str) -> String {
        let bytes = data.as_bytes();
        hex::encode(sha256(bytes))
    }
}

impl Signable for serde_json::Value {
    fn sign<T: Signer>(&mut self, creator: &T) -> Result<&mut serde_json::Value, String> {
        let creation_date = Utc::now().to_rfc3339();
        let mut options = serde_json::json!({
            "type": "RsaSignature2017",
            "creator": creator.get_key_id(),
            "created": creation_date
        });

        let options_hash = Self::hash(
            &serde_json::json!({
                "@context": "https://w3id.org/identity/v1",
                "created": creation_date
            })
            .to_string(),
        );
        let document_hash = Self::hash(&self.to_string());
        let to_be_signed = options_hash + &document_hash;

        let signature = general_purpose::STANDARD.encode(
            &creator
                .sign(&to_be_signed)
                .map_err(|e| format!("couldn't sign {e:?}"))?,
        );

        options["signatureValue"] = serde_json::Value::String(signature);
        self["signature"] = options;
        Ok(self)
    }

    fn verify<T: Signer>(mut self, creator: &T) -> bool {
        let signature_obj =
            if let Some(sig) = self.as_object_mut().and_then(|o| o.remove("signature")) {
                sig
            } else {
                //signature not present
                return false;
            };
        let signature = if let Ok(sig) = general_purpose::STANDARD
            .decode(&signature_obj["signatureValue"].as_str().unwrap_or(""))
        {
            sig
        } else {
            return false;
        };
        let creation_date = &signature_obj["created"];
        let options_hash = Self::hash(
            &serde_json::json!({
                "@context": "https://w3id.org/identity/v1",
                "created": creation_date
            })
            .to_string(),
        );
        let creation_date = creation_date.as_str();
        if creation_date.is_none() {
            return false;
        }
        let creation_date = DateTime::parse_from_rfc3339(creation_date.unwrap());
        if creation_date.is_err() {
            return false;
        }
        let diff = creation_date.unwrap().signed_duration_since(Utc::now());
        let future = Duration::hours(12);
        let past = Duration::hours(-12);
        if !(diff < future && diff > past) {
            return false;
        }
        let document_hash = Self::hash(&self.to_string());
        let to_be_signed = options_hash + &document_hash;
        creator.verify(&to_be_signed, &signature).unwrap_or(false)
    }
}
