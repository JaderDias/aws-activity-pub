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
