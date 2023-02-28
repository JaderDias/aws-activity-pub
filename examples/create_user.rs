#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let preferred_username = args[1].clone();
    let db_client = rust_lambda::dynamodb::get_client().await;
    if let Ok(_url) = std::env::var("LOCAL_DYNAMODB_URL") {
        rust_lambda::dynamodb::create_table_if_not_exists(&db_client).await;
    }
    let _ = rust_lambda::model::user::create(
        db_client,
        rust_lambda::dynamodb::DEFAULT_TABLE_NAME,
        preferred_username.as_str(),
    )
    .await;
}
