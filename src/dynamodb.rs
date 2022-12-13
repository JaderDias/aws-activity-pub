use aws_sdk_dynamodb::error::{GetItemError, PutItemError};
use aws_sdk_dynamodb::model::AttributeValue;
use aws_sdk_dynamodb::types::SdkError;
use aws_sdk_dynamodb::Client;

pub const PARTITION_KEY_NAME: &str = "partition";

pub type GetItemResult = Result<aws_sdk_dynamodb::output::GetItemOutput, SdkError<GetItemError>>;
pub type PutItemResult = Result<aws_sdk_dynamodb::output::PutItemOutput, SdkError<PutItemError>>;

pub struct DbSettings {
    pub client: Client,
    pub table_name: String,
}

/// # Errors
///
/// Will return `Err` if a connection to the database is no properly established.
pub async fn get_item(
    client: &Client,
    dynamodb_table_name: &str,
    partition: &str,
) -> GetItemResult {
    let value = AttributeValue::S(partition.to_owned());
    client
        .get_item()
        .table_name(dynamodb_table_name)
        .key(PARTITION_KEY_NAME, value)
        .send()
        .await
}

/// # Errors
///
/// Will return `Err` if a connection to the database is no properly established.
pub async fn put_item<S: std::hash::BuildHasher>(
    client: &Client,
    dynamodb_table_name: &str,
    partition: &str,
    values: std::collections::HashMap<String, AttributeValue, S>,
) -> PutItemResult {
    let value = AttributeValue::S(partition.to_owned());
    let mut table = client
        .put_item()
        .table_name(dynamodb_table_name)
        .item(PARTITION_KEY_NAME, value);
    for (key, value) in values {
        table = table.item(key, value);
    }
    table.send().await
}

#[must_use]
pub fn get_uuid() -> String {
    let id = uuid::Uuid::new_v4();
    let mut buffer: [u8; 32] = [b'!'; 32];
    String::from(id.simple().encode_lower(&mut buffer))
}

pub async fn get_client() -> Client {
    if let Ok(url) = std::env::var("LOCAL_DYNAMODB_URL") {
        println!("Using local dynamodb at {url}");
        get_local_client(url).await
    } else {
        let config = aws_config::load_from_env().await;
        Client::new(&config)
    }
}

pub async fn get_local_client(local_dynamodb_url: String) -> Client {
    std::env::set_var("AWS_ACCESS_KEY_ID", "DUMMYIDEXAMPLE");
    std::env::set_var("AWS_SECRET_ACCESS_KEY", "DUMMYEXAMPLEKEY");
    let config = aws_config::from_env().region("us-east-1").load().await;
    let dynamodb_local_config = aws_sdk_dynamodb::config::Builder::from(&config)
        .endpoint_url(local_dynamodb_url)
        .build();
    Client::from_conf(dynamodb_local_config)
}
