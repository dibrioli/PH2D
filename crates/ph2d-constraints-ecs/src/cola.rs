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
use ph2d_constraints::Pose2;
use ph2d_constraints_cola::{Cola, Entrada, Estado as EstadoCola, resolve};
use ph2d_ecs::canal::familia;
use ph2d_ecs::{AlvosDasFamilias, Locked, SimComponent, SimWorld, StableId, Transform, parent_world_transform};
use ph2d_preview_drive::{Driven, Driver, PreviewDrive};
use serde::{Deserialize, Serialize};

use crate::avalia::{local_de, pose};
use crate::ordem::indice;

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

struct Membro {
    e: Entity,
    agora: Transform,
    base: Transform,
    pai: Transform,
    motor: bool,
}

/// A pose de onde a cola parte: a autorada. Se o `Transform` é o que a cola escreveu no quadro passado, o autorado
/// está no ledger; se outra mão escreveu entretanto, é essa mão (a lei do `avalia::base`).
fn base(preview: &PreviewDrive, e: Entity, agora: Transform) -> (Transform, bool) {
    let bits = e.to_bits();
    match (preview.last_written(bits, Driver::ColaPose), preview.authored(bits, Driver::ColaPose)) {
        (Some(Driven::ColaPose(w)), Some(Driven::ColaPose(a))) if w == agora => (a, true),
        _ => (agora, false),
    }
}

/// ⭐ **O passo da cola.** Uma vez por quadro, antes de `evaluate`. Devolve quantas entidades moveu.
pub fn colar(sim: &mut SimWorld, preview: &mut PreviewDrive) -> usize {
    let colas: Vec<Cola> = sim
        .world()
        .iter_entities()
        .filter_map(|r| r.get::<Colas>())
        .flat_map(|c| c.items.iter().copied())
        .collect();
    let ja: BTreeSet<Entity> = preview.driven_by(Driver::ColaPose).into_iter().collect();
    if colas.is_empty() {
        for e in ja {
            preview.release_to_authored(sim, e, Driver::ColaPose);
        }
        sim.world_mut().remove_resource::<EstadosDaCola>();
        sim.world_mut().remove_resource::<Memoria>();
        return 0;
    }

    let idx = indice(sim.world());
    let memoria = sim.world_mut().remove_resource::<Memoria>().unwrap_or_default().0;
    let mut membros: BTreeMap<u64, Membro> = BTreeMap::new();
    let mut corpos = BTreeMap::new();
    let ids: BTreeSet<u64> = colas.iter().flat_map(|c| [c.a, c.b]).collect();
    for s in ids {
        let Some(&e) = idx.get(&StableId(s)) else { continue };
        let w = sim.world();
        let Some(&agora) = w.get::<Transform>(e) else { continue };
        let (base, escrito) = base(preview, e, agora);
        let motor = !escrito && preview.drives_other_than(e.to_bits(), Driver::ColaPose);
        let mao = !motor && memoria.get(&e).is_some_and(|t| *t != base);
        let pai = parent_world_transform(w, e);
        let condutor = motor || mao || w.get::<Locked>(e).is_some();
        let p = pose(&Transform::compose(pai, base));
        corpos.insert(s, Entrada { pose: p, condutor });
        membros.insert(s, Membro { e, agora, base, pai, motor });
    }

    let saida = resolve(&colas, &corpos);
    let mut estado_de: BTreeMap<u64, EstadoCola> = BTreeMap::new();
    let mut por_motor: BTreeSet<u64> = BTreeSet::new();
    for g in &saida.grupos {
        let motor = g.membros.iter().any(|m| membros.get(m).is_some_and(|x| x.motor));
        for &m in &g.membros {
            estado_de.insert(m, g.estado);
            if motor && g.estado == EstadoCola::Definido {
                por_motor.insert(m);
            }
        }
    }

    let mut movidas = 0;
    let mut conduzidas = BTreeSet::new();
    let mut nova_memoria = BTreeMap::new();
    for (s, m) in &membros {
        let mut doc = m.base;
        if let Some(&p) = saida.poses.get(s).filter(|_| !corpos[s].condutor) {
            let local = local_de(m.pai, m.base, corpos[s].pose, p);
            if por_motor.contains(s) {
                conduzidas.insert(m.e);
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
                && let Some(mut t) = sim.world_mut().get_mut::<Transform>(m.e)
            {
                *t = local;
            }
        }
        if !m.motor {
            nova_memoria.insert(m.e, doc);
        }
    }
    for e in ja.difference(&conduzidas) {
        preview.release_to_authored(sim, *e, Driver::ColaPose);
    }

    let estados = colas
        .iter()
        .filter_map(|c| Some(((c.a, c.id), *estado_de.get(&c.a)?)))
        .collect();
    let alvos: Vec<u64> = membros.values().map(|m| m.e.to_bits()).collect();
    let w = sim.world_mut();
    w.insert_resource(EstadosDaCola(estados));
    w.insert_resource(Memoria(nova_memoria));
    w.get_resource_or_insert_with(AlvosDasFamilias::default)
        .publicar(familia::CONSTRAINT, alvos);
    movidas
}
