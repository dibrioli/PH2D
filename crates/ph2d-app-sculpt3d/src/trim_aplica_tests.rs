//! Gates da costura do corte.

use super::alvo_da_lamina;
use ph2d_mesh::shapes;

/// **O alvo da lâmina descreve a densidade que a peça TEM.**
///
/// ⚠️ A barra é a comparação com a **mediana das arestas**, que é a grandeza
/// que o olho compara na foto — e não um epsilon escolhido: as duas réguas
/// medem a mesma coisa por caminhos diferentes, e é a concordância delas que
/// autoriza usar a barata.
#[test]
fn o_alvo_da_lamina_e_a_aresta_que_a_peca_tem() {
    for (nome, m) in [
        ("uv_sphere(24,32)", shapes::uv_sphere(24, 32, 1.0)),
        ("uv_sphere(48,64)", shapes::uv_sphere(48, 64, 1.0)),
        ("tri≈20k", shapes::sphere_with_triangles(20_000, 1.0)),
    ] {
        let p = m.positions();
        let mut tris = Vec::new();
        for f in m.faces() {
            f.triangles(&mut tris);
        }
        let d = |a: u32, b: u32| {
            let (a, b) = (p[a as usize], p[b as usize]);
            ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
        };
        let mut e: Vec<f32> = tris
            .iter()
            .flat_map(|t| [d(t[0], t[1]), d(t[1], t[2]), d(t[2], t[0])])
            .collect();
        e.sort_by(f32::total_cmp);
        let mediana = e[e.len() / 2];
        let alvo = alvo_da_lamina(&m);
        let razao = alvo / mediana;
        assert!(
            (1.0..=1.25).contains(&razao),
            "{nome}: o alvo ({alvo}) afastou-se da mediana das arestas ({mediana}): {razao:.2}×"
        );
    }
}

/// ⛔⛔ **A LEI TEM DE CHEGAR AO CORTE** — o defeito que este gate existe para
/// impedir é o caro: a `ph2d-trim` sabe tesselar, os gates dela ficam verdes, e
/// o produto continua a pedir a lâmina mínima. *Uma lei sem chamador e uma lei
/// ausente produzem o mesmo app.*
///
/// ⚠️ A agulha é a CHAMADA e não o nome — foi um gate que casava um nome que
/// deixou passar uma mutação neste mesmo módulo.
#[test]
fn o_corte_pede_a_lamina_na_densidade_da_peca() {
    let src = include_str!("trim_aplica.rs");
    assert!(
        src.contains("ph2d_trim::Resolucao::Ate(alvo_da_lamina(self.mesh()))"),
        "o corte deixou de pedir a lâmina tesselada à densidade da peça"
    );
    assert!(
        !src.contains("Resolucao::Minima"),
        "o corte voltou à lâmina mínima — é o defeito do report de 2026-09-15"
    );
}
