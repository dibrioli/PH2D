use ph2d_color::SrgbRgba;
use ph2d_imageio::{
    BlendMode, ColorProfile, DecodedImage, Error, ExportFormat, ExportOpts, ImageBuffer,
    ImageExporter, Layer, LayerKind, LayerStack,
};
use std::io::{Cursor, Write};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

use crate::blend::write_blend_mode;

/// ORA spec version emitted in `stack.xml`. Audit nova M-N4
/// (2026-05-26): was `"0.0.6"` but openraster.org/baseline declares
/// `"0.0.5"`; strict consumers (older Krita / MyPaint) can reject
/// a higher version. Downgrade to the documented spec. Const exists
/// so writer + pin test agree.
const ORA_SPEC_VERSION: &str = "0.0.5";

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
pub(crate) fn write_stack_xml(stack: &LayerStack) -> Result<String, Error> {
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
pub(crate) fn xml_escape(s: &str) -> String {
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
