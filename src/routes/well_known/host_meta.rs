use rocket::http::ContentType;

#[derive(rocket::Responder)]
pub struct HostMeta(String, ContentType);

#[rocket::get("/.well-known/host-meta")]
pub fn handler(settings: &rocket::State<crate::settings::Settings>) -> HostMeta {
    let body = format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<XRD xmlns=\"http://docs.oasis-open.org/ns/xri/xrd-1.0\">\n  <Link rel=\"lrdd\" template=\"{}/.well-known/webfinger?resource={{uri}}\" type=\"application/xrd+xml\" />\n</XRD>", settings.base_url);
    let content_type = ContentType::new("application", "xrd+xml");
    HostMeta(body, content_type)
}
