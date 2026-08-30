use crate::hocr_parser::HocrBbox;
use crate::routes::projects::models::{CropEdges, Hint, ProcessingSettings};
use image::{GenericImage, GenericImageView, ImageFormat};
use std::io::Cursor;


fn write_image_without_unneeded_alpha(
    img: image::DynamicImage,
    format: ImageFormat,
) -> Result<Vec<u8>, String> {
    let mut buf = Cursor::new(Vec::new());

    match format {
        ImageFormat::Jpeg => img
            .to_rgb8()
            .write_to(&mut buf, format)
            .map_err(|e| e.to_string())?,
        _ => img.write_to(&mut buf, format).map_err(|e| e.to_string())?,
    }

    Ok(buf.into_inner())
}

/// White out the crop-edge margins of an image in memory.
/// The image dimensions are unchanged so all hOCR coordinates stay valid.
pub fn apply_crop_mask(
    bytes: &[u8],
    crop: CropEdges,
    settings: Option<&ProcessingSettings>,
    hints: &[Hint],
) -> Result<Vec<u8>, String> {
    apply_image_pipeline(bytes, Some(crop), settings, hints)
}

/// Unified image processing pipeline: load, crop mask, and apply processing settings.
pub fn apply_image_pipeline(
    bytes: &[u8],
    crop: Option<CropEdges>,
    settings: Option<&ProcessingSettings>,
    hints: &[Hint],
) -> Result<Vec<u8>, String> {
    let has_crop = crop
        .map(|c| c.left > 0 || c.top > 0 || c.right > 0 || c.bottom > 0)
        .unwrap_or(false);

    let has_settings = settings.map(|s| s.has_effect()).unwrap_or(false);
    let has_hints = !hints.is_empty();

    if !has_crop && !has_settings && !has_hints {
        return Ok(bytes.to_vec());
    }

    let format = image::guess_format(bytes).map_err(|e| e.to_string())?;
    let mut img = image::load_from_memory(bytes).map_err(|e| e.to_string())?;

    if let Some(crop) = crop {
        apply_crop_mask_to_image(&mut img, crop);
    }

    if has_hints {
        apply_hints_to_image(&mut img, hints);
    }

    if let Some(settings) = settings {
        img = apply_processing_settings(img, settings);
    }

    write_image_without_unneeded_alpha(img, format)
}

/// Apply desaturation, contrast, and brightness adjustments to a DynamicImage.
pub fn apply_processing_settings(
    mut img: image::DynamicImage,
    settings: &ProcessingSettings,
) -> image::DynamicImage {
    if !settings.has_effect() {
        return img;
    }

    if settings.desaturate {
        img = img.grayscale();
    }

    if settings.contrast != 0.0 {
        img = img.adjust_contrast(settings.contrast);
    }

    if settings.brightness != 0.0 {
        img = img.brighten(settings.brightness as i32);
    }

    img
}

/// White out regions defined by hints (dropcaps and images).
pub fn apply_hints_to_image(img: &mut image::DynamicImage, hints: &[Hint]) {
    if hints.is_empty() {
        return;
    }

    let (w, h) = (img.width(), img.height());
    let white = image::Rgba([255u8, 255, 255, 255]);

    for hint in hints {
        let area = hint.area;
        let left = area.left.min(w);
        let right = area.right.min(w);
        let top = area.top.min(h);
        let bottom = area.bottom.min(h);

        for y in top..bottom {
            for x in left..right {
                img.put_pixel(x, y, white);
            }
        }
    }
}

/// White out the crop-edge margins of an image in memory (modifies image in place).
pub fn apply_crop_mask_to_image(img: &mut image::DynamicImage, crop: CropEdges) {
    if crop.left == 0 && crop.top == 0 && crop.right == 0 && crop.bottom == 0 {
        return;
    }

    let (w, h) = (img.width(), img.height());
    let white = image::Rgba([255u8, 255, 255, 255]);

    let left = crop.left.min(w);
    let right = crop.right.min(w);
    let top = crop.top.min(h);
    let bot = crop.bottom.min(h);

    // Top strip
    for y in 0..top {
        for x in 0..w {
            img.put_pixel(x, y, white);
        }
    }
    // Bottom strip
    for y in h.saturating_sub(bot)..h {
        for x in 0..w {
            img.put_pixel(x, y, white);
        }
    }
    // Left strip
    for x in 0..left {
        for y in 0..h {
            img.put_pixel(x, y, white);
        }
    }
    // Right strip
    for x in w.saturating_sub(right)..w {
        for y in 0..h {
            img.put_pixel(x, y, white);
        }
    }
}

/// Extract a content area from a page image, blotting out anything outside the page's
/// crop edges and blotting out specific child image blocks.
pub fn extract_and_process_carea_image(
    page_img_bytes: &[u8],
    carea_bbox: HocrBbox,
    page_crop: CropEdges,
    image_blocks: &[HocrBbox],
    hints: &[Hint],
    padding: u32,
    settings: Option<&ProcessingSettings>,
) -> Result<Vec<u8>, String> {
    let format = image::guess_format(page_img_bytes).map_err(|e| e.to_string())?;
    let img = image::load_from_memory(page_img_bytes).map_err(|e| e.to_string())?;

    let (pw, ph) = (img.width() as i32, img.height() as i32);
    let white = image::Rgba([255u8, 255, 255, 255]);

    // 1. Blot out regions outside page crop edges (on the FULL image)
    let c_left = page_crop.left as i32;
    let c_top = page_crop.top as i32;
    let c_right = pw - page_crop.right as i32;
    let c_bottom = ph - page_crop.bottom as i32;

    // We only need to blot if the carea overlaps the crop zones
    // But it's easier to just blot the whole crop zone on the original image if it's not too big.
    // However, the instructions say: "Blot out regions that fall outside the page's crop_edges."
    // And "Crop to the carea bbox."
    
    // Let's crop first to save memory/time, then blot relative to the new crop.
    let target_left = carea_bbox.left().max(0);
    let target_top = carea_bbox.top().max(0);
    let target_right = carea_bbox.right().min(pw);
    let target_bottom = carea_bbox.bottom().min(ph);
    
    let target_width = (target_right - target_left).max(0) as u32;
    let target_height = (target_bottom - target_top).max(0) as u32;

    if target_width == 0 || target_height == 0 {
        return Err("Carea has zero width or height".to_string());
    }

    let mut cropped = img.view(target_left as u32, target_top as u32, target_width, target_height).to_image();

    // 2. Blot out regions that fall outside the page's crop_edges, relative to the cropped image.
    for y in 0..target_height {
        let abs_y = target_top + y as i32;
        for x in 0..target_width {
            let abs_x = target_left + x as i32;
            
            let is_outside_crop = abs_x < c_left || abs_x >= c_right || abs_y < c_top || abs_y >= c_bottom;
            
            if is_outside_crop {
                cropped.put_pixel(x, y, white);
                continue;
            }

            // 3. Blot out provided child image block bboxes.
            for img_bbox in image_blocks {
                if abs_x >= img_bbox.left() && abs_x < img_bbox.right() && abs_y >= img_bbox.top() && abs_y < img_bbox.bottom() {
                    cropped.put_pixel(x, y, white);
                    break;
                }
            }

            // 4. Blot out regions defined by hints.
            for hint in hints {
                let area = hint.area;
                if abs_x >= area.left as i32 && abs_x < area.right as i32 && abs_y >= area.top as i32 && abs_y < area.bottom as i32 {
                    cropped.put_pixel(x, y, white);
                    break;
                }
            }
        }
    }

    if padding > 0 {
        let new_width = target_width + 2 * padding;
        let new_height = target_height + 2 * padding;
        let mut padded = image::ImageBuffer::from_pixel(new_width, new_height, white);

        for y in 0..target_height {
            for x in 0..target_width {
                padded.put_pixel(x + padding, y + padding, *cropped.get_pixel(x, y));
            }
        }
        let mut final_img = image::DynamicImage::ImageRgba8(padded);
        if let Some(s) = settings {
            final_img = apply_processing_settings(final_img, s);
        }
        write_image_without_unneeded_alpha(final_img, format)
    } else {
        let mut final_img = image::DynamicImage::ImageRgba8(cropped);
        if let Some(s) = settings {
            final_img = apply_processing_settings(final_img, s);
        }
        write_image_without_unneeded_alpha(final_img, format)
    }
}
