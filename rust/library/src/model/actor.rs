use crate::activitypub::object::Object;
use crate::settings::Settings;
use openssl::{
    hash::MessageDigest,
    pkey::{PKey, Public},
    rsa::Rsa,
    sign::Verifier,
};

use tracing::{event, Level};

pub async fn get_public_key(
    actor_id: &str,
    settings: &rocket::State<Settings>,
) -> Result<PKey<Public>, String> {
    let domain = get_domain(actor_id)?;
    if domain == settings.domain_name {
        let url_split = actor_id.split('/').collect::<Vec<&str>>();
        let username = url_split[url_split.len() - 1];
        return super::user::get_public_key(username, settings).await;
    }
    if let Some(actor) = get_from_cache(actor_id, settings).await {
        let public_key = actor.public_key.unwrap();
        let rsa = Rsa::public_key_from_pem(public_key.public_key_pem.as_bytes()).unwrap();
        return PKey::from_rsa(rsa).map_err(|err| format!("failed to parse public key: {err:?}"));
    }

    Err("actor not found".to_owned())
}

fn get_domain(actor_id: &str) -> Result<&str, String> {
    let url_split = actor_id.split('/').collect::<Vec<&str>>();
    if url_split.len() < 4 {
        return Err("invalid actor id".to_owned());
    }

    Ok(url_split[2])
}

impl crate::activitypub::verifier::Verifier for PKey<Public> {
    /// Verify if the signature is valid
    fn verify(&self, data: &str, signature: &[u8]) -> Result<bool, String> {
        event!(
            Level::DEBUG,
            public_key = hex::encode(self.public_key_to_der().unwrap())
        );
        let mut verifier = Verifier::new(MessageDigest::sha256(), self)
            .map_err(|e| format!("Failed to create verifier {e:?}"))?;
        verifier
            .update(data.as_bytes())
            .map_err(|e| format!("Failed to update verifier {e:?}"))?;
        verifier
            .verify(signature)
            .map_err(|e| format!("Failed to verify {e:?}"))
    }
}

async fn get_from_cache(actor_id: &str, settings: &rocket::State<Settings>) -> Option<Object> {
    if let Some(actor) = get_from_db(actor_id, settings).await {
        return Some(actor);
    }

    let actor = get_from_url(actor_id).await;
    create(actor_id, &actor, settings).await.ok()?;
    Some(actor)
}

async fn get_from_db(actor_id: &str, settings: &rocket::State<Settings>) -> Option<Object> {
    let domain = get_domain(actor_id).unwrap();
    let partition = format!("actor/{domain}");
    let get_item_output = crate::dynamodb::get_item(
        &settings.db_client,
        &settings.table_name,
        partition.as_str(),
        actor_id,
        "publicKey",
    )
    .await
    .unwrap();
    if let Some(item) = get_item_output.item {
        let actor: Object = serde_dynamo::from_item(item).unwrap();
        return Some(actor);
    }

    None
}

/// # Panics
///
/// Will panic if it insert the new row.
async fn create(
    actor_id: &str,
    object: &Object,
    settings: &rocket::State<Settings>,
) -> Result<(), String> {
    let domain = get_domain(actor_id)?;
    let partition = format!("actor/{domain}");
    crate::dynamodb::put_item(
        &settings.db_client,
        &settings.table_name,
        partition.as_str(),
        actor_id,
        object,
    )
    .await
    .unwrap();
    Ok(())
}

async fn get_from_url(user_url: &str) -> Object {
    let http_client = reqwest::Client::new();
    let actual_response = http_client
        .get(user_url)
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await
        .unwrap();
    event!(Level::DEBUG, "{actual_response:?}");
    let text = &actual_response.text().await.unwrap();
    event!(Level::DEBUG, text);
    serde_json::from_str::<Object>(text).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activitypub::object::PublicKey;

    #[tokio::test]
    async fn test_get() {
        // Arrange
        let expected_object = Object {
            actor: None,
            atom_uri: None,
            attachment: Some(Vec::new()),
            attributed_to: None,
            cc: None,
            content: None,
            context: crate::activitypub::context::default(),
            conversation: None,
            devices: Some("https://example.com/users/test_username/collections/devices".to_owned()),
            discoverable: Some(false),
            followers: Some("https://example.com/users/test_username/followers".to_owned()),
            following: Some("https://example.com/users/test_username/following".to_owned()),
            id: Some("https://example.com/users/test_username".to_owned()),
            in_reply_to: None,
            in_reply_to_atom_uri: None,
            inbox: Some("https://example.com/users/test_username/inbox".to_owned()),
            manually_approves_followers: Some(false),
            name: Some("test_username".to_owned()),
            object: None,
            outbox: Some("https://example.com/users/test_username/outbox".to_owned()),
            preferred_username: Some("test_username".to_owned()),
            public_key: Some(PublicKey {
                id: "https://example.com/users/test_username#main-key".to_owned(),
                owner: "https://example.com/users/test_username".to_owned(),
                public_key_pem: r#"-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA2eb+zBEDtwpqAxLGy12D
rn1khJ2KWsyEn00UB5gj0UZQS9K1wzmLMCGRjayEDKBFMvgozJncN5fygRRYEpAC
Iw4iglLta4PlB3ATLltnpidYcCZXMGSs7AVwXGfuGPmZV+wqUaiYekQv0yMnX8Vx
4JTMk4GNo3mY1UJRLQxQXaYSrnQ58Vg2bs8RStcA3NpzTzn4FEz4wtWDDpUpGGFd
p6JWobKcNxtBAaebWwJbPHOy2/3EXAFUgsClyftczO3ARWXDIL82IO/YBVSbh0IF
nwRTxyaBbvB3mPAh63fyt9YivwEZtMwUSFRmlNO//E4Zwz2f872Ij2H81mMTEwRq
aQIDAQAB
-----END PUBLIC KEY-----
"#
                .to_owned(),
            }),
            published: Some("2023-01-19T00:00:00Z".to_owned()),
            r#type: Some("Person".to_owned()),
            sensitive: None,
            summary: Some(String::new()),
            tag: Some(Vec::new()),
            to: None,
            url: Some("https://example.com/@test_username".to_owned()),
            extra: serde_json::Value::Null,
        };

        // Act
        let actual = get_from_url("https://example.com/users/test_username").await;

        // Assert
        assert_eq!(actual, expected_object);
    }
}
