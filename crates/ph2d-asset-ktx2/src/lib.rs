#![forbid(unsafe_code)]
//! `ph2d-asset-ktx2` — Khronos KTX2 texture container decoder (Fase 1,
//! codec-only).
//!
//! The KTX2 container splits a single byte stream into header + Data
//! Format Descriptor + key/value metadata + mip pyramid, and is the
//! Khronos-blessed delivery format for GPU-compressed textures
//! (BC7/BC6H desktop, ASTC mobile + Apple Silicon, ETC2 Android
//! fallback) plus uncompressed RGBA8 / RGBA16F. PH2D's SKILL §11.10
//! lists those formats as the v1 cooked-texture target.
//!
//! ## Scope of THIS crate (Fase 1)
//!
//! - **Parse a `.ktx2` byte buffer** into a typed [`Ktx2Image`] (format,
//!   dimensions, per-mip bytes).
//! - **Map VkFormat → [`Ktx2Format`]** for the subset PH2D will use,
//!   surfacing the rest as [`Ktx2Format::Unsupported`] so callers can
//!   make an explicit decision rather than silently mis-decoding.
//! - **Reject unsafe inputs** (oversize dimensions, oversize total
//!   payload, supercompression that we have not wired) — mirrors the
//!   defensive limits already in [`ph2d_asset`'s PNG/JPEG path].
//!
//! Out of scope for Fase 1 — these need foundational edits in
//! `ph2d-asset` / `ph2d-render` and will land behind an ADR:
//!
//! - No conversion to `Asset::*` variants — caller owns that mapping.
//! - No `wgpu::TextureFormat` mapping — `ph2d-render` decides per
//!   pipeline (atlas vs individual, sRGB vs linear).
//! - No supercompression decode (zstd / BasisLZ supercompression
//!   schemes are rejected with an explicit error).
//! - No transcoding (BC7 stays BC7; no runtime decode-to-RGBA8).
//! - No writer / cooker — read path only.
//!
//! ## Two-world placement (ADR-0021)
//!
//! This crate runs at **load time**, never in the render / physics /
//! audio hot path (HR-3). One owned heap allocation per mip level is
//! acceptable here; the engine's hot-path code consumes the decoded
//! mip slices read-only.

mod decode;
mod error;
mod format;
mod image;
mod limits;
mod patch;

#[cfg(test)]
mod tests;

pub use decode::{decode_ktx2_bytes, encode_uncompressed_rgba8};
pub use error::Ktx2Error;
pub use format::Ktx2Format;
pub use image::{Ktx2Image, MipLevel, PremulIntent};
pub use limits::{
    MAX_DIMENSION, MAX_KVD_ENTRIES, MAX_KVD_KEY_BYTES, MAX_KVD_VALUE_BYTES, MAX_LEVELS,
    MAX_TOTAL_BYTES, PH2D_PREMUL_KEY,
};
pub use patch::{Ktx2PatchError, encode_premul_value, patch_premul_intent};
