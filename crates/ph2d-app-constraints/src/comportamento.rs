//! ***Behaviours*, lado da família** (W5.4, `docs/BlenderConstraints/06_W5_resto.md` §3.2): o instante da
//! vista para os puros e, para cada Atrair, o deslocamento do simulador — re-simulado sobre a pose
//! ANIMADA do dono e do alvo (a mesma «cena no quadro» da Spring), então o scrub dá o mesmo bit.

use std::collections::{BTreeMap, BTreeSet};

use bevy_ecs::entity::Entity;
use ph2d_constraints::Kind;
use ph2d_constraints::comportamento::Perfil;
use ph2d_constraints_ecs::comportamento::Atracao;
use ph2d_constraints_ecs::{Comportamentos, Constraints, Lei};
use ph2d_ecs::{SimWorld, StableId};

use crate::relogio::Relogio;

/// O que cada item com estado simula: o Atrair (o deslocamento do dono) ou a mola do centro de uma
/// órbita (o atraso do centro atrás do alvo).
#[derive(Clone, Copy)]
enum Sim {
    Atrair(Atracao),
    Centro(Lei<2>),
}

/// Instala [`Comportamentos`] deste quadro.
pub(crate) fn preparar(sim: &mut SimWorld, relogio: Relogio<'_>) {
    let w = sim.world_mut();
    let itens: Vec<_> = w
        .iter_entities()
        .filter_map(|e| Some((e.id(), e.get::<Constraints>()?)))
        .flat_map(|(e, c)| {
            c.items.iter().filter_map(move |it| {
                let Kind::Comportamento(b) = it.kind else {
                    return None;
                };
                let s = match (b.perfil, b.segue) {
                    (Perfil::Atrair, _) => Sim::Atrair(Atracao::from(&b)),
                    (Perfil::Orbitar, Some(m)) => Sim::Centro(Lei {
                        duracao: m.duracao,
                        ressalto: m.ressalto,
                        angular: [false; 2],
                    }),
                    _ => return None,
                };
                Some((e.to_bits(), it.id, s, it.target))
            })
        })
        .collect();
    let mut cs = w.remove_resource::<Comportamentos>().unwrap_or_default();
    cs.t = relogio.t;
    cs.deslocamentos.clear();
    let vivos: BTreeSet<(u64, u32)> = itens.iter().map(|a| (a.0, a.1)).collect();
    cs.sims.retain(|k, _| vivos.contains(k));
    cs.historia.retain(|k, _| vivos.contains(k));
    let fps = relogio.doc.fps_display;
    // Os alvos por `StableId`, num índice só (a procura por um é linear no mundo).
    let alvos: BTreeSet<StableId> = itens.iter().filter_map(|a| a.3).collect();
    let idx: BTreeMap<StableId, u64> = if alvos.is_empty() {
        BTreeMap::new()
    } else {
        let mut q = w.query::<(Entity, &StableId)>();
        q.iter(w)
            .filter(|(_, s)| alvos.contains(s))
            .map(|(e, s)| (*s, e.to_bits()))
            .collect()
    };
    if fps <= 0.0 {
        w.insert_resource(cs);
        return;
    }
    // A pose animada de cada `(entidade, quadro)` uma vez por quadro: o alvo é partilhado.
    let mut poses: BTreeMap<(u64, i64), Option<[f64; 2]>> = BTreeMap::new();
    #[expect(clippy::cast_possible_truncation, reason = "um quadro cabe em i64")]
    let agora = (relogio.t * fps).floor() as i64;
    for (dono, id, lei, alvo) in itens {
        let Some(&alvo) = alvo.and_then(|s| idx.get(&s)) else {
            continue;
        };
        // Um alvo sem curva (arrastado à mão) não tem passado na timeline: a história guarda a pose
        // dele no quadro em que mudou — sem ela a mola re-simulava tudo com a pose de AGORA e colava.
        let animado = relogio
            .doc
            .bindings()
            .iter()
            .any(|x| x.entity == alvo && !x.missing && x.prop.as_sprite_transform().is_some());
        let vivo = ph2d_ecs::world_transform(w, Entity::from_bits(alvo))
            .map(|t| [f64::from(t.translation.x), f64::from(t.translation.y)]);
        let historia = cs.historia.entry((dono, id)).or_default();
        if animado {
            historia.clear();
        } else if let Some(v) = vivo {
            let _ = historia.split_off(&(agora + 1));
            if historia.range(..=agora).next_back().map(|(_, x)| *x) != Some(v) {
                historia.insert(agora, v);
            }
        }
        let historia = &cs.historia[&(dono, id)];
        let mut em = |b: u64, q: i64| {
            if b == alvo && !animado {
                return historia
                    .range(..=q)
                    .next_back()
                    .or_else(|| historia.iter().next())
                    .map(|(_, x)| *x)
                    .or(vivo);
            }
            *poses.entry((b, q)).or_insert_with(|| {
                #[expect(clippy::cast_precision_loss, reason = "um quadro")]
                let t = q as f64 / fps;
                ph2d_timeline::world_pose_at(w, relogio.doc, b, t)
                    .map(|p| [f64::from(p.translation.x), f64::from(p.translation.y)])
            })
        };
        let s = cs.sims.entry((dono, id)).or_default();
        let q = relogio.t * fps;
        let d = match lei {
            Sim::Atrair(l) => {
                let mut entrada = |k: i64| match (em(alvo, k), em(dono, k)) {
                    (Some(a), Some(d)) => [a[0] - d[0], a[1] - d[1]],
                    _ => [0.0; 2],
                };
                s.corre_no_laco(q, relogio.laco, l, fps, &mut entrada)
            }
            Sim::Centro(l) => {
                let mut entrada = |k: i64| em(alvo, k).unwrap_or([0.0; 2]);
                s.corre_no_laco(q, relogio.laco, l, fps, &mut entrada)
            }
        };
        cs.deslocamentos.insert((dono, id), d);
    }
    w.insert_resource(cs);
}
