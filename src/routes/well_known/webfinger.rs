use rocket::serde::json::Json;

#[rocket::get("/.well-known/webfinger?<resource>")]
pub fn handler(resource: &str) -> Option<Json<serde_json::Value>> {
    let split = resource.split(':').collect::<Vec<&str>>();
    if split[0] != "acct" || split.len() < 2 {
        return None;
    }
    let sub_split = split[1].split('@').collect::<Vec<&str>>();
    if sub_split.len() != 2 {
        return None;
    }
    let username = sub_split[0];
    let domain = sub_split[1];
    Some(Json(serde_json::json!({
      "subject": resource,
      "links": [{
        "rel": "self",
        "type": "application/activity+json",
        "href": format!("https://{domain}/@{username}")
      }]
    })))
}
