//! Gates do que se VÊ de uma solda — módulo irmão do [`super::tests`] pelo tecto de LOC (HR-18), e
//! o corte é por RESPONSABILIDADE: aquele mede *o que a solda FAZ à cena* (arcos, ids, poses, nós);
//! este mede *o que o artista vê* — a marca do nó partilhado e a tinta do traço.

use super::*;
use ph2d_vec_scene::{Rgba8, StrokeSpec, VertexKind};

fn v(x: f64, y: f64) -> VecVertex {
    VecVertex {
        anchor: [x, y],
        in_handle: [x, y],
        out_handle: [x, y],
        kind: VertexKind::Corner,
        corner_radius: 0.0,
    }
}

fn reta(scene: &mut VecScene, a: [f64; 2], b: [f64; 2]) -> u64 {
    scene.push_path(VecPath {
        verts: vec![v(a[0], a[1]), v(b[0], b[1])],
        closed: false,
        stroke: Some(StrokeSpec::new(Rgba8::new(9, 9, 9, 255), 2.0)),
        ..VecPath::default()
    })
}

/// Todas as PONTAS da cena — a mesma sonda do irmão, e pela mesma razão: uma que lesse `p.verts`
/// mediria só o primeiro arco de uma rede composta.
fn pontas(scene: &VecScene) -> Vec<[f64; 2]> {
    let mut out = Vec::new();
    for p in scene.paths() {
        for c in 0..p.contour_count() {
            let Some((verts, closed)) = p.contour(c) else {
                continue;
            };
            if closed || verts.len() < 2 {
                continue;
            }
            out.push(verts[0].anchor);
            out.push(verts[verts.len() - 1].anchor);
        }
    }
    out
}

fn cena() -> (VecScene, ph2d_vec_edit::History, ph2d_vec_edit::PenTool) {
    (
        VecScene::new(),
        ph2d_vec_edit::History::default(),
        ph2d_vec_edit::PenTool::default(),
    )
}

/// ⭐⭐⭐ **A MARCA e o ARRASTO respondem à MESMA pergunta** — nos DOIS sentidos.
///
/// ⚠️ **Este é o gate que impede a marca de ensinar uma lei falsa.** O anel existe porque o Enio
/// não tinha como VER se o nó era partilhado (report de 2026-09-01); se ele acendesse por uma
/// segunda régua, acenderia onde o dedo não arrasta junto — e o instrumento que existe para lhe
/// ensinar a lei ensinaria a errada.
///
/// ⛔ Um sentido só não chega: *"todo nó marcado tem juntas"* passa com uma lista vazia.
#[test]
fn the_mark_and_the_drag_answer_the_same_question() {
    let (mut scene, mut hist, mut pen) = cena();
    let a = scene.push_path(ph2d_vec_scene::ellipse([0.0, 0.0], 100.0, 100.0));
    let b = scene.push_path(ph2d_vec_scene::ellipse([120.0, 0.0], 100.0, 100.0));
    pen.select_many(&[a, b]);
    apply_vec_weld(&mut scene, &mut hist, &mut pen, &VecXforms::new(), 0.0);

    let nos = pen.welded_nodes(&scene);
    assert_eq!(
        nos.len(),
        2,
        "dois circulos soldados tem DOIS nos partilhados: {nos:?}"
    );
    // IDA: todo nó marcado tem de ter junta.
    for &(id, i) in &nos {
        assert!(
            !pen.welded_with(&scene, id, i).is_empty(),
            "a marca acendeu num ponto que o arrasto nao ve como junta ({id}, {i})"
        );
    }
    // VOLTA: toda ponta com junta tem de estar sob uma marca.
    let marcados: Vec<[f64; 2]> = nos
        .iter()
        .map(|&(id, i)| scene.path(id).unwrap().verts[i].anchor)
        .collect();
    for p in scene.paths() {
        for i in [0, p.verts.len() - 1] {
            if pen.welded_with(&scene, p.id, i).is_empty() {
                continue;
            }
            let at = p.verts[i].anchor;
            assert!(
                marcados
                    .iter()
                    .any(|m| (m[0] - at[0]).abs() < 1e-9 && (m[1] - at[1]).abs() < 1e-9),
                "o arrasto leva esta ponta junto e nada a marca: {at:?}"
            );
        }
    }
}

/// ⚠️ **Sem solda não há marca** — o anel é a leitura de um FACTO, não decoração de selecção.
#[test]
fn nothing_is_marked_before_the_weld() {
    let (mut scene, mut hist, mut pen) = cena();
    let a = reta(&mut scene, [-100.0, 0.0], [0.0, 0.0]);
    let b = reta(&mut scene, [0.3, -0.2], [100.0, 0.0]);
    assert!(
        pen.welded_nodes(&scene).is_empty(),
        "duas pontas PERTO nao sao um no' — so' duas pontas no mesmo sitio sao"
    );
    pen.select_many(&[a, b]);
    apply_vec_weld(&mut scene, &mut hist, &mut pen, &VecXforms::new(), 2.0);
    assert_eq!(
        pen.welded_nodes(&scene).len(),
        1,
        "depois de ligar, o no' tem de aparecer"
    );
}

/// ⭐⭐⭐ **UMA LINHA NOVA SOLDA-SE À PONTA DE UMA REDE QUE JÁ EXISTE.**
///
/// ⚠️⚠️ É o fluxo normal depois de 2026-09-02: a primeira solda deixa **um** objecto composto, e a
/// segunda tem de o reconhecer. A porta de entrada (`editavel_no_sitio`) recusava compostos e a
/// enumeração de pontas era `[0, verts.len() - 1]` — as duas juntas viam **só o primeiro arco**, e
/// pendurar uma linha na ponta de uma rede simplesmente não pegava.
///
/// ⛔ E nada se cruza aqui: a rede **não** se dissolve, o id dela sobrevive e só a ponta da linha
/// nova se muda.
#[test]
fn a_new_line_welds_onto_the_end_of_an_existing_network() {
    let (mut scene, mut hist, mut pen) = cena();
    let h = reta(&mut scene, [-10.0, 0.0], [10.0, 0.0]);
    let vt = reta(&mut scene, [0.0, -10.0], [0.0, 10.0]);
    pen.select_many(&[h, vt]);
    apply_vec_weld(&mut scene, &mut hist, &mut pen, &VecXforms::new(), 0.0);
    let rede = scene.paths()[0].id;
    assert!(
        scene.path(rede).is_some_and(|p| p.contour_count() == 4),
        "a fixtura tem de partir de uma rede COMPOSTA"
    );

    // Uma linha que nasce a 0,42 da ponta direita da rede (10, 0) e vai para longe.
    let nova = reta(&mut scene, [10.3, 0.3], [40.0, 30.0]);
    pen.select_many(&[rede, nova]);
    apply_vec_weld(&mut scene, &mut hist, &mut pen, &VecXforms::new(), 1.0);

    assert_eq!(
        scene.paths().len(),
        2,
        "nada se cruza: a rede e a linha continuam a ser dois objectos"
    );
    assert!(
        scene.path(rede).is_some_and(|p| p.contour_count() == 4),
        "a rede nao se dissolve por emprestar uma ponta"
    );
    assert!(
        !pen.welded_with(&scene, nova, 0).is_empty(),
        "a ponta da linha nova tem de partilhar o no' com um arco da rede"
    );
    let ponta = scene.path(nova).unwrap().verts[0].anchor;
    let na_rede = pontas(&scene)
        .into_iter()
        .filter(|p| (p[0] - ponta[0]).abs() < 1e-9 && (p[1] - ponta[1]).abs() < 1e-9)
        .count();
    assert_eq!(
        na_rede, 2,
        "a ponta da linha e a da rede sao a MESMA coordenada"
    );
}

/// ⭐⭐⭐ **A REDE SOLDADA TEM TAMPA REDONDA** — o report de 2026-09-02 com foto (*"com stroke muito
/// largo, o stroke se quebra na peça com weld"*).
///
/// ⚠️ **A causa é estrutural**: o traço é aplicado ao caminho inteiro e a kurbo põe **tampa**, nunca
/// junta, na ponta de cada sub-caminho. Cada arco da rede é um sub-caminho ⇒ **todo nó é um par de
/// tampas**, e com tampa recta sobra uma cunha vazia de `(w/2)·tan(θ/2)` — invisível num traço
/// fino, um rasgo num traço largo.
///
/// ⭐ A tampa redonda é o meio-disco de raio `w/2` na ponta: no nó, a união dos meios-discos cobre o
/// disco inteiro, e portanto a cunha, para **qualquer** ângulo.
#[test]
fn the_welded_network_is_stroked_with_round_caps() {
    let (mut scene, mut hist, mut pen) = cena();
    let h = reta(&mut scene, [-10.0, 0.0], [10.0, 0.0]);
    let vt = reta(&mut scene, [0.0, -10.0], [0.0, 10.0]);
    assert_eq!(
        scene.path(h).and_then(|p| p.stroke.as_ref()).map(|s| s.cap),
        Some(ph2d_vec_scene::LineCap::Butt),
        "a fixtura tem de partir de uma tampa RECTA, senao o gate nao mede nada"
    );
    pen.select_many(&[h, vt]);

    apply_vec_weld(&mut scene, &mut hist, &mut pen, &VecXforms::new(), 0.0);

    let rede = &scene.paths()[0];
    assert!(rede.contour_count() > 1, "a fixtura tem de dar uma REDE");
    assert_eq!(
        rede.stroke.as_ref().map(|s| s.cap),
        Some(ph2d_vec_scene::LineCap::Round),
        "sem tampa redonda o traco largo abre uma cunha em cada no'"
    );
}
