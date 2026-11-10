//! Portões da régua com o trecho (doc 17 item 2, maquete B6): o trecho armado É o laço da régua, e os
//! nomes dos marcadores nunca pisam os números.

use super::*;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::screens::hero::HeroLayout;
use ph2d_editor_core::screens::slot::{Slot, SlotSet};
use ph2d_tokens::Theme;
use ph2d_vector::{DrawOp, Shape};

const PX_S: f64 = 90.0;
const REGIAO: Rect = Rect {
    x: 100.0,
    y: 18.0,
    w: 800.0,
    h: 200.0,
};

/// Os marcadores da cena 2 (takeoff 0 · top 1,1 · land 2) na aba Keys, o loop do clipe `[0, 5)`.
fn snap() -> TimelineViewSnapshot {
    TimelineViewSnapshot {
        keys_mode: true,
        loop_range: Some((0.0, 5.0)),
        markers: [(0.0, "takeoff"), (1.1, "top"), (2.0, "land")]
            .map(|(t, n)| (t, n.to_string(), None))
            .to_vec(),
        ..TimelineViewSnapshot::default()
    }
}

/// Pinta a régua nos cartões, com o trecho `par` armado; devolve as ops, as caixas e os ids que a régua registou.
fn pinta(snap: &TimelineViewSnapshot, par: Option<(usize, usize)>) -> (Vec<DrawOp>, Vec<Rect>) {
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(Some(true));
    let t = par.and_then(|(a, b)| Trecho::novo(t_do_snap(snap), a, b, 0, true));
    TRECHO.with(|c| c.set(t));
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
    let clock = crate::ruler_clock::clock_for(crate::tab::Tab::Keys, snap);
    ctx.scene.record();
    crate::ruler::paint(&mut ctx, Theme::Forge, REGIAO, 0.0, PX_S, snap, clock);
    let ops = ctx.scene.take_record();
    let caixas = ph2d_ui_testkit::caixas_pintadas(ctx.scene);
    TRECHO.with(|c| c.set(None));
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(None);
    (ops, caixas)
}

fn x(t: f64) -> f64 {
    f64::from(REGIAO.x) + t * PX_S
}

/// As faixas do laço (a cor e a altura da faixa fina), `(x0, x1)`.
fn faixas_do_laco(ops: &[DrawOp]) -> Vec<(f64, f64)> {
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(Some(true));
    let (h, cor) = crate::ruler::faixa_do_laco(Theme::Forge);
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(None);
    ops.iter()
        .filter_map(|o| match o {
            DrawOp::Fill {
                brush: ph2d_vector::Brush::Solid(c),
                path,
                ..
            } if *c == cor => Some(path.bounding_box()),
            _ => None,
        })
        .filter(|b| (b.height() - f64::from(h)).abs() < 0.5)
        .map(|b| (b.x0, b.x1))
        .collect()
}

/// ⭐ **O trecho armado É o laço que a régua mostra** (maquete B6: a `.c13lp` vai de takeoff a top): a
/// faixa do laço sai de `x(0)` a `x(1,1)`, nenhuma faixa do loop do clipe `[0, 5)`, e as pontas do loop
/// do clipe não pintam nem agarram (editar um loop que não toca é um controlo que mente); a caixa do
/// trecho (`.c13sec`) tem a régua inteira. Sem trecho, volta o do clipe. *Mutação: pintar
/// `snap.loop_range` na faixa ⇒ reprova.*
#[test]
fn o_trecho_armado_e_o_laco_da_regua() {
    let s = snap();
    let (ops, _) = pinta(&s, Some((0, 1)));
    let faixas = faixas_do_laco(&ops);
    let perto = |(a, b): (f64, f64), (c, d): (f64, f64)| (a - c).abs() < 1.0 && (b - d).abs() < 1.0;
    assert!(
        faixas.iter().any(|&f| perto(f, (x(0.0), x(1.1)))),
        "a faixa do laço é o trecho: {faixas:?}"
    );
    assert!(
        !faixas.iter().any(|&f| perto(f, (x(0.0), x(5.0)))),
        "o laço do clipe some: {faixas:?}"
    );
    let (fundo, _) = tintas(Theme::Forge);
    let caixa = ops.iter().any(
        |o| matches!(o, DrawOp::Fill { brush: ph2d_vector::Brush::Solid(c), .. } if *c == fundo),
    );
    assert!(
        !caixa,
        "a caixa do trecho mora na faixa Markers, não na régua"
    );
    let (ops, _) = pinta(&s, None);
    let faixas = faixas_do_laco(&ops);
    assert!(
        faixas.iter().any(|&f| perto(f, (x(0.0), x(5.0)))),
        "sem trecho, o laço do clipe: {faixas:?}"
    );
}

/// As caixas de texto (glifos: a caixa de um glifo é a origem, altura 0) de uma régua pintada.
fn textos(caixas: &[Rect]) -> Vec<Rect> {
    caixas.iter().copied().filter(|r| r.h == 0.0).collect()
}

/// ⭐ **A régua dos cartões mostra TODOS os números** (a foto 24 perdia o 0, o 1 e o 2 para os nomes):
/// os marcadores moram na faixa «Markers», por isso a régua com nomes pinta exactamente os mesmos textos
/// que sem eles — um por segundo inteiro à vista (0..8 a 90 px/s em 800 px), e nenhum nome. Medido nas
/// caixas PINTADAS (`caixas_pintadas`). *Mutação: pintar os marcadores na régua ⇒ reprova.*
#[test]
fn a_regua_mostra_todos_os_numeros() {
    let s = snap();
    let mut sem = s.clone();
    for m in &mut sem.markers {
        m.1.clear();
    }
    let (_, n) = pinta(&sem, None);
    let (_, w) = pinta(&s, Some((0, 1)));
    let (numeros, com) = (textos(&n), textos(&w));
    let primeiros: std::collections::BTreeSet<i32> = numeros.iter().map(|r| r.x as i32).collect();
    let segundos = (f64::from(REGIAO.w) / PX_S).floor() as usize + 1;
    assert!(
        primeiros.len() >= segundos,
        "um número por segundo: {primeiros:?}"
    );
    assert_eq!(com, numeros, "com nomes, a régua pinta os mesmos textos");
}
