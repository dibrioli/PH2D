//! **O Smudge da aquarela não FORKA o canvas.**
//!
//! `smear_wet_base` muta a base que o composite lê, via `Arc::make_mut`. Enquanto `wet_session_base` e
//! `watercolor_base` apontarem o MESMO `Arc`, esse `make_mut` vê dois donos fortes e **clona o
//! documento inteiro** — e a re-partilha no fim da função restabelece o par, então o fork acontecia em
//! **TODO evento de smudge**, não uma vez. Medido a 4096² com os knobs do report do Enio de
//! 2026-08-02: o carimbo custava **49,6 ms/quadro** contra 1,6 sem Smudge, e crescia **7,4×** com a
//! tela enquanto o resto da aquarela é limitado pela PEGADA.
//!
//! ⚠️ **Os dois gates não são redundantes, e é por desenho.** Um passe canvas-sized NOVO passaria pelo
//! primeiro (o ponteiro fica estável se ninguém realoca *esta* alocação) e cairia no segundo; e o
//! primeiro pega o defeito **sem relógio**, que é o que o torna confiável numa máquina disputada
//! (doc 28 §5.49). É o par que a §5.12 usou no Wet Paint, pelo mesmo motivo.

use super::*;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_painter_brush::Falloff;
use std::time::Instant;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Aquarela com o Smudge LIGADO — sem ele não há o que medir (o knob nasce em 0, que é exatamente
/// por que a tabela de ablação do doc 31 o usou como piso de ruído e nunca mediu o custo dele).
fn smudging(size: u32, radius: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let b = BrushSpec {
        radius_px: radius,
        hardness: 0.5,
        falloff: Falloff::Smooth,
        strength: 1.0,
        color: [0.8, 0.2, 0.1],
        space_attenuation: false,
        wet_smudge: 0.197,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.set_paint_media(PaintMedia::Watercolor);
    t
}

/// **A PROPRIEDADE, e ela não precisa de relógio: o smudge não FORKA o canvas.**
///
/// `Arc::make_mut` com dois donos **aloca e copia**; com um dono ele devolve o mesmo buffer. O produto
/// conta os forks (`WashCadence::base_forks`), então o oráculo é uma CONTAGEM — vale com a máquina
/// disputada, e falha pelo motivo certo.
///
/// ⚠️ O oráculo do ENDEREÇO do buffer foi tentado primeiro e **descartado por medição**: o alocador
/// devolve a alocação recém-liberada, então o ponteiro sai e VOLTA, e o gate lia *"não moveu"* sobre um
/// produto que copiava 67 MB em todo evento. *Um oráculo que o alocador pode satisfazer por acidente
/// não é oráculo.*
#[test]
fn the_smudge_does_not_fork_the_canvas_on_every_event() {
    const SIZE: u32 = 512;
    const RADIUS: f32 = 40.0;
    let mut t = smudging(SIZE, RADIUS);
    let mid = f64::from(SIZE / 2) as f32;
    let x0 = RADIUS + 10.0;

    t.on_canvas_pointer(cp([x0, mid], PointerPhase::Down));
    t.paint_tick(1.0 / 60.0);
    // O pen-down congela a base; o que este gate mede são os eventos DEPOIS dele.
    t.wash.base_forks = 0;
    let mut smears = 0u32;
    for i in 1..=8 {
        t.on_canvas_pointer(cp(
            [x0 + f64::from(i) as f32 * 12.0, mid],
            PointerPhase::Move,
        ));
        t.paint_tick(1.0 / 60.0);
        smears += 1;
    }
    t.on_canvas_pointer(cp([x0 + 96.0, mid], PointerPhase::Up));

    // Controle positivo: sem esfregar não há fork a medir, e o gate seria verde por vácuo.
    assert!(
        smears > 0 && t.paint.brush.wet_smudge > 0.0 && t.paint.wet_session_base.is_some(),
        "a fixture tem de ESFREGAR sobre uma base congelada — senão este gate não mede nada"
    );
    assert_eq!(
        t.wash.base_forks, 0,
        "o smudge forkou o canvas em {} de {smears} eventos — o `make_mut` voltou a ver dois donos \
         fortes e está CLONANDO o documento (67 MB a 4096²) a cada evento de ponteiro",
        t.wash.base_forks
    );
}

/// **A CONSEQUÊNCIA: o carimbo é limitado pela PEGADA, não pela TELA.**
///
/// Um pincel cobre o mesmo número de texels seja qual for o documento, então quadruplicar a área não
/// pode mover o custo. É razão e não wall-clock **de propósito** — uma barra absoluta mede o perfil
/// deste box, e a razão sobrevive à deriva dele (o mesmo passo de produto já foi medido a 14,5 e a
/// 30,2 ms nesta máquina sem uma linha mudar).
#[test]
#[cfg_attr(debug_assertions, ignore = "razão de perf: só em release")]
fn the_smudging_stamp_is_footprint_bound_not_canvas_bound() {
    const RADIUS: f32 = 250.0;
    const EV: u32 = 4;
    const FRAMES: u32 = 12;

    let stamp_ms = |size: u32| -> f64 {
        let mut t = smudging(size, RADIUS);
        let mid = f64::from(size / 2) as f32;
        let x0 = RADIUS + 20.0;
        let step = 600.0 / f64::from(FRAMES * EV) as f32;
        t.on_canvas_pointer(cp([x0, mid], PointerPhase::Down));
        let _ = t.take_preview_arc();
        let mut ms = Vec::new();
        let mut k = 0u32;
        for _ in 0..FRAMES {
            let t0 = Instant::now();
            for _ in 0..EV {
                k += 1;
                t.on_canvas_pointer(cp(
                    [x0 + step * f64::from(k) as f32, mid],
                    PointerPhase::Move,
                ));
            }
            ms.push(t0.elapsed().as_secs_f64() * 1e3);
            t.paint_tick(1.0 / 60.0);
            let _ = t.take_preview_arc();
        }
        ms.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        ms[ms.len() / 2]
    };

    // ⚠️ O par é 2048/4096 porque é ONDE o fenômeno foi medido (7,4× com o defeito, 1,01× sem). Um par
    // menor mede o fork contra um fundo grande demais e fica verde sobre o bug.
    let small = stamp_ms(2048);
    let big = stamp_ms(4096);
    let ratio = big / small;
    assert!(
        ratio < 2.0,
        "o carimbo com Smudge cresceu {ratio:.2}× para 4× a área ({small:.3} → {big:.3} ms) — ele \
         voltou a caminhar o PLANO, e a causa conhecida é o fork do canvas no `make_mut`"
    );
}
