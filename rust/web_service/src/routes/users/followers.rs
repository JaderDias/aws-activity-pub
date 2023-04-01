use rocket::http::ContentType;
use rocket::serde::json::Json;

#[derive(rocket::Responder)]
pub struct Followers(Json<serde_json::Value>, ContentType);

#[rocket::get("/users/<username>/followers")]
pub fn handler(username: &str, settings: &rocket::State<library::settings::Settings>) -> Followers {
    let user_uri = format!("{}/users/{username}", settings.base_url);
    let body = Json(serde_json::json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": format!("{user_uri}/followers"),
        "type": "OrderedCollection",
        "totalItems": 1, // TODO: real number from db
        "first": format!("{user_uri}/followers?page=1"),
    }));
    let content_type: ContentType =
        ContentType::new("application", "activity+json").with_params([("charset", "utf-8")]);
    Followers(body, content_type)
}
