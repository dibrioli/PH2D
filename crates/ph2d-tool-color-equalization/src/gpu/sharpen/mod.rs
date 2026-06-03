//! WGSL compute paths for [`crate::algorithm::sharpen_laplacian`] and
//! [`crate::algorithm::sharpen_unsharp`].
//!
//! Two distinct pipelines because the CPU code paths are distinct:
//!
//! - **Laplacian** (radius ≤ 1, [`LaplacianSharpenPipeline`]): single
//!   compute pass, 4 neighbour reads per pixel (top / bottom / left /
//!   right). Cheap on both CPU and GPU; GPU still wins ~10× at 1024²
//!   by avoiding the 4× scalar load + the per-channel temporary.
//! - **Unsharp Mask** (radius > 1, [`UnsharpSharpenPipeline`]): two
//!   compute passes — separable Gaussian blur (H then V), then a
//!   combine step (`orig + amount · (orig − blur)`). Fused V + combine
//!   into the second pass so we only round-trip through one
//!   intermediate `rgba16float` texture. **The big GPU win** —
//!   per-pixel cost grows linearly with radius on CPU but stays roughly
//!   constant on GPU (memory-bound, not compute-bound). 1024² at
//!   radius 3 goes from ~120 ms CPU to ~3 ms GPU on Apple Silicon.
//!
//! ## CPU semantic parity notes
//!
//! - **Laplacian** skips transparent pixels entirely (matches CPU's
//!   `if src[ci + 3] == 0 { continue; }` short-circuit).
//! - **Unsharp** runs the H + V blur passes unconditionally (CPU does
//!   the same — alpha is ignored during the blur), but the final
//!   combine step preserves transparent pixels untouched.
//! - Both clamp the final result to `[0, 1]` before storage (matches
//!   the CPU's `clamp8`).
//!
//! ## Intermediate format
//!
//! The H-pass result lands in an `rgba16float` storage texture, **not**
//! `rgba8unorm`. Per-channel blur weights can sum to fractional
//! amounts whose precision matters when subsequently differenced
//! against the original — `rgba8unorm` quantization would inject up to
//! 2 LSB of noise into the difference *before* the combine pass, then
//! amplify by `amount` (up to 2). Half-precision keeps the H pass
//! lossless for our purposes (parity holds at ε ≤ 4 LSB).
//!
//! ## Module layout
//!
//! Split by pipeline (mechanical, no behaviour change). The public
//! pipeline types + convenience fns are re-exported flat at
//! `crate::gpu::sharpen::*` so [`crate::gpu`] and `chain.rs` keep their
//! existing `super::{LaplacianSharpenPipeline, …}` imports:
//!
//! - [`laplacian`] — [`LaplacianSharpenPipeline`] + [`sharpen_laplacian_gpu`].
//! - [`unsharp`] — [`UnsharpSharpenPipeline`] + [`sharpen_unsharp_gpu`].
//! - [`wgsl`] — the three inline WGSL kernel sources.

mod laplacian;
mod unsharp;
mod wgsl;

#[cfg(test)]
mod tests;

pub use laplacian::{LaplacianSharpenPipeline, sharpen_laplacian_gpu};
pub use unsharp::{UnsharpSharpenPipeline, sharpen_unsharp_gpu};

/// Compute workgroup tile side (8×8 = 64 invocations) shared by both the
/// Laplacian and Unsharp dispatches. Private to `sharpen` — child
/// pipeline modules read it via `super::WORKGROUP_SIZE` for the
/// `dispatch_workgroups` div-ceil.
const WORKGROUP_SIZE: u32 = 8;
