use super::context::StringOrContext;
use serde::{Deserialize, Serialize};

// TODO: use activitystreams = "0.6.2"

#[derive(Serialize, Deserialize)]
pub struct Object {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment: Option<Vec<Attachment>>,
    #[serde(rename = "atomUri")]
    pub atom_uri: Option<String>,
    #[serde(rename = "attributedTo")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributed_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(rename = "@context")]
    #[serde(skip_deserializing)]
    pub context: Option<Vec<StringOrContext>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "inReplyTo")]
    pub in_reply_to: Option<String>,
    #[serde(rename = "inReplyToAtomUri")]
    pub in_reply_to_atom_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<String>,
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensitive: Option<bool>,
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
