//! Os gates da [`super`] — a família COMPONENTE de ponta a ponta sobre a câmera.

use super::*;
use crate::component_registry_for_tests::registo;
use ph2d_ecs::GameCamera;
use ph2d_ecs::canal_componente::chave;

const CAMERA: &str = "ph2d::ecs::GameCamera";

fn cena() -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn(GameCamera::default()).id();
    (sim, e)
}

fn k(field_id: u16, faixa: u16) -> FieldKey {
    chave(CAMERA, field_id, faixa).expect("cabe")
}

/// A câmera tem os seus seis campos como canais: altura, offset x/y, prioridade, activa, máscara,
/// dolly — e nada de outro componente.
#[test]
fn a_camera_publica_os_seus_campos() {
    let reg = registo();
    let (sim, e) = cena();
    let todos = campos(&reg, sim.world(), e);
    let quer = [
        k(0, 0),
        k(1, 0),
        k(1, 1),
        k(2, 0),
        k(3, 0),
        k(4, 0),
        k(5, 0),
    ];
    for q in quer {
        assert!(todos.contains(&q), "falta {q:?} em {todos:?}");
    }
    assert_eq!(todos.len(), quer.len(), "{todos:?}");
}

/// ⭐ **Cada campo lê o que se escreveu**, pela forma do valor guardado (real, faixa, inteiro,
/// booleano), e a escrita devolve o antes/depois para a pré-visualização.
#[test]
fn cada_campo_le_o_que_escreveu() {
    let reg = registo();
    let (mut sim, e) = cena();
    for (kk, x) in [
        (k(0, 0), 12.5),
        (k(1, 1), -3.0),
        (k(2, 0), 4.0),
        (k(3, 0), 0.0),
        (k(5, 0), 0.25),
    ] {
        let esc = escrever(&reg, sim.world_mut(), e, kk, x).expect("mudou");
        assert_ne!(esc.antes, esc.depois);
        assert_eq!(ler(&reg, sim.world(), e, kk), Some(x), "{kk:?}");
    }
    let c = sim.world().get::<GameCamera>(e).expect("câmera");
    assert_eq!(
        (c.height_world, c.offset[1], c.priority, c.active),
        (12.5, -3.0, 4, false)
    );
    assert!(
        escrever(&reg, sim.world_mut(), e, k(0, 0), 12.5).is_none(),
        "o mesmo valor não é escrita"
    );
}

/// ⛔ **A lei do componente vale para a curva** (`sanitize_camera`): uma altura impossível não chega
/// ao componente como foi escrita, e o eco diz o que ficou.
#[test]
fn a_lei_da_camera_vale_para_a_curva() {
    let reg = registo();
    let (mut sim, e) = cena();
    sim.world_mut()
        .get_resource_or_insert_with(CanaisEscritos::default)
        .escrever(e.to_bits(), k(0, 0), -5.0);
    let mut drive = PreviewDrive::default();
    assert!(aplicar(&mut sim, &reg, &mut drive));
    let h = sim
        .world()
        .get::<GameCamera>(e)
        .expect("câmera")
        .height_world;
    assert!(h > 0.0, "a altura negativa passou: {h}");
}

/// ⭐⭐ **A curva chega ao componente e NÃO ao documento**: a captura repõe o autorado.
#[test]
fn a_curva_chega_ao_componente_e_nao_ao_documento() {
    let reg = registo();
    let (mut sim, e) = cena();
    let antes = sim
        .world()
        .get::<GameCamera>(e)
        .expect("câmera")
        .height_world;
    sim.world_mut()
        .get_resource_or_insert_with(CanaisEscritos::default)
        .escrever(e.to_bits(), k(0, 0), antes + 7.0);
    let mut drive = PreviewDrive::default();
    assert!(aplicar(&mut sim, &reg, &mut drive));
    assert_eq!(
        sim.world()
            .get::<GameCamera>(e)
            .expect("câmera")
            .height_world,
        antes + 7.0
    );
    let vivo = drive.substitute(&mut sim);
    assert_eq!(
        sim.world()
            .get::<GameCamera>(e)
            .expect("câmera")
            .height_world,
        antes,
        "a captura não repôs o autorado"
    );
    PreviewDrive::restore(&mut sim, &vivo);
    publicar(&mut sim, &reg, &[e.to_bits()]);
    let l = sim.world().resource::<CanaisLidos>();
    assert_eq!(l.valor(e.to_bits(), k(0, 0)), Some(antes + 7.0));
}
