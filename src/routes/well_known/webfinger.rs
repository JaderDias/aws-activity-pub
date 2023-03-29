use crate::model::user;
use rocket::serde::json::Json;
use tracing::{event, Level};

#[rocket::get("/.well-known/webfinger?<resource>")]
pub async fn handler(
    resource: &str,
    settings: &rocket::State<crate::settings::Settings>,
) -> Option<Json<serde_json::Value>> {
    let split = resource.splitn(2, ':').collect::<Vec<&str>>();
    event!(Level::DEBUG, "{:?}", split);
    if split[0] != "acct" || split.len() < 2 {
        return None;
    }
    let sub_split = split[1].split('@').collect::<Vec<&str>>();
    if sub_split.len() != 2 {
        return None;
    }
    let username = sub_split[0];
    let domain = sub_split[1];
    if domain != settings.domain_name {
        return None;
    }

    if let Some(_) = user::get(username, settings).await {
        return Some(Json(serde_json::json!({
          "subject": resource,
          "links": [{
            "rel": "self",
            "type": "application/activity+json",
            "href": format!("{}/users/{username}", settings.base_url)
          }]
        })));
    }

    None
}
