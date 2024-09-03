use axum::{extract::Path, http::StatusCode, response::Json, routing::get, Router};
use base64::{engine::general_purpose, Engine as _};
use image::ImageOutputFormat;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use tracing::{debug, error, info};
use url::Url;

#[derive(Deserialize, Serialize)]
struct FaviconResponse {
    favicons: HashMap<String, String>,
}

async fn resize_image(img_data: &[u8], size: u32) -> Option<Vec<u8>> {
    image::load_from_memory(img_data)
        .ok()
        .map(|img| {
            let resized = img.resize(size, size, image::imageops::FilterType::Lanczos3);
            let mut buffer = Vec::new();
            resized
                .write_to(&mut Cursor::new(&mut buffer), ImageOutputFormat::Png)
                .ok()?;
            Some(buffer)
        })
        .flatten()
}

async fn fetch_favicon_url(html: &str, base_url: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let base_url = Url::parse(base_url).ok()?;

    // Check for <link rel="icon"> or <link rel="shortcut icon">
    let icon_selector =
        Selector::parse(r#"link[rel~="icon"], link[rel~="shortcut icon"]"#).unwrap();
    if let Some(icon_element) = document.select(&icon_selector).next() {
        if let Some(href) = icon_element.value().attr("href") {
            return base_url.join(href).ok().map(|u| u.to_string());
        }
    }

    // Check for <link rel="apple-touch-icon">
    let apple_icon_selector = Selector::parse(r#"link[rel~="apple-touch-icon"]"#).unwrap();
    if let Some(icon_element) = document.select(&apple_icon_selector).next() {
        if let Some(href) = icon_element.value().attr("href") {
            return base_url.join(href).ok().map(|u| u.to_string());
        }
    }

    // Check for <meta property="og:image"> (Open Graph image)
    let og_image_selector = Selector::parse(r#"meta[property="og:image"]"#).unwrap();
    if let Some(og_element) = document.select(&og_image_selector).next() {
        if let Some(content) = og_element.value().attr("content") {
            return base_url.join(content).ok().map(|u| u.to_string());
        }
    }

    // If none of the above worked, try common favicon locations
    let common_locations = [
        "/favicon.ico",
        "/favicon.png",
        "/apple-touch-icon.png",
        "/apple-touch-icon-precomposed.png",
    ];

    // for location in &common_locations {
    //     let favicon_url = base_url.join(location).ok()?;
    //     if Client::new()
    //         .head(favicon_url.as_str())
    //         .send()
    //         .await
    //         .is_ok()
    //     {
    //         return Some(favicon_url.to_string());
    //     }
    // }

    None
}

async fn fetch_favicon(Path(url): Path<String>) -> Result<Json<FaviconResponse>, StatusCode> {
    info!("Try to fetch favicon for: {url}");
    let client = Client::new();
    let base_url = format!("http://{}", url);

    // Fetch the HTML
    let resp = client
        .get(&base_url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (compatible; MSIE 10.0; Windows NT 6.1; Trident/6.0)",
        )
        .send()
        .await
        .map_err(|e| {
            error!("Trying to fetch HTML {e}");
            StatusCode::BAD_REQUEST
        })?;

    let html = resp.text().await.map_err(|e| {
        error!("Trying to parse the HTML as text {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Find favicon URL
    let favicon_url = fetch_favicon_url(&html, &base_url)
        .await
        .unwrap_or_else(|| format!("{}/favicon.ico", base_url));

    info!("Got the favicon url: {favicon_url}");

    // Fetch favicon
    let favicon_resp = client
        .get(&favicon_url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (compatible; MSIE 10.0; Windows NT 6.1; Trident/6.0)",
        )
        .send()
        .await
        .map_err(|e| {
            error!("Trying to get favicon url: {e}");
            StatusCode::BAD_REQUEST
        })?;

    info!("Fetched the favicon: {favicon_resp:?}");

    let favicon_data = favicon_resp.bytes().await.map_err(|e| {
        error!("Get the favicon as bytes: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    debug!("Got the favicon data: {favicon_data:?}");

    // Resize favicon
    let sizes = vec![16, 32, 48, 64];
    let mut resized_favicons = HashMap::new();

    info!("Trying to resize favicon");

    for size in sizes {
        if let Some(resized) = resize_image(&favicon_data, size).await {
            let base64 = general_purpose::STANDARD.encode(&resized);
            debug!("Resized to {size}");

            resized_favicons.insert(
                format!("{}x{}", size, size),
                format!("data:image/png;base64,{}", base64),
            );
        }
    }

    Ok(Json(FaviconResponse {
        favicons: resized_favicons,
    }))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new().route("/favicon/:url", get(fetch_favicon));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
