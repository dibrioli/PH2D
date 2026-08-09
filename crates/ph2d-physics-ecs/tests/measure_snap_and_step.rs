//! **O DEGRAU QUE ELE SOBE E O DEGRAU QUE ELE DESCE** — o item 4 da §7 do
//! plano 07 (*"`snap_to_ground` mínimo útil, contra a altura do degrau que o
//! `autostep` sobe"*).
//!
//! # ⚠️ O item pergunta a relação entre dois números, e o produto usa UM
//!
//! `bridge/player.rs` enche o [`CharacterParams`] assim:
//!
//! ```text
//!   snap_distance: cfg.ride.cling_distance,
//!   step_height:   cfg.ride.cling_distance,
//! ```
//!
//! com o comentário — correto — de que *"os dois números saem da configuração
//! que o artista JÁ autora"*. O que ninguém mediu é que eles respondem a
//! **perguntas diferentes**:
//!
//! * `snap_distance` — *quão longe abaixo dos pés o controlador ainda cola?*
//!   (o que impede o personagem de se soltar a DESCER)
//! * `step_height` — *que altura de degrau ele SOBE sozinho?*
//!
//! e a `cling_distance` foi autorada para uma TERCEIRA (*a faixa acima do
//! repouso em que a mola ainda age* — `RideConfig`, o modo dinâmico). Esta
//! sonda mede o que cada metade compra, para o número ter tabela em vez de
//! herança.
//!
//! ⚠️ **O modo DINÂMICO é o controle e não usa nenhum dos dois** — ele não
//! passa pelo controlador do rapier: quem o levanta é a MOLA. Sem essa coluna o
//! leitor não distingue *"o autostep sobe"* de *"qualquer perna sobe"*.
//!
//! # O que ela mediu (2026-08-09): uma metade vale 1:1, a outra é INERTE
//!
//! **SUBIR** — o degrau segue o número, e o excedente é a cápsula a escorregar
//! por cima de um lábio (o que ela sobe com o autostep desligado):
//!
//! ```text
//!   cling        0.05   0.10   0.15   0.25   0.40   0.60
//!   CINEMATICO   0.10   0.16   0.20   0.28   0.40   0.60
//!   dinamico     0.44   0.44   0.44   0.44   0.44   0.44   <- a MOLA, nao o numero
//! ```
//!
//! **DESCER — a medição de manhã dizia INERTE, e a §8.1 a derrubou no mesmo
//! dia.** Fica escrita porque o mecanismo é o achado:
//!
//! Antes da cura, variar a `cling_distance` de 0,05 a 2,00 deixava **todas as
//! 24 células idênticas ao quarto decimal**, e a mutação `snap_to_ground: None`
//! produzia a **MESMA tabela, número por número** — o snap **nunca disparava**.
//!
//! ⚠️ **E o CONTROLE POSITIVO é o que tornou isso uma afirmação** — sem ele
//! *"nada mudou"* é indistinguível de *"o arquivo não está no laço"*. Armado
//! `autostep: None` no mesmo `CharacterParams`, a coluna de subida colapsa de
//! 0,10-0,60 para 0,06-0,10 ⇒ a sonda VÊ aquele struct.
//!
//! # ⚠️ O porquê está na fonte do rapier, e é o MESMO mecanismo da §8.1
//!
//! `rapier2d-0.28/src/control/character_controller.rs::snap_to_ground` abre com
//!
//! ```text
//!   if result.translation.dot(&self.up) < -1.0e-5 {
//! ```
//!
//! ou seja: o snap só age quando o deslocamento EFETIVO do tique tem componente
//! para BAIXO. E a lei cinemática **zerava a vertical enquanto no chão** (a
//! `supported_velocity` sem piso) — que é, palavra por palavra, o mecanismo do
//! salto da descida.
//!
//! ⇒ **as duas coisas eram UM defeito**, e curar a §8.1 devolveu o snap ao ar
//! de graça, exatamente como esta sonda previu.
//!
//! # A §7.4 RE-MEDIDA, com a lei curada (o mínimo útil EXISTE)
//!
//! ```text
//!   cling   0.02   0.04   0.06   0.08   0.10   0.12   0.15   0.25 .. 2.00
//!   salto   0.639  0.417  0.417  0.417  0.010  0.010  0.010  0.010 .. 0.010
//! ```
//!
//! ⇒ **o `snap_to_ground` mínimo útil está entre 0,08 e 0,10 m** para esta
//! cápsula, e o default de `0,25` tem **2,5× de margem**. O acoplamento
//! `snap = step = cling` segue barato: a metade *subir* pede 1:1 o degrau que o
//! artista quer, e a *descer* pede um piso pequeno que o mesmo número cobre com
//! folga. Gate: `the_ground_snap_is_load_bearing_again`.
//!
//! ⚠️ **A tabela de BEIRADA não mudou, e é correto:** sair de um penhasco é
//! CAIR — o `floor_snap_length` do Godot existe para descer escadas, não para
//! colar o personagem a uma quina.
//!
//! Rodar: `cargo test -p ph2d-physics-ecs --release --test measure_snap_and_step
//! -- --ignored --nocapture`

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, PlayerInput,
    PlayerMode, RigidBody,
};

/// A cápsula canônica deste módulo (a mesma do `measure_foot_gap`).
const HALF_HEIGHT: f32 = 0.3;
const RADIUS: f32 = 0.2;
/// O topo do chão de baixo.
const FLOOR_TOP: f32 = 0.25;
/// A altura de flutuação — bem acima do piso geométrico desta cápsula.
const FLOAT: f32 = 0.9;
/// Onde o degrau começa. O personagem nasce à esquerda e anda para a direita.
const STEP_X: f32 = 0.0;
const START_X: f32 = -3.0;

fn spawn_player(sim: &mut SimWorld, kinematic: bool, cling: f32, x: f32, y: f32) -> Entity {
    let mut e = sim.world_mut().spawn((
        Name::new("Player"),
        RigidBody {
            kind: if kinematic {
                BodyKind::Kinematic
            } else {
                BodyKind::Dynamic
            },
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: HALF_HEIGHT,
                radius: RADIUS,
            },
            ..Collider::default()
        },
        LockRotation,
        PlatformPlayer {
            float_height: FLOAT,
            cling_distance: cling,
            speed: 3.0,
            ..PlatformPlayer::default()
        },
        Transform::from_translation(Vec2::new(x, y)),
    ));
    if kinematic {
        e.insert(PlayerMode::Kinematic);
    }
    e.id()
}

fn slab(sim: &mut SimWorld, name: &str, cx: f32, top: f32, half_x: f32) {
    // Uma laje GROSSA: o topo é o que importa, e um chão fino deixaria o
    // personagem cair através dele num tique rápido.
    let half_y = 2.0;
    sim.world_mut().spawn((
        Name::new(name),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid { half_x, half_y },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(cx, top - half_y)),
    ));
}

/// **SUBIR:** anda contra um degrau de altura `step_h` e devolve
/// `(passou o degrau?, x final, y final)`.
///
/// ⚠️ O oráculo é **passar**, não a altura: subir meio degrau e ficar preso lá
/// não é subir, e uma asserção só em `y` aceitaria isso.
fn climb(kinematic: bool, cling: f32, step_h: f32) -> (bool, f32, f32) {
    let mut sim = SimWorld::new();
    slab(&mut sim, "Lower", START_X - 5.0, FLOOR_TOP, 10.0);
    slab(&mut sim, "Step", STEP_X + 10.0, FLOOR_TOP + step_h, 10.0);
    let who = spawn_player(&mut sim, kinematic, cling, START_X, FLOOR_TOP + FLOAT + 0.2);
    let mut bridge = PhysicsBridge::new();
    for t in 1..=120u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    bridge.set_player_input(
        who,
        PlayerInput {
            drive: 1.0,
            ..PlayerInput::default()
        },
    );
    for t in 121..=360u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let t = sim.world().get::<Transform>(who).expect("player");
    let (x, y) = (t.translation.x, t.translation.y);
    // Passou = está bem para dentro do degrau (a face dele é `STEP_X`).
    (x > STEP_X + 1.0, x, y)
}

/// O degrau mais alto que ele sobe, varrido em centímetros.
fn tallest_step(kinematic: bool, cling: f32) -> f32 {
    let mut best = 0.0;
    let mut h = 0.02_f32;
    while h <= 0.605 {
        if climb(kinematic, cling, h).0 {
            best = h;
        }
        h += 0.02;
    }
    best
}

/// **DESCER:** anda para fora de uma beirada de altura `drop_h` e devolve
/// `(quantos METROS depois da quina o pé volta ao repouso, andou)`.
///
/// ⚠️ **A 1ª versão desta função media a PIOR folga sobre o chão de baixo, e
/// isso não separa nada:** no instante em que ele cruza a quina o pé está,
/// **por definição**, `drop_h` acima do chão de baixo — cole ele no tique
/// seguinte ou voe por três metros. A tabela saía `0,11 / 0,31 / 0,61` para
/// beiradas de `0,10 / 0,30 / 0,60` em TODA coluna, ou seja media a beirada e
/// chamava isso de resposta.
///
/// A grandeza que distingue as duas coisas é a **DISTÂNCIA de aterragem**: o
/// personagem colado desce dentro de um tique, o que voa descreve uma parábola
/// e leva metros. E ela é justa entre os modos porque o alvo é o repouso **de
/// cada um** (o dinâmico PAIRA `float − meia-extensão` acima do chão, a D1).
fn walk_off(kinematic: bool, cling: f32, drop_h: f32) -> (f32, f32) {
    let top = FLOOR_TOP + drop_h;
    let mut sim = SimWorld::new();
    slab(&mut sim, "Upper", START_X - 5.0, top, 10.0);
    slab(&mut sim, "Lower", STEP_X + 12.0, FLOOR_TOP, 10.0);
    let who = spawn_player(&mut sim, kinematic, cling, START_X, top + FLOAT + 0.2);
    let mut bridge = PhysicsBridge::new();
    for t in 1..=120u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let foot = |sim: &SimWorld| {
        sim.world()
            .get::<Transform>(who)
            .expect("player")
            .translation
            .y
            - (HALF_HEIGHT + RADIUS)
    };
    // O repouso DESTE modo, medido no chão de cima: é o alvo da aterragem.
    let rest = foot(&sim) - top;
    let x0 = sim
        .world()
        .get::<Transform>(who)
        .expect("player")
        .translation
        .x;
    bridge.set_player_input(
        who,
        PlayerInput {
            drive: 1.0,
            ..PlayerInput::default()
        },
    );
    // A quina do chão de cima: o `slab` de cima é centrado em `START_X - 5`
    // com meia-largura 10, então a face direita dele é `START_X + 5`.
    let edge = START_X + 5.0;
    let mut crossed: Option<f32> = None;
    let mut landed: Option<f32> = None;
    for t in 121..=420u64 {
        bridge.dispatch(&mut sim, true, t);
        let x = sim
            .world()
            .get::<Transform>(who)
            .expect("player")
            .translation
            .x;
        if x > edge {
            if crossed.is_none() {
                crossed = Some(x);
            }
            // ⚠️ **A tolerância é 6 cm, e o número não é folga:** o repouso do
            // modo cinemático **não é reproduzível** — ele varia de 1,0 a 4,5 cm
            // conforme a aproximação (o achado do item 5 da §7). Uma tolerância
            // de 2 cm sobre o repouso medido no chão de CIMA nunca fecha quando
            // ele pousa 3 cm mais alto no de baixo, e a tabela reportava
            // *"nunca aterrou"* para uma aterragem perfeita.
            if landed.is_none() && foot(&sim) - FLOOR_TOP <= rest + 0.06 {
                landed = Some(x);
            }
        }
    }
    let x1 = sim
        .world()
        .get::<Transform>(who)
        .expect("player")
        .translation
        .x;
    let run = match (crossed, landed) {
        (Some(a), Some(b)) => b - a,
        // Nunca aterrou dentro da janela: reporta o que ele andou depois da
        // quina, para a tabela dizer "pelo menos isto" em vez de mentir zero.
        (Some(a), None) => x1 - a,
        _ => f32::NAN,
    };
    (run, x1 - x0)
}

#[test]
#[ignore = "sonda"]
fn measure_what_each_half_of_the_number_buys() {
    println!("\n=== O DEGRAU QUE ELE SOBE (autostep) ===");
    println!(
        "capsula half {HALF_HEIGHT} / radius {RADIUS}, float {FLOAT}; \
         `step_height` = `cling_distance`\n"
    );
    println!(
        "{:>10}  {:>18}  {:>18}",
        "cling", "CINEMATICO sobe", "dinamico sobe"
    );
    for cling in [0.05_f32, 0.10, 0.15, 0.25, 0.40, 0.60] {
        let k = tallest_step(true, cling);
        let d = tallest_step(false, cling);
        println!("{cling:>10.2}  {k:>18.2}  {d:>18.2}");
    }
    println!(
        "\nLEITURA: se a coluna CINEMATICO seguir o `cling`, o degrau E' o\n\
         numero; se ela saturar, quem manda e' outra coisa. A coluna dinamica\n\
         nao passa pelo controlador -- quem a levanta e' a MOLA."
    );

    println!("\n=== O DEGRAU QUE ELE DESCE (snap_to_ground) ===");
    println!("(quantos METROS depois da quina o pe volta ao repouso -- a 3 m/s)\n");
    println!(
        "{:>10}  {:>10}  {:>16}  {:>16}",
        "cling", "beirada", "CINEMATICO", "dinamico"
    );
    for cling in [0.05_f32, 0.15, 0.25, 0.60] {
        for drop in [0.10_f32, 0.30, 0.60] {
            let (k, _) = walk_off(true, cling, drop);
            let (d, _) = walk_off(false, cling, drop);
            println!("{cling:>10.2}  {drop:>10.2}  {k:>16.4}  {d:>16.4}");
        }
    }
    println!(
        "\nLEITURA: ~0 m = ele COLOU no degrau de baixo dentro de um tique; um\n\
         numero grande = saiu em VOO e caiu em parabola. O `cling` em que a\n\
         coluna vira e' o `snap_to_ground` minimo util, que e' o que a §7.4 pede.\n"
    );
}

/// **O SALTO DA DESCIDA CONTRA O `cling`** — a §8.1 tem um knob?
///
/// O defeito aberto da §8.1 (o personagem desce a rampa aos pulos) é sobre o
/// EIXO da absorção, mas quem devia trazê-lo de volta ao chão a cada tique é o
/// `snap_to_ground`. Se subir o `cling` o curar, existe uma resposta de PRODUTO
/// para um defeito que hoje só tem resposta de desenho.
#[test]
#[ignore = "sonda"]
fn measure_whether_a_bigger_cling_cures_the_descent_hop() {
    println!("\n=== O SALTO DA DESCIDA CONTRA O `cling` (rampa 25 graus, 6 m/s) ===\n");
    println!("{:>10}  {:>14}  {:>14}", "cling", "salto (m)", "andou (m)");
    for cling in [
        0.02_f32, 0.04, 0.06, 0.08, 0.10, 0.12, 0.15, 0.25, 0.50, 1.00, 2.00,
    ] {
        let (hop, ran) = ramp_hop(cling);
        println!("{cling:>10.2}  {hop:>14.4}  {ran:>14.2}");
    }
    println!(
        "\nLEITURA: se o salto CAIR com o `cling`, o `snap` alcanca o chao e o\n\
         defeito da §8.1 tem mitigacao autorada; se nao mexer, o snap nunca foi\n\
         a metade que falta e a §8.1 continua sendo so' o eixo.\n"
    );
}

/// **O `snap_to_ground` VOLTOU A VIVER, e este é o mínimo útil** — a §7.4,
/// re-medida depois de a §8.1 ser curada.
///
/// ⚠️ **Esta é a nota que a cura obrigou a reconferir** (CLAUDE.md §0). Antes
/// dela, variar o `cling` de 0,05 a 2,00 não movia um número e a resposta desta
/// §7.4 era *"o mínimo útil não existe"*. Com o piso da absorção, o
/// deslocamento do tique passa a ter componente para baixo, o rapier deixa de
/// recusar o snap, e o degrau aparece:
///
/// ```text
///   cling   0.02   0.04   0.06   0.08   0.10   0.12   0.15   0.25 .. 2.00
///   salto   0.639  0.417  0.417  0.417  0.010  0.010  0.010  0.010 .. 0.010
/// ```
///
/// ⇒ **o mínimo útil está entre 0,08 e 0,10 m** para esta cápsula, e o default
/// (`RideConfig::STARTING_POINT`, 0,25) tem **2,5× de margem**.
#[test]
fn the_ground_snap_is_load_bearing_again() {
    let (small, _) = ramp_hop(0.02);
    let (default, _) = ramp_hop(0.25);
    assert!(
        default < 0.05,
        "no default o personagem segue a rampa: saltou {default:.4} m"
    );
    assert!(
        small > 10.0 * default.max(1.0e-4),
        "e o knob tem de MORDER: com `cling` 0,02 o salto era 0,639 contra \
         {default:.4} — se os dois ficaram iguais, o `snap_to_ground` voltou a \
         ser inerte e a §7.4 tem de ser reescrita outra vez"
    );
}

/// **O DEGRAU QUE ELE SOBE É O NÚMERO QUE O ARTISTA AUTOROU** — a metade VIVA
/// da §7.4.
///
/// Das duas perguntas que a `cling_distance` responde no
/// [`CharacterParams`], esta é a que funciona: o degrau segue o número, e o
/// excedente é a cápsula a escorregar por cima de um lábio (~6 cm, o que ela
/// sobe **sem** autostep nenhum).
///
/// ⚠️ **A resolução da varredura é 2 cm** e entra no limite inferior — sem isso
/// o gate seria uma aposta na grade e não na propriedade.
///
/// ⚠️ **E ele não é vazio:** o intervalo medido vai de 0,10 a 0,60 m (6×), então
/// um produto que ignorasse o número não passaria por acidente. Mutação provada:
/// `autostep: None` colapsa a coluna para **0,06-0,10 em toda a faixa** e o
/// limite inferior sangra em `cling >= 0,15`.
#[test]
fn the_step_it_climbs_is_the_number_the_artist_authored() {
    /// Um passo da varredura, mais folga.
    const GRID: f32 = 0.021;
    /// O que a cápsula sobe sozinha, medido com o autostep desligado.
    const CAPSULE_SLIDE: f32 = 0.08;
    for cling in [0.15_f32, 0.25, 0.40, 0.60] {
        let climbed = tallest_step(true, cling);
        assert!(
            climbed >= cling - GRID,
            "com `cling_distance` {cling} ele tinha de subir o degrau que \
             autorou, e parou em {climbed:.2} m"
        );
        assert!(
            climbed <= cling + CAPSULE_SLIDE,
            "o degrau autorado e' um LIMITE, nao uma sugestao: com {cling} ele \
             subiu {climbed:.2} m"
        );
    }
}

/// **A SUBIDA caminhada** — o outro lado do piso, e o que decide se o `min(0)`
/// dele é load-bearing ou higiene.
///
/// Devolve `(altura ganha, andou)`.
#[test]
#[ignore = "sonda"]
fn measure_what_the_floors_min_does_to_a_climb() {
    println!("\n=== A SUBIDA (o piso seria POSITIVO sem o `min(0)`) ===\n");
    println!("{:>10}  {:>14}  {:>12}", "rampa", "subiu (m)", "andou (m)");
    for slope in [10.0_f32, 25.0, 40.0] {
        let (rise, ran) = ramp_walk(-slope, 0.25);
        println!("{slope:>10.0}  {rise:>14.4}  {ran:>12.2}");
    }
    println!(
        "\nLEITURA: com o `min(0)` estes numeros sao os do mundo de ANTES do\n\
         piso (o piso e' zero a subir). Sem ele, o piso vira positivo e a\n\
         absorcao passa a EMPURRAR para cima -- compare as duas corridas.\n"
    );
}

/// A descida caminhada de 25°, com o `cling` escolhido: `(salto, andou)`.
fn ramp_hop(cling: f32) -> (f32, f32) {
    let (hop, ran, _) = ramp_run(25.0, cling);
    (hop, ran)
}

/// A caminhada numa rampa: `(altura ganha, andou)`. Inclinação NEGATIVA sobe.
fn ramp_walk(slope_deg: f32, cling: f32) -> (f32, f32) {
    let (_, ran, rise) = ramp_run(slope_deg, cling);
    (rise, ran)
}

/// `(salto perpendicular, andou, altura ganha)`.
fn ramp_run(slope_deg: f32, cling: f32) -> (f32, f32, f32) {
    let theta = slope_deg.to_radians();
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Ramp"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 60.0,
                half_y: FLOOR_TOP,
            },
            ..Collider::default()
        },
        Transform {
            rotation: -theta,
            ..Transform::from_translation(Vec2::new(0.0, 0.0))
        },
    ));
    let mut e = sim.world_mut().spawn((
        Name::new("Player"),
        RigidBody {
            kind: BodyKind::Kinematic,
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: HALF_HEIGHT,
                radius: RADIUS,
            },
            ..Collider::default()
        },
        LockRotation,
        PlatformPlayer {
            float_height: FLOAT,
            cling_distance: cling,
            speed: 6.0,
            ..PlatformPlayer::default()
        },
        Transform::from_translation(Vec2::new(-15.0, 15.0 * theta.tan() + 1.0)),
    ));
    let y_start_ref = 15.0 * theta.tan();
    e.insert(PlayerMode::Kinematic);
    let who = e.id();
    let mut bridge = PhysicsBridge::new();
    for t in 1..=180u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let perp = |sim: &SimWorld| {
        let t = sim.world().get::<Transform>(who).expect("player");
        (t.translation.x * theta.sin() + t.translation.y * theta.cos()) - FLOOR_TOP
    };
    let settled = perp(&sim);
    let p0 = *sim.world().get::<Transform>(who).expect("player");
    let (x0, y0) = (p0.translation.x, p0.translation.y);
    bridge.set_player_input(
        who,
        PlayerInput {
            drive: 1.0,
            ..PlayerInput::default()
        },
    );
    let mut worst = settled;
    for t in 181..=420u64 {
        bridge.dispatch(&mut sim, true, t);
        worst = worst.max(perp(&sim));
    }
    let t1 = *sim.world().get::<Transform>(who).expect("player");
    let _ = y_start_ref;
    (
        worst - settled,
        t1.translation.x - x0,
        t1.translation.y - y0,
    )
}
