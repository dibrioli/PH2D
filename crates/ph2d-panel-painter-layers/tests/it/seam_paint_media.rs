//! **The paint's MEDIUM is one dropdown, and exactly one medium's section is on screen.**
//!
//! Until 2026-07-22 the three media were three independent `Enable` checkboxes, one at the head of each
//! section (Enio: *"temos na seção de modo da pintura 3 checkbox … no lugar dos checkbox coloque um
//! dropdown para o modo de pintura com os 4 modos. O padrão é o Digital normal"*). Three booleans
//! express eight states of which four mean anything, and the fourth medium — the plain Blender-style
//! brush — had no name at all: it was "none of the boxes".
//!
//! These gates are **pointer-driven** for the reason this crate's seam files keep paying for: a widget
//! that paints, registers a hit rect and is forwarded by `event.rs` is still stone dead under the mouse
//! unless `populate` gave it an `InteractiveState`. A dropdown has *two* such widgets — the chip and,
//! once open, every option — so both halves are clicked here rather than synthesised.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::tool::Tool;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_tool_painter::{PaintMedia, PainterTool};
use ph2d_ui_testkit::MockPanelHost;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 900.0)
}

/// The four media, paired with a widget that only ITS section paints — the section header is the
/// honest probe, because it is the one id every medium's section owns unconditionally.
const SECTIONS: [(PaintMedia, NodeId, &str); 3] = [
    (
        PaintMedia::Watercolor,
        core_ids::PAINTER_WATERCOLOR_SECTION,
        "Watercolor",
    ),
    (
        PaintMedia::Impasto,
        core_ids::PAINTER_IMPASTO_SECTION,
        "Impasto",
    ),
    (
        PaintMedia::WetPaint,
        core_ids::PAINTER_WETPAINT_SECTION,
        "Wet Paint",
    ),
];

fn painted(tool: &PainterTool) -> (MockPanelHost, PainterLayersPanelState, Vec<(NodeId, Rect)>) {
    set_current_brush(Some(tool.brush_settings()));
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let rects = host.paint::<PainterLayersPanel>(&mut st, viewport());
    (host, st, rects)
}

fn rect_of(rects: &[(NodeId, Rect)], id: NodeId) -> Option<Rect> {
    rects
        .iter()
        .find(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
}

fn centre(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

/// One real click, all the way through: dispatcher → panel → bus → tool.
fn click_through(
    host: &mut MockPanelHost,
    st: &mut PainterLayersPanelState,
    tool: &mut PainterTool,
    x: f32,
    y: f32,
) {
    for ev in host.click_at(x, y) {
        host.apply_panel_event::<PainterLayersPanel>(st, ev);
    }
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
        }
    }
}

/// **THE gate: a pointer can pick every medium.**
///
/// Opens the chip with a click, repaints so the popover registers its options, and clicks the option —
/// the two-step a dropdown actually is. A synthetic `Click(option_id)` would pass with the chip dead
/// under the mouse, which is exactly how the Wet Paint Enable checkbox once shipped unclickable.
///
/// **Mutations that bleed:** drop the chip's deferred popover pass in `paint_brush_popovers` (the
/// options never reach the screen — *"clicking the chip did not open it"*), or drop its row from the
/// `option_route` table (the option is clicked and decodes to nothing — *"did not switch the medium"*).
///
/// ⚠️ **A third mutation SURVIVES, and it is documented rather than fixed:** removing
/// `PAINTER_BRUSH_MEDIA` from `populate`'s Dropdown loop changes nothing, because `paint_dropdown_chip`
/// already does `register_if_absent` for every chip it paints. The populate entry is belt to that
/// braces — it keeps this chip in the same list as its eight siblings, and it is what
/// `architecture_panel_wiring_parity` reads. Two layers, one gate: worth knowing, not worth pretending.
#[test]
fn a_pointer_can_pick_every_medium() {
    for i in 0..PaintMedia::COUNT {
        let want = PaintMedia::from_u8(i);
        // Start from a medium that is NOT the target, so "it was already selected" cannot pass this.
        let mut tool = PainterTool::default();
        if want == PaintMedia::Digital {
            tool.set_paint_media(PaintMedia::Impasto);
        }
        let (mut host, mut st, rects) = painted(&tool);
        let chip = rect_of(&rects, core_ids::PAINTER_BRUSH_MEDIA)
            .unwrap_or_else(|| panic!("the Paint Mode chip is not painted at all"));
        let (cx, cy) = centre(chip);
        click_through(&mut host, &mut st, &mut tool, cx, cy);

        // Re-paint with the chip OPEN: this is the pass that puts the options on screen.
        set_current_brush(Some(tool.brush_settings()));
        let rects = host.paint::<PainterLayersPanel>(&mut st, viewport());
        let opt_id = core_ids::painter_brush_media_option_id(i);
        let opt = rect_of(&rects, opt_id).unwrap_or_else(|| {
            panic!(
                "clicking the Paint Mode chip did not open it — option {:?} ({}) never reached the \
                 screen, so no pointer can select it",
                want,
                want.name()
            )
        });
        let (ox, oy) = centre(opt);
        click_through(&mut host, &mut st, &mut tool, ox, oy);

        assert_eq!(
            tool.paint_media(),
            want,
            "clicking the {:?} option did not switch the medium — the decode/option-route row or the \
             tool's SelectOption arm is missing",
            want
        );
    }
}

/// **Exactly one medium's section is painted, and Digital paints none.**
///
/// This is what buying the dropdown bought: the exclusivity is a property of the UI's shape, not a
/// hand-maintained set of `!other_flag` guards (the old code carried one — "hide Watercolor while Wet
/// Paint is armed" — and had no such guard between Impasto and the other two).
///
/// **Mutation that must bleed:** let the `match` in `paint_brush_sections` fall through to paint every
/// section.
#[test]
fn exactly_one_medium_section_is_painted() {
    // Digital: the default, and the absence of the other three.
    let (_h, _s, rects) = painted(&PainterTool::default());
    for (_, id, name) in SECTIONS {
        assert!(
            rect_of(&rects, id).is_none(),
            "the Digital medium paints the {name} section — Digital is the plain Blender-style brush, \
             which is precisely the absence of the other three"
        );
    }
    assert!(
        rect_of(&rects, core_ids::PAINTER_BRUSH_MEDIA).is_some(),
        "Digital paints no Paint Mode chip either — there would be no way to reach a medium at all"
    );

    for (media, id, name) in SECTIONS {
        let mut tool = PainterTool::default();
        tool.set_paint_media(media);
        let (_h, _s, rects) = painted(&tool);
        assert!(
            rect_of(&rects, id).is_some(),
            "{name} is selected and its own section is not painted"
        );
        for (other, other_id, other_name) in SECTIONS {
            if other != media {
                assert!(
                    rect_of(&rects, other_id).is_none(),
                    "{name} is selected and the {other_name} section is painted too — two media over \
                     one brush is two answers to 'what does this stroke do'"
                );
            }
        }
    }
}

/// **DEFECT REPRO (Enio, 2026-07-22): "ao entrar em Impasto e depois sair e selecionar Wet Paint,
/// widgets como o seletor de cor sumiram".**
///
/// They did, and the missing colour picker was the mild half. `Knife` and `Sculpt` exist only because
/// Impasto is on, and they SURVIVED it being switched off — so the artist picked Wet Paint and was left
/// holding the palette knife, with `paints_no_color()` true and the Colour and Blend rows hidden under a
/// panel that named a different medium at the top.
///
/// Measured before the fix (`via Knife` / `via Sculpt`): `colour=false blend=false`, and the medium the
/// tool reported was Wet Paint the whole time.
///
/// ⚠️ **Every destination is walked, and `Digital` is the one that matters.** Two independent rules can
/// rescue this — the *leave* rule (`set_brush_impasto` bringing you out of a mode that
/// `cannot_outlive` the medium) and the *enter* rule (step 3 of `set_paint_media`, "choosing a medium
/// USES it"). For Watercolor and Wet Paint the enter rule fires and hides whether the leave rule works
/// at all; **Digital deliberately has no enter rule** (it owns no mode — the rail tools are digital), so
/// it is the only destination that tests the leave rule alone. The first draft of this gate went to Wet
/// Paint only and stayed GREEN with the defect reinstalled.
///
/// **Mutation that must bleed:** drop the `cannot_outlive` branch from `set_brush_impasto` — the
/// `Digital` row goes red, the other two do not.
#[test]
fn leaving_impasto_with_one_of_its_tools_in_hand_does_not_strand_you_there() {
    for tool_wire in ["knife", "sculpt"] {
        for dest in [
            PaintMedia::Digital,
            PaintMedia::WetPaint,
            PaintMedia::Watercolor,
        ] {
            let mut tool = PainterTool::default();
            tool.set_paint_media(PaintMedia::Impasto);
            tool.set_paint_tool_mode(tool_wire); // pick one of the Impasto TOOL list's own modes
            tool.set_paint_media(dest); // …and now leave for another medium

            assert_eq!(
                tool.paint_media(),
                dest,
                "the medium did not switch to {dest:?} after holding the {tool_wire}"
            );
            let bs = tool.brush_settings();
            assert!(
                !bs.paints_no_color(),
                "after leaving Impasto for {dest:?} from the {tool_wire}, the brush still lays no \
                 pigment — the mode outlived its medium (is_smear={}, is_sculpt={})",
                bs.is_smear,
                bs.is_sculpt
            );

            let (_h, _s, rects) = painted(&tool);
            assert!(
                rect_of(&rects, core_ids::PAINTER_COLOR_THUMB).is_some(),
                "the colour picker vanished after Impasto → {tool_wire} → {dest:?} (Enio's report)"
            );
            // ⚠️ The Blend chip is checked everywhere EXCEPT Watercolor, where it is hidden on
            // purpose: the optical wash deposits source-over with its own Beer–Lambert optics and
            // never reads `BrushBlend`, so the chip would be a live-looking dead control (doc 13 #4).
            // That exemption is a different law, and folding it in here would have this gate quietly
            // assert the opposite of what that one does.
            if dest != PaintMedia::Watercolor {
                assert!(
                    rect_of(&rects, core_ids::PAINTER_BRUSH_BLEND).is_some(),
                    "the Blend chip vanished after Impasto → {tool_wire} → {dest:?} (Enio's report)"
                );
            }
        }
    }
}
