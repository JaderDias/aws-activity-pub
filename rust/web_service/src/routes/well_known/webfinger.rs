use library::model::user;
use rocket::http::ContentType;
use rocket::serde::json::Json;
use tracing::{event, Level};

#[derive(rocket::Responder)]
pub struct Webfinger(Json<serde_json::Value>, ContentType);

#[rocket::get("/.well-known/webfinger?<resource>")]
pub async fn handler(
    resource: &str,
    settings: &rocket::State<library::settings::Settings>,
) -> Option<Webfinger> {
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

    if (user::get(username, settings).await).is_some() {
        let body = Json(serde_json::json!({
          "subject": resource,
          "links": [{
            "rel": "self",
            "type": "application/activity+json",
            "href": format!("{}/users/{username}", settings.base_url)
          }]
        }));
        let content_type =
            ContentType::new("application", "jrd+json").with_params([("charset", "utf-8")]);
        return Some(Webfinger(body, content_type));
    }

    None
}
