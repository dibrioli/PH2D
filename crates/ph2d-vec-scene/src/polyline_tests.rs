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
