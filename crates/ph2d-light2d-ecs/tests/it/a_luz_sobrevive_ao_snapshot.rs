//! Undo e save passam pelo `WorldSnapshot`: um componente fora do registo é descartado em
//! silêncio. A luz tem de sair e voltar inteira.

use ph2d_ecs::scene::{
    ComponentRegistry, WorldSnapshot, register_ecs_components, snapshot_to_world, world_to_snapshot,
};
use ph2d_ecs::{Name, SimWorld, Transform, TransformPropagationState, WorklistBuf};
use ph2d_light2d_ecs::{
    BloqueiaLuz, Luz, NOMES_CANONICOS, NaoRecebeLuz, TipoDeLuz, register_light2d_components,
};

fn registo() -> ComponentRegistry {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    register_light2d_components(&mut reg);
    reg
}

#[test]
fn a_porta_regista_os_tres_nomes() {
    let mut reg = ComponentRegistry::new();
    register_light2d_components(&mut reg);
    let mut nomes: Vec<&str> = reg.iter().map(|e| e.canonical_name).collect();
    nomes.sort_unstable();
    assert_eq!(nomes, NOMES_CANONICOS);
}

#[test]
fn a_luz_sobrevive_ao_snapshot_e_restore() {
    let reg = registo();
    let luz = Luz {
        cor: [0.9, 0.7, 0.2],
        intensidade: 3.5,
        tamanho_m: 0.25,
        alcance_m: f32::INFINITY,
        tipo: TipoDeLuz::Ponto,
    };
    let ceu = Luz {
        alcance_m: 12.0,
        tipo: TipoDeLuz::Ambiente,
        ..luz
    };
    let bloqueia = BloqueiaLuz { opacidade: 0.4 };

    let mut a = SimWorld::new();
    a.world_mut()
        .spawn((Name::new("lampada"), Transform::IDENTITY, luz));
    a.world_mut()
        .spawn((Name::new("ceu"), Transform::IDENTITY, ceu));
    a.world_mut().spawn((
        Name::new("parede"),
        Transform::IDENTITY,
        bloqueia,
        NaoRecebeLuz,
    ));

    let mut state = TransformPropagationState::new(a.world_mut());
    let mut worklist = WorklistBuf::new();
    let mut snap = WorldSnapshot::new();
    world_to_snapshot(a.world_mut(), &mut state, &mut worklist, &reg, &mut snap).unwrap();

    let mut b = SimWorld::new();
    let back = snapshot_to_world(b.world_mut(), &snap, &reg).unwrap();
    let achar = |nome: &str| {
        back.iter()
            .copied()
            .find(|&e| b.world().get::<Name>(e).is_some_and(|n| n.as_str() == nome))
            .unwrap_or_else(|| panic!("{nome} não voltou"))
    };
    assert_eq!(b.world().get::<Luz>(achar("lampada")), Some(&luz));
    assert_eq!(b.world().get::<Luz>(achar("ceu")), Some(&ceu));
    let parede = achar("parede");
    assert_eq!(b.world().get::<BloqueiaLuz>(parede), Some(&bloqueia));
    assert!(b.world().get::<NaoRecebeLuz>(parede).is_some());
}
