//! **Gradient ▸ Palette ramp** (decisão do dono P8 §9.1; lei da spec §13.3 do MVP): o arrasto
//! caminha pelas cores da paleta entre fg e bg; entre dois degraus, a matriz escolhe.

use ph2d_pixel_core::dither::Dither;
use ph2d_pixel_core::gradient::{Gradient, Shape, palette_ramp};

const RAMP: [[u8; 4]; 5] = [
    [0, 0, 0, 255],
    [60, 0, 0, 255],
    [120, 0, 0, 255],
    [180, 0, 0, 255],
    [240, 0, 0, 255],
];

fn drag(dither: Dither) -> Gradient {
    Gradient {
        shape: Shape::Linear,
        from: (0, 0),
        to: (32, 0),
        fg: RAMP[0],
        bg: RAMP[4],
        dither,
    }
}

#[test]
fn the_ends_are_pinned_and_every_cell_is_a_rung() {
    for d in Dither::ALL {
        let g = drag(d);
        assert_eq!(g.ramp_color_at(&RAMP, (-5, 3)), RAMP[0], "{d:?} antes de A");
        assert_eq!(
            g.ramp_color_at(&RAMP, (40, 3)),
            RAMP[4],
            "{d:?} depois de B"
        );
        for x in 0..=32 {
            for y in 0..8 {
                assert!(
                    RAMP.contains(&g.ramp_color_at(&RAMP, (x, y))),
                    "{d:?} ({x},{y})"
                );
            }
        }
    }
}

/// Exactamente num degrau a célula é sólida; a meio caminho, a matriz dá metade de cada.
#[test]
fn on_a_rung_is_solid_and_halfway_is_half_and_half() {
    let g = drag(Dither::Bayer4);
    for y in 0..4 {
        assert_eq!(
            g.ramp_color_at(&RAMP, (8, y)),
            RAMP[1],
            "x = 8 é o degrau 1"
        );
        assert_eq!(
            g.ramp_color_at(&RAMP, (16, y)),
            RAMP[2],
            "x = 16 é o degrau 2"
        );
    }
    // x = 12 está a meio entre os degraus 1 e 2 (f = 1,5): numa coluna da Bayer 4, 2 de cada.
    let col: Vec<[u8; 4]> = (0..4).map(|y| g.ramp_color_at(&RAMP, (12, y))).collect();
    assert_eq!(col.iter().filter(|&&c| c == RAMP[2]).count(), 2, "{col:?}");
    assert_eq!(col.iter().filter(|&&c| c == RAMP[1]).count(), 2, "{col:?}");
}

/// Sem matriz: o degrau mais próximo (sem pontilhado).
#[test]
fn without_a_matrix_the_nearest_rung_wins() {
    let g = drag(Dither::None);
    assert_eq!(g.ramp_color_at(&RAMP, (11, 0)), RAMP[1], "f = 1,375");
    assert_eq!(g.ramp_color_at(&RAMP, (12, 0)), RAMP[2], "f = 1,5 sobe");
}

#[test]
fn the_palette_ramp_runs_from_fg_to_bg_either_way() {
    let pal: Vec<[u8; 4]> = (0..8u8).map(|i| [i * 30, 0, 0, 255]).collect();
    assert_eq!(palette_ramp(&pal, 2, 5), pal[2..=5].to_vec());
    let back: Vec<[u8; 4]> = pal[2..=5].iter().rev().copied().collect();
    assert_eq!(palette_ramp(&pal, 5, 2), back);
    assert_eq!(palette_ramp(&pal, 3, 3), vec![pal[3]]);
}
