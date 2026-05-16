//! `paint_hierarchy_row` — per-row painter for the hierarchy panel.
//! Extracted from `hierarchy/mod.rs` in Wave 2 PR 11.7b. Called by
//! `super::panel_painter::paint_hierarchy` for each row in display
//! order. Reads thread-local rename-target state via `super`.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_hierarchy_row(
    entity: &fixture::HierarchyEntity,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    row_id: Option<ph2d_a11y::NodeId>,
    mut hit_index: Option<&mut HitIndex>,
    has_children: bool,
    is_collapsed: bool,
    direct_match: bool,
) {
    if entity.selected {
        fill_rounded_rect(
            scene,
            rect,
            Radius::Sm.px(),
            resolve(ColorToken::AccentSoft, theme),
        );
        stroke_rounded_rect(
            scene,
            rect,
            Radius::Sm.px(),
            1.0,
            resolve(ColorToken::Accent, theme),
        );
    }
    let indent_w = 16.0 * entity.indent as f32;
    let pad = 10.0_f32;
    // M14.6C: chevron column. Always reserves 16 px so child rows
    // align with their parents' entity-icon column. Painted only when
    // `has_children` (otherwise the slot is empty whitespace).
    let chev_w = 12.0_f32;
    let chev_pad = 4.0_f32;
    let chev_x = rect.x + pad + indent_w;
    if has_children {
        let chev_rect = Rect::new(chev_x, rect.y + (rect.h - chev_w) * 0.5, chev_w, chev_w);
        let chev_icon = if is_collapsed {
            IconId::ChevronRight
        } else {
            IconId::ChevronDown
        };
        paint_icon(
            scene,
            chev_icon,
            chev_rect,
            resolve(ColorToken::Text2, theme),
            1.5,
        );
        if let (Some(row_id), Some(idx)) = (row_id, hit_index.as_mut()) {
            // Hit-rect: chevron glyph + padding for 24×24 click target.
            let hit_rect = Rect::new(
                chev_rect.x - 6.0,
                chev_rect.y - 6.0,
                chev_w + 12.0,
                chev_w + 12.0,
            );
            idx.register(
                crate::screens::hero::ids::hier_expand_companion(row_id),
                hit_rect,
            );
        }
    }
    let icon_w = 16.0_f32;
    let icon_x = chev_x + chev_w + chev_pad;
    let icon_rect = Rect::new(icon_x, rect.y + (rect.h - icon_w) * 0.5, icon_w, icon_w);
    let icon_color = if entity.selected {
        ColorToken::Accent
    } else if entity.muted {
        ColorToken::TextDisabled
    } else {
        ColorToken::Text3
    };
    paint_icon(
        scene,
        entity.icon,
        icon_rect,
        resolve(icon_color, theme),
        1.5,
    );

    let mut right_x = rect.x + rect.w - pad;
    // M14.6A: clickable eye icon replaces the legacy visibility dot.
    // Open eye = visible (Text2), closed eye = hidden (TextDisabled).
    let eye_icon = if entity.visible {
        IconId::Eye
    } else {
        IconId::EyeClosed
    };
    let eye_color = if entity.visible {
        ColorToken::Text2
    } else {
        ColorToken::TextDisabled
    };
    let eye_size = 16.0_f32;
    let eye_rect = Rect::new(
        right_x - eye_size,
        rect.y + (rect.h - eye_size) * 0.5,
        eye_size,
        eye_size,
    );
    paint_icon(scene, eye_icon, eye_rect, resolve(eye_color, theme), 1.5);
    if let (Some(row_id), Some(idx)) = (row_id, hit_index.as_mut()) {
        let hit_pad = 4.0_f32;
        let hit_rect = Rect::new(
            eye_rect.x - hit_pad,
            eye_rect.y - hit_pad,
            eye_rect.w + hit_pad * 2.0,
            eye_rect.h + hit_pad * 2.0,
        );
        idx.register(
            crate::screens::hero::ids::hier_eye_companion(row_id),
            hit_rect,
        );
    }
    right_x -= eye_size + 6.0;
    if let Some(swatch) = entity.swatch {
        let sw = 14.0_f32;
        let sw_rect = Rect::new(right_x - sw, rect.y + (rect.h - sw) * 0.5, sw, sw);
        let [r, g, b, a] = swatch;
        fill_rounded_rect(scene, sw_rect, 4.0, VelloColor::from_rgba8(r, g, b, a));
        stroke_rounded_rect(scene, sw_rect, 4.0, 1.0, resolve(ColorToken::Border, theme));
        right_x -= sw + 6.0;
    }
    if let Some(badge) = &entity.badge {
        // Phase 2 polish: render the kind badge as a `Tag` widget
        // tinted by kind (PRF=Accent, UNI=Neutral, CAM=Success,
        // OUT=Warn, etc). Functional in the sense that it shares
        // identity with the rest of the editor's chrome.
        let badge_w = 36.0_f32;
        let badge_h = 18.0_f32;
        let badge_rect = Rect::new(
            right_x - badge_w,
            rect.y + (rect.h - badge_h) * 0.5,
            badge_w,
            badge_h,
        );
        let tone = match badge.as_str() {
            "PRF" => TagTone::Accent,
            "UNI" => TagTone::Neutral,
            "OUT" => TagTone::Warn,
            "CAM" => TagTone::Success,
            "TIL" => TagTone::Neutral,
            "TRG" => TagTone::Danger,
            "LGT" => TagTone::Warn,
            "SPR" => TagTone::Accent,
            _ => TagTone::Neutral,
        };
        let tag = Tag::new(ph2d_a11y::NodeId(0), badge)
            .tone(tone)
            .state(if entity.muted {
                TagState::Disabled
            } else {
                TagState::Normal
            });
        paint_tag(&tag, badge_rect, scene, text_system, theme);
        right_x -= badge_w + 6.0;
    }

    let name_x = icon_rect.x + icon_w + 8.0;
    let name_color = if entity.muted {
        ColorToken::TextDisabled
    } else if direct_match {
        // M14.6 E: rows whose name literally matched the search query
        // get the Accent color so the eye locks onto hits even when
        // ancestors are painted alongside them for context.
        ColorToken::Accent
    } else {
        ColorToken::Text1
    };
    paint_text(
        text_system,
        scene,
        &entity.name,
        name_x,
        rect.y + (rect.h - TypeToken::Sm.px()) * 0.5,
        TypeToken::Sm.px(),
        (right_x - name_x).max(0.0),
        resolve(name_color, theme),
    );
}
