#!/usr/bin/env bash

#NONCE=`curl http://localhost:8080/get_nonce`

. ./.env

timestamp=$(date +%s)
export NONCE=$((timestamp / 30))

echo "Key: $SECRET_KEY"
echo "Nonce: $NONCE"
# echo "NONCE: $NONCE"

SIGNATURE=`echo -n $NONCE | openssl dgst -sha1 -hmac "$SECRET_KEY"`

echo "Signature: $SIGNATURE"

curl -X POST http://localhost:8080/image \
    -H "Content-Type: multipart/form-data" \
    -H "X-Signature: $SIGNATURE" \
    -H "X-Nonce: $NONCE" \
    -H "X-Dir-Index: 2" \
    -F file=@./IMG_9211.jpg

