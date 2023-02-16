use rocket::serde::json::Json;

#[rocket::get("/users/<username>/following")]
pub fn handler(
    username: &str,
    settings: &rocket::State<crate::settings::Settings>,
) -> Json<serde_json::Value> {
    let domain = settings.domain_name.as_str();
    let user_uri = format!("https://{domain}/users/{username}");
    Json(serde_json::json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": format!("{user_uri}/following"),
        "type": "OrderedCollection",
        "totalItems": 1, // TODO: real number from db
        "first": format!("{user_uri}/following?page=1"),
    }))
}
