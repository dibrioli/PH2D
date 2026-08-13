//! Gates das primitivas de **blocagem** — cilindro e toro.
//!
//! O oráculo é ESTRUTURAL (contagens, fechamento, valência, a caixa), nunca um
//! literal de ponto flutuante: estas funções chamam `sin`/`cos` da `std`, que
//! não são pinadas bit a bit entre sistemas operacionais — o cabeçalho do
//! [`super`] já diz isso, e este arquivo é quem obedece.

use super::*;
use crate::Aabb;

/// ⚠️ **A pergunta tem UMA porta** (`Mesh::is_closed`), e este alias existe só
/// para as asserções abaixo continuarem legíveis. A cópia que morava aqui
/// reconstruía a adjacência à mão — e uma segunda resposta a *"isto é um
/// sólido?"* divergiria da que o renderer usa para decidir se pode remover
/// linha escondida.
fn is_closed(mesh: &Mesh) -> bool {
    mesh.is_closed()
}

#[test]
fn a_cylinder_is_a_closed_solid_with_quads_around_and_fans_on_the_caps() {
    let seg = 12;
    let m = cylinder(seg, 1.0, 2.0);
    assert_eq!(m.vert_count(), 2 + 2 * seg, "dois polos e dois anéis");
    assert_eq!(m.face_count(), 3 * seg, "por fatia: duas tampas e um quad");
    let quads = (0..m.face_count())
        .filter(|&i| !m.faces()[i].is_tri())
        .count();
    assert_eq!(quads, seg, "o CORPO é quad puro — é onde se esculpe");
    assert!(is_closed(&m), "uma primitiva de blocagem é um SÓLIDO");
}

/// ⚠️ **O toro não tem polo nenhum**, e é a propriedade que o torna a fixture de
/// topologia regular: todo vértice tem valência 4, então nada no alisamento
/// precisa contornar uma estrela.
#[test]
fn a_torus_is_all_quads_and_every_vertex_has_valence_four() {
    let (mj, mn) = (16, 8);
    let m = torus(mj, mn, 1.0, 0.35);
    assert_eq!(m.vert_count(), mj * mn);
    assert_eq!(m.face_count(), mj * mn);
    assert!(
        (0..m.face_count()).all(|i| !m.faces()[i].is_tri()),
        "quad puro"
    );
    assert!(is_closed(&m), "e fechado");

    let adj = crate::Adjacency::build(m.vert_count(), m.faces());
    for v in 0..m.vert_count() {
        assert_eq!(adj.valence(v), 4, "o vértice {v} tem de ter valência 4");
    }
}

/// As duas cabem na caixa que os parâmetros prometem. ⚠️ Com folga para BAIXO
/// de propósito: um polígono inscrito não alcança o círculo (a razão é
/// `cos(π/n)`), então exigir a igualdade seria pedir que a malha fosse a forma
/// ideal — e reprovaria a primitiva certa.
#[test]
fn the_primitives_fit_the_box_their_parameters_promise() {
    let cyl = cylinder(24, 1.0, 3.0);
    let b = Aabb::from_points(cyl.positions());
    assert!(
        (b.max[1] - 1.5).abs() < 1e-5 && (b.min[1] + 1.5).abs() < 1e-5,
        "a altura é exata"
    );
    assert!(
        b.max[0] <= 1.0 + 1e-5 && b.max[0] > 0.99,
        "o raio, inscrito"
    );

    let t = torus(24, 12, 2.0, 0.5);
    let b = Aabb::from_points(t.positions());
    assert!(b.max[0] <= 2.5 + 1e-5 && b.max[0] > 2.45, "maior + menor");
    assert!(
        (b.max[1] - 0.5).abs() < 1e-5,
        "a espessura é o raio MENOR, exata: {}",
        b.max[1]
    );
}

/// Um `segments` degenerado é CLAMPADO, não recusado. ⚠️ Recusar devolveria um
/// `Result` a um chamador que é um GESTO do artista, e o gesto não tem o que
/// fazer com um erro; três é o menor número de fatias que ainda fecha um sólido.
#[test]
fn a_degenerate_segment_count_is_floored_into_a_solid() {
    for m in [cylinder(0, 1.0, 1.0), cylinder(2, 1.0, 1.0)] {
        assert_eq!(m.vert_count(), 2 + 2 * 3);
        assert!(is_closed(&m));
    }
    let t = torus(1, 1, 1.0, 0.3);
    assert_eq!(t.vert_count(), 9);
    assert!(is_closed(&t));
}

/// **A ESFERA DE ESCULTURA É A DO SCULPTGL: 98304 quads, e nenhum triângulo.**
///
/// ⚠️ O 98304 é uma **consequência** da regra (`while faces < 50_000`), não um
/// literal do código — este gate existe para a consequência não mudar em
/// silêncio se alguém afinar o teto. A contagem é a que o Enio nomeou ao pedir
/// a troca, e ela vem do `subdivideClamp` do SculptGL, que aplicado a um cubo
/// passa por 6 → 24 → 96 → 384 → 1536 → 6144 → 24576 e para em 98304.
///
/// ⚠️ **"Nenhum triângulo" é metade da entrega.** Um cubo subdividido é todo
/// quad por construção, e é isso que faz dela uma malha que se subdivide de
/// novo sem degenerar — o oposto do leque de polo da esfera UV.
#[test]
fn the_sculpt_sphere_is_the_sculptgl_one() {
    let m = sculpt_sphere(1.0);
    assert_eq!(m.face_count(), 98_304, "a contagem do SculptGL");
    assert_eq!(m.vert_count(), 98_306);
    assert!(
        m.faces().iter().all(|f| !f.is_tri()),
        "um cubo subdividido é todo quad"
    );
    assert!(is_closed(&m), "e fechada");
}

/// **A RAZÃO DE ARESTA É O ARGUMENTO INTEIRO DA TROCA, e ela é um número.**
///
/// ⚠️ Este é o gate que diz por que o default mudou: a esfera UV tem um leque de
/// triângulos finíssimos em cada polo e quads esticados no equador, então o
/// MESMO pincel come áreas muito diferentes conforme onde o artista toca.
/// Medido, a razão entre a maior e a menor aresta é **3,9×** na de escultura e
/// **30,6×** na `uv_sphere(96, 144)` — quase oito vezes mais desigual.
///
/// A barra é 10× (bem entre os dois) para o gate falhar por REGRESSÃO DE
/// TOPOLOGIA e não por ruído de ponto flutuante.
#[test]
fn the_sculpt_sphere_has_far_more_even_edges_than_a_uv_sphere() {
    let spread = |m: &Mesh| -> f32 {
        let p = m.positions();
        let (mut lo, mut hi) = (f32::MAX, 0.0f32);
        for f in m.faces() {
            let v = f.verts();
            for i in 0..v.len() {
                let (a, b) = (p[v[i] as usize], p[v[(i + 1) % v.len()] as usize]);
                let d =
                    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
                lo = lo.min(d);
                hi = hi.max(d);
            }
        }
        hi / lo
    };
    let sculpt = spread(&sculpt_sphere(1.0));
    let uv = spread(&uv_sphere(96, 144, 1.0));
    assert!(sculpt < 10.0, "a de escultura ficou desigual: {sculpt:.1}x");
    assert!(uv > 10.0, "a UV deixou de ser o CONTROLE: {uv:.1}x");
    assert!(
        uv > sculpt * 4.0,
        "a UV tinha de ser MUITO mais desigual ({uv:.1}x contra {sculpt:.1}x)"
    );
}

/// **Ela ocupa a MESMA caixa que a esfera que substituiu.**
///
/// ⚠️ Sem a normalização o limite de Catmull-Clark de `cube(1.0)` tem
/// meia-extensão **0,4198**, e a peça abriria 2,4× menor — a câmera é enquadrada
/// por `mesh.bounds()` e o import escala contra o diâmetro das primitivas da
/// cena, então os dois mudariam de significado sem erro nenhum.
///
/// ⚠️ **E a caixa tem de vir do `bounds()`, não das posições:** é ela que a
/// câmera lê, e ela só é verdade se o `rebuild` foi pago depois da escala. Um
/// gate que medisse `Aabb::from_points` passaria com a dívida em aberto.
#[test]
fn the_sculpt_sphere_fills_the_box_the_uv_sphere_filled() {
    for r in [0.5, 1.0, 4.0] {
        let m = sculpt_sphere(r);
        let b = m.bounds();
        for i in 0..3 {
            assert!(
                (b.max[i] - r).abs() < 1e-4 && (b.min[i] + r).abs() < 1e-4,
                "o eixo {i} do raio {r} saiu em [{}, {}]",
                b.min[i],
                b.max[i]
            );
        }
    }
}

/// **Ela NÃO é uma esfera, e o desvio está pinado.**
///
/// ⚠️ Este gate parece afirmar um defeito, e afirma de propósito: o que sai é a
/// superfície-limite de Catmull-Clark de um cubo, e o SculptGL **não**
/// esferifica. Se um dia alguém "consertar" isto normalizando cada vértice para
/// o raio, a peça deixa de ser a da referência — e o espaçamento uniforme, que é
/// a razão de ela ser boa de esculpir, vai junto. O desvio medido é **3,09%**.
#[test]
fn the_sculpt_sphere_is_a_rounded_cube_and_that_is_the_point() {
    let m = sculpt_sphere(1.0);
    let mut r: Vec<f32> = m
        .positions()
        .iter()
        .map(|p| (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt())
        .collect();
    r.sort_by(|a, b| a.partial_cmp(b).expect("sem NaN"));
    let (lo, hi, med) = (r[0], r[r.len() - 1], r[r.len() / 2]);
    let spread = 100.0 * (hi - lo) / med;
    assert!(
        (2.0..5.0).contains(&spread),
        "o desvio de raio saiu {spread:.2}% — abaixo de 2% alguém a esferificou, \
         acima de 5% a suavização mudou"
    );
}
