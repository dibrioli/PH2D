//! Testes de `marker.rs` — arquivo irmao (teto de LOC).

use super::*;
use crate::VertexKind;

/// Uma linha horizontal simples, da esquerda para a direita.
fn line(x0: f64, x1: f64) -> VecPath {
    VecPath {
        verts: vec![VecVertex::corner([x0, 0.0]), VecVertex::corner([x1, 0.0])],
        closed: false,
        ..VecPath::default()
    }
}

/// As pontas que FECHAM REGIÃO — as únicas em que a linha recua, e portanto as únicas em que
/// um descasamento entre a cabeça e o recuo aparece na tela.
const REGION_HEADS: &[Marker] = &[
    Marker::Triangle,
    Marker::Diamond,
    Marker::DiamondOpen,
    Marker::Circle,
    Marker::CircleOpen,
];

/// O CONTORNO real, amostrado — as cúbicas, não só as âncoras. Um filete é um arco: medir a
/// ponta pelas âncoras deixaria o arco (a parte que de fato aparece) fora da conta.
fn outline(p: &VecPath) -> Vec<[f64; 2]> {
    const STEPS: usize = 32;
    let n = p.verts.len();
    let segs = if p.closed { n } else { n.saturating_sub(1) };
    let mut out = Vec::with_capacity(segs * (STEPS + 1));
    for i in 0..segs {
        let (a, b) = (&p.verts[i], &p.verts[(i + 1) % n]);
        let (p0, p1, p2, p3) = (a.anchor, a.out_handle, b.in_handle, b.anchor);
        for k in 0..=STEPS {
            let u = k as f64 / STEPS as f64;
            let v = 1.0 - u;
            let (b0, b1) = (v * v * v, 3.0 * v * v * u);
            let (b2, b3) = (3.0 * v * u * u, u * u * u);
            out.push([
                b0 * p0[0] + b1 * p1[0] + b2 * p2[0] + b3 * p3[0],
                b0 * p0[1] + b1 * p1[1] + b2 * p2[1] + b3 * p3[1],
            ]);
        }
    }
    out
}

/// A PROFUNDIDADE da cabeça: o quanto o contorno avança para TRÁS do bico, ao longo do eixo.
/// É a mesma grandeza que o [`Marker::inset`] promete — vista do outro lado.
fn head_depth(p: &VecPath, tip: [f64; 2], dir: [f64; 2]) -> f64 {
    outline(p)
        .iter()
        .map(|q| -((q[0] - tip[0]) * dir[0] + (q[1] - tip[1]) * dir[1]))
        .fold(f64::MIN, f64::max)
}

/// Área (com sinal) do polígono das âncoras — o shoelace.
fn anchor_area(p: &VecPath) -> f64 {
    let n = p.verts.len();
    let mut s = 0.0;
    for i in 0..n {
        let a = p.verts[i].anchor;
        let b = p.verts[(i + 1) % n].anchor;
        s += a[0] * b[1] - b[0] * a[1];
    }
    s * 0.5
}

/// **A ponta olha para FORA.** É a asserção que separa uma seta de uma seta ao contrário —
/// e ela é fácil de errar por um sinal, exatamente como o espelhamento vertical foi.
#[test]
fn the_arrowhead_points_away_from_the_line_not_into_it() {
    let l = line(0.0, 10.0);

    let (tip, dir) = end_tangent(&l, false).expect("a linha tem fim");
    assert_eq!(tip, [10.0, 0.0], "o fim da linha e a ponta");
    assert!(
        (dir[0] - 1.0).abs() < 1e-9 && dir[1].abs() < 1e-9,
        "no FIM, a tangente aponta para +x (para fora): {dir:?}"
    );

    let (tip0, dir0) = end_tangent(&l, true).expect("a linha tem comeco");
    assert_eq!(tip0, [0.0, 0.0]);
    assert!(
        (dir0[0] + 1.0).abs() < 1e-9,
        "no COMECO, a tangente aponta para -x (tambem para fora): {dir0:?}"
    );

    // E a geometria segue a direção: o triângulo do fim tem o bico no extremo, e o corpo
    // dele fica ATRÁS (do lado da linha), nunca à frente.
    let t = Marker::Triangle
        .build(tip, dir, 1.0, 1.0, 0.0)
        .expect("o triangulo existe");
    let apex = t.verts[0].anchor;
    assert_eq!(apex, tip, "o bico do triangulo E a ponta da linha");
    for v in &t.verts[1..] {
        assert!(
            v.anchor[0] < tip[0],
            "a base do triangulo tem de ficar ATRAS do bico: {:?}",
            v.anchor
        );
    }
}

/// **A ponta escala com a largura do traço** (o `markerUnits = "strokeWidth"` do SVG). Sem
/// isso, uma linha grossa terminaria num alfinete.
#[test]
fn the_marker_scales_with_the_stroke_width() {
    let tip = [0.0, 0.0];
    let dir = [1.0, 0.0];
    let thin = Marker::Triangle
        .build(tip, dir, 1.0, 1.0, 0.0)
        .expect("fino");
    let fat = Marker::Triangle
        .build(tip, dir, 3.0, 1.0, 0.0)
        .expect("grosso");

    let span = |p: &VecPath| {
        let ys: Vec<f64> = p.verts.iter().map(|v| v.anchor[1]).collect();
        ys.iter().copied().fold(f64::MIN, f64::max) - ys.iter().copied().fold(f64::MAX, f64::min)
    };
    assert!(
        (span(&fat) - 3.0 * span(&thin)).abs() < 1e-9,
        "triplicar a largura do traco tem de triplicar a ponta: {} vs {}",
        span(&fat),
        span(&thin)
    );
}

/// **A linha recua para caber a ponta.** Sem o recuo, o traço aparece por dentro de uma
/// ponta vazada e faz uma bolha na base de uma cheia.
#[test]
fn the_line_is_shortened_to_make_room_for_the_head() {
    let l = line(0.0, 10.0);
    let inset = Marker::Triangle.inset(1.0); // em múltiplos da largura
    let w = 1.0;
    let t = trim_path(&l, 0.0, inset * w).expect("sobra linha");
    let end = t.verts.last().expect("tem vertices").anchor;
    assert!(
        (end[0] - (10.0 - inset)).abs() < 1e-9,
        "a linha tinha de parar em {}, parou em {}",
        10.0 - inset,
        end[0]
    );
    // O começo não se mexeu.
    assert_eq!(t.verts[0].anchor, [0.0, 0.0]);
}

/// As pontas SEM região (a aberta e a barra) não recuam nada: não há nada para esconder o
/// traço, e recuar deixaria um vão visível entre a linha e a ponta.
#[test]
fn open_heads_do_not_shorten_the_line() {
    assert_eq!(Marker::Open.inset(1.0), 0.0);
    assert_eq!(Marker::Bar.inset(1.0), 0.0);
    assert_eq!(Marker::None.inset(1.0), 0.0);
    // Já as que fecham região recuam — e o losango, que tem o dobro do comprimento, recua o
    // dobro do triângulo.
    assert!(Marker::Triangle.inset(1.0) > 0.0);
    assert!(
        (Marker::Diamond.inset(1.0) - 2.0 * Marker::Triangle.inset(1.0)).abs() < 1e-9,
        "o losango tem o dobro do comprimento, logo o dobro do recuo"
    );
}

/// Uma linha **mais curta que os recuos** não vira uma linha invertida: vira nenhuma linha.
/// (O caso do usuário que arrasta um conector de 2 px com uma seta gorda.)
#[test]
fn a_line_shorter_than_its_heads_yields_no_line_instead_of_an_inverted_one() {
    let tiny = line(0.0, 1.0);
    assert!(
        trim_path(&tiny, 4.0, 4.0).is_none(),
        "recuar 8 de uma linha de 1 tem de dar NENHUMA linha, nao uma linha ao contrario"
    );
    // E uma que sobra, sobra de verdade.
    assert!(trim_path(&line(0.0, 20.0), 4.0, 4.0).is_some());
}

/// A ponta redonda é um CÍRCULO (quatro cúbicas exatas), não um polígono — a mesma regra do
/// resto do catálogo.
#[test]
fn the_round_head_is_a_real_circle() {
    let c = Marker::Circle
        .build([0.0, 0.0], [1.0, 0.0], 1.0, 1.0, 0.0)
        .expect("o circulo existe");
    assert_eq!(c.verts.len(), 4, "quatro cubicas de 90 graus");
    assert!(c.verts.iter().all(|v| v.kind != VertexKind::Corner));
    // Ele encosta na ponta da linha (tangente ao extremo), não a ultrapassa.
    let max_x = c.verts.iter().map(|v| v.anchor[0]).fold(f64::MIN, f64::max);
    assert!(max_x <= 1e-9, "o circulo nao passa da ponta: {max_x}");
}

/// **A ponta segue a CURVA, não a corda.** Numa curva bem encurvada as duas divergem
/// visivelmente, e uma seta alinhada pela corda sai torta.
#[test]
fn the_head_follows_the_curves_tangent_not_the_chord() {
    // Um quarto de círculo: começa em (0,0) indo para +y, termina em (1,1) indo para +x.
    let path = VecPath {
        verts: vec![
            VecVertex::smooth([0.0, 0.0], [0.0, 0.0], [0.0, 0.55]),
            VecVertex::smooth([1.0, 1.0], [0.45, 1.0], [1.0, 1.0]),
        ],
        closed: false,
        ..VecPath::default()
    };
    let (_, dir) = end_tangent(&path, false).expect("tem fim");
    // A tangente no fim é ~(+1, 0). A CORDA seria ~(0.707, 0.707) — 45° de erro.
    assert!(
        dir[0] > 0.95 && dir[1].abs() < 0.2,
        "a ponta tem de seguir a tangente (~+x), nao a corda (~45 graus): {dir:?}"
    );
}

/// Todo discriminante sobrevive ao round-trip pelo documento, e um desconhecido (save de uma
/// versão futura) resolve para `None` em vez de entrar em pânico.
#[test]
fn every_marker_round_trips_and_an_unknown_one_degrades_gracefully() {
    for &m in ALL_MARKERS {
        assert_eq!(Marker::from_u8(m.as_u8()), Some(m));
    }
    assert_eq!(
        Marker::from_u8(200),
        None,
        "uma ponta do futuro nao entra em panico"
    );
}

// ─── `scale`: o tamanho da cabeça ────────────────────────────────────────────────────────

/// **A cabeça cresce com o `scale` — e o RECUO cresce na mesma proporção.**
///
/// O recuo é o quanto a linha se encolhe para não aparecer por dentro da ponta. Se a cabeça
/// dobra e o recuo não, o traço reaparece ATRAVESSANDO a seta. Este é o bug óbvio da feature,
/// e é por isso que o `scale` é obrigatório nas DUAS funções.
#[test]
fn a_bigger_scale_grows_the_head_and_the_inset_by_the_same_factor() {
    let (tip, dir, w) = ([0.0, 0.0], [1.0, 0.0], 2.0);
    for &m in REGION_HEADS {
        let one = m.build(tip, dir, w, 1.0, 0.0).expect("a ponta existe");
        for scale in [0.5, 2.0, 3.5] {
            let big = m.build(tip, dir, w, scale, 0.0).expect("a ponta existe");
            // 1) a GEOMETRIA cresce: a caixa da ponta multiplica por `scale`.
            let bbox = |p: &VecPath| {
                let o = outline(p);
                let ext = |f: fn(&[f64; 2]) -> f64| {
                    let lo = o.iter().map(f).fold(f64::MAX, f64::min);
                    let hi = o.iter().map(f).fold(f64::MIN, f64::max);
                    hi - lo
                };
                [ext(|q| q[0]), ext(|q| q[1])]
            };
            let (b1, bs) = (bbox(&one), bbox(&big));
            for k in 0..2 {
                assert!(
                    (bs[k] - b1[k] * scale).abs() < 1e-9,
                    "{m:?} eixo {k}: a caixa tinha de ir a {}, foi a {}",
                    b1[k] * scale,
                    bs[k]
                );
            }
            // 2) o RECUO cresce junto — na MESMA proporção, ou a linha atravessa a seta.
            assert!(
                (m.inset(scale) - m.inset(1.0) * scale).abs() < 1e-9,
                "{m:?}: o recuo tinha de escalar com a cabeça"
            );
        }
    }
}

/// **A linha para EXATAMENTE nas costas da cabeça — em todo `scale` e em todo `round`.**
///
/// É o gate-mãe desta feature: `inset(scale)` (o recuo do traço) e a profundidade da ponta
/// construída são a MESMA medida vista dos dois lados. Divergir por excesso = a linha
/// reaparece atravessando a seta; por falta = um VÃO entre o fim do traço e a cabeça.
///
/// Note que o `round` **não** entra no `inset` — por contrato ele depende só do `scale`. É
/// por isso que a quina de junção do losango não arredonda: se arredondasse, a traseira dele
/// recuaria e este gate ficaria vermelho (medido: 1.38·w a `round` 1.0).
#[test]
fn the_line_always_stops_exactly_at_the_back_of_the_head() {
    let (tip, w) = ([3.0, -7.0], 2.0);
    for &m in REGION_HEADS {
        for dir in [[1.0, 0.0], [0.0, -1.0], [0.6, 0.8]] {
            for scale in [0.5, 1.0, 2.5] {
                for round in [0.0, 0.25, 0.5, 1.0] {
                    let geo = m.build(tip, dir, w, scale, round).expect("a ponta existe");
                    let depth = head_depth(&geo, tip, dir);
                    let recuo = m.inset(scale) * w;
                    // Tolerância RELATIVA e apertada (0.1%): o círculo é quatro cúbicas
                    // `KAPPA`, cuja aproximação faz um bojo conhecido de +2.7e-4·r para fora —
                    // não é folga de projeto, é o erro do arco. Os vãos que este gate caça são
                    // de OUTRA ordem (o filete na quina de junção do losango recuaria a
                    // traseira em 4% do recuo já a `round` 0.25; ignorar o `scale` erra 150%).
                    assert!(
                        (depth - recuo).abs() < 1e-3 * recuo,
                        "{m:?} (scale {scale}, round {round}): a linha para a {recuo} do bico, \
                         mas a cabeça vai ate {depth} — sobra vao ou o traco atravessa a seta"
                    );
                }
            }
        }
    }
}

// ─── `round`: as quinas arredondadas ─────────────────────────────────────────────────────

/// **`round = 0` é a IDENTIDADE, byte a byte.** As pontas afiadas de hoje não podem mudar de
/// forma por causa de um parâmetro novo. Golden literal: a geometria de `tip = (0,0)`,
/// `dir = +x`, `w = 1` — os números são exatos em binário, então a igualdade é EXATA.
#[test]
fn zero_round_is_byte_identical_to_the_sharp_head() {
    let (tip, dir) = ([0.0, 0.0], [1.0, 0.0]);
    let sharp = |m: Marker| m.build(tip, dir, 1.0, 1.0, 0.0).expect("a ponta existe");

    let anchors = |m: Marker| -> Vec<[f64; 2]> {
        let p = sharp(m);
        // Toda ponta afiada é um polígono CRU: quina, handles nulos.
        for v in &p.verts {
            assert_eq!(v.kind, VertexKind::Corner, "{m:?}: quina viva");
            assert_eq!(v.in_handle, v.anchor, "{m:?}: handle de entrada nulo");
            assert_eq!(v.out_handle, v.anchor, "{m:?}: handle de saida nulo");
        }
        p.verts.iter().map(|v| v.anchor).collect()
    };
    assert_eq!(
        anchors(Marker::Triangle),
        vec![[0.0, 0.0], [-4.0, 2.0], [-4.0, -2.0]]
    );
    assert_eq!(
        anchors(Marker::Diamond),
        vec![[0.0, 0.0], [-4.0, 2.0], [-8.0, 0.0], [-4.0, -2.0]]
    );
    assert_eq!(
        anchors(Marker::Open),
        vec![[-4.0, 2.0], [0.0, 0.0], [-4.0, -2.0]]
    );
    assert_eq!(anchors(Marker::Bar), vec![[0.0, 2.0], [0.0, -2.0]]);
    assert!(sharp(Marker::Triangle).closed && !sharp(Marker::Open).closed);
    assert!(!sharp(Marker::Bar).closed && sharp(Marker::Diamond).closed);
}

/// **Com `round > 0` a ponta deixa de ter quina viva:** cada quina vira DOIS vértices ligados
/// por uma cúbica, e nenhum deles é um canto puro (o lado do arco tem handle não-degenerado).
/// Um vértice com os dois handles nulos = a quina continua afiada.
#[test]
fn a_positive_round_leaves_no_live_corner_on_the_head() {
    let (tip, dir) = ([0.0, 0.0], [1.0, 0.0]);
    // O triângulo: as TRÊS quinas arredondam (3 -> 6 vertices).
    let t = Marker::Triangle
        .build(tip, dir, 1.0, 1.0, 0.5)
        .expect("a ponta existe");
    assert_eq!(t.verts.len(), 6, "3 quinas x 2 vertices");
    for v in &t.verts {
        let live = v.in_handle == v.anchor && v.out_handle == v.anchor;
        assert!(!live, "quina viva sobrou no triangulo arredondado: {v:?}");
    }
    // O losango: TRÊS quinas arredondam; a de JUNÇÃO (a traseira, onde a linha encosta) fica
    // viva de propósito — ver `the_line_always_stops_exactly_at_the_back_of_the_head`.
    let d = Marker::Diamond
        .build(tip, dir, 1.0, 1.0, 0.5)
        .expect("a ponta existe");
    assert_eq!(d.verts.len(), 7, "3 quinas x 2 + a de juncao, crua");
    let live: Vec<[f64; 2]> = d
        .verts
        .iter()
        .filter(|v| v.in_handle == v.anchor && v.out_handle == v.anchor)
        .map(|v| v.anchor)
        .collect();
    assert_eq!(
        live,
        vec![[-8.0, 0.0]],
        "so a quina de juncao pode ficar viva"
    );
    // O "V" aberto: o BICO arredonda; os dois extremos são pontas de traço, não quinas.
    let o = Marker::Open
        .build(tip, dir, 1.0, 1.0, 0.5)
        .expect("a ponta existe");
    assert_eq!(o.verts.len(), 4, "o bico virou dois vertices");
    assert!(o.verts[1].out_handle != o.verts[1].anchor, "o bico curvou");
    assert!(o.verts[2].in_handle != o.verts[2].anchor);
}

/// **Uma ponta REDONDA não tem quina** — o `round` não a afeta, e isso é o certo: não há o que
/// arredondar, e forçar algo ali seria inventar. A `Bar` é um risco reto: idem.
#[test]
fn a_round_head_has_no_corner_to_round() {
    let (tip, dir) = ([0.0, 0.0], [1.0, 0.0]);
    for m in [Marker::Circle, Marker::CircleOpen, Marker::Bar] {
        let sharp = m.build(tip, dir, 1.0, 1.0, 0.0).expect("a ponta existe");
        for round in [0.5, 1.0] {
            let r = m.build(tip, dir, 1.0, 1.0, round).expect("a ponta existe");
            assert_eq!(r.verts, sharp.verts, "{m:?} nao muda com o round");
        }
    }
}

/// **O filete nunca come mais que METADE da aresta.** Sem esse teto ele atravessaria a aresta
/// que duas quinas vizinhas compartilham: a ponta se auto-cruzaria e a seta viraria uma
/// pastilha. Com `round = 1.0` (o máximo) numa ponta de arestas curtas, a silhueta tem de
/// continuar convexa, de área POSITIVA, e menor que a afiada (o filete só TIRA material).
#[test]
fn the_fillet_never_eats_more_than_half_an_edge() {
    let (tip, dir) = ([0.0, 0.0], [1.0, 0.0]);
    let sharp = Marker::Triangle
        .build(tip, dir, 1.0, 1.0, 0.0)
        .expect("a ponta existe");
    let blob = Marker::Triangle
        .build(tip, dir, 1.0, 1.0, 1.0)
        .expect("a ponta existe");

    // 1) o recuo de cada âncora a partir da quina que a gerou ≤ **meia-aresta ADJACENTE** (o
    //    teto é POR-QUINA: o bico tem duas arestas de 4.472, as da base têm uma de 4.0).
    //    A proveniência é exata, não adivinhada: `filleted` emite os dois vértices da quina
    //    `i` em `verts[2i]` e `verts[2i+1]` — adivinhar pela âncora mais próxima deixaria a
    //    mutação (comer a aresta INTEIRA) passar, porque aí o recuo pousa em cima do vizinho.
    let corners = [[0.0, 0.0], [-4.0, 2.0], [-4.0, -2.0]];
    assert_eq!(blob.verts.len(), corners.len() * 2);
    for (i, c) in corners.iter().enumerate() {
        let (prev, next) = (corners[(i + 2) % 3], corners[(i + 1) % 3]);
        let edge = |q: [f64; 2]| (q[0] - c[0]).hypot(q[1] - c[1]);
        let cap = 0.5 * edge(prev).min(edge(next));
        for v in &blob.verts[i * 2..i * 2 + 2] {
            let d = (v.anchor[0] - c[0]).hypot(v.anchor[1] - c[1]);
            assert!(
                d <= cap + 1e-9,
                "a ancora {:?} recuou {d} da quina {c:?} — passou de meia-aresta ({cap})",
                v.anchor
            );
        }
    }
    // 2) a silhueta continua CONVEXA (o filete corta para dentro; ele nunca cruza). Se a
    //    aresta fosse comida por inteiro, os vértices se ultrapassariam e o sinal viraria.
    let n = blob.verts.len();
    let mut sign = 0.0_f64;
    for i in 0..n {
        let (a, b, c) = (
            blob.verts[i].anchor,
            blob.verts[(i + 1) % n].anchor,
            blob.verts[(i + 2) % n].anchor,
        );
        let cross = (b[0] - a[0]) * (c[1] - b[1]) - (b[1] - a[1]) * (c[0] - b[0]);
        if cross.abs() < 1e-9 {
            continue; // colinear: os dois recuos de uma mesma aresta
        }
        if sign == 0.0 {
            sign = cross.signum();
        }
        assert_eq!(
            cross.signum(),
            sign,
            "a silhueta se dobrou: o filete comeu a aresta inteira"
        );
    }
    // 3) área POSITIVA e reconhecível: o filete só TIRA material (nunca cresce), mas o que
    //    sobra ainda é uma seta, não um confete.
    let (a_sharp, a_blob) = (anchor_area(&sharp).abs(), anchor_area(&blob).abs());
    assert!(
        a_blob > 0.2 * a_sharp,
        "a ponta virou confete: {a_blob} de {a_sharp}"
    );
    assert!(
        a_blob < a_sharp,
        "o filete CRESCEU a ponta ({a_blob} > {a_sharp}) — ele se auto-cruzou"
    );
}

/// O filete daqui é o MESMO de [`crate::corners`] — a construção canônica da crate —, só que
/// parametrizado por fração em vez de raio. Prova NUMÉRICA da concordância (não de boca): num
/// canto reto `t = r`, então filetar um quadrado por `round` tem de dar exatamente o que o
/// `round_closed_corners` dá com raio `r = round · meia-aresta`.
#[test]
fn the_fillet_agrees_with_the_crates_canonical_corner_rounding() {
    let sq = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
    for round in [0.25, 0.5, 1.0] {
        let mine = filleted(&sq, true, &[round; 4]);
        let theirs = crate::corners::round_closed_corners(&sq, &[round * 5.0; 4]);
        assert_eq!(mine.verts.len(), theirs.verts.len());
        for (a, b) in mine.verts.iter().zip(&theirs.verts) {
            for (x, y) in [
                (a.anchor, b.anchor),
                (a.in_handle, b.in_handle),
                (a.out_handle, b.out_handle),
            ] {
                assert!(
                    (x[0] - y[0]).abs() < 1e-12 && (x[1] - y[1]).abs() < 1e-12,
                    "round {round}: {x:?} != {y:?} — a math do filete divergiu do corners.rs"
                );
            }
        }
    }
}

/// **A seta continua apontando para o ALVO depois de escalar e arredondar.** Duas metades:
///
/// 1. a ponta é EQUIVARIANTE à direção — girar `dir` gira a cabeça, não a deforma;
/// 2. a cabeça fica INTEIRA atrás do bico (nada dela avança na frente do alvo), e mesmo
///    arredondada ela ainda ALCANÇA a frente (o filete embota o bico, não o decapita).
///
/// A metade (2) vale para TODA ponta; a (1), só para as poligonais: as âncoras do círculo são
/// autoradas nos eixos do MUNDO (`+x`, `+y`, …), então comparar vértice a vértice ali mediria
/// a contabilidade das âncoras, não a geometria — e um círculo é rotacionalmente simétrico, o
/// que torna a equivariância dele vácua. O que prende a direção do círculo é a metade (2) mais
/// o gate da profundidade (a cabeça deitada para o lado tem profundidade ZERO).
#[test]
fn the_head_still_points_at_the_target_after_scaling_and_rounding() {
    let tip = [5.0, 5.0];
    let polygonal = |m: Marker| !matches!(m, Marker::Circle | Marker::CircleOpen);
    for &m in REGION_HEADS {
        for scale in [0.75, 2.0] {
            for round in [0.0, 0.6] {
                let east = m.build(tip, [1.0, 0.0], 2.0, scale, round).expect("existe");
                let north = m.build(tip, [0.0, 1.0], 2.0, scale, round).expect("existe");
                // 1) girar 90° a direção = girar 90° a cabeça, em torno do bico.
                if polygonal(m) {
                    for (e, n) in east.verts.iter().zip(&north.verts) {
                        let rot = [
                            tip[0] - (e.anchor[1] - tip[1]),
                            tip[1] + (e.anchor[0] - tip[0]),
                        ];
                        assert!(
                            (n.anchor[0] - rot[0]).abs() < 1e-9
                                && (n.anchor[1] - rot[1]).abs() < 1e-9,
                            "{m:?}: a cabeca nao girou com a direcao ({:?} vs {rot:?})",
                            n.anchor
                        );
                    }
                }
                // 2) nada da cabeça passa do bico, e ela ainda chega perto dele.
                let ahead = outline(&east)
                    .iter()
                    .map(|q| q[0] - tip[0])
                    .fold(f64::MIN, f64::max);
                let depth = m.inset(scale) * 2.0;
                assert!(ahead <= 1e-3, "{m:?}: a cabeca passou do alvo em {ahead}");
                assert!(
                    ahead > -0.5 * depth,
                    "{m:?}: o filete decapitou a ponta (ela para {ahead} atras do alvo)"
                );
            }
        }
    }
}
