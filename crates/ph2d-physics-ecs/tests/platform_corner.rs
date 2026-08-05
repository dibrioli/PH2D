//! **A QUINA** (W10) — os gates de COMPORTAMENTO, com o rapier de verdade.
//!
//! A lei pura tem os dela na `ph2d-platformer` (dado um perfil, qual escape).
//! Estes fazem a pergunta que só a simulação responde: *o pulo que encostava na
//! beirada agora PASSA, o que batia num teto continua batendo, e o personagem
//! não sai derivando de lado depois?*
//!
//! ⚠️ **O oráculo é o PICO do pulo**, não a posição num instante: um encosto
//! rouba altura, e a assistência é boa exatamente quando o pico volta a ser o
//! que seria sem obstáculo nenhum. Medir "ele está acima de tal linha no tique
//! N" seria medir o relógio junto.

#[path = "platform_scene.rs"]
mod scene_fixture;

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, PhysicsBridge, PlatformPlayer, PlayerInput, RigidBody,
};
use scene_fixture::{FLOAT_HEIGHT, pose, scene};

/// A meia-largura da cápsula da fixture (a caixa dela é `2·radius`).
const HALF_W: f32 = 0.2;
/// A face de baixo da beirada, acima do repouso do personagem.
const UNDER: f32 = FLOAT_HEIGHT + 0.5 + 1.2;
/// O pico de um pulo SEM obstáculo nenhum, medido nesta fixture.
const FREE_PEAK: f32 = 0.833;

struct Rig {
    sim: SimWorld,
    bridge: PhysicsBridge,
    player: ph2d_ecs::Entity,
}

/// Chão + beirada cobrindo `clip` metros do lado direito da cabeça. `reach = 0`
/// é o CONTROLE (a assistência desligada), e `air` deixa o gate escolher se o
/// controle aéreo freia ou conserva o arco.
fn rig(clip: f32, reach: f32, air: f32) -> Rig {
    let (mut sim, bridge, player) = scene(0.0, 0.0);
    let half_x = 2.0;
    sim.world_mut().spawn((
        Name::new("Ledge"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(HALF_W - clip + half_x, UNDER + 0.5)),
    ));
    let mut q = sim
        .world_mut()
        .try_query::<(&Name, &mut PlatformPlayer)>()
        .expect("a query do player");
    for (n, mut p) in q.iter_mut(sim.world_mut()) {
        if n.as_str() == "Player" {
            p.corner_reach = reach;
            p.air_acceleration = air;
        }
    }
    Rig {
        sim,
        bridge,
        player,
    }
}

/// Um bloco à ESQUERDA, com a face voltada ao personagem em `face` e o topo em
/// `top`.
///
/// ⚠️ **O `top` é a razão de este helper existir e não ser uma parede alta**, e a
/// primeira versão deste arquivo errou exatamente aí: uma parede que sobe acima
/// da cabeça aparece no **perfil do teto** (os raios sobem a partir do topo da
/// caixa, e ela está ali), então o escape já é recusado pelo perfil e o gate da
/// folga lateral fica **verde com a folga lateral deletada** — a fixture não
/// continha o fenômeno.
///
/// O caso que só a `side_clear` cobre é o bloco **baixo**: a cabeça já passou
/// dele, o CORPO não. É o pulo para fora de um vão estreito rente a uma beirada.
fn low_block(sim: &mut SimWorld, face: f32, top: f32) {
    let half = 1.0;
    let bottom = 0.6;
    sim.world_mut().spawn((
        Name::new("Block"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: half,
                half_y: (top - bottom) * 0.5,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(face - half, (top + bottom) * 0.5)),
    ));
}

/// Pula e devolve `(pico, x no fim, x no tique 45)`.
fn jump(r: &mut Rig) -> (f32, f32, f32) {
    let (x0, y0) = pose(&r.sim);
    let mut tick = 0_u64;
    tick += 1;
    r.bridge.dispatch(&mut r.sim, true, tick);
    r.bridge.set_player_input(
        r.player,
        PlayerInput {
            drive: 0.0,
            jump: true,
            down: false,
            dash: false,
        },
    );
    let (mut peak, mut mid) = (y0, x0);
    for k in 0..90 {
        tick += 1;
        r.bridge.dispatch(&mut r.sim, true, tick);
        if k == 2 {
            r.bridge.set_player_input(r.player, PlayerInput::default());
        }
        let (x, y) = pose(&r.sim);
        peak = peak.max(y);
        if k == 45 {
            mid = x;
        }
    }
    let (x_end, _) = pose(&r.sim);
    (peak - y0, x_end - x0, mid - x0)
}

/// **O gate da wave.** Um encosto raso deixa de roubar o pulo.
///
/// ## Medido (2026-08-04, `measure_corner`, `corner_reach = 0,12`)
///
/// | encosto | pico SEM | pico COM |
/// |---|---|---|
/// | 0,10 m | 0,727 | **0,833** |
///
/// ⚠️ **O CONTROLE é a metade que dá sentido ao gate:** sem ele, "o pico é 0,83"
/// seria verdade também numa cena sem beirada nenhuma, e o teste passaria com a
/// assistência inteira deletada e a beirada fora do caminho.
#[test]
fn a_shallow_clip_no_longer_steals_the_jump() {
    let (off, _, _) = jump(&mut rig(0.10, 0.0, 20.0));
    assert!(
        off < FREE_PEAK - 0.05,
        "controle: sem assistencia o encosto TEM de roubar altura (pico {off:.3})"
    );

    let (on, _, _) = jump(&mut rig(0.10, 0.12, 20.0));
    assert!(
        (on - FREE_PEAK).abs() < 0.02,
        "com assistencia o pulo alcanca a altura LIVRE: {on:.3} vs {FREE_PEAK}"
    );
}

/// ⚠️ **Um TETO de verdade continua um teto** — e é o que separa a assistência
/// de um teletransporte.
///
/// **Mutação que deve sangrar:** devolver o alcance quando a busca falha.
#[test]
fn a_real_ceiling_stops_the_head_just_the_same() {
    // A cabeça INTEIRA tapada: nenhum deslocamento dentro do alcance livra.
    let (off, _, _) = jump(&mut rig(0.40, 0.0, 20.0));
    let (on, _, _) = jump(&mut rig(0.40, 0.12, 20.0));
    assert!(
        (on - off).abs() < 1.0e-3,
        "com a cabeca tapada os dois picos tem de ser o MESMO: {on:.3} vs {off:.3}"
    );
    assert!(on < FREE_PEAK - 0.1, "e bem abaixo do livre: {on:.3}");
}

/// ⚠️ **A assistência não empurra ninguém para dentro do que está ao lado.**
///
/// A beirada está à direita, então o escape seria para a esquerda — e é lá que
/// esta cena põe um bloco a 2 cm, **com o topo abaixo da cabeça** (ver
/// [`low_block`]: uma parede alta seria vista pelo perfil do teto e o gate ficaria
/// verde sem a lei que ele julga).
///
/// **Mutação que deve sangrar:** ignorar `side_clear` em `corner_escape`.
#[test]
fn the_assist_refuses_to_shove_the_body_into_what_is_beside_it() {
    // O topo do bloco fica logo abaixo da cabeça no instante do encosto (o
    // centro do corpo está por volta de 2,06 e o topo da caixa, 0,5 acima).
    let mut r = rig(0.10, 0.12, 20.0);
    low_block(&mut r.sim, -HALF_W - 0.02, 2.45);
    let (blocked, drift, _) = jump(&mut r);

    let (control, _, _) = jump(&mut rig(0.10, 0.0, 20.0));
    assert!(
        (blocked - control).abs() < 0.02,
        "com a saida ocupada o pulo tem de ser o do CONTROLE: {blocked:.3} vs {control:.3}"
    );
    assert!(
        drift.abs() < 0.03,
        "e o personagem nao pode ter sido empurrado contra o bloco: {drift:.3}"
    );
}

/// ⚠️ **A correção é um DESLOCAMENTO, não um impulso** — o gate que pina a
/// decisão de projeto do topo de `ph2d_platformer::corner`.
///
/// O oráculo precisa de `air_acceleration = 0` (o controle aéreo conserva o
/// arco, ver [`ph2d_platformer::WalkConfig::air_acceleration`]): com ele
/// ligado, um `boost` lateral seria FREADO pelo próprio controle aéreo e o gate
/// não distinguiria as duas implementações. Desligado, uma velocidade lateral
/// inventada sobrevive para sempre — e o personagem continua a deslizar muito
/// depois de a beirada ter ficado para trás.
///
/// **Mutação que deve sangrar:** aplicar o escape como `boost` (`escape / dt`)
/// em vez de `nudge_body`.
#[test]
fn the_correction_moves_the_body_and_does_not_launch_it() {
    let (peak, x_end, x_mid) = jump(&mut rig(0.10, 0.12, 0.0));
    assert!(
        (peak - FREE_PEAK).abs() < 0.05,
        "premissa: sem controle aereo o pulo ainda passa: {peak:.3}"
    );
    // O deslocamento total é da ordem do escape (poucos centímetros)...
    assert!(
        x_end.abs() < 0.25,
        "o desvio total tem de ser da ordem do escape, nao de um lancamento: {x_end:.3}"
    );
    // ...e ele PARA de crescer depois que a assistência deixa de agir.
    let after = (x_end - x_mid).abs();
    assert!(
        after < 0.02,
        "depois de livrar a quina o personagem nao pode CONTINUAR a deslizar: \
         {after:.3} m no ultimo terco do voo (total {x_end:.3})"
    );
}

/// **`corner_reach = 0` é o mundo de antes desta wave**, e o gate afirma a
/// identidade em vez de um limiar.
#[test]
fn zero_reach_is_the_world_before_this_wave() {
    let (a, xa, _) = jump(&mut rig(0.10, 0.0, 20.0));
    let (b, xb, _) = jump(&mut rig(0.10, 0.0, 20.0));
    assert_eq!((a, xa), (b, xb), "a cena e' determinista");
    assert!(
        xa.abs() < 1.0e-3,
        "sem alcance ninguem desloca ninguem: {xa:.4}"
    );
}
