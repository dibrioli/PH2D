#![forbid(unsafe_code)]
//! `ph2d-imageio` — foundational image I/O contract (ADR-0054, W0.T1).
//!
//! Defines what every `crates/ph2d-imageio-<slug>/` satellite crate
//! implements:
//!
//! - [`ImageImporter`] — bytes → [`DecodedImage`].
//! - [`ImageExporter`] — [`DecodedImage`] → bytes.
//! - [`DecodedImage`] — 5-variant canonical decode result (LDR flat,
//!   HDR flat, layered, animated, vector). **Cap FROZEN at 5.**
//! - [`ImageBuffer<P>`] — row-major RGBA pixel array carrying a
//!   [`ColorProfile`].
//! - [`ImportOpts`] / [`ExportOpts`] — caller preferences. **Cap
//!   FROZEN at 6 fields for `ExportOpts`.**
//! - [`Error`] — 8-variant failure taxonomy. **Cap FROZEN at 8.**
//! - [`ColorProfile`] — 8-variant color-space tag. **Cap FROZEN at 8.**
//! - [`MagicHint`] — input shape for [`ImageImporter::supports`].
//!
//! Plus runtime collections in [`registry`]:
//!
//! - [`ImporterRegistry`] — boot-time `Vec<Box<dyn ImageImporter>>`.
//! - [`ExporterRegistry`] — boot-time `Vec<Box<dyn ImageExporter>>`.
//!
//! ## What's NOT here
//!
//! Concrete format support. PNG / JPEG / WebP / GIF / PSD / EXR / SVG /
//! etc. live in their own `crates/ph2d-imageio-<slug>/` and depend on
//! this crate for the contract. The runtime registry is filled by
//! `ph2d_imageio_registry_init::register_all_{importers,exporters}` —
//! a codegen'd aggregation point (W0.T3, scanned by
//! `tools/ph2d-imageio-sync`, W0.T2).
//!
//! ## Invariants
//!
//! - **HR-1** platform-agnostic — no `#[cfg(target_os = ...)]`, no
//!   `std::fs`, no sockets. The shell hands `&[u8]`; the importer never
//!   reads from disk on its own.
//! - **HR-3** not hot — `import` / `export` are user-click triggered,
//!   never per-frame. Allocation is free.
//! - **HR-6** asset = blake3 — the importer doesn't compute the hash,
//!   but the bytes it consumed are blake3-hashed by the caller before
//!   `AssetDb` insertion (the importer's `Result<DecodedImage>` is
//!   keyed on that hash by the cache).
//! - **ADR-0022** no `HashMap` in public surface — lookup is linear over
//!   `Vec` (N stays in low double digits).
//!
//! The arch-gate in `tests/architecture_imageio_contract_surface.rs`
//! locks the surface size (method counts, variant counts, field counts)
//! per ADR-0054 §2.1. Any growth = Coord-A only + amendment ADR.

mod buffer;
mod color;
mod decoded;
mod error;
mod exporter;
mod importer;
mod opts;
mod registry;

pub use buffer::ImageBuffer;
pub use color::ColorProfile;
pub use decoded::{AnimFrame, BlendMode, DecodedImage, Layer, LayerStack, VectorDoc};
pub use error::Error;
pub use exporter::ImageExporter;
pub use importer::ImageImporter;
pub use opts::{ExportOpts, ImportOpts, MagicHint, ToneMap};
pub use registry::{ExporterRegistry, ImporterRegistry};
