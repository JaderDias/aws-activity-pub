use rocket::serde::json::Json;

#[rocket::get("/users/<username>/statuses/<status_id>/activity")]
pub async fn handler(
    username: &str,
    status_id: &str,
    settings: &rocket::State<library::settings::Settings>,
) -> Json<serde_json::Value> {
    let object = super::get_object(username, status_id, settings).await;

    let id = format!(
        "{}/users/{username}/statuses/{status_id}/activity",
        settings.base_url
    );
    Json(serde_json::json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": id,
        "type": "Create",
        "actor": object.actor,
        "published": object.published,
        "to": object.to,
        "cc": object.cc,
        "object": object
    }))
}
