use super::sprite_sheet_subrect;

const FULL: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

#[test]
fn default_grid_is_identity() {
    assert_eq!(sprite_sheet_subrect(FULL, 1, 1, 0), FULL);
    // Zero counts floor to 1 → still identity.
    assert_eq!(sprite_sheet_subrect(FULL, 0, 0, 5), FULL);
}

#[test]
fn two_by_two_selects_cells() {
    // frame 0 = top-left, 1 = top-right, 2 = bottom-left, 3 = bottom-right.
    assert_eq!(sprite_sheet_subrect(FULL, 2, 2, 0), [0.0, 0.0, 0.5, 0.5]);
    assert_eq!(sprite_sheet_subrect(FULL, 2, 2, 1), [0.5, 0.0, 1.0, 0.5]);
    assert_eq!(sprite_sheet_subrect(FULL, 2, 2, 2), [0.0, 0.5, 0.5, 1.0]);
    assert_eq!(sprite_sheet_subrect(FULL, 2, 2, 3), [0.5, 0.5, 1.0, 1.0]);
}

#[test]
fn frame_past_grid_clamps_to_last_cell() {
    assert_eq!(sprite_sheet_subrect(FULL, 2, 2, 99), [0.5, 0.5, 1.0, 1.0]);
}

#[test]
fn subrect_respects_a_non_unit_base_rect() {
    // An atlas region [0.2, 0.4, 0.6, 0.8] split 2×1 → left half.
    let base = [0.2, 0.4, 0.6, 0.8];
    let left = sprite_sheet_subrect(base, 2, 1, 0);
    assert!((left[0] - 0.2).abs() < 1e-6 && (left[2] - 0.4).abs() < 1e-6);
    assert!((left[1] - 0.4).abs() < 1e-6 && (left[3] - 0.8).abs() < 1e-6);
}

/// Every frame of an atlas sheet lands on its own texels: the cell starts at `x + w·k` and keeps the cell's width.
/// Owner report 09/11 (idle 128×32, 4×1): cut from the INSET rect, frame k sat `0,5 − 0,25·k` texels off.
#[test]
fn every_atlas_sheet_frame_lands_on_its_own_texels() {
    let atlas = 2048_u32;
    let region = crate::AtlasRegion {
        x: 37,
        y: 5,
        w: 128,
        h: 32,
    };
    let s = atlas as f32;
    let texels = |uv: [f32; 4]| uv.map(|c| c * s);
    for k in 0..4 {
        let [u0, v0, u1, v1] = texels(sprite_sheet_subrect(region.uv_exact(atlas), 4, 1, k));
        let start = 37.0 + 32.0 * k as f32;
        assert!(
            (u0 - start).abs() < 1e-3,
            "frame {k} starts at {u0}, its texels at {start}"
        );
        assert!(
            (u1 - u0 - 32.0).abs() < 1e-3 && (v1 - v0 - 32.0).abs() < 1e-3,
            "frame {k} is {}×{}",
            u1 - u0,
            v1 - v0
        );
        assert!((v0 - 5.0).abs() < 1e-3);
        // The half-texel clip moves the FINAL cell, the same for every frame.
        let [c0, _, c1, _] = texels(super::inset_half_texel(
            sprite_sheet_subrect(region.uv_exact(atlas), 4, 1, k),
            (0.5 / s, 0.5 / s),
        ));
        assert!(
            (c0 - start - 0.5).abs() < 1e-3 && (c1 - start - 31.5).abs() < 1e-3,
            "clipped frame {k}: {c0}..{c1}"
        );
    }
    // Control: the inset rect really does slide (the defect this test guards).
    let slid = texels(sprite_sheet_subrect(region.uv(atlas), 4, 1, 3))[0] - (37.0 + 96.0);
    assert!(
        (slid - (0.5 - 0.75)).abs() < 1e-3,
        "inset rect frame 3 offset {slid}"
    );
}
