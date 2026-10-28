//! O arranjo vivo sobre um mundo de verdade: canvas, filhos com `StableId`, as caixas que as âncoras deixaram.

use super::{Arranjos, Caixa2, Movido};
use crate::hud_bridge::View;
use ph2d_ecs::hud_arranjo::{Forca, Lado, Relacao, UiArranjo};
use ph2d_ecs::{ChildOf, Entity, Fit, SimWorld, StableId, Transform, UiCanvas};
use ph2d_vec_entities::entities::VecEntityMap;
use ph2d_vec_scene::{VecScene, rectangle};

const PONT: StableId = StableId(11);
const MOEDA: StableId = StableId(12);
const MAPA: StableId = StableId(13);

/// Canvas `32 × 18` na origem, escala 1; três filhos (pontuação, moeda, minimapa).
fn cena(lacos: Vec<(Relacao<StableId>, Forca)>) -> (SimWorld, VecEntityMap, [Entity; 3]) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    for _ in 0..3 {
        scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    }
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
    let canvas = sim
        .world_mut()
        .spawn((
            StableId(10),
            Transform::IDENTITY,
            UiCanvas {
                ref_w: 32.0,
                ref_h: 18.0,
                fit: Fit::Expand,
            },
            UiArranjo { lacos },
        ))
        .id();
    let mut kids: Vec<Entity> = map.values().map(|&b| Entity::from_bits(b)).collect();
    kids.sort_unstable();
    for (k, id) in kids.iter().zip([PONT, MOEDA, MAPA]) {
        sim.world_mut().entity_mut(*k).insert((ChildOf(canvas), id));
    }
    (sim, map, [kids[0], kids[1], kids[2]])
}

fn moeda_ao_lado() -> Vec<(Relacao<StableId>, Forca)> {
    vec![
        (Relacao::LarguraDoConteudo { a: PONT }, Forca::Media),
        (
            Relacao::AoLado {
                a: MOEDA,
                b: PONT,
                lado: Lado::Direita,
                folga: 0.5,
            },
            Forca::Requerida,
        ),
        (
            Relacao::NaoInvade {
                a: PONT,
                b: MAPA,
                lado: Lado::Direita,
                folga: 0.5,
            },
            Forca::Requerida,
        ),
    ]
}

/// A pontuação no canto de cima à esquerda com `largura` de texto; a moeda onde o artista a pôs; o minimapa
/// colado à direita da vista (`meia` = meia largura da vista), como a âncora o deixa.
fn caixas(largura: f64, meia: f64) -> impl Fn(usize) -> Caixa2 {
    move |i| match i {
        0 => ([-15.0, 7.0], [-15.0 + largura, 8.5]),
        1 => ([-6.0, 7.25], [-5.0, 8.25]),
        _ => ([meia - 5.0, 4.0], [meia - 1.0, 8.5]),
    }
}

fn quadro(
    arr: &mut Arranjos,
    sim: &SimWorld,
    map: &VecEntityMap,
    kids: [Entity; 3],
    meia: f64,
    largura: f64,
) -> Vec<Movido> {
    let c = caixas(largura, meia);
    let vista = View {
        center: [0.0, 0.0],
        half: [meia as f32, 9.0],
    };
    arr.arranja(sim, map, Some(vista), |k| {
        kids.iter().position(|&x| x == k).map(&c)
    })
}

fn onde(mov: &[Movido], k: Entity, sem: Caixa2) -> Caixa2 {
    mov.iter().find(|m| m.kid == k).map_or(sem, |m| m.para)
}

#[test]
fn the_coin_stays_beside_a_growing_score() {
    let (sim, map, kids) = cena(moeda_ao_lado());
    let mut arr = Arranjos::default();
    for largura in [2.0, 3.5, 5.25, 8.0] {
        let mov = quadro(&mut arr, &sim, &map, kids, 16.0, largura);
        let pont = onde(&mov, kids[0], caixas(largura, 16.0)(0));
        let moeda = onde(&mov, kids[1], caixas(largura, 16.0)(1));
        assert_eq!(
            pont,
            caixas(largura, 16.0)(0),
            "texto {largura}: a pontuação não se mexe"
        );
        assert_eq!(
            moeda.0[0],
            pont.1[0] + 0.5,
            "texto {largura}: a moeda colada com a folga"
        );
        assert_eq!(
            (moeda.0[1], moeda.1[1] - moeda.0[1]),
            (7.25, 1.0),
            "a moeda só anda em x"
        );
    }
}

#[test]
fn a_narrow_screen_shrinks_the_score_before_it_overlaps() {
    let (sim, map, kids) = cena(moeda_ao_lado());
    let mut arr = Arranjos::default();
    let mut encolheu = false;
    // Expand: a caixa só estreita quando a janela, vinda de LARGA, volta ao formato de referência (meia = 16).
    for meia in [24.0, 20.0, 16.0] {
        let mov = quadro(&mut arr, &sim, &map, kids, meia, 27.0);
        let c = caixas(27.0, meia);
        let (pont, mapa) = (onde(&mov, kids[0], c(0)), onde(&mov, kids[2], c(2)));
        assert!(
            pont.1[0] + 0.5 <= mapa.0[0] + 1e-9,
            "vista {meia}: invadiu o minimapa: {mov:?}"
        );
        assert_eq!(mapa, c(2), "vista {meia}: o minimapa é a âncora, não cede");
        let (w, h) = (pont.1[0] - pont.0[0], pont.1[1] - pont.0[1]);
        assert!(
            (h / 1.5 - w / 27.0).abs() < 1e-9,
            "vista {meia}: o texto encolhe POR IGUAL ({w} × {h})"
        );
        encolheu |= w < 27.0;
    }
    assert!(encolheu, "a vista mais estreita tinha de encolher o número");
}

#[test]
fn the_arrangement_is_the_same_bit_every_frame() {
    let (sim, map, kids) = cena(moeda_ao_lado());
    let mut arr = Arranjos::default();
    let primeiro = quadro(&mut arr, &sim, &map, kids, 8.0, 16.0);
    assert!(!primeiro.is_empty());
    for meia in [16.0, 8.0, 12.0, 8.0] {
        let q = quadro(&mut arr, &sim, &map, kids, meia, 16.0);
        if meia == 8.0 {
            assert_eq!(q, primeiro, "ida e volta mudou");
        }
    }
    assert_eq!(
        quadro(&mut Arranjos::default(), &sim, &map, kids, 8.0, 16.0),
        primeiro,
        "outro solver"
    );
}

#[test]
fn nothing_moves_without_an_arrangement_or_when_it_is_already_satisfied() {
    let (mut sim, map, kids) = cena(Vec::new());
    let mut arr = Arranjos::default();
    assert!(
        quadro(&mut arr, &sim, &map, kids, 16.0, 4.0).is_empty(),
        "sem relações"
    );
    // A moeda já está colada: nada a publicar.
    let lacos = vec![(
        Relacao::AoLado {
            a: MOEDA,
            b: PONT,
            lado: Lado::Direita,
            folga: 5.0,
        },
        Forca::Requerida,
    )];
    let canvas = sim.world().get::<ChildOf>(kids[0]).unwrap().parent();
    sim.world_mut()
        .entity_mut(canvas)
        .insert(UiArranjo { lacos });
    assert!(
        quadro(&mut arr, &sim, &map, kids, 16.0, 4.0).is_empty(),
        "já satisfeita"
    );
    // Uma relação para um irmão que não existe é inerte.
    let lacos = vec![(
        Relacao::AoLado {
            a: MOEDA,
            b: StableId(99),
            lado: Lado::Direita,
            folga: 5.0,
        },
        Forca::Requerida,
    )];
    sim.world_mut()
        .entity_mut(canvas)
        .insert(UiArranjo { lacos });
    assert!(
        quadro(&mut arr, &sim, &map, kids, 16.0, 2.0).is_empty(),
        "irmão que não existe"
    );
}
