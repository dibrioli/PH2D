//! A ida (edição → componente) e a volta (componente → instantâneo) da secção MOTION INPUT.

use ph2d_ecs::{MotionInput, MotionSource, SimWorld};
use ph2d_editor_core::motion_input_edits::MotionInputFieldEdit as E;

use super::{apply, apply_all, build_info, source_de_u8};

fn cena() -> (SimWorld, u64) {
    let mut sim = SimWorld::default();
    let e = sim.world_mut().spawn(MotionInput::default()).id();
    (sim, e.to_bits())
}

fn comp(sim: &SimWorld, bits: u64) -> MotionInput {
    sim.world()
        .get::<MotionInput>(ph2d_ecs::Entity::from_bits(bits))
        .expect("o componente")
        .clone()
}

/// O instantâneo lê cada campo do componente, e a lista do grafo vem de quem a passa.
#[test]
fn o_instantaneo_reflecte_o_componente() {
    let (mut sim, bits) = cena();
    {
        let e = ph2d_ecs::Entity::from_bits(bits);
        let mut m = sim
            .world_mut()
            .get_mut::<MotionInput>(e)
            .expect("o componente");
        m.name = "hp".into();
        m.source = MotionSource::Color;
        m.value = 0.25;
        m.color = [0.1, 0.2, 0.3, 0.4];
    }
    let info = build_info(&sim, bits, 2, || vec!["hp".into(), "speed".into()]).expect("info");
    assert_eq!(info.entity_bits, bits);
    assert_eq!(info.name, "hp");
    assert_eq!(info.source, MotionSource::Color.index());
    assert_eq!(info.value.to_bits(), 0.25_f32.to_bits());
    assert_eq!(info.color, [0.1, 0.2, 0.3, 0.4]);
    assert_eq!(info.nomes_do_grafo, vec!["hp".to_string(), "speed".into()]);
    assert!(!info.tem_vida, "sem Health, sem vida");
    assert_eq!(info.selected_count, 2);

    // Com `Health` a coluna muda — a mesma porta que a ponte lê.
    sim.world_mut()
        .entity_mut(ph2d_ecs::Entity::from_bits(bits))
        .insert(ph2d_physics_ecs::Health {
            max: 10.0,
            start: 10.0,
            ..Default::default()
        });
    assert!(build_info(&sim, bits, 1, Vec::new).expect("info").tem_vida);
}

/// Sem o componente não há secção — e a varredura do grafo nem corre.
#[test]
fn sem_componente_nao_ha_seccao() {
    let mut sim = SimWorld::default();
    let bits = sim.world_mut().spawn_empty().id().to_bits();
    let info = build_info(&sim, bits, 1, || {
        panic!("o grafo não se varre sem o componente")
    });
    assert!(info.is_none());
}

/// «O campo já tem o valor da edição?»
type Escreveu = fn(&MotionInput) -> bool;

/// Cada edição escreve o seu campo e devolve `true`; repetida, devolve `false` (o undo é por diff).
#[test]
fn cada_edicao_escreve_o_seu_campo_so_quando_muda() {
    let (mut sim, bits) = cena();
    let casos: [(E, Escreveu); 4] = [
        (E::Name("vida".into()), |m| m.name == "vida"),
        (E::Source(MotionSource::Damage.index()), |m| {
            m.source == MotionSource::Damage
        }),
        (E::Value(0.75), |m| m.value.to_bits() == 0.75_f32.to_bits()),
        (E::Color([0.5, 0.0, 1.0, 1.0]), |m| {
            m.color == [0.5, 0.0, 1.0, 1.0]
        }),
    ];
    for (edit, escreveu) in &casos {
        assert!(!escreveu(&comp(&sim, bits)), "CONTROLO: {edit:?} já valia");
        assert!(apply(&mut sim, bits, edit), "{edit:?} mudou o mundo");
        assert!(escreveu(&comp(&sim, bits)), "{edit:?} escreveu o campo");
        assert!(!apply(&mut sim, bits, edit), "{edit:?} repetida é inerte");
    }
    // Uma fonte fora da faixa e um número não-finito são inertes.
    assert!(!apply(&mut sim, bits, &E::Source(9)));
    assert!(!apply(&mut sim, bits, &E::Value(f32::NAN)));
    assert!(!apply_all(&mut sim, &[(bits, E::Name("vida".into()))]));
    assert!(apply_all(
        &mut sim,
        &[(bits, E::Name("vida".into())), (bits, E::Value(0.0))]
    ));
}

/// A ponte `u8` ⇄ fonte faz a ida-e-volta para toda fonte.
#[test]
fn a_fonte_faz_a_ida_e_volta() {
    for s in MotionSource::ALL {
        assert_eq!(source_de_u8(s.index()), Some(s));
    }
    assert_eq!(source_de_u8(MotionSource::ALL.len() as u8), None);
}
