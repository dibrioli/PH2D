//! A sonda da cena 105 + os gates que mantêm a mensagem dela honesta.
//!
//! ⚠️ **Uma cena cuja mensagem cita números tem de os medir**, senão a primeira
//! wave que mexer num default a transforma num folheto — a lição que a cena 104
//! já carrega, e cuja primeira versão publicou números de OUTRA fixture.
//!
//! ⚠️ **E os gates aqui julgam a CENA, não a lei.** A lei tem os dela
//! (`ph2d-platformer::swim_tests` e `lib_swim_tests`) e o produto tem os dele
//! (`ph2d-physics-ecs/tests/player_swims.rs`). O que só esta cena pode afirmar é
//! que **os dois corpos que ela monta são comparáveis** (senão o artista olha
//! para uma diferença que não é a que a wave produziu) e que **a poça rasa é de
//! facto rasa** — o número que o passo 1 da mensagem promete.

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::{PhysicsBridge, PlayerInput};

/// A altura MÉDIA do nadador na segunda metade de seis segundos, largado
/// submerso na poça funda desta cena, com a entrada que se pedir — relativa ao
/// ponto de largada.
///
/// ⚠️ **Ele é TELEPORTADO para dentro da poça**, e é honesto: é onde o artista
/// chega andando pelo cais no passo 2, e medir a caminhada junto misturaria o
/// trajeto com o regime que a mensagem descreve.
///
/// ⚠️ **É uma MÉDIA e não um instante.** O corpo na água oscila, e uma amostra
/// única de um sistema que oscila não é um repouso — medido no harness desta
/// wave antes da correção, *segurar para baixo* lia **acima** de *não pedir
/// nada*, por 5%, sobre leis que na saturação produzem o MESMO motor.
fn swims(who: &str, input: PlayerInput) -> f32 {
    const START: f32 = -1.0;
    let mut sim = SimWorld::new();
    build_swim_scene(sim.world_mut());
    let target = named(&sim, who);
    sim.world_mut()
        .get_mut::<Transform>(target)
        .expect("o corpo tem de ter pose")
        .translation = Vec2::new(POOL_X, START);

    let mut bridge = PhysicsBridge::new();
    let (mut sum, mut n) = (0.0f64, 0u32);
    for t in 0..=360u64 {
        for e in players(&sim) {
            bridge.set_player_input(e, input);
        }
        bridge.dispatch(&mut sim, true, t);
        if t > 180 {
            sum += f64::from(y_of(&sim, who));
            n += 1;
        }
    }
    #[allow(clippy::cast_possible_truncation)]
    ((sum / f64::from(n)) as f32 - START)
}

/// A entidade de um nome — os dois corpos desta cena têm nomes distintos, e é
/// por eles que a sonda os alcança.
fn named(sim: &SimWorld, who: &str) -> ph2d_ecs::Entity {
    let mut found = None;
    let mut q = sim
        .world()
        .try_query::<(ph2d_ecs::Entity, &Name)>()
        .unwrap();
    for (e, n) in q.iter(sim.world()) {
        if n.as_str() == who {
            found = Some(e);
        }
    }
    found.expect("o corpo tem de existir")
}

fn players(sim: &SimWorld) -> Vec<ph2d_ecs::Entity> {
    let mut out = Vec::new();
    let Some(mut q) = sim
        .world()
        .try_query::<(ph2d_ecs::Entity, &ph2d_physics_ecs::PlatformPlayer)>()
    else {
        return out;
    };
    for (e, _) in q.iter(sim.world()) {
        out.push(e);
    }
    out
}

fn y_of(sim: &SimWorld, who: &str) -> f32 {
    let mut found = None;
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == who {
            found = Some(t.translation.y);
        }
    }
    found.expect("o corpo tem de existir")
}

fn holding(jump: bool, down: bool) -> PlayerInput {
    PlayerInput {
        jump,
        down,
        ..PlayerInput::default()
    }
}

/// **A SONDA** — imprime os três números que a mensagem cita.
#[test]
#[ignore = "sonda: roda a pedido"]
fn probe_smoke_105() {
    let dive = swims("Swimmer", holding(false, true));
    let idle = swims("Swimmer", PlayerInput::default());
    let rise = swims("Swimmer", holding(true, false));
    println!("\n== cena 105: a altura MEDIA da 2a metade, relativa a largada ==");
    println!("  baixo  {dive:+.4}");
    println!("  parado {idle:+.4}");
    println!("  cima   {rise:+.4}");
    println!("\n  pub(crate) const DIVE_IDLE_RISE: [f32; 3] = [{dive:.4}, {idle:.4}, {rise:.4}];");
    // E a boia, para o contraste do passo 4.
    println!("\n  a BOIA (swim_speed 0), com o mesmo dedo:");
    for (label, input) in [
        ("baixo ", holding(false, true)),
        ("parado", PlayerInput::default()),
        ("cima  ", holding(true, false)),
    ] {
        println!("    {label} {:+.4}", swims("Floater", input));
    }
}

/// **Os números da mensagem são os DESTA cena** — o gate que impede o folheto.
#[test]
fn the_message_quotes_this_scenes_numbers() {
    let measured = [
        swims("Swimmer", holding(false, true)),
        swims("Swimmer", PlayerInput::default()),
        swims("Swimmer", holding(true, false)),
    ];
    for (i, (m, published)) in measured.iter().zip(DIVE_IDLE_RISE).enumerate() {
        assert!(
            (m - published).abs() < 0.05,
            "o numero {i} da mensagem ({published:+.4}) nao e' o que a cena mede ({m:+.4}); \
             rode a sonda `probe_smoke_105` e atualize o DIVE_IDLE_RISE"
        );
    }
}

/// **A ORDEM é a que a mensagem promete** — baixo, parado, cima.
///
/// ⚠️ É a propriedade, e ela sobrevive a qualquer afinação dos defaults; os
/// literais acima são só o que a mensagem IMPRIME.
#[test]
fn down_dives_and_up_rises_in_this_scene() {
    let dive = swims("Swimmer", holding(false, true));
    let idle = swims("Swimmer", PlayerInput::default());
    let rise = swims("Swimmer", holding(true, false));
    assert!(dive < idle && idle < rise, "{dive} < {idle} < {rise}");
    assert!(dive < 0.0, "o passo 4 promete um MERGULHO: {dive}");
}

/// **O VERDE NADA COMO O AZUL** — a pergunta do Enio, na cena.
///
/// ⚠️ **A afirmação é DIRECIONAL e não de igualdade**, e a diferença é real: um
/// corpo cinemático integra a água ele mesmo (`kinematic_advance`) enquanto o
/// dinâmico a recebe do solver por sub-passo, então os dois **repousam a alturas
/// ligeiramente diferentes** — o mesmo motivo pelo qual a cena 101 monta os dois
/// modos a alturas distintas de propósito. O que a cena promete, e este gate
/// pina, é que **a espécie do corpo não é uma pergunta que a água faça**.
#[test]
fn the_kinematic_swimmer_answers_the_same_buttons() {
    let dive = swims("KinSwimmer", holding(false, true));
    let idle = swims("KinSwimmer", PlayerInput::default());
    let rise = swims("KinSwimmer", holding(true, false));
    assert!(dive < idle && idle < rise, "{dive} < {idle} < {rise}");
    assert!(dive < 0.0, "o verde tambem MERGULHA: {dive}");
}

/// **A BOIA não obedece aos botões** — o controle da cena, e o que faz da
/// diferença um knob em vez de uma coincidência.
#[test]
fn the_floater_ignores_the_buttons() {
    let dive = swims("Floater", holding(false, true));
    let idle = swims("Floater", PlayerInput::default());
    let rise = swims("Floater", holding(true, false));
    assert!(
        (dive - idle).abs() < 1.0e-3 && (rise - idle).abs() < 1.0e-3,
        "sem a capacidade os botoes sao mudos: {dive} / {idle} / {rise}"
    );
    assert!(
        (idle - FLOATER_RISE).abs() < 0.05,
        "e o numero que a mensagem publica e' o desta cena: {idle} vs {FLOATER_RISE}"
    );
}

/// ⚠️ **A POÇA RASA É RASA** — o número que o passo 1 promete, medido na
/// geometria que a cena de facto monta.
///
/// Sem este gate, mexer na altura da poça (ou na `float_height`) faria a
/// mensagem prometer uma caminhada onde o artista veria um nado, e nada
/// reclamaria.
#[test]
fn the_shallow_puddle_is_below_the_threshold() {
    // O centro do corpo de pé no cais, e a superfície da poça rasa.
    let standing = FLOAT;
    let surface = PUDDLE_X.mul_add(0.0, PUDDLE_HALF[1] * 2.0);
    assert!(
        standing > surface,
        "de pe' no cais o centro ({standing}) tem de ficar ACIMA da superficie ({surface})"
    );
    // 20% submerso é o que a tabela do `measure_the_swim_threshold` lê como
    // `0,68` — abaixo do limiar default de `1,0`.
    let sunk =
        (surface - (standing - (CAP_HALF_H + CAP_RADIUS))) / (2.0 * (CAP_HALF_H + CAP_RADIUS));
    assert!(
        (0.10..0.30).contains(&sunk),
        "a poca rasa tem de molhar entre 10% e 30% do corpo: {sunk}"
    );
}

/// **Os três corpos são comparáveis** — mesma forma, mesma densidade, e só a
/// capacidade (e a espécie) diferem.
///
/// ⚠️ **A comparação é contra o PRIMEIRO**, e é o que faz o gate escalar: com
/// pares escritos à mão, o sujeito número quatro nasce fora da comparação sem
/// ninguém reclamar.
#[test]
fn the_two_subjects_differ_only_in_the_capability() {
    let mut sim = SimWorld::new();
    build_swim_scene(sim.world_mut());
    let mut seen: Vec<(String, ph2d_physics_ecs::PlatformPlayer, ColliderShape, f32)> = Vec::new();
    let mut q = sim
        .world()
        .try_query::<(&Name, &ph2d_physics_ecs::PlatformPlayer, &Collider)>()
        .unwrap();
    for (n, p, c) in q.iter(sim.world()) {
        seen.push((n.as_str().to_string(), *p, c.shape, c.density));
    }
    assert_eq!(seen.len(), 3, "a cena monta TRES sujeitos: {seen:?}");
    let head = &seen[0];
    for other in &seen[1..] {
        assert_eq!(head.2, other.2, "a forma tem de ser a mesma: {other:?}");
        assert!(
            (head.3 - other.3).abs() < 1.0e-6,
            "e a densidade tambem: {other:?}"
        );
        // A única diferença permitida na LEI é a velocidade de nado.
        let mut a_cfg = head.1;
        a_cfg.swim_speed = other.1.swim_speed;
        assert_eq!(
            a_cfg, other.1,
            "so' a `swim_speed` pode diferir — senao o artista compara outra coisa"
        );
    }
    let speeds: Vec<f32> = seen.iter().map(|s| s.1.swim_speed).collect();
    assert!(
        speeds.iter().any(|s| *s < 0.5) && speeds.iter().any(|s| *s > 0.5),
        "e ela TEM de diferir, senao nao ha' ablacao: {speeds:?}"
    );
}
