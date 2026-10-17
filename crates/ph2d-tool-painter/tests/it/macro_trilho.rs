//! ⭐ **As ferramentas do trilho entram no estado inicial** (W7, `docs/Macros/04_plano_W7.md` item 4): o
//! que a fonte do Clone, a Seleção e a Máscara sabem ANTES de armar é estado do tool fora do documento e
//! fora do painel de pincel — sem ele no estado inicial, o tool novo que toca a macro pinta outra coisa.
//!
//! Cada caso prepara a ferramenta antes de armar, grava traços (e, na Seleção, uma forma acrescentada A
//! GRAVAR, que se recompõe da lista de formas), toca a macro num tool novo e compara a tela ao byte.
//! Controlo: o mesmo caso sem a preparação dá outra tela (a preparação decide o que se pinta).

use super::macro_comum::{Gravador, cp, texels_diferentes, toca};
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::tool::{CanvasPaintTool, PanelEvent, PointerPhase, RasterEditTool, Tool};
use ph2d_tool_painter::{PaintMedia, PainterTool, ids};

const N: u32 = 160;

fn ferramenta() -> PainterTool {
    let mut t = PainterTool::default();
    // Metade de cima a azul: o Clone copia de lá para baixo.
    let mut px = vec![255u8; (N * N * 4) as usize];
    for y in 0..N / 2 {
        for x in 0..N {
            let i = ((y * N + x) * 4) as usize;
            px[i..i + 4].copy_from_slice(&[30, 70, 200, 255]);
        }
    }
    t.set_source(px, N, N);
    t.set_paint_media(PaintMedia::Digital);
    t.set_brush_size_px(14.0);
    t.set_brush_color_srgb8([200, 40, 30]);
    t
}

fn quadro(t: &mut PainterTool, g: Option<&mut Gravador>) {
    let _ = t.take_preview_arc();
    t.on_tick(1000.0 / 60.0);
    if let Some(g) = g {
        g.quadro(t);
    }
}

/// Um evento do painel pela porta gravada (a do produto), e um quadro.
fn painel(t: &mut PainterTool, g: Option<&mut Gravador>, ev: PanelEvent) {
    t.handle_panel_event(ev);
    quadro(t, g);
}

/// A ferramenta do trilho, pelo botão do trilho.
fn modo(t: &mut PainterTool, g: Option<&mut Gravador>, m: &str) {
    painel(
        t,
        g,
        PanelEvent::SelectOption(core_ids::PAINTER_PAINT_MODE, m.to_owned()),
    );
}

/// Um traço de `de` a `ate` (px), um evento por quadro.
fn traco(t: &mut PainterTool, mut g: Option<&mut Gravador>, de: [f32; 2], ate: [f32; 2]) {
    const AMOSTRAS: usize = 16;
    for i in 0..=AMOSTRAS {
        let f = i as f32 / AMOSTRAS as f32;
        let fase = match i {
            0 => PointerPhase::Down,
            AMOSTRAS => PointerPhase::Up,
            _ => PointerPhase::Move,
        };
        let p = [de[0] + (ate[0] - de[0]) * f, de[1] + (ate[1] - de[1]) * f];
        t.on_canvas_pointer(cp(p, fase));
        quadro(t, g.as_deref_mut());
    }
}

#[derive(Clone, Copy, Debug)]
enum Caso {
    Clone,
    Selecao,
    Mascara,
}

/// `prepara` = a ferramenta do caso antes de armar; devolve `(tela do tool que gravou, tela tocada)`.
fn grava_e_toca(caso: Caso, prepara: bool) -> (Vec<u8>, Vec<u8>) {
    let mut t = ferramenta();
    match caso {
        Caso::Clone => {
            modo(&mut t, None, "clone");
            if prepara {
                painel(
                    &mut t,
                    None,
                    PanelEvent::Click(ids::PAINTER_BRUSH_CLONE_SET_SOURCE),
                );
                traco(&mut t, None, [80.0, 30.0], [80.0, 30.0]);
                assert!(t.clone_has_source(), "a fonte escolheu-se");
                painel(
                    &mut t,
                    None,
                    PanelEvent::Click(ids::PAINTER_BRUSH_CLONE_ALIGNED),
                );
            }
        }
        Caso::Selecao => {
            if prepara {
                modo(&mut t, None, "selection");
                painel(
                    &mut t,
                    None,
                    PanelEvent::Click(ids::PAINTER_SEL_MODE_IDS[2]),
                );
                traco(&mut t, None, [20.0, 20.0], [80.0, 140.0]);
                painel(
                    &mut t,
                    None,
                    PanelEvent::SetValue(ids::PAINTER_SEL_FEATHER_SLIDER, 0.3),
                );
                assert!(t.selection_active(), "a seleção existe");
                modo(&mut t, None, "brush");
            }
        }
        Caso::Mascara => {
            if prepara {
                modo(&mut t, None, "mask");
                painel(&mut t, None, PanelEvent::Click(ids::PAINTER_MASK_BRUSH[0]));
                traco(&mut t, None, [10.0, 100.0], [150.0, 110.0]);
                assert!(t.mask_scratch_active(), "a máscara pintou-se");
                painel(&mut t, None, PanelEvent::Click(ids::PAINTER_MASK_COLOR[2]));
                modo(&mut t, None, "brush");
            }
        }
    }
    let mut g = Gravador::arma(&mut t);
    match caso {
        Caso::Clone => {
            traco(&mut t, Some(&mut g), [20.0, 120.0], [140.0, 130.0]);
            traco(&mut t, Some(&mut g), [30.0, 150.0], [130.0, 100.0]);
        }
        Caso::Selecao => {
            traco(&mut t, Some(&mut g), [5.0, 60.0], [155.0, 70.0]);
            // A GRAVAR: mais uma forma, somada (recompõe-se da lista de formas que o inicial levou).
            modo(&mut t, Some(&mut g), "selection");
            painel(
                &mut t,
                Some(&mut g),
                PanelEvent::Click(ids::PAINTER_SEL_MODE_IDS[2]),
            );
            painel(
                &mut t,
                Some(&mut g),
                PanelEvent::Click(ids::PAINTER_SEL_OP_IDS[1]),
            );
            traco(&mut t, Some(&mut g), [100.0, 90.0], [150.0, 150.0]);
            modo(&mut t, Some(&mut g), "brush");
            traco(&mut t, Some(&mut g), [5.0, 130.0], [155.0, 120.0]);
        }
        Caso::Mascara => {
            traco(&mut t, Some(&mut g), [5.0, 105.0], [155.0, 100.0]);
            traco(&mut t, Some(&mut g), [80.0, 60.0], [80.0, 150.0]);
        }
    }
    let doc = g.para(&mut t);
    let original = t.run_full().0;
    let mut toc = toca(&doc).expect("a reprodução não diverge");
    (original, toc.tool_mut().run_full().0)
}

#[test]
fn o_estado_das_ferramentas_do_trilho_viaja_no_inicial() {
    let mut falhas = Vec::new();
    for caso in [Caso::Clone, Caso::Selecao, Caso::Mascara] {
        let (a, b) = grava_e_toca(caso, true);
        let d = texels_diferentes(&a, &b);
        if d > 0 {
            falhas.push(format!("{caso:?}: {d} texels"));
        }
    }
    assert!(
        falhas.is_empty(),
        "o tool novo não sabe o que a ferramenta sabia antes de armar: {falhas:?}"
    );
}

/// O controlo: sem a preparação a tela do tool que gravou é outra — o caso mede a preparação.
#[test]
fn sem_a_preparacao_a_tela_e_outra() {
    for caso in [Caso::Clone, Caso::Selecao, Caso::Mascara] {
        let (com, _) = grava_e_toca(caso, true);
        let (sem, _) = grava_e_toca(caso, false);
        assert!(
            texels_diferentes(&com, &sem) > 0,
            "{caso:?}: a preparação não muda a tela — o gate não a mede"
        );
    }
}
