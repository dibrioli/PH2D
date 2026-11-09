//! ⭐ **O que ANEXAR um componente de física faz além de inserir o ponto neutro** (ADR-0166 / F3).
//!
//! ⚠️ Irmão de [`crate::physics_apply`] pelo teto de 600 LOC da shell, e o corte é por
//! ASSUNTO: lá fica o que uma EDIÇÃO do Inspector faz; aqui, o que a CRIAÇÃO semeia. ⛔ Não devolva
//! uma destas funções ao irmão — o teto volta a estourar no gate seguinte.

use ph2d_ecs::{Entity, SimWorld};

/// ⭐ **A caixa que casa com o DESENHO** — as meias-extensões do `Sprite` desta entidade, ou o
/// meio-metro de reserva quando não há sprite nenhuma.
///
/// ⚠️ **Existe porque a lei estava escrita TRÊS vezes** (o `Add`, o `AddShape` e — desde a F3 — o
/// seed da paleta), e uma lei em três sítios diverge no dia em que um deles for corrigido. *Uma lei
/// escrita em dois sítios ainda não é uma lei; só uma PORTA é.*
pub(crate) fn sprite_half_extents(world: &ph2d_ecs::World, entity: Entity) -> [f32; 2] {
    world
        .get::<ph2d_render::Sprite>(entity)
        .map_or([0.5, 0.5], |s| {
            [(s.size[0] * 0.5).max(1e-3), (s.size[1] * 0.5).max(1e-3)]
        })
}

/// ⭐ **Anexar um `Collider` semeia uma caixa do tamanho do sprite** (ADR-0166 F3; ver
/// [`crate::component_seed`]): o `Collider::default()` é uma bola de meio metro e o
/// `insert_default` do registo é type-erased. ⚠️ Só morde na forma AINDA NEUTRA — um collider
/// autorado nunca é reescrito (a lei do `AddShape`), o que a torna idempotente.
pub fn seed_attached_collider(sim: &mut SimWorld, entity_bits: u64) {
    use ph2d_physics_ecs::{Collider, ColliderShape};
    let entity = Entity::from_bits(entity_bits);
    let Some(mut col) = sim.world().get::<Collider>(entity).copied() else {
        return;
    };
    if col.shape != ColliderShape::default() {
        return;
    }
    let half = sprite_half_extents(sim.world(), entity);
    col.shape = ColliderShape::Cuboid {
        half_x: half[0],
        half_y: half[1],
    };
    sim.world_mut().entity_mut(entity).insert(col);
}

/// ⭐⭐⭐ **UM CORPO DEBAIXO DE UM CONTROLADOR CINEMÁTICO NASCE `Kinematic`** (dono 19/09: *«quando
/// eu coloquei a física travou»*). O `RigidBody::default()` é `Dynamic`, e um mover de vista de cima
/// ou um projéctil sobre um corpo dinâmico é inerte: a pose é do SOLVER (`bridge::pose_owner`) e a
/// gravidade leva o objecto. O `TopDownPlayer` e o `ProjectileMotion` REQUEREM `RigidBody`, logo é
/// o caminho de omissão da paleta.
///
/// ⚠️ **Registada em TRÊS nomes porque há DUAS ordens de chegada:** o corpo chega depois do mover
/// (semente do `RigidBody`), ou a cascata anexa o corpo ANTES do mover (semente do mover — quando o
/// corpo nasce, `controlador_cinematico` ainda lê `false`). Uma lei, uma porta, três nomes.
///
/// ⚠️ Conservadora e idempotente: só morde no `BodyKind::default()`, nunca rebaixa um `Static`
/// autorado.
pub fn seed_kinematic_controller_body(sim: &mut SimWorld, entity_bits: u64) {
    use ph2d_physics_ecs::{BodyKind, RigidBody};
    let entity = Entity::from_bits(entity_bits);
    if !ph2d_physics_ecs::controlador_cinematico(sim.world(), entity) {
        return;
    }
    let Some(corpo) = sim.world().get::<RigidBody>(entity).copied() else {
        return;
    };
    if corpo.kind != BodyKind::default() {
        return;
    }
    sim.world_mut().entity_mut(entity).insert(RigidBody {
        kind: BodyKind::Kinematic,
    });
}

/// ⭐⭐ **UM AGENTE DE NAVEGAÇÃO FAZ O MOVER OUVI-LO** (plano 30, W4).
///
/// O agente PEDE e o mover ANDA: a navegação escreve a intenção no canal do `TopDownPlayer`, e a
/// ponte **salta** um mover que ainda ouve o teclado (os dois falariam ao mesmo tempo). ⇒ o gesto de
/// anexar o agente É a declaração *«este objecto é conduzido pela navegação»*, e a semente cumpre-a:
///
/// - `default_controls = false` — **sempre**, mesmo num mover já autorado: um agente com o teclado
///   ligado não anda, e o Inspector diria *«turn off its Default Controls»* sobre um gesto cujo
///   sentido único é esse;
/// - a direcção passa a `Free` **só se ainda for a de fábrica** (`EightWay`): um caminho anda em
///   qualquer ângulo, e oito rumos fariam o agente serrar à volta de uma diagonal — mas um artista que
///   escolheu `FourWay` para um inimigo de grelha escolheu-o, e a semente não o desfaz.
///
/// ⚠️ **A cascata anexa o mover ANTES do agente** (o catálogo pede-o), logo quando esta semente corre
/// o mover já está lá — e o corpo já é `Kinematic` (a semente dele).
pub fn seed_nav_agent(sim: &mut SimWorld, entity_bits: u64) {
    let entity = Entity::from_bits(entity_bits);
    let Some(mut mover) = sim
        .world()
        .get::<ph2d_physics_ecs::TopDownPlayer>(entity)
        .copied()
    else {
        return;
    };
    mover.conduzido_pela_navegacao();
    sim.world_mut().entity_mut(entity).insert(mover);
}

/// ⭐ **Anexar um `Physics Body` faz nascer a forma dele** (opção B, B3a): se o corpo ainda não tem
/// forma ([`ph2d_physics_ecs::main_shape`]), nasce a filha «Collision Shape» com a caixa do desenho
/// ([`ph2d_physics_ecs::nasce_com_forma`]); depois, a lei do controlador cinemático. Idempotente (um
/// Reset não cria segunda forma).
pub fn seed_attached_body(sim: &mut SimWorld, entity_bits: u64) {
    forma_que_falta(sim.world_mut(), Entity::from_bits(entity_bits));
    seed_kinematic_controller_body(sim, entity_bits);
}

/// A forma de um corpo que ainda não a tem: a filha com a caixa do desenho. `None` se `entity` não é
/// corpo ou já tem forma — a porta do seed e do «Add Shape».
pub(crate) fn forma_que_falta(world: &mut ph2d_ecs::World, entity: Entity) -> Option<Entity> {
    if world.get::<ph2d_physics_ecs::RigidBody>(entity).is_none()
        || ph2d_physics_ecs::tem_forma(world, entity)
    {
        return None;
    }
    let [half_x, half_y] = sprite_half_extents(world, entity);
    let caixa = ph2d_physics_ecs::ColliderShape::Cuboid { half_x, half_y };
    Some(ph2d_physics_ecs::nasce_com_forma(world, entity, caixa))
}

/// A semente de um componente: recebe o mundo e os bits da entidade acabada de anexar.
/// ⚠️ Estruturalmente IGUAL a `ph2d_app_components::component_seed::Seed`, e escrita aqui de
/// propósito — nomear a outra família seria a aresta que a A1 cortou.
pub type Seed = fn(&mut SimWorld, u64);

/// ⭐ **As sementes que ESTA família sabe** — o que anexar um componente de física faz além de inserir
/// o ponto neutro.
///
/// ⚠️ A família de componentes NÃO a conhece: é a composição (a shell) que a entrega à porta
/// `ph2d_app_components::component_attach::attach_by_name`. Até à auditoria de arquitectura de
/// 2026-09-12 (A1) a `ph2d-app-components` importava estas duas funções pelo nome, e uma família não
/// chama outra (ADR-0075). O tipo é o ponteiro de função estrutural, para que esta crate não
/// precise de nomear a outra.
pub const COMPONENT_SEEDS: &[(&str, Seed)] = &[
    ("ph2d::physics::Collider", seed_attached_collider),
    ("ph2d::physics::NavAgent", seed_nav_agent),
    (
        "ph2d::physics::PlatformPlayer",
        crate::inspector::player::seed_attached_player,
    ),
    // ⭐ **As TRÊS entradas da mesma lei** — ver [`seed_kinematic_controller_body`]: são duas
    // ordens de chegada, e cada uma precisa de uma delas.
    (
        "ph2d::physics::ProjectileMotion",
        seed_kinematic_controller_body,
    ),
    ("ph2d::physics::RigidBody", seed_attached_body),
    (
        "ph2d::physics::TopDownPlayer",
        seed_kinematic_controller_body,
    ),
];
