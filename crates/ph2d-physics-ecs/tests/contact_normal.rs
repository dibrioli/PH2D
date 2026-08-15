//! **A NORMAL DO CONTATO** (`W-HitNormal`) — o `get_last_slide_collision` do
//! Godot e o `OnControllerColliderHit` da Unity, o último item da cauda do §3.A.
//!
//! ⚠️ **O canal já respondia *em que eu bati*; o que faltava era a ORIENTAÇÃO.**
//! Medido antes de uma linha ser escrita: o `contacts()` genérico nomeia o par do
//! player em **todos os três modos** (Dynamic / Kinematic / Pure — 155 de 180
//! tiques contra uma parede, 163 empurrando um caixote 15,7 m), então o item
//! nunca precisou de um canal por-player. O que ele não carregava era a normal —
//! e é ela que decide *parede ou chão*, orienta uma faísca e deixa alguém
//! deslizar pelo que encostou.
//!
//! ⚠️ **E foi por isso que a resposta NÃO é uma lista de `CharacterHit`:** essa
//! lista existe só nos dois modos que passam pelo controlador cinemático, e no
//! Dynamic estaria **vazia em silêncio** — o *"funciona às vezes"* que esta linha
//! recusa desde a `W-Ceiling`.
//!
//! O oráculo destes gates é a **GEOMETRIA da cena**: onde cada corpo está diz
//! para que lado a normal tem de apontar, sem o teste saber nada da fórmula.

#[path = "platform_water_scene.rs"]
mod scene;

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, PhysicsBridge, PlatformPlayer, RigidBody,
};
use ph2d_platformer::PlayerInput;
use scene::{FLOAT, floor, subject_tuned};

fn wall(sim: &mut SimWorld, x: f32) {
    sim.world_mut().spawn((
        Name::new("Wall"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.5,
                half_y: 3.0,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(x, 3.0)),
    ));
}

fn crate_at(sim: &mut SimWorld, x: f32) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new("Crate"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.3,
                    half_y: 0.3,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, 0.3)),
        ))
        .id()
}

fn name_of(sim: &SimWorld, e: Entity) -> String {
    sim.world()
        .get::<Name>(e)
        .map_or_else(|| format!("{e:?}"), |n| n.0.to_string())
}

/// A cena: o personagem anda para a direita empurrando um caixote contra uma
/// parede. Devolve, para cada par tocando no fim, `(nome de a, nome de b, normal)`.
fn touching() -> (SimWorld, Vec<(String, String, [f32; 2])>) {
    let mut sim = SimWorld::new();
    floor(&mut sim, 0.0);
    wall(&mut sim, 3.0);
    let who = subject_tuned(
        &mut sim,
        true,
        FLOAT,
        Some(PlatformPlayer {
            float_height: FLOAT,
            ..PlatformPlayer::default()
        }),
    );
    crate_at(&mut sim, 1.5);
    let mut bridge = PhysicsBridge::new();
    for t in 1..=160u64 {
        bridge.set_player_input(
            who,
            PlayerInput {
                drive: 1.0,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, t);
    }
    let out = bridge
        .contacts()
        .iter()
        .map(|c| (name_of(&sim, c.a), name_of(&sim, c.b), c.normal))
        .collect();
    (sim, out)
}

/// **A normal aponta de `a` para `b`, e quem julga é onde os corpos ESTÃO.**
///
/// Três orientações independentes numa cena só — direita, esquerda e para baixo —
/// e nenhuma comparada contra um número escrito à mão: o oráculo sai das POSES.
///
/// ⚠️ **Sem este gate a normal seria publicada com o sinal da convenção da
/// BIBLIOTECA** (`local_n1` é de *collider1* para *collider2*, e o par é
/// publicado em ordem de HANDLE, que não tem relação nenhuma com aquela) — um
/// sinal cara-ou-coroa do lado de quem lê, que é o que o
/// [`ph2d_physics::CharacterHit::normal`] avisa e o hook one-way já pagou.
#[test]
fn the_contact_normal_points_from_a_to_b() {
    let (sim, pairs) = touching();
    assert!(pairs.len() >= 3, "a cena tem de ter contato: {pairs:?}");

    let mut checked = 0;
    for (a, b, n) in &pairs {
        let (ax, ay) = scene::xy_of(&sim, a);
        let (bx, by) = scene::xy_of(&sim, b);
        let (dx, dy) = (bx - ax, by - ay);
        // ⚠️ **O oráculo é o SEMI-ESPAÇO, não um eixo** — e a primeira versão deste
        // gate errou aqui: ela pedia que a normal se alinhasse com o eixo dominante
        // da separação entre CENTROS, e reprovou sobre um produto correto. Uma
        // cápsula alta encostada na FACE de um caixote baixo tem os centros
        // separados sobretudo em `y` e uma normal de superfície **horizontal**
        // (medido: separação `y` 0,586 contra normal `[-0,997, 0,079]`). Direção
        // entre centros não é normal de superfície; o que a geometria de fato
        // garante é que a normal aponta para o lado em que `b` está.
        let proj = n[0] * dx + n[1] * dy;
        assert!(
            proj > 0.0,
            "{a} -> {b}: {b} esta' em ({dx:.3}, {dy:.3}) e a normal {n:?} aponta para o \
             outro lado (projecao {proj:.3})"
        );
        checked += 1;
    }
    assert!(checked >= 3, "so {checked} pares conferidos");
}

/// **A normal é UNITÁRIA** — quem a usa para orientar uma faísca ou refletir uma
/// velocidade não deve ter de normalizá-la primeiro (e quem esquecer de o fazer
/// não deve descobrir por um efeito de tamanho errado).
#[test]
fn the_contact_normal_is_a_unit_vector() {
    let (_sim, pairs) = touching();
    assert!(!pairs.is_empty());
    for (a, b, n) in &pairs {
        let len = n[0].hypot(n[1]);
        assert!(
            (len - 1.0).abs() < 1e-3,
            "{a} -> {b}: |n| = {len:.6}, nao 1 ({n:?})"
        );
    }
}

/// **O par de nomes é o MESMO da normal** — o controle que impede o gate de cima
/// de ficar verde sobre um canal que publica pares e orientações desalinhados.
///
/// Ele afirma o que a cena tem: o caixote está ENTRE o personagem e a parede,
/// então tem de haver um par com cada um dos dois, e as duas normais têm de
/// apontar para lados OPOSTOS a partir dele.
#[test]
fn the_crate_is_pushed_from_one_side_and_holds_against_the_other() {
    let (_sim, pairs) = touching();
    let find = |x: &str, y: &str| {
        pairs
            .iter()
            .find(|(a, b, _)| (a == x && b == y) || (a == y && b == x))
            .map(|(a, _, n)| if a == x { *n } else { [-n[0], -n[1]] })
    };
    let to_subject = find("Crate", "Subject").expect("o caixote toca o personagem");
    let to_wall = find("Crate", "Wall").expect("o caixote toca a parede");
    assert!(
        to_subject[0] < -0.5 && to_wall[0] > 0.5,
        "o caixote e' empurrado pela esquerda contra a parede a' direita: \
         para o personagem {to_subject:?}, para a parede {to_wall:?}"
    );
}

// ── E O CASO QUE A CENA ACIMA NÃO CONTÉM ─────────────────────────────────────
//
// ⚠️ **A mutação que apaga o flip de ordem SOBREVIVE aos três gates de cima**, e
// isso é fixture, não buraco de lei: com **um** collider por corpo a ordem de
// collider que a fase estreita usa acompanha a de handle, então o ramo da troca
// nunca corre. Quem o exercita é um corpo **COMPOSTO** (W-Compound) — medido:
// pondo um `assert!(!swapped)` no produto, os **sete** testes de `compound.rs`
// disparam e nenhum outro da suíte o faz.

/// Um corpo em L — um braço com uma perna pendurada como SEGUNDA forma do mesmo
/// corpo — largado sobre o chão. É o menor fixture que põe a ordem de collider em
/// desacordo com a de handle.
fn compound_touching() -> (SimWorld, Vec<(String, String, [f32; 2])>) {
    let mut sim = SimWorld::new();
    floor(&mut sim, 0.0);
    let arm = sim
        .world_mut()
        .spawn((
            Name::new("Arm"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 1.0,
                    half_y: 0.2,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 2.0)),
        ))
        .id();
    sim.world_mut().spawn((
        Name::new("Leg"),
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.2,
                half_y: 1.0,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.8, -1.0)),
        ph2d_ecs::ChildOf(arm),
    ));

    let mut bridge = PhysicsBridge::new();
    for t in 1..=200u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let out = bridge
        .contacts()
        .iter()
        .map(|c| (name_of(&sim, c.a), name_of(&sim, c.b), c.normal))
        .collect();
    (sim, out)
}

/// **A ordem PUBLICADA manda, mesmo quando a da biblioteca discorda dela.**
///
/// ⚠️ **Este é o gate que prova o flip**, e a mutação que o mata é apagar o
/// `sign`: num corpo composto a fase estreita pode reportar o par com os
/// colliders em ordem oposta à dos corpos, e sem a negação a normal sai a
/// apontar para o lado contrário ao do `a`/`b` que o leitor recebeu — um sinal
/// que ninguém consegue prever de fora.
#[test]
fn the_published_order_wins_over_the_librarys_even_on_a_compound_body() {
    let (sim, pairs) = compound_touching();
    assert!(!pairs.is_empty(), "o L tem de assentar no chao: {pairs:?}");
    for (a, b, n) in &pairs {
        let (ax, ay) = scene::xy_of(&sim, a);
        let (bx, by) = scene::xy_of(&sim, b);
        let proj = n[0] * (bx - ax) + n[1] * (by - ay);
        assert!(
            proj > 0.0,
            "{a} -> {b}: a normal {n:?} aponta para o lado errado (projecao {proj:.3})"
        );
    }
}

/// SONDA: o que a cena composta publica, para o A/B do flip.
#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn measure_what_the_compound_scene_publishes() {
    let (sim, pairs) = compound_touching();
    for (a, b, n) in &pairs {
        let (ax, ay) = scene::xy_of(&sim, a);
        let (bx, by) = scene::xy_of(&sim, b);
        println!(
            "{a:6} ({ax:5.2},{ay:5.2}) -> {b:6} ({bx:5.2},{by:5.2})  n = [{:6.3},{:6.3}]",
            n[0], n[1]
        );
    }
}
