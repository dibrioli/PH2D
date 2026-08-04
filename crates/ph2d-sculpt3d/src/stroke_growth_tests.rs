//! Os gates do traço que **a topologia dinâmica** trouxe.
//!
//! ⚠️ Irmão e não um bloco a mais no `stroke_tests.rs`, e o corte é por
//! ASSUNTO: ali mora *o que um dab faz*, aqui *o que acontece quando a malha
//! cresce debaixo do traço*. O pai bateu no teto de LOC no dia em que estes
//! dois entraram, e juntá-los teria sido o argumento errado — o tamanho é o
//! sintoma, o assunto é a razão.

use super::*;

/// **A LEI DO TRAÇO SOBREVIVE À MALHA CRESCER** — o gate da topologia dinâmica
/// do lado do traço.
///
/// ⚠️ O que está em julgamento é o `pre`: depois de um refino, um vértice que já
/// tinha sido tocado tem de continuar medindo a partir de onde ele estava no
/// PEN-DOWN, e não de onde o dab anterior o deixou. A alternativa que parece
/// óbvia — chamar `begin` outra vez depois de refinar — é exatamente a doença do
/// produto-por-dab que esta casa curou quatro vezes no relevo do Painter, e ela
/// passa despercebida porque o traço continua *funcionando*: ele só fica mais
/// forte quanto mais o refino disparar.
///
/// ⚠️ **A fixture refina de VERDADE** (`ph2d_mesh::refine_in_sphere`), e a
/// primeira versão dela não: ela crescia o traço sem crescer a malha, o que o
/// `assert` do `dab` recusa — corretamente. Um par `grow_to`/malha que não anda
/// junto não é o que o produto faz.
#[test]
fn growing_the_stroke_keeps_the_frozen_base() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(16, 24, 1.0);
    mesh.triangulate();
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    let dab = Dab::at([0.0, 0.0, 1.0], 0.5, [0.0, 0.0, -1.0]);
    stroke.dab(&mut mesh, &Brush::default(), &dab, Symmetry::default());

    let touched = stroke.touched().to_vec();
    let base = stroke.base_positions().to_vec();
    assert!(!touched.is_empty(), "a fixture contém o fenômeno");
    let n_before = mesh.vert_count();

    let target = ph2d_mesh::edge_target(0.5, 1.0);
    let r = ph2d_mesh::refine_in_sphere(&mut mesh, [0.0, 0.0, 1.0], 0.5, target);
    assert!(
        matches!(r, ph2d_mesh::Refine::Done { .. }),
        "a fixture TEM de refinar: {r:?}"
    );
    stroke.grow_to(mesh.vert_count());

    assert_eq!(
        stroke.touched(),
        &touched[..],
        "a janela do traço sobrevive"
    );
    assert_eq!(
        stroke.base_positions(),
        &base[..],
        "e o `pre` de cada vértice tocado é o MESMO — um `begin` aqui zeraria os dois"
    );

    // O dab seguinte continua medindo do `pre`: os vértices JÁ TOCADOS não se
    // movem de novo. (Os nascidos no refino movem-se, e devem: para eles este é
    // o primeiro toque.)
    let after_one: Vec<[f32; 3]> = mesh.positions()[..n_before].to_vec();
    stroke.dab(&mut mesh, &Brush::default(), &dab, Symmetry::default());
    let moved: f32 = touched
        .iter()
        .map(|&v| {
            let (a, b) = (mesh.positions()[v as usize], after_one[v as usize]);
            (a[0] - b[0]).abs() + (a[1] - b[1]).abs() + (a[2] - b[2]).abs()
        })
        .sum();
    assert!(
        moved < 1e-4,
        "o mesmo dab sobre o mesmo `pre` é idempotente; os tocados andaram {moved} — \
         o traço está compondo"
    );
}

/// `grow_to` nunca ENCOLHE — e o gate existe porque encolher seria a forma
/// silenciosa de perder a janela: os índices altos sumiriam do `slot` e o dab
/// seguinte os trataria como nunca-vistos, capturando um `pre` que é o
/// resultado do dab anterior.
///
/// ⚠️ **A primeira versão deste gate media `capacity_bytes`, e a mutação passou
/// por ela:** `Vec::resize` para menos NÃO devolve capacidade, então o oráculo
/// não podia ver o encolhimento que ele julgava. O oráculo certo é o produto —
/// o dab seguinte, que compara `slot.len()` com a contagem de vértices e panica
/// quando elas discordam.
#[test]
fn growing_to_a_smaller_count_is_a_noop() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(8, 12, 1.0);
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    stroke.grow_to(3);
    let moved = stroke.dab(
        &mut mesh,
        &Brush::default(),
        &Dab::at([0.0, 0.0, 1.0], 0.5, [0.0, 0.0, -1.0]),
        Symmetry::default(),
    );
    assert!(moved > 0, "o traço continua dimensionado para ESTA malha");
}
