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
    let info =
        build_info(&sim, bits, 2, false, || vec!["hp".into(), "speed".into()]).expect("info");
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
    assert!(
        build_info(&sim, bits, 1, false, Vec::new)
            .expect("info")
            .tem_vida
    );
}

/// Sem o componente não há secção — e a varredura do grafo nem corre.
#[test]
fn sem_componente_nao_ha_seccao() {
    let mut sim = SimWorld::default();
    let bits = sim.world_mut().spawn_empty().id().to_bits();
    let info = build_info(&sim, bits, 1, false, || {
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

/// ⭐ MI-11: a vida lê-se no cartão com o jogo PARADO; em Play o cartão diz o que a ponte
/// publicou e quem TAPA o nome (o grafo lê a primeira linha: um script vem antes do objecto).
#[test]
fn o_cartao_le_o_que_chega_ao_grafo() {
    use ph2d_editor_core::motion_input_edits::{MotionInputAoVivo as V, MotionInputOutro as O};
    let (mut sim, bits) = cena();
    {
        let mut m = sim
            .world_mut()
            .get_mut::<MotionInput>(ph2d_ecs::Entity::from_bits(bits))
            .unwrap();
        m.name = "hurt".into();
        m.source = MotionSource::Health;
    }
    com_vida(&mut sim, bits, 7.5);
    let parado = build_info(&sim, bits, 1, false, Vec::new).unwrap();
    assert_eq!(parado.ao_vivo, Some(V::Numero(0.75)));
    assert_eq!(parado.tapado_por, None);

    let out = ph2d_runtime::SignalOutbox::new();
    crate::motion_input_bridge::frame(&mut sim, &out);
    let jogo = build_info(&sim, bits, 1, true, Vec::new).unwrap();
    assert_eq!(
        jogo.ao_vivo,
        Some(V::Numero(0.75)),
        "o que a ponte publicou"
    );
    assert_eq!(jogo.tapado_por, None, "ninguém escreve antes");

    sim.world_mut()
        .resource_mut::<ph2d_ecs::MotionInputs>()
        .set(
            "hurt",
            ph2d_ecs::motion_inputs::Escritor::Script,
            ph2d_ecs::motion_inputs::Valor::Numero(0.2),
        );
    let tapado = build_info(&sim, bits, 1, true, Vec::new).unwrap();
    assert_eq!(tapado.tapado_por, Some((O::Script, V::Numero(0.2))));

    sim.world_mut()
        .get_mut::<MotionInput>(ph2d_ecs::Entity::from_bits(bits))
        .unwrap()
        .source = MotionSource::Damage;
    let dano = build_info(&sim, bits, 1, true, Vec::new).unwrap();
    assert_eq!(dano.ao_vivo, Some(V::Disparos(0)));
    assert_eq!(
        build_info(&sim, bits, 1, false, Vec::new).unwrap().ao_vivo,
        None,
        "o dano parado não tem leitura"
    );
}

/// ⭐ O nome inicial: o primeiro do grafo que outro `Motion Input` não usa; o de fábrica só sai
/// se ninguém o escreveu; sem grafo publicado fica «input».
#[test]
fn o_nome_inicial_vem_do_grafo() {
    let (mut sim, bits) = cena();
    super::semeia_o_nome(&mut sim, bits);
    assert_eq!(
        comp(&sim, bits).name,
        "input",
        "sem nomes publicados nada muda"
    );
    sim.world_mut()
        .insert_resource(ph2d_ecs::MotionGraphNames(vec![
            "hurt".into(),
            "speed".into(),
        ]));
    let outro = sim
        .world_mut()
        .spawn(MotionInput {
            name: "hurt".into(),
            ..MotionInput::default()
        })
        .id();
    super::semeia_o_nome(&mut sim, bits);
    assert_eq!(comp(&sim, bits).name, "speed", "o primeiro livre");
    sim.world_mut().get_mut::<MotionInput>(outro).unwrap().name = "speed2".into();
    super::semeia_o_nome(&mut sim, bits);
    assert_eq!(
        comp(&sim, bits).name,
        "speed",
        "o que já não é de fábrica não se toca"
    );
}

/// ⭐ A semente corre pela PORTA de anexar (a tabela própria da família), sem tabela injectada.
#[test]
fn anexar_pela_porta_da_o_nome_do_grafo() {
    let mut sim = SimWorld::default();
    sim.world_mut()
        .insert_resource(ph2d_ecs::MotionGraphNames(vec!["wind".into()]));
    let bits = sim.world_mut().spawn_empty().id().to_bits();
    let reg = crate::component_registry_for_tests::registo();
    crate::component_attach::attach_by_name(&mut sim, &reg, &[], bits, "ph2d::ecs::MotionInput")
        .expect("anexa");
    assert_eq!(comp(&sim, bits).name, "wind");
}
