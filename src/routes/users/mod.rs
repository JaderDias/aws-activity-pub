use crate::activitypub::object::{Object, PublicKey};
use crate::rsa;
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
        statuses::handler,
    ]
}

#[derive(rocket::Responder)]
pub enum UserResponse {
    A(String, ContentType),
}

#[rocket::get("/users/<username>")]
pub async fn handler(
    username: &str,
    settings: &rocket::State<crate::settings::Settings>,
) -> Option<UserResponse> {
    let domain = settings.domain_name.as_str();
    if let Some(user) = crate::model::user::get(username, settings).await {
        let public_key = rsa::der_to_pem(user.public_key.as_ref().unwrap());
        let user_uri = format!("https://{domain}/users/{username}");
        let context: serde_json::Value = serde_json::from_str(
            r#"[
            "https://www.w3.org/ns/activitystreams",
            "https://w3id.org/security/v1",
            {
                "manuallyApprovesFollowers":"as:manuallyApprovesFollowers",
                "toot":"http://joinmastodon.org/ns#",
                "featured":{"@id":"toot:featured",
                "@type":"@id"},
                "featuredTags":{"@id":"toot:featuredTags",
                "@type":"@id"},
                "alsoKnownAs":{"@id":"as:alsoKnownAs",
                "@type":"@id"},
                "movedTo":{"@id":"as:movedTo",
                "@type":"@id"},
                "schema":"http://schema.org#",
                "PropertyValue":"schema:PropertyValue",
                "value":"schema:value",
                "discoverable":"toot:discoverable",
                "Device":"toot:Device",
                "Ed25519Signature":"toot:Ed25519Signature",
                "Ed25519Key":"toot:Ed25519Key",
                "Curve25519Key":"toot:Curve25519Key",
                "EncryptedMessage":"toot:EncryptedMessage",
                "publicKeyBase64":"toot:publicKeyBase64",
                "deviceId":"toot:deviceId",
                "claim":{"@type":"@id",
                "@id":"toot:claim"},
                "fingerprintKey":{"@type":"@id",
                "@id":"toot:fingerprintKey"},
                "identityKey":{"@type":"@id",
                "@id":"toot:identityKey"},
                "devices":{"@type":"@id",
                "@id":"toot:devices"},
                "messageFranking":"toot:messageFranking",
                "messageType":"toot:messageType",
                "cipherText":"toot:cipherText",
                "suspended":"toot:suspended"
            }
        ]"#,
        )
        .unwrap();
        let content_type =
            ContentType::new("application", "activity+json").with_params(("charset", "utf-8"));
        let body = serde_json::json!(Object {
            actor: None,
            atom_uri: None,
            attachment: Some(Vec::new()),
            attributed_to: None,
            cc: None,
            content: None,
            context,
            conversation: None,
            devices: Some(format!("{user_uri}/collections/devices")),
            discoverable: Some(false),
            followers: Some(format!("{user_uri}/followers")),
            following: Some(format!("{user_uri}/following")),
            id: Some(format!("{user_uri}")),
            in_reply_to: None,
            in_reply_to_atom_uri: None,
            inbox: Some(format!("{user_uri}/inbox")),
            manually_approves_followers: Some(false),
            name: Some(username.to_owned()),
            object: None,
            outbox: Some(format!("{user_uri}/outbox")),
            preferred_username: Some(username.to_owned()),
            public_key: Some(PublicKey {
                id: format!("{user_uri}#main-key"),
                owner: user_uri,
                public_key_pem: public_key,
            }),
            published: Some(user.get_published_time()),
            r#type: "Person".to_owned(),
            sensitive: None,
            summary: Some(String::new()),
            tag: Some(Vec::new()),
            to: None,
            url: Some(format!("https://{domain}/@{username}")),
            extra: serde_json::Value::Null,
        });
        return Some(UserResponse::A(body.to_string(), content_type));
    }

    None
}
