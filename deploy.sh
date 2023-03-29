#!/bin/bash
S3_BUCKET=${1}
if [ -z "$S3_BUCKET" ]; then
	echo "Usage: deploy.sh <s3_bucket>"
	exit 1
fi

mkdir -p dist/web_service ||
	exit 1
mkdir -p dist/dynamodb_stream ||
	exit 1
if uname -a | grep x86_64; then
	docker-compose -f docker/build/docker-compose.yml up \
		--build \
		--exit-code-from amazonlinux2 ||
		exit 1
	cp target/release/rust_lambda dist/web_service/bootstrap
	cp target/release/rust_lambda dist/dynamodb_stream/bootstrap
else
	# Faster compilation on Mac Apple Silicon
	rustup target install x86_64-unknown-linux-musl &&
		TARGET_CC=x86_64-linux-musl-gcc \
			RUSTFLAGS="-C linker=x86_64-linux-musl-gcc" \
			cargo build \
			  --release \
			  --all-targets \
			  --target x86_64-unknown-linux-musl ||
		exit 1
	cp target/x86_64-unknown-linux-musl/release/rust_lambda dist/web_service/bootstrap
	cp target/x86_64-unknown-linux-musl/release/examples/dynamodb_stream dist/dynamodb_stream/bootstrap
fi
rm dist/web_service.zip
zip -jr dist/web_service.zip dist/web_service ||
	exit 1
rm dist/dynamodb_stream.zip
zip -jr dist/dynamodb_stream.zip dist/dynamodb_stream ||
	exit 1
if uname -a | grep Darwin; then
	WEB_SERVICE_CODE_OBJECT_KEY=$(md5 dist/web_service.zip | cut -d' ' -f4)
	DYNAMODB_STREAM_CODE_OBJECT_KEY=$(md5 dist/dynamodb_stream.zip | cut -d' ' -f4)
else
	WEB_SERVICE_CODE_OBJECT_KEY=$(md5sum dist/web_service.zip | cut -d' ' -f1)
	DYNAMODB_STREAM_CODE_OBJECT_KEY=$(md5sum dist/dynamodb_stream.zip | cut -d' ' -f1)
fi
aws s3 cp dist/web_service.zip "s3://$S3_BUCKET/$WEB_SERVICE_CODE_OBJECT_KEY" &&
aws s3 cp dist/dynamodb_stream.zip "s3://$S3_BUCKET/$DYNAMODB_STREAM_CODE_OBJECT_KEY" &&
	aws cloudformation deploy \
		--template-file cloudformation.yml \
		--stack-name "rust-lambda" \
		--parameter-overrides \
		  "CodeBucket=$S3_BUCKET" \
		  "DynamodbStreamLambdaCodeObjectKey=$WEB_SERVICE_CODE_OBJECT_KEY" \
		  "WebServiceCodeObjectKey=$DYNAMODB_STREAM_CODE_OBJECT_KEY" \
		--capabilities CAPABILITY_IAM
