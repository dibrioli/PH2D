//! O arranjo do HUD (`UiArranjo`, ADR-0194): grava e abre; a porta da ferramenta troca e recusa.

use ph2d_ecs::hud_arranjo::{Eixo, Forca, Lado, Relacao, UiArranjo};
use ph2d_ecs::{SimWorld, StableId, Transform, UiCanvas};

const PONT: StableId = StableId(11);
const MOEDA: StableId = StableId(12);
const MAPA: StableId = StableId(13);

fn ao_lado(a: StableId, b: StableId, lado: Lado) -> Relacao<StableId> {
    Relacao::AoLado {
        a,
        b,
        lado,
        folga: 0.25,
    }
}

#[test]
fn a_hud_arrangement_survives_save_and_open() {
    use ph2d_ecs::scene::{ComponentRegistry, WorldSnapshot, snapshot_to_world, world_to_snapshot};
    use ph2d_ecs::{TransformPropagationState, WorklistBuf};
    let mut reg = ComponentRegistry::new();
    ph2d_ecs::scene::register_ecs_components(&mut reg);
    let mut sim = SimWorld::new();
    let mut arr = UiArranjo::default();
    assert!(arr.junta(Relacao::LarguraDoConteudo { a: PONT }, Forca::Media));
    assert!(arr.junta(ao_lado(MOEDA, PONT, Lado::Direita), Forca::Requerida));
    assert!(arr.junta(
        Relacao::Centrado {
            a: MOEDA,
            b: PONT,
            eixo: Eixo::Y
        },
        Forca::Forte
    ));
    sim.world_mut().spawn((
        StableId(10),
        Transform::IDENTITY,
        UiCanvas::default(),
        arr.clone(),
    ));
    let mut snap = WorldSnapshot::new();
    let (mut st, mut wl) = (
        TransformPropagationState::new(sim.world_mut()),
        WorklistBuf::default(),
    );
    world_to_snapshot(sim.world_mut(), &mut st, &mut wl, &reg, &mut snap).unwrap();
    let lido: WorldSnapshot = postcard::from_bytes(&postcard::to_allocvec(&snap).unwrap()).unwrap();
    let mut outro = SimWorld::new();
    snapshot_to_world(outro.world_mut(), &lido, &reg).unwrap();
    let abertos: Vec<UiArranjo> = outro
        .world_mut()
        .query::<&UiArranjo>()
        .iter(outro.world())
        .cloned()
        .collect();
    assert_eq!(abertos, vec![arr]);
}

#[test]
fn a_new_arrow_between_the_same_pair_replaces_the_old_on_that_axis() {
    let mut arr = UiArranjo::default();
    assert!(arr.junta(ao_lado(MOEDA, PONT, Lado::Direita), Forca::Requerida));
    assert!(arr.junta(
        Relacao::Centrado {
            a: PONT,
            b: MOEDA,
            eixo: Eixo::Y
        },
        Forca::Forte
    ));
    // Outra seta entre os dois, no X: troca a de X, a do Y fica.
    assert!(arr.junta(
        Relacao::NaoInvade {
            a: PONT,
            b: MOEDA,
            lado: Lado::Direita,
            folga: 0.5
        },
        Forca::Requerida
    ));
    assert_eq!(arr.lacos.len(), 2);
    assert!(matches!(arr.lacos[0].0, Relacao::Centrado { .. }));
    assert!(matches!(arr.lacos[1].0, Relacao::NaoInvade { .. }));
}

#[test]
fn an_arrow_that_closes_a_circle_is_refused_and_changes_nothing() {
    let mut arr = UiArranjo::default();
    assert!(arr.junta(ao_lado(MOEDA, PONT, Lado::Direita), Forca::Requerida));
    assert!(arr.junta(ao_lado(MAPA, MOEDA, Lado::Direita), Forca::Requerida));
    let antes = arr.clone();
    // O minimapa à ESQUERDA da pontuação: pontuação < moeda < mapa < pontuação.
    assert!(!arr.junta(ao_lado(MAPA, PONT, Lado::Esquerda), Forca::Requerida));
    assert_eq!(arr, antes);
    assert!(arr.apaga(0) && !arr.apaga(9));
    assert!(
        arr.junta(ao_lado(MAPA, PONT, Lado::Esquerda), Forca::Requerida),
        "sem a primeira já não há círculo"
    );
}

#[test]
fn a_group_of_more_than_the_ceiling_is_refused() {
    use ph2d_ecs::hud_arranjo::TECTO_GRUPO;
    let mut arr = UiArranjo::default();
    let id = |i: usize| StableId(100 + i as u64);
    for i in 1..TECTO_GRUPO {
        assert!(
            arr.junta(ao_lado(id(i), id(i - 1), Lado::Direita), Forca::Requerida),
            "{i}"
        );
    }
    let antes = arr.clone();
    assert!(!arr.junta(
        ao_lado(id(TECTO_GRUPO), id(TECTO_GRUPO - 1), Lado::Direita),
        Forca::Requerida
    ));
    assert_eq!(arr, antes, "o 65.º elemento não entra e nada muda");
    assert!(
        arr.junta(
            ao_lado(StableId(1), StableId(2), Lado::Direita),
            Forca::Requerida
        ),
        "outro grupo entra"
    );
}
