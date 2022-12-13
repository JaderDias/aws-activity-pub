#!/bin/bash
S3_BUCKET=${1}
if [ -z "$S3_BUCKET" ]
then
    echo "Usage: deploy.sh <s3_bucket>"
    exit 1
fi

if uname -a | grep x86_64; then
  docker-compose -f docker/build/docker-compose.yml up \
    --exit-code-from amazonlinux2 \
  || exit 1
  cp target/release/rust_lambda dist/amazonlinux2/bootstrap
else
  # Faster compilation on Mac Apple Silicon
  rustup target install x86_64-unknown-linux-musl \
  && TARGET_CC=x86_64-linux-musl-gcc \
  RUSTFLAGS="-C linker=x86_64-linux-musl-gcc" \
  cargo build --release --target x86_64-unknown-linux-musl \
  || exit 1
  cp target/x86_64-unknown-linux-musl/release/rust_lambda dist/amazonlinux2/bootstrap
fi
rm dist/amazonlinux2.zip \
&& zip -jr dist/amazonlinux2.zip dist/amazonlinux2 \
&& LAMBDA_CODE_OBJECT_KEY=`md5sum dist/amazonlinux2.zip | cut -d' ' -f1` \
&& aws s3 cp dist/amazonlinux2.zip "s3://$S3_BUCKET/$LAMBDA_CODE_OBJECT_KEY" \
&& aws cloudformation deploy \
  --template-file cloudformation.yml \
  --stack-name "rust-lambda" \
  --parameter-overrides "LambdaCodeBucket=$S3_BUCKET" "LambdaCodeObjectKey=$LAMBDA_CODE_OBJECT_KEY" \
  --capabilities CAPABILITY_IAM
