use crate::activitypub::digest::Digest;
use crate::activitypub::sign::{self, SignatureValidity::Valid};
use crate::settings::Settings;
use core::fmt::Error;
use rocket::{
    request::{FromRequest, Outcome, Request},
    response::status::BadRequest,
};

pub struct CustomHeaders<'a>(pub rocket::http::HeaderMap<'a>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for CustomHeaders<'r> {
    type Error = Error;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let headers = request.headers().clone();
        Outcome::Success(Self(headers))
    }
}

#[rocket::post("/users/<username>/inbox", data = "<data>")]
pub async fn handler(
    username: &str,
    headers: CustomHeaders<'_>,
    data: rocket::serde::json::Json<serde_json::Value>,
    settings: &rocket::State<Settings>,
) -> Result<String, BadRequest<&'static str>> {
    let activity = data.into_inner();
    let actor_id = activity["actor"]
        .as_str()
        .ok_or(BadRequest(Some("Missing actor id for activity")))?;
    if let Some(actor) = crate::model::user::get(actor_id, settings).await {
        let digest_header = headers.0.get_one("digest").unwrap();
        if Valid == sign::verify_http_headers(&actor, &headers.0, &Digest(digest_header.to_owned()))
        {
            let partition = format!("users/{username}/followers");
            let values = serde_dynamo::to_item(&activity).unwrap();
            crate::dynamodb::put_item(
                &settings.db_client,
                &settings.table_name,
                partition.as_str(),
                actor_id,
                values,
            )
            .await
            .unwrap();
            return Ok(String::new());
        }

        return Err(BadRequest(Some("Invalid digest")));
    }

    Err(BadRequest(Some("Missing actor")))
}
