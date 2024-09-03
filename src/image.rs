use image::ImageFormat;
use std::io::Cursor;

pub fn resize_image(img_data: &[u8], size: u32) -> Option<Vec<u8>> {
    image::load_from_memory(img_data)
        .ok()
        .map(|img| {
            let resized = img.resize(size, size, image::imageops::FilterType::Lanczos3);
            let mut buffer = Vec::new();
            resized
                .write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)
                .ok()?;
            Some(buffer)
        })
        .flatten()
}