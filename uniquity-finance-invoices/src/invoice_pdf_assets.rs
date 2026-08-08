//! Bundled static assets for invoice PDF Typst templates.

use std::path::Path;

/// Filename written into the Typst work directory for the proprietor signature.
pub const SIGNATURE_IMAGE: &str = "sign.jpg";

const SIGNATURE_JPG: &[u8] = include_bytes!("../assets/sign.jpg");

/// Copy bundled PDF assets into `asset_dir` so Typst `#image(...)` paths resolve.
pub fn write_bundled_pdf_assets(asset_dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(asset_dir).map_err(|e| e.to_string())?;
    std::fs::write(asset_dir.join(SIGNATURE_IMAGE), SIGNATURE_JPG)
        .map_err(|e| format!("write signature image: {e}"))?;
    Ok(())
}
