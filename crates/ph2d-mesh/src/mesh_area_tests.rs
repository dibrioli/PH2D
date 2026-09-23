//! ⭐⭐ **Os gates da ÁREA** — a da peça e a de cada face.
//!
//! ⚠️ Cortado do [`super`] por TECTO DE LOC em 2026-09-23, por
//! RESPONSABILIDADE: aqui mora *quanto de superfície há*.

use crate::shapes;

/// ⭐⭐⭐ **A ÁREA POR FACE SOMA A ÁREA DA PEÇA — e as faces NÃO são iguais.**
///
/// ⛔⛔ A segunda metade é a que decide: sem ela, uma implementação que
/// devolvesse `vec![surface_area() / face_count(); n]` passa na primeira **e o
/// consumidor da graduação lê um nível só para a peça inteira** — que é
/// exactamente o produto que a P2 existe para deixar de ser.
///
/// ⚠️ A tolerância é RELATIVA e não absoluta: as duas somas partilham a
/// aritmética do triângulo e divergem na ORDEM da acumulação, que é a decisão
/// escrita na [`Mesh::face_areas`] — *a `surface_area` alimenta tectos
/// de outra obra e não pode mudar um ULP por causa desta porta*.
#[test]
fn a_area_por_face_soma_a_area_da_peca_e_elas_diferem() {
    for (nome, m) in [
        ("esfera", shapes::uv_sphere(12, 16, 1.0)),
        ("cubo", shapes::cube(1.0)),
    ] {
        let areas = m.face_areas();
        assert_eq!(areas.len(), m.face_count(), "{nome}: uma área por face");
        assert!(
            areas.iter().all(|a| *a > 0.0),
            "{nome}: uma face de área zero"
        );

        let soma: f32 = areas.iter().sum();
        let peca = m.surface_area();
        let rel = (soma - peca).abs() / peca;
        assert!(
            rel < 1e-5,
            "{nome}: soma {soma} contra peça {peca} ({rel:e})"
        );

        // ⭐ CONTROLO: numa esfera UV as faces do pólo são bem mais pequenas
        //   que as do equador, e é essa dispersão que a graduação consome.
        if nome == "esfera" {
            let (mut lo, mut hi) = (f32::MAX, 0.0f32);
            for a in &areas {
                lo = lo.min(*a);
                hi = hi.max(*a);
            }
            assert!(
                hi / lo > 1.5,
                "{nome}: as faces são todas do mesmo tamanho ({lo} a {hi}) — \
                 esta fixtura não contém o fenómeno"
            );
        }
    }
}
