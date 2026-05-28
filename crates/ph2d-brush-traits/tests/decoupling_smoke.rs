//! Decoupling smoke per ADR-0067:
//! - This crate compiles against neither `ph2d-painter-*` nor
//!   `ph2d-vector-*` (verified by Cargo.toml inspection at the
//!   workspace level — `cargo tree -p ph2d-brush-traits` must show
//!   only `serde`, `glam`, `ph2d-color` as direct deps).
//! - The trait is object-safe given a pinned `Target`.
//! - Default `display_name` is overrideable.

use std::borrow::Cow;

use glam::Vec2;
use ph2d_brush_traits::{BrushEngine, BrushRef, StampSpec};

/// Minimal in-test impl over a `Vec<StampSpec>` capture buffer so we
/// can exercise the trait surface without GPU deps.
struct TestEngine {
    handle: BrushRef,
    label: &'static str,
}

impl BrushEngine for TestEngine {
    type Target = Vec<StampSpec>;

    fn stamp(&self, target: &mut Self::Target, sample: StampSpec) {
        target.push(sample);
    }

    fn brush_handle(&self) -> BrushRef {
        self.handle
    }

    fn display_name(&self) -> Cow<'_, str> {
        Cow::Borrowed(self.label)
    }
}

#[test]
fn brush_engine_records_stamps_into_target_buffer() {
    let engine = TestEngine {
        handle: BrushRef::from_u64(0xCAFE),
        label: "Test Pencil 2B",
    };
    let mut buf: Vec<StampSpec> = Vec::new();
    engine.stamp(
        &mut buf,
        StampSpec {
            pos: Vec2::new(10.0, 20.0),
            ..StampSpec::default()
        },
    );
    engine.stamp(
        &mut buf,
        StampSpec {
            pos: Vec2::new(30.0, 40.0),
            ..StampSpec::default()
        },
    );
    assert_eq!(buf.len(), 2);
    assert_eq!(buf[0].pos, Vec2::new(10.0, 20.0));
    assert_eq!(buf[1].pos, Vec2::new(30.0, 40.0));
    assert_eq!(engine.brush_handle(), BrushRef::from_u64(0xCAFE));
    assert_eq!(engine.display_name(), Cow::Borrowed("Test Pencil 2B"));
}

#[test]
fn brush_engine_is_object_safe_with_pinned_target() {
    let engine = TestEngine {
        handle: BrushRef::from_u64(1),
        label: "obj-safe",
    };
    let boxed: Box<dyn BrushEngine<Target = Vec<StampSpec>>> = Box::new(engine);
    let mut buf = Vec::new();
    boxed.stamp(&mut buf, StampSpec::default());
    assert_eq!(buf.len(), 1);
    assert_eq!(boxed.brush_handle(), BrushRef::from_u64(1));
}

#[test]
fn brush_ref_none_sentinel_round_trips() {
    let none = BrushRef::NONE;
    assert!(none.is_none());
    let some = BrushRef::from_u64(42);
    assert!(!some.is_none());
}

#[test]
fn stamp_spec_default_is_neutral() {
    let s = StampSpec::default();
    assert_eq!(s.pos, Vec2::ZERO);
    assert_eq!(s.tangent, Vec2::X);
    assert_eq!(s.pressure, 1.0);
    assert_eq!(s.size_scale, 1.0);
    assert_eq!(s.azimuth, 0.0);
}

/// Compile-time assertion that BrushRef implements the serde traits
/// required by `.ph2d-vector` / `.ph2d-brush` postcard schemas — if the
/// derives are lost, this stops compiling. No runtime allocation.
#[test]
fn brush_ref_serde_derives_present() {
    fn assert_serde<T: serde::Serialize + serde::de::DeserializeOwned>() {}
    assert_serde::<BrushRef>();
}
