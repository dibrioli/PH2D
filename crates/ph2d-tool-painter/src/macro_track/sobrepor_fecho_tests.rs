//! **O que é partilhado entre duas pinceladas sobrepostas** (W16, `docs/Macros/14_W16_plano.md` §2.3) — as
//! três leis que a prova de mutação de 22/10 deixou sem gate: o quadro corre em cada traço guardado, o
//! grupo sobreposto é UM passo do desfazer, e a água do Wet Paint fecha no último pen-up.

use super::etapas::{self, LeitorDoPainter};
use super::passo::{Passo, faixa_de};
use super::roteiro::{Pincelada, grava_nos_meios};
use super::tocar::TocadorDoPainter;
use crate::PaintMedia;
use ph2d_editor_core::tool::{PointerPhase, RasterEditTool};
use ph2d_macro::{Track, codec};

const LADO: u32 = 128;

fn sobreposta(meio: PaintMedia) -> Track {
    // LITERAL-COLOR-OK: as cores de pinceladas GRAVADAS, não da UI.
    let p = |cor, y| Pincelada {
        cor,
        tamanho_px: 18.0,
        de: [0.15, y],
        ate: [0.85, y],
        curva: 0.0,
    };
    let (b, _) =
        grava_nos_meios(LADO, &[p([220, 30, 30], 0.25), p([30, 40, 220], 0.75)], &[meio]).expect("grava");
    let g = codec::read(&b).expect("lê-se").tracks.remove(0);
    let e = etapas::etapas(&g).expect("lê-se");
    let mut s = g;
    let d = i64::try_from((e[0].t0.0 + e[0].t1.0) / 2).expect("µs") - i64::try_from(e[1].t0.0).expect("µs");
    etapas::desloca(&mut s, 1, d).expect("sobrepor é aceite");
    s.tocavel(&LeitorDoPainter).expect("cabe")
}

fn tocador(t: &Track) -> TocadorDoPainter {
    TocadorDoPainter::novo(&t.initial, t.events.iter().map(|e| e.payload.as_slice())).expect("tocador")
}

/// O índice do `Up` da faixa `n`.
fn sobe(t: &Track, n: u32) -> usize {
    t.events
        .iter()
        .position(|e| {
            faixa_de(&e.payload) == n
                && matches!(
                    Passo::descodifica(&e.payload),
                    Ok(Passo::Ponteiro {
                        fase: PointerPhase::Up,
                        ..
                    })
                )
        })
        .expect("a faixa levanta")
}

/// Cada tique corre o quadro também no traço guardado (o aerógrafo, a cauda do estabilizador, a poça
/// que cresce parada). Mutação que sangra: o tocador sem `tique_das_faixas`.
#[test]
fn o_quadro_corre_em_cada_traco_guardado() {
    let t = sobreposta(PaintMedia::Digital);
    let mut toc = tocador(&t);
    let mut vistos = 0;
    for e in &t.events {
        toc.aplica(&e.payload).expect("toca");
        if matches!(Passo::descodifica(&e.payload), Ok(Passo::Tique(_)))
            && !toc.faixas_abertas().is_empty()
        {
            vistos += 1;
            assert!(
                toc.faixas_abertas().iter().all(|g| g.quadro_corrido()),
                "um traço guardado ficou sem o quadro"
            );
        }
    }
    assert!(vistos > 3, "controlo: houve quadros com um traço guardado ({vistos})");
}

/// Duas pinceladas sobrepostas são UM passo do desfazer: um Ctrl+Z leva as duas. Mutação que sangra: cada
/// pen-down a abrir o seu passo.
#[test]
fn o_grupo_sobreposto_e_um_passo_do_desfazer() {
    let t = sobreposta(PaintMedia::Digital);
    let inicio = tocador(&t).tool_mut().run_full().0;
    let mut toc = tocador(&t);
    for e in &t.events {
        toc.aplica(&e.payload).expect("toca");
    }
    assert!(toc.tool_mut().undo_last(), "há um passo");
    let depois = toc.tool_mut().run_full().0;
    let diferentes = depois.chunks(4).zip(inicio.chunks(4)).filter(|(a, b)| a != b).count();
    assert_eq!(diferentes, 0, "um Ctrl+Z levou as duas");
}

/// A água é das duas: o 1.º pen-up não fecha o traço directo do motor; o último fecha. Mutação que
/// sangra: `wetpaint_stroke_end` sem a guarda dos traços fora.
#[test]
fn a_agua_do_wet_paint_fecha_no_ultimo_pen_up() {
    let t = sobreposta(PaintMedia::WetPaint);
    let (primeiro, ultimo) = (sobe(&t, 0), sobe(&t, 2));
    assert!(primeiro < ultimo, "controlo: a vermelha levanta primeiro");
    let mut toc = tocador(&t);
    for (i, e) in t.events.iter().enumerate() {
        toc.aplica(&e.payload).expect("toca");
        if i == primeiro {
            assert!(toc.tool().wet_traco_directo_aberto(), "a azul ainda pinta na água");
        }
        if i == ultimo {
            assert!(!toc.tool().wet_traco_directo_aberto(), "a última fecha-a");
        }
    }
}
