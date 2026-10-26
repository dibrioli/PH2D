//! ⭐⭐⭐ **A IK de um osso, vista pela família** — o desenho (o losango, a faixa da corrente), o dedo
//! (arrastar o alvo), os verbos (*Add IK*, *Look At*, *Remove IK*) e a pergunta *«este osso é
//! governado?»*.
//!
//! Desde a W2 das constraints (`docs/BlenderConstraints/02_W2_ossos.md`) a IK é um **item da pilha**
//! (`ph2d_constraints::Kind::Ik` dentro de `ph2d_constraints_ecs::Constraints`, alvo = o `target` do
//! item, *Mix* = a `influence`) e quem a resolve é a fase da pilha (`ph2d_app_constraints::quadro`).
//! Esta porta lê e escreve o item só pela API de `ph2d_constraints_ecs::ik_item`. O
//! `ph2d_skeleton_ecs::IkGoal` é só a forma gravada de antes da W2, convertida no quadro.
//!
//! ⚠️ O que a IK escreve nos ossos é PRÉ-VISUALIZAÇÃO (`Driver::ConstraintPose`): o artista autora a
//! pose do ALVO; a rotação dos ossos governados é derivada dele.

use ph2d_constraints_ecs::{ik_de, iks};
use ph2d_ecs::{ChildOf, Entity, SimWorld, StableId, Transform};

use ph2d_preview_drive::PreviewDrive;

/// **O índice `StableId → entidade` de TUDO** — o alvo de uma IK é um objecto qualquer, não um osso.
/// Construído uma vez por chamada, como o doc do [`ph2d_ecs::entity_of_stable_id`] manda.
fn index(sim: &SimWorld) -> std::collections::BTreeMap<StableId, Entity> {
    sim.world()
        .iter_entities()
        .filter_map(|er| er.get::<StableId>().map(|s| (*s, er.id())))
        .collect()
}

/// ⭐⭐⭐ **ESTE OSSO É GOVERNADO POR UMA IK?** — o ângulo dele é DERIVADO, e não autorado.
///
/// ⛔ Report do dono (2026-09-08: *«a animação não rodou ao rotacionar o bone»*): um osso da corrente
/// de uma IK não pode ser um CONTROLO — a pilha reescreve-lhe a rotação depois do passe do controlo.
/// Varre as IKs da cena (um punhado), não a corrente de cada osso.
#[must_use]
pub fn is_governed(sim: &SimWorld, bone: Entity) -> bool {
    iks(sim.world())
        .into_iter()
        .any(|(tip, _, _, ik)| governed(sim, tip, ik.chain).contains(&bone))
}

/// ⭐⭐⭐ **A CORRENTE QUE CADA IK GOVERNA, em MUNDO** — as juntas, da raiz da corrente à ponta (report
/// do dono, 2026-09-14: *«não temos uma linha indicativa do IK Chain»*). A população é a MESMA da
/// pilha ([`ph2d_skeleton_live::goal::governed`]).
#[must_use]
pub fn chains(sim: &SimWorld) -> Vec<(u64, Vec<[f64; 2]>)> {
    // `iks` já sai ordenado por bits.
    iks(sim.world())
        .into_iter()
        .filter_map(|(tip, _, _, ik)| {
            let corrente = ph2d_skeleton_live::goal::governed(sim, tip, ik.chain);
            let (juntas, _) = ph2d_skeleton_live::goal::joints_of(sim, &corrente)?;
            (juntas.len() >= 2).then(|| (tip.to_bits(), juntas))
        })
        .collect()
}

/// ⭐⭐⭐ **OS ALVOS DA CENA** — `(osso, alvo, origem do osso, ponta do osso)` em MUNDO, ordenado.
///
/// ⚠️ **Uma varredura, três consumidores**: o desenho (o losango e o tracejado), o dedo (o realce) e
/// o arrasto. O segmento do osso vem junto porque o tamanho do losango sai da mesma porta da bolinha
/// (`joint_radius_px`). Um osso cuja IK perdeu o alvo **não entra**.
pub fn anchors(sim: &SimWorld) -> Vec<ph2d_skeleton_render::Goal> {
    // ⚠️ A saída cedo vem ANTES do índice: isto corre no DESENHO de todo quadro, e a cena comum não
    // tem IK nenhuma.
    let todas = iks(sim.world());
    if todas.is_empty() {
        return Vec::new();
    }
    let idx = index(sim);
    let segs = ph2d_skeleton_live::skin_live::bone_segments(sim);
    todas
        .into_iter()
        .filter_map(|(tip, _, item, _)| {
            let alvo = idx.get(&item.target?).copied()?;
            let (_, o, ponta) = segs.iter().copied().find(|(x, _, _)| *x == tip.to_bits())?;
            let a = ph2d_vec_entities::transform::xform_of_transform(
                ph2d_vec_entities::transform::world_transform(sim, alvo),
            )
            .apply([0.0, 0.0]);
            Some((tip.to_bits(), a, o, ponta))
        })
        .collect()
}

/// ⭐ **O REALCE QUE O LOSANGO DO ALVO RECEBE** — e em *Criar* ele não recebe nenhum: ali o `Tip` quer
/// dizer «daqui nasce um filho», e o losango pode estar LONGE da ponta.
#[must_use]
pub fn goal_hover(
    criar: bool,
    hover: Option<ph2d_skeleton_render::BoneHover>,
) -> Option<ph2d_skeleton_render::BoneHover> {
    if criar { None } else { hover }
}

/// ⭐⭐⭐ **QUE PONTAS RECEBEM ANEL, e o modo decide** (ordem do dono, 2026-09-09). Em *Criar* o anel
/// é *«daqui nasce um filho»* e vale para todo osso; em *Transformar* é o *end effector* e só existe
/// em quem fecha a corrente e não tem IK (ali o losango do alvo substitui-o).
#[must_use]
pub fn ring_targets(sim: &SimWorld, criar: bool) -> Vec<u64> {
    if criar {
        ph2d_skeleton_live::skin_live::visible_bone_polylines(sim)
            .into_iter()
            .map(|(bits, _)| bits)
            .collect()
    } else {
        unanchored_ends(sim)
    }
}

/// As pontas de corrente SEM item IK.
pub fn unanchored_ends(sim: &SimWorld) -> Vec<u64> {
    ph2d_skeleton_live::skin_live::chain_ends(sim)
        .into_iter()
        .filter(|&b| ik_de(sim.world(), Entity::from_bits(b)).is_none())
        .collect()
}

/// ⭐⭐⭐ **O ALVO que a IK de `bone` persegue** — a entidade cuja pose o artista AUTORA quando arrasta
/// a ponta. Dois consumidores: o [`drag_anchor`] e a mão do quadro (o apply da timeline não pode
/// escrever por cima dele enquanto o dedo o move, e o AutoKey cunha-o).
///
/// `None` sem item IK, ou quando o alvo foi apagado.
#[must_use]
pub fn target_of(sim: &SimWorld, bone: Entity) -> Option<Entity> {
    let (_, item, _) = ik_de(sim.world(), bone)?;
    index(sim).get(&item.target?).copied()
}

/// **Move o alvo da IK deste osso para `world`.** `false` se ele não tem alvo vivo.
///
/// ⚠️ O arrasto da ponta deixa de posar a corrente (a pilha reescrevê-la-ia no quadro seguinte) e
/// passa a mover o **objecto autorado**, que é o que o documento guarda.
pub fn drag_anchor(sim: &mut SimWorld, bone: Entity, world: [f64; 2]) -> bool {
    let Some(alvo) = target_of(sim, bone) else {
        return false;
    };
    // O espaço do PAI do ALVO — a pose local vive nele.
    let pai = sim.world().get::<ChildOf>(alvo).map(ChildOf::parent);
    let pai_mundo = pai.map_or(ph2d_vec_scene::Xform::IDENTITY, |p| {
        ph2d_vec_entities::transform::xform_of_transform(
            ph2d_vec_entities::transform::world_transform(sim, p),
        )
    });
    let Some(inv) = pai_mundo.inverse() else {
        return false;
    };
    let p = inv.apply(world);
    let Some(mut t) = sim.world_mut().get_mut::<Transform>(alvo) else {
        return false;
    };
    #[expect(
        clippy::cast_possible_truncation,
        reason = "o `Transform` da casa é f32; a geometria do documento é f64"
    )]
    {
        t.translation = ph2d_core::Vec2::new(p[0] as f32, p[1] as f32);
    }
    true
}

/// **APAGA a IK** deste osso, o alvo com ela, **e devolve à corrente a pose que o artista autorou**.
/// `true` se havia uma.
///
/// ⚠️ O alvo morre junto: ele existe *para* a IK (Ctrl+Z traz os dois de volta).
/// ⛔ Sem a devolução, apagar a IK ASSAVA a pose dela no documento (report do dono, 2026-09-07:
/// *«Remove IK … não funciona plenamente»*): o motor foi DESLIGADO, e o vivo é dele — ver
/// [`PreviewDrive::release_to_authored`]. O motor da pilha é o `Driver::ConstraintPose`.
pub fn remove(sim: &mut SimWorld, bone: Entity, preview: &mut PreviewDrive) -> bool {
    let Some((_, item, ik)) = ik_de(sim.world(), bone) else {
        return false;
    };
    // A corrente lê-se ANTES de o item sair.
    let corrente = governed(sim, bone, ik.chain);
    if let Some(alvo) = item.target.and_then(|t| index(sim).get(&t).copied()) {
        let _ = sim.world_mut().despawn(alvo);
    }
    ph2d_constraints_ecs::tirar_ik(sim.world_mut(), bone);
    for e in corrente {
        preview.release_to_authored(sim, e, ph2d_preview_drive::Driver::ConstraintPose);
    }
    true
}

/// ⭐⭐⭐ **CRIA a IK** deste osso — delegação para [`ph2d_constraints_ecs::add_ik`].
pub fn add(sim: &mut SimWorld, bone: Entity) -> Option<Entity> {
    ph2d_constraints_ecs::add_ik(sim, bone)
}

/// ⭐⭐⭐ **O verbo *Look At*** — delegação para [`ph2d_constraints_ecs::add_look_at`].
pub fn add_look_at(sim: &mut SimWorld, bone: Entity) -> Option<Entity> {
    ph2d_constraints_ecs::add_look_at(sim, bone)
}

/// **A IK deste osso APONTA?** — delegação para [`ph2d_constraints_ecs::aponta`] (a porta que o
/// painel, a pilha e o verbo partilham).
#[must_use]
pub fn aponta(sim: &SimWorld, bone: Entity) -> bool {
    ph2d_constraints_ecs::aponta(sim, bone)
}

/// **A corrente que esta IK governa** — delegação para [`ph2d_skeleton_live::goal::governed`].
pub fn governed(sim: &SimWorld, tip: Entity, chain: u32) -> Vec<Entity> {
    ph2d_skeleton_live::goal::governed(sim, tip, chain)
}

#[cfg(test)]
#[path = "goal_tests.rs"]
mod tests;

/// ⭐ **O que a IK escreve é PRÉ-VISUALIZAÇÃO** — o `tests` mede a LEI por quadro; este mede o que
/// acontece ao DOCUMENTO (quem larga, quem escreve por cima, e o que sobra quando o motor é desligado).
#[cfg(test)]
#[path = "goal_ledger_tests.rs"]
mod ledger_tests;

// ⚠️⚠️ **A FIXTURA DO BRAÇO vive aqui, sob `test-support`, e não no módulo de teste** (HOWTO §2.5):
// partilham-na quatro módulos de teste desta crate e o `skeleton_shell_seam_tests` da shell.
#[cfg(any(test, feature = "test-support"))]
use ph2d_ecs::{Name, RootOrder};
#[cfg(any(test, feature = "test-support"))]
use ph2d_skeleton_ecs::Bone;

#[cfg(any(test, feature = "test-support"))]
/// Um braço de dois ossos deitado no `+X`, com a raiz na origem: ombro `(0,0)→(10,0)`, cotovelo
/// `(10,0)→(20,0)`. Devolve `(sim, [ombro, cotovelo])`.
pub fn braco() -> (SimWorld, [Entity; 2]) {
    let mut sim = SimWorld::default();
    let ombro = osso(&mut sim, "Shoulder", [0.0, 0.0], 10.0, None);
    let cotovelo = osso(&mut sim, "Elbow", [10.0, 0.0], 10.0, Some(ombro));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    (sim, [ombro, cotovelo])
}

/// O construtor de um osso da fixtura (atravessa com ela).
#[cfg(any(test, feature = "test-support"))]
pub fn osso(
    sim: &mut SimWorld,
    nome: &str,
    pos: [f32; 2],
    len: f64,
    pai: Option<Entity>,
) -> Entity {
    let e = sim
        .world_mut()
        .spawn((
            Transform {
                translation: ph2d_core::Vec2::new(pos[0], pos[1]),
                ..Transform::IDENTITY
            },
            Name::new(nome),
            RootOrder(0),
            Bone {
                length: len,
                strength: 1.0,
                ..Default::default()
            },
        ))
        .id();
    if let Some(p) = pai {
        sim.world_mut().entity_mut(e).insert(ChildOf(p));
    }
    e
}

#[cfg(test)]
#[path = "sonda_do_lado_do_joelho_tests.rs"]
mod sonda_do_lado_do_joelho_tests;

// ⏱️ **A sonda do APONTAR**: a IK com a corrente em UM entrega o apontar, e com que erro.
#[cfg(test)]
#[path = "sonda_do_apontar_tests.rs"]
mod sonda_do_apontar_tests;
