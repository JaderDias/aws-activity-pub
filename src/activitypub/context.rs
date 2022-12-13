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
pub enum StringOrContext {
    String(String),
    Context(Box<Context>),
}
