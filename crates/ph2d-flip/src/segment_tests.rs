//! Gates do domínio SEGMENT (ADR-0114 §4.B). Cada um nomeia a MUTAÇÃO que o derruba —
//! um gate que nenhuma mutação derruba não está guardando nada
//! ([[reference_topic_mutation_proofs]]).
//!
//! Os casos de `piece_of_point` foram conferidos **um a um** contra o
//! `foreach_curve_segment` da referência (Blender 5.2, `grease_pencil_select.cc`), que é
//! quem sabe a resposta certa; a única divergência é a verruga do `clamp_range`, gateada em
//! [`a_cut_on_the_seam_keeps_the_seam_point_in_the_piece_before_it`].

use super::*;

/// Um quadrado FECHADO de 4 pontos. Segmentos (convenção `segments()`): 0 = base, 1 =
/// direita, 2 = topo, 3 = **a costura** (esquerda, `3→0`).
fn square() -> Vec<Cutter> {
    let p = [
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 0.0),
        Vec2::new(10.0, 10.0),
        Vec2::new(0.0, 10.0),
    ];
    (0..4).map(|i| (p[i], p[(i + 1) % 4])).collect()
}

/// Uma linha reta horizontal em `y`, de `x0` a `x1` (o cortador). Corta as arestas
/// VERTICAIS do quadrado (1 e 3) — uma horizontal é colinear com as horizontais, e
/// colinear **não cruza**.
fn h_line(y: f32, x0: f32, x1: f32) -> Cutter {
    (Vec2::new(x0, y), Vec2::new(x1, y))
}

/// O gêmeo vertical: corta as arestas HORIZONTAIS do quadrado (0 e 2).
fn v_line(x: f32, y0: f32, y1: f32) -> Cutter {
    (Vec2::new(x, y0), Vec2::new(x, y1))
}

/// **Um fixture de CURVA** — uma cúbica amostrada (aberta, 13 pontos), NÃO um polígono: o
/// BUGS #18 provou que fixture de polígono esconde bug de curva
/// ([[reference_topic_fixture_discipline]]). Segmentos curtos, nenhum eixo-alinhado, e
/// frações de corte que não são "números redondos".
fn curve_points() -> Vec<Vec2> {
    // Cúbica de Bézier (0,0) → (20,20), controles (14,0) e (20,6): uma curva que dobra.
    // Avaliada por de Casteljau escrito à mão — sem transcendental (HR-5).
    let (p0, p1, p2, p3) = (
        Vec2::new(0.0, 0.0),
        Vec2::new(14.0, 0.0),
        Vec2::new(20.0, 6.0),
        Vec2::new(20.0, 20.0),
    );
    (0..13)
        .map(|i| {
            let t = i as f32 / 12.0;
            let u = 1.0 - t;
            let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
            Vec2::new(
                a * p0.x + b * p1.x + c * p2.x + d * p3.x,
                a * p0.y + b * p1.y + c * p2.y + d * p3.y,
            )
        })
        .collect()
}

/// Os segmentos de uma polilinha ABERTA (a convenção de `segments()`: sem costura).
fn open_segs(p: &[Vec2]) -> Vec<Cutter> {
    p.windows(2).map(|w| (w[0], w[1])).collect()
}

/// Um cortador que atravessa o segmento `(a,b)` EXATAMENTE no meio (λ = 0.5), perpendicular
/// e curto o bastante para não alcançar os vizinhos.
fn stab(a: Vec2, b: Vec2) -> Cutter {
    let m = Vec2::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5);
    let d = b - a;
    let perp = Vec2::new(-d.y, d.x); // mesmo comprimento do segmento
    (
        Vec2::new(m.x - perp.x * 0.3, m.y - perp.y * 0.3),
        Vec2::new(m.x + perp.x * 0.3, m.y + perp.y * 0.3),
    )
}

/// Os pontos que um clique acende: o pedaço do ponto-sonda.
fn lit(cuts: &[Option<f32>], n: usize, closed: bool, i: usize, t: f32) -> Vec<usize> {
    let owner = piece_of_point(cuts, n, closed);
    let probe = probe_point(cuts, n, i, t);
    let want = owner[probe];
    (0..n).filter(|&p| owner[p] == want).collect()
}

// ── os cortes ───────────────────────────────────────────────────────────────────

/// **Mutação que sangra:** não ignorar os 3 vizinhos (remover o `continue` em `cuts`) — aí
/// TODO segmento cruza os vizinhos na ponta que compartilha com eles (`EXACT`), e o traço
/// inteiro vira corte: `[Some, Some, Some, Some]` em vez de `[None; 4]`.
#[test]
fn the_three_neighbours_of_a_segment_never_cut_it() {
    let sq = square();
    let c = cuts(&sq, 0..4, true);
    assert_eq!(
        c,
        vec![None; 4],
        "um quadrado sozinho nao tem corte nenhum: um segmento encostar nos vizinhos \
         nas pontas nao e um cruzamento"
    );
}

/// O irmão de PRESENÇA do gate acima ([[feedback_absence_gate_needs_a_presence_sibling]]):
/// pular vizinho **não pode virar pular tudo**. Um traço que cruza a SI MESMO (um "8") tem
/// de se cortar nos não-vizinhos — senão o gate de ausência ficaria verde com um `cuts` que
/// devolve `None` sempre.
#[test]
fn a_stroke_that_crosses_itself_cuts_itself() {
    // Um "8" aberto: sobe, cruza a própria perna, desce.
    let p = [
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 10.0),
        Vec2::new(0.0, 10.0),
        Vec2::new(10.0, 0.0),
    ];
    let segs = open_segs(&p);
    let c = cuts(&segs, 0..segs.len(), false);
    assert!(
        c.iter().filter(|x| x.is_some()).count() == 2,
        "os dois segmentos NAO-vizinhos que se cruzam (0 e 2) se cortam: {c:?}"
    );
    assert!(c[0].is_some() && c[2].is_some(), "sao o 0 e o 2: {c:?}");
}

/// **Mutação que sangra:** guardar o ÚLTIMO cruzamento em vez do mais próximo (trocar
/// `lambda < x` por `lambda > x`) — o corte sairia em 0.75 e o pedaço iria até o lugar
/// errado. É a limitação HERDADA da referência (um corte por segmento) e ela é *documentada*,
/// não acidental: quem escolhe é o mais PRÓXIMO.
#[test]
fn only_the_nearest_cut_of_a_segment_is_kept() {
    let sq = square();
    let mut all = sq.clone();
    // Dois cortadores atravessam a MESMA aresta direita (segmento 1, de (10,0) a (10,10)):
    // um em y=2.5 (λ=0.25) e outro em y=7.5 (λ=0.75).
    all.push(h_line(2.5, 5.0, 15.0));
    all.push(h_line(7.5, 5.0, 15.0));
    let c = cuts(&all, 0..4, true);
    assert_eq!(
        c[1],
        Some(0.25),
        "fica o mais PROXIMO do inicio do segmento"
    );
}

// ── os pedaços (o vetor de donos) ───────────────────────────────────────────────

/// O **fallback** do `§11`, e o caso comum do balde (o balde produz traço FECHADO).
///
/// **Mutação que sangra:** tirar o fallback — na forma do vetor de donos, "tirar o
/// fallback" é fazer `piece_of_point` devolver algo que não seja um pedaço só; qualquer
/// coisa que quebre isso deixa um clique numa forma fechada intacta **sem acender nada**.
#[test]
fn a_closed_stroke_with_no_cuts_is_one_piece_the_whole_curve() {
    let sq = square();
    let c = cuts(&sq, 0..4, true);
    let owner = piece_of_point(&c, 4, true);
    assert_eq!(owner, vec![0; 4], "sem corte: a curva toda e UM pedaco");
    assert_eq!(
        lit(&c, 4, true, 2, 0.5),
        vec![0, 1, 2, 3],
        "clicar em qualquer aresta acende a forma inteira"
    );
}

/// Os "**DOIS ranges**" do `§11`: o pedaço que enrola aparece como duas corridas no vetor
/// de donos. Quadrado cortado nos segmentos 0 (base) e 2 (topo).
///
/// **Mutação que sangra:** não dar a volta ao semear o dono do ponto 0 (trocar
/// `last_cut.map_or(0, …)` por `0`) — o pedaço que enrola se parte em dois e o clique na
/// aresta esquerda deixaria de acender o ponto 0.
#[test]
fn the_piece_that_wraps_the_seam_lights_both_ranges() {
    let mut all = square();
    all.push(v_line(5.0, -5.0, 5.0)); // corta a base (segmento 0) em λ=0.5
    all.push(v_line(5.0, 5.0, 15.0)); // corta o topo (segmento 2) em λ=0.5
    let c = cuts(&all, 0..4, true);
    assert_eq!(c, vec![Some(0.5), None, Some(0.5), None], "dois cortes");
    let owner = piece_of_point(&c, 4, true);
    assert_eq!(owner, vec![1, 0, 0, 1], "o pedaco 1 = {{3,0}} — ENROLA");
    // Conferido contra a referência: range1={0}, range2={3} → o mesmo conjunto.
    assert_eq!(
        lit(&c, 4, true, 3, 0.5),
        vec![0, 3],
        "clicar na costura acende os dois lados dela"
    );
    assert_eq!(
        lit(&c, 4, true, 1, 0.5),
        vec![1, 2],
        "e o outro pedaco e o complemento"
    );
}

/// **A divergência deliberada da referência** (ver o doc do módulo): com o corte na
/// COSTURA, o `clamp_range` do Blender satura `3+1` em `3` e entrega o ponto 3 ao pedaço
/// errado (`{2}` + `{0,1,3}`). Aqui o `+1` ENROLA e a geometria manda: o ponto 3 está antes
/// do corte em `mid(3→0)`, logo é do pedaço que vem antes dele.
///
/// **Mutação que sangra:** trocar o `(i + 1) % n_points` do `probe_point`/`id_of_cut` por
/// uma saturação — volta a ser a resposta do Blender, e este gate fica vermelho.
#[test]
fn a_cut_on_the_seam_keeps_the_seam_point_in_the_piece_before_it() {
    let mut all = square();
    all.push(h_line(5.0, 5.0, 15.0)); // corta a direita (segmento 1) em λ=0.5
    all.push(h_line(5.0, -5.0, 5.0)); // corta a COSTURA (segmento 3) em λ=0.5
    let c = cuts(&all, 0..4, true);
    assert_eq!(
        c,
        vec![None, Some(0.5), None, Some(0.5)],
        "corte na costura"
    );
    let owner = piece_of_point(&c, 4, true);
    assert_eq!(
        owner,
        vec![1, 1, 0, 0],
        "o ponto 3 e do pedaco 0 ({{2,3}}), NAO do que enrola — o Blender diria {{0,1,3}}"
    );
    assert_eq!(lit(&c, 4, true, 2, 0.5), vec![2, 3], "topo → {{2,3}}");
    assert_eq!(lit(&c, 4, true, 0, 0.5), vec![0, 1], "base → {{0,1}}");
}

/// Um corte EM CIMA de um ponto (`λ = 0`) **inclui** aquele ponto no pedaço novo — o
/// *"start point with zero fraction is included"* da referência. Não é hipótese de
/// laboratório: arte eixo-alinhada produz λ exato o tempo todo.
///
/// Repare no `c` esperado: um cortador que passa por um VÉRTICE acerta os **dois**
/// segmentos que ali se encontram (o de trás no `λ=1`, o da frente no `λ=0`) — são dois
/// cortes no MESMO lugar, e o pedaço entre eles nasce vazio. Vazio é a resposta certa
/// (ninguém consegue clicar num pedaço sem extensão) e é o que mantém o resto correto.
///
/// **Mutação que sangra:** tratar `λ = 0` como `λ > 0` (remover o braço `cut == Some(0.0)`)
/// — o ponto 2 cairia no pedaço anterior e `owner[2] == owner[3]` fica vermelho.
#[test]
fn a_cut_exactly_on_a_point_puts_that_point_in_the_new_piece() {
    let p: Vec<Vec2> = (0..4).map(|i| Vec2::new(i as f32 * 10.0, 0.0)).collect();
    let segs = open_segs(&p);
    let mut all = segs.clone();
    all.push(v_line(20.0, -5.0, 5.0)); // passa EM CIMA do ponto 2 = (20,0)
    let c = cuts(&all, 0..segs.len(), false);
    assert_eq!(
        c,
        vec![None, Some(1.0), Some(0.0)],
        "o vertice e ponta de um segmento e comeco do outro"
    );
    let owner = piece_of_point(&c, 4, false);
    assert_eq!(
        owner,
        vec![0, 0, 2, 2],
        "o ponto 2 abre o pedaco novo (o 1 nasce vazio: os dois cortes sao o mesmo lugar)"
    );
    assert_eq!(lit(&c, 4, false, 2, 0.5), vec![2, 3], "e o clique concorda");
}

/// Traço ABERTO: as pontas são cortes implícitos, e sem corte nenhum ele é um pedaço só.
/// Conferido contra a referência (`starts = [(0, 0.0)]` ⇒ 1 segmento ⇒ a curva toda).
#[test]
fn an_open_stroke_is_one_piece_until_something_cuts_it() {
    let p: Vec<Vec2> = (0..4).map(|i| Vec2::new(i as f32 * 10.0, 0.0)).collect();
    let segs = open_segs(&p);
    let c = cuts(&segs, 0..segs.len(), false);
    assert_eq!(c, vec![None; 3], "nada o cruza");
    assert_eq!(piece_of_point(&c, 4, false), vec![0; 4], "um pedaco so");

    // Agora um cortador atravessa o segmento 1 (de (10,0) a (20,0)) em x=15 ⇒ λ=0.5.
    let mut all = segs.clone();
    all.push((Vec2::new(15.0, -5.0), Vec2::new(15.0, 5.0)));
    let c = cuts(&all, 0..segs.len(), false);
    assert_eq!(c, vec![None, Some(0.5), None], "um corte no meio");
    assert_eq!(
        piece_of_point(&c, 4, false),
        vec![0, 0, 1, 1],
        "a cabeca {{0,1}} e a cauda {{2,3}} — conferido com a referencia"
    );
}

// ── o clique ────────────────────────────────────────────────────────────────────

/// **O gate central do §4.B**: um clique num segmento entre 2 cortes acende **só aquele
/// pedaço**.
///
/// **Mutação que sangra:** ignorar os cortes (fazer `cuts` devolver `vec![None; m]`) — aí
/// tudo vira um pedaço só e o clique acende o traço INTEIRO, que é exatamente o
/// comportamento do domínio Stroke que este modo existe para não ser.
#[test]
fn a_click_between_two_cuts_lights_only_that_piece() {
    // Uma linha reta horizontal de 5 pontos (x = 0,10,20,30,40), cortada 2×.
    let p: Vec<Vec2> = (0..5).map(|i| Vec2::new(i as f32 * 10.0, 0.0)).collect();
    let segs = open_segs(&p);
    let mut all = segs.clone();
    all.push((Vec2::new(5.0, -5.0), Vec2::new(5.0, 5.0))); // corta o segmento 0
    all.push((Vec2::new(35.0, -5.0), Vec2::new(35.0, 5.0))); // corta o segmento 3
    let c = cuts(&all, 0..segs.len(), false);
    assert_eq!(c, vec![Some(0.5), None, None, Some(0.5)], "dois cortes");
    assert_eq!(
        lit(&c, 5, false, 2, 0.5),
        vec![1, 2, 3],
        "o miolo acende so o miolo"
    );
    assert_eq!(
        lit(&c, 5, false, 0, 0.2),
        vec![0],
        "antes do 1o corte: so o 0"
    );
    assert_eq!(
        lit(&c, 5, false, 3, 0.9),
        vec![4],
        "depois do ultimo corte: so o 4"
    );
}

/// O clique no MESMO segmento cai em pedaços diferentes conforme o lado do corte.
///
/// **Mutação que sangra:** o `probe_point` devolver sempre `i` (ignorar o `t > lambda`) —
/// clicar depois do corte acenderia o pedaço de antes dele.
#[test]
fn the_side_of_the_cut_the_click_fell_on_decides_the_piece() {
    let p: Vec<Vec2> = (0..3).map(|i| Vec2::new(i as f32 * 10.0, 0.0)).collect();
    let segs = open_segs(&p);
    let mut all = segs.clone();
    all.push((Vec2::new(5.0, -5.0), Vec2::new(5.0, 5.0))); // corta o segmento 0 em λ=0.5
    let c = cuts(&all, 0..segs.len(), false);
    assert_eq!(lit(&c, 3, false, 0, 0.2), vec![0], "antes do corte");
    assert_eq!(lit(&c, 3, false, 0, 0.8), vec![1, 2], "depois do corte");
}

// ── a curva (o fixture que o polígono esconderia) ───────────────────────────────

/// O mesmo contrato sobre uma **CURVA** amostrada: segmentos curtos, nenhum eixo-alinhado.
/// O BUGS #18 mostrou que fixture de polígono esconde bug de curva — e a lasca do Vector
/// mostrou o mesmo. Aqui: dois cortes ⇒ três pedaços que PARTICIONAM os pontos, e o clique
/// no do meio acende só ele.
#[test]
fn a_sampled_curve_cuts_into_pieces_that_partition_its_points() {
    let p = curve_points();
    let n = p.len();
    let segs = open_segs(&p);
    let mut all = segs.clone();
    all.push(stab(segs[2].0, segs[2].1)); // atravessa o segmento 2 em λ=0.5
    all.push(stab(segs[8].0, segs[8].1)); // atravessa o segmento 8 em λ=0.5
    let c = cuts(&all, 0..segs.len(), false);
    let hits: Vec<usize> = c
        .iter()
        .enumerate()
        .filter(|(_, x)| x.is_some())
        .map(|(i, _)| i)
        .collect();
    assert_eq!(hits, vec![2, 8], "so os dois segmentos apunhalados: {c:?}");

    let owner = piece_of_point(&c, n, false);
    let ids: std::collections::BTreeSet<u32> = owner.iter().copied().collect();
    assert_eq!(ids.len(), 3, "tres pedacos: {owner:?}");
    // Particiona: todo ponto tem dono, e os pedaços são contíguos num traço aberto.
    assert_eq!(owner, vec![0, 0, 0, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2]);
    assert_eq!(
        lit(&c, n, false, 5, 0.5),
        vec![3, 4, 5, 6, 7, 8],
        "o pedaco do meio, e so ele"
    );
}
