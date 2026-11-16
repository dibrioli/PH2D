//! Grounded e Speed de um PERSONAGEM (`PlatformPlayer`) pelo rapier de verdade: a cena é a mesma dos
//! gates de comportamento da `ph2d-physics-ecs` (incluída, não copiada).

#[path = "../../../../crates/ph2d-physics-ecs/tests/it/platform_scene.rs"]
mod platform_scene;

use ph2d_app_components::motion_input_bridge::frame;
use ph2d_app_components::motion_input_fonte::{Fontes, no_chao, velocidade};
use ph2d_ecs::motion_inputs::Valor;
use ph2d_ecs::{ChildOf, MotionInput, MotionInputs, MotionSource, SimWorld, Transform};
use ph2d_physics_ecs::PhysicsBridge;
use ph2d_runtime::SignalOutbox;
use std::collections::BTreeMap;

fn numero(sim: &SimWorld, nome: &str) -> Option<f32> {
    let m = sim.world().get_resource::<MotionInputs>()?;
    match m.valores().find(|(n, _)| *n == nome)?.1.as_slice() {
        [Valor::Numero(v)] => Some(*v),
        _ => None,
    }
}

/// O personagem da cena, a `altura` acima do ponto em que pousa, com Speed nele e Grounded num
/// filho; `ticks` de física e um quadro da ponte.
fn personagem(altura: f32, ticks: u64) -> (SimWorld, PhysicsBridge, ph2d_ecs::Entity) {
    let (mut sim, mut bridge, p) = platform_scene::scene(0.0, 0.0);
    sim.world_mut()
        .get_mut::<Transform>(p)
        .unwrap()
        .translation
        .y += altura;
    let mi = |nome: &str, source| MotionInput {
        name: nome.into(),
        source,
        ..MotionInput::default()
    };
    sim.world_mut()
        .entity_mut(p)
        .insert(mi("rapidez", MotionSource::Speed));
    sim.world_mut()
        .spawn((mi("chao", MotionSource::Grounded), ChildOf(p)));
    for t in 1..=ticks {
        bridge.dispatch(&mut sim, true, t);
    }
    let accoes = BTreeMap::new();
    let f = Fontes {
        dt: 1.0 / 60.0,
        physics: Some(&bridge),
        accoes: &accoes,
        registry: None,
    };
    frame(&mut sim, &SignalOutbox::new(), &f);
    (sim, bridge, p)
}

/// ⭐ Grounded: `1` com o personagem em pé no chão, `0` no ar — a postura `Ground` do rapier.
#[test]
fn grounded_is_one_on_the_floor_and_zero_in_the_air() {
    let (sim, bridge, p) = personagem(0.0, 60);
    assert_eq!(no_chao(sim.world(), Some(&bridge), p), Some(1.0));
    assert_eq!(numero(&sim, "chao"), Some(1.0), "em pé no chão");
    let (sim, bridge, p) = personagem(20.0, 10);
    assert_eq!(no_chao(sim.world(), Some(&bridge), p), Some(0.0));
    assert_eq!(numero(&sim, "chao"), Some(0.0), "a cair, 20 m acima");
}

/// ⭐ Speed de um personagem: o módulo de `PlayerView::velocity` (a cair, > 0; parado no chão, ~0).
#[test]
fn speed_reads_the_character_velocity() {
    let (sim, bridge, p) = personagem(20.0, 20);
    let v = bridge
        .player_view(p)
        .expect("o personagem tem leitura")
        .velocity;
    let esperado = v[0].hypot(v[1]);
    assert!(esperado > 1.0, "um terço de segundo a cair: {esperado}");
    assert_eq!(velocidade(sim.world(), Some(&bridge), p), Some(esperado));
    assert_eq!(numero(&sim, "rapidez"), Some(esperado));
    let (sim, _, _) = personagem(0.0, 120);
    let parado = numero(&sim, "rapidez").expect("Speed em pé");
    assert!(parado < 0.05, "em pé sem entrada: {parado}");
}

/// O cartão de um objecto com Motion Input e vida `4/10` (o arnês do `motion_input_inspector_tests`).
fn cena() -> (SimWorld, u64) {
    let mut sim = SimWorld::default();
    let e = sim.world_mut().spawn(MotionInput::default()).id();
    (sim, e.to_bits())
}

fn com_vida(sim: &mut SimWorld, bits: u64, pontos: f64) {
    sim.world_mut()
        .entity_mut(ph2d_ecs::Entity::from_bits(bits))
        .insert((
            ph2d_physics_ecs::Health {
                max: 10.0,
                start: 10.0,
                ..Default::default()
            },
            ph2d_physics_ecs::HealthNow {
                pontos,
                escudo: 0.0,
                morta: false,
            },
        ));
}

/// ⭐ Property parada: o instantâneo lê o campo pelo funil; em Play, o que a ponte publicou.
#[test]
fn a_property_parada_le_o_campo_e_em_play_o_que_a_ponte_publicou() {
    use ph2d_editor_core::motion_input_edits::MotionInputAoVivo::Numero as N;
    let ((mut sim, bits), reg) = (cena(), crate::comp::registo());
    let e = ph2d_ecs::Entity::from_bits(bits);
    let h = reg
        .iter()
        .filter_map(|r| r.desc)
        .find(|d| d.fields.iter().any(|f| f.serde_key() == "invincible_s"));
    let mut m = sim.world_mut().get_mut::<MotionInput>(e).unwrap();
    (m.source, m.property) = (
        MotionSource::Property,
        format!("{}/max", h.unwrap().canonical_name),
    );
    com_vida(&mut sim, bits, 4.0);
    let info = |s: &SimWorld, j| {
        ph2d_app_components::motion_input_inspector::build_info(
            s,
            bits,
            (1, j),
            Some(&reg),
            (Vec::new, Vec::new),
        )
        .unwrap()
        .ao_vivo
    };
    assert_eq!(info(&sim, false), Some(N(10.0)), "parado: o campo");
    let (a, out) = (Default::default(), ph2d_runtime::SignalOutbox::new());
    let f = ph2d_app_components::motion_input_fonte::Fontes {
        dt: 0.0,
        physics: None,
        accoes: &a,
        registry: Some(&reg),
    };
    ph2d_app_components::motion_input_bridge::frame(&mut sim, &out, &f);
    sim.world_mut()
        .get_mut::<ph2d_physics_ecs::Health>(e)
        .unwrap()
        .max = 20.0;
    assert_eq!(
        (info(&sim, true), info(&sim, false)),
        (Some(N(10.0)), Some(N(20.0))),
        "Play: o publicado"
    );
}

fn par(sim: &SimWorld, nome: &str) -> Option<[f32; 2]> {
    let m = sim.world().get_resource::<MotionInputs>()?;
    match m.valores().find(|(n, _)| *n == nome)?.1.as_slice() {
        [Valor::Vetor(v)] => Some(*v),
        _ => None,
    }
}

/// ⭐ MI-5 Velocity: o PAR da velocidade do corpo do pai (a cair, `y < 0`), o mesmo de que o Speed é o tamanho.
#[test]
fn velocity_sends_the_pair_of_a_moving_body() {
    let (mut sim, bridge, p) = personagem(20.0, 20);
    let mi = MotionInput {
        name: "vel".into(),
        source: MotionSource::Velocity,
        ..MotionInput::default()
    };
    sim.world_mut().spawn((mi, ChildOf(p)));
    let accoes = BTreeMap::new();
    let f = Fontes {
        dt: 1.0 / 60.0,
        physics: Some(&bridge),
        accoes: &accoes,
        registry: None,
    };
    frame(&mut sim, &SignalOutbox::new(), &f);
    let v = par(&sim, "vel").expect("Velocity manda um par");
    assert!(v[1] < -1.0, "a cair: {v:?}");
    assert_eq!(Some(v[0].hypot(v[1])), numero(&sim, "rapidez"));
}

/// ⭐ MI-5 Move: `(direita − esquerda, cima − baixo)` das acções `move_*`; Remap 0→2 age no TAMANHO
/// (a direção fica) e sem nenhuma `move_*` no mapa a linha sai.
#[test]
fn move_sends_the_2d_action_and_remap_keeps_its_direction() {
    let mut sim = SimWorld::default();
    let mut mi = MotionInput {
        name: "andar".into(),
        source: MotionSource::Move,
        ..MotionInput::default()
    };
    let e = sim.world_mut().spawn(mi.clone()).id();
    let forca = |s| ph2d_ecs::ActionSample {
        strength: s,
        ..Default::default()
    };
    let accoes: BTreeMap<String, _> = [
        ("move_right", forca(1.0)),
        ("move_left", forca(0.0)),
        ("move_up", forca(1.0)),
        ("move_down", forca(0.0)),
    ]
    .into_iter()
    .map(|(n, a)| (n.to_string(), a))
    .collect();
    let quadro = |sim: &mut SimWorld, accoes: &BTreeMap<String, _>| {
        let f = Fontes {
            dt: 1.0 / 60.0,
            physics: None,
            accoes,
            registry: None,
        };
        frame(sim, &SignalOutbox::new(), &f);
    };
    quadro(&mut sim, &accoes);
    assert_eq!(par(&sim, "andar"), Some([1.0, 1.0]));
    (mi.remap, mi.remap_from, mi.remap_to) = (true, 0.0, 2.0 * 2f32.sqrt());
    sim.world_mut().entity_mut(e).insert(mi);
    quadro(&mut sim, &accoes);
    let v = par(&sim, "andar").expect("par");
    let k = 0.5 / 2f32.sqrt();
    assert!(
        (v[0] - k).abs() < 1e-6 && (v[1] - k).abs() < 1e-6,
        "tamanho ½, a 45°: {v:?}"
    );
    quadro(&mut sim, &BTreeMap::new());
    assert_eq!(par(&sim, "andar"), None, "sem move_* a fonte não existe");
}

/// ⭐ MI-5 Property de campo com DUAS faixas manda o PAR (`Valor::Vetor`), nunca um número:
/// `ScrollMotion/velocity = (3, −4)` do pai chega inteiro ao `value.input`.
#[test]
fn a_property_of_a_two_lane_field_sends_the_pair() {
    let reg = crate::comp::registo();
    let mut sim = SimWorld::default();
    let velocity = [3.0, -4.0];
    let pai = sim
        .world_mut()
        .spawn(ph2d_ecs::ScrollMotion { velocity })
        .id();
    let mi = MotionInput {
        name: "par".into(),
        source: MotionSource::Property,
        property: "ph2d::ecs::ScrollMotion/velocity".into(),
        ..MotionInput::default()
    };
    sim.world_mut().spawn((mi, ChildOf(pai)));
    let accoes = BTreeMap::new();
    let f = Fontes {
        dt: 1.0 / 60.0,
        physics: None,
        accoes: &accoes,
        registry: Some(&reg),
    };
    frame(&mut sim, &SignalOutbox::new(), &f);
    assert_eq!(par(&sim, "par"), Some(velocity), "o par inteiro");
    assert_eq!(numero(&sim, "par"), None, "um par não vira número");
}
