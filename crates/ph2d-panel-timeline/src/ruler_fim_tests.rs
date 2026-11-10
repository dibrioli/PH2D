//! Portões da linha do fim nos cartões (dono 10/11): sai depois das peças, de alto a baixo.

use super::*;
use ph2d_editor_core::screens::hero::HeroLayout;
use ph2d_editor_core::screens::slot::{Slot, SlotSet};
use ph2d_timeline::{StackHost, StripSource, TimelineDoc, TimelineState};
use ph2d_vector::{DrawOp, Shape};

/// Pinta as faixas do Arrange e depois a régua (a ordem de `paint.rs`) com uma peça `[1, 3)` a
/// cruzar o fim autorado em 2 s; devolve as ops, quantas eram das faixas e a régua.
fn pinta_arrange(cartoes: bool) -> (Vec<DrawOp>, usize, Rect) {
    let mut doc = TimelineDoc::new();
    let l = doc.add_lane_in(StackHost::Document, "Base".into()).unwrap();
    doc.add_strip_to(StackHost::Document, l, StripSource::Clip(0), 1.0, 3.0)
        .unwrap();
    let mut st = TimelineState::new();
    st.doc = doc;
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &ph2d_core::Playhead::default(), false);
    snap.view_length_seconds = 2.0;
    snap.view_length_explicit = true;
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(Some(cartoes));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::TimelinePanel>();
    let area = Rect::new(0.0, 0.0, 1280.0, 800.0);
    let layout = HeroLayout::for_viewport(area);
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = ph2d_text::TextSystem::without_system_fonts();
    let mut ctx = PaintCtx {
        host: &mut host,
        layout: &layout,
        slot: layout
            .slot_rects(SlotSet::of(Slot::Bottom))
            .get(Slot::Bottom),
        viewport: area,
        scene: &mut scene,
        text_system: &mut text,
    };
    let g = crate::geom::Geom {
        region: Rect::new(0.0, 0.0, 900.0, 240.0),
        label_w: 100.0,
        time_area: Rect::new(100.0, 18.0, 800.0, 222.0),
        rows: Rect::new(100.0, 40.0, 800.0, 200.0),
        scrollbar: Rect::new(900.0, 40.0, 0.0, 200.0),
    };
    let view = crate::graph::TimeView {
        time_x: 100.0,
        right: 900.0,
        view_start: 0.0,
        px_per_s: 200.0,
    };
    let state = crate::state::TimelinePanelState {
        tab: crate::tab::Tab::Arrange,
        ..crate::state::TimelinePanelState::default()
    };
    ctx.scene.record();
    crate::stack_lane_paint::paint(&mut ctx, Theme::Forge, &g, view, &state, &snap);
    let faixas = ctx.scene.take_record().len();
    ctx.scene.record();
    crate::stack_lane_paint::paint(&mut ctx, Theme::Forge, &g, view, &state, &snap);
    let clock = crate::ruler_clock::clock_for(state.tab, &snap);
    paint(
        &mut ctx,
        Theme::Forge,
        g.time_area,
        0.0,
        200.0,
        &snap,
        clock,
    );
    let ops = ctx.scene.take_record();
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(None);
    (ops, faixas, g.time_area)
}

/// Os índices dos preenchimentos na tinta da linha do fim com a altura cheia da régua em `x`.
fn linhas_do_fim(ops: &[DrawOp], regiao: Rect, x: f64) -> Vec<usize> {
    let cor = resolve(ColorToken::TimelinePlayhead, Theme::Forge);
    ops.iter()
        .enumerate()
        .filter(|(_, o)| match o {
            DrawOp::Fill {
                brush: ph2d_vector::Brush::Solid(c),
                path,
                ..
            } => {
                let b = path.bounding_box();
                *c == cor
                    && b.x0 <= x
                    && x <= b.x1
                    && (b.height() - f64::from(regiao.h)).abs() < 1.0
            }
            _ => false,
        })
        .map(|(i, _)| i)
        .collect()
}

/// ⭐ **Nos cartões a linha do fim sai DEPOIS das peças, régua e faixas inteiras** (dono 10/11: «a
/// linha que marca o fim desapareceu»): uma peça `[1, 3)` cruza o fim em 2 s (x = 100 + 400). Fora
/// dos cartões não há linha (a saída de referência não muda). *Mutação: tirar a chamada a
/// `pinta_linha_do_fim` ⇒ nenhuma linha e reprova.*
#[test]
fn nos_cartoes_a_linha_do_fim_sai_depois_das_pecas() {
    let (ops, faixas, regiao) = pinta_arrange(true);
    assert!(faixas > 0, "a peça pintou-se");
    let linhas = linhas_do_fim(&ops, regiao, 500.0);
    assert!(
        !linhas.is_empty(),
        "a linha do fim, altura cheia, em x = 500"
    );
    assert!(
        linhas.iter().all(|i| *i >= faixas),
        "a linha {linhas:?} sai depois das {faixas} ops das faixas"
    );
    let (ops, _, regiao) = pinta_arrange(false);
    assert!(
        linhas_do_fim(&ops, regiao, 500.0).is_empty(),
        "o clássico não muda"
    );
}
