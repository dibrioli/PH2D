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
        ..VecPath::default()
    })
}

/// **Todas as PONTAS da cena** — os dois extremos de cada contorno ABERTO de cada caminho.
///
/// ⚠️⚠️ **Uma sonda que lesse `p.verts` mediria só o PRIMEIRO arco.** Desde 2026-09-02 a rede
/// soldada é UM caminho composto (o report do Enio: *"deveria criar apenas 1"*), e as oito pontas
/// de duas linhas cruzadas vivem todas no mesmo objecto — a régua tem de percorrer contornos, senão
/// chama de verde uma rede que se desfez.
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

/// Quantos contornos a cena inteira tem (a contagem de ARCOS, agora que eles não são caminhos).
fn contornos(scene: &VecScene) -> usize {
    scene.paths().iter().map(VecPath::contour_count).sum()
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
///
/// ⭐⭐⭐ **E os quatro arcos são UM OBJECTO** (report de 2026-09-02: *"o weld cria uma grande
/// quantidade de path na hierarquia quando na verdade deveria criar apenas 1"*). Um `VecPath` é uma
/// entidade ECS (ADR-0110) ⇒ uma linha na Hierarquia, uma pose e um gizmo; quatro deles davam
/// quatro de cada, e mover um **rasgava** a rede que soldar promete manter inteira.
#[test]
fn two_crossing_lines_become_four_arcs_that_meet_at_one_point() {
    let (mut scene, mut hist, mut pen) = cena();
    let h = reta(&mut scene, [-10.0, 0.0], [10.0, 0.0]);
    let vt = reta(&mut scene, [0.0, -10.0], [0.0, 10.0]);
    pen.select_many(&[h, vt]);

    apply_vec_weld(&mut scene, &mut hist, &mut pen, &VecXforms::new(), 0.0);

    assert_eq!(scene.paths().len(), 1, "a rede soldada e' UM objecto");
    assert_eq!(
        contornos(&scene),
        4,
        "duas linhas cruzadas dao QUATRO arcos — um contorno cada"
    );
    // As oito pontas: quatro nas extremidades, quatro no cruzamento.
    let no_centro = pontas(&scene)
        .into_iter()
        .filter(|a| a[0].abs() < 1e-6 && a[1].abs() < 1e-6)
        .count();
    assert_eq!(
        no_centro, 4,
        "as quatro pontas do meio tem de cair EXACTAMENTE no cruzamento"
    );
    let rede = &scene.paths()[0];
    assert!(rede.stroke.is_some(), "o estilo viaja para a rede");
    assert!(
        !rede.closed && rede.subpaths.iter().all(|c| !c.closed),
        "todo contorno da rede e' um ARCO aberto"
    );
}

/// ⭐⭐⭐ **A REDE ESCREVE-SE NO LUGAR DO PARTICIPANTE MAIS AO FUNDO** — o id, a fatia de z e a
/// entidade ECS que o representa sobrevivem, como no Trim (`vec_trim::apply`).
///
/// ⚠️ Sem isto o artista perde o nome que deu à linha: um `insert_path` daria um objecto novo,
/// baptizado `Path N` pela fábrica, e a Hierarquia mostraria um estranho onde estava o traço dele.
///
/// ⛔ **E os outros participantes somem** — é o preço declarado de *"soldar consome os traços"*.
#[test]
fn the_network_is_written_in_place_of_the_bottom_most_line() {
    let (mut scene, mut hist, mut pen) = cena();
    let fundo = reta(&mut scene, [-10.0, 0.0], [10.0, 0.0]);
    let topo = reta(&mut scene, [0.0, -10.0], [0.0, 10.0]);
    pen.select_many(&[fundo, topo]);

    apply_vec_weld(&mut scene, &mut hist, &mut pen, &VecXforms::new(), 0.0);

    assert!(
        scene.path(fundo).is_some(),
        "o id do participante mais ao fundo tem de sobreviver — a rede escreve-se NELE"
    );
    assert!(
        scene.path(topo).is_none(),
        "o outro participante e' consumido"
    );
}

/// ⭐⭐⭐ **TRÊS linhas pelo mesmo ponto continuam a ser UM objecto** — e é aqui que se vê que a lei
/// não é *"dois viram um"*, mas *"a rede é uma coisa"*.
///
/// ⚠️ Sem esta segunda fixtura, um emit que juntasse **pares** passaria no gate acima e deixaria
/// dois objectos num asterisco.
#[test]
fn three_lines_through_one_point_are_still_one_object() {
    let (mut scene, mut hist, mut pen) = cena();
    let a = reta(&mut scene, [-10.0, 0.0], [10.0, 0.0]);
    let b = reta(&mut scene, [0.0, -10.0], [0.0, 10.0]);
    let c = reta(&mut scene, [-7.0, -7.0], [7.0, 7.0]);
    pen.select_many(&[a, b, c]);

    apply_vec_weld(&mut scene, &mut hist, &mut pen, &VecXforms::new(), 0.0);

    assert_eq!(scene.paths().len(), 1, "um asterisco e' UM objecto");
    assert_eq!(
        contornos(&scene),
        6,
        "tres linhas partidas ao meio: 6 arcos"
    );
    let no_centro = pontas(&scene)
        .into_iter()
        .filter(|p| p[0].abs() < 1e-6 && p[1].abs() < 1e-6)
        .count();
    assert_eq!(no_centro, 6, "as seis pontas caem no mesmo no'");
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
    assert_eq!(scene.paths().len(), 2, "a rede + o intocado");
    assert_eq!(
        contornos(&scene),
        5,
        "4 arcos na rede + o contorno do intocado"
    );
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
        contornos(&scene),
        4,
        "sem a pose na conta, as duas linhas nao se encontram"
    );
    // ⚠️ E a rede escreve-se no espaço LOCAL do anfitrião: com a pose dele na identidade, o
    // cruzamento fica na origem — se os arcos ficassem em mundo sem descer, a rede saltaria.
    assert!(
        pontas(&scene)
            .into_iter()
            .any(|p| p[0].abs() < 1e-6 && p[1].abs() < 1e-6),
        "o no' tem de cair na origem do espaco do anfitriao"
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
        ..VecPath::default()
    });
    let diagonal = reta(&mut scene, [-20.0, 0.0], [20.0, 0.0]);
    pen.select_many(&[quadrado, diagonal]);

    apply_vec_weld(&mut scene, &mut hist, &mut pen, &VecXforms::new(), 0.0);

    // O quadrado é cortado nos dois lados e a linha nos dois cruzamentos.
    assert_eq!(scene.paths().len(), 1, "a rede e' UM objecto");
    assert!(
        contornos(&scene) >= 5,
        "a rede tem de ter os arcos dos dois: {} contornos",
        contornos(&scene)
    );
    let rede = &scene.paths()[0];
    assert!(
        !rede.closed && rede.subpaths.iter().all(|c| !c.closed),
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
    assert_eq!(scene.paths().len(), 1, "a rede e' UM objecto");
    assert_eq!(
        contornos(&scene),
        4,
        "dois circulos que se cruzam dao 4 arcos"
    );

    // METADE 1 — no nó de cima, as duas pontas são **a mesma coordenada**, bit a bit.
    let cima: Vec<[f64; 2]> = pontas(&scene).into_iter().filter(|p| p[1] > 50.0).collect();
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

    let no_destino = pontas(&scene)
        .into_iter()
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
