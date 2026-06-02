//! Vector scene export / import (W2) — save/load the committed vector scene
//! to a user-chosen `.ph2dvec` file.
//!
//! The app has no whole-scene persistence yet (sprites are load-only via the
//! `AssetDb`); this is a self-contained, safe vector-asset export/import so
//! drawn paths survive a reload. **Interim**: superseded once a proper
//! whole-scene / project save lands (which should own the trigger semantics).
//!
//! Format = a length-framed bundle over the EXISTING bounded per-asset codec
//! (`save_vector_asset` / `load_and_validate_vector_asset`): magic + version +
//! count, then `[u32 len][asset bytes]` per asset. Every asset still passes
//! `bounded_decode` + `validate` on load and the count is capped, so a
//! malformed / oversized / hostile file can neither over-allocate nor read out
//! of bounds. Triggered by Cmd+S / Cmd+O while a vector tool is active.

use ph2d_editor::toast::Toast;
use ph2d_vector::{Ph2dVectorAsset, load_and_validate_vector_asset, save_vector_asset};

use crate::App;

/// Defensive cap on the asset count a bundle may declare — bounds the load
/// allocation before any per-asset decode runs.
const MAX_BUNDLE_ASSETS: u32 = 100_000;

/// Magic so a foreign / truncated file is rejected before any decode.
const BUNDLE_MAGIC: &[u8; 4] = b"PHVB"; // PH2D Vector Bundle
/// Bundle layout version (the per-asset payload carries its own schema version).
const BUNDLE_VERSION: u32 = 1;

/// Serialize the committed scene into a length-framed bundle.
fn save_bundle(paths: &[Ph2dVectorAsset]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    out.extend_from_slice(BUNDLE_MAGIC);
    out.extend_from_slice(&BUNDLE_VERSION.to_le_bytes());
    out.extend_from_slice(&(paths.len() as u32).to_le_bytes());
    for asset in paths {
        let bytes = save_vector_asset(asset).map_err(|e| format!("encode: {e:?}"))?;
        out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        out.extend_from_slice(&bytes);
    }
    Ok(out)
}

/// Parse a bundle. The count is capped, every length is checked against the
/// remaining bytes, and each asset goes through `load_and_validate_vector_asset`
/// (bounded + validated) — so a hostile file can't over-allocate or over-read.
fn load_bundle(mut cur: &[u8]) -> Result<Vec<Ph2dVectorAsset>, String> {
    if take(&mut cur, 4)? != BUNDLE_MAGIC {
        return Err("not a PH2D vector bundle".into());
    }
    let version = read_u32(&mut cur)?;
    if version != BUNDLE_VERSION {
        return Err(format!("unsupported bundle version {version}"));
    }
    let count = read_u32(&mut cur)?;
    if count > MAX_BUNDLE_ASSETS {
        return Err(format!(
            "bundle declares {count} assets (> {MAX_BUNDLE_ASSETS})"
        ));
    }
    let mut paths = Vec::new();
    for _ in 0..count {
        let len = read_u32(&mut cur)? as usize;
        let bytes = take(&mut cur, len)?;
        let asset = load_and_validate_vector_asset(bytes).map_err(|e| format!("decode: {e:?}"))?;
        paths.push(asset);
    }
    Ok(paths)
}

/// Read a little-endian `u32`, advancing the cursor.
fn read_u32(cur: &mut &[u8]) -> Result<u32, String> {
    let b = take(cur, 4)?;
    Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

/// Split `n` bytes off the front of the cursor, erroring if truncated.
fn take<'a>(cur: &mut &'a [u8], n: usize) -> Result<&'a [u8], String> {
    if cur.len() < n {
        return Err("truncated bundle".into());
    }
    let (head, rest) = cur.split_at(n);
    *cur = rest;
    Ok(head)
}

impl App {
    fn push_toast(&mut self, toast: Toast) {
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.toasts.push(toast);
        }
    }

    /// `true` if any of the five vector tools is the active tool — the gate for
    /// the Cmd+S / Cmd+O export/import shortcuts (so they don't shadow a future
    /// global save/open).
    pub(crate) fn any_vector_tool_active(&self) -> bool {
        let Some(active) = self.gfx.as_ref().and_then(|g| g.tools.active()) else {
            return false;
        };
        let id = active.id();
        use ph2d_editor::ToolId;
        id == ToolId::new("vector_pen")
            || id == ToolId::new("vector_pencil")
            || id == ToolId::new("vector_shape")
            || id == ToolId::new("vector_select")
            || id == ToolId::new("vector_direct")
    }

    /// Cmd+S while a vector tool is active — export the committed scene to a
    /// user-chosen `.ph2dvec` file. Returns `true` (consumes the key).
    pub(crate) fn save_vector_scene(&mut self) -> bool {
        if self.committed_vector_pen_paths.is_empty() {
            self.push_toast(Toast::info("No vector paths to save"));
            return true;
        }
        let Some(path) = rfd::FileDialog::new()
            .add_filter("PH2D Vector Scene", &["ph2dvec"])
            .set_file_name("scene.ph2dvec")
            .save_file()
        else {
            return true; // cancelled
        };
        let result = save_bundle(&self.committed_vector_pen_paths)
            .and_then(|bytes| std::fs::write(&path, bytes).map_err(|e| e.to_string()));
        match result {
            Ok(()) => {
                let n = self.committed_vector_pen_paths.len();
                self.push_toast(Toast::success(format!("Saved {n} vector path(s)")));
            }
            Err(e) => self.push_toast(Toast::error(format!("Save failed: {e}"))),
        }
        true
    }

    /// Cmd+O while a vector tool is active — import a `.ph2dvec` file, replacing
    /// the committed scene. Returns `true` (consumes the key).
    pub(crate) fn load_vector_scene(&mut self) -> bool {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("PH2D Vector Scene", &["ph2dvec"])
            .pick_file()
        else {
            return true; // cancelled
        };
        let result = std::fs::read(&path)
            .map_err(|e| e.to_string())
            .and_then(|bytes| load_bundle(&bytes));
        match result {
            Ok(paths) => {
                let n = paths.len();
                self.committed_vector_pen_paths = paths;
                self.vector_selection.clear();
                self.push_toast(Toast::success(format!("Loaded {n} vector path(s)")));
            }
            Err(e) => self.push_toast(Toast::error(format!("Load failed: {e}"))),
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_bundle_round_trips() {
        let bytes = save_bundle(&[]).expect("encode empty");
        assert!(load_bundle(&bytes).expect("decode empty").is_empty());
    }

    #[test]
    fn rejects_foreign_magic() {
        let mut b = b"XXXX".to_vec();
        b.extend_from_slice(&1u32.to_le_bytes());
        b.extend_from_slice(&0u32.to_le_bytes());
        assert!(load_bundle(&b).is_err());
    }

    #[test]
    fn rejects_truncated_header() {
        let bytes = save_bundle(&[]).expect("encode");
        assert!(load_bundle(&bytes[..bytes.len() - 1]).is_err());
    }

    #[test]
    fn rejects_absurd_count() {
        let mut b = BUNDLE_MAGIC.to_vec();
        b.extend_from_slice(&BUNDLE_VERSION.to_le_bytes());
        b.extend_from_slice(&(MAX_BUNDLE_ASSETS + 1).to_le_bytes());
        assert!(load_bundle(&b).is_err());
    }
}
