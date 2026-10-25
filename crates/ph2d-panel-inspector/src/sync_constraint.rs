//! **A semente do cartão CONSTRAINTS** — todo o quadro, a força e o «Enabled» de cada linha vêm do
//! instantâneo (é assim que um `Ctrl+Z` se vê neles), MENOS a barra que o dedo segura: reescrevê-la
//! durante o arrasto faria o polegar recuar sob o dedo.

use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::widget::{CheckboxValue, SliderState};

/// Chamada pelo [`crate::sync_sections`].
pub(crate) fn sync(host: &mut dyn PanelHostInternal) {
    let Some(info) = crate::state_components::current_inspector_constraints() else {
        return;
    };
    for ((row, &forca), &ligada) in info
        .rows
        .iter()
        .zip(crate::ids::INSP_CONSTRAINT_FORCA.iter())
        .zip(crate::ids::INSP_CONSTRAINT_LIGADA.iter())
    {
        if !matches!(host.store().slider(forca), Some((SliderState::Dragging, _)))
            && let Some(InteractiveState::Slider { value, .. }) = host.store_mut().get_mut(forca)
        {
            *value = row.forca.clamp(0.0, 1.0);
        }
        if let Some(InteractiveState::Checkbox { value, .. }) = host.store_mut().get_mut(ligada) {
            *value = if row.ligada {
                CheckboxValue::Checked
            } else {
                CheckboxValue::Unchecked
            };
        }
    }
}
