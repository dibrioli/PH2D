//! AVIF integration tests (W3.T4 Path C). Mirrors the structure of
//! `ph2d-imageio-jxl` / `-hdr-radiance` tests. Because we now have a
//! real encoder, fixtures are produced in-process (`encode` → `decode`
//! round-trip) — no external `.avif` files needed.

use ph2d_color::{LinearRgba, SrgbRgba};
use ph2d_imageio::{
    ColorProfile, DecodedImage, Error, ExportFormat, ExportOpts, ImageBuffer, ImageExporter,
    ImageImporter, ImportOpts, MagicHint, MagicMatch,
};
use ph2d_imageio_avif::{AvifExporter, AvifImporter};

fn ldr_fixture(w: u32, h: u32, profile: ColorProfile) -> ImageBuffer<SrgbRgba> {
    let mut pixels = Vec::with_capacity((w * h) as usize);
    for i in 0..(w * h) {
        let v = (i % 256) as u8;
        pixels.push(SrgbRgba([v, 255 - v, v / 2, 255]));
    }
    ImageBuffer {
        width: w,
        height: h,
        pixels,
        color_profile: profile,
    }
}

fn encode(img: &DecodedImage, quality: u8) -> Vec<u8> {
    AvifExporter
        .export(
            img,
            &ExportOpts {
                format: ExportFormat::Avif,
                quality,
                ..ExportOpts::default()
            },
        )
        .expect("AVIF encode")
}

fn decode(bytes: &[u8]) -> DecodedImage {
    AvifImporter
        .import(bytes, &ImportOpts::default())
        .expect("AVIF decode")
}

// ── magic / dispatch ────────────────────────────────────────────────

#[test]
fn magic_recognition() {
    let avif = [
        0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p', b'a', b'v', b'i', b'f', 0, 0,
    ];
    assert_eq!(
        AvifImporter.supports(MagicHint::Bytes(&avif)),
        MagicMatch::Strong
    );
    // HEIF (brand mif1) must NOT match.
    let heif = [
        0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p', b'm', b'i', b'f', b'1', 0, 0,
    ];
    assert_eq!(
        AvifImporter.supports(MagicHint::Bytes(&heif)),
        MagicMatch::None
    );
    assert_eq!(
        AvifImporter.supports(MagicHint::Extension("avif")),
        MagicMatch::Weak
    );
    assert!(AvifExporter.supports_format(ExportFormat::Avif));
    assert!(!AvifExporter.supports_format(ExportFormat::Png));
}

// ── robustness ──────────────────────────────────────────────────────

#[test]
fn import_rejects_empty_as_truncated() {
    let err = AvifImporter
        .import(&[], &ImportOpts::default())
        .expect_err("empty");
    assert!(matches!(err, Error::Truncated));
}

#[test]
fn truncated_avif_does_not_panic() {
    // A valid-looking ftyp header followed by nothing. Must return an
    // error (Truncated/Decode), never panic the process (catch_unwind).
    let bytes = [
        0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p', b'a', b'v', b'i', b'f', 0, 0, 0, 0,
    ];
    let err = AvifImporter
        .import(&bytes, &ImportOpts::default())
        .expect_err("truncated");
    assert!(matches!(err, Error::Decode(_) | Error::Truncated));
}

#[test]
fn hostile_garbage_does_not_panic() {
    // Random bytes with AVIF-ish magic — exercises the catch_unwind +
    // error-mapping path. The only contract: it returns, doesn't abort.
    let mut bytes = vec![
        0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p', b'a', b'v', b'i', b'f',
    ];
    bytes.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x42, 0xFF, 0x01]);
    let _ = AvifImporter.import(&bytes, &ImportOpts::default());
}

// ── real decode/encode round-trips ──────────────────────────────────

#[test]
fn real_decode_smoke_and_lossless_roundtrip() {
    // Encode a known LDR fixture losslessly, decode it back, and assert
    // exact pixel equality (quality=100 ⇒ identity matrix, no loss).
    let original = ldr_fixture(16, 8, ColorProfile::Srgb);
    let bytes = encode(&DecodedImage::Flat(original.clone()), 100);
    assert!(bytes.len() > 12, "encoder produced real output");
    assert_eq!(&bytes[4..8], b"ftyp", "output is an ISOBMFF/AVIF file");

    let DecodedImage::Flat(decoded) = decode(&bytes) else {
        panic!("expected Flat for an 8-bit sRGB AVIF");
    };
    assert_eq!(decoded.width, 16);
    assert_eq!(decoded.height, 8);
    assert_eq!(decoded.color_profile, ColorProfile::Srgb);
    assert_eq!(
        decoded.pixels, original.pixels,
        "lossless 8-bit RGBA must round-trip exactly"
    );
}

#[test]
fn encode_decode_roundtrip_lossy_is_close() {
    // Lossy (quality 90) round-trip — not exact, but close.
    let original = ldr_fixture(32, 32, ColorProfile::Srgb);
    let bytes = encode(&DecodedImage::Flat(original.clone()), 90);
    let DecodedImage::Flat(decoded) = decode(&bytes) else {
        panic!("expected Flat");
    };
    assert_eq!(decoded.width, 32);
    assert_eq!(decoded.pixels.len(), original.pixels.len());
    // YUV444 q90 should keep every channel within a small delta.
    let mut max_delta = 0i32;
    for (o, g) in original.pixels.iter().zip(decoded.pixels.iter()) {
        for c in 0..3 {
            max_delta = max_delta.max((i32::from(o.0[c]) - i32::from(g.0[c])).abs());
        }
    }
    assert!(max_delta <= 24, "lossy delta too large: {max_delta}");
}

#[test]
fn hdr_wide_gamut_roundtrip() {
    // FlatHdr (scene-linear, Rec.2020) → 10-bit PQ AVIF → FlatHdr.
    // Round-trips the wide-gamut HDR path: nclx must decode back to
    // LinearRec2020 and pixel values within 10-bit PQ quantization.
    let original = ImageBuffer {
        width: 4,
        height: 4,
        pixels: vec![
            LinearRgba::new(0.18, 0.5, 1.0, 1.0),
            LinearRgba::new(2.0, 0.25, 4.0, 1.0),
            LinearRgba::new(0.0, 0.0, 0.0, 1.0),
            LinearRgba::new(10.0, 8.0, 6.0, 0.5),
        ]
        .into_iter()
        .cycle()
        .take(16)
        .collect(),
        color_profile: ColorProfile::LinearRec2020,
    };
    let bytes = encode(&DecodedImage::FlatHdr(original.clone()), 100);
    let DecodedImage::FlatHdr(decoded) = decode(&bytes) else {
        panic!("expected FlatHdr for a 10-bit PQ Rec.2020 AVIF");
    };
    assert_eq!(decoded.width, 4);
    assert_eq!(decoded.height, 4);
    assert_eq!(
        decoded.color_profile,
        ColorProfile::LinearRec2020,
        "wide-gamut HDR must decode back as LinearRec2020"
    );
    // 10-bit PQ has ~0.5-2% relative precision in the SDR range; allow
    // generous tolerance (the property tested is "HDR survives", not
    // bit-exactness — that's physically impossible through 10-bit PQ).
    for (i, (o, g)) in original
        .pixels
        .iter()
        .zip(decoded.pixels.iter())
        .enumerate()
    {
        for (label, (ov, gv)) in [
            ("r", (o.r(), g.r())),
            ("g", (o.g(), g.g())),
            ("b", (o.b(), g.b())),
        ] {
            let abs = (ov - gv).abs();
            let rel = abs / ov.max(0.05);
            assert!(
                rel < 0.10 || abs < 0.02,
                "px {i} {label}: orig={ov} got={gv} rel={rel}"
            );
        }
    }
}

#[test]
fn grid_image_handling_documented() {
    // We don't have a grid-image fixture in-tree (encoding a grid needs
    // avifEncoderAddImageGrid, which we don't expose). The contract we
    // CAN assert: a normal single-image AVIF decodes as one composited
    // image (libavif stitches grids transparently into the primary
    // image, so the importer sees a single Flat/FlatHdr either way).
    // This guards that the single-image path — the grid fallthrough —
    // stays working. Documented limitation: grid is decoded as its
    // composited primary, never as separate cells.
    let original = ldr_fixture(8, 8, ColorProfile::Srgb);
    let bytes = encode(&DecodedImage::Flat(original), 100);
    assert!(matches!(decode(&bytes), DecodedImage::Flat(_)));
}

#[test]
fn displayp3_sdr_roundtrip_preserves_profile() {
    // Wide-gamut SDR path: encode Flat tagged Display-P3 losslessly,
    // decode, and confirm the SMPTE432 primaries round-trip back to
    // ColorProfile::DisplayP3 (not silently downgraded to Srgb).
    let original = ldr_fixture(8, 8, ColorProfile::DisplayP3);
    let bytes = encode(&DecodedImage::Flat(original.clone()), 100);
    let DecodedImage::Flat(decoded) = decode(&bytes) else {
        panic!("expected Flat");
    };
    assert_eq!(
        decoded.color_profile,
        ColorProfile::DisplayP3,
        "Display-P3 primaries must round-trip"
    );
    assert_eq!(decoded.pixels, original.pixels, "lossless P3 exact");
}

#[test]
fn hdr_rec709_roundtrip_non_wide_gamut() {
    // HDR path with Rec.709 (non-wide) primaries: exercises the
    // LinearRec709 branch + 10-bit PQ BT.709 nclx, distinct from the
    // Rec.2020 test above.
    let original = ImageBuffer {
        width: 2,
        height: 2,
        pixels: vec![
            LinearRgba::new(0.5, 1.0, 2.0, 1.0),
            LinearRgba::new(3.0, 0.1, 0.2, 1.0),
            LinearRgba::new(0.0, 0.0, 0.0, 1.0),
            LinearRgba::new(8.0, 8.0, 8.0, 1.0),
        ],
        color_profile: ColorProfile::LinearRec709,
    };
    let bytes = encode(&DecodedImage::FlatHdr(original.clone()), 100);
    let DecodedImage::FlatHdr(decoded) = decode(&bytes) else {
        panic!("expected FlatHdr");
    };
    assert_eq!(
        decoded.color_profile,
        ColorProfile::LinearRec709,
        "Rec.709 HDR must decode back as LinearRec709 (not Rec2020)"
    );
    for (i, (o, g)) in original
        .pixels
        .iter()
        .zip(decoded.pixels.iter())
        .enumerate()
    {
        for (label, (ov, gv)) in [
            ("r", (o.r(), g.r())),
            ("g", (o.g(), g.g())),
            ("b", (o.b(), g.b())),
        ] {
            let abs = (ov - gv).abs();
            let rel = abs / ov.max(0.05);
            assert!(rel < 0.10 || abs < 0.02, "px {i} {label}: {ov} vs {gv}");
        }
    }
}

// ── exporter rejects non-raster / multi-image ───────────────────────

#[test]
fn encode_rejects_animated_and_vector_and_layered() {
    for img in [
        DecodedImage::Animated(vec![]),
        DecodedImage::Vector(Default::default()),
    ] {
        let err = AvifExporter
            .export(
                &img,
                &ExportOpts {
                    format: ExportFormat::Avif,
                    ..ExportOpts::default()
                },
            )
            .expect_err("non-raster must be Unsupported");
        assert!(matches!(err, Error::Unsupported(_)));
    }
}
