//! **A semente da secção DRAWING RULES** — a medida de cada linha vem do instantâneo (é assim que um
//! `Ctrl+Z` ou a curva se vê nela), MENOS o campo que a mão segura: reescrevê-lo durante a escrita ou
//! o arrasto faria o valor recuar sob o dedo.

use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::widget::Unit;

/// Chamada pelo [`crate::sync_sections`].
pub(crate) fn sync(host: &mut dyn PanelHostInternal) {
    let Some(info) = crate::state_components::current_inspector_cad() else {
        return;
    };
    for (row, &id) in info.rows.iter().zip(crate::ids::INSP_CAD_MEDIDA.iter()) {
        let Some(v) = row.medida else {
            continue;
        };
        if host.store().focus_id() != Some(id)
            && host.store().number_input_drag().map(|d| d.id) != Some(id)
        {
            host.store_mut().set_number_value(id, f64::from(v));
        }
    }
}

/// O sufixo e o passo do arrasto: 1 px numa distância e num raio, 1 grau num ângulo — a unidade do
/// canal, a que a curva grava.
pub(crate) const fn unidade_e_passo(graus: bool) -> (Unit, f64) {
    if graus {
        (Unit::Degrees, 1.0)
    } else {
        (Unit::Px, 1.0)
    }
}
