//! **A semente dos campos da secção SCRIPT** (TOP-20 #16, W3).
//!
//! ⚠️ **Irmão de [`crate::sync_sections`] por CAP de FICHEIRO**, como o do cérebro.
//!
//! ⚠️ **De ARESTA, nunca por quadro** — reescrever sempre apagaria o que o artista está a digitar
//! antes de o commit da shell chegar. A aresta aqui é a ASSINATURA do instantâneo: a entidade, o
//! ficheiro e cada linha resolvida. Ela muda quando o artista troca de objecto, quando o ficheiro é
//! gravado com outras declarações, e quando um `Reset` devolve uma linha ao default — os três casos
//! em que o campo tem de mostrar outra coisa.
//!
//! ⚠️ **As caixas semeiam-se em TODO quadro**: um clique numa caixa é um commit instantâneo, e não
//! há texto a meio para apagar (a lei que a secção da câmera escreve).

use std::hash::{Hash, Hasher};

use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::script_edits::{InspectorScriptInfo, InspectorScriptValue as V};
use ph2d_editor_core::widget::CheckboxValue;

use crate::state::InspectorState;

/// A assinatura do instantâneo — a aresta da semente.
fn assinatura(info: &InspectorScriptInfo) -> u64 {
    let mut h = std::hash::DefaultHasher::new();
    info.entity_bits.hash(&mut h);
    info.source.hash(&mut h);
    for p in &info.props {
        p.name.hash(&mut h);
        p.own.hash(&mut h);
        match &p.value {
            V::Number(n) => n.to_bits().hash(&mut h),
            V::Bool(b) => b.hash(&mut h),
            V::Text(t) => t.hash(&mut h),
        }
        for x in [p.min, p.max, p.step] {
            x.map(f64::to_bits).hash(&mut h);
        }
    }
    h.finish()
}

/// **A semente da secção** — chamada pelo [`crate::sync_sections`].
pub(crate) fn sync(
    host: &mut dyn PanelHostInternal,
    inspector_state: &mut InspectorState,
    entity_changed: bool,
) {
    let Some(info) = crate::state_components::current_inspector_script() else {
        inspector_state.last_script_sig = None;
        return;
    };
    for (p, &id) in info.props.iter().zip(crate::ids::INSP_SCRIPT_BOOL.iter()) {
        if let (V::Bool(on), Some(InteractiveState::Checkbox { value, .. })) =
            (&p.value, host.store_mut().get_mut(id))
        {
            *value = if *on {
                CheckboxValue::Checked
            } else {
                CheckboxValue::Unchecked
            };
        }
    }
    let sig = assinatura(&info);
    if !entity_changed && inspector_state.last_script_sig == Some(sig) {
        return;
    }
    inspector_state.last_script_sig = Some(sig);
    let focus = host.store().focus_id();
    let drag = host.store().number_input_drag().map(|d| d.id);
    escreve_texto(host, focus, crate::ids::INSP_SCRIPT_SOURCE, &info.source);
    for (i, p) in info.props.iter().enumerate() {
        match &p.value {
            V::Number(v) => {
                let Some(&id) = crate::ids::INSP_SCRIPT_NUM.get(i) else {
                    break;
                };
                faixa(host, id, p.min, p.max, p.step);
                if focus != Some(id) && drag != Some(id) {
                    host.store_mut().set_number_value(id, *v);
                }
            }
            V::Text(t) => {
                if let Some(&id) = crate::ids::INSP_SCRIPT_TEXT.get(i) {
                    escreve_texto(host, focus, id, t);
                }
            }
            V::Bool(_) => {}
        }
    }
}

/// **A faixa de um campo é a PISTA do script** (Q6 do oráculo: ela manda no que se escreve e
/// arrasta, e não no que está gravado).
///
/// ⚠️ **Sem as duas pontas, o campo é uma caixa SEM intervalo** — e o arrasto proporcional sobre um
/// intervalo que não termina não tem sentido; o `rate` dá-lhe uma escala calibrada (a combinação
/// que o `set_number_drag_rate` documenta como a certa).
fn faixa(
    host: &mut dyn PanelHostInternal,
    id: ph2d_a11y::NodeId,
    min: Option<f64>,
    max: Option<f64>,
    step: Option<f64>,
) {
    let passo = step.unwrap_or(1.0); // LITERAL-PX-OK: a shell semeia sempre; isto é a rede
    let store = host.store_mut();
    store.set_number_range(id, min.unwrap_or(f64::MIN), max.unwrap_or(f64::MAX), passo);
    if min.is_some() && max.is_some() {
        store.clear_number_drag_rate(id);
    } else {
        store.set_number_drag_rate(id, passo);
    }
}

/// Escreve um campo de texto, **menos o que está em FOCO** — ele é do dedo.
fn escreve_texto(
    host: &mut dyn PanelHostInternal,
    focus: Option<ph2d_a11y::NodeId>,
    id: ph2d_a11y::NodeId,
    value: &str,
) {
    if focus == Some(id) {
        return;
    }
    if let Some(InteractiveState::TextInput {
        text,
        caret,
        selection_anchor,
        ..
    }) = host.store_mut().get_mut(id)
    {
        text.clear();
        text.push_str(value);
        *caret = text.len();
        *selection_anchor = None;
    }
}
