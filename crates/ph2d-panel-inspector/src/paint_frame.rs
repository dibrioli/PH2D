//! The chrome every live Inspector section is wrapped in, lifted out of
//! `paint.rs`.
//!
//! `live_section!` is a macro **defined inside** `paint_inspector` (it closes
//! over a dozen per-frame locals), so every line of its body counts toward
//! that function's length — and `paint_inspector` sits at a frozen LOC
//! allowance that may only shrink. Adding §11 Physics Body pushed it over, so
//! the macro now does the two things only a macro can (capture the locals,
//! run the caller's block) and delegates the rest here.
//!
//! This is the split the LOC gate itself prescribes: helpers that take the
//! per-frame mutables plus `y` and hand `y` back.

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::paint::stroke_rounded_rect;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{Radius, Spacing, StrokeToken};
use ph2d_vector::VectorScene;

use ph2d_editor_core::interaction::NoteData;
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::widget::INSPECTOR_SCROLLBAR_ID;
use ph2d_editor_core::widget::showcase::LAST_SECTION_TOPS_Y;
use ph2d_editor_core::widget::{self};
use ph2d_tokens::{ColorToken, TypeToken};

use crate::state::{set_last_inspector_content_h, set_last_inspector_visible_h};
use ph2d_editor_core::widget::panel_chrome::HIGHLIGHTER_RGBA;
use ph2d_editor_core::widget::showcase::{paint_one_note, push_section_top_y};

/// Record where a section starts and make its header clickable.
#[allow(clippy::too_many_arguments)]
pub(crate) fn begin_section(
    section_tops_y: &mut Vec<f32>,
    hit_index: &mut HitIndex,
    inner_x: f32,
    inner_w: f32,
    body_top_y: f32,
    y_before: f32,
    section_id: NodeId,
    header_h: f32,
) {
    push_section_top_y(section_tops_y, y_before - body_top_y);
    hit_index.register(section_id, Rect::new(inner_x, y_before, inner_w, header_h));
}

/// §11 Physics Body + §12 Physics Joint, frame and all.
///
/// Lifted out of `paint_inspector` for the same reason the section frame and
/// phase B were lifted before it: that orchestrator is under a ratcheting LOC
/// cap, and §12 pushed it past the line §11 had already ratcheted down to. The
/// two live here together because they are one family — the joint section's
/// creation gesture is a button in the body section — and because keeping them
/// adjacent is what makes their two note slots obviously distinct.
///
/// Returns the new `y`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_physics_sections(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    section_tops_y: &mut Vec<f32>,
    inner_x: f32,
    inner_w: f32,
    body_top_y: f32,
    mut y: f32,
    header_h: f32,
    physics: Option<&ph2d_editor_core::screens::hero::InspectorPhysicsInfo>,
    joint: Option<&ph2d_editor_core::screens::hero::InspectorJointInfo>,
    notes_body: &[(usize, NoteData)],
    notes_joint: &[(usize, NoteData)],
) -> f32 {
    // §11 Physics Body — offered for ANY Transform-bearing entity, with or
    // without a body: the empty state is the Add button, and without it a
    // sprite could never become physical (ADR-0131 D8).
    if let Some(phys) = physics {
        y = crate::paint::paint_section_separator_at(scene, theme, inner_x, inner_w, y);
        let y_before = y;
        begin_section(
            section_tops_y,
            hit_index,
            inner_x,
            inner_w,
            body_top_y,
            y_before,
            ids::INSP_LIVE_PHYSICS_SECTION,
            header_h,
        );
        let new_y = crate::sections::paint_physics_section(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            inner_x,
            inner_w,
            y,
            phys,
        );
        y = finish_section(
            scene,
            text_system,
            hit_index,
            store,
            inner_x,
            inner_w,
            ids::INSP_LIVE_PHYSICS_SECTION,
            y_before,
            new_y,
            notes_body,
        );
    }
    // §12 Physics Joint — only for an entity that IS a joint. Unlike §11 it has
    // no empty face: there is nothing to offer on an object that is not a
    // joint, and the gesture that creates one lives in §11, where the two
    // bodies you want to join are what you are looking at.
    if let Some(j) = joint {
        y = crate::paint::paint_section_separator_at(scene, theme, inner_x, inner_w, y);
        let y_before = y;
        begin_section(
            section_tops_y,
            hit_index,
            inner_x,
            inner_w,
            body_top_y,
            y_before,
            ids::INSP_LIVE_JOINT_SECTION,
            header_h,
        );
        let new_y = crate::sections::paint_joint_section(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            inner_x,
            inner_w,
            y,
            j,
        );
        y = finish_section(
            scene,
            text_system,
            hit_index,
            store,
            inner_x,
            inner_w,
            ids::INSP_LIVE_JOINT_SECTION,
            y_before,
            new_y,
            notes_joint,
        );
    }
    y
}

/// The chrome that wraps EVERY live section: the highlighter outline a user
/// can paint over a section, and the sticky notes anchored to it.
#[allow(clippy::too_many_arguments)]
pub(crate) fn finish_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    inner_x: f32,
    inner_w: f32,
    section_id: ph2d_a11y::NodeId,
    y_before: f32,
    new_y: f32,
    notes: &[(usize, NoteData)],
) -> f32 {
    let mut new_y = new_y;
    if let Some(color_idx) = store.section_outline_color(section_id) {
        let rgba = HIGHLIGHTER_RGBA[color_idx.min(4) as usize];
        let pad = Spacing::Xs.px();
        let block = Rect::new(
            inner_x - pad,
            y_before - pad,
            inner_w + pad * 2.0,
            (new_y - y_before + pad * 2.0).max(0.0),
        );
        let outline_color = ph2d_vector::Color::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]); // LITERAL-COLOR-OK: HIGHLIGHTER_RGBA palette
        stroke_rounded_rect(
            scene,
            block,
            Radius::Md.px(),
            StrokeToken::Thick.px(),
            outline_color,
        );
    }
    for (slot, note) in notes {
        paint_one_note(
            scene,
            text_system,
            hit_index,
            store,
            inner_x,
            inner_w,
            &mut new_y,
            note,
            *slot,
        );
    }
    new_y
}

/// Is there anything at all to show, or is the Inspector empty?
///
/// A named question rather than a disjunction buried mid-function: it decides
/// whether the panel paints its "nothing selected" placeholder, and every new
/// section has to be remembered here — a fact that is easier to keep true
/// when it has a name and a signature that changes when you forget.
#[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
pub(crate) fn any_live_section(flags: [bool; 9]) -> bool {
    flags.iter().any(|&b| b)
}

/// The per-frame numbers `publish_and_finish` needs, bundled — sixteen loose
/// parameters is a signature nobody can call correctly twice.
pub(crate) struct PanelFinish {
    pub any_section: bool,
    pub has_selection: bool,
    pub inner_x: f32,
    pub inner_w: f32,
    pub content_top: f32,
    pub content_bottom: f32,
    pub body_top_y: f32,
    pub y: f32,
    pub scroll_y: f32,
    pub rect: Rect,
}

/// Phase B of the Inspector paint: the empty-state placeholder, the scroll
/// bookkeeping the host reads next frame, and the scrollbar.
///
/// Lifted out of `paint_inspector` for the same reason as the section frame —
/// its LOC allowance is frozen and every new section costs ~18 lines. The
/// vector panel splits its own paint the same way (`seed_and_publish`).
pub(crate) fn publish_and_finish(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    f: PanelFinish,
    section_tops_y: Vec<f32>,
) {
    if !f.any_section {
        let placeholder = if f.has_selection {
            "No properties yet for the selected entity."
        } else {
            "Select an entity in the Hierarchy to inspect its properties."
        };
        let line_h = TypeToken::Sm.px() + Spacing::Xs.px();
        let center_y = f.content_top + (f.content_bottom - f.content_top) * 0.5 - line_h * 0.5;
        paint_text(
            text_system,
            scene,
            placeholder,
            f.inner_x + Spacing::Md.px(),
            center_y,
            TypeToken::Sm.px(),
            (f.inner_w - Spacing::Xl.px()).max(80.0), // LITERAL-PX-OK: minimum placeholder text width
            resolve(ColorToken::Text3, theme),
        );
    }

    let content_h = (f.y - f.body_top_y).max(0.0);
    let visible_h = (f.content_bottom - f.content_top).max(0.0);
    set_last_inspector_content_h(content_h);
    set_last_inspector_visible_h(visible_h);
    LAST_SECTION_TOPS_Y.with(|t| *t.borrow_mut() = section_tops_y);

    if widget::scrollbar_is_needed(content_h, visible_h) {
        let body = Rect::new(f.rect.x, f.content_top, f.rect.w, visible_h);
        let track = widget::scrollbar_track_rect(body);
        let thumb = widget::scrollbar_thumb_rect(track, f.scroll_y, content_h, visible_h);
        let is_active = matches!(store.scrollbar_drag(), Some(d) if d.panel == ids::INSP_PANEL);
        widget::paint_scrollbar(
            body, f.scroll_y, content_h, visible_h, is_active, scene, theme,
        );
        hit_index.register(INSPECTOR_SCROLLBAR_ID, thumb);
    }
}
