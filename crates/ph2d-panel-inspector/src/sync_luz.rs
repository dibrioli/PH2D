//! **A semente dos campos da secção LUZ** (Luz 2D, W4) — sem ela o painel mostraria os valores de
//! FÁBRICA do `populate` sobre uma lâmpada autorada, e escrever por cima de um gravaria o default.
//!
//! A COR corre por quadro (o seletor escreve a amostra; a diferença vai ao barramento; fora do
//! seletor, a amostra é re-semeada do documento — é assim que um `Ctrl+Z` se vê nela).

use std::hash::{Hash, Hasher};

use ph2d_editor_core::luz_edits::InspectorLuzInfo;
use ph2d_editor_core::panel::PanelHostInternal;

use crate::state::InspectorState;

/// ⚠️ Só o que SEMEIA um widget entra — os avisos mudam por quadro e não escrevem campo nenhum.
fn assinatura(i: &InspectorLuzInfo) -> u64 {
    let mut h = std::hash::DefaultHasher::new();
    i.entity_bits.hash(&mut h);
    i.intensidade.to_bits().hash(&mut h);
    i.tamanho_m.to_bits().hash(&mut h);
    i.alcance_m.to_bits().hash(&mut h);
    h.finish()
}

fn cor(host: &mut dyn PanelHostInternal, i: &InspectorLuzInfo) {
    use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
    use ph2d_editor_core::luz_edits::LuzFieldEdit;
    let id = crate::ids::INSP_LUZ_COR;
    let [r, g, b] = i.cor_srgb;
    let gravada = [r, g, b, u8::MAX];
    if host.store().picker_target() == Some(id) {
        if let Some(e) = host.store().widget_color(id)
            && e[..3] != gravada[..3]
        {
            host.bus_mut().push(EditorAction::InspectorComponentEdit {
                entity_bits: i.entity_bits,
                edit: ComponentEdit::Luz(LuzFieldEdit::Cor([e[0], e[1], e[2]])),
            });
        }
    } else {
        host.store_mut().set_widget_color(id, gravada);
    }
}

/// **A semente da secção LUZ** — chamada pelo [`crate::sync_sections`].
pub(crate) fn sync(
    host: &mut dyn PanelHostInternal,
    inspector_state: &mut InspectorState,
    entity_changed: bool,
) {
    let Some(info) = crate::state_components::current_inspector_luz() else {
        inspector_state.last_luz_sig = None;
        return;
    };
    cor(host, &info);
    let sig = assinatura(&info);
    if !entity_changed && inspector_state.last_luz_sig == Some(sig) {
        return;
    }
    inspector_state.last_luz_sig = Some(sig);
    let focus = host.store().focus_id();
    let drag = host.store().number_input_drag().map(|d| d.id);
    let mut campos = vec![
        (
            crate::ids::INSP_LUZ_INTENSIDADE,
            f64::from(info.intensidade),
        ),
        (crate::ids::INSP_LUZ_TAMANHO, f64::from(info.tamanho_m)),
    ];
    // `∞` não se escreve numa caixa: desligado, o número fica o último que o artista usou.
    if info.alcance_m.is_finite() {
        campos.push((crate::ids::INSP_LUZ_ALCANCE, f64::from(info.alcance_m)));
    }
    for (id, v) in campos {
        if focus == Some(id) || drag == Some(id) {
            continue; // a mão do artista ganha ao instantâneo
        }
        host.store_mut().set_number_value(id, v);
    }
}

/// **A semente do cartão LIGHT OCCLUDER** — a opacidade do documento na caixa (sem ela, a caixa
/// mostraria a partida do `populate` sobre um oclusor autorado). Um número só, semeado todo o
/// quadro fora da mão do artista — é assim que um `Ctrl+Z` se vê nela.
pub(crate) fn sync_corpo(host: &mut dyn PanelHostInternal) {
    let Some(opacidade) =
        crate::state_components::current_inspector_corpo_luz().and_then(|i| i.oclusor)
    else {
        return;
    };
    let id = crate::ids::INSP_LUZ_OPACIDADE;
    let ocupada = host.store().focus_id() == Some(id)
        || host.store().number_input_drag().map(|d| d.id) == Some(id);
    if !ocupada {
        host.store_mut().set_number_value(id, f64::from(opacidade));
    }
}
