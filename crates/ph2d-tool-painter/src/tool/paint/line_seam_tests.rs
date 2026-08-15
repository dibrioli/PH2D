//! **O SEAM do card Line** — o clique de cada slider chega ao barramento E leva a algum lugar.
//!
//! ⚠️ **Este arquivo nasce de um buraco MEDIDO, e o buraco era da W6:** os três sliders da fita
//! shiparam com id, row, `populate`, encaminhamento e setter — e **nenhum gate os exercitava**. Um
//! `grep` pelo id nos testes do repo não devolvia nada. As duas metades que o `wiring_parity` cobre
//! (*o widget existe* e *ele está registrado*) são cegas à terceira: **o clique chega ao tool?** —
//! e a quarta, **a sequência leva a algum lugar?**, é categoria à parte, porque um id pode ser
//! roteado para um setter que escreve o campo errado.
//!
//! ⚠️ **Este é o defeito exato que esta wave pagou uma hora a diagnosticar**, um sistema adiante: a
//! faixa nascia com o motor a costurar 343 travessas por traço e o depósito **mudo**, porque a porta
//! `sews_threads` enumerava os tipos. Cada elo da corrente precisa de um gate; a corrente inteira
//! *parecendo* ligada não é um.
//!
//! O oráculo é o CAMPO do pincel, nunca o setter — chamar o setter e afirmar que ele escreveu é o
//! oráculo auto-referente que esta casa já pegou verde três vezes.

use super::media::PaintMedia;
use crate::tool::PainterTool;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::tool::{PanelEvent, Tool};
use ph2d_painter_brush::line_kind::LineKind;

fn tool() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::Digital);
    t
}

/// **CADA slider do card Line chega ao pincel pelo barramento** — id → `handle_panel_event` → campo.
///
/// ⚠️ **A tabela pergunta pelo CAMPO que cada id promete**, e é isso que a torna capaz de pegar um
/// encaminhamento cruzado (dois ids no mesmo setter): o valor entra por um id e é lido no campo que
/// o doc daquele id nomeia.
#[test]
fn every_line_card_slider_reaches_the_brush_through_the_bus() {
    #[allow(clippy::type_complexity)]
    let rows: [(&str, ph2d_editor_core::NodeId, fn(&PainterTool) -> f32); 8] = [
        ("Reach", core_ids::PAINTER_LINE_SKETCHY_REACH, |t| {
            t.paint.brush.sketchy_reach
        }),
        ("Density", core_ids::PAINTER_LINE_SKETCHY_DENSITY, |t| {
            t.paint.brush.sketchy_density
        }),
        ("Width", core_ids::PAINTER_LINE_SKETCHY_WIDTH, |t| {
            t.paint.brush.thread_width_px
        }),
        ("Opacity", core_ids::PAINTER_LINE_SKETCHY_OPACITY, |t| {
            t.paint.brush.thread_opacity
        }),
        ("History", core_ids::PAINTER_LINE_WIRE_HISTORY, |t| {
            t.paint.brush.wire_history
        }),
        ("Weight", core_ids::PAINTER_LINE_RIBBON_WEIGHT, |t| {
            t.paint.brush.ribbon_weight
        }),
        ("Friction", core_ids::PAINTER_LINE_RIBBON_FRICTION, |t| {
            t.paint.brush.ribbon_friction
        }),
        ("Rungs", core_ids::PAINTER_LINE_RIBBON_RUNGS, |t| {
            t.paint.brush.ribbon_rungs
        }),
    ];
    for (nome, id, ler) in rows {
        // ⚠️ **O oráculo é a DIFERENÇA entre duas posições da pista, não a diferença contra o
        // default** — e o gate pegou esta fixture minha antes de eu a shipar: o `Reach` mapeia a
        // pista em `0..SKETCHY_REACH_MAX`, então `0,25` cai exactamente no `1.0` de fábrica, e a
        // asserção *"o campo mudou"* reprovava uma row perfeitamente viva. Uma row MUDA devolve o
        // mesmo número nas duas posições; uma viva não pode.
        let mut t = tool();
        t.handle_panel_event(PanelEvent::SetValue(id, 0.25));
        let baixo = ler(&t);
        t.handle_panel_event(PanelEvent::SetValue(id, 0.75));
        let alto = ler(&t);
        assert!(
            (alto - baixo).abs() > 1e-6,
            "o slider {nome} é MUDO no barramento: 0,25 e 0,75 dão o mesmo {baixo}"
        );
    }
}

/// **A `Gravity` é a excepção que prova a régua** — ela nasce em `0` e a pista dela é a mesma, então
/// o gate acima a pegaria por acidente se ela fosse roteada para o campo errado. Aqui ela é
/// perguntada sozinha, contra o valor que a fração `0,5` promete.
#[test]
fn the_ribbon_gravity_slider_lands_on_gravity_and_nothing_else() {
    let mut t = tool();
    let (peso, atrito, travessas) = (
        t.paint.brush.ribbon_weight,
        t.paint.brush.ribbon_friction,
        t.paint.brush.ribbon_rungs,
    );
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_LINE_RIBBON_GRAVITY,
        0.5,
    ));
    assert!(
        (t.paint.brush.ribbon_gravity - 0.5).abs() < 1e-6,
        "a Gravity não chegou: {}",
        t.paint.brush.ribbon_gravity
    );
    assert_eq!(
        t.paint.brush.ribbon_weight, peso,
        "a Gravity mexeu no Weight"
    );
    assert_eq!(
        t.paint.brush.ribbon_friction, atrito,
        "a Gravity mexeu no Friction"
    );
    assert_eq!(
        t.paint.brush.ribbon_rungs, travessas,
        "a Gravity mexeu nos Rungs"
    );
}

/// **A SEQUÊNCIA leva a algum lugar** — a quarta condição, e a única que este arquivo não podia
/// herdar: escolher `Ribbon` no dropdown e arrastar os `Rungs` tem de ligar a FAIXA.
///
/// ⚠️ O oráculo é a porta que o motor E o depósito perguntam (`sews_threads`), não o campo — um
/// campo que se move sem que a porta abra é exatamente o defeito que esta wave encontrou.
#[test]
fn arming_the_ribbon_and_dragging_rungs_turns_the_band_on() {
    let mut t = tool();
    t.paint.brush.line_kind = LineKind::Ribbon;
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_LINE_RIBBON_RUNGS,
        0.0,
    ));
    assert!(
        !t.paint.brush.sews_threads(),
        "com Rungs 0 a faixa tem de estar desligada (é a linha atrasada sozinha)"
    );
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_LINE_RIBBON_RUNGS,
        0.6,
    ));
    assert!(
        t.paint.brush.sews_threads(),
        "arrastar os Rungs não ligou a faixa: o gesto não leva a lugar nenhum"
    );
}
