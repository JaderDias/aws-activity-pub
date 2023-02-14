use serde::{Deserialize, Serialize};

// TODO: use activitystreams = "0.6.2"

#[derive(Serialize, Deserialize)]
pub struct Object {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "atomUri")]
    pub atom_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment: Option<Vec<Attachment>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "attributedTo")]
    pub attributed_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(rename = "@context")]
    pub context: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discoverable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub followers: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub following: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "inReplyTo")]
    pub in_reply_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "inReplyToAtomUri")]
    pub in_reply_to_atom_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inbox: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "manuallyApprovesFollowers")]
    pub manually_approves_followers: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "preferredUsername")]
    pub preferred_username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "serde_bytes")]
    pub private_key: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "serde_bytes")]
    pub public_key: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<String>,
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensitive: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Attachment {
    pub r#type: String,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub url: String,
    pub name: String,
}
