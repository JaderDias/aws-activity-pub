use crate::activitypub::object::Object;
use crate::dynamodb::DbSettings;
use rocket::serde::json::Json;

#[rocket::get("/users/<username>/statuses/<status_id>")]
pub async fn handler(
    username: &str,
    status_id: &str,
    db_settings: &rocket::State<DbSettings>,
) -> Option<Json<Object>> {
    let partition = format!("users/{username}/statuses/{status_id}");
    let get_item_output = crate::dynamodb::get_item(
        &db_settings.client,
        &db_settings.table_name,
        partition.as_str(),
    )
    .await
    .unwrap();
    let item = get_item_output.item.unwrap();
    let mut object: Object = serde_dynamo::from_item(item).unwrap();
    object.context = serde_json::from_str(object.context.as_str().unwrap()).unwrap();
    Some(Json(object))
}
