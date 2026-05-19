//! `paint_hierarchy_row` — per-row painter for the hierarchy panel.
//! Ported from `ph2d_editor_core::screens::hero::hierarchy::row_painter`
//! in Phase C.2; logic unchanged.

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::HitIndex;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_icon, paint_text, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::screens::hero::fixture;
use ph2d_editor_core::screens::hero::ids;
use ph2d_editor_core::widget::{Tag, TagState, TagTone, paint_tag};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{
    ColorToken, ICON_BTN_SIZE_PX, Radius, SECTION_GAP_PX, Spacing, StrokeToken, Theme, TypeToken,
};
use ph2d_vector::{Color as VelloColor, VectorScene};

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_hierarchy_row(
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
    let indent_w = Spacing::Xl.px() * entity.indent as f32;
    let pad = 10.0_f32; // LITERAL-PX-OK: row inset between Spacing::Md(8) and Lg(12); chrome-specific dim
    let chev_w = Spacing::Lg.px();
    let chev_pad = Spacing::Xs.px();
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
            StrokeToken::Default.px(),
        );
        if let (Some(row_id), Some(idx)) = (row_id, hit_index.as_mut()) {
            let hit_rect = Rect::new(
                chev_rect.x - Spacing::Sm.px(),
                chev_rect.y - Spacing::Sm.px(),
                chev_w + Spacing::Lg.px(),
                chev_w + Spacing::Lg.px(),
            );
            idx.register(ids::hier_expand_companion(row_id), hit_rect);
        }
    }
    let icon_w = Spacing::Xl.px();
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
        StrokeToken::Default.px(),
    );

    let mut right_x = rect.x + rect.w - pad;
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
    let eye_size = Spacing::Xl.px();
    let eye_rect = Rect::new(
        right_x - eye_size,
        rect.y + (rect.h - eye_size) * 0.5,
        eye_size,
        eye_size,
    );
    paint_icon(
        scene,
        eye_icon,
        eye_rect,
        resolve(eye_color, theme),
        StrokeToken::Default.px(),
    );
    if let (Some(row_id), Some(idx)) = (row_id, hit_index.as_mut()) {
        let hit_pad = Spacing::Xs.px();
        let hit_rect = Rect::new(
            eye_rect.x - hit_pad,
            eye_rect.y - hit_pad,
            eye_rect.w + hit_pad * 2.0,
            eye_rect.h + hit_pad * 2.0,
        );
        idx.register(ids::hier_eye_companion(row_id), hit_rect);
    }
    right_x -= eye_size + Spacing::Sm.px();
    if let Some(swatch) = entity.swatch {
        let sw = SECTION_GAP_PX;
        let sw_rect = Rect::new(right_x - sw, rect.y + (rect.h - sw) * 0.5, sw, sw);
        let [r, g, b, a] = swatch;
        fill_rounded_rect(
            scene,
            sw_rect,
            Radius::Xs.px(),
            VelloColor::from_rgba8(r, g, b, a), // LITERAL-COLOR-OK: user-color (per-entity accent, not a theme token)
        );
        stroke_rounded_rect(
            scene,
            sw_rect,
            Radius::Xs.px(),
            1.0,
            resolve(ColorToken::Border, theme),
        );
        right_x -= sw + Spacing::Sm.px();
    }
    if let Some(badge) = &entity.badge {
        let badge_w = ICON_BTN_SIZE_PX;
        let badge_h = TypeToken::Lg.px();
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
        right_x -= badge_w + Spacing::Sm.px();
    }

    let name_x = icon_rect.x + icon_w + Spacing::Md.px();
    let name_color = if entity.muted {
        ColorToken::TextDisabled
    } else if direct_match {
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
