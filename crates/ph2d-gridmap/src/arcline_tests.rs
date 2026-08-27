use super::{Signed, measure_arc_lines};
use crate::cut::CutMesh;
use crate::solve::GridMap;
use crate::solve::turn2;

/// ⭐⭐⭐ **A IDENTIDADE QUE A EQUAÇÃO INTEIRA ASSENTA:** `e·(R^rot·v) = turn2(e, −rot)·v`.
///
/// ⚠️ **É aqui que um sinal troca sem que nada deixe de compilar.** O gate não lê as
/// entradas de matriz nenhuma — ele avalia os dois lados e compara, para os quatro `rot`
/// e os dois eixos. *Uma dedução à mão verificada por outra dedução à mão não é uma
/// verificação.*
#[test]
fn the_axis_identity_holds_for_every_turn() {
    let dot = |a: [f32; 2], b: [f32; 2]| a[0].mul_add(b[0], a[1] * b[1]);
    for e in [[1.0, 0.0], [0.0, 1.0]] {
        for rot in 0..4 {
            for v in [[1.0, 0.0], [0.0, 1.0], [3.0, -7.0], [-2.0, 5.0]] {
                let left = dot(e, turn2(v, rot));
                let right = dot(turn2(e, -rot), v);
                assert!(
                    (left - right).abs() < 1e-5,
                    "e={e:?} rot={rot} v={v:?}: {left} != {right}"
                );
            }
        }
    }
}

/// ⭐ E o resultado de `turn2(e, −rot)` é sempre um EIXO com sinal — é isso que faz os
/// coeficientes serem `±1` e a eliminação levar inteiros a inteiros.
#[test]
fn a_quarter_turn_of_an_axis_is_an_axis() {
    for e in [[1.0, 0.0], [0.0, 1.0]] {
        for rot in -4..=4 {
            let v = turn2(e, -rot);
            let zeros = usize::from(v[0].abs() < 1e-6) + usize::from(v[1].abs() < 1e-6);
            assert_eq!(zeros, 1, "e={e:?} rot={rot} ⇒ {v:?} nao e' um eixo");
            assert!((v[0].abs().max(v[1].abs()) - 1.0).abs() < 1e-6);
        }
    }
}

/// A união com sinal compõe: `y_c = σ₂·y_b + δ₂` e `y_b = σ₁·y_a + δ₁` ⇒ o `find` de `c`
/// tem de devolver `σ₁σ₂` e `σ₂δ₁ + δ₂`.
#[test]
fn the_signed_union_composes() {
    let mut uf = Signed::new(3);
    // `y_b = −1·y_a + 4`
    uf.parent[1] = 0;
    uf.sign[1] = -1.0;
    uf.off[1] = 4.0;
    // `y_c = −1·y_b + 3`
    uf.parent[2] = 1;
    uf.sign[2] = -1.0;
    uf.off[2] = 3.0;
    let (root, s, d) = uf.find(2);
    assert_eq!(root, 0);
    assert!((s - 1.0).abs() < 1e-6, "sinal {s}");
    // `y_c = −(−y_a + 4) + 3 = y_a − 1`
    assert!((d + 1.0).abs() < 1e-6, "deslocamento {d}");
}

/// ⚠️ E a compressão de caminho **não pode mudar a resposta** — ela reescreve `sign`/`off`
/// enquanto os lê. *Um `find` que se corrompe a si próprio dá a resposta certa uma vez.*
#[test]
fn path_compression_does_not_move_the_answer() {
    let mut uf = Signed::new(4);
    for (child, parent, s, d) in [
        (1u32, 0u32, -1.0, 4.0),
        (2, 1, -1.0, 3.0),
        (3, 2, -1.0, 1.0),
    ] {
        uf.parent[child as usize] = parent;
        uf.sign[child as usize] = s;
        uf.off[child as usize] = d;
    }
    let first = uf.find(3);
    let second = uf.find(3);
    assert_eq!(first.0, second.0);
    assert!((first.1 - second.1).abs() < 1e-6);
    assert!((first.2 - second.2).abs() < 1e-6);
}

/// Sem costuras não há equação nenhuma, e `0` ali significa **«nada a impor»**.
#[test]
fn no_seams_means_no_equations() {
    let cut = CutMesh::default();
    let w = crate::weld::Weld::default();
    let map = GridMap::default();
    let r = measure_arc_lines(&cut, &w, &map);
    assert_eq!(r.arcs, 0);
    assert_eq!(r.sign_conflicts, 0);
    assert_eq!(r.eliminated, 0);
}

/// ⭐⭐⭐ **O INTERRUPTOR DESLIGADO É INERTE, BIT A BIT.**
///
/// ⚠️ *É o controlo da wave inteira.* Sem ele, «a saída mudou» e «a wave fez alguma
/// coisa» leem-se igual — e a segunda pode ser falsa com a primeira verdadeira.
#[test]
fn the_ties_switch_is_inert_when_off() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(16, 24, 1.0);
    mesh.triangulate();
    let dual = ph2d_crossfield::Dual::build(&mesh);
    let (field, _) = ph2d_crossfield::solve_miq(&dual);
    let layout = ph2d_trace::trace_patches(&mesh, &dual, &field);
    let (cut, _) = crate::cut::cut_along_patches(&mesh, &layout);
    let (combed, _) = crate::comb::comb_patches(&mesh, &layout, &cut);
    let h = 0.2;
    let (a, _) = crate::weld_solve::solve_welded(&mesh, &cut, &combed, h, 4);
    let (b, _) = crate::weld_solve::solve_welded_with(&mesh, &cut, &combed, h, 4, None);
    assert_eq!(a.shift, b.shift);
    assert_eq!(a.uv.len(), b.uv.len());
    for (ra, rb) in a.uv.iter().zip(&b.uv) {
        assert_eq!(ra, rb, "o mapa mudou com o interruptor DESLIGADO");
    }
}

/// ⭐⭐ **E LIGADO ELE MEXE** — a saída deixa de ser a mesma.
///
/// ⚠️ Este gate não afirma que ela ficou **melhor**; afirma que a restrição **entrou**.
/// *Um interruptor que não move nada e um que melhora tudo leem igual num gate de
/// igualdade, e só um deles é o que se construiu.*
#[test]
fn the_ties_change_the_map_when_on() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(16, 24, 1.0);
    mesh.triangulate();
    let dual = ph2d_crossfield::Dual::build(&mesh);
    let (field, _) = ph2d_crossfield::solve_miq(&dual);
    let layout = ph2d_trace::trace_patches(&mesh, &dual, &field);
    let (cut, _) = crate::cut::cut_along_patches(&mesh, &layout);
    let (combed, _) = crate::comb::comb_patches(&mesh, &layout, &cut);
    let h = 0.2;
    let (base, _) = crate::weld_solve::solve_welded(&mesh, &cut, &combed, h, 4);
    let (w, _) = crate::weld::weld(&cut, &combed);
    let ties = super::build_arc_ties(&cut, &w, &base);
    assert!(ties.groups() > 0, "a esfera tem de dar grupos de amarra");
    let (tied, rep) = crate::weld_solve::solve_welded_with(&mesh, &cut, &combed, h, 4, Some(&ties));
    assert!(
        rep.tie_groups > 0,
        "nenhum grupo entrou: {} recusados",
        rep.tie_refused
    );
    let moved = base
        .uv
        .iter()
        .zip(&tied.uv)
        .any(|(ra, rb)| ra.iter().zip(rb).any(|(a, b)| a != b));
    assert!(moved, "as amarras nao moveram o mapa");
}

/// ⭐⭐⭐ **A ÁLGEBRA TEM DE REPRODUZIR A GEOMETRIA** — e é este o gate que decide se a
/// equação está certa.
///
/// O resíduo de [`ArcEquation::residual`] é a **componente atravessada** do arco, lida
/// por dentro (somando termos sobre variáveis). A
/// [`crate::align::measure_arc_quantization`] lê a **mesma** grandeza por fora (a
/// diferença das duas posições). ⚠️ *Dois caminhos independentes até o mesmo número — se
/// discordarem, é a álgebra que está errada, e descobre-se AGORA e não depois da
/// eliminação.*
///
/// ⛔ Sem este gate, um sinal trocado no `off` passaria despercebido até a restrição
/// puxar o mapa para o sítio errado — e ali já haveria duas variáveis a bissecar.
#[test]
fn the_equation_residual_matches_the_geometric_reading() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(16, 24, 1.0);
    mesh.triangulate();
    let dual = ph2d_crossfield::Dual::build(&mesh);
    let (field, _) = ph2d_crossfield::solve_miq(&dual);
    let layout = ph2d_trace::trace_patches(&mesh, &dual, &field);
    let (cut, _) = crate::cut::cut_along_patches(&mesh, &layout);
    let (combed, _) = crate::comb::comb_patches(&mesh, &layout, &cut);
    let (map, _) = crate::weld_solve::solve_welded(&mesh, &cut, &combed, 0.2, 6);
    let (w, _) = crate::weld::weld(&cut, &combed);

    let eqs = super::arc_equations(&cut, &w, &map);
    assert!(!eqs.is_empty(), "a esfera tem de dar equacoes");
    // A leitura geométrica: por arco, o menor componente do deslocamento dos extremos.
    let demand = vec![0u32; cut.seams.len() + 1];
    let geo = crate::align::measure_arc_quantization(&cut, &map, &demand);
    assert_eq!(
        eqs.len(),
        geo.arcs,
        "as duas reguas tem de ver os MESMOS arcos"
    );

    let mut worst = 0.0f32;
    for eq in &eqs {
        worst = worst.max(eq.residual(&w, &map).abs());
    }
    // ⚠️ A barra é o `max` da geométrica, com folga de `f32`: as duas contas somam os
    // mesmos termos por ordens diferentes.
    assert!(
        (worst - geo.across_max).abs() < 1.0e-3,
        "algebra {worst} contra geometria {} — a equacao nao reproduz a leitura",
        geo.across_max
    );
}

/// ⭐ **TODO coeficiente é `±1`** — é isso que faz a eliminação levar inteiros a inteiros.
///
/// ⚠️ *Um coeficiente `2` seria meia célula*, que é exactamente o que o `worst_det` do
/// [`crate::weld_flat`] existe para contar.
#[test]
fn every_arc_coefficient_is_plus_or_minus_one() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(16, 24, 1.0);
    mesh.triangulate();
    let dual = ph2d_crossfield::Dual::build(&mesh);
    let (field, _) = ph2d_crossfield::solve_miq(&dual);
    let layout = ph2d_trace::trace_patches(&mesh, &dual, &field);
    let (cut, _) = crate::cut::cut_along_patches(&mesh, &layout);
    let (combed, _) = crate::comb::comb_patches(&mesh, &layout, &cut);
    let (map, _) = crate::weld_solve::solve_welded(&mesh, &cut, &combed, 0.2, 6);
    let (w, _) = crate::weld::weld(&cut, &combed);
    for eq in super::arc_equations(&cut, &w, &map) {
        for (v, ax, k) in eq.terms {
            assert!(
                (k.abs() - 1.0).abs() < 1e-6,
                "coeficiente {k} em {v:?}[{ax}] — a eliminacao deixaria de ser inteira"
            );
        }
    }
}
