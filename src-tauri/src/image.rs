use std::fs;
use std::path::{Path, PathBuf};

use base64::{engine::general_purpose::STANDARD, Engine as _};

const MAX_SIZE: usize = 5 * 1024 * 1024;

/// Validates magic bytes and size. Returns the file extension ("jpg" or "png") on success.
pub fn validate_image(bytes: &[u8]) -> Result<&'static str, String> {
    if bytes.len() > MAX_SIZE {
        return Err("File exceeds 5 MB limit".to_string());
    }

    let is_jpeg = bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF;
    let is_png = bytes.len() >= 8
        && bytes[0..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    if is_jpeg {
        Ok("jpg")
    } else if is_png {
        Ok("png")
    } else {
        Err("Only JPG and PNG supported".to_string())
    }
}

/// `{dirs::data_dir()}/social-graph/images`, created if missing.
pub fn images_dir() -> PathBuf {
    let mut dir = dirs::data_dir().expect("Kein Daten-Verzeichnis gefunden");
    dir.push("social-graph");
    dir.push("images");
    fs::create_dir_all(&dir).expect("Konnte Bilder-Verzeichnis nicht anlegen");
    dir
}

/// Reads the file at `path` and returns it as a base64 data URL, or `None` if unreadable.
pub fn encode_data_url(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    let ext = path.extension()?.to_str()?.to_lowercase();
    let mime = if ext == "png" { "png" } else { "jpeg" };
    let b64 = STANDARD.encode(&bytes);
    Some(format!("data:image/{mime};base64,{b64}"))
}
