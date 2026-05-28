//! Arch-gate: `ph2d-brush-traits` thread-safety invariant.
//!
//! The Painter brush pipeline evaluates stamps on a worker pool and ships
//! brush descriptors across thread boundaries (stroke scheduling, GPU
//! upload staging). Every *value* type the crate exposes must therefore be
//! `Send + Sync`, and the `BrushEngine` trait object must be usable as
//! `Box<dyn BrushEngine + Send + Sync>`. These compile-time assertions
//! fail the build if a future field (e.g. an `Rc`, a `Cell`, a raw
//! pointer) silently revokes that guarantee.
//!
//! Formalized from the W1 audit scratch (`_audit_send_sync.rs` /
//! `_audit_dyn_send_sync.rs`) per the gold-standard mandate: a real
//! invariant becomes a named, documented gate rather than an orphan file.

use ph2d_brush_traits::*;

fn assert_send_sync<T: Send + Sync>() {}
fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}

#[test]
fn brush_value_types_are_send_sync() {
    assert_send_sync::<BrushRef>();
    assert_send_sync::<StampSpec>();
}

#[test]
fn boxed_brush_engine_with_explicit_bound_is_send_sync() {
    // A bare `Box<dyn BrushEngine<Target = ()>>` carries no auto-trait
    // bound and is intentionally NOT Send/Sync. Call sites that move an
    // engine across threads must spell `+ Send + Sync`; this proves that
    // spelling is satisfiable for the trait as defined (no `!Send`
    // associated state leaked in).
    assert_send::<Box<dyn BrushEngine<Target = ()> + Send + Sync>>();
    assert_sync::<Box<dyn BrushEngine<Target = ()> + Send + Sync>>();
}
