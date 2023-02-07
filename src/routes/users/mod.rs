mod followers;
mod statuses;

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![statuses::handler, followers::handler]
}
