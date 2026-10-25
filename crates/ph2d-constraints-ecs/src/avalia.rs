//! A fase: avalia cada pilha a partir da pose AUTORADA e escreve o resultado pelo `PreviewDrive`.

use std::collections::BTreeMap;

use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Resource;
use ph2d_constraints::{Ctx, Inerte, Kind, Pose2};
use ph2d_ecs::{SimWorld, StableId, Transform, parent_world_transform, world_transform};
use ph2d_physics_ecs::{BodyKind, RigidBody};
use ph2d_preview_drive::{Driven, Driver, PreviewDrive};

use crate::Constraints;
use crate::ordem::{indice, ordem};

/// ⭐ **A porta única de «isto faz alguma coisa?»** — o cartão, o gizmo e os gates leem isto.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Estado {
    Activa,
    Desligada,
    /// Força 0.
    SemForca,
    SemAlvo,
    /// O alvo é o próprio dono.
    AlvoSiMesmo,
    /// A pose do alvo depende desta (directa ou indirectamente): não se avalia.
    Ciclo,
    /// Um corpo dinâmico é da física — use uma junta.
    CorpoDinamico,
    /// O pai (ou o do alvo) tem escala 0: não há espaço local.
    EspacoColapsado,
}

/// As entidades cuja pose VISTA passa a ser a autorada neste quadro (largar um Child Of sem salto):
/// a fase não as conduz uma vez, o `PreviewDrive::settle` esquece-as, e o quadro seguinte parte do
/// que se via. Consumido pela [`evaluate`].
#[derive(Resource, Default, Debug)]
pub struct Adotar(pub std::collections::BTreeSet<u64>);

/// O estado de cada item no último quadro, por `(Entity::to_bits, Item::id)`.
#[derive(Resource, Default, Debug)]
pub struct Estados(pub BTreeMap<(u64, u32), Estado>);

impl Estados {
    #[must_use]
    pub fn of(&self, entity: Entity, item: u32) -> Option<Estado> {
        self.0.get(&(entity.to_bits(), item)).copied()
    }
}

fn pose(t: &Transform) -> Pose2 {
    Pose2 {
        x: f64::from(t.translation.x),
        y: f64::from(t.translation.y),
        rot: f64::from(t.rotation),
        sx: f64::from(t.scale.x),
        sy: f64::from(t.scale.y),
    }
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "o `Transform` da casa é f32; a lei é f64"
)]
fn com_pose(mut t: Transform, p: Pose2) -> Transform {
    t.translation.x = p.x as f32;
    t.translation.y = p.y as f32;
    t.rotation = p.rot as f32;
    t.scale.x = p.sx as f32;
    t.scale.y = p.sy as f32;
    t
}

/// A pose de onde a pilha parte: a AUTORADA. Se o `Transform` é o que a pilha escreveu no quadro
/// passado, o autorado está no ledger; se outra mão escreveu entretanto, é essa mão.
fn base(sim: &SimWorld, preview: &PreviewDrive, e: Entity) -> Option<Transform> {
    let agora = *sim.world().get::<Transform>(e)?;
    let bits = e.to_bits();
    match (
        preview.last_written(bits, Driver::ConstraintPose),
        preview.authored(bits, Driver::ConstraintPose),
    ) {
        (Some(Driven::ConstraintPose(escrita)), Some(Driven::ConstraintPose(autorada)))
            if escrita == agora =>
        {
            Some(autorada)
        }
        _ => Some(agora),
    }
}

fn dinamico(sim: &SimWorld, e: Entity) -> bool {
    sim.world()
        .get::<RigidBody>(e)
        .is_some_and(|b| b.kind == BodyKind::Dynamic)
}

/// Ao NASCER um item (ou trocar de alvo): os tipos que guardam a pose actual acertam o desvio, para
/// que ligar não salte. Devolve o motivo se não deu.
///
/// # Errors
/// [`Inerte`] se falta o alvo ou o espaço colapsou.
pub fn bind_on_create(
    sim: &SimWorld,
    preview: &PreviewDrive,
    owner: Entity,
    kind: &mut Kind,
    target: Option<Entity>,
) -> Result<(), Inerte> {
    if !kind.keeps_current_on_create() {
        return Ok(());
    }
    let ctx = contexto(sim, preview, owner, target).ok_or(Inerte::SemAlvo)?;
    kind.keep_current(&ctx)
}

fn contexto(
    sim: &SimWorld,
    preview: &PreviewDrive,
    owner: Entity,
    target: Option<Entity>,
) -> Option<Ctx> {
    let w = sim.world();
    let parent = parent_world_transform(w, owner);
    let owner_world = Transform::compose(parent, base(sim, preview, owner)?);
    Some(Ctx {
        owner: pose(&owner_world),
        parent: pose(&parent),
        target: target.and_then(|t| world_transform(w, t)).map(|t| pose(&t)),
        target_parent: target.map_or(Pose2::IDENTITY, |t| pose(&parent_world_transform(w, t))),
    })
}

/// ⭐ **A FASE.** Uma vez por quadro, depois da física e antes do esqueleto. Devolve quantas entidades
/// a pilha moveu.
pub fn evaluate(sim: &mut SimWorld, preview: &mut PreviewDrive) -> usize {
    let adotar = sim
        .world_mut()
        .remove_resource::<Adotar>()
        .unwrap_or_default()
        .0;
    let (ordem, ciclo) = ordem(sim.world());
    let mut estados = BTreeMap::new();
    let ja = preview.driven_by(Driver::ConstraintPose);
    let com_pilha: std::collections::BTreeSet<u64> = ordem
        .iter()
        .chain(ciclo.iter())
        .map(|e| e.to_bits())
        .collect();
    for e in ja {
        if !com_pilha.contains(&e.to_bits()) {
            preview.release_to_authored(sim, e, Driver::ConstraintPose);
        }
    }
    for &e in &ciclo {
        if let Some(c) = sim.world().get::<Constraints>(e) {
            for it in &c.items {
                estados.insert((e.to_bits(), it.id), Estado::Ciclo);
            }
        }
        preview.release_to_authored(sim, e, Driver::ConstraintPose);
    }
    let idx = indice(sim.world());
    let mut movidas = 0;
    for e in ordem.into_iter().filter(|e| !adotar.contains(&e.to_bits())) {
        if avalia_uma(sim, preview, &idx, e, &mut estados) {
            movidas += 1;
        }
    }
    sim.world_mut().insert_resource(Estados(estados));
    movidas
}

fn avalia_uma(
    sim: &mut SimWorld,
    preview: &mut PreviewDrive,
    idx: &BTreeMap<StableId, Entity>,
    e: Entity,
    estados: &mut BTreeMap<(u64, u32), Estado>,
) -> bool {
    let Some(pilha) = sim.world().get::<Constraints>(e).cloned() else {
        return false;
    };
    let Some(base_local) = base(sim, preview, e) else {
        return false;
    };
    let mut marca = |id: u32, s: Estado| {
        estados.insert((e.to_bits(), id), s);
    };
    if dinamico(sim, e) {
        for it in &pilha.items {
            marca(it.id, Estado::CorpoDinamico);
        }
        preview.release_to_authored(sim, e, Driver::ConstraintPose);
        return false;
    }
    let w = sim.world();
    let parent = parent_world_transform(w, e);
    let inicial = pose(&Transform::compose(parent, base_local));
    let mut dono = inicial;
    for it in &pilha.items {
        if !it.enabled {
            marca(it.id, Estado::Desligada);
            continue;
        }
        if it.influence <= 0.0 {
            marca(it.id, Estado::SemForca);
            continue;
        }
        let alvo = it.target.and_then(|s| idx.get(&s).copied());
        if alvo == Some(e) {
            marca(it.id, Estado::AlvoSiMesmo);
            continue;
        }
        let ctx = Ctx {
            owner: dono,
            parent: pose(&parent),
            target: alvo.and_then(|t| world_transform(w, t)).map(|t| pose(&t)),
            target_parent: alvo.map_or(Pose2::IDENTITY, |t| pose(&parent_world_transform(w, t))),
        };
        match ph2d_constraints::apply(&it.kind, &ctx, it.influence.min(1.0)) {
            Ok(p) if p.is_finite() => {
                dono = p;
                marca(it.id, Estado::Activa);
            }
            Ok(_) | Err(Inerte::EspacoColapsado) => marca(it.id, Estado::EspacoColapsado),
            Err(Inerte::SemAlvo) => marca(it.id, Estado::SemAlvo),
        }
    }
    // Sem mudança, o autorado ao bit: desfazer o pai em f32 mexeria o último dígito.
    let local = if dono == inicial {
        base_local
    } else {
        let mundo = com_pose(Transform::compose(parent, base_local), dono);
        Transform::inverse_compose(parent, mundo).unwrap_or(base_local)
    };
    let agora = *sim.world().get::<Transform>(e).unwrap_or(&base_local);
    if local == base_local {
        preview.release_to_authored(sim, e, Driver::ConstraintPose);
        return false;
    }
    if local == agora {
        preview.still_driving(e, Driver::ConstraintPose);
        return false;
    }
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(e) {
        *t = local;
    }
    preview.driven(
        e,
        Driven::ConstraintPose(agora),
        Driven::ConstraintPose(local),
    );
    true
}
