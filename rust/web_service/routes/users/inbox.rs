use library::activitypub::digest::Digest;
use library::activitypub::signature;
use library::settings::Settings;
use rocket::response::status::BadRequest;
use tracing::{event, Level};

#[rocket::post("/users/<username>/inbox", data = "<data>")]
pub async fn handler(
    username: &str,
    headers: library::activitypub::headers::Headers<'_>,
    data: String,
    settings: &rocket::State<Settings>,
) -> Result<String, BadRequest<String>> {
    for header in headers.0.iter() {
        event!(Level::DEBUG, "{}: {}", header.name(), header.value());
    }

    event!(Level::DEBUG, "body = {data}");
    let activity: serde_json::Value = serde_json::from_str(&data).unwrap();
    let actor_id = activity["actor"]
        .as_str()
        .ok_or_else(|| BadRequest(Some("Missing actor id for activity".to_owned())))?;
    let public_key = library::model::actor::get_public_key(actor_id, settings)
        .await
        .map_err(|err| BadRequest(Some(err)))?;
    let digest_header = headers.0.get_one("digest").unwrap();
    if signature::is_valid(&public_key, &headers.0, &Digest(digest_header.to_owned())) {
        let partition = format!("users/{username}/followers");
        library::dynamodb::put_item(
            &settings.db_client,
            &settings.table_name,
            partition.as_str(),
            actor_id,
            &activity,
        )
        .await
        .unwrap();
        return Ok(data);
    }

    Err(BadRequest(Some("Invalid signature or digest".to_owned())))
}
