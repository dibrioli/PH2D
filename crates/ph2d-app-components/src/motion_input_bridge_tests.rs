use super::*;
use ph2d_physics_ecs::{Health, HealthNow};
use ph2d_runtime::Signal;
use std::collections::BTreeMap;

fn valor(sim: &SimWorld, nome: &str) -> Vec<Valor> {
    sim.world()
        .get_resource::<MotionInputs>()
        .map(|m| {
            m.valores()
                .find(|(n, _)| *n == nome)
                .map(|(_, l)| l)
                .unwrap_or_default()
        })
        .unwrap_or_default()
}

fn disparos(sim: &SimWorld, nome: &str) -> Vec<u64> {
    sim.world()
        .get_resource::<MotionInputs>()
        .and_then(|m| m.disparos().find(|(n, _)| *n == nome).map(|(_, l)| l))
        .unwrap_or_default()
}

/// Um quadro sem física, sem acções e sem registo (as fontes da onda 1).
fn quadro(sim: &mut SimWorld, out: &SignalOutbox) {
    let accoes = BTreeMap::new();
    let f = Fontes {
        dt: 1.0 / 60.0,
        physics: None,
        accoes: &accoes,
        registry: None,
    };
    frame(sim, out, &f);
}

fn entrada(nome: &str, source: MotionSource, value: f32) -> MotionInput {
    MotionInput {
        name: nome.into(),
        source,
        value,
        ..MotionInput::default()
    }
}

/// ⭐ Um objecto publica o número do componente; dois objectos, duas linhas pela ordem do NOME; um
/// componente retirado deixa de publicar.
#[test]
fn each_object_publishes_its_field_and_a_removed_one_stops() {
    let mut sim = SimWorld::new();
    let out = SignalOutbox::new();
    let b = sim
        .world_mut()
        .spawn((
            entrada("estado", MotionSource::Value, 1.0),
            Name("B".into()),
        ))
        .id();
    sim.world_mut().spawn((
        entrada("estado", MotionSource::Value, 0.5),
        Name("A".into()),
    ));
    quadro(&mut sim, &out);
    assert_eq!(
        valor(&sim, "estado"),
        vec![Valor::Numero(0.5), Valor::Numero(1.0)]
    );
    sim.world_mut().entity_mut(b).remove::<MotionInput>();
    quadro(&mut sim, &out);
    assert_eq!(valor(&sim, "estado"), vec![Valor::Numero(0.5)]);
}

fn com_vida(sim: &mut SimWorld, m: MotionInput) -> bevy_ecs::entity::Entity {
    let h = Health {
        max: 10.0,
        start: 10.0,
        ..Health::default()
    };
    let agora = HealthNow {
        pontos: 10.0,
        escudo: 0.0,
        morta: false,
    };
    sim.world_mut().spawn((h, agora, m)).id()
}

/// ⭐ A vida como razão `0..1`, e o DANO como um disparo por descida (subir não dispara). Um
/// objecto tem UM `Motion Input`: a vida e o dano são dois objectos (ou um `pulse.*` sobre a vida,
/// no grafo).
#[test]
fn health_publishes_its_ratio_and_damage_fires_on_each_drop() {
    let mut sim = SimWorld::new();
    let out = SignalOutbox::new();
    let heroi = com_vida(&mut sim, entrada("vida", MotionSource::Health, 0.0));
    let alvo = com_vida(&mut sim, entrada("dano", MotionSource::Damage, 0.0));
    quadro(&mut sim, &out);
    assert!(disparos(&sim, "dano").is_empty(), "nascer não é dano");
    for (pontos, esperado) in [(7.0, 1u64), (7.0, 1), (9.0, 1), (3.0, 2)] {
        for e in [heroi, alvo] {
            sim.world_mut().get_mut::<HealthNow>(e).unwrap().pontos = pontos;
        }
        quadro(&mut sim, &out);
        assert_eq!(disparos(&sim, "dano"), vec![esperado], "a {pontos} pontos");
    }
    assert_eq!(valor(&sim, "vida"), vec![Valor::Numero(0.3)]);
}

/// ⭐ Um sinal do runtime dispara o gatilho com o nome dele.
#[test]
fn a_runtime_signal_fires_the_trigger_of_its_name() {
    let mut sim = SimWorld::new();
    let mut out = SignalOutbox::new();
    quadro(&mut sim, &out);
    out.publish(Signal::from_control("boom"));
    quadro(&mut sim, &out);
    quadro(&mut sim, &out);
    assert_eq!(
        disparos(&sim, "boom"),
        vec![1],
        "um sinal, um disparo (lido uma vez)"
    );
}

/// ⭐ MI-12: a fonte some em Play (tiram a `Health`) ⇒ a linha do objecto SAI, e o `value.input`
/// volta ao número dele próprio — nunca fica preso no último valor. Trocar a fonte também limpa.
#[test]
fn a_source_that_disappears_stops_publishing_instead_of_freezing() {
    let mut sim = SimWorld::new();
    let out = SignalOutbox::new();
    let e = com_vida(&mut sim, entrada("vida", MotionSource::Health, 0.0));
    quadro(&mut sim, &out);
    assert_eq!(valor(&sim, "vida"), vec![Valor::Numero(1.0)]);
    sim.world_mut()
        .entity_mut(e)
        .remove::<(Health, HealthNow)>();
    quadro(&mut sim, &out);
    assert_eq!(
        valor(&sim, "vida"),
        Vec::<Valor>::new(),
        "sem Health, nada fica preso"
    );

    let d = com_vida(&mut sim, entrada("golpe", MotionSource::Damage, 0.0));
    sim.world_mut().get_mut::<HealthNow>(d).unwrap().pontos = 4.0;
    quadro(&mut sim, &out);
    quadro(&mut sim, &out);
    sim.world_mut().get_mut::<HealthNow>(d).unwrap().pontos = 2.0;
    quadro(&mut sim, &out);
    assert_eq!(
        disparos(&sim, "golpe"),
        vec![1],
        "com a vida presente o dano conta"
    );
    sim.world_mut()
        .entity_mut(d)
        .remove::<(Health, HealthNow)>();
    quadro(&mut sim, &out);
    assert_eq!(disparos(&sim, "golpe"), Vec::<u64>::new());
}

fn numero(sim: &SimWorld, nome: &str) -> Option<f32> {
    match valor(sim, nome).as_slice() {
        [Valor::Numero(v)] => Some(*v),
        _ => None,
    }
}

/// ⭐ Onda 2 — From: a vida de OUTRO objecto, pelo `StableId` (o nome no painel); um From que
/// ninguém tem faz a linha SAIR.
#[test]
fn from_reads_another_objects_health_and_leaves_when_the_name_is_gone() {
    let mut sim = SimWorld::new();
    let out = SignalOutbox::new();
    let heroi = com_vida(&mut sim, MotionInput::default());
    sim.world_mut().entity_mut(heroi).remove::<MotionInput>();
    sim.world_mut()
        .entity_mut(heroi)
        .insert((Name("Hero".into()), ph2d_ecs::StableId(77)));
    sim.world_mut().get_mut::<HealthNow>(heroi).unwrap().pontos = 5.0;
    let mut m = entrada("vida", MotionSource::Health, 0.0);
    m.from = 77;
    let painel = sim.world_mut().spawn(m).id();
    quadro(&mut sim, &out);
    assert_eq!(numero(&sim, "vida"), Some(0.5), "lê a vida do Hero");
    sim.world_mut().get_mut::<MotionInput>(painel).unwrap().from = 78;
    quadro(&mut sim, &out);
    assert_eq!(numero(&sim, "vida"), None, "From sem objecto: a linha sai");
}

/// ⭐ Action: a força da acção pelo nome (apertada 1, solta 0); acção desconhecida sai.
#[test]
fn action_publishes_the_strength_of_the_named_action() {
    let mut sim = SimWorld::new();
    let out = SignalOutbox::new();
    let mut m = entrada("salto", MotionSource::Action, 0.0);
    m.action = "jump".into();
    let e = sim.world_mut().spawn(m).id();
    let mut accoes = BTreeMap::new();
    // A força é ANALÓGICA (gatilho meio apertado = 0.5), nunca o «pressed» digital.
    for (forca, esperado) in [(1.0, Some(1.0)), (0.5, Some(0.5)), (0.0, Some(0.0))] {
        accoes.insert(
            "jump".to_string(),
            ph2d_ecs::ActionSample {
                pressed: forca > 0.0,
                strength: forca,
                ..Default::default()
            },
        );
        let f = Fontes {
            dt: 0.1,
            physics: None,
            accoes: &accoes,
            registry: None,
        };
        frame(&mut sim, &out, &f);
        assert_eq!(numero(&sim, "salto"), esperado);
    }
    sim.world_mut().get_mut::<MotionInput>(e).unwrap().action = "fly".into();
    let f = Fontes {
        dt: 0.1,
        physics: None,
        accoes: &accoes,
        registry: None,
    };
    frame(&mut sim, &out, &f);
    assert_eq!(
        numero(&sim, "salto"),
        None,
        "acção desconhecida: a linha sai"
    );
}

/// ⭐ Property: um campo numérico pelo funil da Timeline (`Health/max`); campo ausente sai.
#[test]
fn property_reads_a_numeric_field_and_a_missing_one_leaves() {
    let mut sim = SimWorld::new();
    let out = SignalOutbox::new();
    let reg = crate::component_registry_for_tests::registo();
    let canonical = reg
        .iter()
        .filter_map(|r| r.desc)
        .find(|d| d.fields.iter().any(|f| f.serde_key() == "invincible_s"))
        .expect("Health no registo")
        .canonical_name;
    let mut m = entrada("max", MotionSource::Property, 0.0);
    m.property = format!("{canonical}/max");
    let e = com_vida(&mut sim, m);
    let accoes = BTreeMap::new();
    let f = Fontes {
        dt: 0.1,
        physics: None,
        accoes: &accoes,
        registry: Some(&reg),
    };
    frame(&mut sim, &out, &f);
    assert_eq!(numero(&sim, "max"), Some(10.0));
    sim.world_mut().get_mut::<MotionInput>(e).unwrap().property = format!("{canonical}/nada");
    frame(&mut sim, &out, &f);
    assert_eq!(numero(&sim, "max"), None, "campo ausente: a linha sai");
}

/// ⭐ Speed: o módulo da velocidade de um corpo a cair; sem física a fonte não existe. Grounded
/// sem personagem de plataforma também não existe.
#[test]
fn speed_reads_a_moving_body_and_grounded_needs_a_platform_player() {
    use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};
    let mut sim = SimWorld::new();
    let out = SignalOutbox::new();
    let corpo = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.5 },
                ..Collider::default()
            },
            ph2d_ecs::Transform::IDENTITY,
            entrada("rapidez", MotionSource::Speed, 0.0),
        ))
        .id();
    let mut chao = entrada("chao", MotionSource::Grounded, 0.0);
    chao.from = 0;
    let filho = sim.world_mut().spawn((chao, ph2d_ecs::ChildOf(corpo))).id();
    quadro(&mut sim, &out);
    assert_eq!(numero(&sim, "rapidez"), None, "sem física, sem Speed");
    let mut bridge = PhysicsBridge::new();
    for t in 1..=30u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let accoes = BTreeMap::new();
    let f = Fontes {
        dt: 0.1,
        physics: Some(&bridge),
        accoes: &accoes,
        registry: None,
    };
    frame(&mut sim, &out, &f);
    let v = numero(&sim, "rapidez").expect("o corpo cai");
    assert!(v > 1.0, "a cair meio segundo: {v}");
    assert_eq!(numero(&sim, "chao"), None, "sem PlatformPlayer não há chão");
    assert!(crate::motion_input_fonte::corpo(sim.world(), filho) == Some(corpo));
}

/// ⭐ Remap 0..900 → 450 = 0.5, com clamp nas pontas.
#[test]
fn remap_maps_the_range_to_zero_one_with_clamp() {
    let mut m = MotionInput {
        remap: true,
        remap_from: 0.0,
        remap_to: 900.0,
        ..MotionInput::default()
    };
    assert_eq!(crate::motion_input_fonte::remapeia(&m, 450.0), 0.5);
    assert_eq!(crate::motion_input_fonte::remapeia(&m, 2000.0), 1.0);
    assert_eq!(crate::motion_input_fonte::remapeia(&m, -5.0), 0.0);
    m.remap_to = 0.0;
    assert_eq!(
        crate::motion_input_fonte::remapeia(&m, 3.0),
        0.0,
        "from == to ⇒ 0"
    );
    m.remap = false;
    assert_eq!(
        crate::motion_input_fonte::remapeia(&m, 3.0),
        3.0,
        "sem a caixa o número passa"
    );
}

/// ⭐ A curva aplica-se DEPOIS do Remap, sobre o `t` em `0..1`: Ease In em 450 de 0..900 é a curva em 0.5
/// (abaixo da recta), nunca a curva sobre 450 cru (que satura) remapeada.
#[test]
fn the_curve_shapes_the_remapped_t_not_the_raw_value() {
    let ease_in = ph2d_editor_core::motion_input_edits::CURVE_PRESETS[1].1;
    let c = ph2d_curve::parse(ease_in).expect("Ease In");
    let m = MotionInput {
        remap: true,
        remap_from: 0.0,
        remap_to: 900.0,
        curve: ease_in.into(),
        ..MotionInput::default()
    };
    let v = crate::motion_input_fonte::remapeia(&m, 450.0);
    assert_eq!(v, c.eval(0.5));
    assert!(v < 0.45, "Ease In fica abaixo da recta em 0.5: {v}");
}

/// ⭐ O dt do jogo: parado = 0 (o Smooth espera e um scrub parado não o escorre); a andar = passos × passo.
#[test]
fn the_game_dt_is_zero_when_stopped() {
    use crate::motores::Relogio;
    assert_eq!(Relogio::do_quadro(false, 3, 0.5).dt_de_jogo(), 0.0);
    assert_eq!(Relogio::do_quadro(true, 3, 0.5).dt_de_jogo(), 1.5);
}

/// ⭐ Smooth: meia-vida 1 s, de 0 para 1 dá 0.5 depois de 1 s (em vários dt); o 1.º quadro nasce no alvo.
#[test]
fn smooth_halves_the_gap_per_half_life_and_is_born_on_target() {
    let mut sim = SimWorld::new();
    let out = SignalOutbox::new();
    let e = sim
        .world_mut()
        .spawn(MotionInput {
            name: "v".into(),
            smooth: 1.0,
            ..MotionInput::default()
        })
        .id();
    let accoes = BTreeMap::new();
    let passo = |sim: &mut SimWorld, dt: f32| {
        let f = Fontes {
            dt,
            physics: None,
            accoes: &accoes,
            registry: None,
        };
        frame(sim, &out, &f);
    };
    sim.world_mut().get_mut::<MotionInput>(e).unwrap().value = 0.7;
    passo(&mut sim, 0.1);
    assert_eq!(numero(&sim, "v"), Some(0.7), "nasce no alvo (não em 0)");
    sim.world_mut().get_mut::<MotionInput>(e).unwrap().value = 0.0;
    passo(&mut sim, 40.0);
    sim.world_mut().get_mut::<MotionInput>(e).unwrap().value = 1.0;
    for dt in [0.25, 0.125, 0.125, 0.5] {
        passo(&mut sim, dt);
    }
    let v = numero(&sim, "v").unwrap();
    assert!((v - 0.5).abs() < 1e-5, "meia-vida de 1 s: {v}");
}

/// ⭐ MI-10: a entrada mora num filho. `from = 0` lê o PAI; dois filhos Damage no mesmo pai disparam cada um
/// (estado próprio, chave = bits do filho); o escritor fala pelo nome do pai; a linha diz «Motion Input · <nome>».
#[test]
fn a_motion_input_child_reads_its_parent_and_siblings_keep_their_own_state() {
    let mut sim = SimWorld::new();
    let out = SignalOutbox::new();
    let pai = com_vida(&mut sim, entrada("x", MotionSource::Value, 0.0));
    sim.world_mut()
        .entity_mut(pai)
        .remove::<MotionInput>()
        .insert(Name("Dummy".into()));
    let filho = |sim: &mut SimWorld, m: MotionInput| {
        let e = ph2d_physics_ecs::pendura_entrada(sim.world_mut(), None, Some(pai), "");
        sim.world_mut().entity_mut(e).insert(m);
        e
    };
    let vida = filho(&mut sim, entrada("vida", MotionSource::Health, 0.0));
    filho(&mut sim, entrada("d1", MotionSource::Damage, 0.0));
    filho(&mut sim, entrada("d2", MotionSource::Damage, 0.0));
    quadro(&mut sim, &out);
    sim.world_mut().get_mut::<HealthNow>(pai).unwrap().pontos = 4.0;
    quadro(&mut sim, &out);
    assert_eq!(
        valor(&sim, "vida"),
        vec![Valor::Numero(0.4)],
        "from = 0 é o pai"
    );
    assert_eq!(
        (disparos(&sim, "d1"), disparos(&sim, "d2")),
        (vec![1], vec![1])
    );
    let m = sim.world().get_resource::<MotionInputs>().unwrap();
    let quem = m.primeira_linha("vida").map(|(e, _)| e.clone());
    assert!(
        matches!(quem, Some(Escritor::Objeto { nome, .. }) if nome == "Dummy"),
        "o escritor é o pai"
    );
    let rotulo = sim.world().get::<Name>(vida).map(|n| n.0.clone());
    assert_eq!(rotulo.as_deref(), Some("Motion Input \u{b7} vida"));
}
