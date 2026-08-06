//! **O que um vão MENOR que o personagem faz com a descida** — a medição que
//! abre (e fecha) o item degenerado da W12.
//!
//! `cargo test -p ph2d-physics-ecs --test measure_drop_retire -- --ignored --nocapture`
//!
//! ⚠️ **Sondas, não gates.** O que elas mediram vive em dois lugares: o aviso
//! corrigido de `bridge::player::retire_drops` (o defeito e as três leis
//! reprovadas) e os gates de `platform_drop_ladder.rs` (o número pinado).
//!
//! # O que ficou provado, em ordem
//!
//! **(1) A nota que isentava o caso era falsa.** Ela dizia *"o personagem não
//! cabe ali"*; medido, entre **1,15 m e 1,55 m** de vão ele fica em pé no
//! degrau de baixo, **perfeitamente estável**, com a cabeça a atravessar o de
//! cima — que é o idioma de uma prancha jump-through.
//!
//! **(2) O preço é a cena inteira.** O bit da descida viaja no CORPO e o gancho
//! limpa os contatos com **qualquer** plataforma one-way, então uma descida que
//! nunca se aposenta apaga todas as pranchas da cena, para sempre.
//!
//! **(3) A virada em que uma prancha sólida CUSPE o personagem é
//! `centro do corpo == base da prancha`**, e segue a base em quatro espessuras
//! (0,05 / 0,10 / 0,20 / 0,30 ⇒ −0,05 / −0,10 / −0,20 / −0,30).
//!
//! **(4) ⚠️ E essa virada é EM REPOUSO — aplicá-la a uma QUEDA não funciona.**
//! Foi o que reprovou a cura: numa prancha de meia-espessura 0,15 o personagem
//! cruza a linha a cair, a prancha volta a ser sólida a cortar-lhe o peito, e
//! ele é **arremessado dois degraus acima** (desce a 5,79, repousa em 7,05).

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, OneWayPlatform, PhysicsBridge, PlatformPlayer,
    PlayerInput, RigidBody,
};

/// A altura de flutuação do personagem — o mesmo número da fixture partilhada.
const FLOAT_HEIGHT: f32 = 0.9;
/// Meia-altura da cápsula (`half_height 0,3 + radius 0,2`).
const BODY_HALF: f32 = 0.5;
/// Meia-espessura de uma prancha.
const PLANK_HALF_Y: f32 = 0.1;
/// O topo da prancha de cima.
const UPPER_TOP: f32 = PLANK_HALF_Y;
/// O topo do chão sólido lá em baixo.
const FLOOR_TOP: f32 = -6.0;

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

fn player_on(sim: &mut SimWorld, y: f32) -> ph2d_ecs::Entity {
    sim.world_mut()
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
                corner_reach: 0.0,
                lift_momentum: 0.0,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(0.0, y)),
        ))
        .id()
}

/// **Duas pranchas jump-through separadas por `gap`, e o personagem em pé na de
/// cima.** O chão sólido lá em baixo separa *"parou numa prancha"* de *"caiu do
/// mundo"*.
fn stack(gap: f32) -> Rig {
    stack_of(gap, PLANK_HALF_Y)
}

fn stack_of(gap: f32, upper_half: f32) -> Rig {
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
        Transform::from_translation(Vec2::new(0.0, FLOOR_TOP - 0.5)),
    ));
    plank(&mut sim, 0.0, upper_half, "Upper");
    plank(&mut sim, -gap, PLANK_HALF_Y, "Lower");
    let player = player_on(&mut sim, UPPER_TOP + FLOAT_HEIGHT);
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

fn y_of(sim: &SimWorld) -> f32 {
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    let mut y = f32::NAN;
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == "Player" {
            y = t.translation.y;
        }
    }
    y
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

/// **(1) O personagem CABE numa pilha de vão curto?** — a premissa da nota.
#[test]
#[ignore]
fn measure_whether_a_short_gap_is_a_broken_scene() {
    eprintln!("== em pe' na prancha de baixo, com a de cima a atravessa'-lo ==");
    eprintln!(
        "  flutuacao {FLOAT_HEIGHT:.2} + meia-altura {BODY_HALF:.2} => a cabeca sobe {:.2} m",
        FLOAT_HEIGHT + BODY_HALF
    );
    eprintln!();
    eprintln!("     vao | descansa em | esperado | cabeca acima da de cima | estavel");
    eprintln!("  -------|-------------|----------|-------------------------|--------");
    for gap in [2.0_f32, 1.6, 1.5, 1.4, 1.2, 1.0, 0.8] {
        let mut r = stack(gap);
        let lower_top = -gap + PLANK_HALF_Y;
        if let Some(mut t) = r.sim.world_mut().get_mut::<Transform>(r.player) {
            t.translation.y = lower_top + FLOAT_HEIGHT;
        }
        let mut t = settle(&mut r, 120, 0);
        let a = y_of(&r.sim);
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for _ in 0..60 {
            t = settle(&mut r, 1, t);
            let y = y_of(&r.sim);
            lo = lo.min(y);
            hi = hi.max(y);
        }
        eprintln!(
            "    {gap:.2} |    {a:+.3}   |  {:+.3}  |         {:+.3}          | {:.4}",
            lower_top + FLOAT_HEIGHT,
            a + BODY_HALF - UPPER_TOP,
            hi - lo
        );
    }
}

/// **(2) O que a descida armada custa** — o pulo mais simples que existe.
///
/// Desce da prancha de cima, pousa na de baixo, e depois **salta e tenta voltar
/// a pousar na de cima**. Com a descida aposentada ele pousa nela; sem, o raio
/// da mola ignora-a e ele volta para o degrau de baixo.
#[test]
#[ignore]
fn measure_what_an_armed_drop_costs() {
    eprintln!("== descer, pousar, e tentar voltar ==");
    eprintln!(
        "  a prancha de cima tem topo {UPPER_TOP:+.2} => pousar nela e' {:+.3}",
        UPPER_TOP + FLOAT_HEIGHT
    );
    eprintln!();
    eprintln!("     vao | apos a descida | apos o pulo de volta | pousou na de cima?");
    eprintln!("  -------|----------------|----------------------|-------------------");
    for gap in [2.0_f32, 1.6, 1.5, 1.4, 1.2, 1.0] {
        let mut r = stack(gap);
        let t = settle(&mut r, 30, 0);
        let t = press(&mut r, down_jump(), 4, 120, t);
        let after_drop = y_of(&r.sim);
        let lower_rest = -gap + PLANK_HALF_Y + FLOAT_HEIGHT;
        let _ = press(&mut r, jump_only(), 6, 150, t);
        let after_jump = y_of(&r.sim);
        eprintln!(
            "    {gap:.2} |     {after_drop:+.3}     |        {after_jump:+.3}         | {}   (a de baixo e' {lower_rest:+.3})",
            if (after_jump - (UPPER_TOP + FLOAT_HEIGHT)).abs() < 0.15 {
                "SIM"
            } else {
                "NAO"
            }
        );
    }
}

/// **(3) A fronteira FINA de onde a prancha o CUSPE**, com a prancha SÓLIDA o
/// tempo todo e nenhuma descida envolvida.
#[test]
#[ignore]
fn measure_where_the_plank_spits_him_out() {
    let plat_mins = UPPER_TOP - 2.0 * PLANK_HALF_Y;
    eprintln!("== a prancha SOLIDA por cima da cabeca, sem descida nenhuma ==");
    eprintln!("  base da prancha de cima: {plat_mins:+.2}");
    eprintln!();
    eprintln!("     vao | descansa em | esperado | cuspido | centro <= base");
    eprintln!("  -------|-------------|----------|---------|---------------");
    let mut gap = 1.45_f32;
    while gap >= 0.899 {
        let mut r = stack(gap);
        let want = -gap + PLANK_HALF_Y + FLOAT_HEIGHT;
        if let Some(mut t) = r.sim.world_mut().get_mut::<Transform>(r.player) {
            t.translation.y = want;
        }
        settle(&mut r, 180, 0);
        let y = y_of(&r.sim);
        eprintln!(
            "    {gap:.2} |    {y:+.3}   |  {want:+.3}  |   {}   |      {}",
            if (y - want).abs() > 0.1 { "SIM" } else { "nao" },
            if want <= plat_mins { "SIM" } else { "nao" }
        );
        gap -= 0.05;
    }
}

/// **(4) A virada segue a BASE da prancha, ou o CENTRO dela?** — a medição que
/// separa uma lei de uma coincidência da espessura.
#[test]
#[ignore]
fn measure_whether_the_boundary_follows_the_planks_underside() {
    eprintln!("== a virada do cuspe, por espessura de prancha ==");
    eprintln!("  a prancha de cima tem centro 0,00 sempre; a base e' -meia-espessura");
    eprintln!();
    eprintln!("  meia-espessura | base   | centro do corpo na virada | base? | centro?");
    eprintln!("  ---------------|--------|---------------------------|-------|--------");
    for ph in [0.05_f32, 0.10, 0.20, 0.30] {
        let mut flip = f32::NAN;
        let mut c = -0.80_f32;
        while c <= 0.20 {
            // ⚠️ O vão sai da meia-espessura da prancha de BAIXO (que é quem
            // sustenta), nunca da de cima — deixar a espessura da de cima vazar
            // para cá põe o personagem acima do próprio repouso, e o assentar
            // dele lê-se como um cuspe.
            let mut r = stack_of(PLANK_HALF_Y + FLOAT_HEIGHT - c, ph);
            if let Some(mut t) = r.sim.world_mut().get_mut::<Transform>(r.player) {
                t.translation.y = c;
            }
            settle(&mut r, 180, 0);
            if (y_of(&r.sim) - c).abs() > 0.05 {
                flip = c;
                break;
            }
            c += 0.01;
        }
        eprintln!(
            "       {ph:.2}      | {:+.2}  |          {flip:+.3}            |  {}  |  {}",
            -ph,
            if (flip + ph).abs() < 0.015 {
                "SIM"
            } else {
                "nao"
            },
            if flip.abs() < 0.015 { "SIM" } else { "nao" }
        );
    }
}

/// **(5) A ESCADA DE TRÊS DEGRAUS, por espessura e por vão** — a sonda que
/// reprovou a cura.
///
/// ⚠️ **É ela que mostra por que o limiar não pode ser mexido sozinho:** com a
/// lei de hoje a escada precisa de `vão >= float + meia-altura + 2·espessura`
/// (1,70 m nas pranchas de 0,15 da cena 91). Toda lei mais frouxa que foi
/// tentada devolveu esta tabela cheia **e** cuspiu o personagem noutro regime —
/// ver o aviso do `retire_drops`.
#[test]
#[ignore]
fn measure_the_ladder_by_thickness_and_rise() {
    eprintln!("== escada de tres degraus: desce UM degrau, e consegue voltar? ==");
    eprintln!("  ⚠️ descer tres seguidos NAO responde a pergunta — no chao toda descida");
    eprintln!("     ja' se aposentou (a caixa ficou inteiramente abaixo de tudo).");
    eprintln!();
    eprintln!("   ph  vao | topo    | desceu  | voltou  | a prancha voltou a ser chao?");
    eprintln!("  --------|---------|---------|---------|-----------------------------");
    for (ph, rise) in (0..14)
        .map(|i| (0.15_f32, 1.60 + 0.05 * i as f32))
        .chain((0..14).map(|i| (0.10_f32, 1.50 + 0.05 * i as f32)))
    {
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
        for i in 0..3 {
            plank(&mut sim, 2.0 + rise * i as f32, ph, &format!("Plank{i}"));
        }
        let player = player_on(&mut sim, 2.0 + rise * 2.0 + ph + FLOAT_HEIGHT);
        let mut r = Rig {
            sim,
            bridge: PhysicsBridge::new(),
            player,
        };
        let t = settle(&mut r, 30, 0);
        let rest = y_of(&r.sim);
        let t = press(&mut r, down_jump(), 4, 130, t);
        let down = y_of(&r.sim);
        press(&mut r, jump_only(), 8, 170, t);
        let back = y_of(&r.sim);
        let one_step = rest - (down - rest).abs().max(0.0);
        let _ = one_step;
        let descended = (rest - down - rise).abs() < 0.1;
        eprintln!(
            "   {ph:.2} {rise:.2} | {rest:+.3} | {down:+.3} | {back:+.3} | {}",
            if !descended {
                "nao DESCEU -- foi arremessado de volta"
            } else if (back - rest).abs() < 0.1 {
                "ok"
            } else {
                "desceu e a prancha ficou FANTASMA"
            }
        );
    }
}
