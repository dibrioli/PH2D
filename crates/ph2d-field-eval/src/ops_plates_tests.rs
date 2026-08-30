//! ⭐⭐⭐ **AS OITO CHAPAS, PROVADAS ANTES DE SEREM LIGADAS** (W106) — o irmão do
//! [`super::super::ops_solids`], pela mesma razão: um erro de geometria descoberto depois de treze
//! ligações é treze ligações a refazer.
//!
//! ⚠️ **A afirmação de uma chapa tem SEMPRE duas metades**, e uma sozinha não vale: *a silhueta é a
//! que se pediu* **e** *a espessura em Z é a que se pediu*. Um contorno certo puxado à altura errada
//! passa em qualquer gate que só olhe o plano XY — e é o erro mais fácil de cometer neste molde,
//! porque a laje entra por uma porta partilhada.
//!
//! ⚠️ Nenhum destes gates lê um relógio ⇒ nenhum pertence à família de flakes de carga do §5.0.

use super::*;
use crate::Field;

fn at(t: &fidget::context::Tree, p: [f64; 3]) -> f64 {
    Field::from_tree(t).at(p[0], p[1], p[2])
}

/// A altura usada por todas — pequena de propósito, para que um erro na laje salte à vista.
const H: f64 = 0.2;

/// ⛔ **O CONTROLO DA FAMÍLIA: toda chapa é uma LAJE.**
///
/// Se este falhar, os gates de silhueta abaixo estão a medir uma forma que não tem espessura
/// nenhuma — e passariam na mesma, porque olham o plano `z = 0`.
#[test]
fn every_plate_is_a_slab_of_the_height_it_was_given() {
    let casos: [(&str, fidget::context::Tree, [f64; 2]); 8] = [
        (
            "engrenagem",
            sd_gear(8, 0.6, 0.9, 0.5, H, 0.0, 0.0),
            [0.0, 0.0],
        ),
        ("cruz", sd_cross(0.8, 0.25, H, 0.0, 0.0), [0.0, 0.0]),
        ("coracao", sd_heart(0.5, H, 0.0, 0.0), [0.0, 0.2]),
        ("lua", sd_moon(0.8, 0.6, 0.45, H, 0.0, 0.0), [-0.5, 0.0]),
        ("gota", sd_drop(0.4, 1.0, H, 0.0, 0.0), [0.0, 0.0]),
        ("fatia", sd_pie(0.8, 0.7, H, 0.0, 0.0), [0.0, 0.4]),
        (
            "trapezio",
            sd_trapezoid(0.7, 0.3, 0.5, H, 0.0, 0.0),
            [0.0, 0.0],
        ),
        ("vesica", sd_vesica(0.7, 0.35, H, 0.0, 0.0), [0.0, 0.0]),
    ];
    for (nome, t, dentro) in casos {
        let [x, y] = dentro;
        // No plano do meio: dentro.
        assert!(
            at(&t, [x, y, 0.0]) < 0.0,
            "{nome}: o ponto de referencia {dentro:?} devia estar DENTRO; deu {}",
            at(&t, [x, y, 0.0])
        );
        // Nas duas faces: na superfície.
        for z in [-H, H] {
            assert!(
                at(&t, [x, y, z]).abs() < 1.0e-6,
                "{nome}: a face z={z} devia estar na superficie; deu {}",
                at(&t, [x, y, z])
            );
        }
        // Acima e abaixo: fora, e à distância certa (a laje é exacta no eixo).
        assert!(
            (at(&t, [x, y, H + 0.3]) - 0.3).abs() < 1.0e-6,
            "{nome}: a 0,3 acima da face a distancia devia ser 0,3; deu {}",
            at(&t, [x, y, H + 0.3])
        );
    }
}

/// ⭐⭐⭐ **A ENGRENAGEM tem DENTES — e a prova é que o raio ALTERNA com o ângulo.**
///
/// ⚠️ **Uma engrenagem sem dentes é um disco, e um disco passa em qualquer gate de silhueta que
/// meça um raio só.** A afirmação tem de ser sobre a *variação*: na direcção de um dente a peça
/// chega a `outer`; a meio caminho entre dois, não passa de `root`.
#[test]
fn a_gear_actually_has_teeth() {
    let (n, root, outer) = (8_u32, 0.6, 0.9);
    let t = sd_gear(n, root, outer, 0.5, H, 0.0, 0.0);
    let passo = std::f64::consts::TAU / f64::from(n);
    for k in 0..n {
        let phi = passo * f64::from(k);
        let ponta = [(outer - 0.02) * phi.cos(), (outer - 0.02) * phi.sin(), 0.0];
        assert!(
            at(&t, ponta) < 0.0,
            "o dente {k} devia chegar a {outer}; em {ponta:?} deu {}",
            at(&t, ponta)
        );
        // ⭐ E entre dois dentes, ao mesmo raio, tem de estar VAZIO — senão é um disco.
        let meio = phi + passo * 0.5;
        let vale = [
            (outer - 0.02) * meio.cos(),
            (outer - 0.02) * meio.sin(),
            0.0,
        ];
        assert!(
            at(&t, vale) > 0.0,
            "entre os dentes {k} e {} devia estar vazio; deu {}",
            k + 1,
            at(&t, vale)
        );
    }
    // ⛔ E o corpo é cheio até `root` em TODA direcção — inclusive no vale.
    for k in 0..n {
        let meio = passo * (f64::from(k) + 0.5);
        let corpo = [(root - 0.02) * meio.cos(), (root - 0.02) * meio.sin(), 0.0];
        assert!(
            at(&t, corpo) < 0.0,
            "o corpo devia ser cheio ate' {root}; deu {}",
            at(&t, corpo)
        );
    }
}

/// ⭐ **A CRUZ tem quatro braços e quatro covas.**
#[test]
fn a_cross_has_four_arms_and_four_notches() {
    let (arm, w) = (0.8, 0.2);
    let t = sd_cross(arm, w, H, 0.0, 0.0);
    // As quatro pontas, quase no fim do braço: dentro.
    for p in [
        [arm - 0.02, 0.0],
        [-(arm - 0.02), 0.0],
        [0.0, arm - 0.02],
        [0.0, -(arm - 0.02)],
    ] {
        assert!(
            at(&t, [p[0], p[1], 0.0]) < 0.0,
            "a ponta {p:?} devia ser cheia"
        );
    }
    // ⭐ As quatro covas — a diagonal, logo depois da largura do braço: VAZIO. É o que separa uma
    // cruz de um quadrado.
    let d = w + 0.05;
    for s in [[1.0, 1.0], [-1.0, 1.0], [1.0, -1.0], [-1.0, -1.0]] {
        assert!(
            at(&t, [s[0] * d, s[1] * d, 0.0]) > 0.0,
            "a cova diagonal {s:?} devia estar vazia"
        );
    }
    // E o centro é cheio.
    assert!(at(&t, [0.0, 0.0, 0.0]) < 0.0);
}

/// ⭐⭐ **O CORAÇÃO tem a COVA em cima e a PONTA em baixo** — e é essa assimetria que o faz coração.
#[test]
fn a_heart_has_a_cleft_on_top_and_a_point_below() {
    let s = 0.5;
    let t = sd_heart(s, H, 0.0, 0.0);
    // A ponta de baixo: o vértice do losango em `(0, −s)`.
    assert!(
        at(&t, [0.0, -s, 0.0]).abs() < 1.0e-6,
        "a ponta de baixo devia estar na superficie; deu {}",
        at(&t, [0.0, -s, 0.0])
    );
    assert!(
        at(&t, [0.0, -s - 0.05, 0.0]) > 0.0,
        "abaixo da ponta e' vazio"
    );
    // ⭐ Os dois lóbulos são cheios acima do topo do losango — é o que os semicírculos acrescentam.
    for x in [-s * 0.5, s * 0.5] {
        assert!(
            at(&t, [x, s * 0.9, 0.0]) < 0.0,
            "o lobulo em x={x} devia ser cheio acima do losango; deu {}",
            at(&t, [x, s * 0.9, 0.0])
        );
    }
    // ⭐⭐ A COVA: no eixo, entre os dois lóbulos, a peça acaba mais cedo do que nos lóbulos.
    let topo_lobulo = s * 0.5 + s / 2.0_f64.sqrt();
    assert!(
        at(&t, [0.0, topo_lobulo, 0.0]) > 0.0,
        "no eixo, a' altura do topo dos lobulos, tem de haver COVA; deu {}",
        at(&t, [0.0, topo_lobulo, 0.0])
    );
}

/// ⭐ **A LUA é um crescente: o meio foi MORDIDO.**
#[test]
fn a_moon_has_a_bite_taken_out_of_it() {
    let (r, bite, off) = (0.8, 0.6, 0.45);
    let t = sd_moon(r, bite, off, H, 0.0, 0.0);
    // O lado oposto à mordida: cheio.
    assert!(
        at(&t, [-r + 0.05, 0.0, 0.0]) < 0.0,
        "o dorso do crescente e' cheio"
    );
    // ⭐ O centro da mordida: VAZIO — é a metade que uma silhueta de disco não distinguiria.
    assert!(
        at(&t, [off, 0.0, 0.0]) > 0.0,
        "o centro da mordida devia estar vazio; deu {}",
        at(&t, [off, 0.0, 0.0])
    );
    // E as duas pontas do crescente existem: acima e abaixo, perto do bordo, ainda há material.
    let yc = (r * r - (off * 0.5).powi(2)).sqrt() * 0.6;
    assert!(at(&t, [0.0, yc, 0.0]) < 0.0 || at(&t, [0.0, -yc, 0.0]) < 0.0);
}

/// ⭐ **A GOTA afina para cima e é redonda em baixo.**
#[test]
fn a_drop_is_round_below_and_pointed_above() {
    let (r, h) = (0.4, 1.0);
    let t = sd_drop(r, h, H, 0.0, 0.0);
    // O fundo é o círculo.
    assert!(
        at(&t, [0.0, -r, 0.0]).abs() < 1.0e-6,
        "o fundo e' o circulo"
    );
    assert!(
        at(&t, [r, 0.0, 0.0]).abs() < 1.0e-6,
        "o equador e' o circulo"
    );
    // A ponta está em `y = h`.
    assert!(
        at(&t, [0.0, h - 0.02, 0.0]) < 0.0,
        "abaixo da ponta e' cheio"
    );
    assert!(
        at(&t, [0.0, h + 0.05, 0.0]) > 0.0,
        "acima da ponta e' vazio"
    );
    // ⭐ E afina: a meia altura entre o equador e a ponta, a largura é MENOR que `r`.
    let meio = h * 0.5;
    assert!(
        at(&t, [r * 0.9, meio, 0.0]) > 0.0,
        "a' meia altura a gota ja' devia ser mais estreita que o raio"
    );
    // ⛔⛔ **O CONE TEM DE ACABAR EM BAIXO.** Sem o corte na altura de tangência, a união com a
    // bolha deixa uma cunha a descer para sempre — e ela é invisível a todo gate que meça o
    // equador. *Uma forma sem tecto passa em qualquer régua que só olhe onde ela é boa.*
    assert!(
        at(&t, [0.0, -h * 2.0, 0.0]) > 0.0,
        "bem abaixo da bolha tem de estar VAZIO — o cone nao pode continuar; deu {}",
        at(&t, [0.0, -h * 2.0, 0.0])
    );
    assert!(
        at(&t, [r * 0.5, -h, 0.0]) > 0.0,
        "a cunha do cone nao pode sobreviver abaixo da bolha"
    );
}

/// ⭐ **A FATIA é um sector: fora da abertura não há nada.**
#[test]
fn a_pie_keeps_only_its_wedge() {
    let (r, ang) = (0.8, 0.6);
    let t = sd_pie(r, ang, H, 0.0, 0.0);
    // No eixo da bissectriz (+Y), dentro do raio: cheio.
    assert!(at(&t, [0.0, r * 0.5, 0.0]) < 0.0);
    assert!(at(&t, [0.0, r * 1.2, 0.0]) > 0.0, "fora do raio e' vazio");
    // ⭐ Fora da abertura, ao mesmo raio: vazio.
    let fora = ang * 1.6;
    assert!(
        at(&t, [r * 0.5 * fora.sin(), r * 0.5 * fora.cos(), 0.0]) > 0.0,
        "fora da abertura devia estar vazio"
    );
    // E o lado oposto nunca.
    assert!(at(&t, [0.0, -r * 0.5, 0.0]) > 0.0);
}

/// ⭐ **O TRAPÉZIO estreita num eixo só** — e é isso que o separa de uma pirâmide truncada.
#[test]
fn a_trapezoid_narrows_on_one_axis_only() {
    let (b, tp, hw) = (0.7, 0.3, 0.5);
    let t = sd_trapezoid(b, tp, hw, H, 0.0, 0.0);
    // Na base (`y = −hw`) a meia-largura é `b`; no topo é `tp`.
    assert!(at(&t, [b - 0.02, -hw + 0.02, 0.0]) < 0.0, "a base e' larga");
    assert!(
        at(&t, [b - 0.02, hw - 0.02, 0.0]) > 0.0,
        "o topo e' estreito: a largura da base nao cabe la'"
    );
    assert!(
        at(&t, [tp - 0.02, hw - 0.02, 0.0]) < 0.0,
        "o topo tem a largura dele"
    );
    // ⭐ E a espessura em Z **não** estreita — é a diferença para o prisma de 4 lados afunilado.
    assert!(
        at(&t, [0.0, hw - 0.02, H * 0.9]) < 0.0,
        "a chapa mantem a espessura no topo"
    );
}

/// ⭐ **A VESICA é uma lente com duas pontas.**
#[test]
fn a_vesica_is_a_lens_with_two_points() {
    let (r, off) = (0.7, 0.35);
    let t = sd_vesica(r, off, H, 0.0, 0.0);
    assert!(at(&t, [0.0, 0.0, 0.0]) < 0.0, "o meio da lente e' cheio");
    // As pontas em `y = ±√(r²−off²)`.
    let yp = (r * r - off * off).sqrt();
    assert!(
        at(&t, [0.0, yp - 0.02, 0.0]) < 0.0,
        "quase na ponta ainda e' cheio"
    );
    assert!(
        at(&t, [0.0, yp + 0.05, 0.0]) > 0.0,
        "alem da ponta e' vazio"
    );
    // ⭐ E é ESTREITA em X: a largura máxima é `r − off`, bem menos que `r`.
    assert!(
        at(&t, [r - off + 0.05, 0.0, 0.0]) > 0.0,
        "a lente nao devia ser tao larga como um disco"
    );
}

/// ⭐⭐⭐ **A MARCHA É SEGURA em todas as oito.**
#[test]
fn every_new_plate_marches_safely() {
    let casos: [(&str, fidget::context::Tree); 8] = [
        ("engrenagem", sd_gear(8, 0.6, 0.9, 0.5, H, 0.0, 0.0)),
        ("cruz", sd_cross(0.8, 0.25, H, 0.0, 0.0)),
        ("coracao", sd_heart(0.5, H, 0.0, 0.0)),
        ("lua", sd_moon(0.8, 0.6, 0.45, H, 0.0, 0.0)),
        ("gota", sd_drop(0.4, 1.0, H, 0.0, 0.0)),
        ("fatia", sd_pie(0.8, 0.7, H, 0.0, 0.0)),
        ("trapezio", sd_trapezoid(0.7, 0.3, 0.5, H, 0.0, 0.0)),
        ("vesica", sd_vesica(0.7, 0.35, H, 0.0, 0.0)),
    ];
    const TETO: f64 = 1.02;
    for (nome, t) in casos {
        let f = Field::from_tree(&t);
        let mut pior: f64 = 0.0;
        for i in -12..=12 {
            for j in -12..=12 {
                for k in -12..=12 {
                    let p = [
                        f64::from(i) / 12.0 * 1.8,
                        f64::from(j) / 12.0 * 1.8,
                        f64::from(k) / 12.0 * 1.8,
                    ];
                    pior = pior.max(f.gradient_norm(p[0], p[1], p[2], 1.0e-4));
                }
            }
        }
        assert!(
            pior <= TETO,
            "{nome}: o campo sobe a {pior} por unidade — a marcha atravessaria a superficie"
        );
        assert!(
            pior > 0.5,
            "{nome}: gradiente {pior} e' baixo demais para ser distancia"
        );
    }
}
