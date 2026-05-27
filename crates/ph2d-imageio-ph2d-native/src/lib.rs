#![forbid(unsafe_code)]
//! `ph2d-imageio-ph2d-native` — Painter native `.ph2d` save format
//! (W1.T5).
//!
//! Unlike PNG/JPEG/WebP/GIF (which lean on the `image` crate and lose
//! semantic information at the codec boundary), `.ph2d` is custom and
//! preserves every variant of [`DecodedImage`] lossless round-trip:
//! `Flat` / `FlatHdr` / `Layered` (with full PSD-grade `LayerKind` +
//! effects + per-layer color profile) / `Animated` / `Vector`.
//!
//! ### Container layout
//!
//! ```text
//! [0..8]   "PH2DNATV"           magic           — Strong match
//! [8..12]  version: u32 LE      schema version  — 1 today (HR-14)
//! [12..44] payload_blake3: [u8; 32]  integrity  — HR-6
//! [44..52] payload_len: u64 LE  postcard size
//! [52..]   payload              postcard-encoded Ph2dNativeV1
//! ```
//!
//! Total header: 52 bytes. The blake3 hash is computed over the payload
//! bytes; importer recomputes and refuses on mismatch with
//! [`ph2d_imageio::Error::Decode`] carrying "integrity check failed".
//!
//! ### HR-14 migration scaffold
//!
//! [`schema::Ph2dNativeV1`] is today's wire shape. Future schema
//! diffs add `Ph2dNativeV2` + `migrate_v1_to_v2` and the importer
//! dispatches on the header `version` field. The scaffold is in
//! place (importer reads version BEFORE deserialising); migration
//! functions land as needed.
//!
//! **Inner version validation** (audit E-1 CRITICAL 2026-05-26): the
//! envelope's top-level `version` is checked at the header, AND every
//! nested `version: u32` field (`LayerStackV1.version`,
//! `LayerV1.version` recursively into `LayerKind::Group`) is validated
//! by [`schema::validate_v1_inner_versions`] before the `From` chain
//! lowers to `DecodedImage`. Without this walk a Painter-v2 build
//! writing `Layer { version: 2, … }` would silently round-trip through
//! a v1 build, losing v2-only fields — exactly the data-loss class
//! HR-14 was written to prevent.
//!
//! ### Out of scope (later waves)
//!
//! - **Schema v2+**: triggered by the first contract change adding
//!   a wire-level field. Migration function `migrate_v1_to_v2(v1) → v2`
//!   lands in the same commit that adds `Ph2dNativeV2`.
//! - **Compressed payload**: `postcard` is uncompressed; a future
//!   feature flag `compressed-payload` can wrap with `zstd` for files
//!   approaching `MAX_PH2D_PAYLOAD_LEN`. No client today.
//! - **Streaming decode**: import currently reads the full payload
//!   into memory. Large `.ph2d` (> 100 MB) would benefit from
//!   block-streaming via `postcard`'s `Cursor` adapter. No client today.

mod schema;

use ph2d_imageio::{
    DecodedImage, Error, ExportFormat, ExportOpts, ExporterRegistry, ImageExporter, ImageImporter,
    ImportOpts, ImporterRegistry, MAX_PH2D_PAYLOAD_LEN, MagicHint, MagicMatch,
};
use schema::{Ph2dContentV1, Ph2dNativeV1, validate_v1_inner_versions};

/// `.ph2d` 8-byte container magic. Distinctive enough to never collide
/// with any other image format in the W1–W3 fan-out.
const PH2D_MAGIC: &[u8; 8] = b"PH2DNATV";

/// Header: 8 magic + 4 version + 32 hash + 8 payload_len = 52 bytes.
const HEADER_LEN: usize = 52;

/// Latest schema version this build can write. Importer dispatches on
/// the header's version, so older files (v1) keep importing once we
/// move to v2+; this constant only constrains the *writer*.
const CURRENT_VERSION: u32 = 1;

// Hard cap on payload size is hoisted to
// `ph2d_imageio::MAX_PH2D_PAYLOAD_LEN` per audit B-H2 (single source).
// Value = `u32::MAX as u64` to avoid 32-bit `usize` overflow.

/// Register the `.ph2d` importer.
pub fn register_importer(reg: &mut ImporterRegistry) {
    reg.register(Box::new(Ph2dNativeImporter));
}

/// Register the `.ph2d` exporter.
pub fn register_exporter(reg: &mut ExporterRegistry) {
    reg.register(Box::new(Ph2dNativeExporter));
}

pub struct Ph2dNativeImporter;

impl ImageImporter for Ph2dNativeImporter {
    fn supports(&self, hint: MagicHint<'_>) -> MagicMatch {
        match hint {
            MagicHint::Bytes(b) if b.starts_with(PH2D_MAGIC) => MagicMatch::Strong,
            MagicHint::Bytes(_) => MagicMatch::None,
            MagicHint::Extension(ext) if ext.eq_ignore_ascii_case("ph2d") => MagicMatch::Weak,
            MagicHint::Extension(_) => MagicMatch::None,
        }
    }

    fn import(&self, src: &[u8], _opts: &ImportOpts) -> Result<DecodedImage, Error> {
        if src.len() < HEADER_LEN {
            return Err(Error::Truncated);
        }
        if &src[0..8] != PH2D_MAGIC {
            return Err(Error::Decode("not a `.ph2d` file (magic mismatch)".into()));
        }
        let version = u32::from_le_bytes(
            src[8..12]
                .try_into()
                .map_err(|_| Error::Decode("header truncated reading version".into()))?,
        );
        let header_hash: [u8; 32] = src[12..44]
            .try_into()
            .map_err(|_| Error::Decode("header truncated reading blake3".into()))?;
        let payload_len = u64::from_le_bytes(
            src[44..52]
                .try_into()
                .map_err(|_| Error::Decode("header truncated reading payload_len".into()))?,
        );
        if payload_len > MAX_PH2D_PAYLOAD_LEN {
            return Err(Error::DimensionExceedsLimit);
        }
        let payload_start = HEADER_LEN;
        let payload_end = payload_start
            .checked_add(usize::try_from(payload_len).map_err(|_| Error::OutOfMemory)?)
            .ok_or(Error::OutOfMemory)?;
        if src.len() < payload_end {
            return Err(Error::Truncated);
        }
        let payload = &src[payload_start..payload_end];
        // HR-6: re-hash payload and compare to header.
        let computed_hash = blake3::hash(payload);
        if computed_hash.as_bytes() != &header_hash {
            return Err(Error::Decode(
                "integrity check failed (blake3 mismatch)".into(),
            ));
        }
        // Dispatch on schema version. v1 only today; v2+ adds a
        // migration arm here.
        match version {
            // 0 is reserved for "uninitialized header" — distinct from
            // a real future schema. Audit `.ph2d-native` L1 follow-up.
            0 => Err(Error::Decode(
                "`.ph2d` header version=0 (uninitialized or zeroed file)".into(),
            )),
            1 => {
                let v1: Ph2dNativeV1 = postcard::from_bytes(payload).map_err(|e| {
                    Error::from_decoder_message(format!("postcard decode failed: {e}"))
                })?;
                if v1.version != 1 {
                    return Err(Error::Decode(format!(
                        "header version=1 but payload claims version={}",
                        v1.version
                    )));
                }
                // Audit E-1 CRITICAL (2026-05-26): walk the envelope
                // validating every nested `version: u32` field
                // (LayerStackV1, LayerV1, recursively into Group).
                // Without this, a Painter-v2 file with inner
                // `Layer { version: 2, … }` would silently round-trip
                // as v1 with v2 fields lost.
                validate_v1_inner_versions(&v1.content)?;
                Ok(v1.content.into())
            }
            other => Err(Error::Unsupported(format!(
                "`.ph2d` schema version {other} not supported by this build (max: {CURRENT_VERSION})"
            ))),
        }
    }
}

pub struct Ph2dNativeExporter;

impl ImageExporter for Ph2dNativeExporter {
    fn supports_format(&self, fmt: ExportFormat) -> bool {
        matches!(fmt, ExportFormat::Ph2dNative)
    }

    fn export(&self, img: &DecodedImage, _opts: &ExportOpts) -> Result<Vec<u8>, Error> {
        let envelope = Ph2dNativeV1 {
            version: CURRENT_VERSION,
            content: Ph2dContentV1::from(img),
        };
        let payload: Vec<u8> = postcard::to_allocvec(&envelope)
            .map_err(|e| Error::Encode(format!("postcard encode failed: {e}")))?;
        let payload_len = payload.len() as u64;
        let hash = blake3::hash(&payload);
        let mut out: Vec<u8> = Vec::with_capacity(HEADER_LEN + payload.len());
        out.extend_from_slice(PH2D_MAGIC);
        out.extend_from_slice(&CURRENT_VERSION.to_le_bytes());
        out.extend_from_slice(hash.as_bytes());
        out.extend_from_slice(&payload_len.to_le_bytes());
        out.extend_from_slice(&payload);
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_color::{LinearRgba, SrgbRgba};
    use ph2d_imageio::{
        AnimBlendOp, AnimFrame, BlendMode, ColorProfile, DisposeOp, ImageBuffer, Layer,
        LayerEffect, LayerEffectKind, LayerKind, LayerStack, VectorDoc,
    };

    fn flat_fixture() -> DecodedImage {
        DecodedImage::Flat(ImageBuffer {
            width: 2,
            height: 2,
            pixels: vec![
                SrgbRgba([255, 0, 0, 255]),
                SrgbRgba([0, 255, 0, 200]),
                SrgbRgba([0, 0, 255, 100]),
                SrgbRgba([128, 128, 128, 50]),
            ],
            color_profile: ColorProfile::Srgb,
        })
    }

    #[test]
    fn supports_recognizes_ph2d_magic_strong() {
        let imp = Ph2dNativeImporter;
        assert_eq!(
            imp.supports(MagicHint::Bytes(PH2D_MAGIC)),
            MagicMatch::Strong
        );
        let mut padded = PH2D_MAGIC.to_vec();
        padded.extend_from_slice(&[0; 32]);
        assert_eq!(imp.supports(MagicHint::Bytes(&padded)), MagicMatch::Strong);
        assert_eq!(
            imp.supports(MagicHint::Bytes(b"NOTPH2DBYTES")),
            MagicMatch::None
        );
        assert_eq!(imp.supports(MagicHint::Bytes(&[])), MagicMatch::None);
    }

    #[test]
    fn supports_recognizes_ph2d_extension_weak() {
        let imp = Ph2dNativeImporter;
        for ext in &["ph2d", "PH2D", "Ph2d"] {
            assert_eq!(imp.supports(MagicHint::Extension(ext)), MagicMatch::Weak);
        }
        assert_eq!(imp.supports(MagicHint::Extension("png")), MagicMatch::None);
    }

    #[test]
    fn exporter_recognizes_ph2d_native_format() {
        let exp = Ph2dNativeExporter;
        assert!(exp.supports_format(ExportFormat::Ph2dNative));
        assert!(!exp.supports_format(ExportFormat::Png));
    }

    /// Bit-exact round-trip for `Flat` — the workhorse case (Painter
    /// saves with no layers). HR-6: integrity check passes; pixels
    /// identical.
    #[test]
    fn roundtrip_flat_bit_exact() {
        let original = flat_fixture();
        let bytes = Ph2dNativeExporter
            .export(
                &original,
                &ExportOpts {
                    format: ExportFormat::Ph2dNative,
                    ..ExportOpts::default()
                },
            )
            .expect(".ph2d export");
        assert!(bytes.starts_with(PH2D_MAGIC));
        assert!(bytes.len() >= HEADER_LEN);

        let decoded = Ph2dNativeImporter
            .import(&bytes, &ImportOpts::default())
            .expect(".ph2d import");
        let (DecodedImage::Flat(orig), DecodedImage::Flat(out)) = (&original, &decoded) else {
            panic!("expected Flat on both sides");
        };
        assert_eq!(orig.width, out.width);
        assert_eq!(orig.height, out.height);
        assert_eq!(orig.pixels.len(), out.pixels.len());
        for (i, (a, b)) in orig.pixels.iter().zip(out.pixels.iter()).enumerate() {
            assert_eq!(a.0, b.0, "pixel {i} drifted");
        }
    }

    /// Round-trip for `Layered` with a non-trivial layer stack (groups,
    /// blend modes, effects, per-layer profile, mask). Proves the
    /// schema mirror covers every PSD-grade field we promise.
    #[test]
    fn roundtrip_layered_preserves_structure() {
        let leaf = Layer {
            version: 1,
            name: "leaf".into(),
            kind: LayerKind::Pixel,
            pixels: Some(ImageBuffer {
                width: 1,
                height: 1,
                pixels: vec![SrgbRgba([10, 20, 30, 255])],
                color_profile: ColorProfile::Srgb,
            }),
            opacity: 0.5,
            blend_mode: BlendMode::Multiply,
            visible: true,
            mask: Some(ImageBuffer {
                width: 1,
                height: 1,
                pixels: vec![128],
                color_profile: ColorProfile::Srgb,
            }),
            effects: vec![LayerEffect {
                kind: LayerEffectKind::DropShadow,
            }],
            color_profile: Some(ColorProfile::DisplayP3),
        };
        let group = Layer {
            version: 1,
            name: "group".into(),
            kind: LayerKind::Group {
                children: vec![leaf],
            },
            pixels: None,
            opacity: 1.0,
            blend_mode: BlendMode::Custom(0xABCD),
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
            color_profile: ColorProfile::AdobeRgb,
        };
        let original = DecodedImage::Layered(stack);

        let bytes = Ph2dNativeExporter
            .export(
                &original,
                &ExportOpts {
                    format: ExportFormat::Ph2dNative,
                    ..ExportOpts::default()
                },
            )
            .expect("export");
        let decoded = Ph2dNativeImporter
            .import(&bytes, &ImportOpts::default())
            .expect("import");
        let DecodedImage::Layered(out) = decoded else {
            panic!("expected Layered");
        };
        assert_eq!(out.version, 1);
        assert_eq!(out.canvas_width, 2);
        assert_eq!(out.canvas_height, 2);
        assert_eq!(out.color_profile, ColorProfile::AdobeRgb);
        assert_eq!(out.layers.len(), 1);
        let g = &out.layers[0];
        assert_eq!(g.name, "group");
        assert_eq!(g.blend_mode, BlendMode::Custom(0xABCD));
        assert!(!g.visible);
        let LayerKind::Group { children } = &g.kind else {
            panic!("expected Group kind");
        };
        assert_eq!(children.len(), 1);
        let leaf = &children[0];
        assert_eq!(leaf.name, "leaf");
        assert_eq!(leaf.blend_mode, BlendMode::Multiply);
        assert!((leaf.opacity - 0.5).abs() < f32::EPSILON);
        assert_eq!(leaf.color_profile, Some(ColorProfile::DisplayP3));
        assert_eq!(leaf.effects.len(), 1);
        assert!(matches!(leaf.effects[0].kind, LayerEffectKind::DropShadow));
        let pix = leaf.pixels.as_ref().expect("leaf has pixels");
        assert_eq!(pix.pixels[0].0, [10, 20, 30, 255]);
        let mask = leaf.mask.as_ref().expect("leaf has mask");
        assert_eq!(mask.pixels[0], 128);
    }

    /// Round-trip for `Animated` preserves frame count + delays +
    /// offsets + dispose ops (none of which any other W1 format
    /// preserves end-to-end; only `.ph2d` does).
    #[test]
    fn roundtrip_animated_preserves_full_fidelity() {
        let frames = vec![
            AnimFrame {
                image: ImageBuffer {
                    width: 1,
                    height: 1,
                    pixels: vec![SrgbRgba([255, 0, 0, 255])],
                    color_profile: ColorProfile::Srgb,
                },
                delay_ms: 100,
                offset_xy: (5, 10),
                dispose_op: DisposeOp::Background,
                blend_op: AnimBlendOp::Source,
                transparent_index: Some(7),
            },
            AnimFrame {
                image: ImageBuffer {
                    width: 1,
                    height: 1,
                    pixels: vec![SrgbRgba([0, 255, 0, 200])],
                    color_profile: ColorProfile::Srgb,
                },
                delay_ms: 200,
                offset_xy: (-3, 0),
                dispose_op: DisposeOp::Previous,
                blend_op: AnimBlendOp::Over,
                transparent_index: None,
            },
        ];
        let original = DecodedImage::Animated(frames);
        let bytes = Ph2dNativeExporter
            .export(
                &original,
                &ExportOpts {
                    format: ExportFormat::Ph2dNative,
                    ..ExportOpts::default()
                },
            )
            .expect("export");
        let decoded = Ph2dNativeImporter
            .import(&bytes, &ImportOpts::default())
            .expect("import");
        let DecodedImage::Animated(out) = decoded else {
            panic!("expected Animated");
        };
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].delay_ms, 100);
        assert_eq!(out[0].offset_xy, (5, 10));
        assert_eq!(out[0].dispose_op, DisposeOp::Background);
        assert_eq!(out[0].transparent_index, Some(7));
        assert_eq!(out[1].delay_ms, 200);
        assert_eq!(out[1].offset_xy, (-3, 0));
        assert_eq!(out[1].dispose_op, DisposeOp::Previous);
        assert_eq!(out[1].blend_op, AnimBlendOp::Over);
        assert_eq!(out[1].transparent_index, None);
    }

    /// Round-trip for `FlatHdr` preserves float linear values bit-exact
    /// (postcard is lossless on f32; blake3 keyed on bytes).
    #[test]
    fn roundtrip_flat_hdr_preserves_floats() {
        let original = DecodedImage::FlatHdr(ImageBuffer {
            width: 1,
            height: 1,
            pixels: vec![LinearRgba::new(0.123, 0.456, 0.789, 1.0)],
            color_profile: ColorProfile::LinearRec709,
        });
        let bytes = Ph2dNativeExporter
            .export(
                &original,
                &ExportOpts {
                    format: ExportFormat::Ph2dNative,
                    ..ExportOpts::default()
                },
            )
            .expect("hdr export");
        let decoded = Ph2dNativeImporter
            .import(&bytes, &ImportOpts::default())
            .expect("hdr import");
        let DecodedImage::FlatHdr(out) = decoded else {
            panic!("expected FlatHdr");
        };
        let arr = out.pixels[0].as_array();
        assert!((arr[0] - 0.123).abs() < f32::EPSILON);
        assert!((arr[1] - 0.456).abs() < f32::EPSILON);
        assert!((arr[2] - 0.789).abs() < f32::EPSILON);
        assert!((arr[3] - 1.0).abs() < f32::EPSILON);
        assert_eq!(out.color_profile, ColorProfile::LinearRec709);
    }

    /// Round-trip for `Vector` — stub today; ensures the variant
    /// survives encode/decode for parity once W3 populates the body.
    #[test]
    fn roundtrip_vector_stub() {
        let original = DecodedImage::Vector(VectorDoc::default());
        let bytes = Ph2dNativeExporter
            .export(
                &original,
                &ExportOpts {
                    format: ExportFormat::Ph2dNative,
                    ..ExportOpts::default()
                },
            )
            .expect("vector export");
        let decoded = Ph2dNativeImporter
            .import(&bytes, &ImportOpts::default())
            .expect("vector import");
        assert!(matches!(decoded, DecodedImage::Vector(_)));
    }

    /// HR-6 integrity check: tampering with payload bytes makes the
    /// hash fail and the importer refuses.
    #[test]
    fn integrity_check_refuses_tampered_payload() {
        let original = flat_fixture();
        let mut bytes = Ph2dNativeExporter
            .export(
                &original,
                &ExportOpts {
                    format: ExportFormat::Ph2dNative,
                    ..ExportOpts::default()
                },
            )
            .expect("export");
        // Flip a byte in the payload (anywhere past HEADER_LEN).
        assert!(bytes.len() > HEADER_LEN + 4);
        bytes[HEADER_LEN + 2] ^= 0xFF;
        let err = Ph2dNativeImporter
            .import(&bytes, &ImportOpts::default())
            .expect_err("tampered payload must be refused");
        match err {
            Error::Decode(msg) => assert!(
                msg.contains("integrity check failed"),
                "expected integrity error, got: {msg}"
            ),
            other => panic!("expected Decode(integrity check failed), got {other:?}"),
        }
    }

    #[test]
    fn import_rejects_short_input_as_truncated() {
        let err = Ph2dNativeImporter
            .import(b"PH2DNATV", &ImportOpts::default())
            .expect_err("8 bytes < HEADER_LEN must fail");
        assert!(matches!(err, Error::Truncated));
    }

    #[test]
    fn import_rejects_wrong_magic_with_decode() {
        let mut bytes = vec![0_u8; HEADER_LEN];
        // Header is correct length but magic doesn't match.
        bytes[0..8].copy_from_slice(b"WRONGMGC");
        let err = Ph2dNativeImporter
            .import(&bytes, &ImportOpts::default())
            .expect_err("bad magic must fail");
        assert!(matches!(err, Error::Decode(_)));
    }

    /// HR-14: a future schema bump must produce `Error::Unsupported`
    /// (not a panic, not a silent miscompile). Synthesize a header
    /// claiming version=999 and confirm.
    #[test]
    fn import_rejects_future_version_with_unsupported() {
        let mut bytes = vec![0_u8; HEADER_LEN];
        bytes[0..8].copy_from_slice(PH2D_MAGIC);
        bytes[8..12].copy_from_slice(&999_u32.to_le_bytes());
        // Empty payload (len=0); blake3 of empty must match.
        let empty_hash = blake3::hash(b"");
        bytes[12..44].copy_from_slice(empty_hash.as_bytes());
        bytes[44..52].copy_from_slice(&0_u64.to_le_bytes());

        let err = Ph2dNativeImporter
            .import(&bytes, &ImportOpts::default())
            .expect_err("version 999 must fail");
        match err {
            Error::Unsupported(msg) => assert!(msg.contains("version 999")),
            other => panic!("expected Unsupported, got {other:?}"),
        }
    }

    /// Audit E-1 CRITICAL (2026-05-26): a v1 envelope carrying an
    /// inner `LayerStack { version: 7, … }` must be refused — not
    /// silently passed through as v1. Synthesise such a payload by
    /// hand (bypass `From<&LayerStack>` which would set version=1)
    /// and confirm the importer rejects.
    #[test]
    fn import_rejects_inner_layer_stack_version_drift() {
        use crate::schema::{ColorProfileV1, LayerStackV1, Ph2dContentV1, Ph2dNativeV1};
        // Synthesise an envelope claiming LayerStack v7.
        let envelope = Ph2dNativeV1 {
            version: 1,
            content: Ph2dContentV1::Layered(LayerStackV1 {
                version: 7, // future schema — must be refused.
                canvas_width: 1,
                canvas_height: 1,
                layers: Vec::new(),
                color_profile: ColorProfileV1::Srgb,
            }),
        };
        let payload: Vec<u8> = postcard::to_allocvec(&envelope).expect("encode");
        let payload_len = payload.len() as u64;
        let hash = blake3::hash(&payload);
        let mut bytes: Vec<u8> = Vec::with_capacity(HEADER_LEN + payload.len());
        bytes.extend_from_slice(PH2D_MAGIC);
        bytes.extend_from_slice(&1_u32.to_le_bytes());
        bytes.extend_from_slice(hash.as_bytes());
        bytes.extend_from_slice(&payload_len.to_le_bytes());
        bytes.extend_from_slice(&payload);

        let err = Ph2dNativeImporter
            .import(&bytes, &ImportOpts::default())
            .expect_err("v7 inner LayerStack must be refused");
        match err {
            Error::Unsupported(msg) => assert!(
                msg.contains("LayerStack schema version 7"),
                "expected version-7 error, got: {msg}"
            ),
            other => panic!("expected Unsupported, got {other:?}"),
        }
    }

    /// Audit `.ph2d-native` L1 (2026-05-26): version=0 distinct from
    /// future schema; should return Decode (uninitialized), not
    /// Unsupported (future).
    #[test]
    fn import_rejects_version_zero_with_decode() {
        let mut bytes = vec![0_u8; HEADER_LEN];
        bytes[0..8].copy_from_slice(PH2D_MAGIC);
        bytes[8..12].copy_from_slice(&0_u32.to_le_bytes());
        let empty_hash = blake3::hash(b"");
        bytes[12..44].copy_from_slice(empty_hash.as_bytes());
        bytes[44..52].copy_from_slice(&0_u64.to_le_bytes());

        let err = Ph2dNativeImporter
            .import(&bytes, &ImportOpts::default())
            .expect_err("version 0 must fail");
        match err {
            Error::Decode(msg) => assert!(
                msg.contains("version=0"),
                "expected version-0 decode error, got: {msg}"
            ),
            other => panic!("expected Decode, got {other:?}"),
        }
    }

    /// HR-5 byte-exact determinism: encoding the same input twice
    /// produces identical bytes (no timestamps, no random salts).
    #[test]
    fn export_is_byte_exact_deterministic() {
        let original = flat_fixture();
        let opts = ExportOpts {
            format: ExportFormat::Ph2dNative,
            ..ExportOpts::default()
        };
        let a = Ph2dNativeExporter.export(&original, &opts).expect("a");
        let b = Ph2dNativeExporter.export(&original, &opts).expect("b");
        assert_eq!(a, b, "HR-5: .ph2d encode must be byte-exact");
    }
}
