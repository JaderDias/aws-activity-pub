use rocket::http::ContentType;

#[derive(rocket::Responder)]
pub struct NodeInfo(String, ContentType);

#[rocket::get("/nodeinfo/2.0")]
pub fn handler() -> NodeInfo {
    let doc = serde_json::json!({
        "version": 2.0,
        "software": {
            "name": "aws_activity_pub",
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

    NodeInfo(doc.to_string(), content_type)
}
