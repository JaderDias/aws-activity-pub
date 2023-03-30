#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        panic!("Usage: create_user username dynamodb_table_name")
    }
    let preferred_username = args[1].clone();
    let table_name = args[2].clone();
    let table_name = table_name.as_str();
    let db_client = rust_lambda::dynamodb::get_client().await;
    if let Ok(_url) = std::env::var("LOCAL_DYNAMODB_URL") {
        rust_lambda::dynamodb::create_table_if_not_exists(&db_client, table_name).await;
    }
    let _ =
        rust_lambda::model::user::create(&db_client, table_name, preferred_username.as_str()).await;
}
