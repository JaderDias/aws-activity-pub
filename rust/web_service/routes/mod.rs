mod nodeinfo;
mod users;
mod well_known;

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![handler, nodeinfo::handler]
        .into_iter()
        .chain(users::routes().into_iter())
        .chain(well_known::routes().into_iter())
        .collect()
}

#[rocket::get("/<path>")]
async fn handler(
    path: &str,
    settings: &rocket::State<library::settings::Settings>,
) -> Option<users::UserResponse> {
    if path.len() < 2 || !path.starts_with('@') {
        return None;
    }
    let username = &path[1..];
    users::handler(username, settings).await
}
