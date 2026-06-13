// Decode `data`, produce a ≤300×500 JPEG thumbnail at `dest`, return
// (scan_width, scan_height, thumb_width, thumb_height) on success.
pub fn generate_thumb(data: &[u8], dest: &std::path::Path) -> Option<(u32, u32, u32, u32)> {
    let img = image::load_from_memory(data).ok()?;
    let (sw, sh) = (img.width(), img.height());
    let thumb = img.thumbnail(300, 500);
    let (tw, th) = (thumb.width(), thumb.height());
    thumb.save(dest).ok()?;
    Some((sw, sh, tw, th))
}

// If `filename` already exists in `scans_dir`, generate a new name that sorts
// immediately after it by appending 'b'..'z' before the extension.
pub fn resolve_scan_filename(scans_dir: &std::path::Path, filename: &str) -> String {
    if !scans_dir.join(filename).exists() {
        return filename.to_string();
    }
    let p = std::path::Path::new(filename);
    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or(filename);
    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
    let dot_ext = if ext.is_empty() { String::new() } else { format!(".{ext}") };
    for c in b'b'..=b'z' {
        let candidate = format!("{stem}{}{dot_ext}", c as char);
        if !scans_dir.join(&candidate).exists() {
            return candidate;
        }
    }
    format!("{stem}_dup{dot_ext}")
}