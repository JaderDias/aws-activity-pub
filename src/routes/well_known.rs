use rocket::serde::json::Json;

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![webfinger]
}

#[rocket::get("/.well-known/webfinger?<resource>")]
fn webfinger(resource: String) -> Option<Json<serde_json::Value>> {
    Some(Json(serde_json::json!({
      "subject": "acct:test_username@example.com",
      "links": [{
        "rel": "self",
        "type": "application/activity+json",
        "href": "https://example.com/test_username"
      }]
    })))
}
