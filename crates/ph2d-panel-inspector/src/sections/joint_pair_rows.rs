//! **O par de um joint** — as duas pontas, a troca entre elas, e a única coisa
//! que é fato sobre o PAR e não sobre a restrição (W-J8).
//!
//! Irmão de `joint.rs`, separado dele quando os dois juntos passaram do cap de
//! 600 LOC do painel, e o corte é o mesmo que a §12 desenha na tela: aqui *entre
//! QUAIS DOIS isto está, e como eles se tratam*; lá *o que a restrição FAZ*.

use super::joint::SWITCH_LABELS;
use super::rows::seg_row;
use super::*;
use ph2d_editor_core::screens::hero::InspectorJointInfo;

/// **The PAIR cluster** (W-J8): the two body rows, the swap, and Collide
/// Connected.
///
/// Grouped because they answer one question — *which two things is this between,
/// and how do they treat each other* — where everything below answers *what does
/// the constraint do*. The swap sits directly under the rows it exchanges, so the
/// button is next to the thing it acts on rather than in a row of verbs.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_pair_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorJointInfo,
) -> f32 {
    let mut yy = paint_body_rows(scene, text_system, theme, hit_index, store, x, w, y, info);
    let h = ROW_H_PX;
    let btn_rect = Rect::new(x, yy, w, h);
    // ⚠️ Always offered, on every kind and whether or not the names resolve: a
    // joint whose Body A was deleted is *exactly* when an artist wants to swap so
    // the surviving end becomes A. Gating it on `bound` would remove the button
    // from the case it is most useful in.
    let btn = Button::new(ids::INSP_JOINT_SWAP, "Swap A / B")
        .kind(ButtonKind::Default)
        .state(
            store
                .button_state(ids::INSP_JOINT_SWAP)
                .unwrap_or(ButtonState::Normal),
        );
    paint_button(&btn, btn_rect, scene, text_system, theme);
    hit_index.register(ids::INSP_JOINT_SWAP, btn_rect);
    yy += h;
    seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        "Collide",
        ids::INSP_JOINT_COLLIDE_GROUP,
        &ids::INSP_JOINT_COLLIDE,
        &SWITCH_LABELS,
        u8::from(info.collide_connected),
    )
}

/// The two per-body rows: the label ("Body A"/"Body B"), the CURRENT body's
/// name, and an eyedropper to re-pick it. Its own fn for the 200-LOC panel-fn
/// cap. Shown for any selected joint — no other object needs selecting (the
/// redesign's whole point). Clicking the eyedropper ARMS a canvas pick; the next
/// click on a body re-binds that end. A body whose name no longer resolves shows
/// "(missing)" dimmed — the per-end replacement for the old combined "not
/// connected" line, which said it once for both ends and could not point at
/// WHICH end broke.
#[allow(clippy::too_many_arguments)]
fn paint_body_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorJointInfo,
) -> f32 {
    let mut yy = y;
    let h = ROW_H_PX;
    let label_font = TypeToken::Sm.px();
    let icon_w = (h * 0.82).min(w); // LITERAL-PX-OK: icon inset ratio (compact square in the row)
    // "Body A" / "Body B" column, wide enough for the label at this font.
    let label_w = (label_font * 3.6).min(w * 0.4); // LITERAL-PX-OK: label = 3.6 char-heights, capped at 0.4 of the row
    let gap = Spacing::Sm.px();
    for (slot_label, name, id, armed) in [
        (
            "Body A",
            &info.body_a_name,
            ids::INSP_JOINT_PICK_A,
            info.pick_armed == 1,
        ),
        (
            "Body B",
            &info.body_b_name,
            ids::INSP_JOINT_PICK_B,
            info.pick_armed == 2,
        ),
    ] {
        let text_y = yy + (h - label_font) * 0.5;
        paint_text(
            text_system,
            scene,
            slot_label,
            x,
            text_y,
            label_font,
            label_w,
            resolve(ColorToken::Text2, theme),
        );
        let shown = display_name(name);
        let name_x = x + label_w;
        let name_w = (w - label_w - icon_w - gap).max(0.0);
        paint_text(
            text_system,
            scene,
            shown,
            name_x,
            text_y,
            label_font,
            name_w,
            resolve(
                if name.is_empty() {
                    ColorToken::Text3
                } else {
                    ColorToken::Text1
                },
                theme,
            ),
        );
        // The eyedropper, right-aligned. Pressed (accent) while its pick is
        // ARMED, so the artist sees which end is waiting for a body click.
        let brect = Rect::new(x + w - icon_w, yy + (h - icon_w) * 0.5, icon_w, icon_w);
        let state = if armed {
            ButtonState::Pressed
        } else {
            store.button_state(id).unwrap_or(ButtonState::Normal)
        };
        paint_icon_button(
            brect,
            IconGlyph::Builtin(IconId::Eyedropper),
            IconButtonStyle::Compact,
            state,
            scene,
            theme,
        );
        hit_index.register(id, brect);
        yy += h;
    }
    yy
}

/// A body name for display, with a stand-in when it could not be resolved.
/// The joint stores a hash, and a hash is not something to show a person —
/// but neither is an empty gap where a name should be.
fn display_name(name: &str) -> &str {
    if name.is_empty() { "(missing)" } else { name }
}
