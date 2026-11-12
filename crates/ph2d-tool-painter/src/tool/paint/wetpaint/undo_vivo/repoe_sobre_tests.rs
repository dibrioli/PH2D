//! O undo vivo repõe a ladeira e o fluxo SÓ nos tiles que o «depois» não partilha com a foto: o bit do `repoe` inteiro.

use super::super::undo_tinta_morre_tests::{AGUADA, DABBER, tela, traco};
use super::REPOE_CHEIO;
use crate::tool::PainterTool;

fn estado(t: &mut PainterTool) -> (Vec<u8>, Vec<u32>) {
    t.wet_bring_home();
    let s = t.paint.wetpaint.session.as_ref().expect("a sessão viva");
    let bits = s
        .engine
        .layers
        .iter()
        .flat_map(|l| {
            let g = &l.grid;
            g.flow_x.iter().chain(&g.flow_y).chain(&g.oleo_antes)
        })
        .map(|x| x.to_bits())
        .collect();
    (t.canvas_rgba.to_vec(), bits)
}

#[test]
fn o_repoe_sobre_o_depois_da_o_bit_do_repoe_inteiro() {
    for meio in [AGUADA, DABBER] {
        let pinta = || {
            let mut t = tela(300, 200, meio, 20.0);
            for i in 0..3 {
                traco(&mut t, 30.0, 270.0, 40.0 + 50.0 * i as f32);
            }
            t
        };
        let (mut a, mut b) = (pinta(), pinta());
        assert!(
            estado(&mut a) == estado(&mut b),
            "{meio}: partiram diferentes"
        );
        for (k, passo) in ["undo", "undo", "redo", "undo", "redo", "redo"]
            .into_iter()
            .enumerate()
        {
            let faz = |t: &mut PainterTool| {
                let feito = if passo == "undo" {
                    t.undo_last()
                } else {
                    t.redo_last()
                };
                t.ticks_em_blocos(1);
                feito
            };
            REPOE_CHEIO.with(|c| c.set(true));
            let fa = faz(&mut a);
            REPOE_CHEIO.with(|c| c.set(false));
            assert!(fa && faz(&mut b), "{meio} {k}: {passo}");
            assert!(estado(&mut a) == estado(&mut b), "{meio} {k}: {passo}");
        }
    }
}
