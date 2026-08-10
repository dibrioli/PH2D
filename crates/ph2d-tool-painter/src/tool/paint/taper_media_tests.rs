//! **The Taper survives the four painting media** — the half of Enio's ask (*"compatível com os 4 modos
//! de pintura"*) that no gate on the engine can see.
//!
//! Each paint mode carries its own [`ph2d_painter_brush::BrushSpec`] slot, so a value written only to
//! the live one is a value the artist loses the moment they pick another medium — and it comes BACK
//! when they return, which is worse than losing it, because the control then reads as haunted. That is
//! the exact shape of the defect the Paint Mode dropdown documented (*"ao entrar em Impasto e depois
//! sair e selecionar Wet Paint, widgets como o seletor de cor sumiram"*), and this is the gate that
//! keeps the taper out of it.
//!
//! ⚠️ The oracle is the LIVE brush after a media switch, never the slot array — reading the array would
//! be checking the fan-out against itself.

use super::PainterTool;
use crate::tool::PaintMedia;

/// **A taper authored in one medium is the same taper in all four.**
///
/// Written once, then read back through every medium in turn — and then again after a full round trip,
/// because a slot that was merely *stale* (rather than never written) shows up only when the artist
/// comes back to where they started.
///
/// **Mutation that must bleed:** drop the `for slot in &mut self.paint.brush_by_mode` loop from
/// `set_taper_field` — the value writes to the live brush, the artist changes medium, and it is gone.
#[test]
fn a_taper_authored_in_one_medium_is_the_taper_in_all_four() {
    let mut t = PainterTool::default();
    t.set_brush_taper_start(2.5);
    t.set_brush_taper_tip_start(0.4);
    t.set_brush_taper_opacity(0.75);

    let media = [
        PaintMedia::Digital,
        PaintMedia::Watercolor,
        PaintMedia::Impasto,
        PaintMedia::WetPaint,
        // Back to the start: a stale slot only shows itself on the return leg.
        PaintMedia::Digital,
    ];
    for m in media {
        t.set_paint_media(m);
        let k = t.brush_settings().taper;
        assert_eq!(
            (k.start, k.tip_start, k.opacity),
            (2.5, 0.4, 0.75),
            "the taper did not survive the switch to {m:?} — it is per-slot, so the artist loses it"
        );
    }
}

/// **Every taper knob is clamped at its own door, whatever a caller hands it.**
///
/// The panel clamps too, but a value sink reachable over the bus is reachable from anywhere, and a
/// length past the cap would be a lag past the cap ([`ph2d_painter_brush::taper`]). Negative and
/// enormous, both directions, one gate.
///
/// **Mutation that must bleed:** drop the `clamp` in `set_brush_taper_start`.
#[test]
fn every_taper_knob_clamps_at_its_own_door() {
    let mut t = PainterTool::default();
    t.set_brush_taper_start(-5.0);
    t.set_brush_taper_tip_start(-1.0);
    t.set_brush_taper_opacity(4.0);
    let k = t.brush_settings().taper;
    assert_eq!(k.start, 0.0, "a negative start length was accepted");
    assert_eq!(k.tip_start, 0.0, "a negative tip was accepted");
    assert_eq!(k.opacity, 1.0, "an opacity past 1 was accepted");

    // ...and the other end of each range, on a second tool so the clamps above cannot mask these.
    let mut t = PainterTool::default();
    t.set_brush_taper_start(1e6);
    t.set_brush_taper_tip_start(9.0);
    let k = t.brush_settings().taper;
    assert_eq!(
        k.start,
        ph2d_painter_brush::taper::MAX_TAPER_DIAMETERS,
        "a length past the cap was accepted"
    );
    assert_eq!(k.tip_start, 1.0, "a tip past 1 was accepted");
}
