//! **A QUE ALTURA O PÉ PARA** — o item 5 da §7 do plano 07, e o último da
//! tabela de números.
//!
//! A §7 pergunta *quanto o `offset` (a folga do controlador) custa em
//! aparência*, e diz que é *"a D1 outra vez, num número menor"*. A D1 é a que
//! nomeia os **34 cm** entre os dois modos (o dinâmico PAIRA por desenho, o
//! cinemático POUSA); este é o resíduo do lado que pousa — o personagem não
//! encosta, ele para a uma folga que o rapier preserva de propósito.
//!
//! ⚠️ **O número já vive numa cena de smoke sem ninguém o ter atribuído:** a
//! `=103` carrega `SNAP_REST = 0,5566` para uma cápsula de meia-extensão `0,5`,
//! ou seja **5,7 cm** de pé no ar — e o `offset` default do rapier é `Relative
//! (0,01)`, que numa cápsula de 1 m de altura vale **1 cm**. Os dois números não
//! fecham, e esta sonda existe para dizer qual é o resto.
//!
//! # O que ela mediu (2026-08-09), e a pergunta estava mal colocada
//!
//! O **dinâmico** é aritmética exacta — a folga é `float_height − meia-extensão`
//! ao quarto decimal (0,0005 · 0,2005 · 0,4005 · 0,7005 para float 0,5 · 0,7 ·
//! 0,9 · 1,2). Isso É a D1, e ela se comporta como o desenho promete.
//!
//! O **cinemático** não tem folga constante. A MESMA cápsula, o MESMO
//! `float_height`, variando só de que altura ela cai:
//!
//! ```text
//!   queda (m)   0.05    0.10    0.25    0.50    1.00    2.00    4.00
//!   folga (cm)  4.53    2.86    1.51    1.30    1.01    4.42    2.09
//! ```
//!
//! ⚠️ **Amplitude ZERO em todas** — é repouso estável, não tremor. Então o
//! número não é o `offset` do controlador (1% da altura ⇒ 1 cm nesta cápsula):
//! é **onde o último sweep o deixou**, com 4,5× de espalhamento e sem
//! monotonia. Dois personagens iguais, largados de alturas diferentes,
//! **param a alturas diferentes** — de 1,0 a 4,5 cm numa cápsula de 1 m.
//!
//! ⚠️ **E isto arranha a coluna que o plano vende para este modo** (*"controle
//! exato da posição — a translação é escrita"*): a translação é escrita, mas
//! onde ela PARA é herdado da aproximação. O `snap_to_ground` do rapier não o
//! corrige — ele existe para não largar o chão a descer uma escada, não para
//! encostar a sola.
//!
//! ⛔ **A cura NÃO foi construída, de propósito:** encostar a sola depois do
//! sweep (o `apply_floor_snap()` do Godot) muda a altura de repouso de TODO
//! personagem cinemático em 1-4,5 cm e move o `physics_ecs_c9` — é mudança de
//! aparência no modo que o módulo recomenda, e a §6 do plano diz que quem
//! decide isso é uma cena. O que fica é o número e um gate de LIMITE.
//!
//! # ⚠️ E a caçada achou um SEGUNDO defeito, maior — que fica ABERTO
//!
//! A mutação `snap_to_ground: None` sobrevivia a toda a suíte, e o motivo era
//! que nenhuma fixture do módulo continha uma **descida caminhada**. Escrita
//! uma, o modo cinemático **desce a rampa aos pulos**, e o modo dinâmico é
//! `0,0000` EXACTO em toda a varredura:
//!
//! ```text
//!   salto da superficie (m)      1 m/s    3 m/s    6 m/s
//!     10 graus  CINEMATICO       0.0170   0.0329   0.0806
//!     25 graus  CINEMATICO       0.0412   0.1459   0.5652
//!     40 graus  CINEMATICO       0.0654   0.4434   1.3258
//!     (o dinamico da' 0.0000 nas NOVE celulas)
//! ```
//!
//! A 40° e 6 m/s ele se afasta **mais que a própria altura**, e não é um
//! transiente: o traço mostra a folga a bater entre 0,51 e 1,07 m a descida
//! inteira.
//!
//! **O mecanismo está atribuído por ablação:** a [`supported_velocity`] absorve
//! ao longo do `up`, e numa rampa a descer isso cancela a componente vertical
//! da velocidade — que é exactamente a parte da caminhada que o faz seguir a
//! superfície. Trocado o eixo pela NORMAL do chão, o salto vai a **0,0000** a
//! 25° e 40° em toda velocidade. É a mesma premissa que a W11 corrigiu no
//! `damping_axis` (*"verdadeira só no plano"*) e que ficou por corrigir aqui.
//!
//! ⛔ **MEDIDO E REJEITADO — a cura do eixo NÃO é a cura, não refaça.** Com a
//! normal, o personagem **parado** numa rampa deriva **0,7297 m** (contra
//! ~0,001) e **nunca assenta** — os dois gates que sangram são o
//! `..._does_not_creep_up_a_ramp` e o `..._landing_does_not_slide_along_the_ramp
//! _normal`. E o `up` está lá de propósito: é o `floor_stop_on_slope` do Godot,
//! que o doc do `kinematic.rs` já nomeia. **As duas metades pedem coisas
//! opostas do mesmo eixo**, e `supported_velocity` não consegue distinguir a
//! deriva da GRAVIDADE da descida COMANDADA pela caminhada, porque quando ela
//! corre as duas já estão somadas num `v` só.
//!
//! ⇒ A cura de verdade é separar as duas antes da soma (absorver só o que o
//! motor **não** comandou), e isso é desenho, não uma linha. Fica ABERTO, com
//! o número ao lado e o gate que o pina abaixo.
//!
//! Rodar: `cargo test -p ph2d-physics-ecs --release --test measure_foot_gap
//! -- --ignored --nocapture`

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, PlayerInput,
    PlayerMode, RigidBody,
};

/// A cápsula canônica deste módulo.
const HALF_HEIGHT: f32 = 0.3;
const RADIUS: f32 = 0.2;
/// A meia-extensão vertical: onde o PÉ fica, relativo ao centro.
const FOOT: f32 = HALF_HEIGHT + RADIUS;
/// O topo do chão.
const FLOOR_TOP: f32 = 0.25;

/// A altura de flutuação que o GESTO do artista daria a esta cápsula — a mesma
/// porta do `apply_player_edit(Add)`, com a mesma margem.
///
/// ⚠️ **A 1ª versão desta sonda usava `0,9` fixo para cápsulas de 0,5 a 4 m** e
/// media a de 4 m **enterrada 60 cm** — o piso geométrico dela é `2,0`. É
/// exactamente a armadilha que o `measure_float_floor` acabou de documentar, e
/// eu caí nela no arquivo seguinte.
fn fitted(half_h: f32, r: f32) -> f32 {
    ph2d_platformer::RideConfig::min_float_height(half_h, r, 1.0) * 1.2
}

/// `(folga do pé no fim, amplitude da folga na janela assentada)`.
///
/// ⚠️ **O endpoint sozinho é ruído** se o corpo oscilar — a lição do
/// `measure_idle`, onde uma linha de tabela lida sem a amplitude ao lado
/// parecia deriva de produto.
fn settle(kinematic: bool, float_height: f32, half_h: f32, r: f32) -> (f32, f32) {
    settle_from(kinematic, float_height, half_h, r, 0.5)
}

/// A mesma coisa, com a ALTURA DE QUEDA escolhida.
fn settle_from(kinematic: bool, float_height: f32, half_h: f32, r: f32, drop: f32) -> (f32, f32) {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 40.0,
                half_y: FLOOR_TOP,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
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
                half_height: half_h,
                radius: r,
            },
            ..Collider::default()
        },
        LockRotation,
        PlatformPlayer {
            float_height,
            ..PlatformPlayer::default()
        },
        Transform::from_translation(Vec2::new(0.0, FLOOR_TOP + float_height + drop)),
    ));
    if kinematic {
        e.insert(PlayerMode::Kinematic);
    }
    let who = e.id();
    let mut bridge = PhysicsBridge::new();
    for t in 1..=300u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let gap = |sim: &SimWorld| {
        sim.world()
            .get::<Transform>(who)
            .map_or(0.0, |t| t.translation.y)
            - (half_h + r)
            - FLOOR_TOP
    };
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for t in 301..=600u64 {
        bridge.dispatch(&mut sim, true, t);
        let g = gap(&sim);
        lo = lo.min(g);
        hi = hi.max(g);
    }
    let _: Entity = who;
    (gap(&sim), hi - lo)
}

#[test]
#[ignore = "sonda"]
fn measure_where_the_foot_stops() {
    println!("\n=== A QUE ALTURA O PE PARA (chao plano, assentado) ===");
    println!(
        "capsula half {HALF_HEIGHT} / radius {RADIUS} (meia-extensao {FOOT}); \
         topo do chao em {FLOOR_TOP}\n"
    );
    println!(
        "{:>14}  {:>12}  {:>14}  {:>12}",
        "float pedido", "modo", "folga do pe (m)", "amplitude"
    );
    for fh in [0.5_f32, 0.7, 0.9, 1.2] {
        let (d, da) = settle(false, fh, HALF_HEIGHT, RADIUS);
        let (k, ka) = settle(true, fh, HALF_HEIGHT, RADIUS);
        println!("{fh:>14.2}  {:>12}  {d:>14.4}  {da:>12.5}", "dinamico");
        println!("{:>14}  {:>12}  {k:>14.4}  {ka:>12.5}", "", "CINEMATICO");
    }

    println!("\n--- E a folga do cinematico ESCALA com a capsula? ---");
    println!("(o `offset` default do rapier e' Relative(0,01) => 1% da ALTURA da forma)\n");
    println!(
        "{:>8}  {:>8}  {:>8}  {:>8}  {:>12}  {:>12}",
        "half", "radius", "altura", "float", "folga do pe", "folga/altura"
    );
    for (h, r) in [
        (0.15_f32, 0.10_f32),
        (0.3, 0.2),
        (0.6, 0.4),
        (1.2, 0.8),
        (0.3, 0.05),
    ] {
        // ⚠️ Cada cápsula recebe o float que o GESTO lhe daria — um número fixo
        // punha as maiores abaixo do próprio piso geométrico.
        let fh = fitted(h, r);
        let (gap, _) = settle(true, fh, h, r);
        let height = 2.0 * (h + r);
        println!(
            "{h:>8.2}  {r:>8.2}  {height:>8.2}  {fh:>8.3}  {gap:>12.4}  {:>12.4}",
            gap / height
        );
    }
    println!(
        "\nLEITURA: se a coluna `folga/altura` for CONSTANTE, a folga e' o\n\
         `offset` relativo do rapier e mais nada; se ela variar, ha um segundo\n\
         termo e a §7.5 tem de o nomear.\n"
    );
}

/// **A FOLGA DEPENDE DE COMO ELE CHEGOU?** — o teste decisivo da §7.5.
#[test]
#[ignore = "sonda"]
fn measure_whether_the_rest_remembers_the_fall() {
    println!("\n=== A FOLGA DO PE CONTRA A ALTURA DA QUEDA (mesma capsula) ===");
    println!("capsula half {HALF_HEIGHT} / radius {RADIUS}, float 0,90\n");
    println!(
        "{:>12}  {:>14}  {:>12}",
        "queda (m)", "folga do pe", "amplitude"
    );
    for drop in [0.05_f32, 0.1, 0.25, 0.5, 1.0, 2.0, 4.0] {
        let (g, a) = settle_from(true, 0.9, HALF_HEIGHT, RADIUS, drop);
        println!("{drop:>12.2}  {g:>14.4}  {a:>12.5}");
    }
    println!(
        "\nLEITURA: se a folga VARIAR com a queda, o repouso do modo cinematico\n\
         nao e' propriedade do corpo -- e' onde o ultimo sweep o deixou, e dois\n\
         personagens iguais param a alturas diferentes.\n"
    );
}

/// **O PÉ NUNCA ATRAVESSA E NUNCA PAIRA MAIS QUE UM DEDO** (§7.5).
///
/// Duas metades, e a primeira é a coluna que o plano vende para este modo:
/// *penetração no impacto — zero por construção*. Uma folga NEGATIVA é o corpo
/// dentro do chão, e é o único valor que o modo promete não produzir.
///
/// ⚠️ **O teto é MEDIDO** (o pior de sete aproximações é 4,53 cm) e existe para
/// a folga não crescer em silêncio; ele **não** afirma que ela é constante — ela
/// não é, e um gate que o afirmasse castigaria justamente a cura.
///
/// ⚠️ **E o CONTROLE é o modo dinâmico**, cuja folga é aritmética exacta
/// (`float_height − meia-extensão`): sem ele o leitor não distingue *"o
/// cinemático pousa com uma folguinha"* de *"a sonda mede errado"*.
#[test]
fn the_foot_never_sinks_and_never_hovers_more_than_a_finger() {
    // O pior caso medido é 4,53 cm; 6 cm deixa 1,3x de distância.
    const CEILING: f32 = 0.06;
    for drop in [0.05_f32, 0.25, 1.0, 4.0] {
        let (gap, amp) = settle_from(true, 0.9, HALF_HEIGHT, RADIUS, drop);
        assert!(
            gap >= 0.0,
            "largado de {drop} m o pe AFUNDOU {gap:.4} m — o modo promete \
             penetracao zero por construcao"
        );
        assert!(
            gap < CEILING,
            "largado de {drop} m o pe parou a {gap:.4} m do chao, acima do teto \
             medido de {CEILING}"
        );
        assert!(
            amp < 1.0e-4,
            "e o repouso tem de ser ESTAVEL: amplitude {amp:.5} m largado de {drop} m"
        );
    }
    // O CONTROLE: no dinâmico a folga é a aritmética da D1, e nada mais.
    for fh in [0.7_f32, 1.2] {
        let (gap, _) = settle(false, fh, HALF_HEIGHT, RADIUS);
        let want = fh - FOOT;
        assert!(
            (gap - want).abs() < 0.002,
            "o dinamico PAIRA por desenho: com float {fh} a folga tem de ser \
             {want:.4} e mediu {gap:.4}"
        );
    }
}

/// A descida caminhada: devolve `(folga no repouso, PIOR folga, andou)`.
fn walk_down(kinematic: bool, slope_deg: f32, speed: f32) -> (f32, f32, f32) {
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
        // Desce para a DIREITA, que é o sentido em que ele vai andar.
        Transform {
            rotation: -theta,
            ..Transform::from_translation(Vec2::new(0.0, 0.0))
        },
    ));
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
            float_height: 0.9,
            speed,
            ..PlatformPlayer::default()
        },
        Transform::from_translation(Vec2::new(-15.0, 15.0 * theta.tan() + 1.0)),
    ));
    if kinematic {
        e.insert(PlayerMode::Kinematic);
    }
    let who = e.id();
    let mut bridge = PhysicsBridge::new();
    for t in 1..=180u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    // A distância PERPENDICULAR do centro à face de cima da rampa.
    let perp = |sim: &SimWorld| {
        let t = sim.world().get::<Transform>(who).expect("player");
        (t.translation.x * theta.sin() + t.translation.y * theta.cos()) - FLOOR_TOP
    };
    let settled = perp(&sim);
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
    let mut worst = settled;
    for t in 181..=420u64 {
        bridge.dispatch(&mut sim, true, t);
        worst = worst.max(perp(&sim));
    }
    let x1 = sim
        .world()
        .get::<Transform>(who)
        .expect("player")
        .translation
        .x;
    (settled, worst, x1 - x0)
}

/// **O PERSONAGEM DESCE A RAMPA AOS PULOS?** — a pergunta que uma mutação
/// sobrevivente abriu, e o modo dinâmico é o controle.
#[test]
#[ignore = "sonda"]
fn measure_whether_he_hops_down_a_ramp() {
    println!("\n=== A DESCIDA CAMINHADA: quanto ele SALTA da superficie ===");
    println!("(a folga perpendicular no repouso e' ~0,51 m -- a extensao da capsula)\n");
    println!(
        "{:>7}  {:>7}  {:>12}  {:>10}  {:>10}  {:>10}",
        "rampa", "vel", "modo", "repouso", "pior", "salto"
    );
    for slope in [10.0_f32, 25.0, 40.0] {
        for speed in [1.0_f32, 3.0, 6.0] {
            for (tag, kin) in [("dinamico", false), ("CINEMATICO", true)] {
                let (rest, worst, _) = walk_down(kin, slope, speed);
                println!(
                    "{slope:>6.0}°  {speed:>7.1}  {tag:>12}  {rest:>10.4}  {worst:>10.4}  \
                     {:>10.4}",
                    worst - rest
                );
            }
        }
    }
    println!(
        "\nLEITURA: a coluna `salto` e' quanto ele se afasta da superficie a\n\
         descer. Se o dinamico segue e o cinematico salta, e' o controlador; se\n\
         os dois saltam, e' a lei da caminhada e nao o modo.\n"
    );
}

/// **O SALTO DA DESCIDA AINDA ESTÁ LÁ, E ESTE É O NÚMERO DELE.**
///
/// ⚠️ **Um gate VERDE sobre um defeito, de propósito** — o precedente é o
/// `the_documented_hardening_is_still_there_and_this_is_its_number` do Painter.
/// Ele não afirma que o produto está certo; ele **pina o defeito** para que a
/// cura tenha de o atualizar deliberadamente, e para que ele não CRESÇA em
/// silêncio enquanto ninguém olha.
///
/// O controle é o modo dinâmico, que é `0,0000` — sem ele o número não
/// distingue *"o cinemático salta"* de *"a sonda mede mal"*.
#[test]
fn the_descent_hop_is_still_there_and_this_is_its_number() {
    // 25°, 6 m/s: o caso do meio da varredura.
    let (_, _, travelled) = walk_down(true, 25.0, 6.0);
    let (rest_k, worst_k, _) = walk_down(true, 25.0, 6.0);
    let (rest_d, worst_d, _) = walk_down(false, 25.0, 6.0);
    assert!(
        travelled > 10.0,
        "a fixture tem de CONTER a descida: andou {travelled:.2} m"
    );
    assert!(
        (worst_d - rest_d).abs() < 1.0e-3,
        "o CONTROLE tem de seguir a superficie: o dinamico saltou {:.4} m",
        worst_d - rest_d
    );
    let hop = worst_k - rest_k;
    assert!(
        (0.4..0.8).contains(&hop),
        "o salto do cinematico a 25°/6 m/s era 0,565 m e mediu {hop:.4} — se \
         DIMINUIU, alguem curou o defeito e este gate tem de ser reescrito; se \
         cresceu, uma wave o piorou"
    );
}
