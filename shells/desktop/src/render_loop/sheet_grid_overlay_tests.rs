//! Gates da grelha desenhada — irmão de [`super`] pelo teto de LOC do shell.
//!
//! ⚠️ **O que se afirma são NÚMEROS, e não pixels.** O traço em si é traço; o que pode estar errado
//! é o sinal do `Y`, o pivô e o espelho — e esses cabem num `assert_eq`.
//!
//! ⚠️ **Os outros sete gates da grelha mudaram-se com ela** para a
//! `ph2d_sprite_screen::sheet_lattice` (2026-09-16). Fica aqui o que CRUZA com os fantasmas da
//! folha, porque o `sim_extract_sheet::cell` mora na shell.

use super::*;
use ph2d_sprite_screen::sheet_lattice::Lattice;

/// Uma sprite de células de `2×2` metros numa grelha `hf × vf`, parada no frame `live`.
///
/// ⚠️ **`Sprite::atlas` nasce CENTRADA**, então o `resolve_anchor` dela é a origem — e é por isso
/// que o gate do pivô abaixo o move de propósito: com o pivô na origem, uma implementação que
/// ignorasse o `resolve_anchor` ficaria verde.
/// ⭐ Devolve o PAR desde o ADR-0164 F1 passo 6: a grelha saiu da `Sprite` para um componente,
/// e o retículo é função dos dois (o tamanho da célula é da sprite; os cortes, da grelha).
fn spr(hf: u32, vf: u32, live: u32) -> (Sprite, ph2d_ecs::SpriteGrid) {
    (
        Sprite::atlas(0, [2.0, 2.0], [1.0; 4]),
        ph2d_ecs::SpriteGrid {
            hframes: hf,
            vframes: vf,
            frame: live,
        },
    )
}

/// O [`lattice`] sobre o par — para os gates lerem como liam.
fn lat(f: &(Sprite, ph2d_ecs::SpriteGrid), ppm: f32, unfolded: bool) -> Option<Lattice> {
    lattice(&f.0, f.1, ppm, unfolded)
}

const PPM: f32 = 100.0;

/// **O retículo e os fantasmas concordam sobre onde cada célula cai.**
///
/// ⚠️ O gate que liga os dois módulos: o `sim_extract_sheet` põe a arte e este põe as linhas, e
/// eles derivam a posição por caminhos diferentes (um por deslocamento relativo, o outro por canto
/// + índice). Se discordarem, as linhas caem no meio dos desenhos — e cada módulo passa sozinho.
#[test]
fn the_lines_land_on_the_cells_the_ghosts_draw() {
    for (hf, vf, live) in [(4u32, 2u32, 0u32), (4, 2, 5), (3, 3, 4), (8, 1, 7)] {
        let s = spr(hf, vf, live);
        let l = lat(&s, PPM, false).unwrap();
        let cells = hf * vf;
        for i in 0..cells {
            // Onde o retículo diz que a célula `i` está (canto + índice, espelho já dentro).
            let (col, row) = (i % hf, i / hf);
            let want_cx = l.x0 + (f64::from(col) + 0.5) * l.cell_w;
            let want_cy = l.y0 - (f64::from(row) + 0.5) * l.cell_h;
            // Onde o fantasma a põe (deslocamento relativo à viva, espelho aplicado no `ghost`).
            let got = match crate::render_loop::sim_extract_sheet::cell(
                &s.0,
                s.1,
                [0.0, 0.0, 1.0, 1.0],
                i,
            ) {
                Some((_, off)) => {
                    let (mut dx, mut dy) = (f64::from(off[0]), f64::from(off[1]));
                    if s.0.flip_x {
                        dx = -dx;
                    }
                    if s.0.flip_y {
                        dy = -dy;
                    }
                    (l.live_cx + dx, l.live_cy + dy)
                }
                // A célula viva não tem fantasma — ela está no centro dela própria.
                None => (l.live_cx, l.live_cy),
            };
            assert!(
                (got.0 - want_cx).abs() < 1.0e-9 && (got.1 - want_cy).abs() < 1.0e-9,
                "grelha {hf}x{vf} viva {live}: a celula {i} esta' em {got:?} e a linha diz \
                 ({want_cx}, {want_cy})"
            );
        }
    }
}
