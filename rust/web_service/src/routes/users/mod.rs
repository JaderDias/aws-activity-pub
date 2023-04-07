use library::activitypub::object::{Object, PublicKey};
use library::rsa;
use rocket::http::ContentType;

mod followers;
mod following;
mod inbox;
mod outbox;
mod statuses;

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![
        handler,
        followers::handler,
        following::handler,
        inbox::handler,
        outbox::handler,
        outbox::page,
        statuses::handler,
    ]
}

#[derive(rocket::Responder)]
pub struct UserResponse(String, ContentType);

#[rocket::get("/users/<username>")]
pub async fn handler(
    username: &str,
    settings: &rocket::State<library::settings::Settings>,
) -> Option<UserResponse> {
    if let Some(user) = library::model::user::get(username, settings).await {
        let public_key = rsa::der_to_pem(user.public_key.as_ref().unwrap());
        let user_uri = format!("{}/users/{username}", settings.base_url);
        let content_type =
            ContentType::new("application", "activity+json").with_params(("charset", "utf-8"));
        let body = serde_json::json!(Object {
            actor: None,
            atom_uri: None,
            attachment: Some(Vec::new()),
            attributed_to: None,
            cc: None,
            content: None,
            context: library::activitypub::context::default(),
            conversation: None,
            devices: Some(format!("{user_uri}/collections/devices")),
            discoverable: Some(false),
            followers: Some(format!("{user_uri}/followers")),
            following: Some(format!("{user_uri}/following")),
            id: Some(user_uri.clone()),
            in_reply_to: None,
            in_reply_to_atom_uri: None,
            inbox: Some(format!("{user_uri}/inbox")),
            manually_approves_followers: Some(false),
            name: Some(username.to_owned()),
            object: None,
            outbox: Some(format!("{user_uri}/outbox")),
            partition_key: None,
            preferred_username: Some(username.to_owned()),
            public_key: Some(PublicKey {
                id: format!("{user_uri}#main-key"),
                owner: user_uri,
                public_key_pem: public_key,
            }),
            published: Some(user.get_published_time()),
            r#type: Some("Person".to_owned()),
            sensitive: None,
            sort_key: None,
            summary: Some(String::new()),
            tag: Some(Vec::new()),
            to: None,
            url: Some(format!("{}/@{username}", settings.base_url)),
            extra: serde_json::Value::Null,
        });
        return Some(UserResponse(body.to_string(), content_type));
    }

    None
}
