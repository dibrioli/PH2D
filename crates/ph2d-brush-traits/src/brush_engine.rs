//! `BrushEngine` trait — the behavior interface bridging Painter and
//! Vector consumers.
//!
//! Per [ADR-0067 §2.3](../../../../docs/architecture/decisions/0067-brush-traits-decoupling.md).
//! Caps congelados: **3 methods** (`stamp` + `brush_handle` + `display_name`).
//! Adding a method requires an `0067-amendment-N.md` ADR.

use crate::{BrushRef, StampSpec};
use std::borrow::Cow;

/// Brush behavior interface — what consumers (Vector
/// pattern-along-path, Painter stroke pipeline) call.
///
/// ## `Target` associated type
///
/// Implementors pick their own target texture type — Painter binds to
/// `wgpu::Texture`, Vector preview nodes may bind to an in-memory
/// raster buffer, headless tests bind to `()`. This keeps the trait
/// free of GPU deps (the ADR-0067 mandate).
///
/// Trait objects pin the target: `Box<dyn BrushEngine<Target = T>>`.
///
/// ## Thread-safety policy
///
/// Does **not** require `Send + Sync` as a supertrait. Painter brush
/// libraries that need to be shared across threads (W2+ stroke
/// pipeline running off the UI thread) pin the marker at the
/// trait-object boundary:
///
/// ```ignore
/// type SharedBrush<T> = Arc<dyn BrushEngine<Target = T> + Send + Sync>;
/// ```
///
/// Single-thread Vector preview nodes can hold non-thread-safe state
/// (e.g. small caches) in their impls without forcing all callers to
/// pay for it. R4 audit Lens-L policy.
pub trait BrushEngine {
    /// Concrete texture / raster target this engine stamps onto.
    type Target;

    /// Stamp the brush at one sample point into `target`.
    fn stamp(&self, target: &mut Self::Target, sample: StampSpec);

    /// Stable persistent reference handle for this brush instance.
    fn brush_handle(&self) -> BrushRef;

    /// Human-readable display name for UI lists. Default returns a
    /// generic "Unnamed Brush" — impls override.
    fn display_name(&self) -> Cow<'_, str> {
        Cow::Borrowed("Unnamed Brush")
    }
}
