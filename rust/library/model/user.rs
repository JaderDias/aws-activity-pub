// copied from https://github.com/Plume-org/Plume/blob/main/plume-models/src/users.rs
use crate::settings::Settings;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use std::time::{SystemTime, UNIX_EPOCH};

use openssl::{
    pkey::{PKey, Public},
    rsa::Rsa,
};
use serde::{Deserialize, Serialize};

const KEYSIZE: u32 = 4096;

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
    pub published_unix_time_seconds: u64,
}

impl User {
    #[must_use]
    pub fn get_published_time(&self) -> String {
        let published_time =
            OffsetDateTime::from_unix_timestamp(self.published_unix_time_seconds as i64)
                .expect("OffsetDateTime::from_unix_timestamp");

        published_time.format(&Rfc3339).unwrap()
    }
}

/// # Panics
///
/// Will panic if it can´t get the user.
pub async fn get_item(
    username: &str,
    db_client: &aws_sdk_dynamodb::Client,
    table_name: &str,
) -> aws_sdk_dynamodb::operation::get_item::GetItemOutput {
    let partition = format!("users/{username}");
    crate::dynamodb::get_item(
        db_client,
        table_name,
        partition.as_str(),
        "user",
        "private_key, public_key, published_unix_time_seconds",
    )
    .await
    .unwrap()
}

/// # Panics
///
/// Will panic if it can´t get the user.
pub async fn get(username: &str, settings: &rocket::State<Settings>) -> Option<User> {
    let get_item_output = get_item(username, &settings.db_client, &settings.table_name).await;
    if let Some(item) = get_item_output.item {
        let user: User = serde_dynamo::from_item(item).unwrap();
        return Some(user);
    }

    None
}

/// # Errors
///
/// Returns an error if the user is not found.
pub async fn get_public_key(
    username: &str,
    settings: &rocket::State<Settings>,
) -> Result<PKey<Public>, String> {
    if let Some(user) = get(username, settings).await {
        let public_key = user.public_key.unwrap();
        let rsa = Rsa::public_key_from_der(&public_key).unwrap();
        return PKey::from_rsa(rsa).map_err(|e| format!("Failed to from_rsa {e:?}"));
    }

    Err("User not found".to_owned())
}

/// # Panics
///
/// Will panic if it can´t generate the private key.
pub async fn create(
    db_client: &aws_sdk_dynamodb::Client,
    table_name: &str,
    preferred_username: &str,
) -> User {
    let since_unix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let keypair = Rsa::generate(KEYSIZE).unwrap();
    let partition = format!("users/{preferred_username}");
    let user = crate::model::user::User {
        preferred_username: Some(preferred_username.to_owned()),
        private_key: Some(keypair.private_key_to_der().unwrap()),
        public_key: Some(keypair.public_key_to_der().unwrap()),
        published_unix_time_seconds: since_unix.as_secs(),
    };
    crate::dynamodb::put_item(db_client, table_name, partition.as_str(), "user", &user)
        .await
        .unwrap();
    user
}
