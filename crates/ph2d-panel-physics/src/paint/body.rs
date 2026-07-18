//! The scrolled body: the collapsible section stack, then the debug rows.
//!
//! Sibling of `paint.rs` because the panel LOC cap is 600 and the two halves
//! grow for different reasons (chrome vs. content).

use ph2d_editor_core::ids;
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    Button, ButtonKind, ButtonState, SectionHeader, paint_button, paint_section_header,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, TypeToken};

use crate::rows;
use crate::state::PhysicsSnapshot;

/// Paint every section, then the debug rows. Returns the y it ended at.
pub(super) fn paint_sections(
    ctx: &mut PaintCtx,
    snapshot: &PhysicsSnapshot,
    x: f32,
    w: f32,
    y_in: f32,
) -> f32 {
    let mut y = y_in;
    let row_gap = Spacing::Sm.px();

    for section in rows::SECTIONS {
        let (open, next_y) = header(ctx, section.id, tr(section.title), x, w, y);
        y = next_y;
        if open {
            for row in section.rows {
                let value = (row.get)(&snapshot.settings);
                let used = super::paint_row(ctx, row, value, x, w, y);
                y += used + row_gap;
            }
        }
        y += Spacing::Md.px();
    }

    // Collision layers. Its own section because the matrix is a different KIND
    // of control from the sliders above — and because it is tall.
    let (open, next_y) = header(
        ctx,
        ids::PHYSICS_SEC_LAYERS,
        tr("panel.physics.section.layers"),
        x,
        w,
        y,
    );
    y = next_y;
    if open {
        y = super::matrix::paint(
            ctx,
            ph2d_physics_ecs::LayerMatrix::from_rows(snapshot.settings.layer_matrix),
            x,
            y,
        );
        y += Spacing::Md.px();
    }

    let (open, next_y) = header(
        ctx,
        ids::PHYSICS_SEC_DEBUG,
        tr("panel.physics.section.debug"),
        x,
        w,
        y,
    );
    y = next_y;
    if !open {
        return y;
    }

    // "Show Colliders" mirrors the shell's flag — the same one the `B` key
    // owns. The pressed state comes from the SNAPSHOT, never from a local
    // toggle, so the key and this control can never disagree.
    y = toggle(
        ctx,
        ids::PHYSICS_SHOW_COLLIDERS,
        tr("panel.physics.show_colliders"),
        snapshot.show_colliders,
        x,
        w,
        y,
    );
    y += row_gap;

    // Read-only facts, drawn as plain text and hit-indexed by nobody.
    //
    // ⚠️ The world scale is `ProjectSettings::pixels_per_meter`, a PROJECT
    // setting (ADR-0131 D4). It is shown so the metre-valued rows above can be
    // read in pixels — NOT so they can be edited here. A second door onto it
    // would diverge from the one in Project Settings.
    y = readout(
        ctx,
        &format!(
            "{}: {:.0} px/m",
            tr("panel.physics.scale"),
            snapshot.pixels_per_meter
        ),
        x,
        w,
        y,
    );
    // Zero is worth showing: it is the difference between "gravity is wrong"
    // and "nothing in this scene has a body yet".
    y = readout(
        ctx,
        &format!("{}: {}", tr("panel.physics.bodies"), snapshot.body_count),
        x,
        w,
        y,
    );
    y += Spacing::Md.px();

    // Reset sits at the BOTTOM, past everything it would undo: a destructive
    // command should not be on the path a hand takes to the first slider.
    y = command(
        ctx,
        ids::PHYSICS_RESET_DEFAULTS,
        tr("panel.physics.reset_defaults"),
        x,
        w,
        y,
    );
    y + row_gap
}

/// A collapsible section header. Returns `(is_open, y_after)`.
fn header(
    ctx: &mut PaintCtx,
    id: ph2d_a11y::NodeId,
    title: &str,
    x: f32,
    w: f32,
    y: f32,
) -> (bool, f32) {
    let theme = ctx.host.theme();
    let h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: section header band height
    let collapsed = ctx.host.store().is_collapsed(id);
    let rect = Rect::new(x, y, w, h);
    let header = SectionHeader::new(id, title).collapsible(!collapsed);
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (_, hit_index) = ctx.host.store_and_hit_index_mut();
    paint_section_header(&header, rect, scene, text_system, theme);
    hit_index.register(id, rect);
    (!collapsed, y + h + Spacing::Sm.px())
}

/// A Button used as a toggle.
///
/// **Not a `Checkbox`**: `Checkbox` emits `Toggled`, which this panel's
/// `event.rs` does not forward, so it would be registered and dead on click —
/// the warning `ph2d-panel-painter-layers` carries for the same reason.
fn toggle(
    ctx: &mut PaintCtx,
    id: ph2d_a11y::NodeId,
    label: &str,
    on: bool,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let theme = ctx.host.theme();
    let rect = Rect::new(x, y, w, ROW_H_PX);
    let state = if on {
        ButtonState::Pressed
    } else {
        ctx.host
            .store()
            .button_state(id)
            .unwrap_or(ButtonState::Normal)
    };
    let kind = if on {
        ButtonKind::Accent
    } else {
        ButtonKind::Default
    };
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (_, hit_index) = ctx.host.store_and_hit_index_mut();
    paint_button(
        &Button::new(id, label).kind(kind).state(state),
        rect,
        scene,
        text_system,
        theme,
    );
    hit_index.register(id, rect);
    y + ROW_H_PX
}

/// A plain action button.
fn command(ctx: &mut PaintCtx, id: ph2d_a11y::NodeId, label: &str, x: f32, w: f32, y: f32) -> f32 {
    let theme = ctx.host.theme();
    let rect = Rect::new(x, y, w, ROW_H_PX);
    let state = ctx
        .host
        .store()
        .button_state(id)
        .unwrap_or(ButtonState::Normal);
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (_, hit_index) = ctx.host.store_and_hit_index_mut();
    paint_button(
        &Button::new(id, label).state(state),
        rect,
        scene,
        text_system,
        theme,
    );
    hit_index.register(id, rect);
    y + ROW_H_PX
}

/// A line of text. Hit-indexed by nobody on purpose — it is a fact, not a
/// control, and an affordance it cannot honour would be worse than plain text.
fn readout(ctx: &mut PaintCtx, text: &str, x: f32, w: f32, y: f32) -> f32 {
    let theme = ctx.host.theme();
    let font = TypeToken::Sm.px();
    paint_text(
        ctx.text_system,
        ctx.scene,
        text,
        x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    y + ROW_H_PX
}
