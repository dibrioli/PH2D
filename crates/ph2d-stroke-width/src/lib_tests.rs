//! Testes de [`crate`] — o arquivo irmão do `lib.rs`.

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

// ── A LISTA DE PARADAS (ADR-0145) ──────────────────────────────────────────────────────

/// **O preset reduz à lista BIT A BIT.** É o gate central do ADR-0145: o motor passou a
/// consumir paradas, e a única coisa que garante que nada do que já shipava se moveu é a
/// redução ser a MESMA função — não uma aproximação com o mesmo desenho.
///
/// ⚠️ A comparação é `==` em `f64`, de propósito. Um épsilon aqui aceitaria uma segunda
/// implementação "quase igual", que é exatamente o que este gate existe para recusar.
#[test]
fn the_preset_reduces_to_the_stop_list_bit_for_bit() {
    for &position in &[0.0, 0.15, 0.35, 0.5, 0.72, 1.0] {
        let p = WidthProfile {
            start: 0.2,
            mid: 1.8,
            end: 0.5,
            position,
        };
        let stops = p.to_stops();
        for k in 0..=200 {
            let t = f64::from(k) / 200.0;
            assert_eq!(
                p.at(t),
                stops.at(t),
                "position={position}, t={t}: o preset e a lista discordam"
            );
        }
        // E fora do domínio, onde os dois clampam.
        for t in [-0.5, 1.5] {
            assert_eq!(p.at(t), stops.at(t), "position={position}, t={t} (fora)");
        }
    }
}

/// **O meio colado numa ponta vira DUAS paradas, não três com posição repetida.** Emitir
/// `(0,start),(0,mid),(1,end)` daria a mesma curva no miolo e a resposta ERRADA na borda —
/// `at(1.0)` com `position = 1` é `mid`, não `end`. O ramo próprio é o que torna a redução
/// exata; sem ele o gate acima falha só nas duas pontas do domínio.
#[test]
fn a_degenerate_preset_emits_two_stops_not_three() {
    let taper = WidthProfile {
        start: 0.2,
        mid: 1.8,
        end: 0.5,
        position: 1.0,
    };
    let stops = taper.to_stops();
    assert_eq!(stops.as_slice().len(), 2);
    assert_eq!(stops.at(1.0), taper.at(1.0));

    let front = WidthProfile {
        position: 0.0,
        ..taper
    };
    assert_eq!(front.to_stops().as_slice().len(), 2);
    assert_eq!(front.to_stops().at(0.0), front.at(0.0));
}

/// **A lista vazia é o traço uniforme.** É o neutro que permite a um documento não guardar
/// perfil nenhum quando não há perfil — e não um valor que por acaso não faz nada.
#[test]
fn an_empty_stop_list_is_the_uniform_stroke() {
    let s = WidthStops::default();
    assert!(s.is_empty());
    assert!(s.is_uniform());
    for k in 0..=10 {
        assert_eq!(s.at(f64::from(k) / 10.0), 1.0);
    }
    assert!(WidthProfile::UNIFORM.to_stops().is_uniform());
}

/// **A construção ORDENA.** A alça do Width Tool vai inserir paradas onde o artista clicar, e
/// uma lista fora de ordem faria o `at` percorrer o bracket errado e devolver lixo em
/// silêncio. A ordenação mora numa porta só, e não na cabeça de quem escreve.
#[test]
fn building_a_list_sorts_it_by_position() {
    let s = WidthStops::new(vec![
        WidthStop {
            pos: 0.8,
            mult: 2.0,
        },
        WidthStop {
            pos: 0.1,
            mult: 0.5,
        },
        WidthStop {
            pos: 0.4,
            mult: 1.0,
        },
    ]);
    let got: Vec<f64> = s.as_slice().iter().map(|x| x.pos).collect();
    assert_eq!(got, vec![0.1, 0.4, 0.8]);
    // E o valor nas paradas é o autorado, em qualquer ordem de entrada.
    assert!((s.at(0.1) - 0.5).abs() < 1e-12);
    assert!((s.at(0.8) - 2.0).abs() < 1e-12);
}

/// Uma posição fora de `[0,1]` é CLAMPADA na construção — a lista guarda o domínio que o `at`
/// promete, em vez de deixar uma parada inalcançável escondida na ponta.
#[test]
fn building_a_list_clamps_positions_into_the_domain() {
    let s = WidthStops::new(vec![
        WidthStop {
            pos: -3.0,
            mult: 0.5,
        },
        WidthStop {
            pos: 9.0,
            mult: 2.0,
        },
    ]);
    let got: Vec<f64> = s.as_slice().iter().map(|x| x.pos).collect();
    assert_eq!(got, vec![0.0, 1.0]);
}

/// **Uma lista com uma parada só é uma largura CONSTANTE.** Não é um caso de uso do painel,
/// mas é o que uma alça arrastada até apagar as vizinhas produz — e o `at` tem de responder
/// sem tocar num índice que não existe.
#[test]
fn a_single_stop_is_a_constant_width() {
    let s = WidthStops::new(vec![WidthStop {
        pos: 0.5,
        mult: 1.7,
    }]);
    for k in 0..=10 {
        assert!((s.at(f64::from(k) / 10.0) - 1.7).abs() < 1e-12);
    }
    assert!(!s.is_uniform());
}

/// O pico da lista nunca é MENOR que `1.0` — quem orça a partir dele (o memo, a bbox de um
/// preview) precisa que a largura autorada do traço seja o piso, não um perfil que só afina.
#[test]
fn the_peak_of_a_thinning_list_is_still_one() {
    let s = WidthStops::new(vec![
        WidthStop {
            pos: 0.0,
            mult: 0.2,
        },
        WidthStop {
            pos: 1.0,
            mult: 0.3,
        },
    ]);
    assert!((s.peak() - 1.0).abs() < 1e-12);
}
