Rantang
===============

Rantang is simple image uploader service with signature verification.

## Example

```
curl -X POST http://localhost:8080/image \
    -H "Content-Type: multipart/form-data" \
    -H "X-Signature:123" \
    -H "X-Nonce:1687508024" \
    -d @/Users/robin/Downloads/IMG_9211.jpg
```
