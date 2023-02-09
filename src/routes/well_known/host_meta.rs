use rocket::http::ContentType;

#[derive(rocket::Responder)]
pub enum HostMeta {
    A(String, ContentType),
}

#[rocket::get("/.well-known/host-meta")]
pub fn handler(settings: &rocket::State<crate::Settings>) -> HostMeta {
    let body = format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<XRD xmlns=\"http://docs.oasis-open.org/ns/xri/xrd-1.0\">\n  <Link rel=\"lrdd\" template=\"https://{}/.well-known/webfinger?resource={{uri}}\" type=\"application/xrd+xml\" />\n</XRD>", settings.domain_name);
    let content_type = ContentType::new("application", "xrd+xml");
    HostMeta::A(body, content_type)
}
