//! **UMA ESCADA DE PRANCHAS APERTADA** — o caso degenerado da W12, medido e
//! **PINADO ONDE ESTÁ**, não corrigido.
//!
//! O irmão `platform_drop.rs` mede uma prancha SOLTA com muito espaço embaixo,
//! e é por isso que ele nunca viu isto: ali a caixa inteira do personagem chega
//! a ficar abaixo da prancha, então a lei de aposentadoria de hoje
//! (`body_maxs <= plat_mins`) funciona. Numa escada apertada ela nunca chega.
//!
//! # ⚠️ Estes gates afirmam o MUNDO DE HOJE, defeito incluído
//!
//! O `retire_drops` isentava este caso com uma frase — *"essa cena já está
//! quebrada sem descida nenhuma (o personagem não cabe ali)"* — e a metade
//! entre parênteses **é falsa**: ele cabe, e fica quieto. O primeiro gate
//! prova isso. O segundo pina o que a lei de hoje FAZ com essa cena, com o
//! número ao lado.
//!
//! ⚠️ **O segundo gate fica VERMELHO no dia em que alguém curar a lei**, e isso
//! é o desenho: a cura é uma decisão de produto (muda QUANDO uma prancha volta
//! a ser sólida, que é coisa que se sente e não se prova), e quem a fizer tem de
//! passar por aqui de propósito em vez de descobrir que um gate ficou verde
//! sozinho. As três leis já construídas e reprovadas estão nomeadas no aviso do
//! `retire_drops` — leia-o antes de tentar a quarta.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, OneWayPlatform, PhysicsBridge, PlatformPlayer,
    PlayerInput, RigidBody,
};

#[path = "platform_scene.rs"]
mod scene_fixture;

use scene_fixture::{FLOAT_HEIGHT, pose};

/// Meia-espessura de uma prancha.
const PLANK_HALF_Y: f32 = 0.1;
/// O topo da prancha de CIMA (a que ele atravessa).
const UPPER_TOP: f32 = PLANK_HALF_Y;
/// Meia-altura da cápsula do personagem (`half_height 0,3 + radius 0,2`).
const BODY_HALF: f32 = 0.5;

fn rest_over(top: f32) -> f32 {
    top + FLOAT_HEIGHT
}

struct Rig {
    sim: SimWorld,
    bridge: PhysicsBridge,
    player: ph2d_ecs::Entity,
}

fn plank(sim: &mut SimWorld, centre_y: f32, half_y: f32, name: &str) {
    sim.world_mut().spawn((
        Name::new(name),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 40.0,
                half_y,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, centre_y)),
        OneWayPlatform,
    ));
}

/// **Duas pranchas jump-through separadas por `gap`**, com o personagem em pé
/// na de cima e um chão sólido bem lá em baixo.
fn ladder(gap: f32) -> Rig {
    ladder_of(gap, PLANK_HALF_Y)
}

fn ladder_of(gap: f32, half: f32) -> Rig {
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
        Transform::from_translation(Vec2::new(0.0, -6.5)),
    ));
    plank(&mut sim, 0.0, half, "Upper");
    plank(&mut sim, -gap, half, "Lower");

    let player = sim
        .world_mut()
        .spawn((
            Name::new("Player"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Capsule {
                    half_height: 0.3,
                    radius: 0.2,
                },
                ..Collider::default()
            },
            LockRotation,
            PlatformPlayer {
                float_height: FLOAT_HEIGHT,
                // A quina e a memória do chão fora do caminho: esta cena não tem
                // teto nem plataforma móvel, e desligá-las explicitamente é o
                // que mantém o gate a medir UMA coisa.
                corner_reach: 0.0,
                lift_momentum: 0.0,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(0.0, rest_over(half))),
        ))
        .id();

    Rig {
        sim,
        bridge: PhysicsBridge::new(),
        player,
    }
}

fn settle(r: &mut Rig, ticks: u64, from: u64) -> u64 {
    let mut t = from;
    for _ in 0..ticks {
        t += 1;
        r.bridge.dispatch(&mut r.sim, true, t);
    }
    t
}

fn press(r: &mut Rig, input: PlayerInput, hold: u64, then: u64, from: u64) -> u64 {
    r.bridge.set_player_input(r.player, input);
    let t = settle(r, hold, from);
    r.bridge.set_player_input(r.player, PlayerInput::default());
    settle(r, then, t)
}

fn down_jump() -> PlayerInput {
    PlayerInput {
        drive: 0.0,
        jump: true,
        down: true,
        dash: false,
    }
}

fn jump_only() -> PlayerInput {
    PlayerInput {
        drive: 0.0,
        jump: true,
        down: false,
        dash: false,
    }
}

/// **O personagem CABE numa escada apertada** — a metade da nota antiga que a
/// medição derrubou.
///
/// Vão de 1,20 m: ele sobe 1,40 m acima do apoio, então a cabeça atravessa a
/// prancha de cima. Isso é o idioma de uma jump-through, não uma cena partida —
/// e ele fica **quieto** ali.
#[test]
fn a_short_gap_ladder_is_a_scene_the_character_fits_in() {
    let gap = 1.20_f32;
    let lower_top = -gap + PLANK_HALF_Y;

    let mut r = ladder(gap);
    // Posto no degrau de baixo: é essa a cena que a nota antiga chamava de
    // impossível.
    if let Some(mut t) = r.sim.world_mut().get_mut::<Transform>(r.player) {
        t.translation.y = rest_over(lower_top);
    }
    let mut t = settle(&mut r, 120, 0);
    let (_, settled) = pose(&r.sim);
    assert!(
        (settled - rest_over(lower_top)).abs() < 0.05,
        "ele tem de descansar no degrau de baixo ({:.3}), e esta' em {settled:.3}",
        rest_over(lower_top)
    );
    assert!(
        settled + BODY_HALF > UPPER_TOP,
        "premissa: a cabeca ({:.3}) atravessa a prancha de cima ({UPPER_TOP:.3})",
        settled + BODY_HALF
    );

    // E fica QUIETO — sem tremor, sem afundar.
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for _ in 0..60 {
        t = settle(&mut r, 1, t);
        let (_, y) = pose(&r.sim);
        lo = lo.min(y);
        hi = hi.max(y);
    }
    assert!(
        hi - lo < 0.001,
        "a pose tem de ser estavel, e variou {:.4} m em 60 tiques",
        hi - lo
    );
}

/// **A DESCIDA NUNCA SE APOSENTA NUMA ESCADA APERTADA — e este é o número.**
///
/// Depois de descer um degrau, um pulo simples **não** volta a pousar na
/// prancha de cima: ela ficou atravessável para sempre, e com ela todas as
/// outras da cena (o bit viaja no CORPO, e o gancho one-way limpa os contatos
/// com qualquer plataforma).
///
/// ⚠️ **Este gate afirma o defeito, não a cura.** Ele fica vermelho no dia em
/// que a lei mudar — ver o aviso do módulo.
#[test]
fn the_drop_never_retires_on_a_short_gap_ladder_and_this_is_its_number() {
    let gap = 1.20_f32;
    let lower_top = -gap + PLANK_HALF_Y;

    let mut r = ladder(gap);
    let t = settle(&mut r, 30, 0);
    let t = press(&mut r, down_jump(), 4, 120, t);

    let (_, dropped) = pose(&r.sim);
    assert!(
        (dropped - rest_over(lower_top)).abs() < 0.1,
        "premissa: a descida em si funciona, e ele para no degrau de baixo ({:.3}); esta' em {dropped:.3}",
        rest_over(lower_top)
    );

    press(&mut r, jump_only(), 6, 150, t);
    let (_, climbed) = pose(&r.sim);
    assert!(
        (climbed - rest_over(lower_top)).abs() < 0.1,
        "HOJE a prancha de cima fica fantasma e ele volta ao degrau de baixo ({:.3}); \
         parou em {climbed:.3} — se isto ficou perto de {:.3}, a lei foi CURADA e este \
         gate tem de ser reescrito de proposito (ver o aviso do modulo)",
        rest_over(lower_top),
        rest_over(UPPER_TOP)
    );
}

/// **A SEGUNDA BORDA: logo acima do limiar ele é ARREMESSADO DE VOLTA — e este
/// é o número.**
///
/// A lei de hoje aposenta a descida quando a caixa do personagem está
/// inteiramente abaixo da prancha, e **isso acontece a meio da queda**. Numa
/// prancha grossa (meia-espessura 0,15 — a da cena 91) há uma faixa de vãos em
/// que ela volta a ser sólida com o personagem ainda a atravessá-la, e o
/// contato atira-o para cima: **ele não desce de todo**.
///
/// Medido, a faixa de arremesso é **1,75 m a 1,85 m** (com `1,90+` a funcionar
/// e `1,70-` a virar fantasma) — ou seja o mecanismo tem uma janela útil e as
/// **DUAS** bordas dela são defeitos. A cena 91 usa `2,00`, dez centímetros
/// acima da borda, o que ninguém sabia.
///
/// ⚠️ **Este gate afirma o defeito, não a cura** — e ele é o irmão do de cima:
/// os dois têm de mudar juntos no dia em que a lei mudar.
#[test]
fn a_thick_plank_ladder_throws_him_back_and_this_is_its_number() {
    const THICK: f32 = 0.15;
    let gap = 1.80_f32;
    let top = rest_over(THICK);

    let mut r = ladder_of(gap, THICK);
    let t = settle(&mut r, 30, 0);
    press(&mut r, down_jump(), 4, 130, t);

    let (_, after) = pose(&r.sim);
    assert!(
        (after - top).abs() < 0.1,
        "HOJE o vao 1,80 com prancha grossa ARREMESSA-O de volta ao degrau de cima ({top:.3}); \
         ele parou em {after:.3} — se isto passou a descer um degrau ({:.3}), a lei foi \
         CURADA e este gate tem de ser reescrito de proposito (ver o aviso do modulo)",
        rest_over(-gap + THICK)
    );
}

/// **O CONTROLE: com espaço a mesma escada funciona.**
///
/// ⚠️ Sem ele, *"a prancha fica fantasma"* poderia ser lido como *"a retirada
/// nunca funciona"* — e ela funciona, o que é precisamente o que torna o caso
/// apertado um defeito em vez de uma feature ausente. Este gate fica VERDE
/// antes e depois de qualquer cura.
#[test]
fn a_wide_gap_ladder_retires_the_drop_as_it_should() {
    let gap = 2.0_f32;
    let lower_top = -gap + PLANK_HALF_Y;

    let mut r = ladder(gap);
    let t = settle(&mut r, 30, 0);
    let t = press(&mut r, down_jump(), 4, 120, t);
    let (_, dropped) = pose(&r.sim);
    assert!(
        (dropped - rest_over(lower_top)).abs() < 0.1,
        "vao largo: esperado {:.3}, e parou em {dropped:.3}",
        rest_over(lower_top)
    );

    press(&mut r, jump_only(), 6, 150, t);
    let (_, climbed) = pose(&r.sim);
    assert!(
        (climbed - rest_over(UPPER_TOP)).abs() < 0.1,
        "vao largo: a prancha de cima TEM de voltar a ser chao ({:.3}), e ele parou em {climbed:.3}",
        rest_over(UPPER_TOP)
    );
}
