use rocket::serde::json::Json;

#[rocket::get("/users/<username>/following")]
pub fn handler(
    username: &str,
    settings: &rocket::State<library::settings::Settings>,
) -> Json<serde_json::Value> {
    let user_uri = format!("{}/users/{username}", settings.base_url);
    Json(serde_json::json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": format!("{user_uri}/following"),
        "type": "OrderedCollection",
        "totalItems": 1, // TODO: real number from db
        "first": format!("{user_uri}/following?page=1"),
    }))
}
