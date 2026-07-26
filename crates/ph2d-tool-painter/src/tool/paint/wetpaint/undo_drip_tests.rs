//! **O ESCORRIDO QUE SOBROU DO UNDO** (smoke do Enio, 2026-07-26) — filho de [`super`].
//!
//! Report, com foto: *"quase tudo perfeito, mas o escorrido de um traço sobrou do Undo (o traço acima
//! foi desfeito corretamente)"*. O traço some, e duas gotinhas ficam na tela.
//!
//! ⚠️ **A água CONTINUA correndo depois do pen-up.** O `close_stroke` grava a entrada de undo com o
//! `after` do instante do pen-up; a sim segue tickando e **compositando no `canvas_rgba`** — escritas de
//! canvas **sem entrada de undo nenhuma**. O que o Ctrl+Z devolve depois disso é a pergunta.

use super::*;
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use ph2d_painter_brush::Falloff;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Canvas branco + pincel molhado com GRAVIDADE alta — é ela que faz o escorrido.
fn dripping(w: u32, h: u32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
    let b = BrushSpec {
        radius_px: 12.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.8, 0.1, 0.1],
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.set_paint_media(PaintMedia::WetPaint);
    t
}

/// **A ÁGUA CORRE DEPOIS DO PEN-UP, E O UNDO TEM DE LEVAR O QUE ELA PINTOU.**
///
/// A entrada de undo é gravada no `close_stroke` com o `after` do PEN-UP. Todo tick seguinte da sim
/// composita pigmento novo no canvas **sem gravar entrada nenhuma** — e é literalmente isso que um
/// escorrido é: tinta que chega onde o traço não passou, depois que ele acabou.
///
/// O oráculo é o único que importa para o artista: **desfazer o traço devolve a tela de antes dele**,
/// ao byte. Sem isso, o Ctrl+Z deixa marcas que ninguém consegue apagar com Ctrl+Z (o próximo Ctrl+Z
/// desfaz outra coisa).
#[test]
fn undoing_a_wet_stroke_takes_the_drip_the_sim_painted_after_the_pen_up() {
    let mut t = dripping(160, 220);
    // Gravidade no talo: o escorrido tem de existir para o gate poder medi-lo.
    use ph2d_wet_paint::tuning::{KNOB_DEFS, Knob};
    t.paint
        .wetpaint
        .knobs
        .set(Knob::Gravity, KNOB_DEFS[Knob::Gravity as usize].max);
    t.paint.wetpaint.knobs.water = 1.0;
    let pristine = t.canvas_rgba.as_ref().clone();

    // Um traço curto no ALTO da tela, para o escorrido ter para onde descer.
    t.on_canvas_pointer(cp([40.0, 30.0], PointerPhase::Down));
    for k in 1..=6u8 {
        t.on_canvas_pointer(cp([40.0 + f32::from(k) * 8.0, 30.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([90.0, 30.0], PointerPhase::Up));
    let at_pen_up = t.canvas_rgba.as_ref().clone();

    // …e a água segue correndo. É aqui que o escorrido nasce, e nenhuma destas escritas grava undo.
    for _ in 0..240 {
        t.paint_tick(1.0 / 40.0);
    }
    let after_drip = t.canvas_rgba.as_ref().clone();
    let dripped = at_pen_up
        .iter()
        .zip(&after_drip)
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        dripped > 0,
        "a fixture nao contem o fenomeno: a sim nao pintou NADA depois do pen-up (gravidade? ticks?)"
    );

    assert!(t.undo_last(), "havia um traco para desfazer");
    let restored = t.canvas_rgba.as_ref().clone();
    let leftover = pristine
        .iter()
        .zip(&restored)
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        leftover, 0,
        "o undo deixou {leftover} bytes na tela ({dripped} bytes tinham sido pintados pela sim depois \
         do pen-up). O traco foi desfeito; o ESCORRIDO sobrou."
    );
}

/// **DOIS traços, e o escorrido nasce ENTRE eles** — a forma que a divergência documentada exige.
///
/// Com UM traço só o undo sai limpo (o gate acima), e a razão é que o `cursor` do histórico é o `after`
/// do pen-up, que **não** tem o escorrido: instalá-lo o apaga. Com DOIS, o `cursor` no momento de
/// desfazer o primeiro já é um estado **que contém o escorrido**, e o delta só reescreve a JANELA do
/// passo desfeito — o que estiver fora dela **sobrevive**.
///
/// ⚠️ É literalmente a única mudança de comportamento que o doc do [`crate::undo`] nomeia:
/// *"o antigo instalava o snapshot inteiro e APAGAVA aquela escrita; o delta a PRESERVA fora da janela
/// do passo desfeito. Não há repro conhecido … fica escrito porque é a diferença que um smoke poderia
/// encontrar."* O smoke do Enio encontrou.
#[test]
fn undoing_back_through_a_drip_that_was_painted_between_two_strokes() {
    let mut t = dripping(160, 220);
    use ph2d_wet_paint::tuning::{KNOB_DEFS, Knob};
    t.paint
        .wetpaint
        .knobs
        .set(Knob::Gravity, KNOB_DEFS[Knob::Gravity as usize].max);
    t.paint.wetpaint.knobs.water = 1.0;
    let pristine = t.canvas_rgba.as_ref().clone();

    let stroke = |t: &mut PainterTool, y: f32| {
        t.on_canvas_pointer(cp([40.0, y], PointerPhase::Down));
        for k in 1..=6u8 {
            t.on_canvas_pointer(cp([40.0 + f32::from(k) * 8.0, y], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([90.0, y], PointerPhase::Up));
    };

    stroke(&mut t, 30.0);
    let at_pen_up = t.canvas_rgba.as_ref().clone();
    for _ in 0..240 {
        t.paint_tick(1.0 / 40.0);
    }
    let dripped = at_pen_up
        .iter()
        .zip(t.canvas_rgba.as_ref())
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        dripped > 0,
        "a fixture nao contem o fenomeno: nada escorreu"
    );

    // O segundo traço, BEM LONGE do primeiro, para a janela dele não cobrir o escorrido.
    stroke(&mut t, 190.0);

    assert!(t.undo_last(), "o 2o traco");
    assert!(t.undo_last(), "o 1o traco");
    let leftover = pristine
        .iter()
        .zip(t.canvas_rgba.as_ref())
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        leftover, 0,
        "depois de desfazer os DOIS tracos sobraram {leftover} bytes na tela ({dripped} tinham sido \
         pintados pela sim entre eles). O traco foi desfeito; o ESCORRIDO sobrou."
    );
}
