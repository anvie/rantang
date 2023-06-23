#!/usr/bin/env bash

#NONCE=`curl http://localhost:8080/get_nonce`

. ./.env

timestamp=$(date +%s)
NONCE=$((timestamp / 30))

echo "Key: $SECRET_KEY"
echo "NONCE: $NONCE"

SIGNATURE=`echo -n "$NONCE" | openssl dgst -sha1 -hmac "$SECRET_KEY"`

curl -X POST http://localhost:8080/image \
    -H "Content-Type: multipart/form-data" \
    -H "X-Signature: $SIGNATURE" \
    -H "X-Nonce: $NONCE" \
    -F file=@./IMG_9211.jpg
