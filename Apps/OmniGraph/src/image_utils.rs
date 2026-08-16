use crate::hocr_parser::HocrBbox;
use crate::routes::projects::models::CropEdges;
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
pub fn apply_crop_mask(bytes: &[u8], crop: CropEdges) -> Result<Vec<u8>, String> {
    if crop.left == 0 && crop.top == 0 && crop.right == 0 && crop.bottom == 0 {
        return Ok(bytes.to_vec());
    }

    let format = image::guess_format(bytes).map_err(|e| e.to_string())?;
    let mut img = image::load_from_memory(bytes).map_err(|e| e.to_string())?;

    let (w, h) = (img.width(), img.height());
    let white = image::Rgba([255u8, 255, 255, 255]);

    let left  = crop.left.min(w);
    let right = crop.right.min(w);
    let top   = crop.top.min(h);
    let bot   = crop.bottom.min(h);

    // Top strip
    for y in 0..top {
        for x in 0..w { img.put_pixel(x, y, white); }
    }
    // Bottom strip
    for y in h.saturating_sub(bot)..h {
        for x in 0..w { img.put_pixel(x, y, white); }
    }
    // Left strip (full height so corners are covered)
    for x in 0..left {
        for y in 0..h { img.put_pixel(x, y, white); }
    }
    // Right strip
    for x in w.saturating_sub(right)..w {
        for y in 0..h { img.put_pixel(x, y, white); }
    }

    write_image_without_unneeded_alpha(img, format)
}

/// Extract a content area from a page image, blotting out anything outside the page's
/// crop edges and blotting out specific child image blocks.
pub fn extract_and_process_carea_image(
    page_img_bytes: &[u8],
    carea_bbox: HocrBbox,
    page_crop: CropEdges,
    image_blocks: &[HocrBbox],
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
        }
    }

    write_image_without_unneeded_alpha(image::DynamicImage::ImageRgba8(cropped), format)
}
