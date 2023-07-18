use actix_cors::Cors;
use actix_error::ErrorBadRequest;
/// MIT License
///
/// Copyright (c) 2023 Robin Syihab <r@nu.id>
///
/// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"),
/// to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense,
/// and/or sell copies of the Software and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
///
/// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
///
/// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES
/// OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS
/// BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF
/// OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

/// Project Rantang
///
/// Author: Robin Syihab <r@nu.id>
///
use actix_multipart::Multipart;
use actix_web::{
    error as actix_error, middleware, web, App, HttpRequest, HttpResponse, HttpServer,
};
use anyhow::Result;
use dotenvy::dotenv;
use error::{to_str_err, ApiResult, MyError};
use futures::{StreamExt, TryStreamExt};
use image::ImageFormat;
use log::debug;
use serde_json::json;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, io};

use crate::error::img_error;

mod error;
#[cfg(test)]
mod tests;

mod crypto;
mod nonce;

/// Gets the header value from an [`HttpRequest`](HttpRequest) object.
///
/// # Arguments
///
/// * `key` - A string slice with the name of the header.
/// * `req` - An [`HttpRequest`](HttpRequest) reference.
///
/// # Errors
///
/// If the `key` is not present in the `req`'s headers, or if the value of the
/// header
/// is an empty string, an [`ErrorBadRequest`](ErrorBadRequest) is returned.
///
/// # Examples
///
/// ```
/// let req = HttpRequest::new(Method::GET, "localhost".parse().unwrap(),
/// false);
/// let res = get_header_value("key", &req);
/// assert!(res.is_err());
/// ```
pub(crate) fn get_header_value<'a>(key: &str, req: &'a HttpRequest) -> Result<&'a str, MyError> {
    let value = req
        .headers()
        .get(key)
        .ok_or_else(|| ErrorBadRequest(format!("No {} header", key)))?
        .to_str()
        .map_err(to_str_err)?;

    if value.is_empty() {
        return Err(ErrorBadRequest(format!("{} header is empty", key)).into());
    }

    Ok(value)
}

/// Verifies a signature against a range of nonces using a secret key.
///
/// # Arguments
///
/// * `secret_key` - A string slice that holds the secret key.
/// * `nonce` - A slice of 64 bit unsigned integers.
/// * `signature` - A string slice that holds the signature to verify.
///
/// # Examples
///
/// ```
/// let secret_key = "mysecretkey";
/// let nonce = [1, 2, 3];
/// let signature = "signhere";
/// assert!(verify_signature_nonce_range(secret_key, nonce, signature));
/// ```
fn verify_signature_nonce_range(secret_key: &str, nonce: &[u64], signature: &str) -> bool {
    let mut result = false;
    for n in nonce {
        if crypto::verify_signature(
            secret_key.as_bytes(),
            (*n).to_string().as_bytes(),
            signature,
        ) {
            result = true;
            break;
        }
    }
    result
}

async fn save_file(req: HttpRequest, mut payload: Multipart) -> ApiResult {
    let mut save_result: Result<(), io::Error> = Ok(());
    let max_size = 20 * 1024 * 1024; // 20mb

    let mut out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let secret_key = env::var("SECRET_KEY").expect("SECRET_KEY not set");

    let signature = get_header_value("X-Signature", &req)?.trim();
    debug!("[client] signature: {}", signature);
    let nonce_from_client = get_header_value("X-Nonce", &req)?.trim();
    let nonce = nonce::nonce();

    let calculated_signature =
        crypto::sign_message(secret_key.as_bytes(), &nonce.to_string().as_bytes());
    debug!("[server] signature: {}", calculated_signature);

    debug!(
        "NONCE: client <> server - {} <> {}",
        nonce_from_client, nonce
    );

    let nonce_range: [u64; 3] = [nonce - 1, nonce, nonce + 1];

    if !verify_signature_nonce_range(&secret_key, &nonce_range, signature) {
        return Err(ErrorBadRequest("Invalid signature.").into());
    }

    let dir_index = if let Ok(dir_index) = get_header_value("X-Dir-Index", &req) {
        debug!("client req dir_index: {}", dir_index);
        out_dir = env::var(format!("OUT_DIR_{}", dir_index))
            .map_err(|_| ErrorBadRequest(format!("Unknown dir index: {}", dir_index)))?;
        Some(dir_index)
    } else {
        None
    };

    let mut tmp_filepath: Option<String> = None;
    let mut extension: Option<String> = None;

    if let Ok(Some(mut field)) = payload.try_next().await {
        debug!("field: {:?}", &field);
        let content = field.content_disposition();
        let filename = content
            .get_filename()
            .ok_or_else(|| ErrorBadRequest("No filename in content disposition"))?;

        debug!("filename: {}", filename);

        // filename cannot contains /
        if filename.contains('/') {
            return Err(ErrorBadRequest("Invalid filename.").into());
        }

        let a_nonce = nonce::nonce();
        let tmp_filepath_internal = format!("{}/tmp-{}-{}", out_dir, a_nonce, filename);
        debug!("tmp_filepath_internal: {}", tmp_filepath_internal);

        let mut f = File::create(&tmp_filepath_internal)?;

        tmp_filepath = Some(tmp_filepath_internal);

        let mut length = 0usize;

        let mut format: Option<ImageFormat> = None;

        while let Some(chunk) = field.next().await {
            let chunk = chunk.unwrap();
            let file_head = &chunk[0..10];
            length += chunk.len();
            if length > max_size {
                save_result = Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "File size exceeds 20 MB limit",
                ));
                break;
            }
            f.write_all(&chunk).unwrap();
            if format.is_none() {
                format = Some(image::guess_format(file_head).map_err(img_error)?);
                match format {
                    Some(image::ImageFormat::Png) => {
                        extension = Some("png".to_string());
                    }
                    Some(image::ImageFormat::Jpeg) => {
                        extension = Some("jpg".to_string());
                    }
                    _ => {
                        save_result = Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Invalid file format. Must be JPEG or PNG.",
                        ));
                        break;
                    }
                }
            }
        }
    }
    match save_result {
        Ok(_) => {
            // calculate hash and rename it accordingly
            if let Some(tmp_filepath) = tmp_filepath {
                let hash = move_by_hash(&tmp_filepath)?;

                Ok(HttpResponse::Ok().json(json!({
                    "nonce": nonce,
                    "sha1": hash,
                    "extension": extension.unwrap_or("jpg".to_string()),
                    "dindex": dir_index
                })))
            } else {
                Err(actix_error::ErrorBadRequest("No file uploaded").into())
            }
        }
        Err(e) => Err(actix_web::error::InternalError::new(
            e,
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        ))?,
    }
}

/// Moves the given file, renaming it with its SHA1 hash as a file name.
///
/// # Arguments
///
/// * `src_path` - A string slice that holds the path to the file.
///
/// # Returns
///
/// A `Result` containing a `String` with the file's SHA1 hash.
///
/// # Examples
///
/// ```
/// let hash = move_by_hash("my_image.jpg").unwrap();
/// ```
fn move_by_hash(src_path: &str) -> Result<String, io::Error> {
    let mut path = PathBuf::from(src_path);

    let extension = Path::new(src_path)
        .extension()
        .and_then(|a| a.to_str())
        .unwrap_or_else(|| "jpg");

    let mut file = File::open(src_path)?;
    let hash = crypto::get_sha1_file(&mut file)?;

    path.set_file_name(format!("{}.{}", hash, extension));

    let new_path = path.to_str().unwrap().to_string();

    debug!("old_path: {}", src_path);
    debug!("new_path: {}", new_path);

    std::fs::rename(src_path, new_path.clone())?;

    Ok(hash)
}

async fn get_nonce() -> ApiResult {
    let a_nonce = nonce::nonce();
    Ok(HttpResponse::Ok().body(a_nonce.to_string()))
}

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().expect(".env not found.");
    env_logger::init();

    println!("Welcome to Rantang version {}", env!("CARGO_PKG_VERSION"));

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");

    // check if exists and create if not
    if !Path::new(&out_dir).exists() {
        std::fs::create_dir_all(&out_dir).expect("Failed to create OUT_DIR");
    }

    let mut out_dir_count = 0;
    for (key, value) in env::vars() {
        if key.starts_with("OUT_DIR_") {
            let dir = value;
            // check if exists and create if not
            if !Path::new(&dir).exists() {
                std::fs::create_dir_all(&dir).expect(&format!("Failed to create {}", &dir));
            }
            out_dir_count += 1;
            debug!("out dir #{}: {}", out_dir_count, dir);
        }
    }
    debug!("total out dir: {}", out_dir_count);

    let cors_allow_all =
        env::var("CORS_ALLOW_ALL").ok().as_ref().map(|a| a.as_str()) == Some("true");

    if cors_allow_all {
        debug!("CORS_ALLOW_ALL is set to true. Allowing all origins.");
        HttpServer::new(|| {
            let cors = Cors::default()
                .allow_any_origin()
                .allow_any_header()
                .allow_any_method();
            App::new()
                .wrap(cors)
                .wrap(middleware::Logger::default())
                .route("/get_nonce", web::get().to(get_nonce))
                .route("/image", web::post().to(save_file))
        })
        .bind("127.0.0.1:8080")?
        .run()
        .await?;
    } else {
        HttpServer::new(|| {
            App::new()
                .wrap(middleware::Logger::default())
                .route("/get_nonce", web::get().to(get_nonce))
                .route("/image", web::post().to(save_file))
        })
        .bind("127.0.0.1:8080")?
        .run()
        .await?;
    }

    Ok(())
}
