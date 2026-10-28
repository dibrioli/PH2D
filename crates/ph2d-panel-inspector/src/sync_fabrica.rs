//! **A semente dos números de FACTORY · LIFECYCLE · PROJECTILE.**
//!
//! ⛔ As três secções nasceram SEM semente: as caixas mostravam o valor de partida do `populate`
//! sobre qualquer objecto (rows WRITE-ONLY), e com o projecto em px o `12 m/s` de partida lia-se
//! `12 px/s`. Aresta = a ASSINATURA do instantâneo (a lei das irmãs); a mão do artista ganha.

use std::hash::{Hash, Hasher};

use ph2d_editor_core::panel::PanelHostInternal;

use crate::ids as i;
use crate::state::InspectorState;

/// Os números das três secções, pela ordem do `populate`.
fn numeros() -> Vec<(ph2d_a11y::NodeId, f64)> {
    let mut v = Vec::new();
    if let Some(info) = crate::state_components::current_inspector_factory() {
        if let Some(f) = info.factory.as_ref() {
            #[allow(clippy::cast_precision_loss)]
            let semente = f.seed as f64;
            v.extend([
                (i::INSP_FACTORY_AREA_W, f64::from(f.area[0])),
                (i::INSP_FACTORY_AREA_H, f64::from(f.area[1])),
                (i::INSP_FACTORY_BURST, f64::from(f.burst)),
                (i::INSP_FACTORY_ALIVE_MAX, f64::from(f.alive_max)),
                (i::INSP_FACTORY_TOTAL_MAX, f64::from(f.total_max)),
                (i::INSP_FACTORY_SEED, semente),
            ]);
        }
        if let Some(l) = info.lifecycle.as_ref() {
            if let Some(s) = l.lifetime_s {
                v.push((i::INSP_LIFE_SECONDS, f64::from(s)));
            }
            if let Some(m) = l.outside_margin {
                v.push((i::INSP_LIFE_OUTSIDE_MARGIN, f64::from(m)));
            }
        }
    }
    if let Some(p) = crate::state_components::current_inspector_projectile() {
        v.extend([
            (i::INSP_PJ_SPEED, f64::from(p.initial_speed)),
            (i::INSP_PJ_ACCEL, f64::from(p.acceleration)),
            (i::INSP_PJ_MAX_SPEED, f64::from(p.max_speed)),
            (i::INSP_PJ_GRAVITY, f64::from(p.gravity)),
            (i::INSP_PJ_BOUNCINESS, f64::from(p.bounciness)),
            (i::INSP_PJ_MAX_BOUNCES, f64::from(p.max_bounces)),
            (i::INSP_PJ_RANGE, f64::from(p.range)),
            (i::INSP_PJ_HOMING_ACCEL, f64::from(p.homing_accel)),
        ]);
    }
    v
}

/// Chamada pelo [`crate::sync_sections`].
pub(crate) fn sync(
    host: &mut dyn PanelHostInternal,
    inspector_state: &mut InspectorState,
    entity_changed: bool,
) {
    let v = numeros();
    let mut h = std::hash::DefaultHasher::new();
    for (id, n) in &v {
        (id, n.to_bits()).hash(&mut h);
    }
    let sig = h.finish();
    if !entity_changed && inspector_state.last_fabrica_sig == Some(sig) {
        return;
    }
    inspector_state.last_fabrica_sig = Some(sig);
    let focus = host.store().focus_id();
    let drag = host.store().number_input_drag().map(|d| d.id);
    for (id, n) in v {
        if focus != Some(id) && drag != Some(id) {
            crate::unidade_mundo::semeia(host, id, n);
        }
    }
}
