use crate::routes::projects::models::CropEdges;
use image::GenericImage;
use std::io::Cursor;

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

    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, format).map_err(|e| e.to_string())?;
    Ok(buf.into_inner())
}
