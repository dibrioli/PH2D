//! The gates of WHERE A SPATIAL EXTENT LIVES (`compute/units.rs`, `docs/3D/30` §14): the grid is
//! the slider of always, AO BIT; a surface maps the same thumb to its own units and reads the
//! number in % of its size; and the list of the spatial slots is the one the mapping obeys.

use super::*;

/// One param of each kind that has sliders (the defaults, nudged off zero).
fn uma_de_cada() -> Vec<AdjustmentParams> {
    AdjustmentKind::ALL
        .iter()
        .map(|&k| {
            let mut p = AdjustmentParams::neutral_for(k);
            for slot in 0..adjustment_slider_params(&p).len() {
                set_adjustment_slider_param(&mut p, slot, 0.37);
            }
            p
        })
        .collect()
}

/// ⭐⭐ **The grid is the slider of always, ao bit** — `Pixels` changes nothing, and the values
/// are the px the 2D always had (a full travel of the blur reaches `100 px`, the sharpen radius
/// `20 px`).
#[test]
fn the_grid_units_are_the_slider_of_always() {
    for p in uma_de_cada() {
        assert_eq!(
            adjustment_slider_params_in(&p, SpatialUnits::Pixels),
            adjustment_slider_params(&p)
        );
        assert_eq!(
            adjustment_slider_numbers_in(&p, SpatialUnits::Pixels),
            adjustment_slider_numbers(&p)
        );
    }
    let mut g = AdjustmentParams::GaussianBlur(GaussianBlurParams::default());
    set_adjustment_slider_param(&mut g, 0, 1.0);
    assert_eq!(
        g,
        AdjustmentParams::GaussianBlur(GaussianBlurParams { radius: 100.0 })
    );
    let mut s = AdjustmentParams::Sharpen(SharpenParams::default());
    set_adjustment_slider_param(&mut s, 1, 1.0);
    let AdjustmentParams::Sharpen(sp) = s else {
        unreachable!()
    };
    assert_eq!(sp.radius, 20.0);
}

/// ⭐⭐⭐ **A surface maps the thumb to its own units**: a full travel of the blur reaches
/// `SURFACE_RADIUS_MAX · size` (the sharpen radius a fifth of it, the grid's proportion), and the
/// number reads `% of the size`.
#[test]
fn a_surface_maps_the_thumb_to_its_own_units() {
    let u = SpatialUnits::Surface { size: 4.0 };
    let mut g = AdjustmentParams::GaussianBlur(GaussianBlurParams::default());
    set_adjustment_slider_param_in(&mut g, 0, 1.0, u);
    let AdjustmentParams::GaussianBlur(gp) = g else {
        unreachable!()
    };
    assert!(
        (gp.radius - SURFACE_RADIUS_MAX * 4.0).abs() < 1e-7,
        "{}",
        gp.radius
    );
    let mut s = AdjustmentParams::Sharpen(SharpenParams::default());
    set_adjustment_slider_param_in(&mut s, 1, 1.0, u);
    let AdjustmentParams::Sharpen(sp) = s else {
        unreachable!()
    };
    assert!(
        (sp.radius - SURFACE_RADIUS_MAX * 4.0 * 0.2).abs() < 1e-7,
        "{}",
        sp.radius
    );
    assert_eq!(
        adjustment_slider_numbers_in(&g, u)[0],
        SliderNumber::Affine {
            scale: SURFACE_RADIUS_MAX * 100.0,
            offset: 0.0,
            integer: false
        }
    );
    assert!((adjustment_slider_params_in(&g, u)[0].1 - 1.0).abs() < 1e-6);
}

/// ⭐⭐ **The spatial slots are exactly the slots whose thumb moves between units** — a list that
/// forgot one would seed a surface's new adjustment in px (a `20`-unit Bloom on a piece of
/// diagonal `3,5`).
#[test]
fn the_spatial_slots_are_the_ones_the_units_move() {
    let u = SpatialUnits::Surface { size: 7.0 };
    for p in uma_de_cada() {
        let (a, b) = (
            adjustment_slider_params_in(&p, SpatialUnits::Pixels),
            adjustment_slider_params_in(&p, u),
        );
        let moved: Vec<usize> = (0..a.len()).filter(|&i| a[i].1 != b[i].1).collect();
        let declared: Vec<usize> = spatial_extent_slots(&p)
            .iter()
            .copied()
            .filter(|_| !p.kind().reads_the_image_plane())
            .collect();
        assert_eq!(moved, declared, "{:?}", p.kind());
    }
}

/// ⭐ **Rescaling keeps every slider where it was** — the seed of a surface's new adjustment.
#[test]
fn rescaling_keeps_the_thumbs() {
    let u = SpatialUnits::Surface { size: 3.2 };
    for mut p in uma_de_cada() {
        let antes = adjustment_slider_params(&p);
        rescale_spatial_params(&mut p, SpatialUnits::Pixels, u);
        let depois = adjustment_slider_params_in(&p, u);
        for (a, b) in antes.iter().zip(&depois) {
            assert!((a.1 - b.1).abs() < 1e-5, "{:?}: {a:?} → {b:?}", p.kind());
        }
    }
}
