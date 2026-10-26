//! As edições da secção DRAWING RULES sobre o documento: a medida escrita re-resolve a folha (toda regra continua
//! cumprida, e a medida tirada dos cantos é a escrita) e apagar tira exactamente aquela regra.

use super::*;
use ph2d_vec_constraints::violacao;
use ph2d_vec_scene::cad::SegCad;
use ph2d_vec_scene::{VecPath, VecVertex, VertId, Xform};

/// Um L de três cantos: o 1.º preso na origem, o 1.º lado horizontal, o 2.º vertical com 40 px. O path tem um afim
/// que escala por 2 e desloca — as medidas são da FOLHA, não da geometria local.
fn cena() -> (VecScene, VecXforms, VecPathId) {
    let mut scene = VecScene::new();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [15.0, 0.0], [15.0, 20.0]]
            .into_iter()
            .map(VecVertex::corner)
            .collect(),
        ..VecPath::default()
    });
    let v = |i| PontoCad::Vert(id, VertId(i));
    *scene.regras_mut() = vec![
        RegraCad::nova(Lei::Fixo(v(1), [10.0, 5.0])),
        RegraCad::nova(Lei::Horizontal(SegCad(v(1), v(2)))),
        RegraCad::nova(Lei::Vertical(SegCad(v(2), v(3)))),
        RegraCad::nova(Lei::Distancia(v(2), v(3), 40.0)),
    ];
    scene.normaliza_ids();
    let xf: VecXforms = [(id, Xform([2.0, 0.0, 0.0, 2.0, 10.0, 5.0]))].into();
    (scene, xf, id)
}

fn folha<'a>(scene: &'a VecScene, xf: &VecXforms, id: VecPathId) -> Folha<'a> {
    Folha::new(scene.path(id).map(|p| (p, xform_of(xf, id))))
}

/// ⭐⭐ Escrever `55` na distância: a folha re-resolve, toda regra cumpre-se e os cantos medem `55`.
///
/// **Mutações que devem sangrar:** não re-resolver (os cantos ficam a `40`) · resolver sem o afim (a medida
/// sai na escala local) · escrever na regra errada.
#[test]
fn escrever_a_medida_re_resolve_a_folha_a_partir_do_desenho() {
    let (mut scene, xf, id) = cena();
    let dist = scene.regras()[3].id.0;
    assert!(aplica(&mut scene, &xf, CadFieldEdit::Medida(dist, 55.0)));
    assert_eq!(scene.regras()[3].lei.medida(), Some(55.0));
    let f = folha(&scene, &xf, id);
    let pior = violacao(&f, scene.regras());
    assert!(pior <= 1e-6, "uma regra ficou por cumprir: {pior}");
    let canto = |i| {
        let p = scene
            .path(id)
            .and_then(|p| p.vert_by_id(VertId(i)))
            .expect("o canto");
        xform_of(&xf, id).apply(p.anchor)
    };
    let (a, b) = (canto(2), canto(3));
    let medida = (b[0] - a[0]).hypot(b[1] - a[1]);
    assert!((medida - 55.0).abs() <= 1e-6, "os cantos medem {medida}");
}

/// ⭐ Uma REF não se escreve: só mostra a medida.
#[test]
fn uma_ref_nao_se_escreve() {
    let (mut scene, xf, _) = cena();
    scene.regras_mut()[3].referencia = true;
    let dist = scene.regras()[3].id.0;
    assert!(!aplica(&mut scene, &xf, CadFieldEdit::Medida(dist, 55.0)));
    assert_eq!(scene.regras()[3].lei.medida(), Some(40.0));
}

/// ⭐⭐ Apagar tira EXACTAMENTE a regra pedida — as outras ficam, com os mesmos ids e pela mesma ordem.
#[test]
fn apagar_tira_exactamente_aquela_regra() {
    let (mut scene, xf, _) = cena();
    let ids: Vec<RegraId> = scene.regras().iter().map(|r| r.id).collect();
    assert!(aplica(&mut scene, &xf, CadFieldEdit::Apagar(ids[1].0)));
    let depois: Vec<RegraId> = scene.regras().iter().map(|r| r.id).collect();
    assert_eq!(depois, vec![ids[0], ids[2], ids[3]]);
    assert!(
        !aplica(&mut scene, &xf, CadFieldEdit::Apagar(ids[1].0)),
        "a 2.ª vez não há o que apagar"
    );
}

/// ⭐ O instantâneo leva só as regras cujo DONO é o path da entidade, com o canal da medida de cada uma.
#[test]
fn o_instantaneo_leva_as_regras_do_dono_com_o_canal() {
    let (mut scene, xf, id) = cena();
    let outro = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [5.0, 0.0]]
            .into_iter()
            .map(VecVertex::corner)
            .collect(),
        ..VecPath::default()
    });
    scene.normaliza_ids();
    let w = |i| PontoCad::Vert(outro, VertId(i));
    scene
        .regras_mut()
        .push(RegraCad::nova(Lei::Horizontal(SegCad(w(1), w(2)))));
    scene.normaliza_ids();
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn(VecPathRef(id)).id().to_bits();
    let cores = CadCores::default();
    let i = info(&sim, &scene, &cores, &xf, e).expect("o path é dono de regras");
    assert_eq!(i.entity_bits, e);
    assert_eq!(i.rows.len(), 4, "a regra do OUTRO path não é desta secção");
    let d = &i.rows[3];
    assert_eq!(d.nome, "app.vec.cad.lei.distancia");
    assert_eq!(d.medida, Some(40.0));
    let k = chave(MEDIDA, d.id);
    assert_eq!(d.canal, [k.familia, k.campo, k.i, k.j]);
    assert_eq!(i.rows[1].medida, None, "a horizontal não tem medida");
    let solta = sim.world_mut().spawn(()).id().to_bits();
    assert!(info(&sim, &scene, &cores, &xf, solta).is_none());
}
