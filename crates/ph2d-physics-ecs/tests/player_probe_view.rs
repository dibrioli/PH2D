//! **A LEITURA DOS SENSORES** (`W-Probes`) — o que a ponte publica sobre onde
//! cada sensor do player olhou, e o que ele respondeu.
//!
//! ⚠️ **O que estes gates protegem não é o desenho, é a PORTA:** a geometria
//! publicada tem de ser a MESMA que foi castada. Um desenho derivado por uma
//! segunda aritmética divergiria no primeiro dia em que alguém mexesse numa das
//! duas — e o sintoma seria um overlay que mente sobre o que o produto mede,
//! que é a única coisa pior do que overlay nenhum.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, PlayerInput,
    ProbeKind, ProbeMark, ProbeShape, ProbeState, RigidBody,
};

const HALF_H: f32 = 0.3;
const RADIUS: f32 = 0.2;
const FLOAT: f32 = 0.9;
const CLING: f32 = 0.1;
const WALL_REACH: f32 = 0.15;
const CORNER_REACH: f32 = 0.12;
const CROUCH: f32 = 0.5;

struct Rig {
    sim: SimWorld,
    bridge: PhysicsBridge,
    player: ph2d_ecs::Entity,
}

impl Rig {
    fn run(&mut self, from: u64, ticks: u64, input: PlayerInput) -> u64 {
        let mut t = from;
        for _ in 0..ticks {
            self.bridge.set_player_input(self.player, input);
            t += 1;
            self.bridge.dispatch(&mut self.sim, true, t);
        }
        t
    }

    fn marks(&self) -> Vec<ProbeMark> {
        self.bridge.player_probe_marks().to_vec()
    }

    fn of(&self, kind: ProbeKind) -> Vec<ProbeMark> {
        self.marks()
            .into_iter()
            .filter(|m| m.kind == kind)
            .collect()
    }
}

/// Chão comprido + player. `armed` liga as três capacidades condicionais.
fn rig(armed: bool) -> Rig {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 40.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, -0.5)),
    ));
    let player = sim
        .world_mut()
        .spawn((
            Name::new("Player"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Capsule {
                    half_height: HALF_H,
                    radius: RADIUS,
                },
                ..Collider::default()
            },
            LockRotation,
            PlatformPlayer {
                float_height: FLOAT,
                cling_distance: CLING,
                wall_reach: WALL_REACH,
                wall_slide_speed: if armed { 2.0 } else { 0.0 },
                wall_jump_height: if armed { 2.0 } else { 0.0 },
                corner_reach: if armed { CORNER_REACH } else { 0.0 },
                crouch_height: if armed { CROUCH } else { 0.0 },
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(0.0, FLOAT)),
        ))
        .id();
    Rig {
        sim,
        bridge: PhysicsBridge::new(),
        player,
    }
}

fn as_ray(m: &ProbeMark) -> ([f32; 2], [f32; 2], f32, Option<f32>) {
    match m.shape {
        ProbeShape::Ray {
            origin,
            dir,
            reach,
            hit,
        } => (origin, dir, reach, hit),
        _ => panic!("esperava um raio, veio {:?}", m.shape),
    }
}

/// **A PERNA é lida em TODO tique, e o alcance publicado é o da LEI.**
///
/// ⚠️ O alcance é `float_height + cling_distance` porque é isso que a lei
/// considera chão — e ele é DERIVADO de propósito (um knob próprio seria a
/// segunda porta para *"até onde a perna alcança"*). O que faltava era ele ser
/// **visto**, e é isso que esta marca entrega.
#[test]
fn the_leg_is_read_every_tick_and_publishes_the_reach_the_law_uses() {
    let mut r = rig(true);
    r.run(0, 30, PlayerInput::default());
    let g = r.of(ProbeKind::Ground);
    assert_eq!(g.len(), 1, "uma perna, um raio");
    let (_, dir, reach, hit) = as_ray(&g[0]);
    assert_eq!(dir, [0.0, -1.0], "a perna olha para BAIXO");
    assert!(
        (reach - (FLOAT + CLING)).abs() < 1.0e-6,
        "alcance publicado {reach} != float+cling {}",
        FLOAT + CLING
    );
    assert_eq!(g[0].state, ProbeState::Hit, "ha chao debaixo dele");
    let d = hit.expect("um Hit carrega a distancia");
    assert!(
        (d - FLOAT).abs() < 0.05,
        "ele paira a {FLOAT}, entao o chao esta a ~{FLOAT}: {d}"
    );
}

/// **Uma capacidade DESARMADA não tem sensor nenhum na tela.**
///
/// ⚠️ *"Inerte neste tique"* e *"esta capacidade está desligada"* são coisas
/// diferentes, e só a primeira é [`ProbeState::Idle`]. Desenhar seis raios
/// apagados à volta de todo personagem de toda cena onde o pulo de parede nunca
/// foi ligado seria ruído permanente.
#[test]
fn a_disarmed_capability_draws_nothing_at_all() {
    let mut r = rig(false);
    r.run(0, 30, PlayerInput::default());
    assert!(r.of(ProbeKind::Wall).is_empty(), "parede desarmada");
    assert!(r.of(ProbeKind::Corner).is_empty(), "quina desarmada");
    assert!(r.of(ProbeKind::Side).is_empty(), "folga lateral desarmada");
    assert!(r.of(ProbeKind::Headroom).is_empty(), "agachar desarmado");
    assert_eq!(
        r.of(ProbeKind::Ground).len(),
        1,
        "a perna nao e' condicional e continua la'"
    );
}

/// **ARMADO e não perguntado é `Idle`, COM o alcance** — a metade nova.
///
/// Parado no chão: a lei não casta o flanco (não está no ar, não empurra), não
/// casta a quina (não sobe) e não casta o teto (não estava agachado). As três
/// marcas existem mesmo assim, porque é o ALCANCE que o artista afina.
#[test]
fn armed_but_not_asked_is_idle_and_still_shows_the_reach() {
    let mut r = rig(true);
    r.run(0, 30, PlayerInput::default());

    for k in [ProbeKind::Wall, ProbeKind::Corner, ProbeKind::Side] {
        let ms = r.of(k);
        assert!(!ms.is_empty(), "{k:?} armado tem marca");
        assert!(
            ms.iter().all(|m| m.state == ProbeState::Idle),
            "parado no chao, {k:?} nao e' perguntado: {:?}",
            ms.iter().map(|m| m.state).collect::<Vec<_>>()
        );
    }
    // E o alcance publicado do flanco e' o da lei: meia-largura + wall_reach.
    let w = r.of(ProbeKind::Wall);
    let (_, _, reach, _) = as_ray(&w[0]);
    assert!(
        (reach - (RADIUS + WALL_REACH)).abs() < 1.0e-5,
        "alcance do flanco {reach} != meia-largura + reach {}",
        RADIUS + WALL_REACH
    );
}

/// **Sem direção, o flanco é publicado dos DOIS lados.**
///
/// ⚠️ O sensor lateral olha para onde o jogador empurra. Parado não há lado, e
/// escolher um seria desenhar um alcance no sítio errado.
#[test]
fn with_no_drive_the_flank_is_published_on_both_sides() {
    let mut r = rig(true);
    r.run(0, 30, PlayerInput::default());
    let w = r.of(ProbeKind::Wall);
    let sides: Vec<f32> = w.iter().map(|m| as_ray(m).1[0]).collect();
    assert!(
        sides.iter().any(|s| *s < 0.0) && sides.iter().any(|s| *s > 0.0),
        "os dois lados: {sides:?}"
    );

    // E com direcao, so' o lado que a lei olha.
    let mut r = rig(true);
    r.run(
        0,
        30,
        PlayerInput {
            drive: 1.0,
            ..PlayerInput::default()
        },
    );
    let w = r.of(ProbeKind::Wall);
    assert!(
        w.iter().all(|m| as_ray(m).1[0] > 0.0),
        "empurrando para a direita, so' o lado direito"
    );
}

/// **As alturas do flanco são as da porta da LEI**, não uma grade local.
///
/// ⚠️ Este é o gate da PORTA ÚNICA: os três raios nascem no centro do corpo com
/// os deslocamentos de [`wall_offsets`], e é exatamente isso que a leitura
/// publica. Uma aritmética local do lado do desenho poria o sensor visível meio
/// corpo acima do que foi castado.
///
/// [`wall_offsets`]: ph2d_platformer::wall_offsets
#[test]
fn the_flank_heights_come_from_the_laws_door() {
    let mut r = rig(true);
    r.run(
        0,
        30,
        PlayerInput {
            drive: 1.0,
            ..PlayerInput::default()
        },
    );
    let w = r.of(ProbeKind::Wall);
    assert_eq!(w.len(), 3, "o flanco tem tres amostras");
    let cy = as_ray(&w[0]).0[1];
    let half = HALF_H + RADIUS;
    let mut ys: Vec<f32> = w.iter().map(|m| as_ray(m).0[1] - cy).collect();
    ys.sort_by(f32::total_cmp);
    let want = ph2d_platformer::wall_offsets(half);
    let mut want: Vec<f32> = want.to_vec();
    want.sort_by(f32::total_cmp);
    for (got, exp) in ys.iter().zip(want.iter()) {
        assert!(
            (got - exp).abs() < 1.0e-4,
            "alturas publicadas {ys:?} != porta da lei {want:?}"
        );
    }
}

/// **O agachar é uma FORMA VARRIDA, e o deslocamento é a subida.**
///
/// ⚠️ Depois da `W-ShapeCast` este sensor varre o corpo; publicá-lo como um raio
/// faria o overlay desenhar uma linha que o produto não usa.
#[test]
fn the_crouch_sensor_publishes_a_swept_shape_whose_offset_is_the_rise() {
    let mut r = rig(true);
    r.run(0, 30, PlayerInput::default());
    let h = r.of(ProbeKind::Headroom);
    assert_eq!(h.len(), 1);
    match h[0].shape {
        ProbeShape::Sweep { body, offset } => {
            assert_eq!(body, r.player, "a forma varrida e' a do PLAYER");
            assert!(offset[0].abs() < 1.0e-6, "ele sobe reto");
            assert!(
                (offset[1] - (FLOAT - CROUCH)).abs() < 1.0e-5,
                "a subida e' de pe' menos agachado: {offset:?}"
            );
        }
        other => panic!("esperava uma varredura, veio {other:?}"),
    }
}

/// **Agachado e a soltar, o teto RESPONDE** — e responde `Clear` num céu limpo.
#[test]
fn releasing_the_crouch_under_open_sky_reads_clear() {
    let mut r = rig(true);
    let t = r.run(
        0,
        60,
        PlayerInput {
            down: true,
            ..PlayerInput::default()
        },
    );
    r.run(t, 1, PlayerInput::default());
    let h = r.of(ProbeKind::Headroom);
    assert_eq!(
        h[0].state,
        ProbeState::Clear,
        "ceu limpo: perguntou e nao achou"
    );
}

/// **Sem passo não há sensor** — o `hold` limpa a leitura.
///
/// ⚠️ **PAUSADO não é `hold`, e a distinção é deliberada:** um quadro pausado
/// sobre o mesmo tique não limpa nada, e a leitura descreve o ÚLTIMO TIQUE — a
/// mesma propriedade que as cruzes de contato, os triggers e a linha d'água já
/// têm (todas escritas pelo `step`). Consequência nomeada: arrastar um corpo com
/// o relógio parado move o desenho e deixa os sensores onde o último tique os
/// leu, até o relógio andar. Limpar aqui seria APAGAR os sensores exatamente no
/// gesto em que o artista os quer olhar.
///
/// ⚠️ Marcas de um tique que já não corre descrevem um mundo que o artista pode
/// desmontar com a mão: a lição que os contatos pagaram, aplicada ao terceiro
/// canal de leitura antes de ela custar um smoke.
#[test]
fn holding_the_clock_clears_the_reading() {
    let mut r = rig(true);
    let t = r.run(0, 30, PlayerInput::default());
    assert!(!r.marks().is_empty(), "andando, ha' leitura");
    // Fisica DESARMADA: a porta que o toggle Physics do transporte usa.
    r.bridge.hold(&mut r.sim, t);
    assert!(
        r.marks().is_empty(),
        "com a fisica em hold nao ha' sensor a reportar"
    );
}

/// **O perfil da quina publica o vão que ele varre**, e ele é o da porta.
#[test]
fn the_ceiling_profile_publishes_the_span_the_law_scans() {
    let mut r = rig(true);
    r.run(0, 30, PlayerInput::default());
    let c = r.of(ProbeKind::Corner);
    assert_eq!(c.len(), 1, "um perfil");
    match c[0].shape {
        ProbeShape::Profile {
            half_width, reach, ..
        } => {
            assert!(
                (half_width - RADIUS).abs() < 1.0e-4,
                "meia-largura da caixa: {half_width}"
            );
            assert!(
                (reach - CORNER_REACH).abs() < 1.0e-6,
                "o alcance lateral autorado: {reach}"
            );
        }
        other => panic!("esperava um perfil, veio {other:?}"),
    }
}
