//! Testes do filete de quina.
//!
//! O que eles provam não é "compilou": é que o filete é um **arco de verdade** — em 90° e
//! fora dele. A implementação fácil (só `KAPPA`) passa no teste de 90° e falha no de 45°, e é
//! exatamente por isso que os dois existem.

use super::*;

/// Um ponto da cúbica entre dois vértices, em `t`.
fn cubic(a: &VecVertex, b: &VecVertex, t: f64) -> [f64; 2] {
    let (p0, p1, p2, p3) = (a.anchor, a.out_handle, b.in_handle, b.anchor);
    let u = 1.0 - t;
    let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    [
        w0 * p0[0] + w1 * p1[0] + w2 * p2[0] + w3 * p3[0],
        w0 * p0[1] + w1 * p1[1] + w2 * p2[1] + w3 * p3[1],
    ]
}

/// **O filete tem de ser um ARCO** — não uma curva qualquer que passa perto.
///
/// Mede o que define um arco: todo ponto dele fica à MESMA distância de um centro. O centro
/// sai da geometria da quina (na bissetriz, a `hypot(t, r)` do vértice), e não da
/// implementação — senão o oráculo herdaria o bug que deveria pegar.
fn assert_is_an_arc(verts: &[VecVertex], corner: [f64; 2], tol_frac: f64) {
    // Os dois vértices do filete são os do MEIO (o primeiro e o último são as pontas da linha).
    assert_eq!(
        verts.len(),
        4,
        "1 quina => 2 pontas + 2 tangencias: {verts:?}"
    );
    let (p1, p2) = (verts[1].anchor, verts[2].anchor);

    // A bissetriz, e o raio efetivo: o triângulo (vértice, tangência, centro) é retângulo, com
    // catetos `t` (do vértice à tangência) e `r` (da tangência ao centro).
    let t = (p1[0] - corner[0]).hypot(p1[1] - corner[1]);
    let m = [
        (p1[0] + p2[0]) * 0.5 - corner[0],
        (p1[1] + p2[1]) * 0.5 - corner[1],
    ];
    let ml = m[0].hypot(m[1]);
    assert!(ml > 1e-9, "as duas tangencias colapsaram no vertice");
    let bis = [m[0] / ml, m[1] / ml];

    // `r` sai do vínculo de tangência: |centro − P1| ⊥ (P1 − vértice). Resolvendo para a
    // distância `d` do vértice ao centro ao longo da bissetriz: d = t / cos(metade do ângulo),
    // e cos(metade) é a projeção da aresta unitária na bissetriz.
    let e1 = [(p1[0] - corner[0]) / t, (p1[1] - corner[1]) / t];
    let cos_half = e1[0] * bis[0] + e1[1] * bis[1];
    assert!(cos_half > 1e-9, "bissetriz degenerada");
    let d = t / cos_half;
    let centre = [corner[0] + bis[0] * d, corner[1] + bis[1] * d];
    let r = (d * d - t * t).max(0.0).sqrt();
    assert!(r > 1e-9, "raio efetivo nulo");

    for k in 0..=32 {
        let p = cubic(&verts[1], &verts[2], f64::from(k) / 32.0);
        let dist = (p[0] - centre[0]).hypot(p[1] - centre[1]);
        let err = (dist - r).abs() / r;
        assert!(
            err < tol_frac,
            "o filete NAO e um arco: em t={:.2} o raio deu {dist:.6}, esperado {r:.6} \
             (erro {:.3}%, teto {:.3}%)",
            f64::from(k) / 32.0,
            err * 100.0,
            tol_frac * 100.0
        );
    }
}

/// A quina de 90° — a única que o roteador ortogonal produz, e a que a implementação "só
/// KAPPA" acerta.
#[test]
fn a_right_angle_corner_becomes_a_true_quarter_arc() {
    let pts = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]];
    let v = round_polyline(&pts, 2.0);
    // 0,027% é o erro conhecido da cúbica que aproxima um quarto de círculo.
    assert_is_an_arc(&v, [10.0, 0.0], 0.001);
}

/// **A quina AGUDA — o gate que a implementação preguiçosa não passa.**
///
/// Um filete calculado com o `KAPPA` de 90° erra o braço da cúbica em ~50% aqui (0,55·t contra
/// os 0,37·t corretos), e a "curva" deixa de ser um arco de um jeito bem visível. É a diferença
/// entre um caso particular e a fórmula.
#[test]
fn a_sharp_corner_is_a_true_arc_too_not_just_the_right_angle_case() {
    // Quina de 45°: as duas arestas saem do vértice com 45° entre elas.
    let pts = [[0.0, 0.0], [10.0, 0.0], [10.0 - 7.071, 7.071]];
    let v = round_polyline(&pts, 1.5);
    assert_is_an_arc(&v, [10.0, 0.0], 0.01);
}

/// Uma quina OBTUSA (135°) — o outro lado do intervalo.
#[test]
fn an_obtuse_corner_is_a_true_arc_as_well() {
    let pts = [[0.0, 0.0], [10.0, 0.0], [17.071, 7.071]];
    let v = round_polyline(&pts, 2.0);
    assert_is_an_arc(&v, [10.0, 0.0], 0.002);
}

/// **Raio zero devolve o caminho AFIADO, idêntico.** O default do fluxograma é a quina viva; se
/// arredondar mudasse a geometria de quem não pediu nada, todo conector do desenho se mexeria.
#[test]
fn a_zero_radius_leaves_the_sharp_path_byte_identical() {
    let pts = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [20.0, 10.0]];
    let v = round_polyline(&pts, 0.0);
    let want: Vec<VecVertex> = pts.iter().map(|&p| VecVertex::corner(p)).collect();
    assert_eq!(v, want, "raio 0 nao pode tocar na geometria");
}

/// **O raio é um TETO, não uma promessa.** Um raio maior que a aresta faria dois filetes
/// vizinhos se comerem, e a linha daria um laço para trás — o jeito mais feio de um número
/// grande demais falhar. O clamp a metade da aresta satura em vez de quebrar.
#[test]
fn an_oversized_radius_saturates_instead_of_looping_back_on_itself() {
    // O trecho do meio tem 2 unidades; o raio pedido é 50.
    let pts = [[0.0, 0.0], [10.0, 0.0], [10.0, 2.0], [20.0, 2.0]];
    let v = round_polyline(&pts, 50.0);

    // Toda âncora continua DENTRO da caixa da polilinha original: nada escapou para trás.
    for w in &v {
        let p = w.anchor;
        assert!(
            (-1e-9..=20.0 + 1e-9).contains(&p[0]) && (-1e-9..=2.0 + 1e-9).contains(&p[1]),
            "o filete estourou a caixa da rota (laco para tras): {p:?}"
        );
    }
    // E o caminho continua AVANÇANDO em x: as âncoras nunca recuam (um laço recuaria).
    for pair in v.windows(2) {
        assert!(
            pair[1].anchor[0] >= pair[0].anchor[0] - 1e-9,
            "a rota voltou para tras: {:?} -> {:?}",
            pair[0].anchor,
            pair[1].anchor
        );
    }
}

/// **As pontas não se mexem** — e isso é o que mantém a seta apontando para o lugar certo. A
/// tangente que o `end_tangent` lê nasce do primeiro/último segmento; arredondá-los giraria a
/// ponta de seta.
#[test]
fn the_two_ends_of_the_line_are_never_touched() {
    let pts = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]];
    let v = round_polyline(&pts, 3.0);
    assert_eq!(v.first().expect("primeiro").anchor, [0.0, 0.0]);
    assert_eq!(v.last().expect("ultimo").anchor, [10.0, 10.0]);
    // E elas continuam sendo QUINAS (handles colados): a tangente sai do segmento, limpa.
    assert_eq!(v[0].in_handle, v[0].anchor);
    assert_eq!(v[0].out_handle, v[0].anchor);
}

/// Colineares não têm quina: o filete não pode inventar uma curva onde a linha é reta.
#[test]
fn collinear_points_get_no_fillet() {
    let pts = [[0.0, 0.0], [5.0, 0.0], [10.0, 0.0]];
    let v = round_polyline(&pts, 2.0);
    assert_eq!(v.len(), 3, "nenhum vertice novo: {v:?}");
    assert!(
        v.iter()
            .all(|w| w.in_handle == w.anchor && w.out_handle == w.anchor)
    );
}

/// **O spline PASSA pelos pontos da rota** — ele suaviza as dobras, não inventa um caminho
/// novo. Se ele se afastasse dos vértices, o conector curvo deixaria de desviar dos obstáculos
/// que o A* contornou, e a curva atravessaria a forma que a rota evitou.
#[test]
fn the_smoothed_route_still_passes_through_every_point_the_router_chose() {
    let pts = [[0.0, 0.0], [5.0, 0.0], [5.0, 5.0], [10.0, 5.0]];
    let v = smooth_polyline(&pts, 1.0 / 3.0);
    assert_eq!(v.len(), pts.len(), "um vertice por ponto da rota");
    for (i, p) in pts.iter().enumerate() {
        assert_eq!(v[i].anchor, *p, "o spline saiu do ponto {i} da rota");
    }
    // E ha curvatura de verdade nos pontos INTERNOS (senao "suavizar" nao suavizou nada).
    assert!(v[1].in_handle != v[1].anchor && v[1].out_handle != v[1].anchor);
    assert!(v[2].in_handle != v[2].anchor && v[2].out_handle != v[2].anchor);
}

/// **As duas PONTAS guardam a tangente do proprio segmento** — e e dela que a ponta de seta
/// tira a direcao. Uma tangente "suavizada" no extremo faria a seta apontar para o lado.
#[test]
fn the_ends_keep_the_direction_the_line_leaves_the_shape_with() {
    // Sai na horizontal (para +x) e chega na vertical (para +y).
    let pts = [[0.0, 0.0], [5.0, 0.0], [5.0, 5.0]];
    let v = smooth_polyline(&pts, 1.0 / 3.0);

    // A tangente de saida do 1o vertice e HORIZONTAL (o rumo do 1o segmento), nao a media de
    // nada: o `in_handle` dele nem existe (e a ancora), e o `out_handle` corre em +x.
    let first = &v[0];
    assert_eq!(first.in_handle, first.anchor, "a 1a ponta nao tem entrada");
    let out = [
        first.out_handle[0] - first.anchor[0],
        first.out_handle[1] - first.anchor[1],
    ];
    assert!(
        out[0] > 0.0 && out[1].abs() < 1e-9,
        "a linha tem de SAIR na horizontal (o rumo do stub), e saiu em {out:?} — a seta \
         apontaria para o lado"
    );
    // Idem na chegada: vertical, subindo.
    let last = v.last().expect("ultimo");
    assert_eq!(last.out_handle, last.anchor);
    let inn = [
        last.anchor[0] - last.in_handle[0],
        last.anchor[1] - last.in_handle[1],
    ];
    assert!(
        inn[1] > 0.0 && inn[0].abs() < 1e-9,
        "a linha tem de CHEGAR na vertical, e chegou em {inn:?}"
    );
}

/// **A CURVA NAO ESCAPA DO CAMINHO** — o defeito que o Enio viu ("curvas muito exageradas").
///
/// A 1a versao media o braco do handle pela corda entre os VIZINHOS. Numa rota ortogonal com uma
/// perna longa e outra curta, essa corda e muito maior que o segmento curto: o handle passa do
/// vertice seguinte, a cubica estoura para fora, e o conector vira um S enorme que nao tem nada a
/// ver com a rota que o A* escolheu.
///
/// O gate mede o que o olho ve: a curva AMOSTRADA nao pode sair da caixa da propria polilinha,
/// mais uma folga estreita. Um spline que passa pelos pontos e nao escapa deles cabe ai; um que
/// estoura, nao.
/// A folga que um spline BEM-COMPORTADO precisa: o abaulamento legitimo entre dois pontos, como
/// fracao do maior lado da rota. Apertada de proposito — com 12% (o meu 1o chute) o gate ficava
/// VERDE com o bug presente, que e o pior tipo de teste que existe.
const ESCAPE_PAD: f64 = 0.02;

#[test]
fn the_smoothed_curve_never_escapes_the_route_it_smooths() {
    // Uma perna LONGA e uma CURTA — a geometria que expunha o bug.
    let pts = [[0.0, 0.0], [40.0, 0.0], [40.0, 3.0], [44.0, 3.0]];
    let v = smooth_polyline(&pts, 1.0 / 3.0);

    // A caixa da polilinha, com 12% de folga (o abaulamento legitimo de um spline).
    let (lo, hi): ([f64; 2], [f64; 2]) = ([0.0, 0.0], [44.0, 3.0]);
    let pad = ESCAPE_PAD * (hi[0] - lo[0]).max(hi[1] - lo[1]);

    for w in v.windows(2) {
        let (p0, p1, p2, p3) = (w[0].anchor, w[0].out_handle, w[1].in_handle, w[1].anchor);
        for k in 0..=32 {
            let t = f64::from(k) / 32.0;
            let u = 1.0 - t;
            let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
            let p = [
                a * p0[0] + b * p1[0] + c * p2[0] + d * p3[0],
                a * p0[1] + b * p1[1] + c * p2[1] + d * p3[1],
            ];
            assert!(
                p[0] >= lo[0] - pad
                    && p[0] <= hi[0] + pad
                    && p[1] >= lo[1] - pad
                    && p[1] <= hi[1] + pad,
                "a curva ESTOUROU para fora da rota em {p:?} (caixa {lo:?}..{hi:?}, folga {pad:.2}) \
                 — o braco do handle esta medido pela corda dos vizinhos, e nao pelo segmento"
            );
        }
    }
}

/// O braco NUNCA passa do vertice seguinte. E a formulacao local do gate acima, e a que explica
/// por que a curva nao escapa: um handle mais longo que o segmento leva a cubica para depois do
/// proximo ponto, e ela tem de voltar.
#[test]
fn a_handle_never_reaches_past_the_next_anchor() {
    let pts = [[0.0, 0.0], [40.0, 0.0], [40.0, 3.0], [44.0, 3.0]];
    let v = smooth_polyline(&pts, 1.0 / 3.0);
    for i in 0..v.len() - 1 {
        let (a, b) = (&v[i], &v[i + 1]);
        let seg = (b.anchor[0] - a.anchor[0]).hypot(b.anchor[1] - a.anchor[1]);
        let out = (a.out_handle[0] - a.anchor[0]).hypot(a.out_handle[1] - a.anchor[1]);
        let inn = (b.in_handle[0] - b.anchor[0]).hypot(b.in_handle[1] - b.anchor[1]);
        assert!(
            out <= seg * 0.5 + 1e-9 && inn <= seg * 0.5 + 1e-9,
            "handle mais longo que meio segmento ({out:.2}/{inn:.2} contra {seg:.2}): a cubica \
             passa do proximo ponto e volta"
        );
    }
}

/// Tensao zero devolve a polilinha crua — a identidade tem de ser exata (e o caminho por onde
/// um "curvo" com tensao 0 vira um ortogonal, sem caso especial).
#[test]
fn zero_tension_is_the_raw_polyline() {
    let pts = [[0.0, 0.0], [5.0, 0.0], [5.0, 5.0]];
    let v = smooth_polyline(&pts, 0.0);
    let want: Vec<VecVertex> = pts.iter().map(|&p| VecVertex::corner(p)).collect();
    assert_eq!(v, want);
}
