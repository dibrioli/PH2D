//! Os gates de [`Dual::boost_align`] — o ponto de extensão que reforça o alinhamento ao relevo
//! onde quem chama sabe que o relevo manda mais (a calota de um espinho afiado, 2026-09-02).
//!
//! ⚠️ Ele nasceu como INSTRUMENTO (`PH2D_TIP_ALIGN`, `sculpt3d_retopo_one.rs`) e não como cura:
//! medido, a `k = 5` as cinco réguas da grade da peça do dono ficam verdes **e** a extracção deixa
//! um laço de `14` arestas no bico da agulha — o mecanismo está em
//! `docs/3D/quad-remesh/PLANO_a_graduacao_da_ponta.md` §102. O que estes gates defendem é o
//! contrato da porta, não o veredito.

use ph2d_crossfield::Dual;
use ph2d_mesh::shapes;

/// A esfera com uma orelha: as faces têm anisotropia > 0 em quase todo o lado.
fn dual_com_relevo() -> (Dual, usize) {
    let mesh = shapes::uv_sphere_noisy(24, 32, 1.0, 0.15);
    let n = mesh.faces().len();
    (Dual::build(&mesh), n)
}

#[test]
fn o_reforco_multiplica_so_a_confianca_das_faces_pedidas() {
    let (mut dual, n) = dual_com_relevo();
    let before: Vec<(f32, f32)> = (0..n).map(|f| dual.align(f)).collect();
    let alvo: Vec<usize> = (0..n).step_by(7).collect();
    dual.boost_align(alvo.iter().copied(), 5.0);
    let mut tocadas = 0usize;
    for (f, &(a0, c0)) in before.iter().enumerate() {
        let (a1, c1) = dual.align(f);
        assert!((a0 - a1).abs() < 1.0e-9, "o ANGULO nao muda na face {f}");
        if alvo.contains(&f) {
            assert!(
                (c1 - c0 * 5.0).abs() < 1.0e-6,
                "face {f}: {c0} -> {c1}, esperado x5"
            );
            if c0 > 0.0 {
                tocadas += 1;
            }
        } else {
            assert!(
                (c1 - c0).abs() < 1.0e-9,
                "face {f} nao foi pedida e mudou: {c0} -> {c1}"
            );
        }
    }
    assert!(
        tocadas > 0,
        "a fixtura tem de ter faces com anisotropia > 0 entre as pedidas"
    );
}

/// ⛔ Um factor que não seja `> 0` e finito é um no-op — e uma face fora da malha é ignorada
/// em vez de estourar: o chamador do produto deriva as faces de uma bola de caminho.
#[test]
fn factor_invalido_e_face_inexistente_nao_mudam_nada() {
    let (mut dual, n) = dual_com_relevo();
    let before: Vec<(f32, f32)> = (0..n).map(|f| dual.align(f)).collect();
    dual.boost_align(0..n, 0.0);
    dual.boost_align(0..n, -3.0);
    dual.boost_align(0..n, f32::NAN);
    dual.boost_align([n + 10, n * 4], 5.0);
    for (f, &antes) in before.iter().enumerate() {
        assert_eq!(
            dual.align(f),
            antes,
            "face {f} mudou com um factor invalido"
        );
    }
}

/// A anisotropia `0` continua `0` — reforçar uma face sem direcção não lhe inventa uma.
#[test]
fn uma_face_sem_direccao_continua_sem_direccao() {
    let mesh = shapes::uv_sphere(24, 32, 1.0);
    let n = mesh.faces().len();
    let mut dual = Dual::build(&mesh);
    let zeros: Vec<usize> = (0..n).filter(|&f| dual.align(f).1 == 0.0).collect();
    dual.boost_align(zeros.iter().copied(), 100.0);
    for &f in &zeros {
        assert_eq!(dual.align(f).1, 0.0, "face {f} ganhou confianca do nada");
    }
}
