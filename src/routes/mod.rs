mod api;
mod nodeinfo;
mod users;
mod well_known;

use rocket::serde::json::Json;

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![
        handler,
        api::statuses,
        nodeinfo::handler,
        well_known::webfinger,
    ]
    .into_iter()
    .chain(users::routes().into_iter())
    .collect()
}

#[rocket::get("/<path>")]
fn handler(path: &str) -> Option<Json<serde_json::Value>> {
    if path.len() < 2 || !path.starts_with('@') {
        return None;
    }
    let domain = "example.com";
    let username = &path[1..];
    let user_uri = format!("https://{domain}/users/{username}");
    Some(Json(serde_json::json!({
        "@context": [
            "https://www.w3.org/ns/activitystreams",
            "https://w3id.org/security/v1"
        ],
        "id": format!("https://{domain}/{path}"),
        "type": "Person",
        "following": format!("{user_uri}/following"),
        "followers": format!("{user_uri}/followers"),
        "inbox": format!("{user_uri}/inbox"),
        "preferredUsername": username,
        "name": "Example user",
        "summary": "Activity Pub server example.",
        "url": format!("https://{domain}"),
        "manuallyApprovesFollowers": false,
        "discoverable": true,
        "published": "2000-01-01T00:00:00Z"
    })))
}
