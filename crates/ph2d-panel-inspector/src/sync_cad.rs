//! **A semente da secção DRAWING RULES** — a medida de cada linha vem do instantâneo (é assim que um
//! `Ctrl+Z` ou a curva se vê nela), MENOS o campo que a mão segura: reescrevê-lo durante a escrita ou
//! o arrasto faria o valor recuar sob o dedo.
//!
//! O instantâneo fala a unidade do CANAL (metros numa distância e num raio, graus num ângulo); o ecrã
//! fala a do PROJECTO (Settings → px/metros e graus/radianos) — a mesma conversão do cartão
//! CONSTRAINTS ([`crate::sync_constraint::no_ecra`]).

use ph2d_editor_core::cad_edits::InspectorRegraRow;
use ph2d_editor_core::constraint_edits::Unidade;
use ph2d_editor_core::panel::PanelHostInternal;

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
            let v = crate::sync_constraint::no_ecra(unidade(row), v, host.project());
            host.store_mut().set_number_value(id, v);
        }
    }
}

/// A unidade do canal da linha: graus num ângulo, metros no resto.
pub(crate) const fn unidade(row: &InspectorRegraRow) -> Unidade {
    if row.graus {
        Unidade::Graus
    } else {
        Unidade::Metros
    }
}

/// [`crate::sync_constraint::no_ecra`] para quem PINTA (a REF, só texto): as escolhas do projecto que a
/// pintura publicou neste quadro.
pub(crate) fn no_ecra_pintado(row: &InspectorRegraRow, v: f32) -> f64 {
    match unidade(row) {
        Unidade::Graus => {
            crate::state::current_display_angle().from_radians_f64(f64::from(v).to_radians())
        }
        _ => crate::state::current_display_unit()
            .from_meters_f64(f64::from(v), crate::state::current_pixels_per_meter()),
    }
}
