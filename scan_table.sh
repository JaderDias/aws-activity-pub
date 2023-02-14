#!/bin/bash
if ! grep -F '[profile localhost]' < ~/.aws/config; then
  echo -e "[profile localhost]\nregion = us-east-1" >> ~/.aws/config
fi
if ! grep -F '[localhost]' <  ~/.aws/credentials; then
  echo -e "[localhost]\naws_access_key_id = NOT_NEEDED\naws_secret_access_key = NOT_NEEDED" >> ~/.aws/credentials
fi
aws dynamodb scan --table-name table_name --endpoint-url http://localhost:8000 --profile localhost
