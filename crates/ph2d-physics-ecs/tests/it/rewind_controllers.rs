//! **REBOBINAR com os CONTROLADORES** — report do dono, 2026-09-15:
//! *«o Rewind não está funcionando com os projéteis. Eles têm um comportamento diferente a cada
//! rewind»*.
//!
//! A pergunta é a MESMA que o [`crate::platform_tape`] já fez pelo player de plataforma —
//! *arrastar a régua para trás devolve a mesma corrida?* — e a resposta era **não** para os dois
//! controladores que chegaram depois dele.
//!
//! # ⛔⛔ Duas metades, e nenhuma bastava sozinha
//!
//! 1. **O laço de replay do `rewind_to` dirigia SÓ o `drive_players`.** O `drive_topdown` e o
//!    `drive_projectiles` entraram no laço da frente (`dispatch`) e **não** no de trás — é
//!    literalmente o defeito que o cabeçalho do `bridge::tape` narra sobre o player, repetido duas
//!    vezes porque os dois laços são escritos à mão em sítios diferentes. *Uma lei ensinada a uma
//!    metade de um par é uma lei que não existe.*
//! 2. **O `rebuild_from_rest` limpava `player_state` e `topdown_state` e NÃO `projectile_state`.**
//!    Reconstruir do repouso É o tique 0, e no tique 0 nenhuma bala nasceu — mas a memória de voo
//!    sobrevivia ao Reset: `finished` ⇒ a bala nunca mais voa, `travelled` ⇒ morre mais cedo a
//!    cada corrida, `bounces_used` ⇒ deixa de ricochetear, e `velocity` ⇒ o arco arranca com a
//!    velocidade com que morreu. ⭐ **Cada Reset dava um voo diferente, que é o report à letra.**
//!
//! ⚠️ **A cura é UMA PORTA** (`PhysicsBridge::drive_controllers`) chamada pelos dois laços: um
//! quarto controlador **não compila** sem passar por ela, exactamente como o `ControllerMemory` é
//! um TIPO e não um mapa.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, InputTape, LockRotation, PhysicsBridge, PlayerInput,
    ProjectileMotion, RigidBody, TopDownPlayer,
};
use ph2d_projectile::ProjectileLaw;

fn parede(sim: &mut SimWorld, em: Vec2, meio: (f32, f32)) {
    sim.world_mut().spawn((
        Name::new("Parede"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: meio.0,
                half_y: meio.1,
            },
            ..Collider::default()
        },
        Transform::from_translation(em),
    ));
}

/// Uma bala que **ricocheteia** e tem **alcance** — as três metades da memória de voo em jogo de
/// uma vez (velocidade, metros, saltos), que é o que faz um Reset mal feito divergir de três
/// maneiras em vez de uma.
fn cena_bala() -> (SimWorld, PhysicsBridge, Entity) {
    let mut sim = SimWorld::new();
    parede(&mut sim, Vec2::new(3.0, 0.0), (0.5, 4.0));
    let quem = sim
        .world_mut()
        .spawn((
            Name::new("Bala"),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.1 },
                ..Collider::default()
            },
            ProjectileMotion::from_law(
                ProjectileLaw {
                    initial_speed: 8.0,
                    max_bounces: 3,
                    bounciness: 0.8,
                    range: 12.0,
                    ..ProjectileLaw::default()
                },
                0,
            ),
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    (sim, PhysicsBridge::new(), quem)
}

fn pose(sim: &SimWorld, quem: Entity) -> (f32, f32, f32) {
    let t = sim.world().get::<Transform>(quem).expect("o corpo");
    (t.translation.x, t.translation.y, t.rotation)
}

/// A trajectória inteira, tique a tique — ⚠️ **e não a pose final**: uma bala que morre cedo e uma
/// que voa podem pousar no mesmo sítio, e é o CAMINHO que o dono vê.
fn voo(sim: &mut SimWorld, bridge: &mut PhysicsBridge, quem: Entity, ate: u64) -> Vec<(f32, f32)> {
    let mut caminho = Vec::new();
    for t in 1..=ate {
        bridge.dispatch(sim, true, t);
        let p = pose(sim, quem);
        caminho.push((p.0, p.1));
    }
    caminho
}

/// ⭐⭐⭐ **O report do dono: um Reset devolve o MESMO voo.**
///
/// Três corridas seguidas separadas por um Reset, e as três têm de ser **iguais ao bit**.
///
/// ⚠️ **TRÊS e não duas**, e a terceira não é simetria: com a memória a sobreviver, a 2.ª corrida
/// já tem `finished = true` e não anda **nada** — duas corridas comparariam «voou» contra «parada»
/// e uma cura pela metade (limpar só o `finished`) passaria. A 3.ª obriga a que o estado limpo seja
/// o mesmo todas as vezes, que é a propriedade que o dono descreve com *«a cada rewind»*.
#[test]
fn um_reset_devolve_o_mesmo_voo() {
    let (mut sim, mut bridge, quem) = cena_bala();
    let primeira = voo(&mut sim, &mut bridge, quem, 90);
    // ⚠️ **O ALCANCE do caminho, nunca a pose FINAL** — esta bala ricocheteia e volta, então o
    // `x` do último tique é **negativo** e um `> 1.0` ali reprovaria sobre um voo perfeito. A
    // pré-condição é *«ela saiu do sítio»*, e a grandeza que o diz é o máximo do percurso.
    let longe = primeira
        .iter()
        .fold(0.0_f32, |m, p| m.max(p.0.abs().max(p.1.abs())));
    assert!(
        longe > 1.0,
        "a 1.ª corrida tem de ter VOADO, senao o gate compara tres balas paradas (alcance {longe})"
    );

    let mut corridas = vec![primeira];
    for _ in 0..2 {
        // O Reset: a régua volta ao zero, e o transporte fica parado — os dois caminhos que o
        // artista tem até ao tique 0.
        bridge.dispatch(&mut sim, false, 0);
        corridas.push(voo(&mut sim, &mut bridge, quem, 90));
    }

    for (i, c) in corridas.iter().enumerate().skip(1) {
        assert_eq!(
            *c,
            corridas[0],
            "a corrida {} depois de um Reset nao reproduziu a 1.ª — o dono ve um voo diferente a \
             cada rewind (1.ª acaba em {:?}, esta em {:?})",
            i + 1,
            corridas[0].last(),
            c.last()
        );
    }
}

/// ⭐⭐⭐ **Um scrub para o MEIO replaya o projéctil.**
///
/// ⚠️ **O alvo tem de cair no MEIO, e não no zero:** um rewind para o tique 0 replaya **zero**
/// passos, então um gate que só resetasse deixaria a mutação que apaga o `drive_projectiles` do
/// laço de replay **sobreviver** — a armadilha que o `platform_tape` já pagou por escrito.
///
/// As DUAS rotas do `rewind_to` são exercitadas: com o ring quente ele **semeia** de um tique
/// âncora e replaya poucos passos; com o ring vazio ele **reconstrói do repouso** e replaya tudo.
#[test]
fn um_scrub_para_o_meio_replaya_o_projectil() {
    const MEIO: u64 = 47;
    let direto = {
        let (mut sim, mut bridge, quem) = cena_bala();
        voo(&mut sim, &mut bridge, quem, MEIO);
        pose(&sim, quem)
    };
    // ⚠️ Pelo mesmo motivo do gate acima: no tique 47 esta bala já bateu na parede e voltou, logo
    // o sinal do `x` não diz nada — o que diz é a DISTÂNCIA ao repouso.
    assert!(
        direto.0.abs() > 0.5,
        "a corrida de referencia tem de ter andado: {direto:?}"
    );

    for ring_vazio in [false, true] {
        let (mut sim, mut bridge, quem) = cena_bala();
        voo(&mut sim, &mut bridge, quem, 150);
        if ring_vazio {
            bridge.forget_checkpoints();
        }
        bridge.dispatch(&mut sim, false, MEIO);
        let raspado = pose(&sim, quem);
        assert!(
            (direto.0 - raspado.0).abs() < 1.0e-3 && (direto.1 - raspado.1).abs() < 1.0e-3,
            "o scrub para o tique {MEIO} nao reproduziu o voo (ring vazio={ring_vazio}): \
             {direto:?} contra {raspado:?}"
        );
    }
}

/// ⭐⭐ **E o mover de VISTA DE CIMA, que estava fora do MESMO laço.**
///
/// ⚠️ Ele entra aqui e não no ficheiro dele porque a lei medida é a do **replay**, não a do
/// deslize: os dois controladores estavam fora do laço de trás pela mesma linha, e a cura é a
/// mesma porta. *Separá-los faria duas metades de um achado só.*
#[test]
fn um_scrub_para_o_meio_replaya_o_mover_de_vista_de_cima() {
    const MEIO: u64 = 47;

    fn cena() -> (SimWorld, PhysicsBridge, Entity) {
        let mut sim = SimWorld::new();
        let quem = sim
            .world_mut()
            .spawn((
                Name::new("Heroi"),
                RigidBody {
                    kind: BodyKind::Kinematic,
                },
                Collider {
                    shape: ColliderShape::Ball { radius: 0.2 },
                    ..Collider::default()
                },
                LockRotation,
                TopDownPlayer {
                    default_controls: true,
                    ..TopDownPlayer::default()
                },
                Transform::from_translation(Vec2::new(0.0, 0.0)),
            ))
            .id();
        (sim, PhysicsBridge::new(), quem)
    }

    /// Uma fita roteirizada: anda para a direita o tempo todo. Determinística por construção.
    fn fita(ate: u64) -> InputTape {
        let mut t = InputTape::new();
        for i in 1..=ate {
            t.record(
                i,
                PlayerInput {
                    drive: 1.0,
                    drive_y: 0.0,
                    jump: false,
                    down: false,
                    dash: false,
                    grab: false,
                },
            );
        }
        t
    }

    let direto = {
        let (mut sim, mut bridge, quem) = cena();
        let mut t = fita(200);
        for i in 1..=MEIO {
            bridge.dispatch_with_tape(&mut sim, true, i, &mut t);
        }
        pose(&sim, quem)
    };
    assert!(
        direto.0 > 1.0,
        "a corrida de referencia tem de ter andado: {direto:?}"
    );

    for ring_vazio in [false, true] {
        let (mut sim, mut bridge, quem) = cena();
        let mut t = fita(200);
        for i in 1..=150 {
            bridge.dispatch_with_tape(&mut sim, true, i, &mut t);
        }
        if ring_vazio {
            bridge.forget_checkpoints();
        }
        bridge.dispatch_with_tape(&mut sim, false, MEIO, &mut t);
        let raspado = pose(&sim, quem);
        assert!(
            (direto.0 - raspado.0).abs() < 1.0e-3 && (direto.1 - raspado.1).abs() < 1.0e-3,
            "o scrub para o tique {MEIO} nao reproduziu a caminhada (ring vazio={ring_vazio}): \
             {direto:?} contra {raspado:?}"
        );
    }
}
