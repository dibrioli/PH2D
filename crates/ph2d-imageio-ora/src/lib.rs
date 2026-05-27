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

impl ImageImporter for OraImporter {
    fn supports(&self, hint: MagicHint<'_>) -> MagicMatch {
        match hint {
            MagicHint::Bytes(b) if b.starts_with(&ZIP_MAGIC) => MagicMatch::Strong,
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
        let mut archive =
            ZipArchive::new(cursor).map_err(|e| Error::Decode(format!("ORA ZIP open: {e}")))?;

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
            .map_err(|e| Error::Decode(format!("ORA stack.xml parse: {e}")))?;
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
        let layers = parse_stack(&stack_node, &mut archive)?;

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
    let mut file = archive
        .by_name(name)
        .map_err(|e| Error::Decode(format!("ORA missing `{name}`: {e}")))?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)
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
fn parse_stack(
    node: &roxmltree::Node,
    archive: &mut ZipArchive<Cursor<&[u8]>>,
) -> Result<Vec<Layer>, Error> {
    let mut top_first: Vec<Layer> = Vec::new();
    for child in node.children().filter(|n| n.is_element()) {
        let name = child.attribute("name").unwrap_or("").to_string();
        let opacity: f32 = child
            .attribute("opacity")
            .and_then(|s| s.parse().ok())
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
                let children = parse_stack(&child, archive)?;
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
        let mut pixel_layers: Vec<(usize, ImageBuffer<SrgbRgba>)> = Vec::new();
        let mut next_idx = 0_usize;
        collect_pixel_layers(&stack.layers, &mut next_idx, &mut pixel_layers);

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
) {
    // TOP-FIRST: our LayerStack is bottom-up, ORA paints top-first.
    for layer in layers.iter().rev() {
        match (&layer.kind, &layer.pixels) {
            (LayerKind::Pixel, Some(buf)) => {
                out.push((*next_idx, buf.clone()));
                *next_idx += 1;
            }
            (LayerKind::Group { children }, _) => {
                collect_pixel_layers(children, next_idx, out);
            }
            // Adjustment / Text / Smart and pixel-less Pixel layers
            // skip (no PNG to emit). ORA spec doesn't model these
            // kinds — round-trip via ORA loses semantic info; .ph2d
            // preserves.
            _ => {}
        }
    }
}

/// Hand-write the `stack.xml` document. ORA's XML is small enough
/// that pulling a second XML dep (quick-xml) isn't worth it.
fn write_stack_xml(stack: &LayerStack) -> Result<String, Error> {
    let mut s = String::new();
    s.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    s.push('\n');
    s.push_str(&format!(
        r#"<image w="{}" h="{}" version="0.0.6">"#,
        stack.canvas_width, stack.canvas_height
    ));
    s.push('\n');
    s.push_str("  <stack>");
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

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
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
    fn supports_recognizes_zip_magic_strong() {
        let imp = OraImporter;
        assert_eq!(
            imp.supports(MagicHint::Bytes(&ZIP_MAGIC)),
            MagicMatch::Strong
        );
        let mut padded = ZIP_MAGIC.to_vec();
        padded.extend_from_slice(&[0; 32]);
        assert_eq!(imp.supports(MagicHint::Bytes(&padded)), MagicMatch::Strong);
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
