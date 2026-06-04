use ph2d_color::SrgbRgba;
use ph2d_imageio::{
    ColorProfile, DecodedImage, Error, ImageBuffer, ImageImporter, ImportOpts, Layer, LayerKind,
    LayerStack, MAX_RASTER_DIMENSION, MagicHint, MagicMatch,
};
use std::io::{Cursor, Read};
use zip::ZipArchive;

use crate::blend::parse_blend_mode;

/// ORA import driver.
pub struct OraImporter;

/// ZIP local-file-header magic.
pub(crate) const ZIP_MAGIC: [u8; 4] = [b'P', b'K', 0x03, 0x04];

impl ImageImporter for OraImporter {
    fn supports(&self, hint: MagicHint<'_>) -> MagicMatch {
        // Audit W2.T6 H-1 (2026-05-26): ZIP magic `PK\x03\x04` is
        // generic — KRA, ODT, EPUB, JAR, .docx all start with it.
        // Claiming `Strong` on bare ZIP magic meant ORA was stealing
        // dispatch from every other ZIP-container format the registry
        // would later host (.kra W3+ candidate, even our own
        // .ph2d-native is a postcard container, NOT ZIP — false alarm
        // was for KRA-like formats). Fix: claim only `Weak` on ZIP
        // magic + Weak on extension. A future `peek_mimetype()` helper
        // could promote to Strong, but until 2+ ZIP-container importers
        // exist, the simple downgrade is enough.
        match hint {
            MagicHint::Bytes(b) if b.starts_with(&ZIP_MAGIC) => MagicMatch::Weak,
            MagicHint::Bytes(_) => MagicMatch::None,
            MagicHint::Extension(ext) if ext.eq_ignore_ascii_case("ora") => MagicMatch::Weak,
            MagicHint::Extension(_) => MagicMatch::None,
        }
    }

    fn import(&self, src: &[u8], _opts: &ImportOpts) -> Result<DecodedImage, Error> {
        if src.is_empty() {
            return Err(Error::Truncated);
        }
        let cursor = Cursor::new(src);
        let mut archive = ZipArchive::new(cursor)
            .map_err(|e| Error::from_decoder_message(format!("ORA ZIP open: {e}")))?;

        // Audit-10 Lens Z HIGH #2 (2026-05-26): cap entry count. A
        // hostile ZIP with 8M false entries inflates the central
        // directory metadata to ~1.5 GiB before the mimetype check.
        if archive.len() > ph2d_imageio::MAX_ARCHIVE_ENTRIES {
            return Err(Error::Decode(format!(
                "ORA archive has {} entries (> MAX_ARCHIVE_ENTRIES={}); \
                 refuse to walk further (DoS defence)",
                archive.len(),
                ph2d_imageio::MAX_ARCHIVE_ENTRIES
            )));
        }

        // Validate mimetype.
        let mime = read_zip_text(&mut archive, "mimetype")?;
        if mime.trim() != "image/openraster" {
            return Err(Error::Decode(format!(
                "ORA: mimetype mismatch (got {mime:?})"
            )));
        }

        // Parse stack.xml.
        let stack_xml = read_zip_text(&mut archive, "stack.xml")?;
        let doc = roxmltree::Document::parse(&stack_xml)
            .map_err(|e| Error::from_decoder_message(format!("ORA stack.xml parse: {e}")))?;
        let root = doc.root_element();
        if root.tag_name().name() != "image" {
            return Err(Error::Decode(
                "ORA stack.xml: root element must be <image>".into(),
            ));
        }

        let canvas_width = parse_dim_attr(&root, "w")?;
        let canvas_height = parse_dim_attr(&root, "h")?;
        if canvas_width > MAX_RASTER_DIMENSION || canvas_height > MAX_RASTER_DIMENSION {
            return Err(Error::DimensionExceedsLimit);
        }
        if canvas_width == 0 || canvas_height == 0 {
            return Err(Error::Decode(format!(
                "ORA stack.xml: zero-sized canvas {canvas_width}×{canvas_height}"
            )));
        }

        let stack_node = root
            .children()
            .find(|n| n.is_element() && n.tag_name().name() == "stack")
            .ok_or_else(|| Error::Decode("ORA stack.xml: missing root <stack>".into()))?;
        // Audit-9 Lens T CRITICAL T-#1 + Lens S H-S1 + Lens T MEDIUM T-#2
        // (2026-05-26): bound recursion depth + total layer count
        // BEFORE walking. Hostile stack.xml with 50K nested <stack>
        // would stack-overflow the thread (uncatchable SIGSEGV).
        // 4096 layers with duplicate src= refs would re-decompress
        // the same PNG amplifying memory.
        let mut total_layers: usize = 0;
        let layers = parse_stack(&stack_node, &mut archive, 0, &mut total_layers)?;

        Ok(DecodedImage::Layered(LayerStack {
            version: 1,
            canvas_width,
            canvas_height,
            layers,
            // ORA spec assumes whole-document sRGB; future per-layer
            // profile work is out of scope (W2+ amendment if a real
            // file appears with non-sRGB ORA).
            color_profile: ColorProfile::Srgb,
        }))
    }
}

fn parse_dim_attr(node: &roxmltree::Node, key: &str) -> Result<u32, Error> {
    node.attribute(key)
        .and_then(|s| s.parse::<u32>().ok())
        .ok_or_else(|| Error::Decode(format!("ORA stack.xml: missing or invalid {key} attr")))
}

fn read_zip_text(archive: &mut ZipArchive<Cursor<&[u8]>>, name: &str) -> Result<String, Error> {
    let file = archive
        .by_name(name)
        .map_err(|e| Error::Decode(format!("ORA missing `{name}`: {e}")))?;
    // Audit-10 Lens Z MEDIUM #3 (2026-05-26): cap via `Read::take` so
    // a hostile entry declaring `uncompressed_size = u64::MAX` of
    // whitespace doesn't blow `String::with_capacity` past
    // `MAX_ARCHIVE_TEXT_BYTES`. The cap covers `stack.xml` (the only
    // text entry we read) — bytes entries (PNG layers) use the
    // implicit cap from `MAX_RASTER_DIMENSION² × 4`.
    let mut bounded = file.take(ph2d_imageio::MAX_ARCHIVE_TEXT_BYTES);
    let mut buf = String::new();
    bounded
        .read_to_string(&mut buf)
        .map_err(|e| Error::Decode(format!("ORA read `{name}`: {e}")))?;
    Ok(buf)
}

fn read_zip_bytes(archive: &mut ZipArchive<Cursor<&[u8]>>, name: &str) -> Result<Vec<u8>, Error> {
    let mut file = archive
        .by_name(name)
        .map_err(|e| Error::Decode(format!("ORA missing `{name}`: {e}")))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .map_err(|e| Error::Decode(format!("ORA read `{name}`: {e}")))?;
    Ok(buf)
}

fn decode_layer_png(bytes: &[u8]) -> Result<ImageBuffer<SrgbRgba>, Error> {
    let img = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .map_err(|e| Error::Decode(format!("ORA layer PNG decode: {e}")))?;
    let (width, height) = (img.width(), img.height());
    if width > MAX_RASTER_DIMENSION || height > MAX_RASTER_DIMENSION {
        return Err(Error::DimensionExceedsLimit);
    }
    let rgba = img.to_rgba8();
    let pixels: Vec<SrgbRgba> = rgba
        .chunks_exact(4)
        .map(|c| SrgbRgba([c[0], c[1], c[2], c[3]]))
        .collect();
    Ok(ImageBuffer {
        width,
        height,
        pixels,
        color_profile: ColorProfile::Srgb,
    })
}

/// Recursively parse a `<stack>` node. ORA lists children TOP-FIRST
/// (consistent with SVG); our [`LayerStack::layers`] convention is
/// bottom-up. Reverse on the way out.
///
/// Audit-9 Lens T CRITICAL T-#1 (2026-05-26): `depth` parameter caps
/// nested `<stack>` recursion at [`MAX_LAYER_DEPTH`] to defend against
/// stack-overflow DoS (uncatchable SIGSEGV). `total_layers` parameter
/// caps cumulative layer count at [`MAX_LAYER_COUNT`] to defend
/// against read-amplification via duplicate `src=` refs.
fn parse_stack(
    node: &roxmltree::Node,
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    depth: usize,
    total_layers: &mut usize,
) -> Result<Vec<Layer>, Error> {
    if depth > ph2d_imageio::MAX_LAYER_DEPTH {
        return Err(Error::Decode(format!(
            "ORA <stack> nesting exceeds MAX_LAYER_DEPTH={} \
             (DoS defence against stack overflow)",
            ph2d_imageio::MAX_LAYER_DEPTH
        )));
    }
    let mut top_first: Vec<Layer> = Vec::new();
    for child in node.children().filter(|n| n.is_element()) {
        *total_layers = total_layers.saturating_add(1);
        if *total_layers > ph2d_imageio::MAX_LAYER_COUNT {
            return Err(Error::Decode(format!(
                "ORA layer count exceeds MAX_LAYER_COUNT={} \
                 (DoS defence against read amplification)",
                ph2d_imageio::MAX_LAYER_COUNT
            )));
        }
        let name = child.attribute("name").unwrap_or("").to_string();
        // Audit-10 Lens Z CRITICAL #1 (2026-05-26): reject `NaN` and
        // `±Inf` in opacity — `f32::from_str` accepts them, and a
        // NaN propagated through the compositor poisons the entire
        // render target (and persists into `.ph2d-native` save →
        // multi-session poisoning). Clamp finite values to [0, 1]
        // per Layer::opacity contract.
        let opacity: f32 = child
            .attribute("opacity")
            .and_then(|s| s.parse::<f32>().ok())
            .filter(|v| v.is_finite())
            .map(|v| v.clamp(0.0, 1.0))
            .unwrap_or(1.0);
        let visible = child
            .attribute("visibility")
            .map(|s| s == "visible")
            .unwrap_or(true);
        let composite = child.attribute("composite-op").unwrap_or("svg:src-over");
        let blend_mode = parse_blend_mode(composite);

        match child.tag_name().name() {
            "layer" => {
                let src_path = child
                    .attribute("src")
                    .ok_or_else(|| Error::Decode("ORA <layer> missing src= attr".into()))?;
                // Audit-9 Lens T MEDIUM T-#2 (2026-05-26): reject `src=`
                // containing `..` or backslash (zip-slip via XML — we
                // don't write to disk, but echoing the path back in
                // error messages would be a path-existence oracle).
                if src_path.contains("..") || src_path.contains('\\') {
                    return Err(Error::Decode(
                        "ORA <layer src=>: path traversal segments rejected".into(),
                    ));
                }
                let png_bytes = read_zip_bytes(archive, src_path)?;
                let pixels = decode_layer_png(&png_bytes)?;
                top_first.push(Layer {
                    version: 1,
                    name,
                    kind: LayerKind::Pixel,
                    pixels: Some(pixels),
                    opacity,
                    blend_mode,
                    visible,
                    mask: None,
                    effects: Vec::new(),
                    color_profile: None,
                });
            }
            "stack" => {
                let children = parse_stack(&child, archive, depth + 1, total_layers)?;
                top_first.push(Layer {
                    version: 1,
                    name,
                    kind: LayerKind::Group { children },
                    pixels: None,
                    opacity,
                    blend_mode,
                    visible,
                    mask: None,
                    effects: Vec::new(),
                    color_profile: None,
                });
            }
            // Ignore unknown elements per ORA spec extensibility rules.
            _ => {}
        }
    }
    top_first.reverse();
    Ok(top_first)
}
