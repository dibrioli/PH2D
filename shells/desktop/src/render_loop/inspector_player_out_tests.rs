//! **A SAÍDA do player, do lado da SHELL** (`W-PlayerOut`, A3) — irmão do
//! `inspector_player_tests` pelo teto de LOC, cortado por ASSUNTO.
//!
//! Lá mora *o que os verbos da §14 ESCREVEM no componente*; aqui, *o que o
//! personagem PUBLICA e quem fica sabendo*. As duas famílias crescem por waves
//! diferentes, e é isso que as torna dois arquivos e não um partido ao meio.
//!
//! ⚠️ **É o degrau do MEIO de uma volta com três:** o seam do painel prova que o
//! clique chega ao barramento, o gate da ponte prova que o marcador vira SINAL,
//! e sem este nada prova que o barramento e o marcador se encontram.

use super::inspector_player::{apply_player_edit, build_player_info};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_editor::PlayerFieldEdit;
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};

const CAPSULE: ColliderShape = ColliderShape::Capsule {
    half_height: 0.3,
    radius: 0.2,
};

/// A MESMA fixture do irmão — um corpo Dynamic com a cápsula canônica.
///
/// ⚠️ **Copiada e não importada**, porque um `mod` de teste não é um módulo
/// público: os dois vivem sob o `render_loop`, e um `pub(crate)` no helper do
/// irmão exporia uma fixture ao produto para poupar sete linhas.
fn body() -> (SimWorld, u64) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Name::new("Subject"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: CAPSULE,
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 1.0)),
        ))
        .id();
    (sim, e.to_bits())
}

/// **O verbo da saída de sinais ANEXA e REMOVE o marcador** (`W-PlayerOut`, A3).
///
/// ⚠️ **As duas metades, e a de REMOVER é a que um gate esquece:** o marcador é
/// o booleano inteiro, então um `insert` incondicional deixaria o chip a pintar
/// *Off* sobre um componente presente — o artista desliga e o personagem
/// continua a gritar, com a suíte verde.
///
/// ⚠️ E ele é a costura que fecha a volta: o seam do painel prova que o clique
/// chega ao BARRAMENTO, o gate da ponte prova que o marcador vira SINAL, e este
/// é o degrau do meio.
#[test]
fn the_emit_signals_verb_attaches_and_detaches_the_marker() {
    let (mut sim, bits) = body();
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Add);
    let e = ph2d_ecs::Entity::from_bits(bits);

    assert!(
        build_player_info(&sim, bits, 0.0, 0.0, None)
            .is_some_and(|i| !i.emits_signals),
        "um player novo nasce em SILÊNCIO"
    );

    apply_player_edit(&mut sim, bits, PlayerFieldEdit::EmitSignals(true));
    assert!(
        sim.world()
            .get::<ph2d_physics_ecs::PlayerSignals>(e)
            .is_some(),
        "ligar anexa o marcador"
    );
    assert!(build_player_info(&sim, bits, 0.0, 0.0, None).is_some_and(|i| i.emits_signals));

    apply_player_edit(&mut sim, bits, PlayerFieldEdit::EmitSignals(false));
    assert!(
        sim.world()
            .get::<ph2d_physics_ecs::PlayerSignals>(e)
            .is_none(),
        "desligar REMOVE o marcador — um arquivo não carrega um no-op"
    );
    assert!(
        build_player_info(&sim, bits, 0.0, 0.0, None).is_some_and(|i| !i.emits_signals),
        "e a §14 volta a mostrar Off"
    );
}

/// **O readout viaja da ponte para a §14 sem uma segunda derivação.**
///
/// ⚠️ O oráculo é a TRADUÇÃO, não o número: a postura vira o tag do
/// `FootingKind` e nada mais é recomputado aqui.
#[test]
fn the_live_readout_carries_what_the_law_published() {
    let (mut sim, bits) = body();
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Add);

    assert!(
        build_player_info(&sim, bits, 0.0, 0.0, None).is_some_and(|i| i.live.is_none()),
        "sem leitura da ponte a §14 não inventa uma"
    );

    let view = ph2d_physics_ecs::PlayerView {
        footing: ph2d_physics_ecs::FootingKind::Steep,
        facing: -1.0,
        velocity: [2.5, -0.5],
        ..Default::default()
    };
    let info = build_player_info(&sim, bits, 0.0, 0.0, Some(view)).expect("a §14 existe");
    let live = info.live.expect("com leitura, ela chega");
    assert_eq!(
        live.footing_tag,
        ph2d_physics_ecs::FootingKind::Steep.tag(),
        "a postura viaja pela porta única do tag"
    );
    assert_eq!(live.facing, -1.0);
    assert_eq!(live.velocity, [2.5, -0.5]);
}

