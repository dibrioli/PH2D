//! Testes de `renderer.rs` — irmão por tecto de LOC (`workspace_src_files_under_loc_cap`).
//!
//! Corte mecânico: o `mod tests` saiu inteiro, verbatim, do ficheiro que o continha.

use super::*;

fn inst(texture_id: u32) -> RenderInstance {
    RenderInstance {
        world_pos: [0.0, 0.0],
        size: [1.0, 1.0],
        atlas_uv: [0.0, 0.0, 1.0, 1.0],
        tint: [1.0, 1.0, 1.0, 1.0],
        basis: RenderInstance::IDENTITY_BASIS,
        texture_id,
        premultiplied: 0.0,
        anchor: [0.0, 0.0],
        per_corner_tint: [[1.0; 4]; 4],
        opacity: 1.0,
        flip_uv: 0,
        z_order: 0,
        sampling: 0,
        uv_xform: RenderInstance::IDENTITY_UV_XFORM,
        clip_group: RenderInstance::CLIP_GROUP_NONE,
        clip_meta: 0,
        sub_order: 0,
    }
}

#[test]
fn compute_runs_empty_input_emits_no_runs() {
    let mut runs = Vec::new();
    compute_runs(&[], &mut runs);
    assert!(runs.is_empty());
}

#[test]
fn compute_runs_groups_consecutive_same_texture() {
    // Pre-sorted: [0, 0, 0, 7, 7, 12]
    let scratch = [inst(0), inst(0), inst(0), inst(7), inst(7), inst(12)];
    let mut runs = Vec::new();
    compute_runs(&scratch, &mut runs);
    assert_eq!(
        runs,
        vec![
            DrawRun {
                texture_id: 0,
                sampling: 0,
                start: 0,
                end: 3,
                clip_group: 0,
                clip_role: 0,
                mask_role: 0,
                blend: 0,
                mesh: 0,
            },
            DrawRun {
                texture_id: 7,
                sampling: 0,
                start: 3,
                end: 5,
                clip_group: 0,
                clip_role: 0,
                mask_role: 0,
                blend: 0,
                mesh: 0,
            },
            DrawRun {
                texture_id: 12,
                sampling: 0,
                start: 5,
                end: 6,
                clip_group: 0,
                clip_role: 0,
                mask_role: 0,
                blend: 0,
                mesh: 0,
            },
        ]
    );
}

/// ⭐ **Uma instância desenhada como MALHA é um run só dela** — mesmo com a textura, a amostragem e a
/// mistura iguais às das vizinhas. Fundida num run de quads, ela seria desenhada como quad (ou as
/// vizinhas como a malha dela).
#[test]
fn a_mesh_instance_is_its_own_run_even_with_the_same_texture() {
    let mut com_malha = inst(7);
    com_malha.flip_uv |= 1 << RenderInstance::MESH_SHIFT;
    let scratch = [inst(7), com_malha, inst(7)];
    let mut runs = Vec::new();
    compute_runs(&scratch, &mut runs);
    let resumo: Vec<(u32, u32, u32)> = runs.iter().map(|r| (r.start, r.end, r.mesh)).collect();
    assert_eq!(resumo, vec![(0, 1, 0), (1, 2, 1), (2, 3, 0)]);
}

#[test]
fn compute_runs_singleton_per_instance() {
    // All distinct textures → N runs of 1.
    let scratch = [inst(1), inst(2), inst(3)];
    let mut runs = Vec::new();
    compute_runs(&scratch, &mut runs);
    assert_eq!(runs.len(), 3);
    for r in &runs {
        assert_eq!(r.end - r.start, 1);
    }
}

#[test]
fn compute_runs_reuses_vec_capacity() {
    // First call grows the vec; second call with a different
    // shape must NOT allocate again (HR-3 alloc-free hot path).
    let mut runs = Vec::with_capacity(8);
    let cap = runs.capacity();
    compute_runs(&[inst(0), inst(0)], &mut runs);
    compute_runs(&[inst(5), inst(6), inst(7), inst(7)], &mut runs);
    assert!(runs.capacity() >= cap, "capacity must not shrink");
    assert_eq!(runs.len(), 3);
}

#[test]
fn compute_runs_atlas_constant() {
    let scratch = [inst(RenderInstance::ATLAS_TEXTURE_ID)];
    let mut runs = Vec::new();
    compute_runs(&scratch, &mut runs);
    assert_eq!(runs[0].texture_id, RenderInstance::ATLAS_TEXTURE_ID);
    assert_eq!(runs[0].texture_id, 0);
}
