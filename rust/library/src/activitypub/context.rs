use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct FocalPoint {
    #[serde(rename = "@container")]
    pub _container: String,
    #[serde(rename = "@id")]
    pub _id: String,
}

#[derive(Deserialize, Serialize)]
pub struct Context {
    pub ostatus: String,
    #[serde(rename = "atomUri")]
    pub atom_uri: String,
    #[serde(rename = "inReplyToAtomUri")]
    pub in_reply_to_atom_uri: String,
    pub conversation: String,
    pub sensitive: String,
    pub toot: String,
    #[serde(rename = "votersCount")]
    pub voters_count: String,
    pub blurhash: String,
    #[serde(rename = "focalPoint")]
    pub focal_point: FocalPoint,
    #[serde(rename = "Hashtag")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hashtag: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum Extension {
    String(String),
    Context(Box<Context>),
}

#[must_use]
pub fn default() -> Option<serde_json::Value> {
    serde_json::from_str(
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
    .ok()
}
