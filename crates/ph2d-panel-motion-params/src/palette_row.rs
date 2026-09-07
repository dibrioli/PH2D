//! **A row de PALETA do painel** — hoje um ADAPTADOR: o editor mudou-se para a crate-folha
//! [`ph2d_param_editors`]. Ver o cabeçalho de [`crate::curve_row`].

use crate::snapshot::PaletteRow;
use ph2d_editor_core::interaction::HitIndex;
use ph2d_param_editors::EditorKey;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// As duas metades da chave — as amostras pelo NOME do param, que é o que a shell deriva.
pub(crate) fn chave(slot: usize, name: &str) -> (String, String) {
    (
        format!("motion_param/pal/{slot}"),
        format!("motion_param/pal_swatch/{name}"),
    )
}

/// Desenha a row de paleta. Devolve a altura usada.
#[expect(
    clippy::too_many_arguments,
    reason = "espelha a porta de paint do paint_rows"
)]
pub(crate) fn paint_palette_row(
    row: &PaletteRow,
    slot: usize,
    x: f32,
    w: f32,
    y: f32,
    label_font: f32,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    out: &mut super::gradient_row::ColourRowWidgets,
) -> f32 {
    let (own, swatch) = chave(slot, row.name);
    ph2d_param_editors::palette::paint(
        &row.label,
        &row.value,
        EditorKey {
            own: &own,
            swatch: &swatch,
        },
        x,
        w,
        y,
        label_font,
        hit_index,
        scene,
        text_system,
        theme,
        out,
    )
}

#[cfg(test)]
#[path = "palette_row_tests.rs"]
mod tests;
