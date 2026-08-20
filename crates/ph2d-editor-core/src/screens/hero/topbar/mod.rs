//! TopBar painter — 5 pill clusters + centered wordmark.

use super::HeroLayout;
use super::fixture;
use super::ids;
use crate::icons::IconId;
use crate::interaction::{HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use crate::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use crate::widget::{ButtonState, IconGlyph, PILL_PADDING_PX, Tooltip, paint_tooltip};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme};
use ph2d_vector::VectorScene;

/// Register every TopBar widget into the [`WidgetStore`]. Called
/// once by `HeroScreen::pre_populate_store`.
pub fn populate(store: &mut WidgetStore) {
    for id in [
        ids::TOPBAR_THEME,
        ids::TOPBAR_SAVE,
        ids::TOPBAR_SAVE_AS,
        ids::TOPBAR_OPEN,
        ids::TOPBAR_IMAGE_TOOLS,
        ids::TOPBAR_AUDIO_MIXER,
        ids::TOPBAR_AUDIO_EDITOR,
        // Vector pill MUST be registered here (not only painted/hit-indexed in
        // cluster_painter.rs): a pill absent here has no `InteractiveState`, so
        // pointer-Up never emits `Click` and the tool is dead on click.
        // (Audit 2026-06-02 killer.)
        ids::TOPBAR_VECTOR,
        // Motion Nodes pill — same parity requirement as the Vector pill above
        // (painted + hit-indexed in the fixture → MUST be registered here or the
        // pill is dead on click). Motion Nodes M0.T9.
        ids::TOPBAR_MOTION,
        // Flip pill — same parity requirement (registered here or dead on click).
        // ADR-0114 W2.
        ids::TOPBAR_FLIP,
        ids::TOPBAR_PHYSICS,
        ids::TOPBAR_TOKENS,
        ids::TOPBAR_AUTHORED,
        // Sculpt 3D (ADR-0150) — mesma exigência de paridade dos pills acima: sem registro AQUI
        // ele desenha e nasce morto sob o mouse.
        ids::TOPBAR_SCULPT3D,
        ids::TOPBAR_WIDGET_GALLERY,
        ids::TOPBAR_GRID_SETTINGS,
        ids::TOPBAR_SETTINGS,
        ids::TOPBAR_PROJECT,
        ids::TOPBAR_PLAY_BUTTON,
        ids::TOPBAR_PAUSE,
        ids::TOPBAR_RESET,
        ids::TOPBAR_RIGHT_LAYERS,
        ids::TOPBAR_RIGHT_ASSETS,
        ids::TOPBAR_RIGHT_SCRIPT,
    ] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    // **A fila de Image Tools regista-se pela MESMA porta por que é pintada.**
    //
    // ⚠️ Aqui esteve uma lista de dez `ids::IMAGE_ACTION_*` escritos à mão, e foi ela que matou o
    // pill `[SHEET]` (Enio, 2026-08-19: *«botão sheet não funciona»*). O painter deriva a fila do
    // registry — um tool novo aparece na barra sozinho, que é a promessa do drop-crate (ADR-0075)
    // — mas o REGISTO não crescia com ele: sem `InteractiveState` o pill tem
    // `is_focusable() == false`, o Down nunca arma o `active`, o Up nunca emite `Click`. *Ele
    // desenha e nasce morto debaixo do rato*, exatamente como os quatro pills de vetor de
    // `0661862`, e o gate que guarda aquele caso varre `ids::TOPBAR_*` — que uma fila derivada do
    // registry, por construção, não tem.
    //
    // `image_action_pills()` é a mesma função que o painter chama (e traz o seu próprio fallback
    // pré-registry), então painter e registo **não podem** divergir sem uma edição que os separe.
    // Gate do comportamento: `ph2d-tool-registry-init/tests/every_image_tool_pill_dispatches.rs`.
    for pill in image_action_pills() {
        store.register(
            pill.id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    // Group backdrops — Plain so clicks on empty backdrop space
    // emit `Click(<backdrop_id>)` that `apply_event` prints.
    for id in [
        ids::TOPBAR_LEFT_BACKDROP,
        ids::TOPBAR_RIGHT_BACKDROP,
        ids::TOPBAR_IMAGE_TOOLS_BACKDROP,
    ] {
        store.register(id, InteractiveState::Plain);
    }
    // Search input for the project-chip Scene List popover. Lives
    // here so opening the menu doesn't have to allocate state on
    // demand — its buffer is just filtered against the scene list
    // at paint time.
    store.register(
        ids::CTX_SCENE_SEARCH,
        InteractiveState::TextInput {
            state: crate::widget::TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    // Seed the generic tooltip side-table. Previously these strings
    // lived only in `tooltip_for(id)` and the hover painter matched
    // ids directly; now every widget can register its own tooltip
    // via `store.set_tooltip(id, text)` — keeps screens cohesive
    // with no boilerplate per-id lookup.
    for (id, text) in [
        // ASCII shortcuts — the macOS Command glyph U+2318 (⌘) and
        // Return glyph U+21B5 (↵) aren't in our parley font fallback
        // chain and rendered as tofu boxes. `Cmd+S` / `Cmd+Enter` are
        // legible on every theme without a special font.
        (ids::TOPBAR_SAVE, "Save \u{00b7} Cmd+S"),
        (ids::TOPBAR_SAVE_AS, "Save As\u{2026} \u{00b7} Cmd+Shift+S"),
        (ids::TOPBAR_OPEN, "Open \u{00b7} Cmd+O"),
        (ids::TOPBAR_IMAGE_TOOLS, "Image Tools"),
        (ids::TOPBAR_AUDIO_MIXER, "Audio Mixer"),
        (ids::TOPBAR_AUDIO_EDITOR, "Audio Editor"),
        (
            ids::TOPBAR_WIDGET_GALLERY,
            "Widget Gallery \u{00b7} reference",
        ),
        (ids::TOPBAR_PHYSICS, "Physics \u{00b7} W"),
        (ids::TOPBAR_TOKENS, "Tokens \u{00b7} T"),
        (ids::TOPBAR_AUTHORED, "Authored UI"),
        // ⚠️ Nomeia a tecla da OUTRA pergunta: o pill entra e sai, o `D` percorre as três posições.
        (ids::TOPBAR_SCULPT3D, "Sculpt 3D \u{00b7} D cycles"),
        (ids::TOPBAR_GRID_SETTINGS, "Grid Settings"),
        (
            ids::IMAGE_ACTION_TRIM,
            ph2d_i18n::tr("tool.trim_transparency.tooltip"),
        ),
        (
            ids::IMAGE_ACTION_MAKE_SQUARE,
            ph2d_i18n::tr("tool.make_square.tooltip"),
        ),
        (
            ids::IMAGE_ACTION_BGREMOVAL,
            ph2d_i18n::tr("tool.bgremoval.tooltip"),
        ),
        (
            ids::IMAGE_ACTION_REAL_SIZE,
            ph2d_i18n::tr("tool.real_size.tooltip"),
        ),
        (
            ids::IMAGE_ACTION_PADDING,
            ph2d_i18n::tr("tool.padding.tooltip"),
        ),
        (
            ids::IMAGE_ACTION_COLOR_EQUALIZATION,
            ph2d_i18n::tr("tool.color_equalization.tooltip"),
        ),
        (
            ids::IMAGE_ACTION_EQUALIZE_SIZES,
            ph2d_i18n::tr("tool.equalize_sizes.tooltip"),
        ),
        (
            ids::IMAGE_ACTION_RASTERIZE,
            ph2d_i18n::tr("tool.rasterize.tooltip"),
        ),
        (
            ids::IMAGE_ACTION_UPSCALE,
            ph2d_i18n::tr("tool.upscale.tooltip"),
        ),
        (
            ids::IMAGE_ACTION_PAINTER,
            ph2d_i18n::tr("tool.painter.tooltip"),
        ),
        (ids::TOPBAR_SETTINGS, "Project settings"),
        (ids::TOPBAR_PROJECT, "Project"),
        (ids::TOPBAR_PLAY_BUTTON, "Play \u{00b7} Space"),
        (ids::TOPBAR_PAUSE, "Pause \u{00b7} Space"),
        (ids::TOPBAR_RESET, "Reset \u{00b7} to start"),
        (ids::TOPBAR_RIGHT_LAYERS, "Layers"),
        (ids::TOPBAR_RIGHT_ASSETS, "Asset library"),
        (ids::TOPBAR_RIGHT_SCRIPT, "Code \u{00b7} Luau"),
        (ids::TOOL_TRANSLATE, "Translate \u{00b7} G"),
        (ids::TOOL_ROTATE, "Rotate \u{00b7} R"),
        (ids::TOOL_SCALE, "Scale \u{00b7} S"),
        (ids::TOOL_PIVOT, "Pivot"),
        (ids::TOOL_UNDO, "Undo"),
        (ids::TOOL_REDO, "Redo"),
        (ids::HIERARCHY_ADD, "Add entity"),
    ] {
        store.set_tooltip(id, text);
    }
}

/// Apply a [`WidgetEvent`] against TopBar widgets. Returns false so
/// the chrome dispatch chain can still react (Save → menu, Play →
/// run, etc.); this handler only side-effect-prints the clicked
/// chip's name to stdout (Enio 2026-05-25: "cada um dos componentes
/// deve ao click imprimir seu nome no console").
pub fn apply_event(_store: &mut WidgetStore, event: WidgetEvent) -> bool {
    if let WidgetEvent::Click(id) = event
        && let Some(name) = topbar_chip_name(id)
    {
        println!("[topbar] click: {name}");
    }
    false
}

/// Paint a Tooltip floating just above the currently hovered widget.
/// Called by the hero orchestrator after every chrome painter has
/// run so the tooltip lands on top. Tooltip text is read from the
/// store's generic tooltip side-table — populate it via
/// `WidgetStore::set_tooltip(id, text)` from any painter / populate
/// pass.
pub fn paint_hover_tooltip(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &HitIndex,
    store: &WidgetStore,
) {
    let Some(id) = store.hot_id() else {
        return;
    };
    // Suppress the tooltip when the hot widget is an open Dropdown
    // (or any widget whose popover paints directly below the hit
    // rect — they share the exact pill geometry the tooltip wants
    // and would otherwise paint OVER the first option). See
    // `docs/UI_Bugs/README.md` §9.13.
    if matches!(
        store.get(id),
        Some(crate::interaction::InteractiveState::Dropdown { open: true, .. })
            | Some(crate::interaction::InteractiveState::Combobox { open: true, .. })
    ) {
        return;
    }
    let Some(text) = store.tooltip_for(id) else {
        return;
    };
    let Some(target_rect) = hit_index.rect_for(id) else {
        return;
    };
    // Real text measurement instead of `chars × 6.5` — for
    // proportional fonts the approximation is off by 10-30 % and
    // the pill ends up clipped or oversized. See
    // `docs/UI_Bugs/README.md` §3.3.
    let font_size = ph2d_tokens::TypeToken::Sm.px();
    let measured_w = text_system.layout(text, font_size, f32::INFINITY).width();
    let pill_w = (measured_w + Spacing::Xl.px()).max(60.0); // LITERAL-PX-OK: tooltip pill min width (chrome-specific)
    let pill_h = (font_size + 10.0).max(22.0); // LITERAL-PX-OK: tooltip pill height composite + min (chrome-specific)
    // Center over the target; if that would clip the right edge of
    // the viewport, fall back to right-aligning to the target.
    let tip_x = target_rect.x + (target_rect.w - pill_w) * 0.5;
    let tip_rect = Rect::new(
        tip_x,
        target_rect.y + target_rect.h + Spacing::Sm.px(),
        pill_w,
        pill_h,
    );
    let tip = Tooltip::new(NodeId(0), text);
    paint_tooltip(&tip, tip_rect, scene, text_system, theme);
}

#[allow(clippy::too_many_arguments)]
pub fn paint_top_bar(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    image_tools_mode: bool,
    motion: &crate::motion::UiMotion,
) {
    let clusters = fixture::topbar_clusters();
    let row_h = layout.top_bar.h;
    let mut x = layout.top_bar.x;
    let gap = Spacing::Md.px();
    // Left half now holds 7 clusters: Theme, Project (Level), Save,
    // Open, Image Tools, Physics, Audio Mixer (Project moved here 2026-05-24;
    // Audio Mixer added 2026-07-05; Physics added 2026-07-27).
    //
    // ⚠️ **O número CRESCEU junto com a lista, e não é detalhe:** o pill novo
    // entra depois do IMG, então deixar o split em 6 empurraria o Audio Mixer
    // para o grupo da DIREITA — um pill mudando de lado da tela por causa de um
    // vizinho, que ninguém pediu.
    let split = 7.min(clusters.len());
    // Single agrupador backdrop spanning ALL left clusters (Enio
    // 2026-05-24: "Os componentes da esquerda devem ter apenas 1
    // fundo"). RailBg + radius Lg, top edge glued to viewport.y so
    // it touches the top of the screen.
    {
        let mut left_w = 0.0_f32;
        for (i, (_, c)) in clusters[..split].iter().enumerate() {
            if i > 0 {
                left_w += gap;
            }
            left_w += cluster_width(c);
        }
        if left_w > 0.0 {
            paint_topbar_group_backdrop(
                ids::TOPBAR_LEFT_BACKDROP,
                scene,
                theme,
                Rect::new(layout.top_bar.x, layout.top_bar.y, left_w, row_h),
                store.rail_button_size().chip_px(),
                layout.viewport.y,
                hit_index,
            );
        }
    }
    // Left half is always painted — the Image Tools mode keeps the
    // identity / Save / Open / ImageTools cluster visible so the user
    // can exit the mode by clicking ImageTools again.
    for (id, cluster) in &clusters[..split] {
        let rect = Rect::new(x, layout.top_bar.y, cluster_width(cluster), row_h);
        paint_top_bar_cluster(
            *id,
            cluster,
            rect,
            layout.viewport.y,
            scene,
            text_system,
            theme,
            hit_index,
            store,
            motion,
        );
        // Active-state ring on the ImageTools chip when the mode is
        // on. Must mirror `paint_topbar_rail_chip` chip_rect exactly:
        // stack_y = viewport_y + Xxs, chip_y = stack_y + label_band
        // + gap. Centralizar em rect.h fica deslocado porque o
        // backdrop é compacto e colado no topo (não no centro do
        // top_bar).
        if image_tools_mode && *id == ids::TOPBAR_IMAGE_TOOLS {
            let chip_px = store.rail_button_size().chip_px();
            let label_band_h = 11.0_f32; // LITERAL-PX-OK: mirror of rail's LABEL_VISUAL_EXTENT_PX
            let label_to_chip_gap = 3.0_f32; // LITERAL-PX-OK: mirror of rail's LABEL_TO_CHIP_GAP_PX
            let stack_y = layout.viewport.y + Spacing::Xxs.px();
            let chip_y = stack_y + label_band_h + label_to_chip_gap;
            let chip_rect = Rect::new(rect.x + (rect.w - chip_px) * 0.5, chip_y, chip_px, chip_px);
            stroke_rounded_rect(
                scene,
                chip_rect,
                Radius::Sm.px(),
                ph2d_tokens::StrokeToken::Default.px(),
                resolve(ColorToken::Accent, theme),
            );
        }
        x = rect.x + rect.w + gap;
    }
    // The wordmark "PH2D · EDITOR" that used to fill the middle gap
    // is intentionally absent now — the engine's identity is carried
    // by the leftmost theme chip (also labelled "PH2D"). Leaving the
    // gap transparent also keeps the topbar's bg fully see-through.
    // (`x` now holds the left group's right edge — used to clamp the right
    // group below so it can't overlap the left clusters when the bar
    // overflows.)

    if image_tools_mode {
        // Mode on — replace the right half with the image-action row.
        paint_image_action_row(layout, scene, text_system, theme, hit_index, store, motion);
        return;
    }

    // Default mode — paint the right clusters (Project / Play / Right /
    // Settings) right-aligned to the bar.
    let right_clusters = &clusters[split..];
    let mut right_w = 0.0_f32;
    for (i, (_, c)) in right_clusters.iter().enumerate() {
        if i > 0 {
            right_w += gap;
        }
        right_w += cluster_width(c);
    }
    // Right-align the right clusters — but clamp so the group never starts
    // before the left group ends (`x` = left-group right edge + gap). When
    // the bar overflows (too many clusters for the width), this stops the
    // leftmost right clusters (the vector tool pills) from being painted
    // under Save/Open/IMG — they stay on-screen + clickable; the rightmost
    // clusters clip off the right edge instead (restored by the W2-close
    // topbar UI pass). No-op when everything fits.
    let right_x = (layout.top_bar.x + layout.top_bar.w - right_w).max(x);
    // Single agrupador backdrop spanning ALL right clusters (Enio
    // 2026-05-24: "Os componentes da direita apenas um fundo").
    if right_w > 0.0 {
        paint_topbar_group_backdrop(
            ids::TOPBAR_RIGHT_BACKDROP,
            scene,
            theme,
            Rect::new(right_x, layout.top_bar.y, right_w, row_h),
            store.rail_button_size().chip_px(),
            layout.viewport.y,
            hit_index,
        );
    }
    let mut rx = right_x;
    for (id, cluster) in right_clusters {
        let rect = Rect::new(rx, layout.top_bar.y, cluster_width(cluster), row_h);
        paint_top_bar_cluster(
            *id,
            cluster,
            rect,
            layout.viewport.y,
            scene,
            text_system,
            theme,
            hit_index,
            store,
            motion,
        );
        rx = rect.x + rect.w + gap;
    }
}

/// Paint the agrupador backdrop behind a topbar cluster group.
/// Height is computed from `chip_px` so the plate is as tight as the
/// side rail's (Enio 2026-05-25: "altura reduzida que caiba apenas
/// os botões e as labels praticamente sem espaços"). `Radius::Md`
/// matches the side rail's backdrop corner. Horizontal `Sm` bleed
/// each side. Hit-registers FIRST so chips painted afterwards win
/// the hit (HitIndex walks back-to-front).
fn paint_topbar_group_backdrop(
    id: NodeId,
    scene: &mut VectorScene,
    theme: Theme,
    group_rect: Rect,
    chip_px: f32,
    viewport_y: f32,
    hit_index: &mut HitIndex,
) {
    let pad_h = Spacing::Sm.px();
    let pad_v = Spacing::Xxs.px();
    let label_band_h = 11.0_f32; // LITERAL-PX-OK: mirror of rail's LABEL_VISUAL_EXTENT_PX
    let label_to_chip_gap = 3.0_f32; // LITERAL-PX-OK: mirror of rail's LABEL_TO_CHIP_GAP_PX
    let bg_h = chip_px + label_to_chip_gap + label_band_h + pad_v * 2.0;
    // Colado no topo da viewport (Enio 2026-05-25).
    let bg_y = viewport_y;
    let bg = Rect::new(group_rect.x - pad_h, bg_y, group_rect.w + pad_h * 2.0, bg_h);
    hit_index.register(id, bg);
    fill_rounded_rect(
        scene,
        bg,
        Radius::Md.px(),
        resolve(ColorToken::RailBg, theme),
    );
}

/// Cluster-level painter + width helper. Extracted to a sibling
/// file in Wave 2 PR 11.7c so this `mod.rs` stays under the HR-18
/// 600-LOC cap. Same private surface as before; both functions
/// remain `pub(super)`-callable from `paint_top_bar`.
mod chip_name;
mod cluster_painter;
// A fila de Image Tools — a única do topbar DERIVADA do registry. Saiu para irmã quando
// este ficheiro passou o teto de 700 LOC (2026-08-19).
mod image_action_row;
// A tabela de tooltips — saiu para irmã na W4 do 3DModeling (2026-08-22), pelo mesmo teto.
mod tooltips;

use chip_name::topbar_chip_name;
use cluster_painter::{
    TOPBAR_INTER_CHIP_GAP, TOPBAR_RAIL_CHIP_W, cluster_width, paint_top_bar_cluster,
    paint_topbar_rail_chip,
};
pub use image_action_row::image_action_a11y_nodes;
use image_action_row::{image_action_pills, paint_image_action_row};
