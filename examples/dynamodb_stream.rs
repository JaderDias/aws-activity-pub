use aws_lambda_events_extended::dynamodb::DynamoDBEvent;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let (event, _context) = event.into_parts();
    let json_value = serde_json::to_value(event)?;

    let dynamodb_event: DynamoDBEvent = serde_json::from_value(json_value)?;

    for record in dynamodb_event.records {
        if let Some(new_image) = record.dynamodb.new_image {
            let id = new_image.get("id").and_then(|v| v.s.clone());
            println!("New item with ID: {:?}", id);
            // process the new item as needed
        }
    }

    Ok(json!({ "message": "Success" }))
}
