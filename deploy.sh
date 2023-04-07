#!/bin/bash
S3_BUCKET=${1}
if [ -z "$S3_BUCKET" ]; then
	echo "Usage: deploy.sh <s3_bucket>"
	exit 1
fi

mkdir -p dist/dynamodb_stream
mkdir -p dist/web_service
if uname -a | grep x86_64; then
	docker-compose -f docker/build/docker-compose.yml up \
		--build \
		--exit-code-from amazonlinux2 ||
		exit 1
	cp target/release/web_service dist/web_service/bootstrap
	cp target/release/dynamodb_stream dist/dynamodb_stream/bootstrap
else
	# Faster compilation on Mac Apple Silicon
	rustup target install x86_64-unknown-linux-musl &&
		TARGET_CC=x86_64-linux-musl-gcc \
			RUSTFLAGS="-C linker=x86_64-linux-musl-gcc" \
			cargo build \
			  --release \
			  --workspace \
			  --target x86_64-unknown-linux-musl ||
		exit 1
	cp target/x86_64-unknown-linux-musl/release/web_service dist/web_service/bootstrap
	cp target/x86_64-unknown-linux-musl/release/dynamodb_stream dist/dynamodb_stream/bootstrap
fi
aws cloudformation package \
    --template-file cloudformation.yml \
    --s3-bucket "$S3_BUCKET" \
    --output-template-file packaged.yml ||
    exit 1
aws cloudformation deploy \
  --capabilities CAPABILITY_IAM \
  --stack-name "rust-lambda" \
  --template-file packaged.yml
