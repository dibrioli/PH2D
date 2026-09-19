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

/// ⭐ **O que anexar um `Collider` faz além de inserir o ponto neutro** (ADR-0166 / F3 · a emenda
/// medida na F0 — ver [`crate::component_seed`]).
///
/// O `Collider::default()` é uma **bola de meio metro**; debaixo de um sprite quadrado ela é
/// exatamente o desencontro que o Enio apanhou em 2026-07-18. O `insert_default` do registo é
/// type-erased e não pode saber o tamanho do desenho.
///
/// ⚠️ **Só morde na forma AINDA NEUTRA**, e é o que o torna idempotente e seguro: um collider que o
/// artista já autorou nunca é reescrito — a mesma lei que o `AddShape` honra («a porta que CRIA a
/// peça recusa reescrever uma que existe»), medida numa peça `0,17 × 0,91` que voltava `0,10 × 0,50`
/// com tudo zerado.
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

/// ⭐⭐⭐ **UM CORPO DEBAIXO DE UM CONTROLADOR CINEMÁTICO NASCE `Kinematic`** (report do dono,
/// 19/09: *«quando eu coloquei a física travou»*).
///
/// ⛔⛔ **O que ele viu, medido:** o `RigidBody::default()` é **`Dynamic`**, e um mover de vista de
/// cima ou um projéctil debaixo de um corpo dinâmico é **inerte por construção** — a pose passa a
/// ser do SOLVER (`ph2d_physics_ecs::controlador_cinematico` / `bridge::pose_owner`) e a gravidade
/// leva o objecto: `y = −492 m` ao fim de dez segundos, com a câmera a segui-lo. *Em ~2 s não há
/// cena nenhuma no ecrã, e nada do que o artista carregue a traz de volta.*
///
/// ⚠️⚠️ **E o gesto que produz isso é o NORMAL:** o `TopDownPlayer` e o `ProjectileMotion`
/// **requerem** `RigidBody` no catálogo, logo escolher qualquer um deles na paleta anexa o corpo em
/// cascata — sem esta semente, **o caminho de omissão da paleta entrega um componente que não
/// funciona**. O Inspector já diz o porquê em vermelho (*«the body must be kinematic»*), e um aviso
/// certo sobre um valor de fábrica errado ainda é um valor de fábrica errado.
///
/// # ⚠️ Ela é registada em TRÊS nomes porque há DUAS ordens de chegada
///
/// | o artista escolhe | o que acontece | quem tem de semear |
/// |---|---|---|
/// | *Physics Body* num objecto que já tem o mover | o corpo chega por último | a semente do **`RigidBody`** |
/// | *Top-Down Player* / *Projectile Motion* num objecto nu | a **cascata** anexa o corpo ANTES do mover | a semente do **mover** |
///
/// ⛔ A segunda metade não é opcional: a cascata corre **antes** do `attach_one` do dependente (é o
/// que deixa o seed do `PlatformPlayer` medir o collider), logo no momento em que o corpo nasce a
/// entidade ainda **não** tem o controlador — e uma semente só no `RigidBody` leria `false` e
/// deixaria o corpo dinâmico. *As duas entradas são a MESMA função: uma lei, uma porta, três nomes.*
///
/// ⚠️ **Conservadora e idempotente**, como as irmãs: só morde no corpo ainda no **ponto neutro**
/// (`BodyKind::default()`), logo nunca rebaixa um `Static` que o artista pôs, e correr duas vezes
/// não move nada.
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
    ("ph2d::physics::RigidBody", seed_kinematic_controller_body),
    (
        "ph2d::physics::TopDownPlayer",
        seed_kinematic_controller_body,
    ),
];
