//! Gates da COSTURA de **SOLDAR** — o que a lei pura não alcança: a cena, a pose e o undo.

use super::*;
use ph2d_vec_scene::{Rgba8, StrokeSpec, VertexKind, Xform};

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
        id: 0,
        verts: vec![v(a[0], a[1]), v(b[0], b[1])],
        closed: false,
        fill: None,
        stroke: Some(StrokeSpec::new(Rgba8::new(9, 9, 9, 255), 2.0)),
        subpaths: Vec::new(),
        fill_rule: ph2d_vec_scene::FillRule::NonZero,
        effects: Vec::new(),
    })
}

fn cena() -> (VecScene, ph2d_vec_edit::History, ph2d_vec_edit::PenTool) {
    (
        VecScene::new(),
        ph2d_vec_edit::History::default(),
        ph2d_vec_edit::PenTool::default(),
    )
}

/// ⭐⭐⭐ **A IDEIA DO ENIO:** duas linhas cruzadas viram quatro arcos, e as quatro pontas do meio
/// caem **no mesmo sítio** — é isso o nó partilhado.
#[test]
fn two_crossing_lines_become_four_arcs_that_meet_at_one_point() {
    let (mut scene, mut hist, mut pen) = cena();
    let h = reta(&mut scene, [-10.0, 0.0], [10.0, 0.0]);
    let vt = reta(&mut scene, [0.0, -10.0], [0.0, 10.0]);
    pen.select_many(&[h, vt]);

    apply_vec_weld(&mut scene, &mut hist, &mut pen, &VecXforms::new(), 0.0);

    assert_eq!(
        scene.paths().len(),
        4,
        "duas linhas cruzadas dao QUATRO arcos"
    );
    // As oito pontas: quatro nas extremidades, quatro no cruzamento.
    let no_centro = scene
        .paths()
        .iter()
        .flat_map(|p| [p.verts[0].anchor, p.verts[p.verts.len() - 1].anchor])
        .filter(|a| a[0].abs() < 1e-6 && a[1].abs() < 1e-6)
        .count();
    assert_eq!(
        no_centro, 4,
        "as quatro pontas do meio tem de cair EXACTAMENTE no cruzamento"
    );
    assert!(
        scene.paths().iter().all(|p| p.stroke.is_some()),
        "o estilo viaja para cada arco"
    );
}

/// ⚠️ **Um caminho que não encontra ninguém NÃO é tocado** — nem o objecto, nem o id. Sem isto,
/// soldar uma selecção grande dissolveria tudo o que estava só a passar por lá.
#[test]
fn a_path_that_meets_nobody_keeps_its_identity() {
    let (mut scene, mut hist, mut pen) = cena();
    let h = reta(&mut scene, [-10.0, 0.0], [10.0, 0.0]);
    let vt = reta(&mut scene, [0.0, -10.0], [0.0, 10.0]);
    let longe = reta(&mut scene, [100.0, 100.0], [120.0, 100.0]);
    pen.select_many(&[h, vt, longe]);

    apply_vec_weld(&mut scene, &mut hist, &mut pen, &VecXforms::new(), 0.0);

    assert!(
        scene.path(longe).is_some(),
        "o traco distante perdeu o id — ele nao cruza nada"
    );
    assert_eq!(scene.paths().len(), 5, "4 arcos + o intocado");
}

/// ⚠️⚠️ **A POSE entra na conta.** As duas linhas só se cruzam DEPOIS de o `Transform` da segunda a
/// pôr no lugar; medir na geometria local diria que elas não se encontram.
#[test]
fn the_crossing_is_found_in_world_space_not_in_local() {
    let (mut scene, mut hist, mut pen) = cena();
    let h = reta(&mut scene, [-10.0, 0.0], [10.0, 0.0]);
    // Esta nasce longe e é TRAZIDA para cima da outra pela pose.
    let vt = reta(&mut scene, [500.0, -10.0], [500.0, 10.0]);
    let mut xf = VecXforms::new();
    xf.insert(vt, Xform([1.0, 0.0, 0.0, 1.0, -500.0, 0.0]));
    pen.select_many(&[h, vt]);

    apply_vec_weld(&mut scene, &mut hist, &mut pen, &xf, 0.0);
    assert_eq!(
        scene.paths().len(),
        4,
        "sem a pose na conta, as duas linhas nao se encontram"
    );
}

/// **Nada se cruza ⇒ NADA acontece**, e sem passo de undo: um comando que não fez nada não pode
/// gastar um Ctrl+Z.
#[test]
fn a_selection_that_crosses_nothing_is_a_no_op() {
    let (mut scene, mut hist, mut pen) = cena();
    let a = reta(&mut scene, [-10.0, 0.0], [10.0, 0.0]);
    let b = reta(&mut scene, [-10.0, 50.0], [10.0, 50.0]);
    pen.select_many(&[a, b]);
    let antes = scene.paths().len();

    apply_vec_weld(&mut scene, &mut hist, &mut pen, &VecXforms::new(), 0.0);

    assert_eq!(scene.paths().len(), antes);
    assert!(scene.path(a).is_some() && scene.path(b).is_some());
}

/// ⭐⭐ **E é isto que fecha as ÁREAS**: um X dentro de um quadrado dá uma rede em que cada região
/// é cercada por arcos que se encontram nas pontas — o substrato que o balde vai usar.
#[test]
fn a_cross_inside_a_square_becomes_a_network_of_arcs() {
    let (mut scene, mut hist, mut pen) = cena();
    let quadrado = scene.push_path(VecPath {
        id: 0,
        verts: vec![
            v(-10.0, -10.0),
            v(10.0, -10.0),
            v(10.0, 10.0),
            v(-10.0, 10.0),
        ],
        closed: true,
        fill: None,
        stroke: Some(StrokeSpec::new(Rgba8::new(9, 9, 9, 255), 2.0)),
        subpaths: Vec::new(),
        fill_rule: ph2d_vec_scene::FillRule::NonZero,
        effects: Vec::new(),
    });
    let diagonal = reta(&mut scene, [-20.0, 0.0], [20.0, 0.0]);
    pen.select_many(&[quadrado, diagonal]);

    apply_vec_weld(&mut scene, &mut hist, &mut pen, &VecXforms::new(), 0.0);

    // O quadrado é cortado nos dois lados e a linha nos dois cruzamentos.
    assert!(
        scene.paths().len() >= 5,
        "a rede tem de ter os arcos dos dois: {} caminhos",
        scene.paths().len()
    );
    assert!(
        scene.paths().iter().all(|p| !p.closed),
        "depois de soldado nao sobra anel — tudo e' arco"
    );
}

/// ⭐⭐⭐ **O REPORT (Enio, 2026-08-31, com foto):** *"weld dividiu e não soldou (eu que afastei os
/// pontos)"*.
///
/// ⚠️ **A fixtura são dois CÍRCULOS**, e não duas retas: com retas cruzando em coordenadas redondas
/// os dois lados calculam o mesmo ponto por acaso, e o gate passaria com a solda desligada. *Foi a
/// primeira redacção deste teste, e três mutações sobreviveram a ela.*
///
/// As duas metades: as pontas de um nó são **a mesma coordenada**, e **arrastar uma leva as
/// outras** — que é literalmente o teste que ele fez.
#[test]
fn the_welded_node_is_one_point_and_dragging_it_moves_every_arc() {
    let (mut scene, mut hist, mut pen) = cena();
    let a = scene.push_path(ph2d_vec_scene::ellipse([0.0, 0.0], 100.0, 100.0));
    let b = scene.push_path(ph2d_vec_scene::ellipse([120.0, 0.0], 100.0, 100.0));
    pen.select_many(&[a, b]);
    apply_vec_weld(&mut scene, &mut hist, &mut pen, &VecXforms::new(), 0.0);
    assert_eq!(
        scene.paths().len(),
        4,
        "dois circulos que se cruzam dao 4 arcos"
    );

    // METADE 1 — no nó de cima, as duas pontas são **a mesma coordenada**, bit a bit.
    let cima: Vec<[f64; 2]> = scene
        .paths()
        .iter()
        .flat_map(|p| [p.verts[0].anchor, p.verts[p.verts.len() - 1].anchor])
        .filter(|p| p[1] > 50.0)
        .collect();
    // ⚠️ **QUATRO, e não duas**: cada círculo dá DOIS arcos, e cada arco tem uma ponta em cada
    // nó — logo o nó de cima recebe uma ponta de cada um dos quatro arcos. (A minha primeira
    // contagem dizia duas.)
    assert_eq!(cima.len(), 4, "o no' de cima tem quatro pontas: {cima:?}");
    assert!(
        cima.windows(2).all(|w| w[0] == w[1]),
        "as pontas do no' nao sao o MESMO ponto — isto e' dividir, nao soldar: {cima:?}"
    );

    // METADE 2 — o ARRASTO. Agarra uma ponta do nó e leva-a; as duas têm de ir juntas.
    let no = cima[0];
    // ⚠️ `on_press_node`, e não `on_press`: aquele é a CANETA (desenha/insere), este é a seta
    // branca — a ferramenta que agarra um nó, que é o gesto do report.
    let ptw = 0.1; // raio de captura ≈ 1 unidade de mundo, longe de apanhar o vizinho
    let clique = pen.on_press_node(&mut scene, no, ptw, false);
    assert!(
        matches!(clique, ph2d_vec_edit::PenClick::Grabbed),
        "o press tem de AGARRAR a ponta do no', e deu {clique:?}"
    );
    let destino = [no[0] + 30.0, no[1] + 40.0];
    assert!(pen.on_drag(&mut scene, destino, &mut |p| p));
    pen.on_release();

    let no_destino = scene
        .paths()
        .iter()
        .flat_map(|p| [p.verts[0].anchor, p.verts[p.verts.len() - 1].anchor])
        .filter(|p| (p[0] - destino[0]).abs() < 1e-6 && (p[1] - destino[1]).abs() < 1e-6)
        .count();
    assert_eq!(
        no_destino, 4,
        "arrastar uma ponta tem de levar as outras TRES — o no' e' partilhado"
    );
}

/// ⚠️ **Uma ponta SOZINHA não é junta** — e sem esta metade o gate acima passaria com um
/// `welded_with` que devolvesse tudo o que está perto.
#[test]
fn a_lone_endpoint_is_not_a_joint() {
    let (mut scene, _h, pen) = cena();
    let a = reta(&mut scene, [0.0, 0.0], [10.0, 0.0]);
    let _b = reta(&mut scene, [0.0, 50.0], [10.0, 50.0]);
    assert!(pen.welded_with(&scene, a, 0).is_empty());
}

/// ⭐⭐⭐ **O REPORT (Enio, 2026-09-01):** *"ainda não consegue conectar as duas curvas. 2 curvas
/// geram outras duas linhas … mas as linhas não compartilham o mesmo nó"*.
///
/// ⚠️ **Medido antes de mexer:** duas curvas ponta-com-ponta a `0,36` de distância faziam o comando
/// recusar-se (*"nada se cruza"*) — porque **cruzar** era a única forma de se encontrar que ele
/// conhecia. Duas curvas que ACABAM no mesmo sítio partilham um nó tanto quanto duas que se
/// atravessam.
///
/// ⚠️ **E o objecto SOBREVIVE**: nada aqui se cruza, logo nada se dissolve — os dois caminhos ficam
/// com o id, o estilo e a pose, e só a ponta se muda. *Dissolver um traço que ninguém cortou seria
/// cobrar o preço do corte por uma ligação.*
#[test]
fn two_curves_that_meet_at_their_ends_become_one_node() {
    let (mut scene, mut hist, mut pen) = cena();
    let a = reta(&mut scene, [-100.0, 0.0], [0.0, 0.0]);
    let b = reta(&mut scene, [0.3, -0.2], [100.0, 0.0]);
    pen.select_many(&[a, b]);

    apply_vec_weld(&mut scene, &mut hist, &mut pen, &VecXforms::new(), 2.0);

    assert_eq!(scene.paths().len(), 2, "ligar nao dissolve: nada se cruzou");
    assert!(
        scene.path(a).is_some() && scene.path(b).is_some(),
        "os dois caminhos tem de MANTER o id — so' a ponta se mudou"
    );
    let fim_a = *scene.path(a).unwrap().verts.last().unwrap();
    let ini_b = scene.path(b).unwrap().verts[0];
    assert_eq!(
        fim_a.anchor, ini_b.anchor,
        "as duas pontas tem de ser o MESMO ponto, bit a bit"
    );
    // ⚠️ **O CENTROIDE**, e não uma das duas: senão a mesma solda dava geometria diferente conforme
    // a ordem da selecção.
    assert!(
        (fim_a.anchor[0] - 0.15).abs() < 1e-12 && (fim_a.anchor[1] + 0.1).abs() < 1e-12,
        "o no' tem de nascer no MEIO das duas pontas: {:?}",
        fim_a.anchor
    );
    // E o produto conhece a junta — é ela que faz o nó andar inteiro sob o dedo.
    assert_eq!(
        pen.welded_with(&scene, a, 1).len(),
        1,
        "a ponta ligada tem de ter a irma' como junta"
    );
}

/// ⛔ **A FOLGA É UMA CERCA, não um convite.** Com o ímã mais apertado que o vão, as duas curvas
/// ficam onde estavam — e sem passo de undo.
///
/// ⚠️ Sem esta metade, o gate acima passaria com uma solda que junta tudo o que está na selecção,
/// e soldar duas curvas nos dois cantos da tela arrastaria as pontas uma para a outra.
#[test]
fn ends_farther_than_the_magnet_are_left_where_they_are() {
    let (mut scene, mut hist, mut pen) = cena();
    let a = reta(&mut scene, [-100.0, 0.0], [0.0, 0.0]);
    let b = reta(&mut scene, [0.3, -0.2], [100.0, 0.0]);
    pen.select_many(&[a, b]);
    let antes = (
        scene.path(a).unwrap().clone(),
        scene.path(b).unwrap().clone(),
    );

    apply_vec_weld(&mut scene, &mut hist, &mut pen, &VecXforms::new(), 0.1);

    assert_eq!(scene.path(a).unwrap().verts, antes.0.verts);
    assert_eq!(scene.path(b).unwrap().verts, antes.1.verts);
}

/// ⚠️⚠️ **A POSE entra na ligação como entra no cruzamento.** As duas curvas só se encontram DEPOIS
/// de o `Transform` da segunda a trazer para o lugar — e a ponta que se muda desce ao espaço LOCAL
/// dela, senão o objecto salta para o mundo.
#[test]
fn the_meeting_is_found_in_world_space_and_written_in_local() {
    let (mut scene, mut hist, mut pen) = cena();
    let a = reta(&mut scene, [-100.0, 0.0], [0.0, 0.0]);
    let b = reta(&mut scene, [500.3, -0.2], [600.0, 0.0]);
    let mut xf = VecXforms::new();
    xf.insert(b, Xform([1.0, 0.0, 0.0, 1.0, -500.0, 0.0]));
    pen.select_many(&[a, b]);

    apply_vec_weld(&mut scene, &mut hist, &mut pen, &xf, 2.0);

    let fim_a = scene.path(a).unwrap().verts.last().unwrap().anchor;
    let ini_b_local = scene.path(b).unwrap().verts[0].anchor;
    let ini_b_mundo = ph2d_vec_scene::xform_of(&xf, b).apply(ini_b_local);
    assert!(
        (fim_a[0] - ini_b_mundo[0]).abs() < 1e-9 && (fim_a[1] - ini_b_mundo[1]).abs() < 1e-9,
        "no MUNDO as duas pontas tem de coincidir: {fim_a:?} vs {ini_b_mundo:?}"
    );
    assert!(
        ini_b_local[0] > 400.0,
        "a ponta foi escrita em MUNDO em vez de em local — o objecto saltou: {ini_b_local:?}"
    );
}

/// ⛔ **Um caminho com EFEITOS não empresta a ponta.** O que se vê nele é geometria COZIDA, e os
/// vértices autorados já não são as pontas que a medição encontrou — mover o primeiro seria mover
/// outra coisa. Ele fica de fora, intacto.
#[test]
fn a_path_with_effects_does_not_lend_its_endpoint() {
    use ph2d_vec_scene::effect::{FxEntry, PathEffect};
    let (mut scene, mut hist, mut pen) = cena();
    let a = reta(&mut scene, [-100.0, 0.0], [0.0, 0.0]);
    let b = reta(&mut scene, [0.3, -0.2], [100.0, 0.0]);
    scene.path_mut(b).unwrap().effects = vec![FxEntry::new(PathEffect::Trim(
        ph2d_vec_scene::fx_trim::TrimSpec {
            start: 0.1,
            end: 0.9,
            offset: 0.0,
        },
    ))];
    let antes = scene.path(b).unwrap().verts.clone();
    pen.select_many(&[a, b]);

    apply_vec_weld(&mut scene, &mut hist, &mut pen, &VecXforms::new(), 2.0);

    assert_eq!(
        scene.path(b).unwrap().verts,
        antes,
        "o caminho com efeitos teve a ponta mexida"
    );
    assert_eq!(
        scene.path(a).unwrap().verts.last().unwrap().anchor,
        [0.0, 0.0],
        "e sem par, a ponta do outro tambem nao se mexe"
    );
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
