//! Testes do [`crate::vec_connector_panel`] — módulo irmão (teto de 600 LOC por arquivo).
//!
//! O que eles provam, em ordem de importância:
//!
//! 1. **A seção existe SSE há conector na seleção** (`selection_snapshot`): sem isso o
//!    painel mostraria campos de um objeto que não é conector — ou esconderia os de um que
//!    é. O teste é bilateral: inverter a condição sai vermelho dos dois lados.
//! 2. **A edição atinge TODOS os selecionados** — é o pedido do Enio (calibrar o diagrama
//!    inteiro de uma vez). Editar só o primeiro passaria num teste de unidade do componente
//!    e falharia na mão do usuário.
//! 3. **O campo nasce no valor EFETIVO** (o automático), não em zero — senão o número salta
//!    no primeiro toque, longe da linha que o olho vê.
//! 4. **Editar FIXA** (`None` → `Some`) e o valor fixado **não é re-derivado** no frame
//!    seguinte, nem quando as caixas mudam de tamanho.

use super::*;
use ph2d_ecs::{ConnectorEnd, SimWorld};
use ph2d_vec_scene::{VecScene, rectangle};

/// Cena de teste: dois retângulos (A pequeno, B grande) + `n` conectores entre eles.
/// Devolve `(sim, scene, map, [ids dos conectores])`.
fn scene_with_connectors(n: usize) -> (SimWorld, VecScene, VecEntityMap, Vec<VecPathId>) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    // A: [0,0]-[2,1] (meias-extensões 1.0 × 0.5) · B: [8,0]-[12,4] (2.0 × 2.0).
    let a = scene.push_path(rectangle([0.0, 0.0], [2.0, 1.0]));
    let b = scene.push_path(rectangle([8.0, 0.0], [12.0, 4.0]));
    let conns: Vec<VecPathId> = (0..n)
        .map(|_| scene.push_path(ph2d_vec_scene::line([0.0, 0.0], [0.0, 0.0])))
        .collect();
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    for (i, &c) in conns.iter().enumerate() {
        let mut conn = VecConnector::between(a, b);
        #[expect(clippy::cast_possible_truncation, reason = "n <= 3 nos testes")]
        {
            conn.parallel_index = i as u8;
        }
        let e = Entity::from_bits(*map.get(&c).expect("o conector tem entidade"));
        sim.world_mut()
            .get_entity_mut(e)
            .expect("a entidade existe")
            .insert(conn);
    }
    (sim, scene, map, conns)
}

fn xforms(sim: &SimWorld, map: &VecEntityMap) -> VecXforms {
    crate::vec_transform::build(sim, map)
}

/// O jetty automático desta cena: `k · max(min-meia-extensão)` = `0.35 · max(0.5, 2.0)`.
fn expected_auto_jetty() -> f64 {
    connector::auto_jetty((1.0, 0.5), (2.0, 2.0))
}

/// **A seção só existe com um conector na seleção.** Bilateral de propósito: uma seleção de
/// FORMAS (as duas caixas) não pode publicar snapshot nenhum — se publicasse, o painel
/// mostraria Route/Jetty/Spread de um retângulo. E uma seleção que CONTÉM um conector tem
/// de publicar, mesmo quando ele não é o primeiro item (a seleção mistura formas e linhas).
#[test]
fn the_connector_section_shows_up_exactly_when_a_connector_is_selected() {
    let (sim, scene, map, conns) = scene_with_connectors(1);
    let xf = xforms(&sim, &map);
    // As duas FORMAS estão nos ids 1 e 2 (os primeiros empurrados na cena).
    let shapes: Vec<VecPathId> = scene
        .paths()
        .iter()
        .map(|p| p.id)
        .filter(|id| !conns.contains(id))
        .collect();
    assert_eq!(shapes.len(), 2, "precondicao: duas formas na cena");

    assert!(
        selection_snapshot(&sim, &map, &scene, &xf, &shapes).is_none(),
        "so formas selecionadas: a secao Connector NAO pode aparecer"
    );
    assert!(
        selection_snapshot(&sim, &map, &scene, &xf, &[]).is_none(),
        "selecao vazia: a secao Connector NAO pode aparecer"
    );
    // O conector NÃO é o primeiro da seleção — a busca não pode parar no item 0.
    let mixed = vec![shapes[0], conns[0]];
    assert!(
        selection_snapshot(&sim, &map, &scene, &xf, &mixed).is_some(),
        "a selecao contem um conector: a secao TEM de aparecer (mesmo com uma forma na frente)"
    );
}

/// **O campo nasce no valor EFETIVO — o automático, não zero.** É o que faz o número já
/// estar onde o olho vê a linha; um campo em zero SALTARIA no primeiro toque, e calibrar
/// "com o olho" (o pedido) viraria adivinhação.
#[test]
fn a_pristine_connector_publishes_the_automatic_values_not_zero() {
    let (sim, scene, map, conns) = scene_with_connectors(2);
    let xf = xforms(&sim, &map);

    let (route, jetty, spread, corner, _curve) =
        selection_snapshot(&sim, &map, &scene, &xf, &conns[..1]).expect("ha conector");
    assert_eq!(
        route, 1,
        "o default e Orthogonal (o cotovelo do fluxograma)"
    );
    assert!(jetty > 0.0, "o jetty EFETIVO nao pode nascer em zero");
    assert!(
        corner.abs() < 1e-12,
        "a quina do percurso nasce AFIADA (o default do fluxograma classico): {corner}"
    );
    assert!(
        (jetty - expected_auto_jetty()).abs() < 1e-12,
        "o jetty exibido tem de ser o MESMO que o cozimento usa (semente = amostra): \
         exibido {jetty}, automatico {}",
        expected_auto_jetty()
    );
    assert!(
        spread.abs() < 1e-12,
        "o conector de indice 0 sai pelo CENTRO da face"
    );
    // O SEGUNDO conector do mesmo par (index 1) já nasce deslocado — e o painel mostra
    // esse deslocamento, não zero: é o valor que ele teria de fixar para não mudar nada.
    let (_, _, spread1, _, _curve) =
        selection_snapshot(&sim, &map, &scene, &xf, &conns[1..2]).expect("ha conector");
    assert!(
        (spread1 - VecConnector::auto_spread(1, connector::SPREAD_STEP)).abs() < 1e-12,
        "o spread EFETIVO do 2o conector paralelo tem de ser o automatico dele"
    );
}

/// **A edição atinge TODOS os conectores selecionados.** É o pedido: selecionar o diagrama
/// e calibrar de uma vez. Um `apply` que parasse no primeiro passaria em todo teste de
/// unidade do componente e falharia na mão do usuário — este é o gate que morde.
#[test]
fn editing_jetty_reaches_every_selected_connector_not_just_the_first() {
    let (mut sim, scene, map, conns) = scene_with_connectors(3);
    let xf = xforms(&sim, &map);
    let id = ph2d_editor::ids::VECTOR_CONNECTOR_JETTY;

    let changed = edit_selected_connectors(&mut sim, &map, &conns, id, 0.9);
    assert_eq!(changed, 3, "a edicao tem de atingir os TRES selecionados");
    for &c in &conns {
        let e = Entity::from_bits(*map.get(&c).expect("entidade"));
        let conn = sim
            .world()
            .get::<VecConnector>(e)
            .expect("o componente segue la");
        assert_eq!(
            conn.jetty,
            Some(0.9),
            "o conector {c} ficou para tras — a edicao so pegou o primeiro"
        );
        // E o efetivo publicado passa a ser o FIXADO (não o automático).
        assert!((effective(&scene, &xf, conn).0 - 0.9).abs() < 1e-6);
    }

    // Idem para o Spread e para a Route — os três campos usam o mesmo caminho.
    let n = edit_selected_connectors(
        &mut sim,
        &map,
        &conns,
        ph2d_editor::ids::VECTOR_CONNECTOR_SPREAD,
        -0.4,
    );
    assert_eq!(n, 3);
    let n = edit_selected_connectors(
        &mut sim,
        &map,
        &conns,
        ph2d_editor::ids::VECTOR_CONNECTOR_ROUTE,
        0.0,
    );
    assert_eq!(n, 3);
    for &c in &conns {
        let e = Entity::from_bits(*map.get(&c).expect("entidade"));
        let conn = sim.world().get::<VecConnector>(e).expect("componente");
        assert_eq!(conn.spread, Some(-0.4));
        assert_eq!(conn.route, RouteKind::Straight);
    }
}

/// **Editar FIXA, e o valor fixado NÃO é re-derivado.** O `None` → `Some(v)` é o auto→
/// override de qualquer editor: a partir dele o jetty para de acompanhar o tamanho das
/// caixas. O teste faz a caixa CRESCER depois de fixar — o automático mudaria, o fixado não
/// pode mudar (senão a calibração do Enio evaporaria no primeiro arrasto de uma forma).
#[test]
fn a_pinned_value_stops_following_the_automatic_one() {
    let (mut sim, mut scene, map, conns) = scene_with_connectors(1);
    let xf = xforms(&sim, &map);
    let before = selection_snapshot(&sim, &map, &scene, &xf, &conns)
        .expect("ha conector")
        .1;
    assert!((before - expected_auto_jetty()).abs() < 1e-12);

    // O usuário fixa o jetty no painel.
    assert_eq!(
        edit_selected_connectors(
            &mut sim,
            &map,
            &conns,
            ph2d_editor::ids::VECTOR_CONNECTOR_JETTY,
            0.5,
        ),
        1
    );
    let e = Entity::from_bits(*map.get(&conns[0]).expect("entidade"));
    assert_eq!(
        sim.world()
            .get::<VecConnector>(e)
            .expect("componente")
            .jetty,
        Some(0.5),
        "editar tem de FIXAR o valor (None -> Some)"
    );

    // Agora a forma B cresce MUITO: o automático subiria (satura no teto), o fixado não.
    let b = scene
        .paths()
        .iter()
        .map(|p| p.id)
        .find(|id| !conns.contains(id) && *id != scene.paths()[0].id)
        .expect("a segunda forma");
    let big = rectangle([8.0, 0.0], [40.0, 40.0]);
    if let Some(p) = scene.path_mut(b) {
        p.verts = big.verts;
    }
    let xf = xforms(&sim, &map);
    let after = selection_snapshot(&sim, &map, &scene, &xf, &conns)
        .expect("ha conector")
        .1;
    assert!(
        (after - 0.5).abs() < 1e-6,
        "o valor FIXADO foi re-derivado quando a caixa cresceu: {after} (esperado 0.5)"
    );
    assert!(
        (connector::auto_jetty((1.0, 0.5), (16.0, 20.0)) - 0.5).abs() > 1e-6,
        "precondicao do teste: o AUTOMATICO desta cena e diferente de 0.5, senao ele nao prova nada"
    );
}

/// **O GATE QUE IMPORTA: o número que o painel MOSTRA é o que a linha USA.**
///
/// A derivação do automático tem dois leitores — quem **exibe** (este módulo, via
/// `ph2d_tool_vector::connector`) e quem **coze** a rota (`connector_live`, com as suas
/// `jetty_for` / `SPREAD_STEP` privadas). Divergindo as duas cópias, o painel passa a mostrar
/// um número que **não é** o que desenha a linha: o campo nasce em 0,35, o usuário encosta
/// nele, e a linha PULA. É o bug que a lição `feedback_derived_coordinate_seed_must_match_sample`
/// documenta — o tempo remapeado quebrou três vezes assim, e duas delas passaram no unit test.
///
/// A prova é COMPORTAMENTAL: cozinhamos a rota do conector **automático** (`None`) e depois a
/// do mesmo conector com o valor **que o painel publicou** fixado (`Some`). As duas polilinhas
/// têm de ser IDÊNTICAS — porque fixar o efetivo é, por definição, não mudar nada.
///
/// **A cena tem uma PAREDE de propósito.** Numa rota livre (duas caixas de frente uma para a
/// outra) o A\* dobra no meio do vão e o stub do jetty é colinear com o trecho seguinte — o
/// `finish` o colapsa, e a polilinha sai **idêntica para qualquer jetty**. Um gate montado
/// nessa cena passaria de olhos fechados (foi a primeira versão deste teste, e ela não pegou a
/// mutação `JETTY_K = 0.40`). Com um obstáculo no caminho o desvio huga a caixa inflada por
/// `1,5 × jetty`: aí o jetty ENTRA na geometria, e o gate morde. O `parallel_index = 1` faz o
/// mesmo pelo spread (o automático dele é ±`SPREAD_STEP`, não zero).
#[test]
fn the_value_the_panel_shows_is_the_value_the_cooked_line_actually_uses() {
    /// Duas caixas + uma PAREDE entre elas + um conector de índice 1 (spread automático ≠ 0).
    fn scene_with_a_wall() -> (SimWorld, VecScene, VecEntityMap, VecPathId) {
        let mut sim = SimWorld::default();
        let mut scene = VecScene::new();
        let mut map = VecEntityMap::new();
        let a = scene.push_path(rectangle([0.0, 0.0], [2.0, 2.0]));
        let b = scene.push_path(rectangle([10.0, 0.0], [12.0, 2.0]));
        let _wall = scene.push_path(rectangle([5.0, -3.0], [6.0, 3.0]));
        let c = scene.push_path(ph2d_vec_scene::line([0.0, 0.0], [0.0, 0.0]));
        crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
        let mut conn = VecConnector::between(a, b);
        conn.parallel_index = 1;
        let e = Entity::from_bits(*map.get(&c).expect("o conector tem entidade"));
        sim.world_mut()
            .get_entity_mut(e)
            .expect("a entidade existe")
            .insert(conn);
        (sim, scene, map, c)
    }

    /// A polilinha que o `connector_live` REALMENTE desenha para o conector `c`.
    fn cooked(
        sim: &mut SimWorld,
        scene: &mut VecScene,
        map: &VecEntityMap,
        c: VecPathId,
    ) -> Vec<[f64; 2]> {
        let xf = crate::vec_transform::build(sim, map);
        // Cache NOVO: a histerese do lado de saída é memória de FRAME, e as duas rotas têm de
        // partir do mesmo ponto (senão a 2a herdaria o lado da 1a e o teste mentiria).
        let mut cache = crate::connector_live::SideCache::new();
        crate::connector_live::recook(sim, scene, map, &xf, &mut cache);
        scene
            .paths()
            .iter()
            .find(|p| p.id == c)
            .expect("o path do conector")
            .verts
            .iter()
            .map(|v| v.anchor)
            .collect()
    }

    // 1. O conector AUTOMÁTICO (jetty e spread em `None`) — e o que o painel publica dele.
    let (mut sim, mut scene, map, c) = scene_with_a_wall();
    let xf = xforms(&sim, &map);
    let (_, shown_jetty, shown_spread, _, _curve) =
        selection_snapshot(&sim, &map, &scene, &xf, &[c]).expect("ha conector");
    let auto_line = cooked(&mut sim, &mut scene, &map, c);
    assert!(
        auto_line.len() > 2,
        "precondicao: a rota tem de DESVIAR da parede (senao o jetty nao entra na geometria \
         e este gate nao prova nada): {auto_line:?}"
    );

    // 2. O usuário FIXA exatamente o que o painel exibia (encostou no campo e voltou). Se o
    //    número era a verdade, a linha não pode andar um micrometro.
    let (mut sim, mut scene, map, c) = scene_with_a_wall();
    let e = Entity::from_bits(*map.get(&c).expect("entidade"));
    let mut conn = sim
        .world()
        .get::<VecConnector>(e)
        .expect("componente")
        .clone();
    #[expect(
        clippy::cast_possible_truncation,
        reason = "o painel autora em f64; o componente guarda f32 — a mesma travessia do apply"
    )]
    {
        conn.jetty = Some(shown_jetty as f32);
        conn.spread = Some(shown_spread as f32);
    }
    sim.world_mut()
        .get_entity_mut(e)
        .expect("entidade")
        .insert(conn);
    let pinned_line = cooked(&mut sim, &mut scene, &map, c);

    assert_eq!(
        auto_line.len(),
        pinned_line.len(),
        "fixar o valor EXIBIDO mudou a rota — o painel mostra um numero que a linha nao usa \
         (as constantes de `connector_live` e as de `ph2d_tool_vector::connector` divergiram)"
    );
    for (i, (a, b)) in auto_line.iter().zip(pinned_line.iter()).enumerate() {
        assert!(
            (a[0] - b[0]).abs() < 1e-4 && (a[1] - b[1]).abs() < 1e-4,
            "vertice {i}: a linha PULOU quando o usuario fixou o valor que o painel exibia \
             ({a:?} -> {b:?}) — semente != amostra"
        );
    }
}

/// Uma ponta SOLTA não tem caixa — e a rota continua existindo (o piso do automático). O
/// painel não pode publicar `NaN` nem zero num conector meio-solto.
#[test]
fn a_free_end_still_publishes_a_usable_jetty() {
    let (mut sim, scene, map, conns) = scene_with_connectors(1);
    let e = Entity::from_bits(*map.get(&conns[0]).expect("entidade"));
    let mut conn = sim
        .world()
        .get::<VecConnector>(e)
        .expect("componente")
        .clone();
    conn.end = ConnectorEnd::Free { at: [5.0, 5.0] };
    sim.world_mut()
        .get_entity_mut(e)
        .expect("entidade")
        .insert(conn);

    let xf = xforms(&sim, &map);
    let (_, jetty, _, _, _curve) =
        selection_snapshot(&sim, &map, &scene, &xf, &conns).expect("ha conector");
    assert!(
        jetty.is_finite() && jetty > 0.0,
        "ponta solta publicou um jetty inutil: {jetty}"
    );
    // A caixa que sobrou (a forma A: meias-extensões 1.0 × 0.5) é quem dita.
    assert!((jetty - connector::auto_jetty((1.0, 0.5), (0.0, 0.0))).abs() < 1e-12);
}

/// **O Corner (a quina do PERCURSO) atinge TODOS os conectores selecionados** — e satura na
/// faixa que o painel registra na caixa.
///
/// Mesmo pedido do Jetty/Spread, e mesmo gate: selecionar o diagrama e arredondar as dobras de
/// uma vez. Um `apply` que parasse no primeiro passaria no teste de unidade do componente e
/// falharia na mão do usuário. Diferente dos outros dois, o `corner_radius` **não** é `Option`:
/// não há automático que faça sentido, e `0` (afiado) já é a resposta certa — então editar
/// escreve direto, sem auto→override.
#[test]
fn editing_the_corner_reaches_every_selected_connector_and_clamps() {
    let (mut sim, scene, map, conns) = scene_with_connectors(3);
    let xf = xforms(&sim, &map);
    let id = ph2d_editor::ids::VECTOR_CONNECTOR_CORNER;

    let changed = edit_selected_connectors(&mut sim, &map, &conns, id, 0.4);
    assert_eq!(changed, 3, "a edicao tem de atingir os TRES selecionados");
    for &c in &conns {
        let e = Entity::from_bits(*map.get(&c).expect("entidade"));
        let conn = sim.world().get::<VecConnector>(e).expect("componente");
        assert!(
            (f64::from(conn.corner_radius) - 0.4).abs() < 1e-6,
            "o conector {c} ficou para tras — a edicao so pegou o primeiro"
        );
    }
    // E o painel passa a EXIBIR o que foi escrito (semente = amostra: o campo mostra o que a
    // linha usa, senão o número saltaria no toque seguinte).
    let (_, _, _, corner, _curve) =
        selection_snapshot(&sim, &map, &scene, &xf, &conns).expect("conector");
    assert!((corner - 0.4).abs() < 1e-6, "o painel exibiu {corner}");

    // Um valor fora da faixa (digitado, ou de um save corrompido) satura — nunca vira uma
    // dobra maior que a própria rota.
    edit_selected_connectors(&mut sim, &map, &conns, id, 99.0);
    let e = Entity::from_bits(*map.get(&conns[0]).expect("entidade"));
    let conn = sim.world().get::<VecConnector>(e).expect("componente");
    assert!(
        (f64::from(conn.corner_radius) - connector::CORNER.max).abs() < 1e-6,
        "o Corner nao saturou no teto da faixa: {}",
        conn.corner_radius
    );
    edit_selected_connectors(&mut sim, &map, &conns, id, -5.0);
    let conn = sim.world().get::<VecConnector>(e).expect("componente");
    assert!(
        (f64::from(conn.corner_radius) - connector::CORNER.min).abs() < 1e-6,
        "um raio negativo nao existe: {}",
        conn.corner_radius
    );
}

/// O Corner é um campo de CONECTOR: o dreno da shell tem de reconhecê-lo
/// (`is_connector_field_id`). Fora dessa lista o controle pinta, o clique dispara, o evento
/// viaja — e ninguém o aplica. É o "verde-e-morto" que só um gate no seam pega.
#[test]
fn the_corner_is_recognized_as_a_connector_field_by_the_drain() {
    for id in [
        ph2d_editor::ids::VECTOR_CONNECTOR_ROUTE,
        ph2d_editor::ids::VECTOR_CONNECTOR_JETTY,
        ph2d_editor::ids::VECTOR_CONNECTOR_SPREAD,
        ph2d_editor::ids::VECTOR_CONNECTOR_CORNER,
    ] {
        assert!(
            is_connector_field_id(id),
            "o dreno nao reconhece este campo do conector: o controle nasceria MORTO"
        );
    }
    // E não sequestra ids alheios (o Head Round da SETA não é a quina do PERCURSO).
    assert!(!is_connector_field_id(
        ph2d_editor::ids::VECTOR_MARKER_ROUND
    ));
    assert!(!is_connector_field_id(ph2d_editor::ids::VECTOR_WIDTH));
}
