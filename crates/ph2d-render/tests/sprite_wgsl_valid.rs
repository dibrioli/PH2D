//! WGSL gate for `src/shaders/sprite.wgsl` — parse + validate via naga
//! at `cargo test` time so a shader / vertex-layout drift is caught
//! before a GPU pipeline build (which only happens at runtime / smoke).
//!
//! Motivated by ADR-0070-amendment-4 (`rotation: f32` → `basis:
//! [f32; 4]`): the WGSL `InstanceInput.basis` and the Rust
//! `RenderInstance::VERTEX_ATTRIBUTES` `@location(6)` must agree, or the
//! shader silently reads the wrong bytes. Mirrors ph2d-painter-brush's
//! `stamp_wgsl_validates_via_naga`.

const SPRITE_WGSL: &str = include_str!("../src/shaders/sprite.wgsl");

#[test]
fn sprite_wgsl_parses_and_validates() {
    let module = naga::front::wgsl::parse_str(SPRITE_WGSL).unwrap_or_else(|e| {
        panic!(
            "sprite.wgsl failed naga parse:\n{}",
            e.emit_to_string(SPRITE_WGSL)
        )
    });
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    validator
        .validate(&module)
        .unwrap_or_else(|e| panic!("sprite.wgsl failed naga validation: {e:?}"));
}

#[test]
fn sprite_wgsl_instance_basis_is_vec4_at_location_6() {
    // The @location(6) instance input MUST be a 4-component f32 vector to
    // match `RenderInstance::basis: [f32; 4]` (vertex_attr `6 =>
    // Float32x4`). If a future edit shrinks it back to a scalar
    // `rotation`, the GPU would read 12 bytes of the next field as
    // garbage — pin it here.
    let module = naga::front::wgsl::parse_str(SPRITE_WGSL).expect("parse");
    // Find the global/struct member carrying @location(6) by scanning the
    // struct types for a member binding at location 6.
    let mut found_vec4_at_6 = false;
    for (_h, ty) in module.types.iter() {
        if let naga::TypeInner::Struct { members, .. } = &ty.inner {
            for m in members {
                if let Some(naga::Binding::Location { location: 6, .. }) = m.binding {
                    let inner = &module.types[m.ty].inner;
                    if let naga::TypeInner::Vector {
                        size: naga::VectorSize::Quad,
                        scalar,
                        ..
                    } = inner
                    {
                        assert_eq!(
                            scalar.kind,
                            naga::ScalarKind::Float,
                            "@location(6) must be a float vector (basis)"
                        );
                        found_vec4_at_6 = true;
                    }
                }
            }
        }
    }
    assert!(
        found_vec4_at_6,
        "sprite.wgsl @location(6) is not a vec4<f32> — the 2x2 `basis` instance \
         attribute (ADR-0070-amendment-4) is missing or the wrong shape"
    );
}
