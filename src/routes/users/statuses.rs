use crate::activitypub::object::Object;
use rocket::serde::json::Json;

#[rocket::get("/users/<username>/statuses/<status_id>")]
pub async fn handler(
    username: &str,
    status_id: &str,
    settings: &rocket::State<crate::Settings>,
) -> Option<Json<Object>> {
    let partition = format!("users/{username}/statuses/{status_id}");
    let get_item_output = crate::dynamodb::get_item(
        &settings.db_client,
        &settings.table_name,
        partition.as_str(),
    )
    .await
    .unwrap();
    let item = get_item_output.item.unwrap();
    let object: Object = serde_dynamo::from_item(item).unwrap();
    Some(Json(object))
}
