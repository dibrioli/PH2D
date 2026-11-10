//! B02.A3 (AU-AU02-1): o enterro anda só na caixa do que mudou — ao bit do caminho de antes, guardado aqui como oráculo.

use super::super::super::agua_sobre_oleo_tests::{H, W, com_oleo};
use super::*;

impl PainterTool {
    /// O enterro de antes de B02.A3, verbatim: os laços de píxel andam a tela inteira e os de célula a grade.
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
        self.wet_congela_antigo(sess, Some((&tocada, &mudou)));
        tocada
    }

    /// O `wet_congela` de antes de B02.A3, verbatim: clona as marcas e os planos da grade inteira.
    fn wet_congela_antigo(&self, sess: &mut WetSession, agora: Option<(&[bool], &[bool])>) {
        let n = sess.grid.0 * sess.grid.1;
        let (mut enterradas, mut mudou) = match sess.congelada.take() {
            Some(c) => (c.enterradas.clone(), c.mudou.clone()),
            None => (vec![false; n], vec![false; n]),
        };
        if let Some((t, m)) = agora {
            for (e, t) in enterradas.iter_mut().zip(t) {
                *e |= *t;
            }
            for (e, m) in mudou.iter_mut().zip(m) {
                *e |= *m;
            }
        }
        sess.congelada = Some(Arc::new(Congelada {
            mudou,
            enterradas,
            tela: Arc::clone(&self.canvas_rgba),
            base: Arc::clone(&sess.base),
            pigment: sess.planos.pigment.clone(),
            sombra: sess.planos.sombra.clone(),
            caixa: None,
            gw: sess.grid.0,
            gh: sess.grid.1,
            rr: usize::from(sess.ratio),
        }));
    }
}

/// O que os enterros deixam: a tela, a `base`, as tocadas de cada um, a congelada (tela, base, e o pigmento e a sombra
/// só onde ela os lê: nas células `mudou`), as marcas, e o píxel que ela serve a cada píxel da tela.
type Rasto = (
    Vec<u8>,
    Vec<u8>,
    Vec<Vec<bool>>,
    [Vec<u8>; 4],
    [Vec<bool>; 2],
    Vec<Option<[u8; 4]>>,
);

/// A tinta viva com borrões de outro meio, um enterro por rectângulo `(x0, y0, x1, y1)` na mesma sessão: `antigo` = o
/// oráculo. As visitas são as do último enterro.
fn enterra(rs: &[(usize, usize, usize, usize)], antigo: bool) -> (Rasto, u64) {
    let mut t = com_oleo();
    let mut sess = t.paint.wetpaint.session.take().expect("a sessão");
    sess.bring_home();
    let mut tocadas = Vec::new();
    let mut visitas = 0;
    for (i, r) in rs.iter().enumerate() {
        let velha = Arc::clone(&t.canvas_rgba);
        {
            let tela = Arc::make_mut(&mut t.canvas_rgba);
            for y in r.1..r.3 {
                for x in r.0..r.2 {
                    let o = (y * W + x) * 4;
                    tela[o..o + 4].copy_from_slice(&[200, 30 + 60 * i as u8, 30, 255]);
                }
            }
        }
        VISITAS.with(|v| v.set(0));
        tocadas.push(if antigo {
            t.wet_enterra_antigo(&mut sess, &velha)
        } else {
            t.wet_enterra_o_que_mudou(&mut sess, &velha)
        });
        visitas = VISITAS.with(|v| v.get());
    }
    let c = sess.congelada.as_ref().expect("a congelada");
    let nas_mudou = |v: &[u8], k: usize| -> Vec<u8> {
        v.chunks(k)
            .zip(&c.mudou)
            .flat_map(|(b, m)| if *m { b.to_vec() } else { vec![0; b.len()] })
            .collect()
    };
    let (p, base) = (&sess.planos, &sess.base);
    let servidos = (0..W * H)
        .map(|i| {
            c.pixel((i % W, i / W, i * 4), &p.pigment, &p.sombra, base)
                .map(|px| [px[0], px[1], px[2], px[3]])
        })
        .collect();
    let rasto = (
        t.canvas_rgba.as_ref().clone(),
        sess.base.as_ref().clone(),
        tocadas,
        [
            c.tela.as_ref().clone(),
            c.base.as_ref().clone(),
            nas_mudou(&c.pigment, 4),
            nas_mudou(&c.sombra, 1),
        ],
        [c.enterradas.clone(), c.mudou.clone()],
        servidos,
    );
    (rasto, visitas)
}

/// ⭐ B02.A3: um píxel de outro meio sobre a tinta viva — os laços de píxel andam a caixa dele e a orla, e os de célula
/// a caixa que a óptica reescreve (a célula crescida pelo alcance do Óleo Real, que não depende da tela): nem a tela
/// nem a grade.
#[test]
fn o_enterro_visita_so_a_caixa_tocada() {
    let t = com_oleo();
    let sess = t.paint.wetpaint.session.as_ref().expect("a sessão");
    let (gw, gh) = sess.grid;
    let layers: Vec<_> = sess
        .engine
        .layers
        .iter()
        .map(|l| ph2d_wet_paint::render::RenderLayer {
            grid: &l.grid,
            opacity: l.opacity,
            visible: l.visible,
        })
        .collect();
    let oleo = t
        .factos_da_otica()
        .oleo_da_tela(&layers)
        .expect("fixture: Óleo Real");
    let a = ph2d_wet_paint::oleo_real::alcance(&oleo);
    let lado = (2 * a + 1).min(gw.max(gh));
    // Píxeis: a caixa e a orla; células: o retrato de antes, o `mudou`, as marcas e os planos da congelada.
    let tecto = (W * H / 16 + 4 * lado * lado) as u64;
    let (_, visitas) = enterra(&[(60, 88, 61, 89)], false);
    assert!(visitas > 0, "fixture: o enterro anda a caixa");
    assert!(
        tecto < (W * H + gw * gh) as u64,
        "fixture: o tecto ({tecto}) separa a caixa da tela ({}) mais a grade ({gw}×{gh})",
        W * H
    );
    assert!(
        visitas <= tecto,
        "o enterro de 1 píxel visitou {visitas} píxeis e células (tecto {tecto}; a tela tem {}, a grade {gw}×{gh})",
        W * H
    );
}

/// B02.A3: a caixa dá a mesma `base`, as mesmas tocadas e a mesma congelada que a tela inteira, ao bit — um píxel
/// sobre a tinta, um borrão na borda da tela, um longe da tinta, e dois enterros seguidos na mesma sessão (a congelada
/// de antes reusada: as marcas somam e os planos põem-se em dia na caixa de todas).
#[test]
fn o_enterro_pela_caixa_e_o_da_tela_inteira_ao_bit() {
    let casos: [&[(usize, usize, usize, usize)]; 4] = [
        &[(60, 88, 61, 89)],
        &[(200, 70, 240, 160)],
        &[(5, 5, 30, 20)],
        &[(60, 88, 61, 89), (64, 90, 90, 110), (5, 5, 30, 20)],
    ];
    for rs in casos {
        let (novo, _) = enterra(rs, false);
        let (antigo, _) = enterra(rs, true);
        assert!(
            novo.2.iter().all(|t| t.iter().any(|t| *t)),
            "fixture {rs:?}: cada borrão toca células"
        );
        assert!(
            novo.5.iter().any(Option::is_some),
            "fixture {rs:?}: a congelada serve píxeis"
        );
        assert!(
            novo == antigo,
            "o enterro pela caixa {rs:?} difere do da tela inteira"
        );
    }
}
