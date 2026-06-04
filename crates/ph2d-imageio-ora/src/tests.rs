use super::*;
use crate::blend::{parse_blend_mode, write_blend_mode};
use crate::export::{write_stack_xml, xml_escape};
use crate::import::ZIP_MAGIC;
use ph2d_color::SrgbRgba;
use ph2d_imageio::{
    BlendMode, ColorProfile, DecodedImage, Error, ExportFormat, ExportOpts, ImageBuffer,
    ImageExporter, ImageImporter, ImportOpts, Layer, LayerKind, LayerStack, MagicHint, MagicMatch,
};
use std::io::{Cursor, Write};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

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
        xml.push_str(r#"    <stack composite-op="svg:src-over" opacity="1" visibility="visible">"#);
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
