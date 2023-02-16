// copied from https://github.com/Plume-org/Plume/blob/main/plume-models/src/users.rs
use crate::settings::Settings;
use openssl::{
    hash::MessageDigest,
    pkey::PKey,
    rsa::Rsa,
    sign::{Signer, Verifier},
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "preferredUsername")]
    pub preferred_username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "serde_bytes")]
    pub private_key: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "serde_bytes")]
    pub public_key: Option<Vec<u8>>,
}

impl crate::activitypub::sign::Signer for User {
    fn get_key_id(&self) -> String {
        format!("{}#main-key", self.preferred_username.as_ref().unwrap())
    }

    /// Sign some data with the signer keypair
    fn sign(&self, to_sign: &str) -> Result<Vec<u8>, String> {
        let key = PKey::from_rsa(
            Rsa::private_key_from_pem(self.private_key.as_ref().unwrap())
                .map_err(|e| format!("Failed to private_key_from_pem {:?}", e))?,
        )
        .map_err(|e| format!("Failed to from_rsa {:?}", e))?;
        let mut signer = Signer::new(MessageDigest::sha256(), &key)
            .map_err(|e| format!("Failed to create signer {:?}", e))?;
        signer
            .update(to_sign.as_bytes())
            .map_err(|e| format!("Failed to update signer {:?}", e))?;
        signer
            .sign_to_vec()
            .map_err(|e| format!("Failed to sign_to_vec {:?}", e))
    }
    /// Verify if the signature is valid
    fn verify(&self, data: &str, signature: &[u8]) -> Result<bool, String> {
        let key = PKey::from_rsa(
            Rsa::public_key_from_pem(self.public_key.as_ref().unwrap())
                .map_err(|e| format!("Failed to public_key_from_pem {:?}", e))?,
        )
        .map_err(|e| format!("Failed to from_rsa {:?}", e))?;
        let mut verifier = Verifier::new(MessageDigest::sha256(), &key)
            .map_err(|e| format!("Failed to create verifier {:?}", e))?;
        verifier
            .update(data.as_bytes())
            .map_err(|e| format!("Failed to update verifier {:?}", e))?;
        verifier
            .verify(signature)
            .map_err(|e| format!("Failed to verify {:?}", e))
    }
}

pub async fn get(username: &str, settings: &rocket::State<Settings>) -> Option<User> {
    let partition = format!("users/{username}");
    let get_item_output = crate::dynamodb::get_item(
        &settings.db_client,
        &settings.table_name,
        partition.as_str(),
        "user",
    )
    .await
    .unwrap();
    if let Some(item) = get_item_output.item {
        let user: User = serde_dynamo::from_item(item).unwrap();
        return Some(user);
    }

    None
}
