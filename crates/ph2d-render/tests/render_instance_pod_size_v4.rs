//! ABI size gate for the `RenderInstance` (Sprite_projeto §10.5 /
//! §10.10 / §1.7). Pins the GPU instance stride so an accidental field
//! add/remove/reorder can't silently change the upload bandwidth or
//! desync the `vertex_attr_array!` offsets from the struct.
//!
//! This complements the in-crate `vertex_attr_offsets_match_struct`
//! (offset correctness) and the W1.T1.12 `architecture_sprite_inspector_surface`
//! (field-count cap). Here we pin only the total byte size. As of
//! ADR-0070-amendment-4 the size is **156 bytes** (`rotation: f32` grew
//! to `basis: [f32; 4]` so the shader renders skew as a true
//! parallelogram instead of a lossy decomposed rotation). A further
//! bump requires ADR-0070-amendment-N + a re-bench.

use ph2d_render::RenderInstance;

#[test]
fn render_instance_pod_size_is_176_bytes() {
    assert_eq!(
        std::mem::size_of::<RenderInstance>(),
        176,
        "RenderInstance ABI is 176 bytes (amendment-6: +GPU `uv_xform: [f32;4]` for UV \
         tiling/scroll → vertex layout 164 B / 12 attrs; amendment-5: +CPU-only `sampling`). \
         A change desyncs the vertex layout — bump requires ADR-0070-amendment-N + a \
         re-bench of sprites_upload_144b_vs_72b."
    );
}

#[test]
fn render_instance_is_four_byte_aligned() {
    // The ABI is documented as 4-byte aligned with no tail padding
    // (§1.7). `bytemuck::Pod` already forbids padding bytes; this pins
    // the alignment the WGSL instance step-mode layout assumes.
    assert_eq!(
        std::mem::align_of::<RenderInstance>(),
        4,
        "RenderInstance must stay 4-byte aligned (all fields f32/u32-grained)"
    );
}
