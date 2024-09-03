use reqwest::Client;
use scraper::{Html, Selector};
use url::Url;
use std::collections::HashMap;

pub fn fetch_favicon_url(html: &str, base_url: Url) -> Option<String> {
    let document = Html::parse_document(html);
    let mut icon_candidates: HashMap<String, i32> = HashMap::new();

    // Helper function to add a candidate
    fn add_candidate(candidates: &mut HashMap<String, i32>, base_url: &Url, href: &str, priority: i32) {
        if let Ok(full_url) = base_url.join(href) {
            candidates.insert(full_url.to_string(), priority);
        }
    }

    // Check for <link rel="icon"> or <link rel="shortcut icon">
    let icon_selector = Selector::parse(r#"link[rel~="icon"], link[rel~="shortcut icon"]"#).unwrap();
    for icon_element in document.select(&icon_selector) {
        if let Some(href) = icon_element.value().attr("href") {
            let priority = icon_element.value().attr("sizes")
                .and_then(|sizes| sizes.split('x').next())
                .and_then(|size| size.parse::<i32>().ok())
                .unwrap_or(16);  // Default to 16 if size is not specified
            add_candidate(&mut icon_candidates, &base_url, href, priority);
        }
    }

    // Check for <link rel="apple-touch-icon">
    let apple_icon_selector = Selector::parse(r#"link[rel~="apple-touch-icon"]"#).unwrap();
    for icon_element in document.select(&apple_icon_selector) {
        if let Some(href) = icon_element.value().attr("href") {
            let priority = icon_element.value().attr("sizes")
                .and_then(|sizes| sizes.split('x').next())
                .and_then(|size| size.parse::<i32>().ok())
                .unwrap_or(180);  // Apple touch icons are often 180x180
            add_candidate(&mut icon_candidates, &base_url, href, priority);
        }
    }

    // Add common favicon locations
    let common_locations = [
        "/favicon.ico",
        "/favicon.png",
        "/apple-touch-icon.png",
        "/apple-touch-icon-precomposed.png",
        "/touch-icon-iphone.png",
        "/touch-icon-ipad.png",
        "/touch-icon-iphone-retina.png",
        "/touch-icon-ipad-retina.png",
        "/browserconfig.xml",  // For Microsoft tile icons
        "/site.webmanifest",   // Web App Manifest (might contain icon information)
    ];

    for location in &common_locations {
        add_candidate(&mut icon_candidates, &base_url, location, 10);  // Lower priority for common locations
    }

    // Select the highest resolution favicon
    icon_candidates.into_iter().max_by_key(|&(_, priority)| priority).map(|(url, _)| url)
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
