//! ABI size gate for the v4 `RenderInstance` (Sprite_projeto §10.5 /
//! §10.10 / §1.7). Pins the GPU instance stride at **144 bytes** so an
//! accidental field add/remove/reorder can't silently change the upload
//! bandwidth or desync the `vertex_attr_array!` offsets from the struct.
//!
//! This complements the in-crate `vertex_attr_offsets_match_struct`
//! (offset correctness) and the W1.T1.12 `architecture_sprite_inspector_surface`
//! (field-count cap). Here we pin only the total byte size — the single
//! number the spec freezes (§10.11: "RenderInstance size_of: 144 bytes
//! (v4 FROZEN)"). A bump requires ADR-0070-amendment-N.

use ph2d_render::RenderInstance;

#[test]
fn render_instance_pod_size_is_144_bytes_v4() {
    assert_eq!(
        std::mem::size_of::<RenderInstance>(),
        144,
        "RenderInstance v4 ABI is FROZEN at 144 bytes (Sprite_projeto §10.5/§10.11). \
         A change here doubles/shifts the instance-buffer stride and desyncs the \
         vertex layout — bump requires ADR-0070-amendment-N + a re-bench of \
         sprites_upload_144b_vs_72b."
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
