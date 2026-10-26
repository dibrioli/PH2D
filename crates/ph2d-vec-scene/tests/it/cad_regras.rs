//! ⭐ **As regras de CAD viajam no documento do vetor e morrem com o ponto que citam** (W4.4,
//! `docs/BlenderConstraints/04_W4_cad.md` §3): `VecScene.regras` no fim do degrau 24; apagar um vértice ou um
//! path tira as regras que o citam no MESMO assentamento que dá os ids (o que corre antes de cada passo de undo).

use ph2d_vec_scene::cad::{Lei, PontoCad, RegraCad, SegCad};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecVertex, VertId, ellipse};

fn cena() -> (VecScene, VecPathId, VecPathId) {
    let mut c = VecScene::new();
    let a = c.push_path(VecPath {
        verts: (0..4)
            .map(|i| VecVertex::corner([10.0 * f64::from(i), 0.0]))
            .collect(),
        ..VecPath::default()
    });
    let b = c.push_path(ellipse([50.0, 50.0], 8.0, 8.0));
    (c, a, b)
}

fn v(p: VecPathId, i: u32) -> PontoCad {
    PontoCad::Vert(p, VertId(i))
}

fn regras(a: VecPathId, b: VecPathId) -> Vec<RegraCad> {
    vec![
        RegraCad::nova(Lei::Horizontal(SegCad(v(a, 1), v(a, 2)))),
        RegraCad::nova(Lei::Distancia(v(a, 3), PontoCad::Centro(b), 40.0)),
        RegraCad {
            lei: Lei::Raio(b, 8.0),
            referencia: true,
        },
        RegraCad::nova(Lei::Tangente(SegCad(v(a, 3), v(a, 4)), b)),
        RegraCad::nova(Lei::Fixo(v(a, 1), [0.0, 0.0])),
    ]
}

#[test]
fn a_scene_with_rules_round_trips() {
    let (mut c, a, b) = cena();
    *c.regras_mut() = regras(a, b);
    let volta = VecScene::from_bytes(&c.to_bytes().unwrap()).unwrap();
    assert_eq!(volta.regras(), regras(a, b).as_slice());
    assert_eq!(volta, c);
}

#[test]
fn deleting_a_vertex_drops_the_rules_that_cite_it_when_ids_settle() {
    let (mut c, a, b) = cena();
    *c.regras_mut() = regras(a, b);
    let antes = c.clone();
    c.path_mut(a).unwrap().verts.remove(1); // o vértice de id 2
    assert!(
        c.normaliza_ids_desde(Some(&antes)),
        "tirar regras é mudança"
    );
    let r = regras(a, b);
    assert_eq!(c.regras(), &[r[1], r[2], r[3], r[4]]);
    assert!(!c.normaliza_ids_desde(Some(&antes)), "idempotente");
}

#[test]
fn deleting_a_path_drops_every_rule_that_cites_it() {
    let (mut c, a, b) = cena();
    *c.regras_mut() = regras(a, b);
    c.remove_path(b);
    c.normaliza_ids_desde(None);
    let r = regras(a, b);
    assert_eq!(c.regras(), &[r[0], r[4]]);
}
