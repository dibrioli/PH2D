//! Gates do recorte de grid ([`super`]).
//!
//! ⚠️ O oráculo é o **snapshot de folha inteira** que já existia: um recorte, restaurado, tem de
//! devolver o grid ao estado que o snapshot completo devolveria — dentro da janela, e sem tocar em
//! nada fora dela. Comparar contra ele é o que impede as duas metades de divergirem sobre *o que é
//! estado do grid*.

use super::{restore_grid_region, snapshot_grid_region};
use crate::grid::{Grid, restore_grid, snapshot_grid};

/// Uma folha pequena com água em toda parte — a fixture TEM de conter o fenômeno: um grid vazio
/// deixaria todo gate verde por vácuo.
fn wet_sheet(rf: usize) -> Grid {
    let mut g = Grid::with_flow_ratio(24, 16, rf);
    for i in 0..g.cells {
        let f = i as f32;
        g.film[i] = f * 0.01;
        g.susp[i] = f * 0.02;
        g.susp_rgb[i] = [f, f * 0.5, f * 0.25];
        g.sett[i] = f * 0.03;
        g.sett_rgb[i] = [f * 0.25, f, f * 0.5];
        g.wet[i] = (i % 251) as u8;
        g.active[i] = (i % 2) as u8;
        g.bloom[i] = (i % 97) as u8;
    }
    for i in 0..g.flow.cells {
        g.vel_x[i] = i as f32 * 0.1;
        g.vel_y[i] = i as f32 * -0.1;
    }
    g
}

/// **A ida e volta é a identidade** — recortar, sujar a janela e restaurar devolve o grid EXATAMENTE
/// como estava, e o oráculo é o snapshot de folha inteira.
#[test]
fn a_patch_round_trip_restores_the_window_exactly() {
    for rf in [1usize, 3] {
        let mut g = wet_sheet(rf);
        let before = snapshot_grid(&g);
        let patch = snapshot_grid_region(&g, 5, 4, 14, 11).expect("a janela intersecta o interior");
        // Suja a janela inteira (e só ela).
        for y in 4..=11 {
            for x in 5..=14 {
                let i = y as usize * g.s + x as usize;
                g.film[i] = 99.0;
                g.susp[i] = 99.0;
                g.susp_rgb[i] = [1.0, 2.0, 3.0];
                g.sett[i] = 99.0;
                g.sett_rgb[i] = [4.0, 5.0, 6.0];
                g.wet[i] = 7;
                g.active[i] = 1;
                g.bloom[i] = 9;
            }
        }
        for i in 0..g.flow.cells {
            g.vel_x[i] = 42.0;
            g.vel_y[i] = -42.0;
        }
        restore_grid_region(&mut g, &patch);
        // Os oito planos finos voltam ao byte DENTRO da janela…
        for y in 4..=11 {
            for x in 5..=14 {
                let i = y as usize * g.s + x as usize;
                assert_eq!(g.film[i], before.film[i], "film em ({x},{y}), rf={rf}");
                assert_eq!(g.susp[i], before.susp[i], "susp em ({x},{y}), rf={rf}");
                assert_eq!(g.susp_rgb[i], before.susp_rgb[i]);
                assert_eq!(g.sett[i], before.sett[i]);
                assert_eq!(g.sett_rgb[i], before.sett_rgb[i]);
                assert_eq!(g.wet[i], before.wet[i]);
                assert_eq!(g.active[i], before.active[i]);
                assert_eq!(g.bloom[i], before.bloom[i]);
            }
        }
        // …e os dois persistentes de fluxo, na janela projetada.
        let (fx0, fy0, fx1, fy1) = (
            ((5 - 1) / rf as i32).max(0),
            ((4 - 1) / rf as i32).max(0),
            (14 - 1) / rf as i32 + 1,
            (11 - 1) / rf as i32 + 1,
        );
        for fy in fy0.max(1)..=fy1.min(g.flow.h as i32) {
            for fx in fx0.max(1)..=fx1.min(g.flow.w as i32) {
                let i = fy as usize * g.flow.s + fx as usize;
                assert_eq!(g.vel_x[i], before.vel_x[i], "vel_x em ({fx},{fy}), rf={rf}");
                assert_eq!(g.vel_y[i], before.vel_y[i]);
            }
        }
    }
}

/// **Fora da janela ele não toca em NADA** — é o que separa um recorte de um snapshot, e a metade que
/// um gate de ida-e-volta sozinho não vê.
///
/// **Mutação que deve sangrar:** um `put_rows` que erre o stride (escreve linhas vizinhas).
#[test]
fn a_patch_leaves_everything_outside_its_window_alone() {
    let mut g = wet_sheet(1);
    let patch = snapshot_grid_region(&g, 5, 4, 14, 11).expect("janela");
    // Um valor sentinela FORA da janela, que o restore não pode alcançar.
    let out = 3usize * g.s + 3; // (3, 3)
    g.film[out] = -7.5;
    g.susp[out] = -8.5;
    g.wet[out] = 200;
    restore_grid_region(&mut g, &patch);
    assert_eq!(g.film[out], -7.5, "o restore invadiu a vizinhança (film)");
    assert_eq!(g.susp[out], -8.5, "o restore invadiu a vizinhança (susp)");
    assert_eq!(g.wet[out], 200, "o restore invadiu a vizinhança (wet)");
}

/// O recorte **clampa** ao interior e recusa uma janela que não o alcança — a guarda que impede um
/// bbox de dabs fora da folha de virar um índice.
#[test]
fn a_window_outside_the_sheet_is_refused_and_a_straddling_one_is_clamped() {
    let g = wet_sheet(1);
    assert!(snapshot_grid_region(&g, -50, -50, -10, -10).is_none());
    let p = snapshot_grid_region(&g, -5, -5, 3, 3).expect("a janela cruza a borda");
    assert_eq!(p.rect(), (1, 1, 3, 3), "clampada ao interior");
    assert_eq!(p.cells(), 9);
}

/// **O snapshot de folha inteira segue sendo o superconjunto**: restaurar a folha depois de restaurar
/// um recorte não muda nada — as duas portas descrevem o MESMO estado.
#[test]
fn the_full_snapshot_still_agrees_with_the_patch() {
    let mut g = wet_sheet(2);
    let full = snapshot_grid(&g);
    let patch = snapshot_grid_region(&g, 2, 2, 20, 14).expect("janela");
    for i in 0..g.cells {
        g.film[i] = 1.0;
    }
    restore_grid_region(&mut g, &patch);
    let after_patch: Vec<f32> = g.film.clone();
    restore_grid(&mut g, &full);
    for y in 2..=14usize {
        for x in 2..=20usize {
            let i = y * g.s + x;
            assert_eq!(
                after_patch[i], g.film[i],
                "o recorte e o snapshot discordam em ({x},{y})"
            );
        }
    }
}
