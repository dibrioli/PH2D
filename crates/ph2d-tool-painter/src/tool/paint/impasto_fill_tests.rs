//! Gates do **FILL no Impasto** ([`super::impasto_fill`]).
//!
//! As duas metades que os organizam: **o corpo existe** (encher uma região no meio cuja razão de ser é a
//! espessura deixava um decalque plano) e **a borda tem perfil** (um platô com penhasco na fronteira lê
//! como recorte de papel, porque a luz lê `∇h`). E a terceira, que vale tanto quanto: **o digital não se
//! move um byte**.

use super::*;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::CanvasPaintTool;
use ph2d_editor_core::tool::{CanvasPointer, PointerPhase};
use ph2d_painter_brush::Falloff;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Tela branca com pincel preto. `impasto` liga o corpo (Depth cheio, falloff Smooth).
fn canvas(size: u32, radius: f32, impasto: bool) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let b = BrushSpec {
        radius_px: radius,
        hardness: 0.0,
        falloff: Falloff::Smooth,
        color: [0.0, 0.0, 0.0],
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    if impasto {
        t.set_brush_impasto(true);
        t.set_brush_impasto_depth(1.0);
        t.set_brush_falloff(Falloff::Smooth as u8);
    }
    t
}

/// A altura commitada da camada ativa, por texel (`0` sem relevo).
fn height(t: &PainterTool, size: u32, x: u32, y: u32) -> f32 {
    let Some(layer) = t.layers.active() else {
        return 0.0;
    };
    t.heights
        .get(&layer)
        .and_then(|f| f.get((y * size + x) as usize).copied())
        .unwrap_or(0.0)
}

/// **O digital não se move um byte.** Sem corpo em mãos a porta devolve o MESMO `Arc` — não uma cópia
/// igual —, e é isso que garante que o fill digital seja literalmente o código de antes.
///
/// **Mutação que sangra:** trocar a guarda por `true` (perfilar sempre) — o `Arc::ptr_eq` cai.
#[test]
fn a_digital_fill_gets_the_mask_verbatim_and_lays_no_body() {
    let mut t = canvas(32, 4.0, false);
    t.set_rect_selection(0, 0, 16, 32);
    let mask = std::sync::Arc::clone(&t.paint.selection_mask);
    assert!(
        std::sync::Arc::ptr_eq(&t.fill_selection_keep(), &mask),
        "sem corpo, a cobertura de um fill E a mascara — o mesmo Arc, sem copia"
    );
    t.selection_color_fill();
    let layer = t.layers.active().expect("uma camada");
    assert!(
        t.heights
            .get(&layer)
            .is_none_or(|f| f.iter().all(|&v| v == 0.0)),
        "e nenhum relevo nasce de um fill digital"
    );
}

/// **Com corpo em mãos, o Color Fill deposita RELEVO** — a metade que faltava e que o Enio pediu.
///
/// **Mutação que sangra:** o `deposit_fill_body` não ser chamado, ou o `arm_fill_body` recusar.
#[test]
fn an_impasto_color_fill_lays_body_where_it_lays_colour() {
    let mut t = canvas(64, 8.0, true);
    t.set_rect_selection(8, 8, 48, 48);
    t.selection_color_fill();
    assert_eq!(
        t.canvas_rgba[((32 * 64 + 32) * 4) as usize],
        0,
        "a cor foi para o miolo"
    );
    assert!(
        height(&t, 64, 32, 32) > 0.0,
        "e o CORPO tambem: {}",
        height(&t, 64, 32, 32)
    );
    assert_eq!(
        height(&t, 64, 2, 2),
        0.0,
        "fora da selecao nao ha corpo nenhum"
    );
}

/// **A borda da seleção ganha o perfil do Falloff**, e o oráculo é a FORMA da rampa, não a sua
/// existência: do aro para dentro a cobertura tem de subir monotonicamente até saturar, sobre a régua do
/// RAIO do pincel.
///
/// ⚠️ O controle é o `Constant` — o falloff que é `1` em toda parte. Com ele a borda tem de ficar DURA,
/// e é isso que prova que a rampa vem do perfil escolhido e não de um amaciamento que a wave instalou
/// por conta própria.
///
/// **Mutação que sangra:** ignorar o `falloff_weight` (usar `p` cru) — o `Constant` passa a rampar.
#[test]
fn the_selection_edge_wears_the_falloff_in_hand() {
    let probe = |f: Falloff| {
        let mut t = canvas(64, 12.0, true);
        t.set_brush_falloff(f as u8);
        t.set_rect_selection(16, 16, 32, 32);
        let keep = t.fill_selection_keep();
        // Uma travessia da borda esquerda (x = 16) para dentro, na altura do meio.
        (0..14)
            .map(|k| keep[(32 * 64 + 16 + k) as usize])
            .collect::<Vec<u8>>()
    };
    let hard = probe(Falloff::Constant);
    assert!(
        hard.iter().all(|&v| v == 255),
        "o Constant nao rampa nada: {hard:?}"
    );
    let soft = probe(Falloff::Smooth);
    assert!(soft[0] < 64, "o Smooth comeca quase transparente na borda");
    assert!(soft[13] > 240, "e satura ao chegar ao raio do pincel");
    assert!(
        soft.windows(2).all(|w| w[1] >= w[0]),
        "e a rampa e monotona da borda para dentro: {soft:?}"
    );
}

/// **Sem borda não há rampa.** Uma seleção que cobre a tela inteira não tem fronteira visível a
/// perfilar, e inventar uma no limite do documento seria uma moldura que ninguém desenhou.
#[test]
fn a_full_canvas_selection_has_no_edge_to_profile() {
    let mut t = canvas(32, 8.0, true);
    t.selection_select_all();
    let keep = t.fill_selection_keep();
    assert!(
        keep.iter().all(|&v| v == 255),
        "tudo dentro: cobertura cheia em toda parte"
    );
}

/// **O balde deposita corpo pelo MESMO conjunto que a cor pintou** — e o commit é o `close_stroke` do
/// Done, exatamente como num traço.
///
/// **Mutação que sangra:** o `refill_from_snapshot` não armar o corpo (`fill_mask` sempre `None`).
#[test]
fn the_bucket_lays_body_over_the_region_it_flooded() {
    let mut t = canvas(64, 8.0, true);
    // Uma mancha preta no meio de tela branca — o balde despeja vermelho dentro dela.
    for y in 20..44u32 {
        for x in 20..44u32 {
            let i = ((y * 64 + x) * 4) as usize;
            let buf = std::sync::Arc::make_mut(&mut t.canvas_rgba);
            buf[i..i + 3].copy_from_slice(&[0, 0, 0]);
        }
    }
    t.paint.brush.color = [1.0, 0.0, 0.0]; // vermelho sobre a mancha preta: o flood muda algo
    t.set_paint_tool_mode("fill");
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));
    t.fill_commit();
    assert!(
        height(&t, 64, 32, 32) > 0.0,
        "o balde deixou corpo onde despejou: {}",
        height(&t, 64, 32, 32)
    );
    assert_eq!(height(&t, 64, 2, 2), 0.0, "e nao fora da mancha");
}

/// **Cancelar o balde não deixa corpo armado.** O relevo vivo acompanha a cor que ele descrevia; um
/// `fill_cancel` que devolvesse os pixels e guardasse o corpo deixaria relevo sobre tinta que não existe
/// — a doença exata do "Line + último undo" que esta linha já pagou.
///
/// **Mutação que sangra:** tirar o `reset_stroke_height` do `fill_cancel`.
#[test]
fn cancelling_a_bucket_drops_the_body_it_had_armed() {
    let mut t = canvas(64, 8.0, true);
    for y in 20..44u32 {
        for x in 20..44u32 {
            let i = ((y * 64 + x) * 4) as usize;
            let buf = std::sync::Arc::make_mut(&mut t.canvas_rgba);
            buf[i..i + 3].copy_from_slice(&[0, 0, 0]);
        }
    }
    t.paint.brush.color = [1.0, 0.0, 0.0]; // vermelho sobre a mancha preta: o flood muda algo
    t.set_paint_tool_mode("fill");
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));
    assert!(
        !t.paint.relief.stroke_paint.is_empty(),
        "o corpo estava armado antes do cancel"
    );
    t.fill_cancel();
    assert!(
        t.paint.relief.stroke_paint.is_empty(),
        "e o cancel o descartou junto com a cor"
    );
    assert_eq!(height(&t, 64, 32, 32), 0.0, "nada commitado");
}

/// **O raio que mede a rampa é o raio que mede a ALTURA.** Os dois números saem do mesmo lugar, e é isso
/// que impede a borda e a espessura de discordarem sobre que pincel foi este: um fill com pincel grande
/// é um empaste grosso de borda larga.
///
/// **Mutação que sangra:** o `arm_fill_body` escrever um raio fixo em vez do do pincel — a altura para
/// de escalar.
#[test]
fn a_bigger_brush_fills_thicker_and_softer() {
    let peak = |r: f32| {
        let mut t = canvas(96, r, true);
        t.set_rect_selection(16, 16, 64, 64);
        t.selection_color_fill();
        height(&t, 96, 48, 48)
    };
    let (small, big) = (peak(6.0), peak(24.0));
    assert!(
        big > small * 2.0,
        "a altura escala com o raio do pincel: {small} contra {big}"
    );
}
