//! **A semente dos campos da secção WEAPON** — a arma do jogador.
//!
//! ⛔⛔ **Ela nasce no MESMO commit que a secção, e isso é a lição de 19/09**: a secção do RAIO
//! shipou sem esta metade e o painel mostrou **quatro números plausíveis** (os de fábrica) sobre um
//! olho autorado — *eles não parecem partidos, e o artista que escreva por cima de um deles grava o
//! default nos outros três*. Foi uma FOTO que a apanhou, com a suíte inteira verde.
//!
//! ⛔ **Nenhum gate de lei pode vê-la:** os da lei e do dreno entram **abaixo** do `WidgetStore`.
//! *A semente é a única metade da secção cujo sujeito é o widget.*
//!
//! # ⚠️ De ARESTA, nunca por quadro
//!
//! Reescrever sempre apagaria o que o artista está a digitar antes de o commit da shell chegar — a
//! lei que as seis irmãs já escrevem. A aresta é a **assinatura** do instantâneo, e o foco e o
//! arrasto ganham-lhe: *a mão do artista ganha ao instantâneo*.
//!
//! ⚠️ **A MUNIÇÃO VIVA fica de fora da assinatura**, e é deliberado: ela muda a cada tiro e não
//! semeia widget nenhum — pô-la ali faria a secção re-semear **por quadro**, que é exactamente o
//! defeito que a aresta existe para não ter. *É a mesma cerca que a leitura viva do raio tem.*

use std::hash::{Hash, Hasher};

use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::weapon_edits::InspectorWeaponInfo;

use crate::state::InspectorState;

/// ⚠️ **Só o que SEMEIA um widget entra** — ver o cabeçalho sobre a munição viva.
fn assinatura(info: &InspectorWeaponInfo) -> u64 {
    let mut h = std::hash::DefaultHasher::new();
    info.entity_bits.hash(&mut h);
    info.cooldown_ms.hash(&mut h);
    info.reload_ms.hash(&mut h);
    info.on_signal.hash(&mut h);
    info.ammo_counter.hash(&mut h);
    info.reload_on.hash(&mut h);
    info.on_fire.hash(&mut h);
    info.on_empty.hash(&mut h);
    info.on_reloaded.hash(&mut h);
    h.finish()
}

/// **A semente da secção** — chamada pelo [`crate::sync_sections`].
pub(crate) fn sync(
    host: &mut dyn PanelHostInternal,
    inspector_state: &mut InspectorState,
    entity_changed: bool,
) {
    let Some(info) = crate::state_components::current_inspector_weapon() else {
        inspector_state.last_weapon_sig = None;
        return;
    };
    let sig = assinatura(&info);
    if !entity_changed && inspector_state.last_weapon_sig == Some(sig) {
        return;
    }
    inspector_state.last_weapon_sig = Some(sig);
    let focus = host.store().focus_id();
    let drag = host.store().number_input_drag().map(|d| d.id);
    // ⚠️ **A ORDEM é a do [`super::populate_weapon::NUMEROS`]**, e é assim que ela não pode
    // divergir: os dois ids são os mesmos dois, na mesma ordem, e um campo novo tem de entrar nos
    // dois sítios.
    for (id, v) in [
        (crate::ids::INSP_WEAPON_COOLDOWN, info.cooldown_ms),
        (crate::ids::INSP_WEAPON_RELOAD_MS, info.reload_ms),
    ] {
        if focus == Some(id) || drag == Some(id) {
            continue; // a mão do artista ganha ao instantâneo
        }
        #[allow(clippy::cast_precision_loss)]
        host.store_mut().set_number_value(id, v as f64);
    }
    for (id, t) in [
        (crate::ids::INSP_WEAPON_ON_SIGNAL, info.on_signal.as_str()),
        (crate::ids::INSP_WEAPON_AMMO, info.ammo_counter.as_str()),
        (crate::ids::INSP_WEAPON_RELOAD_ON, info.reload_on.as_str()),
        (crate::ids::INSP_WEAPON_ON_FIRE, info.on_fire.as_str()),
        (crate::ids::INSP_WEAPON_ON_EMPTY, info.on_empty.as_str()),
        (
            crate::ids::INSP_WEAPON_ON_RELOADED,
            info.on_reloaded.as_str(),
        ),
    ] {
        crate::sync_text_field::escreve_texto(host, focus, id, t);
    }
}
