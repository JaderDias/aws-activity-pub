use lambda_web::{is_running_on_lambda, launch_rocket_on_lambda, LambdaError};
use library::settings::Settings;
use std::env::var;

mod routes;

#[rocket::main]
async fn main() -> Result<(), LambdaError> {
    library::trace::init();

    let domain_name = var("CUSTOM_DOMAIN").unwrap();
    let rocket = rocket::build()
        .mount("/", routes::routes())
        .manage(Settings {
            base_url: format!("{}://{domain_name}", var("PROTOCOL").unwrap()),
            db_client: library::dynamodb::get_client().await,
            domain_name,
            node_id: library::faas_snowflake_id::get_node_id(),
            table_name: var("DYNAMODB_TABLE").unwrap(),
        });

    if is_running_on_lambda() {
        // Launch on AWS Lambda
        return launch_rocket_on_lambda(rocket).await;
    }

    // Launch local server
    let _ = rocket.launch().await?;
    Ok(())
}
