//! B02.A3 (AU-AU02-1): o enterro anda só na caixa do que mudou — ao bit do caminho de antes, guardado aqui como oráculo.

use super::super::super::agua_sobre_oleo_tests::{H, W, com_oleo};
use super::*;

impl PainterTool {
    /// O enterro de antes de B02.A3, verbatim: os dois laços de píxel andam a tela inteira.
    fn wet_enterra_antigo(&self, sess: &mut WetSession, velha: &[u8]) -> Vec<bool> {
        let nova = &self.canvas_rgba[..];
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let q = usize::from(sess.ratio);
        let (gw, gh) = sess.grid;
        let mut tocada = vec![false; gw * gh];
        let (cx0, cy0, cx1, cy1) = (1, 1, gw, gh);
        let (px0, px1) = (((cx0 - 1) * q).min(w), (cx1 * q).min(w));
        for y in ((cy0 - 1) * q).min(h)..(cy1 * q).min(h) {
            let o = y * w * 4;
            if velha[o + px0 * 4..o + px1 * 4] == nova[o + px0 * 4..o + px1 * 4] {
                continue;
            }
            for x in px0..px1 {
                let p = o + x * 4;
                if velha[p..p + 4] != nova[p..p + 4] {
                    tocada[(y / q) * gw + x / q] = true;
                }
            }
        }
        let base = Arc::make_mut(&mut sess.base);
        for y in 0..h {
            let o = y * w * 4;
            for x in 0..w {
                if tocada[(y / q) * gw + x / q] {
                    let p = o + x * 4;
                    base[p..p + 4].copy_from_slice(&nova[p..p + 4]);
                }
            }
        }
        sess.engine.enterra_celulas(&tocada, gw);
        let (mut x0, mut y0, mut x1, mut y1) = (gw, gh, 0, 0);
        for (k, _) in tocada.iter().enumerate().filter(|(_, t)| **t) {
            x0 = x0.min(k % gw + 1);
            y0 = y0.min(k / gw + 1);
            x1 = x1.max(k % gw + 1);
            y1 = y1.max(k / gw + 1);
        }
        let (pigmento, sombra) = (sess.planos.pigment.clone(), sess.planos.sombra.clone());
        if x1 > 0 {
            self.wet_planos_em_dia(sess, (x0, y0, x1, y1));
        }
        let p = &sess.planos;
        let mudou: Vec<bool> = (0..gw * gh)
            .map(|c| {
                tocada[c]
                    || pigmento[c * 4..c * 4 + 4] != p.pigment[c * 4..c * 4 + 4]
                    || sombra.get(c).copied().unwrap_or(0) != p.sombra.get(c).copied().unwrap_or(0)
            })
            .collect();
        let nova = &self.canvas_rgba[..];
        let amostra = super::super::grid_map::SampleU::new(sess.ratio, gw, gh);
        let (pigment, sombra) = (&sess.planos.pigment, &sess.planos.sombra);
        let base = &mut sess.base;
        for y in 0..h {
            let linha = amostra.row(y);
            let cy = (y / q).min(gh - 1);
            for x in 0..w {
                let c = cy * gw + (x / q).min(gw - 1);
                if !mudou[c] || tocada[c] {
                    continue;
                }
                let o = (y * w + x) * 4;
                let (pa, s) = (
                    linha.at(pigment, x)[3] as f32 / 255.0,
                    f32::from(sombra.get(c).copied().unwrap_or(0)) / 255.0,
                );
                let ba = f32::from(base[o + 3]) / 255.0;
                let ta = f32::from(nova[o + 3]) / 255.0;
                let deixa = (1.0 - pa) * (1.0 - s);
                if ta > 1.0 - deixa * (1.0 - ba) + ORLA_QUE_DESTAPA {
                    let b = &mut Arc::make_mut(base)[o..o + 4];
                    b[..3].copy_from_slice(&nova[o..o + 3]);
                    b[3] = ((1.0 - (1.0 - ta) / deixa) * 255.0).round() as u8;
                }
            }
        }
        self.wet_congela(sess, Some((&tocada, &mudou)));
        tocada
    }
}

/// O que um enterro deixa: a tela, a `base`, as tocadas e a congelada inteira.
type Rasto = (Vec<u8>, Vec<u8>, Vec<bool>, [Vec<u8>; 4], [Vec<bool>; 2]);

/// A tinta viva com um borrão de outro meio no rectângulo `r` (x0, y0, x1, y1): `antigo` = o oráculo.
fn enterra(r: (usize, usize, usize, usize), antigo: bool) -> (Rasto, u64) {
    let mut t = com_oleo();
    let mut sess = t.paint.wetpaint.session.take().expect("a sessão");
    sess.bring_home();
    let velha = Arc::clone(&t.canvas_rgba);
    {
        let tela = Arc::make_mut(&mut t.canvas_rgba);
        for y in r.1..r.3 {
            for x in r.0..r.2 {
                let o = (y * W + x) * 4;
                tela[o..o + 4].copy_from_slice(&[200, 30, 30, 255]);
            }
        }
    }
    VISITAS.with(|v| v.set(0));
    let tocada = if antigo {
        t.wet_enterra_antigo(&mut sess, &velha)
    } else {
        t.wet_enterra_o_que_mudou(&mut sess, &velha)
    };
    let visitas = VISITAS.with(|v| v.get());
    let c = sess.congelada.as_ref().expect("a congelada");
    let rasto = (
        t.canvas_rgba.as_ref().clone(),
        sess.base.as_ref().clone(),
        tocada,
        [
            c.tela.as_ref().clone(),
            c.base.as_ref().clone(),
            c.pigment.clone(),
            c.sombra.clone(),
        ],
        [c.enterradas.clone(), c.mudou.clone()],
    );
    (rasto, visitas)
}

/// ⭐ B02.A3: um píxel de outro meio sobre a tinta viva — os laços de píxel andam a caixa dele e a orla, não a tela.
#[test]
fn o_enterro_visita_so_a_caixa_tocada() {
    let (_, visitas) = enterra((60, 88, 61, 89), false);
    assert!(visitas > 0, "fixture: o enterro anda a caixa");
    assert!(
        visitas <= (W * H / 16) as u64,
        "o enterro de 1 píxel visitou {visitas} píxeis (a tela tem {})",
        W * H
    );
}

/// B02.A3: a caixa dá a mesma `base`, as mesmas tocadas e a mesma congelada que a tela inteira, ao bit — um píxel
/// sobre a tinta, um borrão na borda da tela e um longe da tinta.
#[test]
fn o_enterro_pela_caixa_e_o_da_tela_inteira_ao_bit() {
    for r in [(60, 88, 61, 89), (200, 70, 240, 160), (5, 5, 30, 20)] {
        let (novo, _) = enterra(r, false);
        let (antigo, _) = enterra(r, true);
        assert!(
            novo.2.iter().any(|t| *t),
            "fixture {r:?}: o borrão toca células"
        );
        assert!(
            novo == antigo,
            "o enterro pela caixa {r:?} difere do da tela inteira"
        );
    }
}
