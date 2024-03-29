// copied from https://github.com/Plume-org/Plume/blob/main/plume-models/src/headers.rs
use rocket::http::{Header, HeaderMap};
use rocket::request::{FromRequest, Outcome, Request};

pub struct Headers<'a>(pub HeaderMap<'a>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Headers<'r> {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let mut headers = HeaderMap::new();
        for header in request.headers().clone().into_iter() {
            headers.add(header);
        }
        let uri = request.uri();
        let uri = uri.query().map_or_else(
            || uri.path().as_str().to_owned(),
            |query| format!("{}?{}", uri.path(), query),
        );
        headers.add(Header::new(
            "(request-target)",
            format!("{} {}", request.method().as_str().to_lowercase(), uri),
        ));
        Outcome::Success(Headers(headers))
    }
}

#[must_use]
pub fn select(headers: &HeaderMap<'_>, query: &str) -> String {
    query
        .split_whitespace()
        .map(|header| (header, headers.get_one(header)))
        .map(|(header, value)| format!("{}: {}", header.to_lowercase(), value.unwrap_or("")))
        .collect::<Vec<_>>()
        .join("\n")
}
