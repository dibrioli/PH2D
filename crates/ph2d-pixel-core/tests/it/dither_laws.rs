//! **O dither ordenado** contra o Trial 1.3.18.6 (o Gradient da dinâmica com Bayer 4×4,
//! `oracle_trial_dynamics_velocity.json`): a ordem em que as células acendem, presa ao canvas.

use ph2d_pixel_core::dither::Dither;

#[test]
fn bayer4_lights_cells_in_the_order_the_trial_showed() {
    // y=8 (nível 0), y=10 x≡2 (1), y=28 x par (2), y=30 x≡0 (3), y=29 x≡1 (4), y=31 x≡3 (5).
    for (lvl, (x, y)) in [(0, 0), (2, 2), (2, 0), (0, 2), (1, 1), (3, 3)]
        .into_iter()
        .enumerate()
    {
        assert_eq!(Dither::Bayer4.level(x, y), lvl as u32, "({x},{y})");
        assert_eq!(
            Dither::Bayer4.level(x + 48, y + 8),
            lvl as u32,
            "presa ao canvas"
        );
    }
    assert_eq!(Dither::Bayer2.level(1, 0), 2);
    assert_eq!(Dither::Bayer2.level(0, 1), 3);
}

#[test]
fn every_matrix_is_a_permutation_and_reaches_grows_by_one_cell_per_step() {
    for d in Dither::ALL {
        let n = d.side() as i32;
        let mut seen: Vec<u32> = (0..n)
            .flat_map(|y| (0..n).map(move |x| d.level(x, y)))
            .collect();
        seen.sort_unstable();
        assert_eq!(seen, (0..(n * n) as u32).collect::<Vec<_>>(), "{d:?}");
        for k in 0..=(n * n) {
            let t = f64::from(k) / f64::from(n * n);
            let on = (0..n)
                .flat_map(|y| (0..n).map(move |x| (x, y)))
                .filter(|&(x, y)| d.reaches(x, y, t))
                .count();
            assert_eq!(on as i32, k, "{d:?} t={t}");
        }
    }
}
