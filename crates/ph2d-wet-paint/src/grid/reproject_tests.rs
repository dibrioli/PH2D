//! Gates da água que muda de vista ([`super::reproject_grid`]).
//!
//! ⚠️ A fixture é um TRAÇO de verdade, com a água a escorrer: uma grade pintada
//! à mão não tem a faixa viva, a máscara e a caixa coerentes que o solver
//! exige, e é exactamente essa coerência que a porta tem de devolver.

use super::reproject_grid;
use crate::grid::{Grid, snapshot_grid};
use crate::painter::Engine;

const W: usize = 96;
const H: usize = 72;

/// Um traço horizontal a meio da folha, largado e com a água ainda a correr.
fn molhada() -> Engine {
    let mut e = Engine::new(W, H);
    e.pointer_down(20.0, 20.0, None);
    let mut x = 20.0;
    while x < 60.0 {
        x += 2.0;
        e.pointer_frame(x, 20.0);
    }
    e.pointer_up();
    while e.release_frame(x, 20.0) {}
    for _ in 0..8 {
        e.step_simulation();
    }
    assert!(
        e.active_grid().has_fluid,
        "a fixture tem de conter água a correr"
    );
    e
}

fn idx(g: &Grid, cx: i32, cy: i32) -> usize {
    cx as usize + cy as usize * g.s
}

/// Continuar a simular depois da porta — em DEBUG a rede de invariantes do
/// solver (a faixa viva, a máscara, a caixa) corre a cada passo e reprova aqui.
fn continua(e: &mut Engine, passos: usize) {
    for _ in 0..passos {
        e.step_simulation();
    }
    let g = e.active_grid();
    assert!(g.film.iter().chain(&g.susp).all(|v| v.is_finite()));
}

/// **A identidade devolve a mesma água** — e só a velocidade recomeça.
#[test]
fn a_identidade_devolve_a_mesma_agua() {
    let mut e = molhada();
    let antes = snapshot_grid(e.active_grid());
    reproject_grid(e.active_grid_mut(), |cx, cy| Some((cx, cy)));
    let g = e.active_grid();
    for cy in 1..=H as i32 {
        for cx in 1..=W as i32 {
            let i = idx(g, cx, cy);
            assert_eq!(g.film[i], antes.film[i]);
            assert_eq!(g.susp[i], antes.susp[i]);
            assert_eq!(g.susp_rgb[i], antes.susp_rgb[i]);
            assert_eq!(g.sett[i], antes.sett[i]);
            assert_eq!(g.sett_rgb[i], antes.sett_rgb[i]);
            assert_eq!(g.wet[i], antes.wet[i]);
        }
    }
    assert!(g.vel_x.iter().chain(&g.vel_y).all(|&v| v == 0.0));
    assert!(g.has_fluid, "a água que estava a correr continua a correr");
    continua(&mut e, 30);
}

/// **Um deslocamento leva a água com ele** — a célula nova é a de antes no
/// mesmo ponto da superfície, e onde a vista de antes não via fica seca.
#[test]
fn um_deslocamento_leva_a_agua_com_ele() {
    const D: i32 = 17;
    let mut e = molhada();
    let antes = snapshot_grid(e.active_grid());
    let molhadas_antes = antes.film.iter().filter(|&&f| f > 0.0).count();
    assert!(molhadas_antes > 0);
    reproject_grid(e.active_grid_mut(), |cx, cy| Some((cx - D, cy)));
    let g = e.active_grid();
    for cy in 1..=H as i32 {
        for cx in 1..=W as i32 {
            let i = idx(g, cx, cy);
            if cx - D < 1 {
                assert_eq!(g.film[i], 0.0, "({cx},{cy}) não tinha origem");
                assert_eq!(g.sett[i], 0.0);
                continue;
            }
            let j = idx(g, cx - D, cy);
            assert_eq!(g.film[i], antes.film[j]);
            assert_eq!(g.susp_rgb[i], antes.susp_rgb[j]);
        }
    }
    // O CONTROLO: a água moveu-se mesmo — o sítio de antes do traço ficou seco.
    let i0 = idx(g, 22, 20);
    assert!(antes.film[i0] > 0.0 || antes.susp[i0] > 0.0);
    assert_eq!(g.film[i0], antes.film[idx(g, 22 - D, 20)]);
    assert!(g.has_fluid);
    continua(&mut e, 30);
}

/// **Onde a vista de antes não via, não há água** — e sem água nenhuma a
/// grade diz que parou.
#[test]
fn onde_a_vista_de_antes_nao_via_nao_ha_agua() {
    let mut e = molhada();
    reproject_grid(e.active_grid_mut(), |_, _| None);
    let g = e.active_grid();
    assert!(
        g.film
            .iter()
            .chain(&g.susp)
            .chain(&g.sett)
            .all(|&v| v == 0.0)
    );
    assert!(!g.has_fluid);
    continua(&mut e, 5);
}

/// **Uma origem fora da folha conta como nenhuma** — a porta não lê o anel
/// de fora nem além dele. ⚠️ O anel leva ÁGUA de propósito: vazio, lê-lo
/// daria o mesmo zero que recusá-lo, e a fixture não conteria o fenómeno.
#[test]
fn uma_origem_fora_da_folha_conta_como_nenhuma() {
    let mut e = molhada();
    {
        let g = e.active_grid_mut();
        for cy in 0..=H + 1 {
            let (a, b) = (cy * g.s, cy * g.s + W + 1);
            g.film[a] = 5.0;
            g.film[b] = 5.0;
        }
    }
    reproject_grid(e.active_grid_mut(), |cx, cy| {
        Some(if cx % 2 == 0 {
            (0, cy)
        } else {
            (W as i32 + 1, cy)
        })
    });
    let g = e.active_grid();
    assert!(g.film.iter().chain(&g.susp).all(|&v| v == 0.0));
    assert!(!g.has_fluid);
}
