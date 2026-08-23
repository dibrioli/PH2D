//! Os gates da **viagem entre vistas** (W51).

use super::*;
use crate::field3d_views::{Standard, named_view};

fn from_to(a: Standard, b: Standard) -> Flight {
    Flight {
        from: Orbit {
            rotation: a.rotation(),
            ..Orbit::default()
        },
        to: Orbit {
            rotation: b.rotation(),
            ..Orbit::default()
        },
    }
}

/// ⭐⭐ **A VIAGEM ACABA EXATAMENTE NO DESTINO** — e é uma exigência, não um detalhe.
///
/// ⚠️ O chip da vista acende por [`named_view`], que reconhece a orientação com uma barra de
/// **0,16°**. Uma viagem que pousasse *perto* deixaria o botão apagado para sempre, e o artista
/// leria isso como *"a vista não funcionou"* — com a peça no sítio certo à frente dele.
#[test]
fn the_trip_lands_exactly_on_the_destination() {
    for a in Standard::ALL {
        for b in Standard::ALL {
            let f = from_to(a, b);
            assert_eq!(
                f.at(1.0),
                f.to,
                "{a:?} → {b:?}: em t=1 a câmera tem de SER o destino, escrita, não interpolada"
            );
            assert_eq!(
                named_view(&f.at(1.0)),
                Some(b),
                "{a:?} → {b:?}: o destino não é reconhecido como a vista dele"
            );
        }
    }
}

/// **E começa exatamente na partida** — o outro extremo, que um `t` mal normalizado partiria.
#[test]
fn the_trip_starts_exactly_where_the_camera_was() {
    let f = from_to(Standard::Front, Standard::Top);
    assert_eq!(named_view(&f.at(0.0)), Some(Standard::Front));
}

/// ⭐⭐ **PELO CAMINHO CURTO.**
///
/// ⚠️ `q` e `−q` são a mesma orientação. Sem o `dot < 0`, metade das viagens dá a volta pelo lado
/// comprido — a peça gira 300° para chegar a um sítio a 60°, e da cadeira isso lê como o app a
/// perder o controlo.
///
/// A régua é o **comprimento do percurso**: somar o ângulo entre passos consecutivos e exigir que
/// ele não passe do ângulo direto entre as duas pontas (mais uma folga de amostragem).
#[test]
fn the_trip_takes_the_short_way_round() {
    for a in Standard::ALL {
        for b in Standard::ALL {
            if a == b {
                continue;
            }
            let f = from_to(a, b);
            let direct = angle(f.from.rotation, f.to.rotation);
            let mut walked = 0.0;
            let n = 64;
            for i in 0..n {
                let (t0, t1) = (i as f32 / n as f32, (i + 1) as f32 / n as f32);
                walked += angle(f.at(t0).rotation, f.at(t1).rotation);
            }
            assert!(
                walked <= direct + 0.01,
                "{a:?} → {b:?}: o percurso andou {walked:.3} rad para um destino a {direct:.3} — \
                 deu a volta pelo lado comprido"
            );
        }
    }
}

/// ⭐ **A viagem é MONÓTONA** — cada passo aproxima do destino, nenhum afasta.
///
/// ⚠️ É a metade que um `slerp` mal normalizado ou uma interpolação componente-a-componente
/// quebram: a trajetória continua a acabar no sítio certo, e no meio dela a peça acelera, trava, ou
/// volta atrás.
#[test]
fn every_step_gets_closer_to_the_destination() {
    let f = from_to(Standard::Left, Standard::Top);
    let mut last = angle(f.at(0.0).rotation, f.to.rotation);
    for i in 1..=64 {
        let d = angle(f.at(i as f32 / 64.0).rotation, f.to.rotation);
        assert!(
            d <= last + 1.0e-4,
            "no passo {i} a câmera AFASTOU-SE do destino: {last:.4} → {d:.4}"
        );
        last = d;
    }
}

/// ⭐ **O enquadramento viaja em FRAÇÃO, não em unidades** — a mesma lei do zoom deste módulo
/// (`ZOOM_PER_STEP`, *"para que cada passo aproxime a mesma fração"*).
///
/// ⚠️ Linear, uma viagem de `0,1` a `10` passaria metade do tempo entre `5` e `10`: a peça
/// dispararia para longe e depois rastejaria. O meio geométrico de `0,1` e `10` é `1`.
#[test]
fn the_framing_travels_in_fractions_not_in_units() {
    let f = Flight {
        from: Orbit {
            half_extent: 0.1,
            ..Orbit::default()
        },
        to: Orbit {
            half_extent: 10.0,
            ..Orbit::default()
        },
    };
    let mid = f.at(0.5).half_extent;
    assert!(
        (mid - 1.0).abs() < 1.0e-3,
        "a meio da viagem o enquadramento devia estar em 1,0 (a média geométrica) e está em {mid}"
    );
}

/// **O alvo caminha em linha reta** — ele é um ponto, não uma rotação.
#[test]
fn the_target_walks_in_a_straight_line() {
    let f = Flight {
        from: Orbit {
            target: [0.0, 0.0, 0.0],
            ..Orbit::default()
        },
        to: Orbit {
            target: [4.0, -2.0, 8.0],
            ..Orbit::default()
        },
    };
    let m = f.at(0.25).target;
    assert!(
        (m[0] - 1.0).abs() < 1.0e-5 && (m[1] + 0.5).abs() < 1.0e-5 && (m[2] - 2.0).abs() < 1.0e-5
    );
}

/// O ângulo entre duas orientações, em radianos — a régua dos gates acima.
fn angle(a: [f32; 4], b: [f32; 4]) -> f32 {
    2.0 * dot(a, b).abs().clamp(0.0, 1.0).acos()
}
