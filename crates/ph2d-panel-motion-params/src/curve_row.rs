//! **A row de CURVA do painel** — hoje um ADAPTADOR: o editor mudou-se para a crate-folha
//! [`ph2d_param_editors`], que tem dois hospedeiros (esta row e o cartão do nó no grafo).
//!
//! ⚠️ **A mudança de casa não moveu um único id.** A chave que esta row passa reproduz, letra por
//! letra, a string que os `param_curve_*_id` já usavam (`motion_param/curve/{slot}`) — há gate na
//! folha a prendê-la. *Um id derivado de uma string muda de valor sem que nada deixe de compilar,
//! e um arrasto passaria a falar de um widget que ninguém desenhou.*

use crate::snapshot::CurveRow;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_param_editors::EditorKey;
pub(crate) use ph2d_param_editors::curve::CurveWidgets;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// A chave desta row — ver o aviso do cabeçalho. ⚠️ **A vida do `String` é o motivo de ela ser
/// construída em cada chamador em vez de devolvida daqui**: a `EditorKey` empresta as strings.
macro_rules! chave {
    ($slot:expr) => {
        (
            format!("motion_param/curve/{}", $slot),
            String::from("motion_param/curve_swatch"),
        )
    };
}

/// Desenha a row de curva. Devolve a altura usada.
#[expect(
    clippy::too_many_arguments,
    reason = "espelha a porta de paint do paint_rows"
)]
pub(crate) fn paint_curve_row(
    row: &CurveRow,
    slot: usize,
    x: f32,
    w: f32,
    y: f32,
    label_font: f32,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    out: &mut CurveWidgets,
) -> f32 {
    let (own, swatch) = chave!(slot);
    ph2d_param_editors::curve::paint(
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

/// O arrasto de uma alça, dobrado na curva — ver [`ph2d_param_editors::curve::drain_drag`].
pub(crate) fn drain_drag(store: &mut WidgetStore, slot: usize, value: &str) -> Option<String> {
    let (own, swatch) = chave!(slot);
    ph2d_param_editors::curve::drain_drag(
        store,
        EditorKey {
            own: &own,
            swatch: &swatch,
        },
        value,
    )
}

pub(crate) use ph2d_param_editors::curve::add_point;

/// Apaga o ponto selecionado (senão o último), mantendo pelo menos dois.
pub(crate) fn remove_point(value: &str, slot: usize) -> String {
    let (own, swatch) = chave!(slot);
    ph2d_param_editors::curve::remove_point(
        value,
        EditorKey {
            own: &own,
            swatch: &swatch,
        },
    )
}

/// Cicla a interpolação do segmento do ponto selecionado.
pub(crate) fn cycle_interp(value: &str, slot: usize) -> String {
    let (own, swatch) = chave!(slot);
    ph2d_param_editors::curve::cycle_interp(
        value,
        EditorKey {
            own: &own,
            swatch: &swatch,
        },
    )
}
