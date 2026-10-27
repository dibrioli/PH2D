//! ***Behaviours*, lado da família** (W5.4, `docs/BlenderConstraints/06_W5_resto.md` §3.2): o instante da
//! vista para os puros e, para cada Atrair, o deslocamento do simulador — re-simulado sobre a pose
//! ANIMADA do dono e do alvo (a mesma «cena no quadro» da Spring), então o scrub dá o mesmo bit.

use std::collections::{BTreeMap, BTreeSet};

use bevy_ecs::entity::Entity;
use ph2d_constraints::Kind;
use ph2d_constraints::comportamento::Perfil;
use ph2d_constraints_ecs::comportamento::Atracao;
use ph2d_constraints_ecs::{Comportamentos, Constraints};
use ph2d_ecs::{SimWorld, StableId};

use crate::relogio::Relogio;

/// Instala [`Comportamentos`] deste quadro.
pub(crate) fn preparar(sim: &mut SimWorld, relogio: Relogio<'_>) {
    let w = sim.world_mut();
    let atrair: Vec<_> = w
        .iter_entities()
        .filter_map(|e| Some((e.id(), e.get::<Constraints>()?)))
        .flat_map(|(e, c)| {
            c.items.iter().filter_map(move |it| match it.kind {
                Kind::Comportamento(b) if b.perfil == Perfil::Atrair => {
                    Some((e.to_bits(), it.id, Atracao::from(&b), it.target))
                }
                _ => None,
            })
        })
        .collect();
    let mut cs = w.remove_resource::<Comportamentos>().unwrap_or_default();
    cs.t = relogio.t;
    cs.deslocamentos.clear();
    let vivos: BTreeSet<(u64, u32)> = atrair.iter().map(|a| (a.0, a.1)).collect();
    cs.sims.retain(|k, _| vivos.contains(k));
    let fps = relogio.doc.fps_display;
    // Os alvos por `StableId`, num índice só (a procura por um é linear no mundo).
    let alvos: BTreeSet<StableId> = atrair.iter().filter_map(|a| a.3).collect();
    let idx: BTreeMap<StableId, u64> = if alvos.is_empty() {
        BTreeMap::new()
    } else {
        let mut q = w.query::<(Entity, &StableId)>();
        q.iter(w)
            .filter(|(_, s)| alvos.contains(s))
            .map(|(e, s)| (*s, e.to_bits()))
            .collect()
    };
    // A pose animada de cada `(entidade, quadro)` uma vez por quadro: o alvo é partilhado.
    let mut poses: BTreeMap<(u64, i64), Option<[f64; 2]>> = BTreeMap::new();
    if fps > 0.0 {
        for (dono, id, lei, alvo) in atrair {
            let Some(&alvo) = alvo.and_then(|s| idx.get(&s)) else {
                continue;
            };
            let mut entrada = |q: i64| {
                let mut em = |b: u64| {
                    *poses.entry((b, q)).or_insert_with(|| {
                        #[expect(clippy::cast_precision_loss, reason = "um quadro")]
                        let t = q as f64 / fps;
                        ph2d_timeline::world_pose_at(w, relogio.doc, b, t)
                            .map(|p| [f64::from(p.translation.x), f64::from(p.translation.y)])
                    })
                };
                match (em(alvo), em(dono)) {
                    (Some(a), Some(d)) => [a[0] - d[0], a[1] - d[1]],
                    _ => [0.0; 2],
                }
            };
            let sim = cs.sims.entry((dono, id)).or_default();
            let d = sim.corre_no_laco(relogio.t * fps, relogio.laco, lei, fps, &mut entrada);
            cs.deslocamentos.insert((dono, id), d);
        }
    }
    w.insert_resource(cs);
}
