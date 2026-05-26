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
use ph2d_vector::VectorScene;

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
    // Indent + body padding are ALREADY applied by the caller
    // (`paint.rs::244 — row_rect.x = rect.x + body_pad + indent`).
    // Duplicating either here doubled the visual indent and pushed
    // children much further right than Godot's Scene panel pattern
    // — user 2026-05-24: "atualmente o ícone e nome do filho fica
    // muito mais deslocado para direita que o pai, mas deve ser
    // como na godot: deslocar o filho para direita apenas a largura
    // do ícone." Internal `pad` reduced 10 → 2 so names sit close
    // to the panel's left edge.
    let pad = 8.0_f32; // LITERAL-PX-OK: row left-inset (Enio 2026-05-26: deslocar ícones esquerda + nome mais pra direita; antes era 2 px Godot-tight)
    let chev_w = Spacing::Lg.px();
    // Chev → icon gap tightened Xs (4) → Xxs (2) 2026-05-24 per user:
    // "quero os ícones mais próximos das setas". Godot's Scene panel
    // packs them flush together; we keep a 2-px hairline so click
    // targets don't visually merge.
    let chev_pad = Spacing::Xxs.px();
    let chev_x = rect.x + pad;
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
    // Hit-register the entity icon as its own companion (2026-05-26):
    // double-click on the icon focuses the view; double-click on the
    // name body (row body) triggers rename. Hit pad Md (8 px) — alvo
    // mais generoso (Enio: "expanda um pouco a área sensível").
    if let (Some(row_id), Some(idx)) = (row_id, hit_index.as_mut()) {
        let hit_pad = Spacing::Md.px();
        let hit_rect = Rect::new(
            icon_rect.x - hit_pad,
            icon_rect.y - hit_pad,
            icon_rect.w + hit_pad * 2.0,
            icon_rect.h + hit_pad * 2.0,
        );
        idx.register(ids::hier_icon_companion(row_id), hit_rect);
    }

    // Right-side icon cluster — eye colada na borda direita (pad 0)
    // e gap inter-icon Xxs (2 px) pra ficarem "bem juntos" (Enio
    // 2026-05-26). Antes: pad=2 + gap=Sm (6).
    let icon_cluster_pad = Spacing::Sm.px(); // 8 px da borda direita (Enio 2026-05-26 round 2: "separe os olhos mais 4 pixels da lateral direita (desloque 4 pixels para esquerda)" — antes Xs=4)
    let icon_cluster_gap = 0.0_f32; // LITERAL-PX-OK: Enio 2026-05-26 "reduza a distância entre os ícones"
    let mut right_x = rect.x + rect.w - icon_cluster_pad;
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
    right_x -= eye_size + icon_cluster_gap;
    // ── Group lock (folder icon) — pinta SE locked, sempre clicável.
    // À esquerda do olho. Click toggla `GroupedChildren` em SimWorld
    // via EditorAction::HierToggleGroup (handler no shell). Enio
    // 2026-05-26: "Agrupar: vc pode manipular o pai mas não os filhos".
    let icon_btn = eye_size;
    let group_rect = Rect::new(
        right_x - icon_btn,
        rect.y + (rect.h - icon_btn) * 0.5,
        icon_btn,
        icon_btn,
    );
    let group_color = if entity.group_locked {
        ColorToken::Accent
    } else {
        ColorToken::Text3
    };
    let group_icon = if entity.group_locked {
        IconId::Group
    } else {
        IconId::Ungroup
    };
    paint_icon(
        scene,
        group_icon,
        group_rect,
        resolve(group_color, theme),
        StrokeToken::Default.px(),
    );
    if let (Some(row_id), Some(idx)) = (row_id, hit_index.as_mut()) {
        let hit_pad = Spacing::Xs.px();
        let hit_rect = Rect::new(
            group_rect.x - hit_pad,
            group_rect.y - hit_pad,
            group_rect.w + hit_pad * 2.0,
            group_rect.h + hit_pad * 2.0,
        );
        idx.register(ids::hier_group_companion(row_id), hit_rect);
    }
    right_x -= icon_btn + icon_cluster_gap;
    // ── Lock individual (cadeado) — Enio: "Cadeado trava apenas o
    // objeto. Se este objeto tiver filhos, os filhos podem ser
    // manipulados". Sempre pintado, cor reflete estado.
    let lock_rect = Rect::new(
        right_x - icon_btn,
        rect.y + (rect.h - icon_btn) * 0.5,
        icon_btn,
        icon_btn,
    );
    let lock_color = if entity.locked {
        ColorToken::Accent
    } else {
        ColorToken::Text3
    };
    let lock_icon = if entity.locked {
        IconId::LockKeyhole
    } else {
        IconId::LockKeyholeOpen
    };
    paint_icon(
        scene,
        lock_icon,
        lock_rect,
        resolve(lock_color, theme),
        StrokeToken::Default.px(),
    );
    if let (Some(row_id), Some(idx)) = (row_id, hit_index.as_mut()) {
        let hit_pad = Spacing::Xs.px();
        let hit_rect = Rect::new(
            lock_rect.x - hit_pad,
            lock_rect.y - hit_pad,
            lock_rect.w + hit_pad * 2.0,
            lock_rect.h + hit_pad * 2.0,
        );
        idx.register(ids::hier_lock_companion(row_id), hit_rect);
    }
    right_x -= icon_btn + icon_cluster_gap;
    if let Some(swatch) = entity.swatch {
        let sw = SECTION_GAP_PX;
        let sw_rect = Rect::new(right_x - sw, rect.y + (rect.h - sw) * 0.5, sw, sw);
        // Canonical color swatch painter (single source of truth).
        let cs = ph2d_editor_core::widget::ColorSwatch::new(
            row_id.unwrap_or(ph2d_a11y::NodeId(0)),
            "",
            swatch,
        );
        ph2d_editor_core::widget::paint_color_swatch(&cs, sw_rect, scene, theme);
        right_x -= sw + icon_cluster_gap;
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
        right_x -= badge_w + icon_cluster_gap;
    }

    // Icon → name gap tightened Md (8) → Xs (4) 2026-05-24 per user:
    // "nome mais próximos dos ícones".
    let name_x = icon_rect.x + icon_w + Spacing::Xs.px();
    let name_color = if entity.muted {
        ColorToken::TextDisabled
    } else if direct_match {
        ColorToken::Accent
    } else {
        ColorToken::Text1
    };
    // Hard clip ao espaço entre name_x e o icon cluster — sem clip,
    // `paint_text` faz wrap quando a string excede a largura (nomes
    // longos quebravam em 2 linhas E invadiam os ícones). Com clip,
    // letras à direita simplesmente não aparecem (Enio 2026-05-26:
    // "As letras da direita do nome que invadirão os ícones devem
    // simplesmente não aparecer. Não podem sobrepor os ícones").
    // Reservar gap pequeno antes do primeiro ícone (não colado).
    let name_right_limit = right_x - Spacing::Xxs.px();
    let name_w = (name_right_limit - name_x).max(0.0);
    if name_w > 0.0 {
        let name_clip = ph2d_vector::Rect::new(
            name_x as f64,
            rect.y as f64,
            (name_x + name_w) as f64,
            (rect.y + rect.h) as f64,
        );
        scene.push_clip(&name_clip);
        paint_text(
            text_system,
            scene,
            &entity.name,
            name_x,
            rect.y + (rect.h - TypeToken::Sm.px()) * 0.5,
            TypeToken::Sm.px(),
            name_w,
            resolve(name_color, theme),
        );
        scene.pop_layer();
    }
}
