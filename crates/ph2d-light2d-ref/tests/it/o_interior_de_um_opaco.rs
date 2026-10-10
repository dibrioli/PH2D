//! ⭐⭐⭐ **O interior de um opaco recebe a luz do EXTERIOR mais próximo** (F3 da fila do dono,
//! `docs/Luz2D/01_plano.md` §7.1: «uma linha escura de 1 px junto à face iluminada das paredes»). Um
//! ponto dentro de um corpo opaco recebe a fluência do ponto de fora mais perto dele — a face acesa
//! da parede não mistura zeros, e uma parede que RECEBE luz não sai preta. Um vidro (opacidade < 1)
//! continua a atenuar por dentro.

use ph2d_light2d_ecs::{CenaDeLuz, Emissor, Oclusor};
use ph2d_light2d_ref::fluencia;

use crate::comum::disco;

fn caixa(c: [f32; 2], h: [f32; 2]) -> Vec<[f32; 2]> {
    vec![
        [c[0] - h[0], c[1] - h[1]],
        [c[0] + h[0], c[1] - h[1]],
        [c[0] + h[0], c[1] + h[1]],
        [c[0] - h[0], c[1] + h[1]],
    ]
}

fn cena(opacidade: f32) -> CenaDeLuz {
    CenaDeLuz {
        emissores: vec![disco([0.0, 0.0], 0.2, [1.0; 3], f32::INFINITY)],
        oclusores: vec![Oclusor {
            vertices: caixa([2.0, 0.0], [0.5, 0.5]),
            opacidade,
            albedo: [0.0; 3],
        }],
        ceu: [0.0; 3],
        ..Default::default()
    }
}

/// Logo fora da face `p` (o lado de `fora` a 1 nm).
fn logo_fora(p: [f64; 2], fora: [f64; 2]) -> [f64; 2] {
    [p[0] + 1e-9 * fora[0], p[1] + 1e-9 * fora[1]]
}

#[test]
fn o_interior_de_um_opaco_recebe_o_exterior_mais_proximo() {
    let c = cena(1.0);
    // (dentro, o ponto da fronteira mais perto, a normal de fora)
    for (dentro, face, normal) in [
        ([1.55, 0.1], [1.5, 0.1], [-1.0, 0.0]),
        ([2.0, 0.3], [2.0, 0.5], [0.0, 1.0]),
        ([2.45, -0.2], [2.5, -0.2], [1.0, 0.0]),
    ] {
        let f = fluencia(&c, dentro)[1];
        let g = fluencia(&c, logo_fora(face, normal))[1];
        assert!(
            (f - g).abs() <= 1e-6 * g.max(1e-12),
            "dentro de {dentro:?}: {f} × {g} do exterior mais próximo {face:?}"
        );
    }
    // Controlo: a face acesa recebe luz (o interior deixou de ser 0 por causa dela).
    assert!(fluencia(&c, [1.55, 0.1])[1] > 0.1);
}

#[test]
fn um_vidro_continua_a_atenuar_por_dentro() {
    // No MESMO ponto, a cena sem o vidro: a partida de dentro conta como uma entrada (× ½).
    let p = [1.55, 0.1];
    let dentro = fluencia(&cena(0.5), p)[1];
    let vazio = fluencia(&cena(0.0), p)[1];
    assert!(
        (dentro - 0.5 * vazio).abs() <= 1e-9 * vazio,
        "dentro do vidro: {dentro} × ½ de {vazio}"
    );
}

#[test]
fn um_corpo_que_brilha_recebe_o_exterior_e_nao_a_si_mesmo() {
    let c = CenaDeLuz {
        emissores: vec![
            disco([0.0, 0.0], 0.2, [1.0; 3], f32::INFINITY),
            Emissor::Poligono {
                vertices: caixa([2.0, 0.0], [0.5, 0.5]),
                radiancia: [0.3; 3],
                alcance_m: f32::INFINITY,
            },
        ],
        ceu: [0.0; 3],
        ..Default::default()
    };
    let f = fluencia(&c, [1.55, 0.1])[1];
    let g = fluencia(&c, logo_fora([1.5, 0.1], [-1.0, 0.0]))[1];
    assert!(
        (f - g).abs() <= 1e-6 * g,
        "dentro do corpo que brilha: {f} × {g} do exterior"
    );
}
