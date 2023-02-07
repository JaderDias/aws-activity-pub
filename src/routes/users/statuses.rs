use crate::activitypub::object::Object;
use crate::dynamodb::DbSettings;
use rocket::serde::json::Json;

#[rocket::get("/users/<_username>/statuses/<status_id>")]
pub async fn handler(
    _username: &str,
    status_id: &str,
    db_settings: &rocket::State<DbSettings>,
) -> Option<Json<Object>> {
    let get_item_output =
        crate::dynamodb::get_item(&db_settings.client, &db_settings.table_name, status_id)
            .await
            .unwrap();
    let item = get_item_output.item.unwrap();
    Some(Json(serde_dynamo::from_item(item).ok()?))
}
