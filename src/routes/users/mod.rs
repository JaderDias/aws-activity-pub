mod followers;
mod following;
mod inbox;
mod statuses;

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![
        followers::handler,
        following::handler,
        inbox::handler,
        statuses::handler,
    ]
}
