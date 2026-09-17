//! A ponte dos emissores: a corrida, os sinais e o renascer.

use ph2d_ecs::{ParticleEmitter, SimWorld, StableId, Transform};

use super::ParticlesState;

const DT: f64 = 1.0 / 60.0;

fn cena(cfg: ParticleEmitter) -> (SimWorld, u64) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, StableId(1), cfg))
        .id();
    (sim, e.to_bits())
}

fn base() -> ParticleEmitter {
    ParticleEmitter {
        amount: 20,
        life: 1.0,
        gravity: [0.0, 0.0],
        ..ParticleEmitter::default()
    }
}

fn anda(st: &mut ParticlesState, sim: &mut SimWorld, n: u32, playing: bool) -> usize {
    let mut fim = 0;
    for _ in 0..n {
        fim += st
            .frame(sim, playing, 1, DT, &[], |_| Some(3))
            .finished
            .len();
    }
    fim
}

/// ⭐ **A corrida é o relógio a andar** — parado não nasce ninguém, e o que já vive continua a
/// desenhar-se.
#[test]
fn parado_nada_nasce_e_o_que_vive_desenha_se() {
    let (mut sim, _) = cena(base());
    let mut st = ParticlesState::default();
    anda(&mut st, &mut sim, 10, false);
    assert_eq!(st.instances.len(), 0, "parado desde o princípio: nada");
    anda(&mut st, &mut sim, 30, true);
    let vivas = st.instances.len();
    assert!(vivas > 3, "a correr nascem: {vivas}");
    anda(&mut st, &mut sim, 10, false);
    assert_eq!(st.instances.len(), vivas, "em pausa continuam desenhadas");
}

/// **A profundidade é a do objecto.**
#[test]
fn as_particulas_desenham_na_profundidade_do_objecto() {
    let (mut sim, _) = cena(base());
    let mut st = ParticlesState::default();
    anda(&mut st, &mut sim, 20, true);
    assert!(!st.instances.is_empty());
    assert!(st.instances.iter().all(|i| i.z_order == 3));
}

/// ⭐⭐ **Os sinais ligam, desligam e recomeçam** (L5–L7).
#[test]
fn os_sinais_ligam_desligam_e_recomecam() {
    let cfg = ParticleEmitter {
        emitting: false,
        start_on: "fogo".into(),
        stop_on: "chega".into(),
        restart_on: "outra".into(),
        finished_signal: "acabou".into(),
        ..base()
    };
    let (mut sim, bits) = cena(cfg);
    let mut st = ParticlesState::default();
    anda(&mut st, &mut sim, 20, true);
    assert_eq!(st.instances.len(), 0, "desligado: nada");
    st.frame(&mut sim, true, 1, DT, &["fogo".into()], |_| Some(0));
    anda(&mut st, &mut sim, 20, true);
    let vivas = st.instances.len();
    assert!(vivas > 3, "o sinal ligou: {vivas}");
    st.frame(&mut sim, true, 1, DT, &["chega".into()], |_| Some(0));
    anda(&mut st, &mut sim, 5, true);
    assert!(
        st.instances.len() <= vivas && !st.instances.is_empty(),
        "desligar não mata as vivas"
    );
    // Até morrer a última, e aí o grito.
    let mut gritos = Vec::new();
    for _ in 0..120 {
        gritos.extend(st.frame(&mut sim, true, 1, DT, &[], |_| Some(0)).finished);
    }
    assert_eq!(
        gritos,
        vec![(bits, "acabou".to_string())],
        "L2: um grito só"
    );
    assert_eq!(st.instances.len(), 0);
    // E recomeçar volta a encher.
    st.frame(&mut sim, true, 1, DT, &["outra".into()], |_| Some(0));
    anda(&mut st, &mut sim, 20, true);
    assert!(st.instances.len() > 3, "recomeçou");
}

/// ⭐⭐⭐ **Rebobinar é renascer** — as corridas somem e o emissor volta ao princípio.
#[test]
fn rebobinar_renasce() {
    let (mut sim, _) = cena(base());
    let mut st = ParticlesState::default();
    anda(&mut st, &mut sim, 40, true);
    assert_eq!(st.live_count(), 1);
    assert!(!st.instances.is_empty());
    assert_eq!(st.rewind(), 1, "uma corrida deitada fora");
    assert_eq!(st.instances.len(), 0, "e nada desenhado");
    let depois = anda(&mut st, &mut sim, 3, true);
    assert_eq!(depois, 0);
    assert!(
        st.instances.len() <= 2,
        "o emissor recomeçou do zero: {}",
        st.instances.len()
    );
}

/// **Editar o componente recomeça o emissor** — o grafo é outro.
#[test]
fn editar_o_componente_recomeca() {
    let (mut sim, bits) = cena(base());
    let mut st = ParticlesState::default();
    anda(&mut st, &mut sim, 40, true);
    let antes = st.instances.len();
    assert!(antes > 5);
    let e = ph2d_ecs::Entity::from_bits(bits);
    sim.world_mut().entity_mut(e).insert(ParticleEmitter {
        speed: 9.0,
        ..base()
    });
    anda(&mut st, &mut sim, 2, true);
    assert!(
        st.instances.len() < antes,
        "recomeçou: {} contra {antes}",
        st.instances.len()
    );
}

/// **Um objecto sem o componente não deixa corrida para trás.**
#[test]
fn tirar_o_componente_leva_a_corrida() {
    let (mut sim, bits) = cena(base());
    let mut st = ParticlesState::default();
    anda(&mut st, &mut sim, 20, true);
    assert_eq!(st.live_count(), 1);
    sim.world_mut()
        .entity_mut(ph2d_ecs::Entity::from_bits(bits))
        .remove::<ParticleEmitter>();
    anda(&mut st, &mut sim, 1, true);
    assert_eq!(st.live_count(), 0);
    assert_eq!(st.instances.len(), 0);
}
