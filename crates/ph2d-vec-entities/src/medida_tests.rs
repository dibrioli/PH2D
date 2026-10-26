use ph2d_ecs::canal::familia::CAD;
use ph2d_vec_scene::cad::{RegraCad, SegCad};
use ph2d_vec_scene::{VecVertex, VertId};

use super::*;

/// Um L: H no 1–2, V no 2–3, 1 fixo e a altura 2–3 medida (`40`); devolve a cena, o path e a medida.
fn cena() -> (SimWorld, VecScene, VecPathId, RegraId, u64) {
    let mut scene = VecScene::new();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [30.0, 0.0], [30.0, 40.0]]
            .into_iter()
            .map(VecVertex::corner)
            .collect(),
        ..VecPath::default()
    });
    let v = |i| PontoCad::Vert(id, VertId(i));
    *scene.regras_mut() = vec![
        RegraCad::nova(Lei::Fixo(v(1), [0.0, 0.0])),
        RegraCad::nova(Lei::Horizontal(SegCad(v(1), v(2)))),
        RegraCad::nova(Lei::Vertical(SegCad(v(2), v(3)))),
        RegraCad::nova(Lei::Distancia(v(2), v(3), 40.0)),
    ];
    scene.normaliza_ids();
    let medida = scene.regras()[3].id;
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn(VecPathRef(id)).id().to_bits();
    (sim, scene, id, medida, e)
}

fn y3(scene: &VecScene, id: VecPathId) -> f64 {
    scene.path(id).unwrap().verts[2].anchor[1]
}

#[test]
fn the_document_publishes_each_dimension_on_the_path_that_owns_it() {
    let (_, scene, id, medida, _) = cena();
    assert_eq!(campos(&scene, id), vec![(chave(MEDIDA, medida.0), 40.0)]);
    assert_eq!(chave(MEDIDA, medida.0).familia, CAD);
}

/// O undo fotografa o AUTORADO (o desenho e a medida de antes da curva) e devolve o vivo; uma edição do artista
/// por cima do vivo passa a ser o autorado.
#[test]
fn the_capture_sees_the_authored_drawing_and_an_artist_edit_becomes_it() {
    let (mut sim, mut scene, id, medida, e) = cena();
    let mut cond = ConducaoCad::default();
    let k = chave(MEDIDA, medida.0);
    assert!(aplicar(&mut sim, &mut scene, vec![(e, k, 70.0)], &mut cond));
    cond.settle();
    assert!((y3(&scene, id) - 70.0).abs() < 1e-6, "a curva estica");
    let vivo_antes = scene.clone();
    let vivo = cond.substituir(&mut scene);
    assert_eq!(y3(&scene, id), 40.0);
    assert_eq!(scene.regra(medida).and_then(|r| r.lei.medida()), Some(40.0));
    ConducaoCad::repor(&mut scene, vivo);
    assert_eq!(scene, vivo_antes, "repor devolve o vivo ao bit");
    assert!(
        !aplicar(&mut sim, &mut scene, vec![(e, k, 70.0)], &mut cond),
        "parado: nada a refazer"
    );
    // O artista move o canto 2 (a cena deixou de ser o que o solver escreveu): é o autorado novo.
    scene.path_mut(id).unwrap().verts[1].anchor[0] = 50.0;
    scene.path_mut(id).unwrap().verts[2].anchor[0] = 50.0;
    assert!(aplicar(&mut sim, &mut scene, vec![(e, k, 60.0)], &mut cond));
    assert_eq!(cond.path(id).unwrap().verts[1].anchor[0], 50.0);
    // Um quadro sem a curva: o que se vê passa a ser documento.
    cond.settle();
    cond.settle();
    assert!(cond.is_empty());
}

/// ⭐ O ângulo tem o SEU campo (`canal_cad::ANGULO`): a timeline sabe que o número são graus, e não um comprimento
/// em metros. Uma `Lei::Angulo` publica em `ANGULO` e uma escrita nesse canal chega à regra; a distância fica em
/// `MEDIDA`; e uma escrita no campo ERRADO não é dela.
#[test]
fn an_angle_measure_rides_its_own_field() {
    use ph2d_ecs::canal_cad::ANGULO;
    let (mut sim, mut scene, id, distancia, e) = cena();
    let v = |i| PontoCad::Vert(id, VertId(i));
    scene.regras_mut().push(RegraCad::nova(Lei::Angulo(
        SegCad(v(1), v(2)),
        SegCad(v(2), v(3)),
        std::f64::consts::FRAC_PI_2,
    )));
    scene.normaliza_ids();
    let angulo = scene.regras()[4].id;
    let lei = |r: RegraId| scene.regra(r).unwrap().lei.clone();
    assert_eq!(chave_da(&lei(angulo), angulo), Some(chave(ANGULO, angulo.0)));
    assert_eq!(chave_da(&lei(distancia), distancia), Some(chave(MEDIDA, distancia.0)));
    assert_eq!(chave_da(&scene.regras()[0].lei.clone(), scene.regras()[0].id), None);
    let publicadas = campos(&scene, id);
    assert!(publicadas.contains(&(chave(ANGULO, angulo.0), 90.0)), "{publicadas:?}");
    assert!(publicadas.contains(&(chave(MEDIDA, distancia.0), 40.0)), "{publicadas:?}");
    assert!(
        !aplicar(&mut sim, &mut scene, vec![(e, chave(MEDIDA, angulo.0), 30.0)], &mut ConducaoCad::default()),
        "uma escrita em MEDIDA não é de um ângulo"
    );
    let mut cond = ConducaoCad::default();
    aplicar(&mut sim, &mut scene, vec![(e, chave(ANGULO, angulo.0), 60.0)], &mut cond);
    let m = scene.regra(angulo).and_then(|r| r.lei.medida()).unwrap();
    assert!((m - 60f64.to_radians()).abs() < 1e-9, "a escrita em ANGULO chega à regra: {m}");
}
