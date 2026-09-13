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
