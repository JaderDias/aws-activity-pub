mod statuses;
mod users;

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![statuses::handler, users::handler,]
}
