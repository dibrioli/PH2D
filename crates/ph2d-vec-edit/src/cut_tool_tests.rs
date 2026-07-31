//! Gates da TESOURA ([`super`]) — o 1º consumidor do corte.
//!
//! ⚠️ **As fixtures são grandes** (lados de 100 unidades) de propósito: o gesto compara distâncias
//! contra um `hit_r` de MUNDO, e numa forma de tamanho 1 todo ponto está dentro do raio — a fixture
//! não conteria a distinção entre *cortar no vértice* e *cortar no meio do segmento*, que é a única
//! decisão que este arquivo tem.

use crate::PenTool;
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecVertex, VertexKind};

const HIT: f64 = 6.0;

fn v(x: f64, y: f64) -> VecVertex {
    VecVertex {
        anchor: [x, y],
        in_handle: [x, y],
        out_handle: [x, y],
        kind: VertexKind::Corner,
        corner_radius: 0.0,
    }
}

/// Um quadrado FECHADO de lado 100, com o canto inferior-esquerdo na origem.
fn square(scene: &mut VecScene) -> VecPathId {
    scene.push_path(VecPath {
        verts: vec![v(0.0, 0.0), v(100.0, 0.0), v(100.0, 100.0), v(0.0, 100.0)],
        closed: true,
        ..VecPath::default()
    })
}

fn line(scene: &mut VecScene, pts: &[[f64; 2]]) -> VecPathId {
    scene.push_path(VecPath {
        verts: pts.iter().map(|p| v(p[0], p[1])).collect(),
        closed: false,
        ..VecPath::default()
    })
}

#[test]
fn one_snip_opens_a_closed_shape_without_creating_a_second_object() {
    let mut scene = VecScene::default();
    let id = square(&mut scene);
    let mut pen = PenTool::new();

    // O meio da aresta de baixo.
    assert_eq!(pen.scissors_cut(&mut scene, [50.0, 0.0], HIT), Some(id));

    assert_eq!(scene.paths().len(), 1, "abrir não cria objeto");
    let p = &scene.paths()[0];
    assert!(!p.closed, "a forma abriu");
    assert_eq!(
        p.verts.len(),
        6,
        "4 originais + o que a tesoura inseriu + a costura duplicada"
    );
    assert_eq!(p.verts[0].anchor, [50.0, 0.0], "re-enraizado no corte");
    assert_eq!(p.verts[5].anchor, [50.0, 0.0], "a costura nas duas pontas");
}

/// A cena que o plano pede: **duas tesouradas numa forma fechada dão duas peças abertas.**
#[test]
fn two_snips_cut_a_closed_shape_into_two_open_pieces() {
    let mut scene = VecScene::default();
    square(&mut scene);
    let mut pen = PenTool::new();

    pen.scissors_cut(&mut scene, [50.0, 0.0], HIT).unwrap();
    pen.scissors_cut(&mut scene, [50.0, 100.0], HIT).unwrap();

    assert_eq!(scene.paths().len(), 2, "duas peças");
    for p in scene.paths() {
        assert!(!p.closed, "as duas ficam ABERTAS");
        assert!(p.verts.len() >= 3);
    }
    // As duas metades cobrem o quadrado inteiro: nenhuma aresta se perdeu.
    let total: usize = scene.paths().iter().map(|p| p.verts.len()).sum();
    assert_eq!(total, 8, "4 originais + 2 cortes, cada um duplicado");
}

/// **Clicar EM CIMA de um vértice corta NELE.** Sem esta metade, o clique inseria um segundo
/// vértice coincidente e deixava um segmento de comprimento zero — invisível no desenho e um
/// degrau em todo Simplify/Average/Delete seguinte.
#[test]
fn snipping_on_an_existing_anchor_does_not_insert_a_twin() {
    let mut scene = VecScene::default();
    let id = square(&mut scene);
    let mut pen = PenTool::new();

    // Em cima do canto (100, 0), dentro do raio de captura.
    assert_eq!(pen.scissors_cut(&mut scene, [98.0, 1.0], HIT), Some(id));

    let p = &scene.paths()[0];
    assert_eq!(p.verts.len(), 5, "4 + a costura — NENHUM vértice inserido");
    assert_eq!(p.verts[0].anchor, [100.0, 0.0], "cortou no canto");
    // E não há dois vértices no mesmo ponto no MEIO do caminho.
    for w in p.verts.windows(2) {
        assert_ne!(w[0].anchor, w[1].anchor, "segmento de comprimento zero");
    }
}

#[test]
fn snipping_the_middle_of_an_open_path_splits_it_in_two() {
    let mut scene = VecScene::default();
    let id = line(&mut scene, &[[0.0, 0.0], [100.0, 0.0]]);
    let mut pen = PenTool::new();

    assert_eq!(pen.scissors_cut(&mut scene, [50.0, 0.0], HIT), Some(id));

    assert_eq!(scene.paths().len(), 2);
    let xs: Vec<Vec<f64>> = scene
        .paths()
        .iter()
        .map(|p| p.verts.iter().map(|v| v.anchor[0]).collect())
        .collect();
    assert_eq!(xs[0], vec![0.0, 50.0]);
    assert_eq!(xs[1], vec![50.0, 100.0]);
}

/// A ponta de um caminho aberto não tem o que abrir — o Illustrator também recusa. A recusa tem de
/// ser LIMPA: a forma fica exatamente como estava, sem vértice inserido.
#[test]
fn snipping_the_end_of_an_open_path_is_refused_and_leaves_it_untouched() {
    let mut scene = VecScene::default();
    line(&mut scene, &[[0.0, 0.0], [100.0, 0.0]]);
    let mut pen = PenTool::new();

    assert_eq!(pen.scissors_cut(&mut scene, [1.0, 0.0], HIT), None);

    assert_eq!(scene.paths().len(), 1);
    assert_eq!(scene.paths()[0].verts.len(), 2, "nenhum vértice inserido");
}

#[test]
fn snipping_empty_canvas_does_nothing() {
    let mut scene = VecScene::default();
    square(&mut scene);
    let mut pen = PenTool::new();

    assert_eq!(pen.scissors_cut(&mut scene, [500.0, 500.0], HIT), None);
    assert_eq!(scene.paths().len(), 1);
    assert!(scene.paths()[0].closed, "nada foi aberto");
}

/// O gesto vale **sem pré-selecionar** — a tesoura pega o que está sob o cursor, como as
/// ferramentas de quina e a de largura.
#[test]
fn the_scissors_picks_the_path_under_the_cursor_without_a_prior_selection() {
    let mut scene = VecScene::default();
    let _left = line(&mut scene, &[[0.0, 0.0], [100.0, 0.0]]);
    let right = line(&mut scene, &[[0.0, 500.0], [100.0, 500.0]]);
    let mut pen = PenTool::new();
    assert_eq!(pen.selected(), None, "a premissa: nada selecionado");

    assert_eq!(
        pen.scissors_cut(&mut scene, [50.0, 500.0], HIT),
        Some(right)
    );
    assert_eq!(pen.selected(), Some(right), "e passa a ser o selecionado");
}

/// A seleção de nó não sobrevive a um corte: os índices planos a jusante andam.
#[test]
fn the_scissors_drops_the_vertex_selection_it_invalidates() {
    let mut scene = VecScene::default();
    let id = square(&mut scene);
    let mut pen = PenTool::new();
    pen.select(Some(id));
    pen.selected_verts = vec![3];

    pen.scissors_cut(&mut scene, [50.0, 0.0], HIT).unwrap();

    assert!(pen.selected_verts().is_empty());
}

/// **Cortar no segmento de FECHO** — o que vai do último vértice de volta ao primeiro. Ele só
/// existe num contorno fechado, e o "vértice seguinte" dele **dá a volta**: sem o wrap, o índice
/// pedido cai fora do contorno, a porta devolve `None` e a tesoura recusa em silêncio uma aresta
/// que está desenhada na tela.
///
/// ⚠️ Nenhuma outra fixture deste arquivo corta ali (todas caem na aresta de baixo, o segmento 0),
/// e por isso a mutação que tira o wrap sobrevivia a elas.
#[test]
fn snipping_the_closing_segment_works_because_the_next_vertex_wraps() {
    let mut scene = VecScene::default();
    let id = square(&mut scene);
    let mut pen = PenTool::new();

    // A aresta ESQUERDA (x = 0): o segmento que liga o 4º vértice de volta ao 1º.
    assert_eq!(
        pen.scissors_cut(&mut scene, [0.0, 50.0], HIT),
        Some(id),
        "a tesoura recusou a aresta de fecho -- ela esta' desenhada na tela"
    );

    let p = &scene.paths()[0];
    assert!(!p.closed);
    assert_eq!(p.verts[0].anchor, [0.0, 50.0], "cortou onde se clicou");
}

/// E cortar em cima do vértice que FECHA o contorno (o último) também tem de cair nele, não num
/// gêmeo inserido — é a mesma pergunta do wrap, do outro lado.
#[test]
fn snipping_on_the_last_anchor_of_a_closed_contour_cuts_at_it() {
    let mut scene = VecScene::default();
    let id = square(&mut scene);
    let mut pen = PenTool::new();

    assert_eq!(pen.scissors_cut(&mut scene, [1.0, 98.0], HIT), Some(id));

    let p = &scene.paths()[0];
    assert_eq!(p.verts.len(), 5, "4 + a costura — nenhum inserido");
    assert_eq!(p.verts[0].anchor, [0.0, 100.0]);
}
