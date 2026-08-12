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
            skin: _,
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
    // ⚠️ **Um por PÉ** (`W-FootFan`): a perna é um leque, e a contagem sai da
    // porta do produto em vez de um literal — senão o dia em que o default
    // mudar deixa este gate a afirmar um número que ninguém escolheu.
    let feet = usize::from(PlatformPlayer::default().foot_samples);
    assert_eq!(g.len(), feet, "uma marca por pe' da perna");
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
        usize::from(PlatformPlayer::default().foot_samples),
        "a perna nao e' condicional e continua la', com todos os pes"
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
    let want = ph2d_platformer::wall_offsets(half, ph2d_platformer::WALL_SAMPLES, 1.0);
    // ⚠️ A CONTAGEM, nao o array: desde a W-Probes2 ele e' dimensionado pelo
    // TETO, e a cauda nao usada fica em zero.
    let mut want: Vec<f32> = want[..ph2d_platformer::WALL_SAMPLES].to_vec();
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

/// **Sem passo não há RESPOSTA — mas o sensor continua lá** (`W-Probes2`).
///
/// ⚠️ **Este gate afirmava o contrário e a decisão era minha:** a `W-Probes`
/// escreveu que o `hold` *limpa a leitura*, tratando os sensores como o terceiro
/// canal ao lado dos contatos e dos triggers. São coisas diferentes — um contato
/// descreve um EVENTO que aconteceu e some com a corrida que o produziu; o
/// alcance de um sensor é uma propriedade do CORPO, que existe com o solver
/// desligado. E o gesto de afinar um alcance é **encostar o corpo na parede sem
/// relógio nenhum**, ou seja exatamente o quadro em que a lei antiga apagava
/// tudo.
///
/// O que sobrevive da frase antiga é a metade verdadeira: nenhum passo correu,
/// logo **nenhum sensor perguntou nada** — todo estado sai `Idle`. Marcas com
/// `Hit` aqui descreveriam um mundo que o artista pode desmontar com a mão, que
/// é a lição que os contatos pagaram.
#[test]
fn holding_the_clock_keeps_the_reach_and_drops_the_answers() {
    let mut r = rig(true);
    let t = r.run(0, 30, PlayerInput::default());
    assert!(!r.marks().is_empty(), "andando, ha' leitura");
    assert!(
        r.marks().iter().any(|m| m.state != ProbeState::Idle),
        "andando, algum sensor RESPONDEU"
    );

    // Fisica DESARMADA: a porta que o toggle Physics do transporte usa.
    r.bridge.hold(&mut r.sim, t);
    assert!(
        !r.marks().is_empty(),
        "em hold o ALCANCE continua desenhado — e' o que o artista afina"
    );
    assert!(
        r.marks().iter().all(|m| m.state == ProbeState::Idle),
        "e nenhum deles RESPONDE, porque a lei nao correu: {:?}",
        r.marks().iter().map(|m| m.state).collect::<Vec<_>>()
    );
}

/// **A leitura SEGUE o corpo que o artista arrasta com o relógio parado.**
///
/// ⚠️ **RED-first, com o número do report:** corpo movido de `x = 2.000` para
/// `x = 5.000` no MESMO tique, e a perna publicada continuava em **`x = 2.000`**
/// — os sensores descreviam o último tique simulado. `drive_players` só corre no
/// ramo que dá passo, e o ramo `Equal` (pausado) não o alcança.
#[test]
fn the_reading_follows_a_body_dragged_while_the_clock_stands_still() {
    let mut r = rig(true);
    let t = r.run(0, 30, PlayerInput::default());
    let leg_x = |r: &Rig| {
        r.of(ProbeKind::Ground)
            .first()
            .map(|m| match m.shape {
                ProbeShape::Ray { origin, .. } => origin[0],
                _ => panic!("a perna e' um raio"),
            })
            .expect("ha' uma perna")
    };
    let before = leg_x(&r);

    // O artista arrasta o corpo 3 m para a direita — o MESMO tique.
    {
        let mut tr = r
            .sim
            .world_mut()
            .get_mut::<Transform>(r.player)
            .expect("o player tem Transform");
        tr.translation.x += 3.0;
    }
    r.bridge.dispatch(&mut r.sim, false, t);

    let after = leg_x(&r);
    assert!(
        (after - before - 3.0).abs() < 0.05,
        "a perna acompanha os 3 m: {before:.3} -> {after:.3}"
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

/// **A CONTAGEM AUTORADA chega aos raios que a ponte casta** (`W-Probes2`).
///
/// ⚠️ Esta é a quarta condição da política desta linha — *a sequência leva a
/// algum lugar*. As outras três (o componente existe, a row pinta e regista, o
/// clique chega ao barramento) podem estar todas verdes com o número a morrer
/// no componente, e o sintoma seria um slider que se mexe e não faz nada.
///
/// ⚠️ **Report do Enio:** *"não temos inputs para ajustes dos tamanhos e posições
/// dos sensores nem a quantidade de sensores"*. A `W-Probes` fechou a metade (b)
/// da §4.55 medindo que *cada NÚMERO tem row* — e não fez a pergunta que
/// faltava, que é sobre a GEOMETRIA das amostras.
#[test]
fn the_authored_sample_count_reaches_the_rays_the_bridge_casts() {
    let count = |n: u16| {
        let mut r = rig(true);
        {
            let mut p = r
                .sim
                .world_mut()
                .get_mut::<PlatformPlayer>(r.player)
                .expect("o player tem config");
            p.wall_samples = n;
        }
        r.run(0, 30, PlayerInput::default());
        // Sem direcao o flanco e' publicado dos DOIS lados, logo 2x a contagem.
        r.of(ProbeKind::Wall).len()
    };
    assert_eq!(count(3), 6, "o default sao 3 por lado");
    assert_eq!(count(9), 18, "nove pedidos, nove castados");
    assert_eq!(
        count(8),
        18,
        "uma contagem PAR sobe para a impar seguinte (o meio e' a ancora do desempate)"
    );
}

/// **O ESPALHAMENTO autorado move onde os raios de fora se sentam.**
#[test]
fn the_authored_spread_moves_where_the_outer_rays_sit() {
    let span = |spread: f32| {
        let mut r = rig(true);
        {
            let mut p = r
                .sim
                .world_mut()
                .get_mut::<PlatformPlayer>(r.player)
                .expect("o player tem config");
            p.wall_spread = spread;
        }
        r.run(0, 30, PlayerInput::default());
        let ys: Vec<f32> = r
            .of(ProbeKind::Wall)
            .iter()
            .map(|m| as_ray(m).0[1])
            .collect();
        let lo = ys.iter().copied().fold(f32::INFINITY, f32::min);
        let hi = ys.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        hi - lo
    };
    let full = span(1.0);
    let half = span(0.5);
    assert!(
        (half - full * 0.5).abs() < 0.02,
        "metade do espalhamento, metade do vao: {full:.3} -> {half:.3}"
    );
    assert!(
        span(0.0) < 1.0e-4,
        "espalhamento zero junta tudo na cintura: {}",
        span(0.0)
    );
}

/// **A ANTECEDÊNCIA autorada muda quanto o leque da quina sobe.**
///
/// ⚠️ É ela que dá ALTURA à haste do perfil — o `rise` vale
/// `rel_up · dt · lookahead`, e era uma `const` até esta wave.
#[test]
fn the_authored_lookahead_changes_how_far_the_ceiling_profile_looks() {
    let rise = |ahead: f32| {
        let mut r = rig(true);
        {
            let mut p = r
                .sim
                .world_mut()
                .get_mut::<PlatformPlayer>(r.player)
                .expect("o player tem config");
            p.corner_lookahead = ahead;
        }
        // A subir: e' o unico regime em que a lei pergunta a quina.
        r.run(
            0,
            3,
            PlayerInput {
                jump: true,
                ..PlayerInput::default()
            },
        );
        r.of(ProbeKind::Corner)
            .first()
            .map(|m| match m.shape {
                ProbeShape::Profile { rise, .. } => rise,
                _ => panic!("o perfil e' um Profile"),
            })
            .unwrap_or(f32::NAN)
    };
    let (two, four) = (rise(2.0), rise(4.0));
    assert!(
        two > 0.0 && (four - two * 2.0).abs() < two * 0.05,
        "o dobro da antecedencia, o dobro da subida: {two:.5} -> {four:.5}"
    );
    assert!(
        rise(0.0).abs() < 1.0e-9,
        "zero e' *sem antecedencia*, nao um desligar: {}",
        rise(0.0)
    );
}
