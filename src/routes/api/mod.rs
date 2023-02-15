mod statuses;

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![statuses::handler,]
}
