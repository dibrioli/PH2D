//! A pergunta da W3: quanta luz RECEBE uma superfície de frente (sprite ou vetor, no plano da
//! cena)? `luz_em = F / 2π` — derivação no plano §2.3. Estes gates fixam-na por três fórmulas
//! fechadas, sem passar pela própria implementação.

use std::f64::consts::{PI, TAU};

use ph2d_light2d_ecs::{CenaDeLuz, Emissor};
use ph2d_light2d_ref::luz_em;

use crate::comum::{cena, disco};

/// O FORNO BRANCO: num campo de radiância uniforme `L`, uma superfície branca devolve `L`. É a
/// lei que fixa o fator — qualquer outro (`1/π`, `1/2`, `1`) acende ou apaga a cena inteira.
#[test]
fn o_forno_branco_devolve_o_ceu() {
    let ceu = [0.25_f32, 0.5, 1.75];
    let c = CenaDeLuz {
        emissores: vec![],
        oclusores: vec![],
        ceu,
        ..Default::default()
    };
    for p in [[0.0, 0.0], [3.7, -12.0], [1e3, 1e3]] {
        let e = luz_em(&c, p);
        for k in 0..3 {
            assert!(
                (e[k] - f64::from(ceu[k])).abs() <= 1e-12,
                "canal {k} em {p:?}: {} ≠ céu {}",
                e[k],
                ceu[k]
            );
        }
    }
}

/// O disco do G1 (`F = 2·L·asin(a/r)`), lido como luz recebida.
#[test]
fn o_disco_no_vazio_da_a_formula_fechada() {
    // O raio da cena é `f32`: a fórmula usa o MESMO número, não o `0,3` decimal.
    let (a32, l) = (0.3_f32, 2.0_f64);
    let a = f64::from(a32);
    let c = cena(
        vec![disco([0.0, 0.0], a32, [l as f32; 3], f32::INFINITY)],
        vec![],
    );
    for r in [0.31, 0.5, 1.0, 4.0, 40.0] {
        let e = luz_em(&c, [r, 0.0])[0];
        let esperado = 2.0 * l * (a / r).asin() / TAU;
        assert!(
            (e - esperado).abs() <= 1e-9 * esperado.max(1.0),
            "r = {r}: {e} ≠ {esperado}"
        );
    }
}

/// A luz pontual da C3 (`F = I/r`): a 1 m, uma `Luz` de intensidade `2π` acende uma superfície
/// branca a exatamente 1 — o número que o Inspector da W4 precisa para um padrão legível.
#[test]
fn o_ponto_a_um_metro_com_intensidade_tau_da_branco() {
    let c = cena(
        vec![Emissor::Ponto {
            centro: [0.0, 0.0],
            intensidade: [TAU as f32; 3],
            alcance_m: f32::INFINITY,
        }],
        vec![],
    );
    let e = luz_em(&c, [0.0, 1.0])[0];
    assert!((e - 1.0).abs() <= 1e-6, "{e}");
    let e2 = luz_em(&c, [2.0 * PI, 0.0])[0];
    assert!((e2 - 1.0 / (2.0 * PI)).abs() <= 1e-6, "{e2}");
}
