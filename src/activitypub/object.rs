use serde::{Deserialize, Serialize};
use std::cmp::{Eq, PartialEq};
use std::fmt::Debug;

// TODO: use activitystreams = "0.6.2"

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
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
    pub devices: Option<String>,
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
    pub outbox: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "preferredUsername")]
    pub preferred_username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "publicKey")]
    pub public_key: Option<PublicKey>,
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
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Attachment {
    pub r#type: String,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub url: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct PublicKey {
    pub id: String,
    pub owner: String,
    #[serde(rename = "publicKeyPem")]
    pub public_key_pem: String,
}
