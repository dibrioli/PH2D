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
        (ids::TOPBAR_MODEL3D, "3D Model \u{00b7} implicit field"),
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
