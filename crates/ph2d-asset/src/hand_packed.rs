//! M14.5 B — hand-packed atlas metadata loader.
//!
//! Parses the JSON sidecar an artist's atlas tool emits alongside the
//! PNG sheet. PH2D supports the two de-facto standard formats:
//!
//! - **Aseprite "Hash"** (`File → Export Sprite Sheet → JSON Data → Hash`).
//!   The default format Aseprite produces when an artist exports a
//!   character sheet.
//! - **TexturePacker JSON (Hash)** — same shape Aseprite uses, just
//!   different `meta.app` string. Many third-party tools (ShoeBox,
//!   free-texture-packer) emit this format because it's the simplest
//!   industry-standard layout.
//!
//! We deliberately skip the **Array** variant (`"frames": [ ... ]`)
//! that some Aseprite versions emit by default. It carries the same
//! data ordered as an array; the Hash variant is preferable because
//! the frame names ARE the lookup key the renderer needs at extract
//! time. If a user has Array output, they can re-export with the Hash
//! option ticked.
//!
//! ## Output shape
//!
//! Parsing produces an [`AtlasMeta`] that downstream consumers
//! (renderer's future `HandPackedAtlasStore`) turn into a single
//! `wgpu::Texture` + `BTreeMap<String, AtlasRegion>`. The image
//! itself is loaded separately via the regular [`crate::AssetDb`]
//! PNG path — `AtlasMeta::image_filename` points at the sibling PNG.
//!
//! HR-5 / ADR-0022: `BTreeMap`, not `HashMap`. Region count stays
//! small (typically < 100) so the log-N lookup cost is irrelevant.

use serde::Deserialize;
use std::collections::BTreeMap;

/// One sprite region inside a hand-packed atlas. The rect is in
/// atlas-pixel coordinates with (0, 0) at the texture's top-left —
/// same convention as [`crate::Asset::ImageRgba8`] storage and as
/// `crate::TextureAtlas::region_uv`'s input.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AtlasRegion {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// Parsed metadata for one hand-packed atlas: which PNG to load + the
/// rect each named sprite occupies inside it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtlasMeta {
    /// Filename of the sibling PNG referenced by `meta.image` in the
    /// source JSON. Callers resolve it relative to the JSON file's
    /// directory; the parser doesn't touch the filesystem.
    pub image_filename: String,
    /// Width × height of the source PNG as declared in `meta.size`.
    /// Useful for UV math sanity checks (each region must lie inside
    /// these bounds).
    pub image_size: (u32, u32),
    /// Map from sprite name to its rect. Aseprite encodes the name as
    /// the JSON key under `frames`; for TexturePacker-style sheets
    /// the key is typically `"hero/idle/0001.png"` or similar — we
    /// preserve it verbatim.
    pub regions: BTreeMap<String, AtlasRegion>,
}

impl AtlasMeta {
    /// Look up a region by name. Returns `None` if the name was not
    /// in the source JSON. Hot-path callable (O(log N) on a small N).
    pub fn region(&self, name: &str) -> Option<&AtlasRegion> {
        self.regions.get(name)
    }

    /// Convert a region to a `(u_min, v_min, u_max, v_max)` UV tuple
    /// in 0..1 atlas space — same shape `TextureAtlas::region_uv`
    /// produces. Convenience for the future renderer extract path.
    pub fn region_uv(&self, region: &AtlasRegion) -> [f32; 4] {
        let (w, h) = self.image_size;
        let inv_w = 1.0 / w as f32;
        let inv_h = 1.0 / h as f32;
        [
            region.x as f32 * inv_w,
            region.y as f32 * inv_h,
            (region.x + region.w) as f32 * inv_w,
            (region.y + region.h) as f32 * inv_h,
        ]
    }
}

/// Errors returned by [`parse_atlas_meta`].
#[derive(Debug)]
pub enum AtlasMetaError {
    /// JSON didn't parse (malformed bytes or invalid UTF-8).
    Json(serde_json::Error),
    /// JSON parsed but `meta.image` was empty or missing.
    MissingImageFilename,
    /// `meta.size` was missing or held a non-positive dimension.
    InvalidImageSize,
    /// `frames` was the array variant — we only support Hash. Re-
    /// export the source with Aseprite's "Hash" option ticked.
    UnsupportedFramesShape,
    /// A region rect referenced pixels outside `meta.size`. Almost
    /// always a sign the JSON and the PNG drifted (artist edited one
    /// without re-exporting the other).
    RegionOutsideImage { name: String },
}

impl std::fmt::Display for AtlasMetaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(e) => write!(f, "atlas JSON parse error: {e}"),
            Self::MissingImageFilename => write!(f, "atlas JSON missing meta.image"),
            Self::InvalidImageSize => write!(f, "atlas JSON missing or invalid meta.size"),
            Self::UnsupportedFramesShape => write!(
                f,
                "atlas JSON uses the unsupported Array shape; re-export with Hash"
            ),
            Self::RegionOutsideImage { name } => write!(
                f,
                "atlas region '{name}' extends past meta.size — JSON and PNG out of sync?"
            ),
        }
    }
}

impl std::error::Error for AtlasMetaError {}

// ───────────────────────── Serde shape ───────────────────────────

/// On-disk JSON shape. Both Aseprite and TexturePacker emit a similar
/// structure: `frames` map + `meta` object. We only deserialize the
/// fields we need, ignoring `meta.app`, `meta.version`, `meta.scale`,
/// `meta.smartupdate`, and the per-frame `rotated/trimmed/duration`
/// (which we don't act on — we always treat sprites as axis-aligned
/// non-trimmed full-size).
#[derive(Deserialize)]
struct JsonRoot {
    /// Hash variant: a JSON object keyed by frame name. Array
    /// variant (`Vec<JsonFrame>`) is not deserialized by this type;
    /// it would fail with a type error and surface as Json error.
    frames: BTreeMap<String, JsonFrame>,
    meta: JsonMeta,
}

#[derive(Deserialize)]
struct JsonFrame {
    frame: JsonRect,
}

#[derive(Deserialize)]
struct JsonRect {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

#[derive(Deserialize)]
struct JsonMeta {
    image: String,
    size: JsonSize,
}

#[derive(Deserialize)]
struct JsonSize {
    w: u32,
    h: u32,
}

/// Parse `bytes` as either Aseprite Hash JSON or TexturePacker JSON
/// (same shape — both keyed under `frames` as an object). Returns an
/// [`AtlasMeta`] with regions sorted by name (BTreeMap iteration
/// order is deterministic, useful for tests and snapshot diffs).
///
/// Per HR-3 the function allocates only what's strictly necessary
/// (one String per region name, the regions map itself). serde_json's
/// reader allocates a Value tree internally — fine for asset load
/// time (off the hot path).
pub fn parse_atlas_meta(bytes: &[u8]) -> Result<AtlasMeta, AtlasMetaError> {
    let root: JsonRoot = serde_json::from_slice(bytes).map_err(AtlasMetaError::Json)?;
    if root.meta.image.is_empty() {
        return Err(AtlasMetaError::MissingImageFilename);
    }
    let (w, h) = (root.meta.size.w, root.meta.size.h);
    if w == 0 || h == 0 {
        return Err(AtlasMetaError::InvalidImageSize);
    }
    let mut regions = BTreeMap::new();
    for (name, frame) in root.frames {
        let r = AtlasRegion {
            x: frame.frame.x,
            y: frame.frame.y,
            w: frame.frame.w,
            h: frame.frame.h,
        };
        if r.x + r.w > w || r.y + r.h > h {
            return Err(AtlasMetaError::RegionOutsideImage { name });
        }
        regions.insert(name, r);
    }
    Ok(AtlasMeta {
        image_filename: root.meta.image,
        image_size: (w, h),
        regions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal Aseprite Hash export — 1 frame named `idle_0`.
    const ASEPRITE_HASH_JSON: &str = r#"{
      "frames": {
        "idle_0": {
          "frame": { "x": 0, "y": 0, "w": 32, "h": 32 },
          "rotated": false,
          "trimmed": false,
          "spriteSourceSize": { "x": 0, "y": 0, "w": 32, "h": 32 },
          "sourceSize": { "w": 32, "h": 32 },
          "duration": 100
        }
      },
      "meta": {
        "app": "http://www.aseprite.org/",
        "version": "1.3.0",
        "image": "hero.png",
        "format": "RGBA8888",
        "size": { "w": 256, "h": 256 },
        "scale": "1"
      }
    }"#;

    /// TexturePacker-style export — same shape, different `app`.
    const TEXTUREPACKER_HASH_JSON: &str = r#"{
      "frames": {
        "tile_grass": {
          "frame": { "x": 16, "y": 32, "w": 16, "h": 16 }
        },
        "tile_water": {
          "frame": { "x": 64, "y": 0, "w": 16, "h": 16 }
        }
      },
      "meta": {
        "app": "https://www.codeandweb.com/texturepacker",
        "image": "tileset.png",
        "size": { "w": 128, "h": 128 }
      }
    }"#;

    #[test]
    fn parses_aseprite_hash_format() {
        let meta = parse_atlas_meta(ASEPRITE_HASH_JSON.as_bytes()).unwrap();
        assert_eq!(meta.image_filename, "hero.png");
        assert_eq!(meta.image_size, (256, 256));
        let r = meta.region("idle_0").unwrap();
        assert_eq!(
            *r,
            AtlasRegion {
                x: 0,
                y: 0,
                w: 32,
                h: 32
            }
        );
    }

    #[test]
    fn parses_texturepacker_minimal() {
        let meta = parse_atlas_meta(TEXTUREPACKER_HASH_JSON.as_bytes()).unwrap();
        assert_eq!(meta.image_filename, "tileset.png");
        assert_eq!(meta.regions.len(), 2);
        assert!(meta.region("tile_grass").is_some());
        assert!(meta.region("tile_water").is_some());
    }

    #[test]
    fn region_uv_is_normalized_against_image_size() {
        let meta = parse_atlas_meta(TEXTUREPACKER_HASH_JSON.as_bytes()).unwrap();
        let r = meta.region("tile_grass").unwrap();
        let [u0, v0, u1, v1] = meta.region_uv(r);
        // 16/128 = 0.125, (16+16)/128 = 0.25
        assert!((u0 - 16.0 / 128.0).abs() < 1e-6);
        assert!((v0 - 32.0 / 128.0).abs() < 1e-6);
        assert!((u1 - 32.0 / 128.0).abs() < 1e-6);
        assert!((v1 - 48.0 / 128.0).abs() < 1e-6);
    }

    #[test]
    fn missing_image_field_errors() {
        let bad = r#"{
          "frames": {},
          "meta": { "image": "", "size": { "w": 1, "h": 1 } }
        }"#;
        let err = parse_atlas_meta(bad.as_bytes()).unwrap_err();
        assert!(matches!(err, AtlasMetaError::MissingImageFilename));
    }

    #[test]
    fn zero_image_size_errors() {
        let bad = r#"{
          "frames": {},
          "meta": { "image": "x.png", "size": { "w": 0, "h": 0 } }
        }"#;
        let err = parse_atlas_meta(bad.as_bytes()).unwrap_err();
        assert!(matches!(err, AtlasMetaError::InvalidImageSize));
    }

    #[test]
    fn region_outside_image_errors() {
        let bad = r#"{
          "frames": {
            "huge": { "frame": { "x": 100, "y": 100, "w": 50, "h": 50 } }
          },
          "meta": { "image": "x.png", "size": { "w": 64, "h": 64 } }
        }"#;
        let err = parse_atlas_meta(bad.as_bytes()).unwrap_err();
        assert!(matches!(err, AtlasMetaError::RegionOutsideImage { ref name } if name == "huge"));
    }

    #[test]
    fn array_variant_surfaces_as_json_error() {
        // Aseprite's Array variant has `frames` as an array, not an
        // object. Our root expects an object via BTreeMap, so serde_
        // json reports a type mismatch — surfaced as `Json` for now,
        // which is enough to point the user at re-exporting in Hash
        // mode. A future polish pass could detect this in advance and
        // return `UnsupportedFramesShape` instead.
        let bad = r#"{
          "frames": [
            { "frame": { "x": 0, "y": 0, "w": 1, "h": 1 } }
          ],
          "meta": { "image": "x.png", "size": { "w": 1, "h": 1 } }
        }"#;
        let err = parse_atlas_meta(bad.as_bytes()).unwrap_err();
        assert!(matches!(err, AtlasMetaError::Json(_)));
    }

    #[test]
    fn malformed_json_errors() {
        let err = parse_atlas_meta(b"not json").unwrap_err();
        assert!(matches!(err, AtlasMetaError::Json(_)));
    }

    #[test]
    fn empty_frames_object_is_valid_just_unusable() {
        // An atlas JSON with no frames isn't an error per se — it just
        // produces an `AtlasMeta` whose lookup returns None for any
        // name. The renderer's batching layer skips zero-region atlases
        // gracefully.
        let bytes = r#"{
          "frames": {},
          "meta": { "image": "empty.png", "size": { "w": 16, "h": 16 } }
        }"#;
        let meta = parse_atlas_meta(bytes.as_bytes()).unwrap();
        assert_eq!(meta.regions.len(), 0);
        assert!(meta.region("anything").is_none());
    }
}
