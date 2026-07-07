//! Audio Mixer master-section footer painter — Play Test · loudness (LUFS) ·
//! Limiter, then the collapsible master-effect groups (EQ · Reverb · Delay ·
//! Comp · Ducking). Split out of `paint.rs` to keep it under the panel LOC cap.
//!
//! Each effect group is a canonical collapsible [`SectionHeader`]: its id is
//! `mark_collapsible_section`-registered in `populate`, so the dispatch folds it
//! on click (no `apply_event` arm needed); the body paints only when open.

use crate::paint::MUTE_H;
use crate::paint_widgets::{paint_labeled_slider, paint_toggle};
use crate::{
    AMIX_DELAY, AMIX_DELAY_FEEDBACK, AMIX_DELAY_MIX, AMIX_DELAY_TIME, AMIX_DUCK, AMIX_DUCK_DEPTH,
    AMIX_DUCK_KEY, AMIX_EQ_HIGH, AMIX_EQ_LOW, AMIX_EQ_MID, AMIX_LIMITER, AMIX_PLAY, AMIX_REVERB,
    AMIX_REVERB_MIX, AMIX_REVERB_SIZE, AMIX_SEC_COMP, AMIX_SEC_DELAY, AMIX_SEC_DUCK, AMIX_SEC_EQ,
    AMIX_SEC_REVERB, SUB_BUS_COUNT, SUB_BUS_LABELS, SUB_COMP, SUB_DELAY_SEND, SUB_SEND, snapshot,
};
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::paint::{paint_text_centered, resolve};
use ph2d_editor_core::widget::{SectionHeader, paint_section_header};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

const LUFS_SILENCE_DISPLAY: f32 = -70.0; // LITERAL-PX-OK: below this the loudness reads "-inf" (audio domain)

/// The shared paint context threaded through the footer section painters —
/// bundles the borrows so each section fn takes just `(&mut Ctx, y)`.
struct Ctx<'a> {
    scene: &'a mut VectorScene,
    text_system: &'a mut TextSystem,
    hit_index: &'a mut HitIndex,
    store: &'a WidgetStore,
    theme: Theme,
    /// Content left edge + width (the panel's padded inner column).
    x: f32,
    w: f32,
}

/// The master-section footer below the strips, top-down: Play Test · loudness ·
/// Limiter · EQ · Reverb · Delay · Comp · Ducking. Returns the final `y` (the
/// bottom of the painted content) so the caller can size the scroll region.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_master_section(
    y0: f32,
    content_x: f32,
    content_w: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) -> f32 {
    let mut ctx = Ctx {
        scene,
        text_system,
        hit_index,
        store,
        theme,
        x: content_x,
        w: content_w,
    };
    let mut y = y0;
    y = paint_play_test(&mut ctx, y);
    y = paint_loudness(&mut ctx, y);
    y = paint_limiter(&mut ctx, y);
    y = paint_eq(&mut ctx, y);
    y = paint_reverb(&mut ctx, y);
    y = paint_delay(&mut ctx, y);
    y = paint_comp(&mut ctx, y);
    paint_ducking(&mut ctx, y)
}

/// A full-width toggle row; returns the next `y`.
fn toggle_row(ctx: &mut Ctx, y: f32, label: &str, active: bool, id: NodeId) -> f32 {
    paint_toggle(
        Rect::new(ctx.x, y, ctx.w, MUTE_H),
        label,
        active,
        ColorToken::Accent,
        id,
        ctx.scene,
        ctx.text_system,
        ctx.theme,
        ctx.hit_index,
    );
    y + MUTE_H + Spacing::Sm.px()
}

/// A labeled thin-slider row; returns the next `y`.
fn slider_row(ctx: &mut Ctx, y: f32, label: &str, id: NodeId, value: f32) -> f32 {
    paint_labeled_slider(
        y,
        label,
        id,
        value,
        ctx.x,
        ctx.w,
        ctx.scene,
        ctx.text_system,
        ctx.theme,
        ctx.hit_index,
    )
}

/// Paint a collapsible section header (chevron + uppercase label). Returns
/// `(open, next_y)`; the dispatch flips `is_collapsed` on click.
fn section_header(ctx: &mut Ctx, y: f32, id: NodeId, label: &str) -> (bool, f32) {
    let open = !ctx.store.is_collapsed(id);
    let rect = Rect::new(ctx.x, y, ctx.w, MUTE_H);
    let header = SectionHeader::new(id, label).collapsible(open);
    paint_section_header(&header, rect, ctx.scene, ctx.text_system, ctx.theme);
    ctx.hit_index.register(id, rect);
    (open, y + MUTE_H + Spacing::Sm.px())
}

/// Per-sub-bus labeled rows (used by the Reverb/Delay sends + Comp knobs).
fn sub_bus_rows(
    ctx: &mut Ctx,
    mut y: f32,
    ids: &[NodeId; SUB_BUS_COUNT],
    vals: [f32; SUB_BUS_COUNT],
) -> f32 {
    for i in 0..SUB_BUS_COUNT {
        y = slider_row(ctx, y, SUB_BUS_LABELS[i], ids[i], vals[i]);
    }
    y
}

fn paint_play_test(ctx: &mut Ctx, y: f32) -> f32 {
    // Play Test first — the primary "make sound" control stays reachable.
    let playing = snapshot::play_test();
    toggle_row(
        ctx,
        y,
        if playing { "Stop" } else { "Play Test" },
        playing,
        AMIX_PLAY,
    )
}

fn paint_loudness(ctx: &mut Ctx, y: f32) -> f32 {
    // Momentary loudness (LUFS, BS.1770); "-inf" when effectively silent.
    let lufs = snapshot::loudness();
    let text = if lufs <= LUFS_SILENCE_DISPLAY {
        "-inf LUFS".to_string()
    } else {
        format!("{lufs:.1} LUFS")
    };
    paint_text_centered(
        ctx.text_system,
        ctx.scene,
        &text,
        Rect::new(ctx.x, y, ctx.w, TypeToken::Xs.px()),
        TypeToken::Xs.px(),
        resolve(ColorToken::Text2, ctx.theme),
    );
    y + TypeToken::Xs.px() + Spacing::Md.px()
}

fn paint_limiter(ctx: &mut Ctx, y: f32) -> f32 {
    // Master output limiter — tames peaks below the clip ceiling.
    toggle_row(ctx, y, "Limiter", snapshot::limiter(), AMIX_LIMITER) + Spacing::Sm.px()
}

fn paint_eq(ctx: &mut Ctx, y: f32) -> f32 {
    let (open, mut y) = section_header(ctx, y, AMIX_SEC_EQ, "EQ");
    if open {
        let eq = snapshot::eq();
        y = slider_row(ctx, y, "Low", AMIX_EQ_LOW, eq[0]);
        y = slider_row(ctx, y, "Mid", AMIX_EQ_MID, eq[1]);
        y = slider_row(ctx, y, "High", AMIX_EQ_HIGH, eq[2]);
    }
    y + Spacing::Sm.px()
}

fn paint_reverb(ctx: &mut Ctx, y: f32) -> f32 {
    let (open, mut y) = section_header(ctx, y, AMIX_SEC_REVERB, "Reverb");
    if open {
        y = toggle_row(ctx, y, "Reverb", snapshot::reverb_on(), AMIX_REVERB);
        y = slider_row(ctx, y, "Size", AMIX_REVERB_SIZE, snapshot::reverb_size());
        y = slider_row(ctx, y, "Return", AMIX_REVERB_MIX, snapshot::reverb_mix());
        y = sub_bus_rows(ctx, y, &SUB_SEND, snapshot::sub_send());
    }
    y + Spacing::Sm.px()
}

fn paint_delay(ctx: &mut Ctx, y: f32) -> f32 {
    let (open, mut y) = section_header(ctx, y, AMIX_SEC_DELAY, "Delay");
    if open {
        y = toggle_row(ctx, y, "Delay", snapshot::delay_on(), AMIX_DELAY);
        y = slider_row(ctx, y, "Time", AMIX_DELAY_TIME, snapshot::delay_time());
        y = slider_row(
            ctx,
            y,
            "Fbk",
            AMIX_DELAY_FEEDBACK,
            snapshot::delay_feedback(),
        );
        y = slider_row(ctx, y, "Return", AMIX_DELAY_MIX, snapshot::delay_mix());
        y = sub_bus_rows(ctx, y, &SUB_DELAY_SEND, snapshot::sub_delay_send());
    }
    y + Spacing::Sm.px()
}

fn paint_comp(ctx: &mut Ctx, y: f32) -> f32 {
    let (open, mut y) = section_header(ctx, y, AMIX_SEC_COMP, "Comp");
    if open {
        y = sub_bus_rows(ctx, y, &SUB_COMP, snapshot::sub_comp());
    }
    y + Spacing::Sm.px()
}

fn paint_ducking(ctx: &mut Ctx, y: f32) -> f32 {
    let (open, mut y) = section_header(ctx, y, AMIX_SEC_DUCK, "Ducking");
    if open {
        y = toggle_row(ctx, y, "Ducking", snapshot::ducking(), AMIX_DUCK);
        // Key selector (a plain button — cycles the sidechain key sub-bus).
        let key_label = format!(
            "Key: {}",
            SUB_BUS_LABELS[snapshot::ducking_key() % SUB_BUS_COUNT]
        );
        paint_toggle(
            Rect::new(ctx.x, y, ctx.w, MUTE_H),
            &key_label,
            false,
            ColorToken::Accent,
            AMIX_DUCK_KEY,
            ctx.scene,
            ctx.text_system,
            ctx.theme,
            ctx.hit_index,
        );
        y += MUTE_H + Spacing::Sm.px();
        y = slider_row(ctx, y, "Depth", AMIX_DUCK_DEPTH, snapshot::duck_depth());
    }
    y + Spacing::Sm.px()
}
