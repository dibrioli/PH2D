//! Portões da linha do fim nos cartões (dono 10/11): sai depois das peças, de alto a baixo.

use super::*;
use ph2d_editor_core::screens::hero::HeroLayout;
use ph2d_editor_core::screens::slot::{Slot, SlotSet};
use ph2d_timeline::{StackHost, StripSource, TimelineDoc, TimelineState};
use ph2d_vector::{DrawOp, Shape};

/// Pinta as faixas do Arrange e depois a régua (a ordem de `paint.rs`) com uma peça `[1, 3)` a
/// cruzar o fim autorado em 2 s; devolve as ops, quantas eram das faixas e a régua.
fn pinta_arrange(cartoes: bool) -> (Vec<DrawOp>, usize, Rect) {
    pinta_arrange_com(cartoes, None)
}

fn pinta_arrange_com(cartoes: bool, laco: Option<(f64, f64)>) -> (Vec<DrawOp>, usize, Rect) {
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
    snap.loop_range = laco;
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

/// ⭐ **O laço na régua é a faixa FINA da maquete B no topo** (dono 10/11, onda 3f): nos cartões a
/// faixa do laço `[0,5; 1,5)` tem `TIMELINE_LACO_REGUA_PX` de altura, colada ao topo da régua; o
/// clássico continua a régua inteira. *Mutação: a altura `RULER_H` nos cartões ⇒ reprova.*
#[test]
fn nos_cartoes_o_laco_e_uma_faixa_fina_no_topo_da_regua() {
    for cartoes in [true, false] {
        ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(Some(cartoes));
        let (_, cor) = crate::ruler::faixa_do_laco(Theme::Forge);
        ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(None);
        let (ops, _, regiao) = pinta_arrange_com(cartoes, Some((0.5, 1.5)));
        let faixas: Vec<_> = ops
            .iter()
            .filter_map(|o| match o {
                DrawOp::Fill {
                    brush: ph2d_vector::Brush::Solid(c),
                    path,
                    ..
                } if *c == cor => Some(path.bounding_box()),
                _ => None,
            })
            .filter(|b| (b.x0 - 200.0).abs() < 1.0 && (b.x1 - 400.0).abs() < 1.0)
            .collect();
        let alto = if cartoes {
            ph2d_tokens::chrome::TIMELINE_LACO_REGUA_PX
        } else {
            RULER_H
        };
        assert!(
            faixas
                .iter()
                .any(|b| (b.height() - f64::from(alto)).abs() < 0.5
                    && (b.y0 - f64::from(regiao.y)).abs() < 0.5),
            "cartões={cartoes}: a faixa de {alto} px no topo da régua, achei {faixas:?}"
        );
    }
}

/// `c` (com a opacidade dele) composto sobre `fundo`, em sRGB como o Vello compõe.
fn sobre(c: Color, fundo: Color) -> [f32; 3] {
    let ([r, g, b, a], f) = (c.components, fundo.components);
    [0, 1, 2].map(|i| [r, g, b][i] * a + f[i] * (1.0 - a))
}

fn dist(x: [f32; 3], y: [f32; 3]) -> f64 {
    let lab = |c: [f32; 3]| {
        let u = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        let (l, ch, h) = ph2d_tokens::srgb_to_oklch(u(c[0]), u(c[1]), u(c[2]));
        (l, ch * h.to_radians().cos(), ch * h.to_radians().sin())
    };
    let (p, q) = (lab(x), lab(y));
    ((p.0 - q.0).powi(2) + (p.1 - q.1).powi(2) + (p.2 - q.2).powi(2)).sqrt()
}

/// ⭐ **A faixa do laço lê-se sobre a régua em todo tema** (onda 3f). Piso MEDIDO em `Theme::ALL`
/// (OKLab, a faixa composta sobre o fundo da régua dos cartões): pior 0,367 (Sunstone) → piso 0,33. Os números da régua ficam na tinta de
/// sempre sobre o fundo de sempre (0,332), abaixo da faixa.
#[test]
fn a_faixa_do_laco_le_se_sobre_a_regua_em_todo_tema() {
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(Some(true));
    let mut pior = (f64::MAX, Theme::Forge);
    for theme in Theme::ALL {
        let regua = crate::folha_tinta::cor(ColorToken::TimelineRulerBg, theme);
        let (_, faixa) = crate::ruler::faixa_do_laco(theme);
        let [r, g, b, _] = regua.components;
        let d = dist(sobre(faixa, regua), [r, g, b]);
        if d < pior.0 {
            pior = (d, theme);
        }
    }
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(None);
    assert!(
        pior.0 >= 0.33,
        "laço × régua a {:.3} em {:?}",
        pior.0,
        pior.1
    );
}

/// ⭐ **A caixa do trecho lê-se na régua e os números leem-se nela, em todo tema** (maquete B6, `.c13sec`).
/// Pisos MEDIDOS em `Theme::ALL` (OKLab, sobre o fundo da régua dos cartões), 10/11: fundo × régua pior
/// 0,104 (Sunstone) → 0,09 · fio × régua 0,341 (Sunstone) → 0,30 · número (`Text2`) × caixa 0,195 (Forge) → 0,17.
#[test]
fn a_caixa_do_trecho_le_se_e_deixa_ler_os_numeros_em_todo_tema() {
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(Some(true));
    let mut pior = [(f64::MAX, Theme::Forge); 3];
    for theme in Theme::ALL {
        let regua = crate::folha_tinta::cor(ColorToken::TimelineRulerBg, theme);
        let (fundo, fio) = crate::ruler_trecho::tintas(theme);
        let [r, g, b, _] = regua.components;
        let caixa = sobre(fundo, regua);
        let texto = crate::folha_tinta::cor(ColorToken::Text2, theme).components;
        let d = [
            dist(caixa, [r, g, b]),
            dist(sobre(fio, regua), [r, g, b]),
            dist([texto[0], texto[1], texto[2]], caixa),
        ];
        for (p, d) in pior.iter_mut().zip(d) {
            if d < p.0 {
                *p = (d, theme);
            }
        }
    }
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(None);
    let pisos = [0.09, 0.30, 0.17];
    for (p, piso) in pior.iter().zip(pisos) {
        assert!(p.0 >= piso, "{:.3} < {piso} em {:?}", p.0, p.1);
    }
}
