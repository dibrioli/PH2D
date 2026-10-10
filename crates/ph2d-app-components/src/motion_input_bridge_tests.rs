use super::*;
use ph2d_physics_ecs::{Health, HealthNow};
use ph2d_runtime::Signal;

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
    frame(&mut sim, &out);
    assert_eq!(
        valor(&sim, "estado"),
        vec![Valor::Numero(0.5), Valor::Numero(1.0)]
    );
    sim.world_mut().entity_mut(b).remove::<MotionInput>();
    frame(&mut sim, &out);
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
    frame(&mut sim, &out);
    assert!(disparos(&sim, "dano").is_empty(), "nascer não é dano");
    for (pontos, esperado) in [(7.0, 1u64), (7.0, 1), (9.0, 1), (3.0, 2)] {
        for e in [heroi, alvo] {
            sim.world_mut().get_mut::<HealthNow>(e).unwrap().pontos = pontos;
        }
        frame(&mut sim, &out);
        assert_eq!(disparos(&sim, "dano"), vec![esperado], "a {pontos} pontos");
    }
    assert_eq!(valor(&sim, "vida"), vec![Valor::Numero(0.3)]);
}

/// ⭐ Um sinal do runtime dispara o gatilho com o nome dele.
#[test]
fn a_runtime_signal_fires_the_trigger_of_its_name() {
    let mut sim = SimWorld::new();
    let mut out = SignalOutbox::new();
    frame(&mut sim, &out);
    out.publish(Signal::from_control("boom"));
    frame(&mut sim, &out);
    frame(&mut sim, &out);
    assert_eq!(
        disparos(&sim, "boom"),
        vec![1],
        "um sinal, um disparo (lido uma vez)"
    );
}
