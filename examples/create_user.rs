use openssl::rsa::Rsa;

const KEYSIZE: u32 = 4096;

#[tokio::main]
async fn main() {
    let keypair = Rsa::generate(KEYSIZE).unwrap();
    let preferred_username = "test_username";
    let partition = format!("users/{}", preferred_username);
    let values = serde_dynamo::to_item(rust_lambda::model::user::User {
        preferred_username: Some(preferred_username.to_owned()),
        private_key: Some(keypair.private_key_to_der().unwrap()),
        public_key: Some(keypair.public_key_to_der().unwrap()),
    })
    .unwrap();

    let db_client = rust_lambda::dynamodb::get_client().await;
    if let Ok(_) = std::env::var("LOCAL_DYNAMODB_URL") {
        rust_lambda::dynamodb::create_table_if_not_exists(&db_client).await;
    }

    rust_lambda::dynamodb::put_item(
        &db_client,
        rust_lambda::dynamodb::DEFAULT_TABLE_NAME,
        partition.as_str(),
        "user",
        values,
    )
    .await
    .unwrap();
}
