Rantang
===============

Rantang is a simple image upload server with signature verification built using Rust with Actix. This software allows an authenticated user to upload images to the server with a digital signature.

## Installation

To install Rantang, you need to have Rust and Cargo installed. You can install them from the official website: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

1. Clone this repository.
2. Open a terminal window and navigate to the project directory.
3. Run the following command to start the server:

```bash
cargo run
```

The server should now be running on `http://localhost:8080`.

## Usage

Rantang has a single API endpoint:

### `POST /image`

This endpoint allows a user to upload an image to the server. The request must contain the following headers:

```http
Content-Type: multipart/form-data
X-Signature: HMAC-SHA1(secret + nonce)
X-Nonce: nonce
```

- `secret` is the secret key used to sign the signature. It must be the same as the `SECRET_KEY` environment variable.
- `nonce` is timestamp divided by 30 seconds.

The request must contain a `file` parameter that contains the image file to be uploaded.

The response contains the following fields:

- `nonce` is the nonce used to sign the signature, sent by client in X-Nonce header.
- `sha1` is the SHA1 hash of the uploaded image.

## Example

Curl command to upload an image:

```
curl -X POST http://localhost:8080/image \
    -H "Content-Type: multipart/form-data" \
    -H "X-Signature: $SIGNATURE" \
    -H "X-Nonce: $NONCE" \
    -F file=@./IMG_9211.jpg
```

Example response:

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "nonce": "56250429",
  "sha1": "e1586b201c06a2d440358378f15d6a7987ee4ab6"
}
```

Please take a look at the `test_upload.sh` script for example usage in bash.

### Custom out dir

You can specify the additional output directory by setting the `OUT_DIR_[index]` environment variable. 
Where `[index]` is the index of the output directory, can be any number or string.

Example:

```
OUT_DIR_2=/home/user/images_2
```

To upload to specific output directory defined in `OUT_DIR_2` environment variable, you can add `X-Dir-Index` header with value `2` to the request, for example:

```bash
curl -X POST http://localhost:8080/image \
    -H "Content-Type: multipart/form-data" \
    -H "X-Signature: $SIGNATURE" \
    -H "X-Nonce: $NONCE" \
    -H "X-Dir-Index: 2" \
    -F file=@./IMG_9211.jpg
```

Then the uploaded image will be saved to `/home/user/images_2` directory.

## License

Rantang is licensed under the MIT License. See the `LICENSE` file for more information.

## Contributions

Contributions to Rantang are welcomed. Please open an issue or submit a pull request.
