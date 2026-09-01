//! Gates da ferramenta **TRIM** (plano 38 §4) — a lei pura, sem cena e sem ponteiro.

use super::*;
use crate::VertexKind;

/// Um vértice de quina em `(x, y)` — as alças coincidem com a âncora, que é o que faz o segmento
/// entre dois deles ser uma RETA (`P1 = P0`, `P2 = P3`).
fn v(x: f64, y: f64) -> VecVertex {
    VecVertex {
        anchor: [x, y],
        in_handle: [x, y],
        out_handle: [x, y],
        kind: VertexKind::Corner,
        corner_radius: 0.0,
    }
}

fn path_of(verts: Vec<VecVertex>, closed: bool) -> VecPath {
    VecPath {
        id: 1,
        verts,
        closed,
        fill: None,
        stroke: Some(crate::StrokeSpec::new(crate::Rgba8::new(9, 9, 9, 255), 3.0)),
        subpaths: Vec::new(),
        fill_rule: crate::FillRule::NonZero,
        effects: Vec::new(),
    }
}

/// Um quadrado 10×10 fechado, com os nós nos quatro cantos ⇒ fronteiras em `0, ¼, ½, ¾`.
fn quadrado() -> VecPath {
    path_of(
        vec![v(0.0, 0.0), v(10.0, 0.0), v(10.0, 10.0), v(0.0, 10.0)],
        true,
    )
}

/// Uma reta de DOIS segmentos, aberta: nós em `0, ½, 1`.
fn ele() -> VecPath {
    path_of(vec![v(0.0, 0.0), v(10.0, 0.0), v(10.0, 10.0)], false)
}

fn quase(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

/// **AS QUATRO ESPÉCIES DE FRONTEIRA aparecem na lista, e numa lista SÓ.**
///
/// ⚠️ A metade que importa é a última: um cruzamento entre dois nós entra ENTRE eles, e é isso que
/// o faz vencer quando está mais perto. Duas listas separadas não teriam onde exprimir isso.
#[test]
fn the_four_species_of_boundary_land_in_one_sorted_list() {
    // FECHADO: só os nós (um fechado não tem ponta).
    let q = quadrado();
    let b = boundaries(&q.verts, true, &[]);
    assert_eq!(b.len(), 4, "quatro nos, e nenhuma ponta: {b:?}");
    for (i, f) in b.iter().enumerate() {
        assert!(quase(*f, i as f64 * 0.25), "no {i} em {f}");
    }

    // ABERTO: os nós MAIS as duas pontas — e a ponta final é a fronteira `1`.
    let l = ele();
    let b = boundaries(&l.verts, false, &[]);
    assert_eq!(b.len(), 3, "dois nos + a ponta de chegada: {b:?}");
    assert!(quase(b[0], 0.0) && quase(b[1], 0.5) && quase(b[2], 1.0));

    // CRUZAMENTO: entra ordenado, entre os nós que o cercam.
    let b = boundaries(&l.verts, false, &[0.30]);
    assert_eq!(b.len(), 4);
    assert!(quase(b[1], 0.30), "o cruzamento nao entrou em ordem: {b:?}");
}

/// **O pedaço entre dois nós de um polígono é UM LADO** — nem mais, nem menos. É o *"entre pontos"*
/// do pedido.
#[test]
fn the_piece_between_two_nodes_is_exactly_one_side() {
    let q = quadrado();
    let b = boundaries(&q.verts, true, &[]);
    let (de, ate) = piece_at(&b, true, 0.10).expect("ha' fronteira");
    assert!(quase(de, 0.0) && quase(ate, 0.25), "({de}, {ate})");

    // E o cursor logo depois do 2.º nó cai no lado SEGUINTE — a escolha é por vizinhança, não por
    // um índice qualquer.
    let (de, ate) = piece_at(&b, true, 0.60).expect("ha' fronteira");
    assert!(quase(de, 0.5) && quase(ate, 0.75), "({de}, {ate})");
}

/// ⭐⭐ **UM CRUZAMENTO MAIS PERTO VENCE O NÓ** — o caso *"entre linhas sobrepostas"*.
///
/// ⚠️ A fixtura CONTÉM o fenómeno: sem o cruzamento o mesmo cursor devolveria o lado inteiro, e é
/// essa a segunda asserção. *Um gate que só mostra o resultado com a cura não prova que ela agiu.*
#[test]
fn a_nearer_crossing_beats_the_node() {
    let q = quadrado();
    let sem = boundaries(&q.verts, true, &[]);
    let (_, ate_sem) = piece_at(&sem, true, 0.10).unwrap();
    assert!(
        quase(ate_sem, 0.25),
        "controle: sem cruzamento vai ate' o no'"
    );

    let com = boundaries(&q.verts, true, &[0.15]);
    let (de, ate) = piece_at(&com, true, 0.10).unwrap();
    assert!(
        quase(de, 0.0) && quase(ate, 0.15),
        "o cruzamento tinha de cortar antes do no': ({de}, {ate})"
    );
}

/// **ABERTO, no MEIO ⇒ DOIS contornos.** É o corte que parte a fita.
///
/// ⚠️⚠️ **A 1.ª redacção desta fixtura punha os cruzamentos em `0,30` e `0,60` com o cursor a
/// `0,45` — e reprovou sobre código CORRECTO.** O `ele()` tem um nó em `0,50`, entre os dois, e a
/// lei diz *a fronteira mais PRÓXIMA*, seja de que espécie for: o pedaço certo era `(0,30 · 0,50)`.
/// *Eu escrevi a fixtura a pensar «o cruzamento manda» e a lei nunca disse isso* — ela diz que as
/// quatro espécies vivem numa lista só, e uma lista só não tem precedências. Os cruzamentos
/// mudaram-se para DEPOIS do nó, que é onde a pergunta é sobre o meio de uma fita.
#[test]
fn cutting_the_middle_of_an_open_contour_leaves_two_strands() {
    let l = ele();
    let b = boundaries(&l.verts, false, &[0.55, 0.70]);
    let (de, ate) = piece_at(&b, false, 0.62).expect("ha' fronteira");
    assert!(quase(de, 0.55) && quase(ate, 0.70), "({de}, {ate})");
    let out = sever(&l, 0, de, ate).expect("sobra geometria");
    assert_eq!(
        1 + out.subpaths.len(),
        2,
        "um corte no meio de uma fita da' DUAS fitas"
    );
    assert!(!out.closed && out.subpaths.iter().all(|c| !c.closed));
}

/// **FECHADO ⇒ UM contorno, agora ABERTO.** Cortar um anel abre-o; ele não vira dois.
#[test]
fn cutting_a_closed_contour_opens_it_into_one_strand() {
    let q = quadrado();
    let b = boundaries(&q.verts, true, &[]);
    let (de, ate) = piece_at(&b, true, 0.10).unwrap();
    let out = sever(&q, 0, de, ate).expect("sobra geometria");
    assert!(out.subpaths.is_empty(), "um anel cortado da' UMA fita");
    assert!(!out.closed, "e ela e' ABERTA");
}

/// **A PEÇA TODA ⇒ o caminho é apagado** (o `None`), e é a resposta do Fusion: uma reta que não
/// cruza nada não tem meio para aparar.
#[test]
fn a_single_segment_with_no_crossing_is_the_whole_piece_and_the_path_goes() {
    let reta = path_of(vec![v(0.0, 0.0), v(10.0, 0.0)], false);
    let b = boundaries(&reta.verts, false, &[]);
    let (de, ate) = piece_at(&b, false, 0.5).unwrap();
    assert!(quase(de, 0.0) && quase(ate, 1.0), "a peca toda");
    assert!(
        sever(&reta, 0, de, ate).is_none(),
        "sem geometria a sobrar, quem chama tem de APAGAR o caminho"
    );
}

/// **O ESTILO e os outros CONTORNOS sobrevivem.** Um traço aparado continua com a cor e a largura
/// que tinha, e um compound perde só a fita em que se carregou.
#[test]
fn the_style_and_the_untouched_contours_survive_the_cut() {
    let mut p = quadrado();
    p.subpaths.push(Contour {
        verts: vec![v(2.0, 2.0), v(4.0, 2.0), v(4.0, 4.0)],
        closed: true,
    });
    let antes = p.stroke.clone();
    let b = boundaries(&p.verts, true, &[]);
    let (de, ate) = piece_at(&b, true, 0.10).unwrap();
    let out = sever(&p, 0, de, ate).expect("sobra");
    assert_eq!(out.stroke, antes, "o estilo viaja");
    assert_eq!(out.id, p.id, "e o id tambem — e' o mesmo objecto");
    assert!(
        out.subpaths.iter().any(|c| c.closed && c.verts.len() == 3),
        "o contorno que ninguem tocou tem de sair intacto"
    );
}

/// **Cortar um SUBPATH não toca o primário** — o índice do contorno é o que o hit-test devolve, e
/// os dois lados têm de contar da mesma maneira.
#[test]
fn severing_a_subpath_leaves_the_primary_alone() {
    let mut p = quadrado();
    p.subpaths.push(Contour {
        verts: vec![v(2.0, 2.0), v(6.0, 2.0), v(6.0, 6.0), v(2.0, 6.0)],
        closed: true,
    });
    let sub = contours_of(&p).nth(1).expect("o subpath");
    let b = boundaries(&sub.verts, sub.closed, &[]);
    let (de, ate) = piece_at(&b, true, 0.10).unwrap();
    let out = sever(&p, 1, de, ate).expect("sobra");
    assert_eq!(out.verts.len(), 4, "o primario nao se mexeu");
    assert!(out.closed, "e continua fechado");
    assert!(
        out.subpaths.iter().any(|c| !c.closed),
        "o subpath cortado saiu aberto"
    );
}

/// ⛔⛔ **O PEDAÇO QUE ATRAVESSA A EMENDA** — achado pelo gate de SOLDAR, três horas depois desta
/// wave, e é a divergência que o desenho desta ferramenta existe para proibir.
///
/// O corte vai por `strands_of` (que normaliza a volta) e o realce por `piece_geometry` (que não
/// normalizava): num anel, um pedaço que passa pela costura **não acendia** e ainda assim era
/// comido. ⚠️ **O gate de «a mesma porta» não o apanhou** porque as duas portas só discordam sobre a
/// emenda, e nenhuma fixtura tinha um pedaço que passasse por lá. *Uma porta única prova que a
/// resposta é a mesma; ela não escolhe as perguntas que se fazem.*
#[test]
fn a_piece_that_crosses_the_seam_is_drawn_and_not_only_cut() {
    let q = quadrado();
    // Fronteiras em 0 · ¼ · ½ · ¾ ; o cursor logo ANTES do fecho cai no pedaço `(0,75 → 0)`.
    let b = boundaries(&q.verts, true, &[]);
    let (de, ate) = piece_at(&b, true, 0.9).expect("ha' fronteira");
    assert!(
        ate < de,
        "a fixtura precisa de um pedaco que atravesse a emenda: ({de}, {ate})"
    );

    let realce = piece_geometry(&q.verts, true, de, ate).expect("o realce tem de existir");
    assert!(
        realce.len() >= 2,
        "o realce saiu vazio: {} vertices",
        realce.len()
    );

    // …e ele é o COMPLEMENTO do que sobra: o corte deixa uma fita, o realce mostra a outra.
    let sobra = sever(&q, 0, de, ate).expect("sobra geometria");
    assert!(sobra.subpaths.is_empty() && !sobra.closed);
}

/// **Uma fronteira só num anel ⇒ o pedaço é a VOLTA INTEIRA**, e o realce mostra-a. Com `<` em vez
/// de `<=` na normalização da emenda, este caso devolvia geometria nenhuma.
#[test]
fn a_ring_with_a_single_boundary_highlights_the_whole_loop() {
    let anel = path_of(
        vec![v(0.0, 0.0), v(10.0, 0.0), v(10.0, 10.0), v(0.0, 10.0)],
        true,
    );
    let inteiro = piece_geometry(&anel.verts, true, 0.4, 0.4).expect("a volta inteira");
    assert!(
        inteiro.len() >= 4,
        "so' {} vertices na volta inteira",
        inteiro.len()
    );
}
