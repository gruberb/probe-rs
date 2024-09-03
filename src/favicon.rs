use scraper::{Html, Selector};
use url::Url;

pub fn parse_favicon_url(html: &str, base_url: Url) -> Option<String> {
    let document = Html::parse_document(&html);

    let mut favicon_urls = Vec::new();

    // Helper function to parse the size attribute and extract the resolution
    fn parse_size(size: Option<&str>) -> u32 {
        size.and_then(|s| s.split('x').next())
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0)
    }

    // Check for <link rel="icon" type="image/svg+xml">
    let svg_selector = Selector::parse(r#"link[rel="icon"][type="image/svg+xml"]"#).unwrap();
    for svg_element in document.select(&svg_selector) {
        if let Some(href) = svg_element.value().attr("href") {
            if let Ok(url) = base_url.join(href) {
                // Add SVGs with high priority (considering SVGs as having infinite resolution)
                favicon_urls.push((u32::MAX, url.to_string()));
            }
        }
    }

    // Check for <link rel="icon"> or <link rel="shortcut icon">
    let icon_selector =
        Selector::parse(r#"link[rel~="icon"], link[rel~="shortcut icon"]"#).unwrap();
    for icon_element in document.select(&icon_selector) {
        if let Some(href) = icon_element.value().attr("href") {
            let size = parse_size(icon_element.value().attr("sizes"));
            if let Ok(url) = base_url.join(href) {
                favicon_urls.push((size, url.to_string()));
            }
        }
    }

    // Check for <link rel="apple-touch-icon">
    let apple_icon_selector = Selector::parse(r#"link[rel~="apple-touch-icon"]"#).unwrap();
    for icon_element in document.select(&apple_icon_selector) {
        if let Some(href) = icon_element.value().attr("href") {
            let size = parse_size(icon_element.value().attr("sizes"));
            if let Ok(url) = base_url.join(href) {
                favicon_urls.push((size, url.to_string()));
            }
        }
    }

    // Sort by size in descending order (largest first) and return the first URL
    favicon_urls.sort_by(|a, b| b.0.cmp(&a.0));
    favicon_urls.into_iter().map(|(_, url)| url).next()
}
