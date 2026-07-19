//! Testes de [`crate::width_profile`] — arquivo irmão.

use super::*;

/// Os três valores de controle são ATINGIDOS, exatamente, nos seus lugares. Um perfil que
/// só se aproximasse deles faria o artista digitar `1.0` e receber `0.97`.
#[test]
fn the_three_control_values_are_hit_exactly() {
    let p = WidthProfile {
        start: 0.2,
        mid: 1.8,
        end: 0.5,
        position: 0.35,
    };
    assert!((p.at(0.0) - 0.2).abs() < 1e-12, "start: {}", p.at(0.0));
    assert!((p.at(0.35) - 1.8).abs() < 1e-12, "mid: {}", p.at(0.35));
    assert!((p.at(1.0) - 0.5).abs() < 1e-12, "end: {}", p.at(1.0));
}

/// **A largura é SUAVE no ponto do meio.** Ligar os três com retas deixa um vinco ali — a
/// derivada salta e a silhueta ganha uma quina que ninguém desenhou. O oráculo é a diferença
/// central: com `smoothstep` a inclinação nos dois lados do meio é ~0 e elas CASAM; com lerp
/// elas seriam `(mid−start)/p` e `(end−mid)/(1−p)`, que aqui diferem por mais de 4.
#[test]
fn the_width_has_no_kink_at_the_middle() {
    let p = WidthProfile {
        start: 0.2,
        mid: 1.8,
        end: 0.5,
        position: 0.5,
    };
    let h = 1e-4;
    let left = (p.at(0.5) - p.at(0.5 - h)) / h;
    let right = (p.at(0.5 + h) - p.at(0.5)) / h;
    assert!(
        (left - right).abs() < 0.01,
        "vinco no meio: inclinação {left} à esquerda vs {right} à direita"
    );
}

/// O perfil uniforme devolve `1.0` em todo lugar — é ele que faz "sem perfil" e "perfil
/// neutro" serem a mesma coisa em vez de duas.
#[test]
fn the_uniform_profile_is_one_everywhere() {
    assert!(WidthProfile::UNIFORM.is_uniform());
    for k in 0..=10 {
        let t = f64::from(k) / 10.0;
        assert!((WidthProfile::UNIFORM.at(t) - 1.0).abs() < 1e-12);
    }
    assert!(
        !WidthProfile {
            mid: 2.0,
            ..WidthProfile::UNIFORM
        }
        .is_uniform()
    );
}

/// **O meio colado numa ponta não divide por zero** — e a resposta é o outro trecho inteiro,
/// não `NaN`. Um `NaN` aqui viraria uma largura `NaN`, que envenena a geometria inteira sem
/// dizer de onde veio.
#[test]
fn a_degenerate_position_does_not_divide_by_zero() {
    for pos in [0.0, 1.0] {
        let p = WidthProfile {
            start: 0.2,
            mid: 1.8,
            end: 0.5,
            position: pos,
        };
        for k in 0..=10 {
            let v = p.at(f64::from(k) / 10.0);
            assert!(v.is_finite(), "position={pos}, t={k}/10 deu {v}");
        }
    }
}

/// Fora de `[0,1]` o perfil CLAMPA nas pontas em vez de extrapolar. Quem amostra o fim de um
/// arco recebe `1.0 + 1e-16` de vez em quando, e uma extrapolação ali produziria uma largura
/// que o perfil não contém.
#[test]
fn sampling_outside_the_domain_clamps_instead_of_extrapolating() {
    let p = WidthProfile {
        start: 0.2,
        mid: 1.0,
        end: 0.5,
        position: 0.5,
    };
    assert!((p.at(-0.5) - p.at(0.0)).abs() < 1e-12);
    assert!((p.at(1.5) - p.at(1.0)).abs() < 1e-12);
}

/// O pico é o maior dos três — é o que um consumidor usa para orçar (quanto o traço pode
/// crescer no pior ponto).
#[test]
fn the_peak_is_the_largest_control_value() {
    let p = WidthProfile {
        start: 0.2,
        mid: 1.8,
        end: 0.5,
        position: 0.5,
    };
    assert!((p.peak() - 1.8).abs() < 1e-12);
}
