mod host_meta;
mod webfinger;

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![host_meta::handler, webfinger::handler]
}
