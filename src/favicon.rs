use reqwest::Client;
use scraper::{Html, Selector};
use url::Url;

pub fn fetch_favicon_url(html: &str, base_url: Url) -> Option<String> {
    let document = Html::parse_document(&html);

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

    None
}

pub async fn try_basic_locations(base_url: Url) -> Option<String> {
    // If none of the above worked, try common favicon locations
    let common_locations = [
        "/favicon.ico",
        "/favicon.png",
        "/apple-touch-icon.png",
        "/apple-touch-icon-precomposed.png",
    ];

    for location in &common_locations {
        let favicon_url = base_url.join(location).ok()?;
        if Client::new()
            .head(favicon_url.as_str())
            .send()
            .await
            .ok()?
            .status()
            .is_success()
        {
            return Some(favicon_url.to_string());
        }
    }

    None
}
