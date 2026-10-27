//! A cola no mundo (`docs/BlenderConstraints/05_W5_diferenciais.md` §2.2): o componente [`Colas`] mora no objecto
//! `a` de cada cola e o passo [`colar`] corre no quadro ANTES da pilha, que parte do que ele deixa.
//!
//! Quem CONDUZ um grupo: o membro cujo documento mudou desde o quadro passado (a mão: gizmo, Inspector), o que um
//! motor conduz (curva, script, solver) e o que tem cadeado (`Locked` = âncora, o «chão» do FreeCAD). Gesto ⇒ os
//! parceiros escrevem-se no DOCUMENTO (o undo por diff junta-os ao passo do gesto); motor ⇒ pelo `PreviewDrive`
//! (`Driver::ColaPose`), nunca no documento.

use std::collections::{BTreeMap, BTreeSet};

use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Resource;
use ph2d_constraints_cola::{Cache, Cola, Entrada, Estado as EstadoCola, Pose2, resolve_com};
use ph2d_ecs::canal::familia;
use ph2d_ecs::{
    AlvosDasFamilias, Locked, SimComponent, SimWorld, StableId, Transform,
    parent_world_transform_into,
};
use ph2d_preview_drive::{Driven, Driver, PreviewDrive};
use serde::{Deserialize, Serialize};

use crate::avalia::{local_de, pose};
use crate::ordem::{cada, indice};

/// O nome gravado do componente.
pub const COLAS_TYPE_NAME: &str = "ph2d::constraints::Colas";

/// As colas de que este objecto é o `a` (a cola é simétrica; o dono é só onde ela mora).
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Colas {
    pub items: Vec<Cola>,
    pub next_id: u32,
}

impl SimComponent for Colas {}

impl Colas {
    /// Acrescenta a cola que `f` faz com o id novo; `None` (espaço colapsado) não gasta o id.
    pub fn push(&mut self, f: impl FnOnce(u32) -> Option<Cola>) -> Option<u32> {
        let id = self.next_id.max(1);
        self.items.push(f(id)?);
        self.next_id = id + 1;
        Some(id)
    }

    pub fn remove(&mut self, id: u32) -> bool {
        let n = self.items.len();
        self.items.retain(|c| c.id != id);
        self.items.len() != n
    }
}

/// O estado de cada cola neste quadro, pela chave `(StableId do a, id)` — o cartão dos DOIS objectos lê-o.
#[derive(Resource, Default, Debug)]
pub struct EstadosDaCola(pub BTreeMap<(u64, u32), EstadoCola>);

/// O documento de cada membro no fim do quadro passado: o que mudar entretanto foi uma mão.
#[derive(Resource, Default, Debug)]
struct Memoria(BTreeMap<Entity, Transform>);

/// A estrutura dos grupos de um quadro para o outro (só depende das entradas: o mesmo bit sem ela).
#[derive(Resource, Default)]
struct CacheDaCola(Cache);

struct Membro {
    e: Entity,
    agora: Transform,
    base: Transform,
    pai: Transform,
    /// A pose mundial de entrada.
    pose: Pose2,
    motor: bool,
    estado: Option<EstadoCola>,
    por_motor: bool,
}

/// O membro `s` (os membros vão pela ordem do `StableId`).
fn pos(membros: &[(u64, Membro)], s: u64) -> Option<usize> {
    membros.binary_search_by_key(&s, |m| m.0).ok()
}

/// A pose de onde a cola parte: a autorada. Se o `Transform` é o que a cola escreveu no quadro passado, o autorado
/// está no ledger; se outra mão escreveu entretanto, é essa mão (a lei do `avalia::base`).
fn base(preview: &PreviewDrive, e: Entity, agora: Transform) -> (Transform, bool) {
    let bits = e.to_bits();
    match preview.written_and_authored(bits, Driver::ColaPose) {
        Some((Driven::ColaPose(w), Driven::ColaPose(a))) if w == agora => (a, true),
        _ => (agora, false),
    }
}

/// ⭐ **O passo da cola.** Uma vez por quadro, antes de `evaluate`. Devolve quantas entidades moveu.
pub fn colar(sim: &mut SimWorld, preview: &mut PreviewDrive) -> usize {
    let colas: Vec<Cola> = cada::<Colas>(sim.world())
        .into_iter()
        .flat_map(|(_, c)| c.items.iter().copied())
        .collect();
    let ja: BTreeSet<Entity> = preview.driven_by(Driver::ColaPose).into_iter().collect();
    if colas.is_empty() {
        for e in ja {
            preview.release_to_authored(sim, e, Driver::ColaPose);
        }
        let w = sim.world_mut();
        w.remove_resource::<EstadosDaCola>();
        w.remove_resource::<Memoria>();
        w.remove_resource::<CacheDaCola>();
        return 0;
    }

    let idx = indice(sim.world());
    let memoria = sim
        .world_mut()
        .remove_resource::<Memoria>()
        .unwrap_or_default()
        .0;
    let mut ids: Vec<u64> = colas.iter().flat_map(|c| [c.a, c.b]).collect();
    ids.sort_unstable();
    ids.dedup();
    let mut membros: Vec<(u64, Membro)> = Vec::with_capacity(ids.len());
    let mut corpos = Vec::with_capacity(ids.len());
    let mut buf = Vec::new();
    let w = sim.world();
    let mut tf = w.try_query::<&Transform>();
    let cadeado = w.component_id::<Locked>();
    for s in ids {
        let Some(&e) = idx.get(&StableId(s)) else {
            continue;
        };
        let Some(&agora) = tf.as_mut().and_then(|q| q.get(w, e).ok()) else {
            continue;
        };
        let (base, escrito) = base(preview, e, agora);
        let motor = !escrito && preview.drives_other_than(e.to_bits(), Driver::ColaPose);
        let mao = !motor && memoria.get(&e).is_some_and(|t| *t != base);
        let pai = parent_world_transform_into(w, e, &mut buf);
        let condutor = motor || mao || cadeado.is_some_and(|id| w.entity(e).contains_id(id));
        let p = pose(&Transform::compose(pai, base));
        corpos.push((
            s,
            Entrada {
                pose: p,
                condutor,
                cede: mao,
            },
        ));
        membros.push((
            s,
            Membro {
                e,
                agora,
                base,
                pai,
                pose: p,
                motor,
                estado: None,
                por_motor: false,
            },
        ));
    }
    let corpos: BTreeMap<u64, Entrada> = corpos.into_iter().collect();

    let mut cache = sim
        .world_mut()
        .remove_resource::<CacheDaCola>()
        .unwrap_or_default();
    let saida = resolve_com(&mut cache.0, &colas, &corpos);
    sim.world_mut().insert_resource(cache);
    for g in &saida.grupos {
        let motor = g
            .membros
            .iter()
            .any(|&m| pos(&membros, m).is_some_and(|i| membros[i].1.motor));
        for &m in &g.membros {
            if let Some(i) = pos(&membros, m) {
                membros[i].1.estado = Some(g.estado);
                membros[i].1.por_motor = motor && g.estado == EstadoCola::Definido;
            }
        }
    }

    let mut movidas = 0;
    let mut conduzidas = Vec::new();
    let mut nova_memoria = Vec::with_capacity(membros.len());
    let mut tf = sim.world_mut().query::<&mut Transform>();
    // As poses da saída e os membros vão ambos pela ordem do `StableId`: percorrem-se juntos.
    let mut poses = saida.poses.iter().peekable();
    for (s, m) in &membros {
        let mut doc = m.base;
        while poses.next_if(|(k, _)| *k < s).is_some() {}
        let nova = poses.next_if(|(k, _)| *k == s).map(|(_, p)| *p);
        // O condutor sai com a sua pose ao bit; a mão que CEDEU sai corrigida e reescreve-se também.
        if let Some(p) = nova.filter(|&p| p != m.pose) {
            let local = local_de(m.pai, m.base, m.pose, p);
            if m.por_motor {
                conduzidas.push(m.e);
                if local == m.agora {
                    preview.still_driving(m.e, Driver::ColaPose);
                } else {
                    preview.driven(m.e, Driven::ColaPose(m.agora), Driven::ColaPose(local));
                    movidas += 1;
                }
            } else {
                doc = local;
                if local != m.agora {
                    movidas += 1;
                }
            }
            if local != m.agora
                && let Ok(mut t) = tf.get_mut(sim.world_mut(), m.e)
            {
                *t = local;
            }
        }
        if !m.motor {
            nova_memoria.push((m.e, doc));
        }
    }
    let conduzidas: BTreeSet<Entity> = conduzidas.into_iter().collect();
    for e in ja.difference(&conduzidas) {
        preview.release_to_authored(sim, *e, Driver::ColaPose);
    }

    let estados = colas
        .iter()
        .filter_map(|c| Some(((c.a, c.id), membros[pos(&membros, c.a)?].1.estado?)))
        .collect();
    let alvos: Vec<u64> = membros.iter().map(|(_, m)| m.e.to_bits()).collect();
    let w = sim.world_mut();
    w.insert_resource(EstadosDaCola(estados));
    w.insert_resource(Memoria(nova_memoria.into_iter().collect()));
    w.get_resource_or_insert_with(AlvosDasFamilias::default)
        .publicar(familia::CONSTRAINT, alvos);
    movidas
}
