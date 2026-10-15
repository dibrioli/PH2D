//! **A medição da fracção** (W6b, `docs/Macros/03_plano_W6.md` §6.1): o Splash gravado, e um aluno que
//! faz as pinceladas anteriores certas e a `k` de cada maneira — certa, deslocada, fina, pela metade,
//! noutro lugar, nenhuma, «Mostra-me». Imprime a fracção pintada da parte NOVA e da região inteira.
//!
//! `cargo test -p ph2d-tool-painter --profile smoke --lib macro_track::tutorial::sonda -- --ignored --nocapture`

use super::*;
use crate::macro_track::roteiro::{
    CAUDA, DT_MS, ENTRE, Pincelada, SPLASH, TRACO, grava_pinceladas, ponto,
};
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, Tool};

pub(super) fn quadro(t: &mut PainterTool) {
    drop(t.take_preview_arc());
    t.on_tick(DT_MS);
}

/// O aluno pinta `p` como o roteiro grava: ponteiro a cada quadro, depois o intervalo.
fn pinta(t: &mut PainterTool, p: &Pincelada, lado: f32) {
    t.set_brush_color_srgb8(p.cor);
    t.set_brush_size_px(p.tamanho_px);
    for i in 0..=TRACO {
        #[expect(clippy::cast_precision_loss, reason = "poucos quadros")]
        let f = i as f32 / TRACO as f32;
        let phase = match i {
            0 => PointerPhase::Down,
            i if i == TRACO => PointerPhase::Up,
            _ => PointerPhase::Move,
        };
        t.on_canvas_pointer(CanvasPointer {
            pos: ponto(p, f, lado),
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        });
        quadro(t);
    }
    for _ in 0..ENTRE {
        quadro(t);
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Variante {
    Exacta,
    Desloca(f32),
    Fina,
    Metade,
    Longe,
    Nada,
    MostraMe,
}

/// A pincelada `p` deslocada `d` raios na perpendicular do caminho.
fn deslocada(p: &Pincelada, d: f32, lado: f32) -> Pincelada {
    let (dx, dy) = (p.ate[0] - p.de[0], p.ate[1] - p.de[1]);
    let n = (dx * dx + dy * dy).sqrt().max(1e-6);
    let s = d * p.tamanho_px / lado;
    let (ox, oy) = (-dy / n * s, dx / n * s);
    Pincelada {
        de: [p.de[0] + ox, p.de[1] + oy],
        ate: [p.ate[0] + ox, p.ate[1] + oy],
        ..*p
    }
}

pub(super) fn aluno(track: &Track, k: usize, v: Variante, lado: u32) -> PainterTool {
    #[expect(clippy::cast_precision_loss, reason = "o lado cabe num f32")]
    let l = lado as f32;
    let mut t = inicial::tool_no_inicial(&track.initial).expect("inicial");
    for p in &SPLASH[..k] {
        pinta(&mut t, p, l);
    }
    let p = SPLASH[k];
    match v {
        Variante::Exacta => pinta(&mut t, &p, l),
        Variante::Desloca(d) => pinta(&mut t, &deslocada(&p, d, l), l),
        Variante::Fina => pinta(
            &mut t,
            &Pincelada {
                tamanho_px: p.tamanho_px * 0.5,
                ..p
            },
            l,
        ),
        Variante::Metade => pinta(
            &mut t,
            &Pincelada {
                ate: [(p.de[0] + p.ate[0]) * 0.5, (p.de[1] + p.ate[1]) * 0.5],
                curva: p.curva * 0.5,
                ..p
            },
            l,
        ),
        Variante::Longe => pinta(
            &mut t,
            &Pincelada {
                de: [0.90, 0.96],
                ate: [0.98, 0.90],
                curva: 0.0,
                ..p
            },
            l,
        ),
        Variante::Nada => {}
        Variante::MostraMe => {
            let crits = EnsinoDoPainter::de(track, Some((lado, lado))).expect("ensino");
            mostra(&mut t, track, crits.criterios[k].regiao.k).expect("mostra");
            for _ in 0..ENTRE {
                quadro(&mut t);
            }
        }
    }
    for _ in 0..CAUDA {
        quadro(&mut t);
    }
    t
}

/// A fracção de `idx` pintada com «pintado» = algum canal de alguma camada a mais de `limiar`.
fn fracao(e: &EnsinoDoPainter, t: &PainterTool, idx: &[u32], limiar: u8) -> f32 {
    let aluno = t.camadas_rgba();
    let n = idx
        .iter()
        .filter(|&&i| {
            let k = i as usize * 4;
            aluno.iter().any(|(id, px)| {
                e.inicial
                    .iter()
                    .find(|(j, _)| j == id)
                    .map_or(px[k + 3] > limiar, |(_, ini)| {
                        (0..4).any(|c| px[k + c].abs_diff(ini[k + c]) > limiar)
                    })
            })
        })
        .count();
    #[expect(clippy::cast_precision_loss, reason = "contagens")]
    let f = n as f32 / idx.len().max(1) as f32;
    f
}

#[test]
#[ignore = "sonda de medição (W6b): imprime a tabela da fracção"]
fn sonda_a_fracao_pintada_por_variante() {
    use Variante::*;
    for (lado, variantes) in [
        (
            512u32,
            &[
                Exacta,
                Desloca(0.5),
                Desloca(1.0),
                Desloca(2.0),
                Fina,
                Metade,
                Longe,
                Nada,
                MostraMe,
            ][..],
        ),
        (1024, &[Exacta, Desloca(1.0), Metade, Nada, MostraMe][..]),
    ] {
        let (b, _) = grava_pinceladas(lado, 0, &SPLASH, &[0; 4]).expect("grava");
        let track = ph2d_macro::codec::read(&b).expect("lê").tracks.remove(0);
        let e = EnsinoDoPainter::de(&track, Some((lado, lado))).expect("ensino");
        for (k, c) in e.criterios.iter().enumerate() {
            let toda = rasteriza(&c.regiao, lado, lado);
            println!(
                "lado {lado} · pincelada {} (raio {}): região {} px, nova {} px ({:.0} %)",
                k + 1,
                c.regiao.raio,
                toda.len(),
                c.novos.len(),
                100.0 * c.novos.len() as f32 / toda.len().max(1) as f32
            );
            for &v in variantes {
                let t = aluno(&track, k, v, lado);
                let t0 = std::time::Instant::now();
                let f = e.fracao_pintada(&t, c);
                let us = t0.elapsed().as_micros();
                println!(
                    "  {v:<14?} nova {:>5.1} % (>16: {:>5.1} %) · toda {:>5.1} % (>16: {:>5.1} %) · avalia {us} µs",
                    100.0 * f,
                    100.0 * fracao(&e, &t, &c.novos, 16),
                    100.0 * fracao(&e, &t, &toda, 0),
                    100.0 * fracao(&e, &t, &toda, 16),
                );
            }
        }
    }
}
