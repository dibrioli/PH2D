//! ⭐⭐⭐ **Uma lâmpada é LUZ, não um corpo** (smoke do dono, 10/10: *«cor escura na lâmpada faz com
//! que a lâmpada se comporte como um objeto que gera sombra de outra lâmpada»*). Um disco emissor
//! (`Luz`) não tapa a luz de outra lâmpada nem o céu: quem o vê SOMA a luz dele. Os corpos que
//! brilham (polígonos: sprites e formas) continuam opacos.

use ph2d_light2d_ecs::{CenaDeLuz, Emissor};
use ph2d_light2d_ref::fluencia;

use crate::comum::disco;

/// `2·asin(r/d)·L`: a fluência de um disco uniforme sem nada à frente.
fn livre(p: [f64; 2], c: [f32; 2], r: f32, l: f32) -> f64 {
    let d = (f64::from(c[0]) - p[0]).hypot(f64::from(c[1]) - p[1]);
    2.0 * (f64::from(r) / d).asin() * f64::from(l)
}

#[test]
fn uma_lampada_nao_tapa_a_luz_de_outra_nem_o_ceu() {
    let (forte, escura) = (
        ([0.0_f32, 0.0], 0.3_f32, 4.0_f32),
        ([2.0_f32, 0.0], 0.25_f32, 0.05_f32),
    );
    let ceu = 0.1_f32;
    let cena = CenaDeLuz {
        emissores: vec![
            disco(forte.0, forte.1, [forte.2; 3], f32::INFINITY),
            disco(escura.0, escura.1, [escura.2; 3], f32::INFINITY),
        ],
        ceu: [ceu; 3],
        ..Default::default()
    };
    // Atrás da escura, no eixo das duas: ela está EXATAMENTE entre o ponto e a forte.
    for x in [3.0_f64, 4.5, 7.0] {
        let p = [x, 0.0];
        let esperado = f64::from(ceu) * std::f64::consts::TAU
            + livre(p, forte.0, forte.1, forte.2)
            + livre(p, escura.0, escura.1, escura.2);
        let f = fluencia(&cena, p)[1];
        assert!(
            (f - esperado).abs() <= 1e-9 * esperado,
            "em x = {x}: {f} × {esperado} (a lâmpada escura tapou a forte ou o céu?)"
        );
    }
}

#[test]
fn um_corpo_que_brilha_continua_a_tapar() {
    let quadrado = |c: f32, h: f32| vec![[c - h, -h], [c + h, -h], [c + h, h], [c - h, h]];
    let cena = CenaDeLuz {
        emissores: vec![
            disco([0.0, 0.0], 0.3, [4.0; 3], f32::INFINITY),
            Emissor::Poligono {
                vertices: quadrado(2.0, 0.6),
                radiancia: [0.05; 3],
                alcance_m: f32::INFINITY,
            },
        ],
        ..Default::default()
    };
    let p = [4.0, 0.0];
    let so_o_corpo = fluencia(
        &CenaDeLuz {
            emissores: vec![cena.emissores[1].clone()],
            ..cena.clone()
        },
        p,
    )[1];
    let f = fluencia(&cena, p)[1];
    assert!(
        (f - so_o_corpo).abs() <= 1e-9 * so_o_corpo,
        "o corpo que brilha deixou passar a lâmpada de trás: {f} × {so_o_corpo}"
    );
}
