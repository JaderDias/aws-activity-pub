#!/bin/bash
S3_BUCKET=${1}
if [ -z "$S3_BUCKET" ]; then
	echo "Usage: deploy.sh <s3_bucket>"
	exit 1
fi

function create_zip() {
  local dir=$1
  local zip_file=$2
  zip -jr "$zip_file" "$dir" \
    || exit 1
}

function get_md5sum() {
  local file=$1
  if uname -a | grep Darwin; then
    md5 "$file" | cut -d' ' -f4
  else
    md5sum "$file" | cut -d' ' -f1
  fi
}

function upload() {
  local dir=$1
  local zip_file="$dir.zip"
  create_zip "$dir" "$zip_file"
  local md5sum
  md5sum=$(get_md5sum "$zip_file")
  aws s3 cp "$zip_file" "s3://$S3_BUCKET/$md5sum" \
    || exit 1
  echo "$md5sum"
}

rm dist/*.zip
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
DYNAMODB_STREAM_CODE_OBJECT_KEY=$(upload dist/dynamodb_stream | tail -n1)
WEB_SERVICE_CODE_OBJECT_KEY=$(upload dist/web_service | tail -n1)
echo "DYNAMODB_STREAM_CODE_OBJECT_KEY $DYNAMODB_STREAM_CODE_OBJECT_KEY"
echo "WEB_SERVICE_CODE_OBJECT_KEY $WEB_SERVICE_CODE_OBJECT_KEY"
aws cloudformation deploy \
  --capabilities CAPABILITY_IAM \
  --parameter-overrides \
    "CodeBucket=$S3_BUCKET" \
    "DynamodbStreamCodeObjectKey=$DYNAMODB_STREAM_CODE_OBJECT_KEY" \
    "WebServiceCodeObjectKey=$WEB_SERVICE_CODE_OBJECT_KEY" \
  --stack-name "rust-lambda" \
  --template-file cloudformation.yml
