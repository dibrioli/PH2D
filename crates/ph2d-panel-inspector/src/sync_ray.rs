//! **A semente dos campos da secção RAY SENSOR** (suplente #21).
//!
//! ⛔⛔⛔ **ESTA LINHA FALTAVA, e quem a apanhou foi uma FOTO** (19/09, a cena `PH2D_RAY_SMOKE=1`):
//! o painel mostrava `Direction 0 / −1` e `Reach 1 m` — os valores de **fábrica** do
//! [`super::populate_ray`] — sobre um olho autorado a `(1, 0)` com alcance `6`, **com a linha
//! desenhada a 6 m no canvas ao lado e a leitura viva a dizer `Sees Caixa, at 3,57 m`**.
//!
//! ⚠️⚠️ *Quatro números plausíveis são a pior forma deste defeito:* eles não parecem partidos, e o
//! artista que escreva por cima de um deles **grava o default nos outros três**. É a mesma família
//! que a secção do ÁUDIO e a da CÂMERA pagaram em 10/09 e a da VIGIA DO CONTADOR em 16/09 — e a
//! terceira vez que uma FOTO a apanha depois de a suíte inteira ficar verde.
//!
//! ⛔ **Nenhum gate desta linha podia vê-la:** os da W4 medem a porta da queixa e o dreno das
//! edições, e os dois entram **abaixo** do `WidgetStore`. *A semente é a única metade da secção
//! cujo sujeito é o widget.*
//!
//! # ⚠️ De ARESTA, nunca por quadro
//!
//! Reescrever sempre apagaria o que o artista está a digitar antes de o commit da shell chegar — a
//! lei que as cinco irmãs já escrevem. A aresta é a **assinatura** do instantâneo, e o foco e o
//! arrasto ganham-lhe: *a mão do artista ganha ao instantâneo*.
//!
//! ⚠️ **A leitura VIVA fica de fora da assinatura**, e é deliberado: ela muda a cada tique (`Sees
//! Caixa, at 3,57 m`) e não semeia widget nenhum — pô-la ali faria a secção re-semear **por
//! quadro**, que é exactamente o defeito que a aresta existe para não ter.

use std::hash::{Hash, Hasher};

use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::ray_edits::InspectorRayInfo;

use crate::state::InspectorState;

/// ⚠️ **Só o que SEMEIA um widget entra** — ver o cabeçalho sobre a leitura viva.
fn assinatura(info: &InspectorRayInfo) -> u64 {
    let mut h = std::hash::DefaultHasher::new();
    info.entity_bits.hash(&mut h);
    for v in [
        info.origin_x,
        info.origin_y,
        info.dir_x,
        info.dir_y,
        info.reach,
    ] {
        v.to_bits().hash(&mut h);
    }
    info.layer.hash(&mut h);
    info.on_enter.hash(&mut h);
    info.on_exit.hash(&mut h);
    h.finish()
}

/// **A semente da secção** — chamada pelo [`crate::sync_sections`].
pub(crate) fn sync(
    host: &mut dyn PanelHostInternal,
    inspector_state: &mut InspectorState,
    entity_changed: bool,
) {
    let Some(info) = crate::state_components::current_inspector_ray() else {
        inspector_state.last_ray_sig = None;
        return;
    };
    let sig = assinatura(&info);
    if !entity_changed && inspector_state.last_ray_sig == Some(sig) {
        return;
    }
    inspector_state.last_ray_sig = Some(sig);
    let focus = host.store().focus_id();
    let drag = host.store().number_input_drag().map(|d| d.id);
    // ⚠️ **A ORDEM é a do [`super::populate_ray::NUMEROS`]**, e é assim que ela não pode divergir:
    // os seis ids são os mesmos seis, na mesma ordem, e um campo novo tem de entrar nos dois.
    for (id, v) in [
        (crate::ids::INSP_RAY_ORIGIN_X, info.origin_x),
        (crate::ids::INSP_RAY_ORIGIN_Y, info.origin_y),
        (crate::ids::INSP_RAY_DIR_X, info.dir_x),
        (crate::ids::INSP_RAY_DIR_Y, info.dir_y),
        (crate::ids::INSP_RAY_REACH, info.reach),
        #[allow(clippy::cast_precision_loss)]
        (crate::ids::INSP_RAY_LAYER, info.layer as f32),
    ] {
        if focus == Some(id) || drag == Some(id) {
            continue; // a mão do artista ganha ao instantâneo
        }
        host.store_mut().set_number_value(id, f64::from(v));
    }
    for (id, t) in [
        (crate::ids::INSP_RAY_ON_ENTER, info.on_enter.as_str()),
        (crate::ids::INSP_RAY_ON_EXIT, info.on_exit.as_str()),
    ] {
        crate::sync_text_field::escreve_texto(host, focus, id, t);
    }
}
