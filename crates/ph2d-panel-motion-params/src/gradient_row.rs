//! **A row de GRADIENTE do painel** — hoje um ADAPTADOR: o editor mudou-se para a crate-folha
//! [`ph2d_param_editors`], que tem dois hospedeiros (esta row e o cartão do nó no grafo).
//!
//! ⚠️ **A mudança de casa não moveu um único id** — a chave reproduz letra por letra as strings
//! que os `param_grad_*_id` usavam. Ver o cabeçalho de [`crate::curve_row`].

use crate::snapshot::GradientRow;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_param_editors::EditorKey;
pub(crate) use ph2d_param_editors::gradient::{
    ColourRowWidgets, PRESET_COUNT, add_stop, preset_gradient,
};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// As duas metades da chave desta row. ⚠️ **O prefixo das AMOSTRAS leva o NOME do param e não o
/// `slot`**, porque a shell também o deriva (ela semeia a cor e lê a escolha de volta) e não
/// conhece posições de row.
pub(crate) fn chave(slot: usize, name: &str) -> (String, String) {
    (
        format!("motion_param/grad/{slot}"),
        format!("motion_param/grad_swatch/{name}"),
    )
}

/// Desenha a row de gradiente. Devolve a altura usada.
#[expect(
    clippy::too_many_arguments,
    reason = "espelha a porta de paint do paint_rows"
)]
pub(crate) fn paint_gradient_row(
    row: &GradientRow,
    slot: usize,
    x: f32,
    w: f32,
    y: f32,
    label_font: f32,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    out: &mut ColourRowWidgets,
) -> f32 {
    let (own, swatch) = chave(slot, row.name);
    ph2d_param_editors::gradient::paint(
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

/// ⚠️ **O `name` entra porque a chave o pede** — e o drain só usa a raiz, que não depende dele;
/// passá-lo à mesma mantém UMA construção de chave, em vez de duas que podem divergir.
pub(crate) fn drain_drag(
    store: &mut WidgetStore,
    slot: usize,
    name: &str,
    value: &str,
) -> Option<String> {
    let (own, swatch) = chave(slot, name);
    ph2d_param_editors::gradient::drain_drag(
        store,
        EditorKey {
            own: &own,
            swatch: &swatch,
        },
        value,
    )
}

pub(crate) fn remove_stop(value: &str, slot: usize, name: &str) -> String {
    let (own, swatch) = chave(slot, name);
    ph2d_param_editors::gradient::remove_stop(
        value,
        EditorKey {
            own: &own,
            swatch: &swatch,
        },
    )
}

pub(crate) fn cycle_interp(value: &str, slot: usize, name: &str) -> String {
    let (own, swatch) = chave(slot, name);
    ph2d_param_editors::gradient::cycle_interp(
        value,
        EditorKey {
            own: &own,
            swatch: &swatch,
        },
    )
}

pub(crate) use ph2d_param_editors::gradient::{cycle_hue, cycle_space};
