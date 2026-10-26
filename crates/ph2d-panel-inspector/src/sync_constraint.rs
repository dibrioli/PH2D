//! **A semente do cartão CONSTRAINTS** — todo o quadro, a força, o «Enabled» e os números de cada
//! linha vêm do instantâneo (é assim que um `Ctrl+Z` se vê neles), MENOS o controlo que a mão segura:
//! reescrevê-lo durante o arrasto ou a escrita faria o valor recuar sob o dedo.

use ph2d_editor_core::constraint_edits::Unidade;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::project::ProjectSettings;
use ph2d_editor_core::widget::{CheckboxValue, SliderState, Unit};

/// Chamada pelo [`crate::sync_sections`].
pub(crate) fn sync(host: &mut dyn PanelHostInternal) {
    let Some(info) = crate::state_components::current_inspector_constraints() else {
        return;
    };
    for (k, ((row, &forca), &ligada)) in info
        .rows
        .iter()
        .zip(crate::ids::INSP_CONSTRAINT_FORCA.iter())
        .zip(crate::ids::INSP_CONSTRAINT_LIGADA.iter())
        .enumerate()
    {
        if !matches!(host.store().slider(forca), Some((SliderState::Dragging, _)))
            && let Some(InteractiveState::Slider { value, .. }) = host.store_mut().get_mut(forca)
        {
            *value = row.forca.clamp(0.0, 1.0);
        }
        for (n, tabela) in row
            .numeros
            .iter()
            .zip(crate::ids::INSP_CONSTRAINT_NUMERO.iter())
        {
            let id = tabela[k];
            if host.store().focus_id() != Some(id)
                && host.store().number_input_drag().map(|d| d.id) != Some(id)
            {
                let v = no_ecra(n.unidade, n.valor, host.project());
                host.store_mut().set_number_value(id, v);
            }
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

/// ⭐ **O número do cartão na unidade que o PROJECTO mostra** (Settings → px/metros e
/// graus/radianos, como o Transform): a família fala metros e graus; o ecrã, o que o artista
/// escolheu. [`do_ecra`] é a volta.
pub(crate) fn no_ecra(u: Unidade, v: f32, p: &ProjectSettings) -> f64 {
    match u {
        Unidade::Metros => p
            .display_unit
            .from_meters_f64(f64::from(v), p.pixels_per_meter),
        Unidade::Graus => p.display_angle.from_radians_f64(f64::from(v).to_radians()),
        Unidade::Porcento => f64::from(v) * 100.0, // LITERAL-PX-OK: fracção → por cento (conversão, não UI)
        Unidade::Contagem | Unidade::Segundos => f64::from(v),
    }
}

/// O valor escrito no ecrã de volta a metros/graus.
#[expect(clippy::cast_possible_truncation, reason = "o campo do cartão é f32")]
pub(crate) fn do_ecra(u: Unidade, v: f64, p: &ProjectSettings) -> f32 {
    match u {
        Unidade::Metros => p.display_unit.to_meters_f64(v, p.pixels_per_meter) as f32,
        Unidade::Graus => p.display_angle.to_radians_f64(v).to_degrees() as f32,
        Unidade::Porcento => (v / 100.0) as f32, // LITERAL-PX-OK: por cento → fracção (conversão, não UI)
        Unidade::Contagem | Unidade::Segundos => v as f32,
    }
}

/// O sufixo e o passo do arrasto na unidade do ecrã: 1 px ou 0,01 m; um grau na unidade activa; 1 %;
/// um osso (sem sufixo).
pub(crate) fn unidade_e_passo(u: Unidade) -> (Option<Unit>, f64) {
    use ph2d_editor_core::project::{DisplayAngle, DisplayUnit};
    match u {
        Unidade::Metros => match crate::state::current_display_unit() {
            DisplayUnit::Pixels => (Some(Unit::Px), 1.0),
            DisplayUnit::Meters => (Some(Unit::Meters), 0.01), // LITERAL-PX-OK: passo em metros
        },
        Unidade::Graus => {
            let a = crate::state::current_display_angle();
            let unit = match a {
                DisplayAngle::Degrees => Unit::Degrees,
                DisplayAngle::Radians => Unit::Radians,
            };
            (Some(unit), f64::from(a.from_radians(1.0_f32.to_radians())))
        }
        Unidade::Porcento => (Some(Unit::Percent), 1.0),
        Unidade::Contagem => (None, 1.0),
        Unidade::Segundos => (Some(Unit::Seconds), 0.01), // LITERAL-PX-OK: passo em segundos (centésimo)
    }
}
