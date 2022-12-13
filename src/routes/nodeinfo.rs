use rocket::http::ContentType;

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![nodeinfo]
}

#[derive(rocket::Responder)]
pub enum NodeInfo {
    A(String, ContentType),
}

#[rocket::get("/nodeinfo/2.0")]
pub fn nodeinfo() -> NodeInfo {
    let doc = serde_json::json!({
        "version": 2.0,
        "software": {
            "name": "rust_lambda",
            "version": 1 // TODO: add version
        },
        "protocols": ["activitypub"],
        "services": {"inbound": [], "outbound": []},
        "openRegistrations": true,
        "usage": {
            "users": {"total": 1 }, // TODO: count users
            "localPosts": 1, // TODO: count posts
        }
    });
    let content_type = ContentType::JSON.with_params((
        "profile",
        "http://nodeinfo.diaspora.software/ns/schema/2.0#,",
    ));

    NodeInfo::A(doc.to_string(), content_type)
}
