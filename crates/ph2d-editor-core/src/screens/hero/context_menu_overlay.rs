//! Right-click context-menu overlay.
//!
//! Painted by the hero orchestrator after every panel painter so the
//! floating menu lands above everything. Reads the open-menu state
//! from [`crate::interaction::WidgetStore::context_menu`] and walks a
//! per-kind option list (one match arm per [`ContextMenuKind`]).

use super::fixture;
use super::ids;
use crate::icons::IconId;
use crate::interaction::{ContextMenuKind, HitIndex, InteractiveState, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_icon, paint_text, resolve, stroke_rounded_rect};
use crate::widget::{TextInput, paint_text_input_with_buffer};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{
    ColorToken, ROW_H_PX, Radius, SECTION_GAP_PX, Spacing, StrokeToken, Theme, TypeToken,
};
use ph2d_vector::{Color as VelloColor, VectorScene};

/// Standard context-menu width. Exported so cascade-opening handlers
/// in `chrome/` can decide whether the submenu fits to the right of
/// its parent row or needs to flip to the left.
pub const MENU_W: f32 = 200.0; // LITERAL-PX-OK: context menu fixed width (chrome-specific)
pub(super) const ROW_H: f32 = ROW_H_PX;
/// ⚠️ `fn` e nao `const`: a escala e AUTORAVEL (plano UI/UX W4c.2), entao este numero e lido
/// por quadro em vez de assado na compilacao.
pub(super) fn pad_y() -> f32 {
    Spacing::Sm.px()
}
/// Clamp a menu rect anchored at `(anchor_x, anchor_y)` so it stays
/// inside `viewport`. Used by both the simple-row menu and the scene
/// list — cascading submenus (Settings → PPM) anchored near the
/// right edge would otherwise render off-screen.
///
/// ⚠️ **A lei mudou-se para o [`crate::widget::panel_chrome::clamp_menu_to_viewport`]** quando
/// ganhou um consumidor numa crate de PAINEL (a lista do "+ Track" da timeline). Isto delega:
/// duas cópias divergiriam, e o modo de falha seria *um* menu do app a sair da tela.
fn clamp_to_viewport(anchor_x: f32, anchor_y: f32, w: f32, h: f32, viewport: Rect) -> Rect {
    crate::widget::panel_chrome::clamp_menu_to_viewport(anchor_x, anchor_y, w, h, viewport)
}

/// Five common highlighter colors used for note backgrounds and
/// section outlines. Wave 8 Phase 2.A re-export from
/// `ph2d-editor-core::widget::panel_chrome::HIGHLIGHTER_RGBA` so panel
/// crates don't need to reach into `ph2d_editor::screens::hero` to
/// paint user-placed notes.
pub use crate::widget::panel_chrome::HIGHLIGHTER_RGBA;

/// Does `id` correspond to the currently-active choice for any of
/// the single-choice menus? The row paint draws an accent bullet
/// next to the row whose id matches, mirroring SceneList's "current
/// scene" indicator (2026-05-24 menu standardization).
///
/// One function covers every menu because IDs are unique across all
/// menu kinds — checking `id == active_theme_id || id == active_radius
/// _id || ...` is unambiguous and avoids threading `kind` through the
/// row loop.
pub(super) fn id_is_currently_selected(
    id: NodeId,
    theme: Theme,
    store: &WidgetStore,
    project: &crate::project::ProjectSettings,
    motion: &crate::motion::UiMotion,
) -> bool {
    use crate::project::{DisplayAngle, DisplayUnit, ImageFilterMode};
    use crate::widget::RailButtonSize;
    // ⛔⛔ **AS DEZASSEIS LINHAS DE ALTERNÂNCIA DA BARRA DE MENUS** — os treze módulos, os dois
    // painéis e a régua. Elas nasceram em 2026-08-30 **sem marca nenhuma**: o menu *Window* dizia
    // exactamente a mesma coisa com o Vector aberto e fechado.
    //
    // ⚠️ É a lei que este ficheiro já documenta, paga na unidade de ângulo: *«fiar o clique não é
    // fiar o ESTADO»* — e a barra repetiu-a dezasseis vezes de uma vez. Antes dela a indicação
    // existia: o laço de reconciliação da shell força `Pressed` no pill do tool activo, e o pill
    // lia-o. O pill saiu; a marca não foi com ele para lado nenhum.
    if super::menu_bar::row_is_marked_by_button_state(id) {
        return matches!(
            store.button_state(id),
            Some(crate::widget::ButtonState::Pressed)
        );
    }
    let theme_id = match theme {
        Theme::Forge => ids::CTX_MENU_THEME_FORGE,
        Theme::Workshop => ids::CTX_MENU_THEME_PAINT,
        Theme::Sunstone => ids::CTX_MENU_THEME_SUNSTONE,
        Theme::Blueprint => ids::CTX_MENU_THEME_BLUEPRINT,
    };
    if id == theme_id {
        return true;
    }
    const RADIUS_ROUND_THRESH: f32 = 1.3; // LITERAL-PX-OK: midpoint between Default(1.0) and Round(1.5) radius preset
    let radius_id = if store.radius_scale() < 0.5 {
        ids::CTX_MENU_RADIUS_SHARP
    } else if store.radius_scale() > RADIUS_ROUND_THRESH {
        ids::CTX_MENU_RADIUS_ROUND
    } else {
        ids::CTX_MENU_RADIUS_DEFAULT
    };
    if id == radius_id {
        return true;
    }
    let rail_id = match store.rail_button_size() {
        RailButtonSize::Small => ids::CTX_MENU_RAIL_SIZE_SMALL,
        RailButtonSize::Medium => ids::CTX_MENU_RAIL_SIZE_MEDIUM,
        RailButtonSize::Large => ids::CTX_MENU_RAIL_SIZE_LARGE,
    };
    if id == rail_id {
        return true;
    }
    let ppm_id = match project.pixels_per_meter as i32 {
        16 => Some(ids::CTX_MENU_PPM_16),
        32 => Some(ids::CTX_MENU_PPM_32),
        100 => Some(ids::CTX_MENU_PPM_100),
        256 => Some(ids::CTX_MENU_PPM_256),
        1024 => Some(ids::CTX_MENU_PPM_1024),
        _ => None,
    };
    if ppm_id == Some(id) {
        return true;
    }
    let unit_id = match project.display_unit {
        DisplayUnit::Meters => ids::CTX_MENU_UNIT_METERS,
        DisplayUnit::Pixels => ids::CTX_MENU_UNIT_PIXELS,
    };
    if id == unit_id {
        return true;
    }
    // Angle unit — a irmã do `display_unit` acima (Enio, 2026-08-30). ⚠️ **Esta linha faltou na
    // 1.ª entrega da feature**, e o defeito é da família que este ficheiro existe para curar: o
    // menu abria, o clique funcionava e o valor gravava — mas **nenhuma das duas opções aparecia
    // marcada**, então não havia como ver em que unidade se estava sem abrir o Inspector e
    // comparar. *Fiar o clique não é fiar o ESTADO.*
    let angle_id = match project.display_angle {
        DisplayAngle::Degrees => ids::CTX_MENU_ANGLE_DEGREES,
        DisplayAngle::Radians => ids::CTX_MENU_ANGLE_RADIANS,
    };
    if id == angle_id {
        return true;
    }
    let filter_id = match project.image_filter {
        ImageFilterMode::PixelArt => ids::CTX_MENU_FILTER_PIXELART,
        ImageFilterMode::Smooth => ids::CTX_MENU_FILTER_SMOOTH,
    };
    if id == filter_id {
        return true;
    }
    // Text rendering — value lives on the `paint::text_rendering`
    // thread-local (published per-frame from `HeroScreen.text_rendering`),
    // so we can read it here without threading another param through.
    let text_id = match crate::paint::text_rendering() {
        ph2d_tokens::TextRendering::Default => ids::CTX_MENU_TEXT_DEFAULT,
        ph2d_tokens::TextRendering::CrispHeavy => ids::CTX_MENU_TEXT_CRISP_HEAVY,
        ph2d_tokens::TextRendering::CrispHeavyPlus => ids::CTX_MENU_TEXT_CRISP_HEAVY_PLUS,
        ph2d_tokens::TextRendering::CrispEmbolden => ids::CTX_MENU_TEXT_CRISP_EMBOLDEN,
    };
    if id == text_id {
        return true;
    }
    // Display submenu (VSync / Immediate) — store mirrors the last
    // value `settings_present::apply` published; default `true` matches
    // the shell's `Fifo` baseline.
    let display_id = if store.present_vsync() {
        ids::CTX_MENU_DISPLAY_VSYNC
    } else {
        ids::CTX_MENU_DISPLAY_IMMEDIATE
    };
    if id == display_id {
        return true;
    }
    // Motion — o carácter é um RÁDIO (uma linha acesa das duas) e o reduced motion é um TOGGLE
    // (aceso quando ligado). ⚠️ São perguntas independentes, então são dois `if` e não um `match`:
    // *Expressivo + reduced* tem de conseguir acender as duas linhas ao mesmo tempo.
    let character_id = match motion.character() {
        crate::motion::UiCharacter::Discrete => ids::CTX_MENU_MOTION_DISCRETE,
        crate::motion::UiCharacter::Expressive => ids::CTX_MENU_MOTION_EXPRESSIVE,
    };
    if id == character_id {
        return true;
    }
    if id == ids::CTX_MENU_MOTION_REDUCED && motion.reduced_motion() {
        return true;
    }
    false
}

/// Paint the open context menu (if any) and register hit rects for each item. Called last in the hero
/// paint pipeline so the menu always sits on top. `viewport` clamps the menu rect so a cascade submenu
/// anchored near the right/bottom edge (e.g. Settings → PPM from the topbar gear) stays on-screen.
// ⚠️ O `motion` é o 8º argumento, e passá-lo é deliberado: `UiMotion` é o DONO do carácter, e
// espelhá-lo no `WidgetStore` (como o `present_vsync` faz) daria uma terceira cópia do mesmo facto
// — a viva, o espelho e o ficheiro de preferências. Precedente do argumento extra: `body_desc` na
// física. E o painter recebe `&UiMotion`, nunca `&mut`: quem pinta lê, nunca alveja.
#[allow(clippy::too_many_arguments)]
pub fn paint_context_menu_overlay(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    project: &crate::project::ProjectSettings,
    motion: &crate::motion::UiMotion,
    viewport: Rect,
) {
    let Some(req) = store.context_menu() else {
        return;
    };
    // ⚠️ A tabela mora no `menu_rows` desde que a paleta de comandos passou a oferecer os
    // mesmos verbos: uma segunda lista aqui divergiria da que a paleta lê.
    //
    // ⭐⭐⭐ **A ÚNICA excepção são as rows da ÁREA**, e ela é de natureza: quem as tem é o módulo
    // que está com o canvas, e ele publica-as no store uma vez por quadro
    // (`WidgetStore::area_entries`). Uma tabela estática teria de conhecer os ids de todo módulo do
    // app — o acoplamento que a **D2** existe para não ter.
    //
    // ⚠️ O `Vec` vive nesta função de propósito: o empréstimo é do `store`, que sobrevive a ela, e
    // o laço de rows a seguir é o MESMO. *A row continua a ser `(id, rótulo, swatch)`.*
    let merged: Vec<(NodeId, &str, Option<[u8; 4]>)>;
    let statics = super::menu_rows::menu_rows(req.kind);
    // ⭐⭐ **As duas metades da D2 caem na MESMA aritmética.** Um pulldown de área é o caso em que
    // as estáticas são `&[]` (ver `menu_rows`), e o *File* é o caso em que as duas existem — as
    // linhas do app primeiro, o que o módulo acrescenta depois. ⛔ Um `if` por menu seria a segunda
    // porta a apodrecer.
    let contrib: &[crate::widget::ToolRailEntry] = match req.kind {
        ContextMenuKind::AreaCommands { slot } => store.area_menu_rows(slot),
        kind => store.menu_contrib(kind),
    };
    let items: &[(NodeId, &str, Option<[u8; 4]>)] = if contrib.is_empty() {
        statics
    } else {
        // ⚠️ `filter_map` e não `map`: o `Divider` não tem id nem rótulo, e uma linha de menu
        // sem verbo seria um alvo que consome o clique e não faz nada.
        merged = statics
            .iter()
            .copied()
            .chain(
                contrib
                    .iter()
                    .filter_map(|e| Some((e.node_id()?, e.label()?, None))),
            )
            .collect();
        &merged
    };

    if matches!(req.kind, ContextMenuKind::SceneList) {
        paint_scene_list(req, scene, text_system, theme, hit_index, store, viewport);
        return;
    }
    // ⭐⭐ **O transbordo da fila** — os chips que não couberam, com os MESMOS ids. Ver
    // `hero::tool_bar::bar_split`.
    if matches!(req.kind, ContextMenuKind::ToolBarOverflow) {
        paint_tool_bar_overflow(
            req,
            scene,
            text_system,
            theme,
            hit_index,
            store,
            motion,
            viewport,
        );
        return;
    }
    // Centered single-panel dialogs all share one painter signature — dispatch by kind.
    type DialogFn = fn(&mut VectorScene, &mut TextSystem, Theme, &mut HitIndex, &WidgetStore, Rect);
    let dialog: Option<DialogFn> = match req.kind {
        ContextMenuKind::RenamePaletteDialog => {
            Some(super::context_menu_dialogs::paint_palette_rename_dialog)
        }
        ContextMenuKind::NewImageDialog => {
            Some(super::context_menu_dialogs::paint_new_image_dialog)
        }
        ContextMenuKind::SheetSizeDialog => {
            Some(super::context_menu_dialogs::paint_sheet_size_dialog)
        }
        _ => None,
    };
    if let Some(paint_dialog) = dialog {
        paint_dialog(scene, text_system, theme, hit_index, store, viewport);
        return;
    }
    let total_h = ROW_H * items.len() as f32 + pad_y() * 2.0;
    let rect = clamp_to_viewport(req.x, req.y, MENU_W, total_h, viewport);

    // Floating panel: BgElev fill + Border stroke + Md radius.
    let radius = Radius::Md.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));

    // Rows.
    let row_x = rect.x + Spacing::Xs.px();
    let row_w = rect.w - Spacing::Xs.px() * 2.0;
    // Bullet column width — same gap SceneList uses (bullet x +
    // 10 px → text x). Painted on every row so labels align whether
    // the row is current or not.
    let bullet_col_w: f32 = 10.0; // LITERAL-PX-OK: chrome-specific bullet→text gap
    for (i, (id, label, swatch)) in items.iter().enumerate() {
        let r = Rect::new(row_x, rect.y + pad_y() + ROW_H * i as f32, row_w, ROW_H);
        hit_index.register(*id, r);
        if Some(*id) == store.hot_id() {
            fill_rounded_rect(scene, r, Radius::Sm.px(), resolve(ColorToken::Bg2, theme));
        }
        let pad_x = Spacing::Md.px();
        let icon_size = SECTION_GAP_PX;
        let icon_y = r.y + (r.h - icon_size) * 0.5;
        // Bullet for currently-selected item (SceneList parity,
        // 2026-05-24 menu standardization).
        let is_current = id_is_currently_selected(*id, theme, store, project, motion);
        let bullet_x = r.x + pad_x;
        if is_current {
            let dot = ph2d_vector::Circle::new(
                ph2d_vector::Point::new(bullet_x as f64, (r.y + r.h * 0.5) as f64),
                3.0, // LITERAL-PX-OK: bullet dot radius (chrome accent, matches SceneList)
            );
            scene.inner_mut().fill(
                ph2d_vector::Fill::NonZero,
                ph2d_vector::Affine::IDENTITY,
                &ph2d_vector::Brush::Solid(resolve(ColorToken::Accent, theme)),
                None,
                &dot,
            );
        }
        let glyph_x = bullet_x + bullet_col_w;
        // Leading visual: color swatch for outline picks, "+" icon
        // for create-note. Painted to the right of the bullet column
        // so the bullet always lines up flush-left.
        let has_glyph = swatch.is_some() || matches!(req.kind, ContextMenuKind::CreateNote { .. });
        if let Some(rgba) = swatch {
            let sw = Rect::new(glyph_x, icon_y, icon_size, icon_size);
            fill_rounded_rect(
                scene,
                sw,
                3.0, // LITERAL-PX-OK: context-menu swatch radius (chrome-specific accent)
                VelloColor::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]), // LITERAL-COLOR-OK: user-color — swatch shows an outline-pick rgba, not a theme token
            );
            stroke_rounded_rect(scene, sw, 3.0, 1.0, resolve(ColorToken::Border, theme)); // LITERAL-PX-OK: context-menu swatch radius (chrome-specific accent)
        } else if matches!(req.kind, ContextMenuKind::CreateNote { .. }) {
            paint_icon(
                scene,
                IconId::Add,
                Rect::new(glyph_x, icon_y, icon_size, icon_size),
                resolve(ColorToken::Text2, theme),
                StrokeToken::Default.px(),
            );
        }
        let text_x = if has_glyph {
            glyph_x + icon_size + Spacing::Sm.px()
        } else {
            glyph_x
        };
        let text_y = r.y + (r.h - TypeToken::Sm.px()) * 0.5;
        paint_text(
            text_system,
            scene,
            label,
            text_x,
            text_y,
            TypeToken::Sm.px(),
            (r.x + r.w - text_x - pad_x).max(0.0),
            resolve(
                if is_current {
                    ColorToken::Text1
                } else {
                    ColorToken::Text2
                },
                theme,
            ),
        );
        // Cascade indicator on the SettingsMenu rows — real
        // `IconId::ChevronRight` glyph at the right edge, replacing
        // the Unicode U+25B6 / U+203A trailing character. Enio
        // 2026-05-25: "usando emojis e não ícones".
        if matches!(req.kind, ContextMenuKind::SettingsMenu) {
            let chev = Rect::new(r.x + r.w - pad_x - icon_size, icon_y, icon_size, icon_size);
            paint_icon(
                scene,
                IconId::ChevronRight,
                chev,
                resolve(ColorToken::Text3, theme),
                StrokeToken::Default.px(),
            );
        }
    }
}

/// Paint the Scene List popover.
///
/// Layout: TextInput search field at the top + up to 8 filtered
/// scene rows below. The search uses the regular TextInput state at
/// `CTX_SCENE_SEARCH` (its buffer is the filter query). Each visible
/// result row registers a unique `CTX_SCENE_ROWS[i]` hit so the
/// dispatch / `apply_event` chain can route the selection.
fn paint_scene_list(
    req: crate::interaction::ContextMenuRequest,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    viewport: Rect,
) {
    let menu_w = 260.0_f32; // LITERAL-PX-OK: scene-list popover width (chrome-specific)
    let search_h = 30.0_f32; // LITERAL-PX-OK: scene-list search field height (chrome-specific)
    let row_h = ROW_H;
    let max_rows = ids::CTX_SCENE_ROWS.len();
    // Read the current search query directly from the TextInput
    // state at CTX_SCENE_SEARCH. Empty string when the user hasn't
    // typed anything yet.
    let (query, caret, anchor, ti_state) = match store.get(ids::CTX_SCENE_SEARCH) {
        Some(InteractiveState::TextInput {
            text,
            caret,
            selection_anchor,
            state,
        }) => (text.as_str(), *caret, *selection_anchor, *state),
        _ => ("", 0, None, crate::widget::TextInputState::Normal),
    };
    // Filter scenes case-insensitively. Empty query matches all.
    let lower_q = query.to_lowercase();
    let filtered: Vec<&'static str> = fixture::scenes()
        .iter()
        .copied()
        .filter(|s| lower_q.is_empty() || s.to_lowercase().contains(&lower_q))
        .take(max_rows)
        .collect();

    let row_count = filtered.len().max(1); // reserve at least one row for "No matches"
    let total_h = pad_y() * 2.0 + search_h + Spacing::Xs.px() + row_count as f32 * row_h;
    let rect = clamp_to_viewport(req.x, req.y, menu_w, total_h, viewport);

    // Floating panel surface.
    let radius = Radius::Md.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));

    // Search input row.
    let inner_x = rect.x + Spacing::Xs.px();
    let inner_w = rect.w - Spacing::Xs.px() * 2.0;
    let search_rect = Rect::new(inner_x, rect.y + pad_y(), inner_w, search_h);
    hit_index.register(ids::CTX_SCENE_SEARCH, search_rect);
    let ti = TextInput::new(ids::CTX_SCENE_SEARCH, "")
        .placeholder("Search scenes\u{2026}")
        .visual((ti_state, store.hover_live(ids::CTX_SCENE_SEARCH)));
    paint_text_input_with_buffer(
        &ti,
        Some(query),
        Some(caret),
        anchor,
        search_rect,
        scene,
        text_system,
        theme,
    );

    // Result rows.
    let rows_y0 = search_rect.y + search_rect.h + Spacing::Xs.px();
    let pad_x = Spacing::Md.px();
    let font = TypeToken::Sm.px();
    if filtered.is_empty() {
        let r = Rect::new(inner_x, rows_y0, inner_w, row_h);
        paint_text(
            text_system,
            scene,
            "No matches",
            r.x + pad_x,
            r.y + (r.h - font) * 0.5,
            font,
            r.w - pad_x * 2.0,
            resolve(ColorToken::Text3, theme),
        );
    } else {
        for (i, name) in filtered.iter().enumerate() {
            let row_id = ids::CTX_SCENE_ROWS[i];
            let r = Rect::new(inner_x, rows_y0 + i as f32 * row_h, inner_w, row_h);
            hit_index.register(row_id, r);
            if Some(row_id) == store.hot_id() {
                fill_rounded_rect(scene, r, Radius::Sm.px(), resolve(ColorToken::Bg2, theme));
            }
            // Highlight the currently-active scene with an accent
            // bullet so the user sees "this is what's loaded".
            let is_current = *name == store.current_scene_name();
            let bullet_x = r.x + pad_x;
            let bullet_y = r.y + r.h * 0.5;
            if is_current {
                let dot = ph2d_vector::Circle::new(
                    ph2d_vector::Point::new(bullet_x as f64, bullet_y as f64),
                    3.0, // LITERAL-PX-OK: scene-list bullet dot radius (chrome-specific accent)
                );
                scene.inner_mut().fill(
                    ph2d_vector::Fill::NonZero,
                    ph2d_vector::Affine::IDENTITY,
                    &ph2d_vector::Brush::Solid(resolve(ColorToken::Accent, theme)),
                    None,
                    &dot,
                );
            }
            let text_x = bullet_x + 10.0; // LITERAL-PX-OK: bullet → text gap (chrome-specific)
            let text_y = r.y + (r.h - font) * 0.5;
            paint_text(
                text_system,
                scene,
                name,
                text_x,
                text_y,
                font,
                r.w - (text_x - r.x) - pad_x,
                resolve(
                    if is_current {
                        ColorToken::Text1
                    } else {
                        ColorToken::Text2
                    },
                    theme,
                ),
            );
            let _ = VelloColor::from_rgba8(0, 0, 0, 0); // LITERAL-COLOR-OK: import-keepalive no-op (keeps the VelloColor import live for conditional paths)
            let _ = IconId::Search; // keep IconId import in scope
        }
    }
}

// The centered dialog-style popovers (API-key / vector-prompt / palette-rename) live in the sibling
// `super::context_menu_dialogs` (file-LOC split); this module dispatches to them above.

/// ⭐⭐⭐ **O CORPO DO TRANSBORDO** — os chips que não couberam na fila, numa grelha.
///
/// > Enio, 2026-08-31: *«esse app tem tablets e iPad como alvo. Não podemos ir perdendo espaço.»*
///
/// ⚠️ **Os ids são os MESMOS da fila**, e é isso que torna este corpo barato: quem despacha
/// continua a ser o `chrome::rail_*`, e um verbo copiado para aqui seria a segunda porta que o
/// `CLAUDE.md` §5.0 cataloga como a espécie mais cara de controlo morto.
///
/// ⚠️ **E a lista sai da MESMA função que a faixa usa** (`tool_bar::bar_split`) — nunca de uma
/// segunda conta. Uma cópia da aritmética poria um chip nos dois sítios, ou em nenhum.
#[allow(clippy::too_many_arguments)]
fn paint_tool_bar_overflow(
    req: crate::interaction::ContextMenuRequest,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    motion: &crate::motion::UiMotion,
    viewport: Rect,
) {
    // ⭐ **Publicado por quem o calculou** — a fila, uma vez por quadro (`tool_bar::bar_split`).
    // ⛔ Uma segunda conta aqui poria um chip nos dois sítios, ou em nenhum: a faixa é a única que
    // sabe a largura dela.
    let over: Vec<_> = store.tool_overflow().to_vec();
    if over.is_empty() {
        return;
    }
    let size = store.rail_button_size();
    let pad = Spacing::Sm.px();
    // ⚠️ Uma COLUNA, e não uma grelha: o menu abre debaixo de um chip da faixa, e uma grelha larga
    // taparia a área de desenho que esta wave existe para poupar.
    let rail = crate::widget::ToolRail::new(
        ph2d_a11y::NodeId(204),
        "More tools",
        over.into_iter().collect(),
    );
    // A largura de uma coluna de chips, e a altura pelo passo de linha — as mesmas contas do
    // trilho vertical, para o menu não inventar métrica própria.
    let inner_w = crate::widget::CHIP_X_OFFSET_PX + size.chip_px() + Spacing::Xs.px();
    let inner_h = crate::widget::line_pitch(size.chip_px()) * rail.entries.len() as f32;
    let rect = clamp_to_viewport(
        req.x,
        req.y,
        inner_w + pad * 2.0,
        inner_h + pad * 2.0,
        viewport,
    );
    let radius = Radius::Md.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));
    let content = Rect::new(rect.x + pad, rect.y + pad, inner_w, inner_h);
    crate::widget::paint_tool_rail_axis(
        &rail,
        content,
        scene,
        text_system,
        theme,
        store,
        &|id| Some(motion.get(id).unwrap_or(0.0)),
        motion.travels(),
        crate::widget::RailAxis::Vertical,
    );
    // ⚠️ O fundo ENGOLE o clique (a mesma razão do `RAIL_BACKDROP` da faixa), e vai ANTES dos
    // chips: o `HitIndex` caminha de trás para a frente.
    hit_index.register(ids::RAIL_BACKDROP, rect);
    for slot in crate::widget::entry_rects(&rail, content, size, crate::widget::RailAxis::Vertical)
    {
        if let Some(id) = slot.id {
            hit_index.register(id, slot.rect);
        }
    }
}
