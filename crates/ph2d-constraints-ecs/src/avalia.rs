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
    /// A IK é de ossos: o dono não é um.
    NaoEOsso,
    /// O Action pede um clip que a animação do dono não tem.
    SemClip,
}

/// Guarda das passagens de IKs acopladas por quadro. Medido (doc W2 §2): a mudança cai ~40× por
/// passagem e da 6.ª em diante oscila em 1 ulp — o [`assentou`] pára na 5.ª–6.ª; o tecto só apanha
/// uma cena que não converge.
const MAX_PASSAGENS: usize = 64;

/// Nenhuma rotação nem escala mudou mais que 4 ulp do `f32` (o ciclo-limite medido é 1 ulp).
fn assentou(a: &[Option<Transform>], b: &[Option<Transform>]) -> bool {
    let perto = |x: f32, y: f32| (x - y).abs() <= 4.0 * f32::EPSILON * x.abs().max(1.0);
    a.iter().zip(b).all(|(a, b)| match (a, b) {
        (Some(a), Some(b)) => perto(a.rotation, b.rotation) && perto(a.scale.x, b.scale.x),
        _ => a == b,
    })
}

/// As entidades cuja pose VISTA passa a ser a autorada neste quadro (largar um Child Of sem salto):
/// a fase não as conduz uma vez, o `PreviewDrive::settle` esquece-as, e o quadro seguinte parte do
/// que se via. Consumido pela [`evaluate`].
#[derive(Resource, Default, Debug)]
pub struct Adotar(pub std::collections::BTreeSet<u64>);

/// A poligonal LOCAL de cada forma que um Follow Path segue, por `StableId` (quem a lê da cena
/// vectorial é a família). A fase leva-a ao mundo com o afim do alvo NA HORA, depois da pilha dele.
#[derive(Resource, Default, Debug)]
pub struct Caminhos(pub BTreeMap<StableId, Vec<[f32; 2]>>);

/// Os vértices AUTORADOS (locais, com o id; contornos e `subpaths`) de cada forma que um ponto de
/// forma prende, por `StableId` (quem os lê da cena vectorial é a família).
#[derive(Resource, Default, Debug)]
pub struct Pontos(pub BTreeMap<StableId, Vec<(ph2d_vec_scene::VertId, [f64; 2])>>);

/// O estado de cada item no último quadro, por `(Entity::to_bits, Item::id)`.
#[derive(Resource, Default, Debug)]
pub struct Estados(pub BTreeMap<(u64, u32), Estado>);

impl Estados {
    #[must_use]
    pub fn of(&self, entity: Entity, item: u32) -> Option<Estado> {
        self.0.get(&(entity.to_bits(), item)).copied()
    }
}

pub(crate) fn pose(t: &Transform) -> Pose2 {
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
pub(crate) fn base(sim: &SimWorld, preview: &PreviewDrive, e: Entity) -> Option<Transform> {
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

/// ⭐ Corre `f` com a cena AUTORADA: cada entidade que a pilha conduz volta, durante `f`, à pose de
/// onde a pilha parte ([`base`]) — quem lê a pose animada (a entrada da mola) não lê o que a pilha
/// escreveu no quadro passado.
pub fn com_autorado<R>(
    sim: &mut SimWorld,
    preview: &PreviewDrive,
    f: impl FnOnce(&mut SimWorld) -> R,
) -> R {
    let trocas: Vec<(Entity, Transform, Transform)> = preview
        .driven_by(Driver::ConstraintPose)
        .into_iter()
        .filter_map(|e| {
            let agora = *sim.world().get::<Transform>(e)?;
            Some((e, agora, base(sim, preview, e)?))
        })
        .filter(|(_, agora, autorada)| agora != autorada)
        .collect();
    let poe = |sim: &mut SimWorld, e: Entity, t: Transform| {
        if let Some(mut x) = sim.world_mut().get_mut::<Transform>(e) {
            *x = t;
        }
    };
    for &(e, _, autorada) in &trocas {
        poe(sim, e, autorada);
    }
    let r = f(sim);
    for &(e, agora, _) in &trocas {
        poe(sim, e, agora);
    }
    r
}

/// A pose local do dono para a pose mundial `dono`. Sem mudança, o autorado ao bit: desfazer o pai em
/// f32 mexeria o último dígito.
pub(crate) fn local_de(parent: Transform, base_local: Transform, inicial: Pose2, dono: Pose2) -> Transform {
    if dono == inicial {
        return base_local;
    }
    let mundo = com_pose(Transform::compose(parent, base_local), dono);
    Transform::inverse_compose(parent, mundo).unwrap_or(base_local)
}

/// A poligonal da forma `alvo`, no mundo de agora.
pub(crate) fn caminho_no_mundo(
    sim: &SimWorld,
    alvo: Entity,
    id: StableId,
) -> Option<Vec<[f64; 2]>> {
    let local = sim.world().get_resource::<Caminhos>()?.0.get(&id)?;
    let x = ph2d_vec_entities::transform::xform_of_transform(
        ph2d_vec_entities::transform::world_transform(sim, alvo),
    );
    Some(
        local
            .iter()
            .map(|p| x.apply([f64::from(p[0]), f64::from(p[1])]))
            .collect(),
    )
}

/// O vértice autorado `vert` da forma `alvo`, no mundo de agora (`None` se a forma não o tem).
fn ponto_no_mundo(
    sim: &SimWorld,
    alvo: Entity,
    id: StableId,
    vert: ph2d_vec_scene::VertId,
) -> Option<Pose2> {
    let (_, local) = *sim
        .world()
        .get_resource::<Pontos>()?
        .0
        .get(&id)?
        .iter()
        .find(|(v, _)| *v == vert)?;
    let x = ph2d_vec_entities::transform::xform_of_transform(
        ph2d_vec_entities::transform::world_transform(sim, alvo),
    );
    let [px, py] = x.apply(local);
    Some(Pose2 {
        x: px,
        y: py,
        ..Pose2::IDENTITY
    })
}

/// A pose mundial que o clip do Action dá ao dono no instante que o alvo pede: os canais que o clip
/// anima sobre a pose LOCAL de agora.
#[expect(
    clippy::cast_possible_truncation,
    reason = "o `Transform` da casa é f32"
)]
fn accao(
    sim: &SimWorld,
    e: Entity,
    a: &ph2d_constraints::Action,
    alvo: Option<Entity>,
    (parent, mut local): (Transform, Transform),
) -> Result<Pose2, Estado> {
    let t = alvo
        .and_then(|t| world_transform(sim.world(), t))
        .ok_or(Estado::SemAlvo)?;
    let amostra = sim
        .world()
        .get_resource::<crate::Accoes>()
        .and_then(|m| m.0.get(&(e.to_bits(), a.clip)))
        .ok_or(Estado::SemClip)?;
    let [x, y, rot, sx, sy] = amostra(a.tempo(pose(&t)));
    let poe = |v: Option<f64>, campo: &mut f32| {
        if let Some(v) = v {
            *campo = v as f32;
        }
    };
    poe(x, &mut local.translation.x);
    poe(y, &mut local.translation.y);
    poe(rot, &mut local.rotation);
    poe(sx, &mut local.scale.x);
    poe(sy, &mut local.scale.y);
    Ok(pose(&Transform::compose(parent, local)))
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
    // Quem a pilha conduz: as entidades com pilha e os ossos que uma IK delas escreve.
    let mut com_pilha: std::collections::BTreeSet<u64> = ordem
        .iter()
        .chain(ciclo.iter())
        .map(|e| e.to_bits())
        .collect();
    for &e in &ordem {
        if let Some(c) = sim.world().get::<Constraints>(e) {
            com_pilha.extend(
                crate::ik::correntes(sim.world(), e, c)
                    .iter()
                    .map(|x| x.to_bits()),
            );
        }
    }
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
    let mut avaliadas = std::collections::BTreeSet::new();
    let ordem: Vec<Entity> = ordem
        .into_iter()
        .filter(|e| !adotar.contains(&e.to_bits()))
        .collect();
    for &e in &ordem {
        if avalia_uma(sim, preview, &idx, e, &mut estados, &mut avaliadas) {
            movidas += 1;
        }
        avaliadas.insert(e.to_bits());
    }
    // Duas IKs a puxar ossos comuns: repete-se o grupo NESTE quadro até assentar (a cena continua
    // função do autorado — arrastar a timeline dá o mesmo bit).
    let grupo = crate::ik::acopladas(sim.world(), &ordem);
    if !grupo.is_empty() {
        let pose = |sim: &SimWorld| -> Vec<Option<Transform>> {
            grupo
                .iter()
                .map(|&b| sim.world().get::<Transform>(Entity::from_bits(b)).copied())
                .collect()
        };
        let mut n = 1;
        while n < MAX_PASSAGENS {
            let antes = pose(sim);
            for &e in ordem.iter().filter(|e| grupo.contains(&e.to_bits())) {
                avalia_uma(sim, preview, &idx, e, &mut estados, &mut avaliadas);
            }
            n += 1;
            if assentou(&antes, &pose(sim)) {
                break;
            }
        }
        if std::env::var_os("PH2D_CONSTRAINT_LOG").is_some() {
            eprintln!("[constraints] IKs acopladas: {} passagens", n);
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
    avaliadas: &mut std::collections::BTreeSet<u64>,
) -> bool {
    let Some(pilha) = sim.world().get::<Constraints>(e).cloned() else {
        return false;
    };
    let Some(base_local) = base(sim, preview, e) else {
        return false;
    };
    // O que está no mundo à entrada — a IK escreve o dono a meio da conta.
    let agora = *sim.world().get::<Transform>(e).unwrap_or(&base_local);
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
    let mut parent = parent_world_transform(sim.world(), e);
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
        if let Kind::Ik(ik) = &it.kind {
            let local = local_de(parent, base_local, inicial, dono);
            match crate::ik::avalia(sim, preview, e, local, (ik, alvo, it.influence), avaliadas) {
                Ok(nova) => {
                    parent = parent_world_transform(sim.world(), e);
                    dono = pose(&Transform::compose(parent, nova));
                    marca(it.id, Estado::Activa);
                }
                Err(s) => marca(it.id, s),
            }
            continue;
        }
        if let Kind::Spring(sp) = &it.kind {
            let atraso = sim
                .world()
                .get_resource::<crate::Molas>()
                .and_then(|m| m.atrasos.get(&(e.to_bits(), it.id)).copied())
                .unwrap_or_default();
            dono = sp.aplica(dono, atraso, it.influence);
            marca(it.id, Estado::Activa);
            continue;
        }
        if let Kind::Action(a) = &it.kind {
            match accao(
                sim,
                e,
                a,
                alvo,
                (parent, local_de(parent, base_local, inicial, dono)),
            ) {
                Ok(p) => {
                    dono = ph2d_constraints::blend(dono, p, it.influence.min(1.0));
                    marca(it.id, Estado::Activa);
                }
                Err(s) => marca(it.id, s),
            }
            continue;
        }
        if let Kind::SplineIk(sp) = &it.kind {
            let local = local_de(parent, base_local, inicial, dono);
            let alvo = alvo.zip(it.target);
            match crate::curva::avalia(sim, preview, e, local, (sp, alvo, it.influence), avaliadas)
            {
                Ok(nova) => {
                    parent = parent_world_transform(sim.world(), e);
                    dono = pose(&Transform::compose(parent, nova));
                    marca(it.id, Estado::Activa);
                }
                Err(s) => marca(it.id, s),
            }
            continue;
        }
        if let Kind::LookAt(l) = &it.kind {
            let Some(t) = alvo.and_then(|t| world_transform(sim.world(), t)) else {
                marca(it.id, Estado::SemAlvo);
                continue;
            };
            let ponto = [f64::from(t.translation.x), f64::from(t.translation.y)];
            let local = local_de(parent, base_local, inicial, dono);
            let nova = crate::ik::olha(
                sim,
                e,
                (parent, local),
                ponto,
                l.offset,
                it.influence.min(1.0),
            );
            dono = pose(&Transform::compose(parent, nova));
            marca(it.id, Estado::Activa);
            continue;
        }
        let w = sim.world();
        let caminho = match (&it.kind, alvo, it.target) {
            (Kind::FollowPath(_), Some(a), Some(s)) => caminho_no_mundo(sim, a, s),
            _ => None,
        };
        let ctx = Ctx {
            owner: dono,
            parent: pose(&parent),
            target: match (&it.kind, alvo, it.target) {
                (Kind::PontoDeForma(p), Some(a), Some(s)) => ponto_no_mundo(sim, a, s, p.vert),
                _ => alvo.and_then(|t| world_transform(w, t)).map(|t| pose(&t)),
            },
            target_parent: alvo.map_or(Pose2::IDENTITY, |t| pose(&parent_world_transform(w, t))),
        };
        match ph2d_constraints::apply_with(
            &it.kind,
            &ctx,
            caminho.as_deref(),
            it.influence.min(1.0),
        ) {
            Ok(p) if p.is_finite() => {
                dono = p;
                marca(it.id, Estado::Activa);
            }
            Ok(_) | Err(Inerte::EspacoColapsado) => marca(it.id, Estado::EspacoColapsado),
            Err(Inerte::SemAlvo) => marca(it.id, Estado::SemAlvo),
            Err(Inerte::Corrente) => marca(it.id, Estado::NaoEOsso),
            Err(Inerte::Relogio) => marca(it.id, Estado::SemClip),
        }
    }
    let local = local_de(parent, base_local, inicial, dono);
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(e) {
        *t = local;
    }
    if local == base_local {
        // Outra mão (a curva) escreveu o autorado neste quadro: ele já está no mundo, e devolver o
        // do ledger pisá-lo-ia; o `settle` esquece a condução.
        if base_local != agora {
            preview.release_to_authored(sim, e, Driver::ConstraintPose);
        }
        return false;
    }
    if local == agora {
        preview.still_driving(e, Driver::ConstraintPose);
        return false;
    }
    preview.driven(
        e,
        Driven::ConstraintPose(agora),
        Driven::ConstraintPose(local),
    );
    true
}
