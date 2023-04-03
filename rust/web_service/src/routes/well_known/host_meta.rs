use rocket::http::ContentType;

#[derive(rocket::Responder)]
pub struct HostMeta(String, ContentType);

#[rocket::get("/.well-known/host-meta")]
pub fn handler(settings: &rocket::State<library::settings::Settings>) -> HostMeta {
    let body = format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<XRD xmlns=\"http://docs.oasis-open.org/ns/xri/xrd-1.0\">\n  <Link rel=\"lrdd\" template=\"{}/.well-known/webfinger?resource={{uri}}\"/>\n</XRD>\n", settings.base_url);
    let content_type =
        ContentType::new("application", "xrd+xml").with_params([("charset", "utf-8")]);
    HostMeta(body, content_type)
}
