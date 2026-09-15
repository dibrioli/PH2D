//! **The brush-cursor ring must draw the LIVE dab orientation, not the resting Angle.**
//!
//! With a slot following the stroke (Shape Rake / Flow, Grain Rake) the dab footprint turns onto the
//! stroke tangent every dab. The ring is the picture of that footprint, so it has to turn too — a
//! calligraphic nib aimed with a cursor that points somewhere else is worse than no cursor at all
//! (Enio 2026-07-19: *"permite que o círculo que representa o pincel rotacione em tempo real conforme
//! flow e rake"*).
//!
//! The tool publishes the composed rotor as `BrushSettings::dab_rotor`, and the ring must READ it rather
//! than re-derive an angle from `dab_angle_deg`. Two answers to "which way is this dab pointing?" drift,
//! and the one the artist sees would be the wrong one.
//!
//! ⚠️ **Why an arch-gate over the source:** the ring is pure draw in the shell. There is no unit test that
//! can observe it, and the behavioural gate that covers the rotor
//! (`the_brush_ring_rotor_turns_with_the_stroke_only_when_a_slot_follows`, in `ph2d-tool-painter`) stops at
//! the published value — mutating the SHELL to ignore that value leaves every test in the workspace green.
//! This is the gate that bleeds for that mutation.

use std::fs;
use std::path::PathBuf;

#[test]
fn the_brush_ring_wears_the_live_dab_rotor() {
    let src = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/ph2d-app-painter/src/painter_bridge_brush_ring.rs"),
    )
    .expect("the brush ring source");
    // Strip line comments so the prose above the code cannot satisfy (or trip) the checks below.
    let code: String = src
        .lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n");
    // ⛔⛔ **A PREMISSA DESTE GATE DISSOLVEU em 2026-09-14, e ele ficou MAIS FORTE.** Ele exigia
    // `bs.dab_rotor` no fonte do anel — a orientação viva, sim, mas **remontada ali**: o anel lia o
    // rotor e o achatamento e construía a elipse à mão, noutra crate. Quando a pegada passou a
    // carregar a deformação da arte, essa cópia ficou a desenhar a forma (e o tamanho) de REPOUSO
    // sobre uma arte dobrada.
    //
    // ⇒ o que se exige agora é mais: o anel **não tem lei de orientação nenhuma**. Ele pede a pegada
    // ao motor (`PainterTool::cursor_dab`, que compõe o `follow_rotor` sobre o rumo vivo) e percorre
    // o contorno dela. *Um consumidor sem lei própria não pode divergir do produtor.*
    assert!(
        code.contains("painter.cursor_dab()"),
        "the ring must ask the ENGINE for the footprint it will emit (`PainterTool::cursor_dab`),          which composes the live stroke-follow rotor — not rebuild the ellipse from a snapshot"
    );
    for lei in ["dab_rotor", "dab_angle_deg", "dab_flatten"] {
        assert!(
            !code.contains(lei),
            "the ring re-derives `{lei}` instead of walking the engine's footprint: a second copy              of the dab's orientation law is exactly what left the ring showing the RESTING shape              over deformed art"
        );
    }
}

/// **A hover must reach the painter**, or the ring can only ever aim once you have already committed.
///
/// The tool half is gated in `ph2d-tool-painter`
/// (`hovering_aims_the_brush_ring_before_the_stroke_starts`), but that drives `on_canvas_hover` directly.
/// Only the shell decides whether a real cursor move ever calls it — and `painter_canvas_move` returns
/// early when no stroke is open, which is exactly the branch a hover lives in. Delete the call there and
/// every test in the workspace stays green while the feature is dead in the product.
#[test]
fn a_hover_reaches_the_painter_when_no_stroke_is_open() {
    let src = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/input_dispatch/painter_canvas_input.rs"),
    )
    .expect("the painter canvas input source");
    let code: String = src
        .lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n");
    // The early-return branch for "no stroke open" must deliver the hover before it bails.
    let at = code
        .find("fn painter_canvas_move")
        .expect("painter_canvas_move");
    let body = &code[at..(at + 600).min(code.len())];
    let bail = body.find("STROKE_ACTIVE").expect("the no-stroke guard");
    let guard = &body[bail..];
    let ret = guard.find("return false").expect("the early return");
    assert!(
        guard[..ret].contains("deliver_canvas_hover"),
        "the no-stroke branch of `painter_canvas_move` must deliver the hover before returning, or the \
         brush ring can never aim before the first click"
    );
}
