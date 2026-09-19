//! ⏱️ **O ENVELOPE MUDA MESMO A DEFORMAÇÃO?** — report do dono, 2026-09-18: *«não vi em nenhum dos
//! casos o envelope fazer diferença na deformação»*.
//!
//! ⚠️ A minha resposta anterior foi *«numa forma vectorial ele manda como sempre»*, e ela não estava
//! medida **do lado que ele olhou**: o censo dos knobs media os PESOS, não a geometria deformada.

use ph2d_skeleton::{Skin, SkinBone, Xform};

/// Uma corrente de `n` ossos em linha, cada um com o `strength` dado ao do MEIO.
fn corrente(n: usize, forca_do_meio: f64) -> Skin {
    let ossos: Vec<SkinBone> = (0..n)
        .filter_map(|k| {
            let x = k as f64 * 10.0;
            let forca = if k == n / 2 { forca_do_meio } else { 1.0 };
            SkinBone::new(
                Xform([1.0, 0.0, 0.0, 1.0, x, 0.0]),
                10.0,
                forca,
                Xform::IDENTITY,
                Xform::IDENTITY,
            )
        })
        .collect();
    Skin::new(ossos).expect("a corrente")
}

/// ⭐⭐⭐ **A MEDIÇÃO: com UM osso o envelope é INERTE — os pesos renormalizam para `1`.**
///
/// ⛔⛔ *É a explicação que faltava ao report:* uma forma presa a **um** osso deforma **igual** a
/// qualquer alcance, porque o único osso leva sempre a fatia inteira. O alcance só decide quando
/// há **dois ou mais** a disputar o mesmo ponto.
#[test]
fn com_um_osso_o_envelope_e_inerte_e_com_tres_ele_manda() {
    let peso = |n: usize, forca: f64| {
        let pele = corrente(n, forca);
        let mut w = pele.scratch();
        pele.weights_at([10.0, 4.0], &mut w);
        w
    };
    let um_fraco = peso(1, 0.3);
    let um_forte = peso(1, 4.0);
    assert_eq!(
        um_fraco, um_forte,
        "com UM osso o envelope mudou alguma coisa: entao a explicacao do report esta' errada"
    );

    let tres_fraco = peso(3, 0.3);
    let tres_forte = peso(3, 4.0);
    let d: f64 = tres_fraco
        .iter()
        .zip(&tres_forte)
        .map(|(a, b)| (a - b).abs())
        .sum();
    println!(
        "1 osso: {um_fraco:?} vs {um_forte:?} | 3 ossos: {tres_fraco:?} vs {tres_forte:?} (d {d:.4})"
    );
    assert!(
        d > 1e-3,
        "com TRES ossos o envelope tambem nao muda nada (d {d:.6}): entao ele e' inerte em toda \
         parte, e escondê-lo nas formas vectoriais tambem estava certo"
    );
}
