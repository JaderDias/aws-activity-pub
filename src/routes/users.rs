use crate::activitypub::object::Object;
use crate::dynamodb::DbSettings;
use rocket::serde::json::Json;

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![user_statuses]
}

#[rocket::get("/users/<_username>/statuses/<status_id>")]
pub async fn user_statuses(
    _username: String,
    status_id: String,
    db_settings: &rocket::State<DbSettings>,
) -> Option<Json<Object>> {
    let get_item_output = crate::dynamodb::get_item(
        &db_settings.client,
        &db_settings.table_name,
        status_id.as_str(),
    )
    .await
    .unwrap();
    let item = get_item_output.item.unwrap();
    Some(Json(serde_dynamo::from_item(item).ok()?))
}
