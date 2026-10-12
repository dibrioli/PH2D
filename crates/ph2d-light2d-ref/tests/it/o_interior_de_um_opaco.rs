//! ⭐⭐⭐ **O interior de um opaco recebe a luz do EXTERIOR mais próximo** (F3 da fila do dono,
//! `docs/Luz2D/01_plano.md` §7.1: «uma linha escura de 1 px junto à face iluminada das paredes»). Um
//! ponto dentro de um corpo opaco recebe a fluência do ponto de fora mais perto dele — a face acesa
//! da parede não mistura zeros, e uma parede que RECEBE luz não sai preta. ⭐ W5h: esta lei é agora a
//! `suavidade = 0` do miolo (a omissão é a difusão, `o_miolo_do_disco`). ⭐ W5k: um vidro (opacidade
//! `α < 1`) mistura a luz que o atravessa com a do opaco, por `α`.

use ph2d_light2d_ecs::{CenaDeLuz, Emissor, Oclusor};
use ph2d_light2d_ref::{Gabarito, MIOLO_PADRAO, OpcoesDoMiolo};

/// A lei do exterior mais próximo é o miolo com `suavidade = 0` (W5h: a difusão é a omissão).
fn fluencia(c: &CenaDeLuz, p: [f64; 2]) -> [f64; 3] {
    let op = OpcoesDoMiolo {
        suavidade: 0.0,
        ..MIOLO_PADRAO
    };
    Gabarito::com_miolo(c, op).fluencia(p)
}

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

/// ⭐ W5k — **dentro de um vidro de opacidade `α`, `mix(transmitida, exterior, α)`**: a transmitida é
/// a cena sem o vidro × `1 − α` (a partida de dentro conta como uma entrada), o exterior o do opaco.
#[test]
fn um_vidro_mistura_a_transmitida_e_o_exterior_pela_opacidade() {
    let p = [1.55, 0.1];
    let vazio = fluencia(&cena(0.0), p)[1];
    for a in [0.5_f32, 0.999, 0.001] {
        let c = cena(a);
        let a = f64::from(a);
        let t = (1.0 - a) * vazio;
        let e = fluencia(&c, logo_fora([1.5, 0.1], [-1.0, 0.0]))[1];
        let esperado = t + (e - t) * a;
        let dentro = fluencia(&c, p)[1];
        assert!(
            (dentro - esperado).abs() <= 1e-9 * vazio,
            "dentro do vidro a {a}: {dentro} × {esperado}"
        );
    }
}

/// ⭐ W5k — **Lipschitz em `α`, com o miolo por omissão**: mudar a opacidade de `δ` muda o interior no
/// máximo `2δ` × a luz sem o corpo (até a W5k, de 1 a 0,999 o interior caía a 0,1 %: o corpo PRETO). A
/// régua é a luz da cena VAZIA: atrás do corpo (`[2.4, −0.3]`) a 0,999 passa `0,1 %` dela, o que sobe
/// a sombra quase preta em ~3 % — físico, não um salto.
#[test]
fn o_interior_e_continuo_na_opacidade() {
    let g = |a: f32, p: [f64; 2]| Gabarito::novo(&cena(a)).fluencia(p)[1];
    for p in [[1.55, 0.1], [2.0, 0.0], [2.4, -0.3]] {
        let vazio = g(0.0, p);
        for (a, b) in [(0.999_f32, 1.0_f32), (0.001, 0.0)] {
            let (x, y) = (g(a, p), g(b, p));
            let teto = 2.0 * f64::from((a - b).abs()) * vazio;
            assert!(
                (x - y).abs() <= teto,
                "{p:?}: {x} a {a} × {y} a {b} (teto {teto})"
            );
        }
    }
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
