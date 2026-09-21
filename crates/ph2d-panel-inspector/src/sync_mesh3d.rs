//! **A semente dos campos da secção LIVE MESH** — o catavento.
//!
//! ⛔⛔ **Ela nasce no MESMO commit que a secção, e isso é a lição de 19/09**: a secção do RAIO
//! shipou sem esta metade e o painel mostrou **quatro números plausíveis** (os de fábrica) sobre um
//! objecto autorado — *eles não parecem partidos, e o artista que escreva por cima de um deles
//! grava o default nos outros três*. Foi uma FOTO que a apanhou, com a suíte inteira verde.
//!
//! ⚠️ **De ARESTA, nunca por quadro**, e o `assado` fica FORA da assinatura: ele não semeia widget
//! nenhum (é só a queixa) e pô-lo ali faria a secção re-semear no quadro em que alguém assa — que é
//! exactamente o defeito que a aresta existe para não ter.

use std::hash::{Hash, Hasher};

use ph2d_editor_core::mesh3d_edits::InspectorMesh3dInfo;
use ph2d_editor_core::panel::PanelHostInternal;

use crate::state::InspectorState;

/// ⚠️ **Só o que SEMEIA um widget entra** — ver o cabeçalho sobre o `assado`.
fn assinatura(info: &InspectorMesh3dInfo) -> u64 {
    let mut h = std::hash::DefaultHasher::new();
    info.entity_bits.hash(&mut h);
    info.yaw.to_bits().hash(&mut h);
    info.pitch.to_bits().hash(&mut h);
    info.spin.to_bits().hash(&mut h);
    h.finish()
}

/// **A semente da secção** — chamada pelo [`crate::sync_sections`].
pub(crate) fn sync(
    host: &mut dyn PanelHostInternal,
    inspector_state: &mut InspectorState,
    entity_changed: bool,
) {
    let Some(info) = crate::state_components::current_inspector_mesh3d() else {
        inspector_state.last_mesh3d_sig = None;
        return;
    };
    let sig = assinatura(&info);
    if !entity_changed && inspector_state.last_mesh3d_sig == Some(sig) {
        return;
    }
    inspector_state.last_mesh3d_sig = Some(sig);
    let focus = host.store().focus_id();
    let drag = host.store().number_input_drag().map(|d| d.id);
    // ⚠️ **A ORDEM é a do [`super::populate_mesh3d::NUMEROS`]**, e é assim que ela não pode
    // divergir: os três ids são os mesmos três, na mesma ordem.
    // ⚠️⚠️ **E os dois ângulos entram em GRAUS**, pela porta que os converte — escrever os radianos
    // aqui poria a caixa a mostrar `3,14` para meia volta, e o artista escreveria por cima.
    for (id, v) in [
        (crate::ids::INSP_MESH3D_YAW, info.yaw_graus()),
        (crate::ids::INSP_MESH3D_PITCH, info.pitch_graus()),
        (crate::ids::INSP_MESH3D_SPIN, f64::from(info.spin)),
    ] {
        if focus == Some(id) || drag == Some(id) {
            continue; // a mão do artista ganha ao instantâneo
        }
        host.store_mut().set_number_value(id, v);
    }
}
