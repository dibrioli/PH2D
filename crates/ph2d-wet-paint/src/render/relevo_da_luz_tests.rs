//! B14.4: o emboss da tinta sem óleo segue a luz do cartão; na omissão é o de sempre, ao bit.

use super::*;
use crate::painter::Engine;
use crate::render::{PigmentVisual, RenderLayer, render_pigment_row_visual, render_region};

const W: usize = 64;
const H: usize = 48;

fn pintado() -> Engine {
    let mut e = Engine::new(W, H);
    for y in [14.0, 30.0] {
        e.pointer_down(10.0, y, None);
        let mut x = 10.0;
        while x < 50.0 {
            x += 3.0;
            e.dispatch_pressure_dab(x, y + 0.4 * (x * 0.3).sin(), 8.0, 40.0);
        }
        e.pointer_up();
        while e.release_frame(x, y) {}
    }
    e
}

/// As duas portas do render (a linha do composite e a região de referência) com a luz em `azimute`.
fn imagens(e: &mut Engine, azimute: f64) -> (Vec<u8>, Vec<u8>) {
    e.sim.luz_do_relevo = azimute;
    let p = e.sim.gather_params(&e.tuning);
    let camadas: Vec<_> = e
        .layers
        .iter()
        .map(|l| RenderLayer {
            grid: &l.grid,
            opacity: l.opacity,
            visible: l.visible,
        })
        .collect();
    let visual = PigmentVisual {
        paper: false,
        relevo: true,
    };
    let mut linha = vec![0u8; W * H * 4];
    for (k, row) in linha.chunks_mut(W * 4).enumerate() {
        render_pigment_row_visual(Some(&p), &camadas, visual, k + 1, 1, W, row);
    }
    let mut regiao = vec![0u8; W * H * 4];
    let activa = &e.layers[0].grid;
    render_region(
        &p,
        &camadas,
        activa,
        false,
        false,
        false,
        &mut regiao,
        1,
        1,
        W as i32,
        H as i32,
    );
    (linha, regiao)
}

#[test]
fn os_pesos_do_relevo_na_omissao_sao_os_de_sempre() {
    assert_eq!(
        crate::oleo_real::OleoReal::default().azimute.to_bits(),
        AZIMUTE_DE_FABRICA.to_bits(),
        "a omissão do emboss é a do cartão"
    );
    let p = Engine::new(8, 8)
        .sim
        .gather_params(&crate::tuning::Tuning::default());
    assert_eq!(p.relevo.map(f64::to_bits), [0.5f64, 1.0].map(f64::to_bits));
    // A luz do lado oposto inverte o relevo; o comprimento é o de sempre.
    let [x, y] = pesos_do_relevo(45f64.to_radians());
    assert!(
        (x + 0.5).abs() < 1e-12 && (y + 1.0).abs() < 1e-12,
        "{x} {y}"
    );
}

#[test]
fn o_emboss_segue_a_luz_do_cartao() {
    let mut e = pintado();
    assert!(
        e.layers[0].grid.relevo.iter().any(|&r| r != 0.0),
        "a cena tem relevo a desenhar"
    );
    let (linha_fab, regiao_fab) = imagens(&mut e, AZIMUTE_DE_FABRICA);
    let (linha_op, regiao_op) = imagens(&mut e, 45f64.to_radians());
    let difere = |a: &[u8], b: &[u8]| a.iter().zip(b).filter(|(x, y)| x != y).count();
    let (dl, dr) = (
        difere(&linha_fab, &linha_op),
        difere(&regiao_fab, &regiao_op),
    );
    assert!(
        dl > 0,
        "a linha do composite não segue a luz: {dl} bytes diferentes"
    );
    assert!(
        dr > 0,
        "a região de referência não segue a luz: {dr} bytes diferentes"
    );
    // De volta à omissão: o mesmo bit.
    assert_eq!(imagens(&mut e, AZIMUTE_DE_FABRICA), (linha_fab, regiao_fab));
}
