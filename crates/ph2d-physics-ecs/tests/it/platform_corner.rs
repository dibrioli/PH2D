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

use crate::platform_scene as scene_fixture;

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
/// A Jump Height dos gates de ponto (a de partida do produto).
const JUMP_HEIGHT: f32 = 2.0;

struct Rig {
    sim: SimWorld,
    bridge: PhysicsBridge,
    player: ph2d_ecs::Entity,
}

/// Chão + beirada cobrindo `clip` metros do lado direito da cabeça. `reach = 0`
/// é o CONTROLE (a assistência desligada), e `air` deixa o gate escolher se o
/// controle aéreo freia ou conserva o arco.
fn rig(clip: f32, reach: f32, air: f32) -> Rig {
    rig_at(Some(clip), reach, air, JUMP_HEIGHT)
}

/// [`rig`] à Jump Height `h`; `clip = None` é a cena SEM beirada (o pulo livre).
fn rig_at(clip: Option<f32>, reach: f32, air: f32, h: f32) -> Rig {
    let (mut sim, bridge, player) = scene(0.0, 0.0);
    let half_x = 2.0;
    if let Some(clip) = clip {
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
    }
    let mut q = sim
        .world_mut()
        .try_query::<(&Name, &mut PlatformPlayer)>()
        .expect("a query do player");
    for (n, mut p) in q.iter_mut(sim.world_mut()) {
        if n.as_str() == "Player" {
            p.corner_reach = reach;
            p.air_acceleration = air;
            p.jump_height = h;
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
    let (peak, x_end, mid, _) = jump_vy(r);
    (peak, x_end, mid)
}

/// [`jump`] e a maior subida por tique (m/s): o botão solto no 3º tique, o pulo CORTADO.
fn jump_vy(r: &mut Rig) -> (f32, f32, f32, f32) {
    let (x0, y0) = pose(&r.sim);
    let mut tick = 0_u64;
    tick += 1;
    r.bridge.dispatch(&mut r.sim, true, tick);
    r.bridge.set_player_input(
        r.player,
        PlayerInput {
            drive_y: 0.0,
            drive: 0.0,
            jump: true,
            down: false,
            dash: false,
            grab: false,
        },
    );
    let (mut peak, mut mid, mut vy, mut prev) = (y0, x0, 0.0_f32, y0);
    for k in 0..90 {
        tick += 1;
        r.bridge.dispatch(&mut r.sim, true, tick);
        if k == 2 {
            r.bridge.set_player_input(r.player, PlayerInput::default());
        }
        let (x, y) = pose(&r.sim);
        peak = peak.max(y);
        vy = vy.max((y - prev) * 60.0);
        prev = y;
        if k == 45 {
            mid = x;
        }
    }
    let (x_end, _) = pose(&r.sim);
    (peak - y0, x_end - x0, mid - x0, vy)
}

/// **O gate da wave, em VARREDURA** (Jump Height 1,0–2,6, o pulo cortado no 3º tique): com a
/// assistência o encosto raso (0,10 m) deixa de roubar o pulo (o pico livre ±0,02), um teto de
/// verdade (0,40 m) pára a cabeça igual com e sem ela, ela não lança (subida ≤ a do livre + 0,05
/// m/s) nem desloca mais que o escape (≤ 0,13 m, e pára de deslizar), e onde a cabeça alcança a
/// beirada o CONTROLO sem ajuda perde altura — a metade que dá sentido ao gate.
///
/// Um ponto só (a 2,1056 m, a régua da serra do pulo cortado, GEF 04) deixava a assistência
/// correcta numa altura e calada nas vizinhas. **Mutações que sangram:** `nudge` zerado; devolver
/// o alcance quando a busca falha; o escape como `boost`.
#[test]
fn a_quina_ajuda_em_toda_a_faixa_de_alturas() {
    let mut falhas = Vec::new();
    let mut alcancou = 0;
    for i in 0..=16 {
        let h = 1.0 + i as f32 * 0.1;
        let (free, _, _, vy_free) = jump_vy(&mut rig_at(None, 0.0, 20.0, h));
        let (off, _, _) = jump(&mut rig_at(Some(0.10), 0.0, 20.0, h));
        let (on, _, _, vy_on) = jump_vy(&mut rig_at(Some(0.10), 0.12, 20.0, h));
        let (teto_off, _, _) = jump(&mut rig_at(Some(0.40), 0.0, 20.0, h));
        let (teto_on, _, _) = jump(&mut rig_at(Some(0.40), 0.12, 20.0, h));
        let (_, x_end, x_mid) = jump(&mut rig_at(Some(0.10), 0.12, 0.0, h));
        eprintln!(
            "h={h:.1}: livre {free:.3} · sem {off:.3} · com {on:.3} · teto {teto_off:.3}/{teto_on:.3} \
             · vy {vy_free:.3}/{vy_on:.3} · desvio {x_end:.3} ({x_mid:.3})"
        );
        // A cabeça alcança a beirada (o teto de verdade tira ≥ 5 cm do pico livre): o controlo
        // perde mais que a tolerância da ajuda, e ≥ 5 cm quando a beirada tira ≥ 10 (medido 12/11:
        // a 1,6 m o teto tira 0,052 e o encosto 0,025; a 2,0 m, 0,175 e 0,148).
        let gap = free - teto_off;
        if gap >= 0.05 {
            alcancou += 1;
            let perde = if gap >= 0.1 { 0.05 } else { 0.02 };
            if off > free - perde {
                falhas.push(format!(
                    "h={h:.1}: o controlo não perde altura ({off:.3} vs {free:.3})"
                ));
            }
        }
        if (on - free).abs() >= 0.02 {
            falhas.push(format!("h={h:.1}: com ajuda {on:.3} ≠ livre {free:.3}"));
        }
        if (teto_on - teto_off).abs() >= 1.0e-3 {
            falhas.push(format!(
                "h={h:.1}: o teto deixa passar ({teto_on:.3} vs {teto_off:.3})"
            ));
        }
        if vy_on > vy_free + 0.05 {
            falhas.push(format!("h={h:.1}: lança ({vy_on:.3} > {vy_free:.3})"));
        }
        if x_end.abs() > 0.13 || (x_end - x_mid).abs() >= 0.02 {
            falhas.push(format!(
                "h={h:.1}: desvio {x_end:.3} (tique 45: {x_mid:.3})"
            ));
        }
    }
    assert!(
        alcancou >= 8,
        "premissa: a cabeça alcança a beirada em metade da faixa ({alcancou})"
    );
    assert!(falhas.is_empty(), "a quina falha em: {falhas:?}");
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

/// **`corner_reach = 0` é o mundo de antes desta wave**: a lei não desloca ninguém.
///
/// ⚠️ Sobra a GEOMETRIA: a calota da cápsula toca a quina do lábio numa normal inclinada e, sem
/// atrito (Bug #14), escorrega para fora ~1 cm (medido −0,0099) e pára — o escape que a lei dá
/// chega a 0,13 m. Até o #14 o atrito a segurava, e este gate pedia `< 1 mm`.
#[test]
fn zero_reach_is_the_world_before_this_wave() {
    let (a, xa, mid) = jump(&mut rig(0.10, 0.0, 20.0));
    let (b, xb, _) = jump(&mut rig(0.10, 0.0, 20.0));
    assert_eq!((a, xa), (b, xb), "a cena e' determinista");
    assert!(
        xa.abs() < 0.02 && (xa - mid).abs() < 1.0e-3,
        "sem alcance so' a calota escorrega da quina, e para: {xa:.4} (tique 45: {mid:.4})"
    );
}
