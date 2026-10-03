//! A escolha do céu: a fábrica é o estúdio de sempre, a cerca da porta, e a força zero põe o céu à
//! luz do estúdio.

use super::{Ceu, normalizacao};

#[test]
fn a_fabrica_e_o_estudio_de_sempre() {
    assert_eq!(Ceu::default().embarcado(), None);
    assert_eq!(crate::smoke::view::View::default().ceu, Ceu::default());
}

#[test]
fn a_cerca_da_porta() {
    let c = Ceu::unpack(&[99.0, f32::NAN, 1.0e9, -3.0, 0.7, 7.0]);
    assert_eq!(c.qual as usize, ph2d_sky::Embarcado::TODOS.len());
    assert_eq!(c.giro, 0.0, "um giro que não é número volta a zero");
    assert_eq!(c.forca, super::FORCA_MAX);
    assert_eq!(c.caixa, 0.0);
    assert!(c.fundo);
    assert_eq!(c.desfoque, 1.0);
    assert_eq!(Ceu::unpack(&c.pack()), c, "ida e volta");
}

fn luma(c: [f32; 3]) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

fn normais(n: usize) -> Vec<[f32; 3]> {
    let phi = std::f32::consts::PI * (3.0 - 5.0f32.sqrt());
    (0..n)
        .map(|i| {
            let y = 1.0 - 2.0 * (i as f32 + 0.5) / n as f32;
            let r = (1.0 - y * y).max(0.0).sqrt();
            [r * (phi * i as f32).cos(), y, r * (phi * i as f32).sin()]
        })
        .collect()
}

/// ⭐⭐ **Força `0` = a luz média do estúdio** — a irradiância média (sobre todas as normais) do céu
/// normalizado, COM o sol, é a do estúdio inteiro, medida na LEI do estúdio
/// ([`crate::studio::Studio::irradiance`]) e não na conta fechada da normalização.
#[test]
fn a_forca_zero_poe_o_ceu_a_luz_do_estudio() {
    use ph2d_material::Environment;
    let st = crate::studio::Studio::of_the_product();
    let ns = normais(4000);
    let estudio: f32 = ns.iter().map(|n| luma(st.irradiance(*n))).sum::<f32>() / ns.len() as f32;
    for e in [ph2d_sky::Embarcado::Por, ph2d_sky::Embarcado::Interior] {
        let ceu = ph2d_sky::Ceu::com_sol(&e.panorama());
        assert!(ceu.sol().is_some(), "{e:?} tem sol");
        let env = ph2d_sky::Orientado {
            ceu: &ceu,
            giro: [1.0, 0.0],
            forca: normalizacao(&ceu),
            sol: 1.0,
        };
        let media: f32 = ns.iter().map(|n| luma(env.irradiance(*n))).sum::<f32>() / ns.len() as f32;
        let rel = (media - estudio).abs() / estudio;
        eprintln!(
            "{e:?}: céu {media:.5} · estúdio {estudio:.5} · {:.2} %",
            100.0 * rel
        );
        assert!(rel < 0.02, "{e:?}: {media} contra {estudio}");
    }
}
