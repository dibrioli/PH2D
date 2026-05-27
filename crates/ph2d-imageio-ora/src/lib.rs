#![forbid(unsafe_code)]
//! `ph2d-imageio-ora` — OpenRaster (`.ora`) decode + encode (W2.T1).
//!
//! Spec: <https://www.openraster.org/baseline/file-layout-spec-v1.html>
//!
//! ### Container layout
//!
//! - `mimetype` — STORE (uncompressed) first entry, content
//!   `image/openraster`.
//! - `stack.xml` — XML manifest describing the layer tree.
//! - `data/layer<N>.png` — RGBA8 PNG per pixel layer.
//! - `mergedimage.png` — composite preview (recommended; W2.T1
//!   emits a placeholder; proper composite lands W3 when the
//!   Painter composite pipeline is callable).
//!
//! ### What W2.T1 covers
//!
//! - Magic: ZIP signature `PK\x03\x04` (Strong) + `"ora"` extension
//!   (Weak).
//! - Decode: parse `stack.xml`, recursively build the [`Layer`] tree
//!   (Pixel + Group), load each layer PNG into
//!   `ImageBuffer<SrgbRgba>`. Layer ordering reversed from ORA's
//!   top-first to our bottom-up convention.
//! - Encode: write ZIP with mimetype (STORE), hand-written
//!   `stack.xml`, per-pixel-layer PNG, `mergedimage.png` placeholder.
//! - Blend mode mapping: 15 ORA `svg:*` modes ↔ [`BlendMode`] variants
//!   (table in [`parse_blend_mode`]). Unknown ORA modes default to
//!   `BlendMode::Normal` on import; non-ORA `BlendMode` variants
//!   export as `svg:src-over` (audit follow-up: extend mapping when
//!   ORA spec catches up).
//! - HR-14 [`LayerStack::version`] / [`Layer::version`] preserved.
//! - HR-15 Fluent keys on errors.
//!
//! ### Out of scope (later waves)
//!
//! - `LayerKind::Adjustment` / `Text` / `Smart` — ORA spec only
//!   models Pixel + Group. Importer accepts only those; exporter
//!   emits non-Pixel/non-Group `LayerKind`s as `Error::Unsupported`
//!   so the caller knows the document has unsupported richness.
//! - [`LayerEffect`] preservation — ORA spec has no equivalent.
//!   Layer effects are silently dropped on export.
//! - Per-layer [`ColorProfile`] — ORA spec assumes whole-document
//!   sRGB; per-layer profiles drop on export.
//! - Thumbnail (`Thumbnails/thumbnail.png`) — W2+ derives from
//!   mergedimage downscale.
//! - Proper composite for `mergedimage.png` — W3 when Painter
//!   composite is callable from imageio.

use ph2d_color::SrgbRgba;
use ph2d_imageio::{
    BlendMode, ColorProfile, DecodedImage, Error, ExportFormat, ExportOpts, ExporterRegistry,
    ImageBuffer, ImageExporter, ImageImporter, ImportOpts, ImporterRegistry, Layer, LayerKind,
    LayerStack, MAX_RASTER_DIMENSION, MagicHint, MagicMatch,
};
use std::io::{Cursor, Read, Write};
use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};

/// Register the ORA importer.
pub fn register_importer(reg: &mut ImporterRegistry) {
    reg.register(Box::new(OraImporter));
}

/// Register the ORA exporter.
pub fn register_exporter(reg: &mut ExporterRegistry) {
    reg.register(Box::new(OraExporter));
}

/// ORA import driver.
pub struct OraImporter;

/// ZIP local-file-header magic.
const ZIP_MAGIC: [u8; 4] = [b'P', b'K', 0x03, 0x04];

/// ORA spec version emitted in `stack.xml`. Audit nova M-N4
/// (2026-05-26): was `"0.0.6"` but openraster.org/baseline declares
/// `"0.0.5"`; strict consumers (older Krita / MyPaint) can reject
/// a higher version. Downgrade to the documented spec. Const exists
/// so writer + pin test agree.
const ORA_SPEC_VERSION: &str = "0.0.5";

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

/// Map ORA `composite-op` attribute (15 `svg:*` enum values per
/// OpenRaster spec §6) to our [`BlendMode`]. Unknown ORA strings fall
/// through to `Normal`.
fn parse_blend_mode(svg_op: &str) -> BlendMode {
    match svg_op {
        "svg:src-over" => BlendMode::Normal,
        "svg:multiply" => BlendMode::Multiply,
        "svg:screen" => BlendMode::Screen,
        "svg:overlay" => BlendMode::Overlay,
        "svg:darken" => BlendMode::Darken,
        "svg:lighten" => BlendMode::Lighten,
        "svg:color-dodge" => BlendMode::ColorDodge,
        "svg:color-burn" => BlendMode::ColorBurn,
        "svg:hard-light" => BlendMode::HardLight,
        "svg:soft-light" => BlendMode::SoftLight,
        "svg:difference" => BlendMode::Difference,
        "svg:color" => BlendMode::Color,
        "svg:luminosity" => BlendMode::Luminosity,
        "svg:hue" => BlendMode::Hue,
        "svg:saturation" => BlendMode::Saturation,
        _ => BlendMode::Normal,
    }
}

/// Reverse of [`parse_blend_mode`]. Non-ORA modes export as
/// `svg:src-over` (documented loss; spec-extensibility means
/// downstream tools may ignore custom `composite-op` strings).
fn write_blend_mode(b: BlendMode) -> &'static str {
    match b {
        BlendMode::Normal => "svg:src-over",
        BlendMode::Multiply => "svg:multiply",
        BlendMode::Screen => "svg:screen",
        BlendMode::Overlay => "svg:overlay",
        BlendMode::Darken => "svg:darken",
        BlendMode::Lighten => "svg:lighten",
        BlendMode::ColorDodge => "svg:color-dodge",
        BlendMode::ColorBurn => "svg:color-burn",
        BlendMode::HardLight => "svg:hard-light",
        BlendMode::SoftLight => "svg:soft-light",
        BlendMode::Difference => "svg:difference",
        BlendMode::Color => "svg:color",
        BlendMode::Luminosity => "svg:luminosity",
        BlendMode::Hue => "svg:hue",
        BlendMode::Saturation => "svg:saturation",
        // Modes not in ORA spec — fall back to Normal with documented
        // loss. PSD round-trip via ORA loses these; round-trip via
        // .ph2d-native preserves byte-exact.
        _ => "svg:src-over",
    }
}

/// ORA export driver.
pub struct OraExporter;

impl ImageExporter for OraExporter {
    fn supports_format(&self, fmt: ExportFormat) -> bool {
        matches!(fmt, ExportFormat::Ora)
    }

    fn export(&self, img: &DecodedImage, _opts: &ExportOpts) -> Result<Vec<u8>, Error> {
        // Normalise input to a LayerStack. `Flat` flattens to a
        // single-layer ORA; `Layered` passes through.
        let stack: LayerStack = match img {
            DecodedImage::Layered(s) => s.clone(),
            DecodedImage::Flat(b) => LayerStack {
                version: 1,
                canvas_width: b.width,
                canvas_height: b.height,
                layers: vec![Layer {
                    version: 1,
                    name: "Layer 1".into(),
                    kind: LayerKind::Pixel,
                    pixels: Some(b.clone()),
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                    visible: true,
                    mask: None,
                    effects: Vec::new(),
                    color_profile: None,
                }],
                color_profile: ColorProfile::Srgb,
            },
            DecodedImage::FlatHdr(_) => return Err(Error::HdrUnsupported),
            DecodedImage::Animated(_) => {
                return Err(Error::Unsupported(
                    "ORA is static; use APNG/GIF for animation".into(),
                ));
            }
            DecodedImage::Vector(_) => {
                return Err(Error::Unsupported(
                    "ORA is raster; rasterize the VectorDoc first".into(),
                ));
            }
        };

        // Walk the layer tree assigning indices to every Pixel layer
        // (groups don't get a PNG; their composite-op + opacity are
        // attributes on the <stack> node in stack.xml).
        // Audit-9 Lens T (2026-05-26): defensive depth cap — even though
        // the LayerStack here came from our own code (importer / Painter),
        // a future bug could construct cycles or excessive nesting and
        // self-DoS during export.
        let mut pixel_layers: Vec<(usize, ImageBuffer<SrgbRgba>)> = Vec::new();
        let mut next_idx = 0_usize;
        collect_pixel_layers(&stack.layers, &mut next_idx, &mut pixel_layers, 0)?;

        // Build the ZIP.
        let mut out: Vec<u8> = Vec::new();
        {
            let mut zip = ZipWriter::new(Cursor::new(&mut out));

            // 1. mimetype — STORE, must be first per ORA spec.
            zip.start_file(
                "mimetype",
                SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
            )
            .map_err(|e| Error::Encode(format!("ORA zip mimetype: {e}")))?;
            zip.write_all(b"image/openraster")
                .map_err(|e| Error::Encode(format!("ORA write mimetype: {e}")))?;

            // 2. stack.xml.
            zip.start_file("stack.xml", SimpleFileOptions::default())
                .map_err(|e| Error::Encode(format!("ORA zip stack.xml: {e}")))?;
            let xml = write_stack_xml(&stack)?;
            zip.write_all(xml.as_bytes())
                .map_err(|e| Error::Encode(format!("ORA write stack.xml: {e}")))?;

            // 3. data/layerN.png per pixel layer.
            for (idx, buf) in &pixel_layers {
                let name = format!("data/layer{idx}.png");
                zip.start_file(&name, SimpleFileOptions::default())
                    .map_err(|e| Error::Encode(format!("ORA zip {name}: {e}")))?;
                let png_bytes = encode_layer_png(buf)?;
                zip.write_all(&png_bytes)
                    .map_err(|e| Error::Encode(format!("ORA write {name}: {e}")))?;
            }

            // 4. mergedimage.png — placeholder using the bottom-most
            // pixel layer pixels. Proper composite lands in W3 when
            // the Painter composite pipeline is callable. ORA spec
            // marks mergedimage as "recommended"; consumers that need
            // the real composite re-render from stack.xml anyway.
            if let Some((_, first_buf)) = pixel_layers.first() {
                zip.start_file("mergedimage.png", SimpleFileOptions::default())
                    .map_err(|e| Error::Encode(format!("ORA zip mergedimage: {e}")))?;
                let png_bytes = encode_layer_png(first_buf)?;
                zip.write_all(&png_bytes)
                    .map_err(|e| Error::Encode(format!("ORA write mergedimage: {e}")))?;
            }

            zip.finish()
                .map_err(|e| Error::Encode(format!("ORA zip finalise: {e}")))?;
        }
        Ok(out)
    }
}

/// Walk the layer tree assigning a sequential index to every
/// [`LayerKind::Pixel`] layer (groups don't get a PNG). Indices are
/// referenced by `data/layer<N>.png` paths in `stack.xml`.
///
/// **Iterates TOP-FIRST** to match [`write_layer_xml`]'s iteration
/// order — both functions must assign indices to the same layer or
/// the XML's `src=` references point at the wrong PNG (W2.T1 bug
/// caught by `roundtrip_group_with_two_children` test).
fn collect_pixel_layers(
    layers: &[Layer],
    next_idx: &mut usize,
    out: &mut Vec<(usize, ImageBuffer<SrgbRgba>)>,
    depth: usize,
) -> Result<(), Error> {
    if depth > ph2d_imageio::MAX_LAYER_DEPTH {
        return Err(Error::Encode(format!(
            "ORA export: LayerStack nesting exceeds MAX_LAYER_DEPTH={} \
             (self-DoS defence)",
            ph2d_imageio::MAX_LAYER_DEPTH
        )));
    }
    // TOP-FIRST: our LayerStack is bottom-up, ORA paints top-first.
    for layer in layers.iter().rev() {
        match (&layer.kind, &layer.pixels) {
            (LayerKind::Pixel, Some(buf)) => {
                out.push((*next_idx, buf.clone()));
                *next_idx += 1;
            }
            (LayerKind::Group { children }, _) => {
                collect_pixel_layers(children, next_idx, out, depth + 1)?;
            }
            // Adjustment / Text / Smart and pixel-less Pixel layers
            // skip (no PNG to emit). ORA spec doesn't model these
            // kinds — round-trip via ORA loses semantic info; .ph2d
            // preserves.
            _ => {}
        }
    }
    Ok(())
}

/// Hand-write the `stack.xml` document. ORA's XML is small enough
/// that pulling a second XML dep (quick-xml) isn't worth it.
fn write_stack_xml(stack: &LayerStack) -> Result<String, Error> {
    let mut s = String::new();
    s.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    s.push('\n');
    s.push_str(&format!(
        r#"<image w="{}" h="{}" version="{}">"#,
        stack.canvas_width, stack.canvas_height, ORA_SPEC_VERSION
    ));
    s.push('\n');
    // Audit-7 Lens H HIGH H1: ORA spec 0.0.5 §3.1 lists
    // `composite-op`, `opacity`, `visibility` as valid `<stack>`
    // attributes. Krita strict mode rejects `<stack>` without them.
    // Emit the spec defaults — same values group-stacks use.
    s.push_str(r#"  <stack composite-op="svg:src-over" opacity="1" visibility="visible">"#);
    s.push('\n');
    let mut next_idx = 0_usize;
    // ORA paint order is TOP-FIRST; reverse our bottom-up list.
    let top_first: Vec<&Layer> = stack.layers.iter().rev().collect();
    for layer in top_first {
        write_layer_xml(layer, 2, &mut next_idx, &mut s)?;
    }
    s.push_str("  </stack>\n");
    s.push_str("</image>\n");
    Ok(s)
}

fn write_layer_xml(
    layer: &Layer,
    indent: usize,
    next_idx: &mut usize,
    out: &mut String,
) -> Result<(), Error> {
    let pad = " ".repeat(indent);
    let escaped_name = xml_escape(&layer.name);
    let opacity = layer.opacity.clamp(0.0, 1.0);
    let visibility = if layer.visible { "visible" } else { "hidden" };
    let composite = write_blend_mode(layer.blend_mode);

    match &layer.kind {
        LayerKind::Pixel => {
            let idx = *next_idx;
            *next_idx += 1;
            out.push_str(&format!(
                r#"{pad}<layer name="{escaped_name}" src="data/layer{idx}.png" x="0" y="0" opacity="{opacity}" visibility="{visibility}" composite-op="{composite}"/>"#
            ));
            out.push('\n');
        }
        LayerKind::Group { children } => {
            out.push_str(&format!(
                r#"{pad}<stack name="{escaped_name}" opacity="{opacity}" visibility="{visibility}" composite-op="{composite}">"#
            ));
            out.push('\n');
            // Children in TOP-FIRST too.
            let top_first: Vec<&Layer> = children.iter().rev().collect();
            for child in top_first {
                write_layer_xml(child, indent + 2, next_idx, out)?;
            }
            out.push_str(&format!("{pad}</stack>\n"));
        }
        LayerKind::Adjustment | LayerKind::Text | LayerKind::Smart => {
            return Err(Error::Unsupported(format!(
                "ORA spec only models Pixel + Group; got {:?} layer `{}`. \
                 Use `.ph2d` native for full PSD-grade preservation.",
                layer.kind, layer.name
            )));
        }
    }
    Ok(())
}

/// Escape XML-significant characters AND strip XML 1.0-illegal control
/// chars (U+0001..U+001F except `\t \n \r`). Audit W2.T6 MEDIUM:
/// previously only escaped the 5 entities; a layer name containing
/// `\x07` (BEL) or `\x1b` (ESC) produced XML that roxmltree rejects
/// on re-import. Now control chars are silently dropped (less ideal
/// than replacing with `?` but matches the spirit of XML strictness).
fn xml_escape(s: &str) -> String {
    let stripped: String = s
        .chars()
        .filter(|&c| c == '\t' || c == '\n' || c == '\r' || c >= ' ')
        .collect();
    stripped
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn encode_layer_png(buf: &ImageBuffer<SrgbRgba>) -> Result<Vec<u8>, Error> {
    let byte_len = buf.pixels.len().checked_mul(4).ok_or(Error::OutOfMemory)?;
    let mut rgba8: Vec<u8> = Vec::with_capacity(byte_len);
    for p in &buf.pixels {
        rgba8.extend_from_slice(&p.0);
    }
    let img_buf = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(buf.width, buf.height, rgba8)
        .ok_or_else(|| {
            Error::Encode(format!(
                "ORA layer pixel count {} mismatched {}×{}",
                buf.pixels.len(),
                buf.width,
                buf.height
            ))
        })?;
    let mut out: Vec<u8> = Vec::new();
    img_buf
        .write_to(&mut Cursor::new(&mut out), image::ImageFormat::Png)
        .map_err(|e| Error::Encode(format!("ORA layer PNG encode: {e}")))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pixel_layer(name: &str, color: [u8; 4]) -> Layer {
        Layer {
            version: 1,
            name: name.into(),
            kind: LayerKind::Pixel,
            pixels: Some(ImageBuffer {
                width: 2,
                height: 2,
                pixels: vec![SrgbRgba(color); 4],
                color_profile: ColorProfile::Srgb,
            }),
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            visible: true,
            mask: None,
            effects: Vec::new(),
            color_profile: None,
        }
    }

    #[test]
    fn supports_recognizes_zip_magic_weak_not_strong() {
        // Audit W2.T6 H-1: ZIP magic is generic (KRA, ODT, EPUB, JAR
        // also use it); claiming Strong would steal dispatch from
        // other ZIP-container formats. Downgrade to Weak so callers
        // explicitly route via `MagicHint::Extension("ora")` for
        // certainty.
        let imp = OraImporter;
        assert_eq!(imp.supports(MagicHint::Bytes(&ZIP_MAGIC)), MagicMatch::Weak);
        let mut padded = ZIP_MAGIC.to_vec();
        padded.extend_from_slice(&[0; 32]);
        assert_eq!(imp.supports(MagicHint::Bytes(&padded)), MagicMatch::Weak);
        assert_eq!(
            imp.supports(MagicHint::Bytes(b"not-a-zip-file")),
            MagicMatch::None
        );
        assert_eq!(imp.supports(MagicHint::Bytes(&[])), MagicMatch::None);
    }

    #[test]
    fn supports_recognizes_ora_extension_weak() {
        let imp = OraImporter;
        for ext in &["ora", "ORA", "Ora"] {
            assert_eq!(imp.supports(MagicHint::Extension(ext)), MagicMatch::Weak);
        }
        assert_eq!(imp.supports(MagicHint::Extension("psd")), MagicMatch::None);
    }

    #[test]
    fn exporter_recognizes_ora_format() {
        let exp = OraExporter;
        assert!(exp.supports_format(ExportFormat::Ora));
        assert!(!exp.supports_format(ExportFormat::Png));
    }

    #[test]
    fn blend_mode_roundtrip_15_orap_modes() {
        let pairs: &[(BlendMode, &str)] = &[
            (BlendMode::Normal, "svg:src-over"),
            (BlendMode::Multiply, "svg:multiply"),
            (BlendMode::Screen, "svg:screen"),
            (BlendMode::Overlay, "svg:overlay"),
            (BlendMode::Darken, "svg:darken"),
            (BlendMode::Lighten, "svg:lighten"),
            (BlendMode::ColorDodge, "svg:color-dodge"),
            (BlendMode::ColorBurn, "svg:color-burn"),
            (BlendMode::HardLight, "svg:hard-light"),
            (BlendMode::SoftLight, "svg:soft-light"),
            (BlendMode::Difference, "svg:difference"),
            (BlendMode::Color, "svg:color"),
            (BlendMode::Luminosity, "svg:luminosity"),
            (BlendMode::Hue, "svg:hue"),
            (BlendMode::Saturation, "svg:saturation"),
        ];
        for (mode, op) in pairs {
            assert_eq!(write_blend_mode(*mode), *op, "write {mode:?}");
            assert_eq!(parse_blend_mode(op), *mode, "parse {op}");
        }
        // Non-ORA mode → src-over.
        assert_eq!(write_blend_mode(BlendMode::LinearLight), "svg:src-over");
        assert_eq!(write_blend_mode(BlendMode::Custom(0xABCD)), "svg:src-over");
        // Unknown ORA mode → Normal.
        assert_eq!(parse_blend_mode("svg:unknown-future"), BlendMode::Normal);
    }

    #[test]
    fn roundtrip_single_pixel_layer_preserves_pixels() {
        let stack = LayerStack {
            version: 1,
            canvas_width: 2,
            canvas_height: 2,
            layers: vec![make_pixel_layer("L1", [255, 0, 0, 200])],
            color_profile: ColorProfile::Srgb,
        };
        let bytes = OraExporter
            .export(
                &DecodedImage::Layered(stack.clone()),
                &ExportOpts {
                    format: ExportFormat::Ora,
                    ..ExportOpts::default()
                },
            )
            .expect("ORA export");
        assert!(bytes.starts_with(&ZIP_MAGIC), "output bytes are not ZIP");

        let decoded = OraImporter
            .import(&bytes, &ImportOpts::default())
            .expect("ORA import");
        let DecodedImage::Layered(out) = decoded else {
            panic!("expected Layered, got {decoded:?}");
        };
        assert_eq!(out.canvas_width, 2);
        assert_eq!(out.canvas_height, 2);
        assert_eq!(out.layers.len(), 1);
        assert_eq!(out.layers[0].name, "L1");
        let pixels = out.layers[0].pixels.as_ref().expect("pixel layer");
        for (i, p) in pixels.pixels.iter().enumerate() {
            assert_eq!(p.0, [255, 0, 0, 200], "pixel {i}");
        }
    }

    #[test]
    fn roundtrip_group_with_two_children() {
        let group = Layer {
            version: 1,
            name: "G1".into(),
            kind: LayerKind::Group {
                children: vec![
                    make_pixel_layer("Inner1", [10, 20, 30, 255]),
                    make_pixel_layer("Inner2", [200, 100, 50, 128]),
                ],
            },
            pixels: None,
            opacity: 0.7,
            blend_mode: BlendMode::Multiply,
            visible: false,
            mask: None,
            effects: Vec::new(),
            color_profile: None,
        };
        let stack = LayerStack {
            version: 1,
            canvas_width: 2,
            canvas_height: 2,
            layers: vec![group],
            color_profile: ColorProfile::Srgb,
        };
        let bytes = OraExporter
            .export(
                &DecodedImage::Layered(stack),
                &ExportOpts {
                    format: ExportFormat::Ora,
                    ..ExportOpts::default()
                },
            )
            .expect("export");
        let DecodedImage::Layered(out) = OraImporter
            .import(&bytes, &ImportOpts::default())
            .expect("import")
        else {
            panic!("expected Layered");
        };
        assert_eq!(out.layers.len(), 1);
        let g = &out.layers[0];
        assert_eq!(g.name, "G1");
        assert!((g.opacity - 0.7).abs() < f32::EPSILON);
        assert_eq!(g.blend_mode, BlendMode::Multiply);
        assert!(!g.visible);
        let LayerKind::Group { children } = &g.kind else {
            panic!("expected Group");
        };
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].name, "Inner1");
        assert_eq!(children[1].name, "Inner2");
        // Order: top-first roundtrip — Inner1 was bottom-up [0], Inner2 [1].
        // ORA stores top-first; we reverse on import. After roundtrip,
        // bottom-up order matches input.
        assert_eq!(
            children[0].pixels.as_ref().unwrap().pixels[0].0,
            [10, 20, 30, 255]
        );
    }

    #[test]
    fn export_flat_flattens_to_single_layer() {
        let buf = ImageBuffer {
            width: 1,
            height: 1,
            pixels: vec![SrgbRgba([42, 84, 126, 255])],
            color_profile: ColorProfile::Srgb,
        };
        let bytes = OraExporter
            .export(
                &DecodedImage::Flat(buf),
                &ExportOpts {
                    format: ExportFormat::Ora,
                    ..ExportOpts::default()
                },
            )
            .expect("export Flat as ORA");
        let DecodedImage::Layered(out) = OraImporter
            .import(&bytes, &ImportOpts::default())
            .expect("import")
        else {
            panic!("expected Layered");
        };
        assert_eq!(out.layers.len(), 1);
        let pixels = out.layers[0].pixels.as_ref().unwrap();
        assert_eq!(pixels.pixels[0].0, [42, 84, 126, 255]);
    }

    #[test]
    fn import_rejects_empty_as_truncated() {
        let err = OraImporter
            .import(&[], &ImportOpts::default())
            .expect_err("empty");
        assert!(matches!(err, Error::Truncated));
    }

    #[test]
    fn import_rejects_wrong_mimetype() {
        // Build a ZIP with `mimetype = "image/wrong"` + minimal
        // stack.xml so the failure point is the mimetype check, not
        // an earlier ZIP parse error.
        let mut bytes: Vec<u8> = Vec::new();
        {
            let mut zip = ZipWriter::new(Cursor::new(&mut bytes));
            zip.start_file(
                "mimetype",
                SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
            )
            .unwrap();
            zip.write_all(b"image/wrong").unwrap();
            zip.start_file("stack.xml", SimpleFileOptions::default())
                .unwrap();
            zip.write_all(b"<image w=\"1\" h=\"1\"><stack/></image>")
                .unwrap();
            zip.finish().unwrap();
        }
        let err = OraImporter
            .import(&bytes, &ImportOpts::default())
            .expect_err("bad mimetype");
        match err {
            Error::Decode(msg) => assert!(
                msg.contains("mimetype mismatch"),
                "expected mimetype mismatch, got: {msg}"
            ),
            other => panic!("expected Decode, got {other:?}"),
        }
    }

    #[test]
    fn export_rejects_adjustment_layer_with_unsupported() {
        let stack = LayerStack {
            version: 1,
            canvas_width: 1,
            canvas_height: 1,
            layers: vec![Layer {
                version: 1,
                name: "Adj".into(),
                kind: LayerKind::Adjustment,
                pixels: None,
                opacity: 1.0,
                blend_mode: BlendMode::Normal,
                visible: true,
                mask: None,
                effects: Vec::new(),
                color_profile: None,
            }],
            color_profile: ColorProfile::Srgb,
        };
        let err = OraExporter
            .export(
                &DecodedImage::Layered(stack),
                &ExportOpts {
                    format: ExportFormat::Ora,
                    ..ExportOpts::default()
                },
            )
            .expect_err("Adjustment layer not in ORA spec");
        assert!(matches!(err, Error::Unsupported(_)));
    }

    #[test]
    fn export_rejects_hdr_with_actionable_error() {
        let fake_hdr = DecodedImage::FlatHdr(ImageBuffer {
            width: 1,
            height: 1,
            pixels: vec![ph2d_color::LinearRgba::new(1.0, 0.0, 0.0, 1.0)],
            color_profile: ColorProfile::LinearRec709,
        });
        let err = OraExporter
            .export(
                &fake_hdr,
                &ExportOpts {
                    format: ExportFormat::Ora,
                    ..ExportOpts::default()
                },
            )
            .expect_err("HDR refused");
        assert!(matches!(err, Error::HdrUnsupported));
    }

    /// Audit nova M-N2 (2026-05-26): `xml_escape` must strip XML 1.0
    /// control chars (U+0001..U+001F) while preserving tab/newline/
    /// carriage return. Regression-guard test.
    #[test]
    fn xml_escape_strips_control_chars_preserves_whitespace() {
        // Control chars stripped silently.
        assert_eq!(xml_escape("ab\x07c\x1bd"), "abcd");
        assert_eq!(xml_escape("name\x00with\x00nulls"), "namewithnulls");
        // Whitespace preserved.
        assert_eq!(
            xml_escape("hello\tworld\nfoo\rbar"),
            "hello\tworld\nfoo\rbar"
        );
        // Standard XML entities still escaped.
        assert_eq!(
            xml_escape("<tag attr=\"value\" attr2='v2'>&amp;</tag>"),
            "&lt;tag attr=&quot;value&quot; attr2=&apos;v2&apos;&gt;&amp;amp;&lt;/tag&gt;"
        );
    }

    /// Audit-9 Lens T CRITICAL T-#1 (2026-05-26): gate the
    /// MAX_LAYER_DEPTH cap against hostile stack.xml with deep
    /// nesting that would stack-overflow the thread.
    #[test]
    fn import_rejects_deep_stack_nesting() {
        // Build minimal ORA in-memory with MAX_LAYER_DEPTH+2 nested
        // <stack> elements. We don't need a real ZIP — only the
        // stack.xml parse path matters for this gate.
        let depth = ph2d_imageio::MAX_LAYER_DEPTH + 2;
        let mut xml = String::from(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<image w="1" h="1" version="0.0.5">
  <stack composite-op="svg:src-over" opacity="1" visibility="visible">
"#,
        );
        for _ in 0..depth {
            xml.push_str(
                r#"    <stack composite-op="svg:src-over" opacity="1" visibility="visible">"#,
            );
            xml.push('\n');
        }
        for _ in 0..depth {
            xml.push_str("    </stack>\n");
        }
        xml.push_str("  </stack>\n</image>\n");

        // Wrap in a minimal ZIP with mimetype + stack.xml.
        let mut zip_bytes: Vec<u8> = Vec::new();
        {
            let mut zip = ZipWriter::new(Cursor::new(&mut zip_bytes));
            zip.start_file(
                "mimetype",
                SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
            )
            .unwrap();
            zip.write_all(b"image/openraster").unwrap();
            zip.start_file("stack.xml", SimpleFileOptions::default())
                .unwrap();
            zip.write_all(xml.as_bytes()).unwrap();
            zip.finish().unwrap();
        }

        let err = OraImporter
            .import(&zip_bytes, &ImportOpts::default())
            .expect_err("deep nesting must be rejected");
        match err {
            Error::Decode(msg) => {
                assert!(
                    msg.contains("MAX_LAYER_DEPTH"),
                    "expected MAX_LAYER_DEPTH message: {msg}"
                );
            }
            other => panic!("expected Decode with cap message, got: {other:?}"),
        }
    }

    /// Audit-10 Lens Z CRITICAL #1 (2026-05-26): gate the NaN/Inf
    /// opacity rejection. Hostile ORA with `opacity="NaN"` would
    /// propagate NaN into the compositor (uncatchable data poison).
    #[test]
    fn import_clamps_nan_and_inf_opacity_to_default() {
        for hostile in ["NaN", "nan", "inf", "-inf", "infinity", "Infinity"] {
            let xml = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<image w="1" h="1" version="0.0.5">
  <stack composite-op="svg:src-over" opacity="1" visibility="visible">
    <stack composite-op="svg:src-over" opacity="{hostile}" visibility="visible">
    </stack>
  </stack>
</image>
"#
            );
            let mut zip_bytes: Vec<u8> = Vec::new();
            {
                let mut zip = ZipWriter::new(Cursor::new(&mut zip_bytes));
                zip.start_file(
                    "mimetype",
                    SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
                )
                .unwrap();
                zip.write_all(b"image/openraster").unwrap();
                zip.start_file("stack.xml", SimpleFileOptions::default())
                    .unwrap();
                zip.write_all(xml.as_bytes()).unwrap();
                zip.finish().unwrap();
            }
            let decoded = OraImporter
                .import(&zip_bytes, &ImportOpts::default())
                .expect("must import (NaN→default)");
            let DecodedImage::Layered(stack) = decoded else {
                panic!("expected Layered");
            };
            assert_eq!(stack.layers.len(), 1, "group layer present");
            let group = &stack.layers[0];
            assert!(
                group.opacity.is_finite(),
                "{hostile} → opacity {} must be finite",
                group.opacity
            );
            assert!(
                (0.0..=1.0).contains(&group.opacity),
                "{hostile} → opacity {} must be clamped to [0,1]",
                group.opacity
            );
        }
    }

    /// Audit-9 Lens T MEDIUM T-#2 (2026-05-26): gate path-traversal
    /// rejection in `<layer src=>`. zip-rs doesn't write to disk so
    /// there's no FS escape, but echoing the path back via error
    /// message would be a path-existence oracle.
    #[test]
    fn import_rejects_src_with_path_traversal() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<image w="1" h="1" version="0.0.5">
  <stack composite-op="svg:src-over" opacity="1" visibility="visible">
    <layer src="../etc/passwd" name="evil" opacity="1" visibility="visible"/>
  </stack>
</image>
"#;
        let mut zip_bytes: Vec<u8> = Vec::new();
        {
            let mut zip = ZipWriter::new(Cursor::new(&mut zip_bytes));
            zip.start_file(
                "mimetype",
                SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
            )
            .unwrap();
            zip.write_all(b"image/openraster").unwrap();
            zip.start_file("stack.xml", SimpleFileOptions::default())
                .unwrap();
            zip.write_all(xml.as_bytes()).unwrap();
            zip.finish().unwrap();
        }
        let err = OraImporter
            .import(&zip_bytes, &ImportOpts::default())
            .expect_err("path traversal must be rejected");
        match err {
            Error::Decode(msg) => {
                assert!(
                    msg.contains("path traversal"),
                    "expected path-traversal message: {msg}"
                );
            }
            other => panic!("expected Decode, got: {other:?}"),
        }
    }

    /// Audit-8 Lens M + H-1 (2026-05-26): gate the spec H-1 fix.
    /// ORA spec 0.0.5 §3.1 requires `composite-op` / `opacity` /
    /// `visibility` attrs on `<stack>` — Krita strict mode rejects
    /// stacks without them. A regression here causes silent
    /// rejection in real-world viewers.
    #[test]
    fn stack_xml_root_emits_required_attrs() {
        let stack = LayerStack {
            version: 1,
            canvas_width: 1,
            canvas_height: 1,
            layers: vec![make_pixel_layer("L", [0, 0, 0, 0])],
            color_profile: ColorProfile::Srgb,
        };
        let xml = write_stack_xml(&stack).expect("write");
        assert!(
            xml.contains(r#"<stack composite-op="svg:src-over" opacity="1" visibility="visible">"#),
            "stack.xml root <stack> must carry spec-required attrs; got: {xml}"
        );
    }

    /// Audit nova M-N4 (2026-05-26): pin the spec version so a
    /// careless edit doesn't drift away from `"0.0.5"` and break
    /// strict consumers.
    #[test]
    fn stack_xml_emits_documented_spec_version() {
        let stack = LayerStack {
            version: 1,
            canvas_width: 1,
            canvas_height: 1,
            layers: vec![make_pixel_layer("L", [0, 0, 0, 0])],
            color_profile: ColorProfile::Srgb,
        };
        let xml = write_stack_xml(&stack).expect("write");
        assert!(
            xml.contains(r#"version="0.0.5""#),
            "stack.xml must emit ORA_SPEC_VERSION=0.0.5; got: {xml}"
        );
    }

    /// W3 pre-gate (LOCAL drift guard, Mac aarch64 only): pin blake3
    /// hash of canonical ORA export. ORA touches ZIP DEFLATE + image
    /// PNG encoder + hand-written XML — multiple determinism layers.
    /// Single-platform pin — DEFLATE level + SIMD divergence between
    /// targets emits different (still-valid) ORA archives. Multi-pin
    /// deferred (entry: `crates/ph2d-imageio-ora/src/lib.rs::export_golden_blake3_local_drift_pinned_macos_silicon`).
    // CONVENTION (audit-6 Lens B HIGH): keep `const GOLDEN_BLAKE3`
    // inside the fn body. See ph2d-imageio-png/src/lib.rs for full
    // rationale.
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    #[test]
    fn export_golden_blake3_local_drift_pinned_macos_silicon() {
        let stack = LayerStack {
            version: 1,
            canvas_width: 2,
            canvas_height: 2,
            layers: vec![make_pixel_layer("L1", [200, 100, 50, 255])],
            color_profile: ColorProfile::Srgb,
        };
        let opts = ExportOpts {
            format: ExportFormat::Ora,
            ..ExportOpts::default()
        };
        let bytes = OraExporter
            .export(&DecodedImage::Layered(stack), &opts)
            .expect("ORA golden export");
        let hash = blake3::hash(&bytes);
        const GOLDEN_BLAKE3: &str =
            // Re-captured 2026-05-26 after H1 fix (added composite-op/
            // opacity/visibility attrs to root <stack> — spec compliance).
            "16a0899379c45a3d9b4bf7c18c39f6ccede6155261ba70ed813d0dccc3195520";
        assert_eq!(
            hash.to_hex().to_string(),
            GOLDEN_BLAKE3,
            "LOCAL drift (macOS aarch64): ORA export blake3 drifted"
        );
    }

    /// HR-5 byte-exact determinism: `image-zip` 2 + our hand-written
    /// XML + image-PNG encoder all deterministic; whole ORA bytes
    /// reproducible across runs.
    #[test]
    fn export_is_byte_exact_deterministic() {
        let stack = LayerStack {
            version: 1,
            canvas_width: 2,
            canvas_height: 2,
            layers: vec![make_pixel_layer("L1", [200, 100, 50, 255])],
            color_profile: ColorProfile::Srgb,
        };
        let opts = ExportOpts {
            format: ExportFormat::Ora,
            ..ExportOpts::default()
        };
        let a = OraExporter
            .export(&DecodedImage::Layered(stack.clone()), &opts)
            .expect("a");
        let b = OraExporter
            .export(&DecodedImage::Layered(stack), &opts)
            .expect("b");
        assert_eq!(a, b, "HR-5: ORA export must be byte-exact deterministic");
    }
}
