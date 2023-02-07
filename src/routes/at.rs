use rocket::serde::json::Json;

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![at]
}

#[rocket::get("/<path>")]
fn at(path: &str) -> Option<Json<serde_json::Value>> {
    if path.len() < 2 || !path.starts_with('@') {
        return None;
    }
    let username = &path[1..];
    Some(Json(serde_json::json!({
        "@context": [
            "https://www.w3.org/ns/activitystreams",
            "https://w3id.org/security/v1"
        ],
        "id": format!("https://example.com/{path}"),
        "type": "Person",
        "following": "https://example.com/following",
        "followers": "https://example.com/followers",
        "inbox": "https://example.com/inbox",
        "preferredUsername": username,
        "name": "Example user",
        "summary": "Activity Pub server example.",
        "url": "https://example.com",
        "manuallyApprovesFollowers": false,
        "discoverable": true,
        "published": "2000-01-01T00:00:00Z"
    })))
}
