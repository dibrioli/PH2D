//! **A SEQUÊNCIA leva a algum lugar** (W5) — a quarta condição de UI que a
//! política deste módulo exige, e a que as outras três não implicam.
//!
//! O seam prova que o clique chega ao barramento; a paridade prova que o widget
//! é registrado; o `every_physics_component_is_authorable` prova que alguém o
//! escreve. **Nenhum dos três prova que o gesto INTEIRO produz um personagem que
//! anda** — foi essa a categoria que pegou o passo *"converta para Capsule"* que
//! quase entrou num roteiro de smoke: geometricamente correto, e destruía o
//! tronco.

use super::inspector_player::{apply_player_edit, build_player_info};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_editor::PlayerFieldEdit;
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PlatformPlayer, RigidBody};

fn body(kind: BodyKind, shape: ColliderShape) -> (SimWorld, u64) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Name::new("Hero"),
            RigidBody { kind },
            Collider {
                shape,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    (sim, e.to_bits())
}

const CAPSULE: ColliderShape = ColliderShape::Capsule {
    half_height: 0.3,
    radius: 0.2,
};

/// **O gesto inteiro:** um corpo Dynamic vê a face vazia, o clique cria o
/// player, e os números vêm do ponto de partida da LEI.
#[test]
fn the_empty_face_becomes_a_player_with_the_laws_starting_point() {
    let (mut sim, bits) = body(BodyKind::Dynamic, CAPSULE);
    let before = build_player_info(&sim, bits).expect("todo corpo Dynamic tem a §14");
    assert!(!before.has_player, "ele ainda nao e' um player");

    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Add);
    let after = build_player_info(&sim, bits).expect("a secao continua viva");
    assert!(after.has_player);
    assert_eq!(after.speed, 6.0, "a velocidade do ponto de partida");
    assert_eq!(after.max_slope_deg, 45.0);
}

/// ⚠️ **E ele nasce PAIRANDO, não tangente.**
///
/// O ponto de partida do modelo (`0,5`) deixa esta cápsula exatamente tangente
/// ao chão — ela não flutua, e só uma rampa revela. O `Add` conhece a forma e
/// sobe a altura acima do piso; sem isso a primeira impressão do artista seria
/// um personagem encostado num app cuja tese é que ele paira.
#[test]
fn a_new_player_floats_over_its_own_collider() {
    let (mut sim, bits) = body(BodyKind::Dynamic, CAPSULE);
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Add);
    let info = build_player_info(&sim, bits).unwrap();
    assert!(
        info.min_float_known,
        "uma capsula tem piso computavel — sem isto o resto do gate nao diz nada"
    );
    assert!(
        info.float_height > info.min_float_height,
        "ele tem de nascer ACIMA do piso: {:.3} vs o minimo {:.3}",
        info.float_height,
        info.min_float_height
    );
}

/// O botão de ajuste conserta uma altura curta autorada à mão.
#[test]
fn fit_to_collider_raises_a_short_float_height() {
    let (mut sim, bits) = body(BodyKind::Dynamic, CAPSULE);
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Add);
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::FloatHeight(0.2));
    let short = build_player_info(&sim, bits).unwrap();
    assert!(
        short.float_height < short.min_float_height,
        "a fixture TEM de conter o defeito"
    );

    apply_player_edit(&mut sim, bits, PlayerFieldEdit::FitFloatHeight);
    let fixed = build_player_info(&sim, bits).unwrap();
    assert!(
        fixed.float_height > fixed.min_float_height,
        "o ajuste tem de passar do piso: {:.3} vs {:.3}",
        fixed.float_height,
        fixed.min_float_height
    );
}

/// ⚠️ **Um corpo que não é Dynamic não tem a §14, e a recusa é dupla.**
///
/// A mola é um impulso, e um impulso não move massa infinita. O pintor não a
/// oferece (o info é `None`) **e** o barramento não a honra — porque uma recusa
/// que mora só no laço de pintura não é recusa: os ids vivem no store a sessão
/// inteira, e um clique roteado por outra coisa chegaria aqui.
#[test]
fn a_static_body_gets_neither_the_section_nor_the_write() {
    for kind in [BodyKind::Static, BodyKind::Kinematic] {
        let (mut sim, bits) = body(kind, CAPSULE);
        assert!(
            build_player_info(&sim, bits).is_none(),
            "{kind:?} nao pode receber a secao"
        );
        apply_player_edit(&mut sim, bits, PlayerFieldEdit::Add);
        assert!(
            sim.world()
                .get::<PlatformPlayer>(ph2d_ecs::Entity::from_bits(bits))
                .is_none(),
            "{kind:?} nao pode receber o componente nem por um clique roteado"
        );
    }
}

/// **Remover devolve o corpo a um corpo comum** — e a seção continua viva, com a
/// face vazia, para que ele possa voltar a ser um player.
#[test]
fn remove_gives_the_body_back_and_keeps_the_door_open() {
    let (mut sim, bits) = body(BodyKind::Dynamic, CAPSULE);
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Add);
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Remove);
    let info = build_player_info(&sim, bits).expect("a secao NAO some com o componente");
    assert!(!info.has_player);
}

/// ⚠️ **O amortecimento é clampado no TETO MEDIDO da lei** — acima dele o boost
/// inverte a velocidade em vez de matá-la, e o personagem pipoca.
///
/// Duas camadas: a porta da lei também clampa. Esta existe para o número
/// AUTORADO nunca guardar algo que o motor não vai honrar — um campo que mente
/// sobre si mesmo é pior que um clamp invisível.
#[test]
fn the_damping_is_clamped_to_the_measured_ceiling() {
    let (mut sim, bits) = body(BodyKind::Dynamic, CAPSULE);
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Add);
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::SpringDamping(5.0));
    let info = build_player_info(&sim, bits).unwrap();
    assert_eq!(
        info.spring_damping,
        ph2d_physics_ecs::RideConfig::MAX_DAMPING
    );
}

/// ⚠️ **Uma CAIXA não tem piso computável, e o info o diz** em vez de devolver
/// a fórmula da cápsula.
///
/// A extensão de uma cápsula ao longo de uma normal é `radius + hh·cos θ` (o
/// raio é isotrópico) e a de uma caixa é `half_x·sin θ + half_y·cos θ` — outra
/// fórmula, outro piso. Um número errado apresentado como certo é pior que a
/// ausência dele.
#[test]
fn a_box_reports_no_known_floor() {
    let (sim, bits) = body(
        BodyKind::Dynamic,
        ColliderShape::Cuboid {
            half_x: 0.3,
            half_y: 0.5,
        },
    );
    let info = build_player_info(&sim, bits).unwrap();
    assert!(!info.min_float_known);
}
