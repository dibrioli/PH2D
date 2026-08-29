//! Testes de `lib.rs` — movidos para cá em 2026-08-29 para o ficheiro voltar
//! a caber sob o teto SIMPLES de 700 LOC (`architecture_workspace_file_loc_cap`).
//!
//! ⚠️ A entrada de excepção deste ficheiro foi **DELETADA** da `FILE_OVERAGE_OK`, não
//! baixada — é a única direcção em que aquela tabela anda, e o mesmo que já se fez ao
//! `cook.rs` (M2.N1) e ao `paint_text.rs`.

use super::*;

fn fixture_2x2(color: [u8; 4]) -> ImageBuffer<SrgbRgba> {
    ImageBuffer {
        width: 2,
        height: 2,
        pixels: vec![SrgbRgba(color); 4],
        color_profile: ColorProfile::Srgb,
    }
}

/// Build a synthetic PNG chunk: `[len:u32 BE][type:4][data][crc:4]`.
/// CRC is a placeholder (`has_actl_chunk` doesn't verify CRC).
fn fake_chunk(tag: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut out = (data.len() as u32).to_be_bytes().to_vec();
    out.extend_from_slice(tag);
    out.extend_from_slice(data);
    out.extend_from_slice(&[0; 4]); // fake CRC
    out
}

#[test]
fn supports_only_strong_when_actl_chunk_at_chunk_header() {
    let imp = ApngImporter;

    // 1. Plain PNG magic alone (no chunks) — None.
    assert_eq!(
        imp.supports(MagicHint::Bytes(&PNG_MAGIC)),
        MagicMatch::None,
        "plain PNG magic alone must not claim Strong"
    );

    // 2. Plain PNG (magic + IHDR + IDAT — no acTL) — None.
    let mut plain_png = PNG_MAGIC.to_vec();
    plain_png.extend(fake_chunk(b"IHDR", &[0; 13]));
    plain_png.extend(fake_chunk(b"IDAT", &[0; 16]));
    assert_eq!(
        imp.supports(MagicHint::Bytes(&plain_png)),
        MagicMatch::None,
        "PNG without acTL chunk must return None"
    );

    // 3. Real APNG (magic + IHDR + acTL + IDAT) — Strong.
    let mut real_apng = PNG_MAGIC.to_vec();
    real_apng.extend(fake_chunk(b"IHDR", &[0; 13]));
    real_apng.extend(fake_chunk(b"acTL", &[0; 8]));
    real_apng.extend(fake_chunk(b"IDAT", &[0; 16]));
    assert_eq!(
        imp.supports(MagicHint::Bytes(&real_apng)),
        MagicMatch::Strong,
        "PNG with acTL at chunk header → Strong (real APNG)"
    );

    // 4. Audit residual C-1 (2026-05-26): tEXt chunk containing
    // the literal string "acTL" in its DATA must NOT trigger
    // false-positive. This was the gap of the previous
    // substring scan.
    let mut text_with_actl = PNG_MAGIC.to_vec();
    text_with_actl.extend(fake_chunk(b"IHDR", &[0; 13]));
    // tEXt chunk: keyword=null=text. Text mentions "acTL".
    let text_data = b"Comment\0See acTL chunk in APNG spec";
    text_with_actl.extend(fake_chunk(b"tEXt", text_data));
    text_with_actl.extend(fake_chunk(b"IDAT", &[0; 16]));
    assert_eq!(
        imp.supports(MagicHint::Bytes(&text_with_actl)),
        MagicMatch::None,
        "PNG with 'acTL' as tEXt content must NOT false-positive"
    );

    // 5. Non-PNG magic → None regardless.
    assert_eq!(
        imp.supports(MagicHint::Bytes(b"not-a-png")),
        MagicMatch::None
    );
}

#[test]
fn supports_recognizes_apng_extension_weak() {
    let imp = ApngImporter;
    for ext in &["apng", "APNG", "Apng"] {
        assert_eq!(imp.supports(MagicHint::Extension(ext)), MagicMatch::Weak);
    }
    // png/jpeg/gif: APNG crate does NOT claim them.
    assert_eq!(imp.supports(MagicHint::Extension("png")), MagicMatch::None);
    assert_eq!(imp.supports(MagicHint::Extension("gif")), MagicMatch::None);
}

#[test]
fn exporter_recognizes_apng_format() {
    let exp = ApngExporter;
    assert!(exp.supports_format(ExportFormat::Apng));
    assert!(!exp.supports_format(ExportFormat::Png));
}

/// Plain PNG (no acTL) decodes to Flat via the APNG path.
#[test]
fn import_of_plain_png_yields_flat() {
    // Encode a 2×2 PNG using image crate, then decode via our APNG
    // importer.
    let img = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
        2,
        2,
        vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 128, 128, 128, 200,
        ],
    )
    .expect("rgba8");
    let mut bytes: Vec<u8> = Vec::new();
    image::DynamicImage::ImageRgba8(img)
        .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
        .expect("encode plain PNG");
    let decoded = ApngImporter
        .import(&bytes, &ImportOpts::default())
        .expect("import plain PNG via APNG path");
    let DecodedImage::Flat(out) = decoded else {
        panic!("plain PNG must be Flat through APNG importer");
    };
    assert_eq!(out.width, 2);
    assert_eq!(out.height, 2);
    assert_eq!(out.pixels.len(), 4);
    assert_eq!(out.pixels[0].0, [255, 0, 0, 255]);
}

/// Single-frame round-trip: Flat → PNG bytes → Flat (W2.T3 encoder
/// only supports Flat; this is the workhorse path).
#[test]
fn roundtrip_flat_single_frame_bit_exact() {
    let original = fixture_2x2([200, 100, 50, 255]);
    let bytes = ApngExporter
        .export(
            &DecodedImage::Flat(original.clone()),
            &ExportOpts {
                format: ExportFormat::Apng,
                ..ExportOpts::default()
            },
        )
        .expect("APNG single-frame export");
    assert!(
        bytes.starts_with(&PNG_MAGIC),
        "output bytes are not PNG/APNG"
    );
    let DecodedImage::Flat(out) = ApngImporter
        .import(&bytes, &ImportOpts::default())
        .expect("APNG import single-frame")
    else {
        panic!("expected Flat");
    };
    for (i, p) in out.pixels.iter().enumerate() {
        assert_eq!(p.0, original.pixels[i].0, "pixel {i}");
    }
}

#[test]
fn export_rejects_animated_with_actionable_unsupported() {
    // Audit-aware: the W2.T3 encoder cannot ship multi-frame APNG.
    // Caller must use .ph2d-native (full fidelity) or wait for W3+.
    let anim = DecodedImage::Animated(vec![
        AnimFrame {
            image: fixture_2x2([255, 0, 0, 255]),
            delay_ms: 100,
            offset_xy: (0, 0),
            dispose_op: DisposeOp::None,
            blend_op: AnimBlendOp::Source,
            transparent_index: None,
        },
        AnimFrame {
            image: fixture_2x2([0, 255, 0, 255]),
            delay_ms: 100,
            offset_xy: (0, 0),
            dispose_op: DisposeOp::None,
            blend_op: AnimBlendOp::Source,
            transparent_index: None,
        },
    ]);
    let err = ApngExporter
        .export(
            &anim,
            &ExportOpts {
                format: ExportFormat::Apng,
                ..ExportOpts::default()
            },
        )
        .expect_err("multi-frame APNG must be Unsupported in W2.T3");
    match err {
        Error::Unsupported(msg) => assert!(
            msg.contains("multi-frame"),
            "expected multi-frame note, got: {msg}"
        ),
        other => panic!("expected Unsupported, got {other:?}"),
    }
}

#[test]
fn export_rejects_hdr_with_actionable_error() {
    let fake_hdr = DecodedImage::FlatHdr(ImageBuffer {
        width: 1,
        height: 1,
        pixels: vec![ph2d_color::LinearRgba::new(1.0, 0.0, 0.0, 1.0)],
        color_profile: ColorProfile::LinearRec709,
    });
    let err = ApngExporter
        .export(
            &fake_hdr,
            &ExportOpts {
                format: ExportFormat::Apng,
                ..ExportOpts::default()
            },
        )
        .expect_err("HDR refused");
    assert!(matches!(err, Error::HdrUnsupported));
}

#[test]
fn import_rejects_empty_as_truncated() {
    let err = ApngImporter
        .import(&[], &ImportOpts::default())
        .expect_err("empty");
    assert!(matches!(err, Error::Truncated));
}

/// W3 pre-gate fixture (audit C-1 + Lens C): exercise the
/// multi-frame decode path. Uses `png` 0.18's APNG encoder
/// (`set_animated` + `write_image_data` × N) to synthesize a
/// real 2-frame 2×2 APNG, then asserts our importer extracts
/// 2 frames with correct delays + offsets + dispose/blend ops.
#[test]
fn import_of_2_frame_apng_extracts_metadata() {
    let mut bytes: Vec<u8> = Vec::new();
    {
        let mut encoder = png::Encoder::new(Cursor::new(&mut bytes), 2, 2);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        // set_animated(num_frames=2, num_plays=0) — APNG spec:
        // num_plays=0 = infinite loop. We test 2 distinct frames;
        // the loop count is irrelevant to the decode-path assertions
        // below (delay/offset/blend extraction).
        encoder.set_animated(2, 0).expect("set_animated");
        // Frame 1: delay = 100ms (numer=100, denom=1000).
        encoder.set_frame_delay(100, 1000).expect("frame 1 delay");
        let mut writer = encoder.write_header().expect("write_header");
        // Frame 1 data: 4 pixels red.
        writer
            .write_image_data(&[
                255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255,
            ])
            .expect("frame 1 IDAT");
        // Frame 2: delay = 200ms.
        writer.set_frame_delay(200, 1000).expect("frame 2 delay");
        writer
            .write_image_data(&[
                0, 255, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255,
            ])
            .expect("frame 2 fdAT");
        writer.finish().expect("finish");
    }

    // Sanity: the synthesized buffer is a real APNG.
    assert!(has_actl_chunk(&bytes), "synthesized APNG must have acTL");

    let decoded = ApngImporter
        .import(&bytes, &ImportOpts::default())
        .expect("APNG import");
    let DecodedImage::Animated(frames) = decoded else {
        panic!("expected Animated for 2-frame APNG, got {decoded:?}");
    };
    assert_eq!(frames.len(), 2, "should extract 2 frames");
    // delay_num=100, delay_den=1000 → delay_ms = 100 * 1000 / 1000 = 100.
    assert_eq!(frames[0].delay_ms, 100, "frame 0 delay_ms");
    assert_eq!(frames[1].delay_ms, 200, "frame 1 delay_ms");
    // Frames at full canvas position.
    assert_eq!(frames[0].offset_xy, (0, 0));
    assert_eq!(frames[1].offset_xy, (0, 0));
    // Per-frame pixel sanity.
    assert_eq!(frames[0].image.pixels[0].0, [255, 0, 0, 255]);
    assert_eq!(frames[1].image.pixels[0].0, [0, 255, 0, 255]);
}

/// W3 pre-gate (LOCAL drift guard, Mac aarch64 only): pin blake3
/// hash of canonical APNG (single-frame) export. Single-platform
/// pin — png crate DEFLATE may diverge cross-OS. Multi-pin deferred
/// (entry: `crates/ph2d-imageio-apng/src/lib.rs::export_golden_blake3_local_drift_pinned_macos_silicon`).
// CONVENTION (audit-6 Lens B HIGH): keep `const GOLDEN_BLAKE3`
// inside the fn body. See ph2d-imageio-png/src/lib.rs for full
// rationale.
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[test]
fn export_golden_blake3_local_drift_pinned_macos_silicon() {
    let original = fixture_2x2([200, 100, 50, 255]);
    let opts = ExportOpts {
        format: ExportFormat::Apng,
        ..ExportOpts::default()
    };
    let bytes = ApngExporter
        .export(&DecodedImage::Flat(original), &opts)
        .expect("APNG golden export");
    let hash = blake3::hash(&bytes);
    const GOLDEN_BLAKE3: &str = "9e683a5d773937c9ec0430de37f512909e835f787ceff258aa54a0325f03c8c8";
    assert_eq!(
        hash.to_hex().to_string(),
        GOLDEN_BLAKE3,
        "LOCAL drift (macOS aarch64): APNG single-frame export blake3 drifted"
    );
}

/// HR-5 byte-exact determinism (single-frame path; multi-frame
/// encoder doesn't exist yet).
#[test]
fn export_is_byte_exact_deterministic() {
    let original = fixture_2x2([42, 84, 126, 255]);
    let opts = ExportOpts {
        format: ExportFormat::Apng,
        ..ExportOpts::default()
    };
    let a = ApngExporter
        .export(&DecodedImage::Flat(original.clone()), &opts)
        .expect("a");
    let b = ApngExporter
        .export(&DecodedImage::Flat(original), &opts)
        .expect("b");
    assert_eq!(a, b, "HR-5: APNG single-frame export must be byte-exact");
}

/// Audit-8 Lens M (2026-05-26): gate the MAX_ANIMATION_FRAMES cap
/// added by audit-7 J-1. Hostile acTL claiming `num_frames`
/// beyond the contract cap must be rejected with `Error::Decode`
/// before any allocation.
#[test]
fn import_rejects_actl_num_frames_above_max() {
    // Synthesize an APNG with acTL claiming MAX_ANIMATION_FRAMES+1
    // frames. We don't need to actually write fdAT — the cap
    // check happens BEFORE we try to read the frames.
    let mut bytes: Vec<u8> = Vec::new();
    {
        let mut encoder = png::Encoder::new(Cursor::new(&mut bytes), 2, 2);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .set_animated(ph2d_imageio::MAX_ANIMATION_FRAMES + 1, 0)
            .expect("set_animated");
        encoder.set_frame_delay(100, 1000).expect("delay");
        let mut writer = encoder.write_header().expect("write_header");
        // Frame 1: 4 RGBA pixels red. (The remaining frames won't
        // be written; the importer should reject before reading.)
        writer
            .write_image_data(&[
                255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255,
            ])
            .expect("frame 1");
    }
    let err = ApngImporter
        .import(&bytes, &ImportOpts::default())
        .expect_err("must reject");
    match err {
        Error::Decode(msg) => {
            assert!(
                msg.contains("MAX_ANIMATION_FRAMES"),
                "expected MAX_ANIMATION_FRAMES message: {msg}"
            );
        }
        other => panic!("expected Decode with cap message, got: {other:?}"),
    }
}
