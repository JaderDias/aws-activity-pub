#[tokio::main]
async fn main() {
    let preferred_username = "test_username";
    rust_lambda::dynamodb::create_user(preferred_username).await;
}
