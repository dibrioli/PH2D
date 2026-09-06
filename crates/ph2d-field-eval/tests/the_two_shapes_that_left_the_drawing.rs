//! ⭐⭐⭐ **AS DUAS FORMAS QUE SAÍRAM DO «FICA DESENHADA» (W123), PROVADAS ANTES DE SEREM LIGADAS.**
//!
//! > **Enio, 2026-09-05:** *«usando fórmulas não ficam mais leves? Implemente»*
//!
//! ⚠️ **A recusa que estava escrita respondia a OUTRA pergunta.** O plano dava a espiral e a base
//! ondulada do documento como *«a distância não é fechada»* — o que é **verdade** e não é o que o
//! módulo pede: uma marcha de esferas precisa de um **minorante**, nunca do valor exacto.
//!
//! ⇒ estes gates medem as duas coisas que o minorante tem de cumprir: **a superfície está onde a
//! curva está** (pontos cuja resposta se sabe sem a fórmula) e **o campo nunca promete mais do que
//! anda** (o censo mede isso forma a forma).

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Xform};
use ph2d_field_eval::Field;

fn campo(p: Primitive) -> Field {
    Field::new(
        &FieldDoc::new(
            vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
            NodeId(0),
        )
        .expect("a peça"),
    )
}

const NA_PELE: f64 = 3.0e-3;

#[track_caller]
fn dentro(f: &Field, p: [f64; 3], porque: &str) {
    let v = f.at(p[0], p[1], p[2]);
    assert!(
        v < -NA_PELE,
        "{porque}: {p:?} devia estar DENTRO e leu {v:.5}"
    );
}

#[track_caller]
fn fora(f: &Field, p: [f64; 3], porque: &str) {
    let v = f.at(p[0], p[1], p[2]);
    assert!(v > NA_PELE, "{porque}: {p:?} devia estar FORA e leu {v:.5}");
}

#[track_caller]
fn na_pele(f: &Field, p: [f64; 3], porque: &str) {
    let v = f.at(p[0], p[1], p[2]);
    assert!(
        v.abs() < NA_PELE,
        "{porque}: {p:?} devia estar NA SUPERFÍCIE e leu {v:.5}"
    );
}

fn uma_espiral(turns: f32) -> Primitive {
    Primitive::Spiral {
        radius: 0.10,
        pitch: 0.12,
        turns,
        thickness: 0.03,
        half_height: 0.08,
        round: 0.0,
        chamfer: 0.0,
    }
}

/// ⭐ **A ESPIRAL passa pelo raio de cada volta** — `r₀ + pitch·k` no ângulo zero.
///
/// ⚠️ **É a régua que nenhuma contagem de volume dá**: um campo que fosse um anel maciço, ou uma
/// fita com o passo errado, passaria em qualquer medida agregada e falharia aqui.
#[test]
fn the_spiral_passes_through_the_radius_of_every_turn() {
    let f = campo(uma_espiral(3.0));
    // O centro de cada volta está `thickness` para dentro.
    for (k, r) in [(1, 0.22), (2, 0.34)] {
        let d = f.at(r, 0.0, 0.0);
        assert!(
            (d + 0.03).abs() < 1.0e-3,
            "o centro da volta {k} (r = {r}) devia ler −espessura e leu {d:.5}"
        );
    }
    // ⛔ **O VALE entre duas voltas** — sem ele isto seria um disco.
    fora(&f, [0.16, 0.0, 0.0], "o vale entre a 1.ª e a 2.ª volta");
    fora(&f, [0.28, 0.0, 0.0], "o vale entre a 2.ª e a 3.ª volta");
    // O olho do meio é vazio, e a peça acaba no anel de fora.
    fora(&f, [0.04, 0.0, 0.0], "o olho da espiral");
    fora(&f, [0.50, 0.0, 0.0], "para lá do fim");
    na_pele(&f, [0.46, 0.0, 0.0], "o corte do fim (r₀ + pitch × voltas)");
    na_pele(&f, [0.10, 0.0, 0.0], "o corte do princípio (r₀)");
}

/// ⭐⭐⭐ **A FITA SOBE MEIA VOLTA DE CADA VEZ** — a `180°` o raio da volta é `r₀ + pitch·(k + ½)`.
///
/// ⚠️ **É este gate que separa uma espiral de uma fieira de anéis**: um conjunto de anéis
/// concêntricos passa em todos os pontos do ângulo zero e falha aqui.
#[test]
fn the_ribbon_climbs_half_a_pitch_in_half_a_turn() {
    let f = campo(uma_espiral(3.0));
    // Em `x < 0` (ângulo π) o centro da fita está a meio caminho entre duas voltas.
    for r in [0.16, 0.28, 0.40] {
        let d = f.at(-r, 0.0, 0.0);
        assert!(
            (d + 0.03).abs() < 1.0e-3,
            "a meia volta o centro da fita devia estar em r = {r} e leu {d:.5}"
        );
    }
    // E o que era vale do lado de lá é fita deste, e vice-versa.
    fora(
        &f,
        [-0.22, 0.0, 0.0],
        "a meia volta, o raio de uma volta inteira é VALE",
    );
}

/// ⭐⭐ **Mais voltas não mudam o que já lá estava** — a fita cresce por fora.
#[test]
fn adding_turns_only_grows_the_spiral_outwards() {
    let curta = campo(uma_espiral(2.0));
    let longa = campo(uma_espiral(3.0));
    for r in [0.12, 0.16, 0.22, 0.28] {
        let (a, b) = (curta.at(r, 0.0, 0.0), longa.at(r, 0.0, 0.0));
        assert!(
            (a - b).abs() < 1.0e-6,
            "r = {r}: a espiral curta leu {a:.6} e a longa {b:.6} — o miolo tem de ser o mesmo"
        );
    }
    // ⚠️ **O ponto de prova está a MEIA VOLTA** (`x < 0`): no ângulo zero `r = 0,40` é vale nas
    // duas, e uma régua ali não distinguiria nada.
    fora(
        &curta,
        [-0.40, 0.0, 0.0],
        "a de 2 voltas acaba antes (r_fim = 0,34)",
    );
    dentro(
        &longa,
        [-0.40, 0.0, 0.0],
        "e a de 3 continua (r_fim = 0,46)",
    );
}

fn um_documento(wave: f32) -> Primitive {
    Primitive::Document {
        half_width: 0.40,
        half_span: 0.25,
        wave,
        half_height: 0.08,
        round: 0.0,
        chamfer: 0.0,
    }
}

/// ⭐⭐⭐ **A BASE DO DOCUMENTO É A SENÓIDE, e a superfície está onde ela está.**
///
/// ⚠️ Os pontos saem da fórmula da onda, não de correr o código: com meia onda em `[−w, w]`, o vale
/// está em `x = −w/2` e a crista em `x = +w/2`, cada um afastado `wave` da base.
#[test]
fn the_document_base_is_the_sine_itself() {
    let (w, s, a) = (0.40_f64, 0.25_f64, 0.08_f64);
    let f = campo(um_documento(a as f32));
    dentro(&f, [0.0, 0.0, 0.0], "o meio");
    na_pele(&f, [0.0, s, 0.0], "o topo, que é reto");
    na_pele(&f, [w, 0.0, 0.0], "o flanco direito");
    // ⭐ **Nos dois flancos a onda vale ZERO** — `sin(±π) = 0` —, logo a base encontra-os em `−s`.
    na_pele(&f, [-w, -s + 1.0e-4, 0.0], "o canto de baixo à esquerda");
    na_pele(&f, [w, -s + 1.0e-4, 0.0], "o canto de baixo à direita");
    // ⛔ O VALE e a CRISTA — é o que separa esta forma de um retângulo.
    na_pele(&f, [-w * 0.5, -s - a, 0.0], "o vale da onda");
    na_pele(&f, [w * 0.5, -s + a, 0.0], "a crista da onda");
    dentro(
        &f,
        [-w * 0.5, -s + 0.02, 0.0],
        "acima do vale ainda há peça",
    );
    fora(
        &f,
        [w * 0.5, -s + a - 0.02, 0.0],
        "e sob a crista já não há",
    );
    // E o meio da onda cruza a base exactamente em `x = 0`.
    na_pele(&f, [0.0, -s, 0.0], "o cruzamento da onda com a base");
}

/// ⭐⭐ **Com a onda a ZERO ele é o RETÂNGULO** — e é uma forma, não uma degeneração.
#[test]
fn a_document_without_a_wave_is_a_rectangle() {
    let f = campo(um_documento(0.0));
    na_pele(&f, [0.40, -0.25, 0.0], "o canto de baixo à direita");
    na_pele(&f, [-0.20, -0.25, 0.0], "a base, agora reta");
    dentro(&f, [0.0, -0.23, 0.0], "logo dentro dela");
}

/// ⭐⭐⭐ **O CAMPO NUNCA PROMETE MAIS DO QUE ANDA** — o minorante é honesto nas duas.
///
/// ⚠️ **É o gate que a recusa antiga pedia.** Ela dizia que a distância exacta não é fechada; o que
/// o módulo exige é `‖∇f‖ ≤ 1`, e é isso que se mede — numa casca fina em volta da superfície, que
/// é onde a marcha decide.
#[test]
fn neither_curve_ever_overpromises_the_distance() {
    for (nome, p, e) in [
        ("espiral", uma_espiral(3.0), 0.55),
        ("documento", um_documento(0.08), 0.5),
    ] {
        let f = campo(p);
        let mut pior: f64 = 0.0;
        let passos = 70;
        let at = |t: usize| -e + 2.0 * e * (t as f64 + 0.5) / passos as f64;
        for i in 0..passos {
            for j in 0..passos {
                for k in 0..passos {
                    let (x, y, z) = (at(i), at(j), at(k));
                    if f.at(x, y, z).abs() > 0.03 {
                        continue;
                    }
                    pior = pior.max(f.gradient_norm(x, y, z, 1.0e-4));
                }
            }
        }
        assert!(
            pior <= 1.02,
            "«{nome}»: ‖∇f‖ = {pior:.4} — acima de 1 o campo promete mais do que anda, e a marcha \
             atravessa a superfície"
        );
    }
}

/// ⭐ **As duas são CHAPAS: a espessura é em Z.**
#[test]
fn both_are_plates_with_the_same_thickness_law() {
    for (p, x) in [(uma_espiral(3.0), 0.22), (um_documento(0.08), 0.0)] {
        let nome = ph2d_field::Primitive::kind(&p).key();
        let f = campo(p);
        na_pele(&f, [x, 0.0, 0.08], &format!("«{nome}»: a tampa de cima"));
        na_pele(&f, [x, 0.0, -0.08], &format!("«{nome}»: a tampa de baixo"));
        dentro(&f, [x, 0.0, 0.05], &format!("«{nome}»: dentro da laje"));
        fora(&f, [x, 0.0, 0.11], &format!("«{nome}»: acima da tampa"));
    }
}
