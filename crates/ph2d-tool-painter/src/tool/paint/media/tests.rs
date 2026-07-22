//! The medium switch's own laws — exclusivity, the modes a medium owns, and the two rules that make
//! leaving one safe. The panel-side proof (a pointer picking each medium, one section on screen) lives
//! in `ph2d-panel-painter-layers/tests/seam_paint_media.rs`; this half is the model.

use super::PaintMedia;
use crate::tool::PainterTool;
use crate::tool::paint::PaintMode;

const ALL: [PaintMedia; 4] = [
    PaintMedia::Digital,
    PaintMedia::Watercolor,
    PaintMedia::Impasto,
    PaintMedia::WetPaint,
];

/// **One medium at a time, from any starting point.**
///
/// The three flags used to be three independent checkboxes, so "Watercolor AND Impasto" was a state the
/// artist could reach by hand and nothing downstream had an answer for. Every ordered pair is walked
/// here, because a switch that is only exclusive from the default is not exclusive.
///
/// **Mutation that must bleed:** drop any one of the three `if media != …` leave-steps from
/// `set_paint_media`.
#[test]
fn only_one_medium_is_ever_on() {
    for from in ALL {
        for to in ALL {
            let mut t = PainterTool::default();
            t.set_paint_media(from);
            t.set_paint_media(to);
            assert_eq!(t.paint_media(), to, "{from:?} → {to:?} did not take");
            let b = t.brush_settings();
            let on = [b.watercolor, b.impasto, b.wetpaint];
            assert_eq!(
                on.iter().filter(|f| **f).count(),
                usize::from(to != PaintMedia::Digital),
                "{from:?} → {to:?} left {on:?} — the media are exclusive, and Digital is the absence \
                 of all three"
            );
        }
    }
}

/// **`Digital` does not yank you out of the tool you are holding.**
///
/// It owns no mode on purpose: the rail's Smear / Blur / Clone ARE digital, so picking the medium that
/// means "no medium" must leave them alone. The three real media do the opposite — choosing one USES
/// it — and that asymmetry is the whole content of step 3 in `set_paint_media`.
///
/// **Mutation that must bleed:** drop the `media != PaintMedia::Digital` guard on step 3 (the Smear is
/// swapped for the brush by a control that promised nothing of the kind).
#[test]
fn picking_digital_leaves_a_rail_tool_where_it_is() {
    let mut t = PainterTool::default();
    t.set_paint_tool_mode("smear");
    t.set_paint_media(PaintMedia::Digital);
    assert_eq!(
        t.paint.paint_mode,
        PaintMode::Smear,
        "picking Digital took the artist out of the Smear — the rail tools are digital already"
    );
    // …and the opposite half: picking a real medium from the Smear DOES take the brush, or the
    // dropdown would name a medium whose section is not even painted.
    t.set_paint_media(PaintMedia::Impasto);
    assert_eq!(
        t.paint.paint_mode,
        PaintMode::Paint,
        "picking Impasto from the Smear left a medium selected that cannot act in the mode in hand"
    );
}

/// **A mode that a medium owns cannot outlive it** — the model half of Enio's 2026-07-22 report.
///
/// Driven through `set_brush_impasto` DIRECTLY rather than through `set_paint_media`, and that is the
/// point: the dropdown's step 3 would rescue this for every destination except Digital, so a gate that
/// only ever went through the dropdown would pass with the rule deleted (it did — the first draft of
/// the panel-side gate).
///
/// **Mutation that must bleed:** drop the `cannot_outlive` branch from `set_brush_impasto`.
#[test]
fn switching_impasto_off_brings_you_out_of_its_own_tools() {
    for (wire, mode) in [("knife", PaintMode::Knife), ("sculpt", PaintMode::Sculpt)] {
        let mut t = PainterTool::default();
        t.set_brush_impasto(true);
        t.set_paint_tool_mode(wire);
        assert_eq!(t.paint.paint_mode, mode, "fixture: the {wire} is in hand");
        t.set_brush_impasto(false);
        assert_eq!(
            t.paint.paint_mode,
            PaintMode::Paint,
            "the {wire} outlived the Impasto — it exists only because the medium was on"
        );
    }
}

/// **`Paint` is deliberately NOT orphaned.** The plain brush uses it too, so the wash and the body only
/// ever *reinterpret* it; yanking the artist to the mode they are already in would be a jolt for
/// nothing, and — worse — would end a live stroke session on a switch that need not touch it.
///
/// **Mutation that must bleed:** define `cannot_outlive` as `works_in` (dropping the `!= Paint`).
#[test]
fn the_plain_paint_mode_is_not_owned_by_any_medium() {
    for m in ALL {
        assert!(
            !m.cannot_outlive(PaintMode::Paint),
            "{m:?} claims the plain Paint mode as its own — every medium would evict the brush"
        );
    }
    // And the two that ARE owned, so this gate cannot pass by `cannot_outlive` being false everywhere.
    assert!(PaintMedia::Impasto.cannot_outlive(PaintMode::Knife));
    assert!(PaintMedia::WetPaint.cannot_outlive(PaintMode::WetPaint));
}

/// **The Preset dropdown is not a second door to the medium.**
///
/// A preset writes `watercolor` straight into the `BrushSpec`, one row above the Paint Mode chip, so it
/// could switch medium without the exclusivity rule ever running. Measured before the fix: Wet Paint
/// armed + "Watercolor Basic" left `watercolor = true` **and** `wetpaint = true` — the chip read *Wet
/// Paint* over a watercolor brush; and from Impasto it cleared the live slot while `brush_by_mode[Knife]`
/// kept `impasto = true`, ready to come back on the next tool switch.
///
/// **Mutation that must bleed:** drop the `set_paint_media` tail from `apply_brush_preset`.
#[test]
fn a_brush_preset_switches_medium_through_the_same_door() {
    for from in ALL {
        let mut t = PainterTool::default();
        t.set_paint_media(from);
        t.apply_brush_preset(1); // "Watercolor Basic"
        assert_eq!(
            t.paint_media(),
            PaintMedia::Watercolor,
            "the Watercolor preset left the medium at {:?} (from {from:?})",
            t.paint_media()
        );
        let b = t.brush_settings();
        assert!(
            b.watercolor && !b.impasto && !b.wetpaint,
            "the Watercolor preset from {from:?} left two media on: watercolor={} impasto={} \
             wetpaint={}",
            b.watercolor,
            b.impasto,
            b.wetpaint
        );
        // …and the per-mode slots agree, or the flag comes back on the next tool switch.
        t.set_paint_tool_mode("knife");
        assert!(
            !t.brush_settings().impasto,
            "the Knife slot resurrected Impasto after the preset (from {from:?}) — the preset wrote \
             the live slot only"
        );

        // Digital Basic is the same law in the other direction.
        let mut t = PainterTool::default();
        t.set_paint_media(from);
        t.apply_brush_preset(0);
        assert_eq!(
            t.paint_media(),
            PaintMedia::Digital,
            "the Digital preset from {from:?} left a medium armed"
        );
    }
}

/// The wire `u8` is the panel seam's vocabulary: it is what the dropdown option carries and what
/// `SelectOption` parses, so a reordering is a silently-wrong medium, not a compile error.
#[test]
fn the_wire_values_round_trip_and_are_pinned() {
    for (v, m) in [
        (0u8, PaintMedia::Digital),
        (1, PaintMedia::Watercolor),
        (2, PaintMedia::Impasto),
        (3, PaintMedia::WetPaint),
    ] {
        assert_eq!(PaintMedia::from_u8(v), m, "wire {v} decoded wrong");
        assert_eq!(m.to_u8(), v, "{m:?} encoded wrong");
    }
    assert_eq!(
        PaintMedia::from_u8(200),
        PaintMedia::Digital,
        "an unknown wire must fall back to Digital — the medium that is the absence of the others"
    );
    assert_eq!(PaintMedia::COUNT, 4, "the dropdown paints COUNT options");
}
