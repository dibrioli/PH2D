//! ADR-0025 M14.4c — `AssetDb::insert_image_bytes` accepts PNG /
//! WEBP / JPEG and produces a deterministic `AssetId` for each.

use image::{DynamicImage, ImageFormat, RgbaImage};
use ph2d_asset::{Asset, AssetDb};
use std::io::Cursor;

fn solid_rgba(w: u32, h: u32, color: [u8; 4]) -> RgbaImage {
    RgbaImage::from_fn(w, h, |_x, _y| image::Rgba(color))
}

fn encode(format: ImageFormat, w: u32, h: u32, color: [u8; 4]) -> Vec<u8> {
    let img = solid_rgba(w, h, color);
    let mut buf = Vec::new();
    DynamicImage::ImageRgba8(img)
        .write_to(&mut Cursor::new(&mut buf), format)
        .expect("encode");
    buf
}

#[test]
fn insert_png_bytes_round_trips_dimensions() {
    let bytes = encode(ImageFormat::Png, 12, 16, [255, 0, 0, 255]);
    let db = AssetDb::new();
    let id = db.insert_image_bytes(&bytes).unwrap();
    let asset = db.get(&id).unwrap();
    let Asset::ImageRgba8 { width, height, .. } = &*asset else {
        panic!("expected Asset::ImageRgba8");
    };
    assert_eq!(*width, 12);
    assert_eq!(*height, 16);
}

#[test]
fn insert_webp_bytes_round_trips_dimensions() {
    let bytes = encode(ImageFormat::WebP, 10, 10, [0, 255, 0, 255]);
    let db = AssetDb::new();
    let id = db.insert_image_bytes(&bytes).unwrap();
    let asset = db.get(&id).unwrap();
    let Asset::ImageRgba8 { width, height, .. } = &*asset else {
        panic!("expected Asset::ImageRgba8");
    };
    assert_eq!(*width, 10);
    assert_eq!(*height, 10);
}

#[test]
fn insert_jpeg_bytes_round_trips_dimensions() {
    // JPEG is lossy + has no alpha; encoder drops the alpha channel.
    // We only assert dimensions, not pixel content.
    let bytes = encode(ImageFormat::Jpeg, 8, 8, [128, 128, 128, 255]);
    let db = AssetDb::new();
    let id = db.insert_image_bytes(&bytes).unwrap();
    let asset = db.get(&id).unwrap();
    let Asset::ImageRgba8 { width, height, .. } = &*asset else {
        panic!("expected Asset::ImageRgba8");
    };
    assert_eq!(*width, 8);
    assert_eq!(*height, 8);
}

#[test]
fn unsupported_format_rejected() {
    // Random bytes that don't match any image signature.
    let bytes = b"NOT_AN_IMAGE_FILE_AT_ALL".to_vec();
    let db = AssetDb::new();
    let err = db.insert_image_bytes(&bytes).unwrap_err();
    // Either guess_format fails or downstream decode does — both
    // surface as `AssetError::Decode`.
    match err {
        ph2d_asset::AssetError::Decode { .. } => {}
        other => panic!("expected Decode error, got {other:?}"),
    }
}

#[test]
fn same_bytes_yield_same_asset_id() {
    let bytes = encode(ImageFormat::Png, 4, 4, [10, 20, 30, 255]);
    let db = AssetDb::new();
    let a = db.insert_image_bytes(&bytes).unwrap();
    let b = db.insert_image_bytes(&bytes).unwrap();
    assert_eq!(
        a, b,
        "HR-6: content-addressed AssetId must be deterministic"
    );
}

#[test]
fn distinct_content_yields_distinct_ids() {
    let red = encode(ImageFormat::Png, 4, 4, [255, 0, 0, 255]);
    let blue = encode(ImageFormat::Png, 4, 4, [0, 0, 255, 255]);
    let db = AssetDb::new();
    let id_red = db.insert_image_bytes(&red).unwrap();
    let id_blue = db.insert_image_bytes(&blue).unwrap();
    assert_ne!(id_red, id_blue);
}

/// Smoke-test: same image bytes encoded as PNG vs WEBP yield
/// DIFFERENT AssetIds (content-addressed, not pixel-addressed).
/// This is correct per HR-6 — the file bytes are the identity.
#[test]
fn same_pixels_in_different_formats_yield_different_ids() {
    let png = encode(ImageFormat::Png, 4, 4, [50, 50, 50, 255]);
    let webp = encode(ImageFormat::WebP, 4, 4, [50, 50, 50, 255]);
    let db = AssetDb::new();
    let id_png = db.insert_image_bytes(&png).unwrap();
    let id_webp = db.insert_image_bytes(&webp).unwrap();
    assert_ne!(id_png, id_webp);
}
