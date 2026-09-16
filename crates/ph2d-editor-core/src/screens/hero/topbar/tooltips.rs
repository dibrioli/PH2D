//! **A tabela de dicas do top bar** — irmã de [`super`], cortada quando o teto de LOC de 700
//! disparou.
//!
//! ⚠️ O corte é por **assunto**, e ele compra isolamento: enquanto esta tabela morava dentro do
//! `populate`, toda linha paralela que acrescentasse um pill tocava as mesmas linhas do mesmo
//! arquivo. Uma tabela própria é um ponto de extensão que várias linhas estendem sem se ver — a
//! mesma cura que o `ph2d-i18n` fez com as chaves de painel.
//!
//! ⚠️ **É uma tabela, não um `match`:** o painter lê a dica do store (`set_tooltip`), então um id
//! que falte aqui simplesmente não tem dica — não há braço morto a esconder.

use super::ids;
use crate::interaction::WidgetStore;
use ph2d_i18n::tr;

// Seed the generic tooltip side-table. Previously these strings
// lived only in `tooltip_for(id)` and the hover painter matched
// ids directly; now every widget can register its own tooltip
// via `store.set_tooltip(id, text)` — keeps screens cohesive
// with no boilerplate per-id lookup.
pub(super) fn seed_tooltips(store: &mut WidgetStore) {
    for (id, text) in [
        // ASCII shortcuts — the macOS Command glyph U+2318 (⌘) and
        // Return glyph U+21B5 (↵) aren't in our parley font fallback
        // chain and rendered as tofu boxes. `Cmd+S` / `Cmd+Enter` are
        // legible on every theme without a special font.
        (ids::TOPBAR_SAVE, tr("chrome.topbar.tip.save_cmd_s")),
        (
            ids::TOPBAR_SAVE_AS,
            tr("chrome.topbar.tip.save_as_cmd_shift_s"),
        ),
        (ids::TOPBAR_OPEN, tr("chrome.topbar.tip.open_cmd_o")),
        (ids::TOPBAR_IMAGE_TOOLS, tr("chrome.topbar.tip.image_tools")),
        (ids::TOPBAR_AUDIO_MIXER, tr("chrome.topbar.tip.audio_mixer")),
        (
            ids::TOPBAR_AUDIO_EDITOR,
            tr("chrome.topbar.tip.audio_editor"),
        ),
        (
            ids::TOPBAR_WIDGET_GALLERY,
            tr("chrome.topbar.tip.widget_gallery_reference"),
        ),
        (ids::TOPBAR_PHYSICS, tr("chrome.topbar.tip.physics_w")),
        (ids::TOPBAR_TOKENS, tr("chrome.topbar.tip.tokens_t")),
        (ids::TOPBAR_AUTHORED, tr("chrome.topbar.tip.authored_ui")),
        // ⚠️ Nomeia a tecla da OUTRA pergunta: o pill entra e sai, o `D` percorre as três posições.
        (
            ids::TOPBAR_SCULPT3D,
            tr("chrome.topbar.tip.sculpt_3d_d_cycles"),
        ),
        (
            ids::TOPBAR_MODEL3D,
            tr("chrome.topbar.tip.n3d_model_implicit_field"),
        ),
        (
            ids::TOPBAR_GRID_SETTINGS,
            tr("chrome.topbar.tip.grid_settings"),
        ),
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
        (
            ids::TOPBAR_SETTINGS,
            tr("chrome.topbar.tip.project_settings"),
        ),
        (ids::TOPBAR_PROJECT, tr("chrome.topbar.tip.project")),
        (ids::TOPBAR_PLAY_BUTTON, tr("chrome.topbar.tip.play_space")),
        (ids::TOPBAR_PAUSE, tr("chrome.topbar.tip.pause_space")),
        (ids::TOPBAR_RESET, tr("chrome.topbar.tip.reset_to_start")),
        (ids::TOPBAR_RIGHT_LAYERS, tr("chrome.topbar.tip.layers")),
        (
            ids::TOPBAR_RIGHT_ASSETS,
            tr("chrome.topbar.tip.asset_library"),
        ),
        (ids::TOPBAR_RIGHT_SCRIPT, tr("chrome.topbar.tip.code_luau")),
        (ids::TOOL_TRANSLATE, tr("chrome.topbar.tip.translate_g")),
        (ids::TOOL_ROTATE, tr("chrome.topbar.tip.rotate_r")),
        (ids::TOOL_SCALE, tr("chrome.topbar.tip.scale_s")),
        (ids::TOOL_PIVOT, tr("chrome.topbar.tip.pivot")),
        (ids::TOOL_UNDO, tr("chrome.topbar.tip.undo")),
        (ids::TOOL_REDO, tr("chrome.topbar.tip.redo")),
        (ids::HIERARCHY_ADD, tr("chrome.topbar.tip.add_entity")),
    ] {
        store.set_tooltip(id, text);
    }
}
