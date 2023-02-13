mod followers;
mod following;
mod inbox;
mod outbox;
mod statuses;

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![
        followers::handler,
        following::handler,
        inbox::handler,
        outbox::handler,
        statuses::handler,
    ]
}
