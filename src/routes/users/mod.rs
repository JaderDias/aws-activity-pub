mod followers;
mod following;
mod statuses;

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![statuses::handler, followers::handler, following::handler,]
}
