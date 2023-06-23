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
use actix_web::http::header::ToStrError;
use actix_web::{error, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use dotenvy::dotenv;
use futures::{StreamExt, TryStreamExt};
use image::{ImageError, ImageFormat};
use log::debug;
use serde_json::json;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, io};

#[cfg(test)]
mod tests;

mod crypto;
mod nonce;

fn to_str_err(_: ToStrError) -> error::Error {
    error::ErrorBadRequest("Invalid UTF-8 in header value")
}

fn img_error(e: ImageError) -> error::Error {
    debug!("e: {}", e);
    error::ErrorBadRequest("Invalid image")
}

pub fn get_header_value<'a>(key: &str, req: &'a HttpRequest) -> Result<&'a str, Error> {
    let value = req
        .headers()
        .get(key)
        .ok_or_else(|| error::ErrorBadRequest(format!("No {} header", key)))?
        .to_str()
        .map_err(to_str_err)?;

    if value.is_empty() {
        return Err(error::ErrorBadRequest(format!("{} header is empty", key)));
    }

    Ok(value)
}

async fn save_file(req: HttpRequest, mut payload: Multipart) -> Result<HttpResponse, Error> {
    let mut save_result: Result<(), io::Error> = Ok(());
    let max_size = 20 * 1024 * 1024; // 20mb

    let out_dir = env::var("IMGSRC_DIR").expect("IMGSRC_DIR not set");
    let secret_key = env::var("SECRET_KEY").expect("SECRET_KEY not set");

    let signature = get_header_value("X-Signature", &req)?;
    let nonce = get_header_value("X-Nonce", &req)?;

    if !crypto::verify_signature(secret_key.as_bytes(), nonce.as_bytes(), signature) {
        return Err(error::ErrorBadRequest(
            "Invalid signature. Please check your secret key.",
        ));
    }

    let mut tmp_filepath: Option<String> = None;

    if let Ok(Some(mut field)) = payload.try_next().await {
        debug!("field: {:?}", &field);
        let content_type = field.content_disposition();
        let filename = content_type.get_filename().ok_or_else(|| 
            error::ErrorBadRequest("No filename in content disposition")
        )?;

        debug!("filename: {}", filename);

        let a_nonce = nonce::nonce();
        let tmp_filepath_internal = format!("{}/tmp-{}-{}", out_dir, a_nonce, filename);
        debug!("tmp_filepath_internal: {}", tmp_filepath_internal);

        let mut f = File::create(&tmp_filepath_internal)?;

        tmp_filepath = Some(tmp_filepath_internal);

        let mut length = 0usize;

        let mut format: Option<ImageFormat> = None;

        while let Some(chunk) = field.next().await {
            debug!("field.name(): {}", field.name());
            let chunk = chunk.unwrap();
            let file_head = &chunk[0..10];
            debug!("chunk: {:?}", file_head);
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
                    Some(image::ImageFormat::Png) | Some(image::ImageFormat::Jpeg) => (),
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
            let hash = move_by_hash(&tmp_filepath.unwrap())?;

            Ok(HttpResponse::Ok().json(json!({
                "nonce": nonce,
                "sha1": hash
            })))
        }
        Err(e) => Err(actix_web::error::InternalError::new(
            e,
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        )
        .into()),
    }
}

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

async fn get_nonce() -> Result<HttpResponse, io::Error> {
    let a_nonce = nonce::nonce();
    Ok(HttpResponse::Ok().body(a_nonce.to_string()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().expect(".env not found.");
    env_logger::init();

    println!("Welcome to Rantang version {}", env!("CARGO_PKG_VERSION"));

    std::fs::create_dir_all(env::var("IMGSRC_DIR").expect("IMGSRC_DIR not set"))
        .expect("Failed to create IMGSRC_DIR");

    let _server = HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .route("/get_nonce", web::get().to(get_nonce))
            .route("/image", web::post().to(save_file))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await;

    Ok(())
}
