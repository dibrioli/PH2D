//! Gates do catálogo. O oráculo é a APARÊNCIA/estrutura (para onde o ponto vai, quem tem shear),
//! nunca a fórmula repetida.

use super::{CageSpec, WarpStyle, is_neutral};

/// **`bend == 0` é a IDENTIDADE em todo estilo** — o neutro no-op que a pilha de efeitos exige.
#[test]
fn the_neutral_deform_is_the_identity() {
    assert!(is_neutral(0.0));
    for &s in WarpStyle::ALL {
        for &(u, v) in &[(0.3, -0.7), (-1.0, 1.0), (0.0, 0.0), (0.9, 0.2)] {
            assert_eq!(
                s.deform(u, v, 0.0),
                (u, v),
                "{}: bend 0 tem de devolver o ponto",
                s.label()
            );
        }
    }
}

/// **São NOVE estilos, com rótulos DISTINTOS** — a lista que as duas seções passam a oferecer.
#[test]
fn there_are_nine_distinct_styles() {
    assert_eq!(WarpStyle::ALL.len(), 9);
    let mut labels: Vec<&str> = WarpStyle::ALL.iter().map(|s| s.label()).collect();
    let n = labels.len();
    labels.sort_unstable();
    labels.dedup();
    assert_eq!(labels.len(), n, "há dois estilos com o mesmo rótulo");
}

/// **Todo estilo DEFORMA** — com dobra forte, algum ponto de teste sai do lugar (senão o estilo
/// seria um nome para a identidade).
#[test]
fn every_style_actually_deforms() {
    for &s in WarpStyle::ALL {
        let moved = [(0.5, 0.6), (-0.7, 0.4), (0.8, -0.9)]
            .iter()
            .any(|&(u, v)| s.deform(u, v, 0.6) != (u, v));
        assert!(moved, "{}: bend 0.6 não moveu nenhum ponto", s.label());
    }
}

/// **Arc ARQUEIA** — dobra positiva levanta o meio (`u=0`) acima da linha; negativa o afunda. O
/// oráculo é o deslocamento do ponto central, não a fórmula.
#[test]
fn the_arc_arches() {
    let (_, up) = WarpStyle::Arc.deform(0.0, 0.0, 0.5);
    assert!(up > 0.01, "Arc+ devia levantar o centro, deu {up}");
    let (_, down) = WarpStyle::Arc.deform(0.0, 0.0, -0.5);
    assert!(down < -0.01, "Arc- devia afundar o centro, deu {down}");
}

/// **Só o Rise CISALHA** (move canto); todos os outros mexem só nas barrigas (cantos fixos). É a
/// invariante que mantém a garantia de não-dobra do Envelope válida para os oito primeiros — e diz
/// exatamente onde a via nova do shear tem de valer.
#[test]
fn only_rise_shears_the_cage() {
    for &s in WarpStyle::ALL {
        assert_eq!(
            s.shears(),
            s == WarpStyle::Rise,
            "{}: shear esperado só no Rise",
            s.label()
        );
        let CageSpec { bows, shift } = s.cage();
        // Nenhum número da gaiola é absurdo (±1 é o teto da tabela).
        for row in bows.iter().chain(shift.iter()) {
            for &c in row {
                assert!(
                    c.abs() <= 1.0 + 1e-9,
                    "{}: entrada de gaiola > 1: {c}",
                    s.label()
                );
            }
        }
    }
}

/// **Flag e Wave DIFEREM** — o Flag é em fase (a coluna inteira sobe junto), o Wave é contrafase
/// (cima e baixo vão a lados opostos). Num ponto de cima, com o mesmo `u`, os dois discordam do
/// sinal — é o que separa os dois nomes que antes eram um só.
#[test]
fn flag_and_wave_are_not_the_same_style() {
    // Em `u = 0.5`, `sin(π·u) = 1`. No topo (`v = 1`): Flag sobe por `b`; Wave escala `v` por
    // `(1+b)` ⇒ também sobe, mas por `b·v`. No FUNDO (`v = -1`) eles divergem em SINAL: Flag ainda
    // sobe (+b), Wave desce (o `v` negativo vira mais negativo).
    let b = 0.5;
    let (_, flag_bottom) = WarpStyle::Flag.deform(0.5, -1.0, b);
    let (_, wave_bottom) = WarpStyle::Wave.deform(0.5, -1.0, b);
    assert!(
        flag_bottom > -1.0,
        "Flag no fundo devia SUBIR (fase), deu {flag_bottom}"
    );
    assert!(
        wave_bottom < -1.0,
        "Wave no fundo devia DESCER (contrafase), deu {wave_bottom}"
    );
}
