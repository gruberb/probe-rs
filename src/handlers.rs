use axum::{extract::Query, http::StatusCode, response::Response};
use serde::Deserialize;
use tracing::{debug, error, info, warn};
use url::Url;

use crate::{favicon, image};

#[derive(Deserialize)]
pub struct FaviconQuery {
    pub url: String,       // URL as a query parameter
    pub size: Option<u32>, // Optional size parameter
}

pub async fn fetch_favicon(Query(query): Query<FaviconQuery>) -> Result<Response, StatusCode> {
    let url = query.url.clone();
    info!("Try to fetch favicon for: {url}");
    let client = reqwest::Client::new();
    let base_url = format!("http://{}", url);

    // Fetch the HTML
    let resp = client
        .get(base_url.as_str())
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

    let base_url = Url::parse(&base_url).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Attempt to find the favicon URL
    let favicon_url = match favicon::fetch_favicon_url(&html, base_url.clone()) {
        Some(url) => Some(url),
        None => {
            warn!("No Favicon URL found, trying basic locations");
            favicon::try_basic_locations(base_url.clone()).await
        }
    }
    .unwrap_or_else(|| format!("{}/favicon.ico", base_url));

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

    info!("Fetched the favicon from: {favicon_url}");

    let content_type = favicon_resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string())
        .unwrap_or_default();

    let favicon_data = favicon_resp.bytes().await.map_err(|e| {
        error!("Failed to read favicon data: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    debug!("Got the favicon data, content-type: {content_type}");

    // Check if the favicon is an SVG
    if content_type == "image/svg+xml" {
        // Return the SVG as-is without resizing
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "image/svg+xml")
            .body(favicon_data.into())
            .unwrap());
    }

    // If not SVG, resize the favicon
    let size = query.size.unwrap_or(32);
    let resized_favicon = match image::resize_image(&favicon_data, size) {
        Some(data) => data,
        None => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "image/png") // Assuming PNG after resize
        .body(resized_favicon.into())
        .unwrap())
}
