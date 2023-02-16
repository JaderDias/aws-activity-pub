pub struct Settings {
    pub db_client: aws_sdk_dynamodb::Client,
    pub domain_name: String,
    pub node_id: u64,
    pub table_name: String,
}
