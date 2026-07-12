//! Testes do roteador.
//!
//! O que se testa aqui não é "compilou": é que a rota é **correta** (não entra em forma
//! nenhuma, sai e chega pelo lado certo) e **bonita** (dobra no meio do vão, não raspa a
//! caixa) — e que ela é uma **função total**, porque roda a cada frame enquanto o usuário
//! arrasta uma caixa, e uma rota que falha é uma linha que some.

use super::*;

const J: f64 = 1.0;

fn boxes(a: Aabb, b: Aabb) -> Vec<Aabb> {
    vec![a, b]
}

/// Duas caixas lado a lado, saindo da direita da 1ª e entrando pela esquerda da 2ª.
fn side_by_side(gap: f64) -> (RouteInput<'static>, Vec<Aabb>) {
    let a = Aabb::new([0.0, 0.0], [4.0, 3.0]);
    let b = Aabb::new([4.0 + gap, 0.0], [8.0 + gap, 3.0]);
    let obs = boxes(a, b);
    let input = RouteInput {
        start: EndSpec {
            at: [4.0, 1.5],
            dir: Dir::East,
        },
        end: EndSpec {
            at: [4.0 + gap, 1.5],
            dir: Dir::West,
        },
        kind: RouteKind::Orthogonal,
        jetty: J,
        obstacles: &[],
        spread: 0.0,
        self_loop: None,
    };
    (input, obs)
}

/// Todos os segmentos de uma rota ortogonal são horizontais ou verticais. Se um sair na
/// diagonal, não é uma rota ortogonal — é um rabisco.
fn assert_orthogonal(pts: &[[f64; 2]]) {
    for w in pts.windows(2) {
        let (dx, dy) = ((w[1][0] - w[0][0]).abs(), (w[1][1] - w[0][1]).abs());
        assert!(
            dx < 1e-6 || dy < 1e-6,
            "segmento diagonal {:?} -> {:?} numa rota ORTOGONAL",
            w[0],
            w[1]
        );
    }
}

/// **A rota nunca entra numa forma.** É a invariante que define o roteador: uma linha que
/// atravessa a caixa que ela conecta não conecta nada, risca.
fn assert_avoids(pts: &[[f64; 2]], obstacles: &[Aabb]) {
    for w in pts.windows(2) {
        // Amostra o segmento (é ortogonal, então amostrar basta).
        for k in 1..20 {
            let t = f64::from(k) / 20.0;
            let p = [
                w[0][0] + (w[1][0] - w[0][0]) * t,
                w[0][1] + (w[1][1] - w[0][1]) * t,
            ];
            for (i, o) in obstacles.iter().enumerate() {
                assert!(
                    !o.contains(p),
                    "a rota entrou na caixa {i} em {p:?} — o segmento {:?}->{:?} atravessa a forma",
                    w[0],
                    w[1]
                );
            }
        }
    }
}

/// **O caso mais comum do mundo**: duas caixas lado a lado, alinhadas. A rota é uma RETA — e
/// isso é o custo de dobra funcionando (qualquer desvio custaria uma dobra sem ganhar nada).
#[test]
fn two_aligned_boxes_get_a_straight_line_not_a_detour() {
    let (input, obs) = side_by_side(4.0);
    let r = route(&RouteInput {
        obstacles: &obs,
        ..input
    });
    assert_orthogonal(&r);
    assert_avoids(&r, &obs);
    assert_eq!(
        r.len(),
        2,
        "caixas alinhadas: a rota e uma RETA de dois pontos, nao um zigue-zague: {r:?}"
    );
    assert!(
        r.iter().all(|p| (p[1] - 1.5).abs() < 1e-6),
        "a reta fica na altura das duas saidas: {r:?}"
    );
}

/// **O "Z" dobra no MEIO do vão — não raspando a caixa.**
///
/// Este é *o* teste do módulo. Entre duas caixas desalinhadas, TODA linha vertical dentro do
/// vão dá uma rota com o mesmo comprimento e as mesmas dobras: elas são **todas ótimas**. Um
/// A\* que desempatasse arbitrariamente escolheria uma qualquer, e ela grudaria na borda de
/// uma das caixas — correto, e feio.
///
/// O desempate por centralidade é o que separa "correto" de "bonito", e nenhum paper o
/// menciona. Sem ele este teste fica vermelho **e o algoritmo continua ótimo** — que é
/// precisamente por que ele precisa existir.
#[test]
fn the_z_bends_in_the_middle_of_the_gap_not_against_a_box() {
    let gap = 6.0;
    let a = Aabb::new([0.0, 0.0], [4.0, 3.0]);
    let b = Aabb::new([4.0 + gap, 5.0], [8.0 + gap, 8.0]); // desalinhada em y
    let obs = boxes(a, b);
    let r = route(&RouteInput {
        start: EndSpec {
            at: [4.0, 1.5],
            dir: Dir::East,
        },
        end: EndSpec {
            at: [4.0 + gap, 6.5],
            dir: Dir::West,
        },
        kind: RouteKind::Orthogonal,
        jetty: J,
        obstacles: &obs,
        spread: 0.0,
        self_loop: None,
    });
    assert_orthogonal(&r);
    assert_avoids(&r, &obs);

    // O trecho VERTICAL do Z: onde ele está?
    let vertical = r
        .windows(2)
        .find(|w| (w[1][0] - w[0][0]).abs() < 1e-6 && (w[1][1] - w[0][1]).abs() > 1e-6)
        .expect("o Z tem um trecho vertical");
    let x = vertical[0][0];
    // O vão vai de x=4 (borda da 1ª) a x=10 (borda da 2ª). O meio é 7.
    let mid = 4.0 + gap / 2.0;
    assert!(
        (x - mid).abs() < 1.0,
        "o Z dobrou em x={x}, mas o meio do vao e {mid} — a rota esta raspando uma caixa \
         (o desempate por centralidade nao esta funcionando)"
    );
}

/// **A linha sai pelo lado certo e entra pelo lado certo.** O stub (jetty) é o que impede a
/// rota de dobrar colada na caixa — e a restrição de direção é DURA, não uma penalidade.
#[test]
fn the_route_leaves_and_arrives_along_the_declared_directions() {
    let (input, obs) = side_by_side(6.0);
    let r = route(&RouteInput {
        obstacles: &obs,
        end: EndSpec {
            at: [10.0, 6.0], // desalinhado, para forçar dobras
            dir: Dir::West,
        },
        ..input
    });
    // O primeiro passo é para LESTE (a direção de saída), e anda pelo menos o jetty.
    assert!(
        r[1][0] - r[0][0] >= J - 1e-6 && (r[1][1] - r[0][1]).abs() < 1e-6,
        "a rota tem de SAIR para leste, andando ao menos o jetty: {:?} -> {:?}",
        r[0],
        r[1]
    );
    // O último passo CHEGA vindo de oeste (ou seja: andando para leste).
    let n = r.len();
    assert!(
        r[n - 1][0] - r[n - 2][0] >= J - 1e-6 && (r[n - 1][1] - r[n - 2][1]).abs() < 1e-6,
        "a rota tem de CHEGAR pelo oeste do alvo: {:?} -> {:?}",
        r[n - 2],
        r[n - 1]
    );
}

/// **A rota não passa por dentro das caixas que ela liga** — mesmo quando o caminho curto
/// seria atravessá-las. Aqui as saídas apontam para LADOS OPOSTOS ao destino, então a linha
/// tem de contornar as duas.
#[test]
fn the_route_goes_around_the_boxes_it_connects() {
    let a = Aabb::new([0.0, 0.0], [4.0, 3.0]);
    let b = Aabb::new([10.0, 0.0], [14.0, 3.0]);
    let obs = boxes(a, b);
    let r = route(&RouteInput {
        // Sai para OESTE (longe do alvo) e entra pelo LESTE (o lado de trás do alvo): a rota
        // é obrigada a dar a volta nas duas caixas.
        start: EndSpec {
            at: [0.0, 1.5],
            dir: Dir::West,
        },
        end: EndSpec {
            at: [14.0, 1.5],
            dir: Dir::East,
        },
        kind: RouteKind::Orthogonal,
        jetty: J,
        obstacles: &obs,
        spread: 0.0,
        self_loop: None,
    });
    assert_orthogonal(&r);
    assert_avoids(&r, &obs);
    assert!(r.len() >= 4, "dar a volta exige dobras: {r:?}");
}

/// **A função é TOTAL.** Ela roda a cada frame enquanto o usuário arrasta uma caixa: em algum
/// frame as duas caixas vão se sobrepor, uma vai estar dentro da outra, e as pontas vão
/// coincidir. Nenhum desses casos pode devolver vazio, `NaN`, ou entrar em pânico — a linha
/// tem de continuar lá, ainda que feia.
#[test]
fn the_router_is_a_total_function_even_when_the_boxes_overlap() {
    let cases: [(Aabb, Aabb, [f64; 2], [f64; 2]); 4] = [
        // sobrepostas
        (
            Aabb::new([0.0, 0.0], [4.0, 3.0]),
            Aabb::new([2.0, 1.0], [6.0, 4.0]),
            [4.0, 1.5],
            [2.0, 2.5],
        ),
        // uma DENTRO da outra
        (
            Aabb::new([0.0, 0.0], [10.0, 10.0]),
            Aabb::new([4.0, 4.0], [6.0, 6.0]),
            [10.0, 5.0],
            [4.0, 5.0],
        ),
        // idênticas
        (
            Aabb::new([0.0, 0.0], [4.0, 3.0]),
            Aabb::new([0.0, 0.0], [4.0, 3.0]),
            [4.0, 1.5],
            [0.0, 1.5],
        ),
        // pontas coincidentes
        (
            Aabb::new([0.0, 0.0], [4.0, 3.0]),
            Aabb::new([4.0, 0.0], [8.0, 3.0]),
            [4.0, 1.5],
            [4.0, 1.5],
        ),
    ];
    for (i, (a, b, p0, p1)) in cases.into_iter().enumerate() {
        let obs = boxes(a, b);
        for kind in [RouteKind::Orthogonal, RouteKind::Straight] {
            let r = route(&RouteInput {
                start: EndSpec {
                    at: p0,
                    dir: Dir::East,
                },
                end: EndSpec {
                    at: p1,
                    dir: Dir::West,
                },
                kind,
                jetty: J,
                obstacles: &obs,
                spread: 0.0,
                self_loop: None,
            });
            assert!(r.len() >= 2, "caso {i}: rota vazia — a linha sumiu da tela");
            assert!(
                r.iter().all(|p| p[0].is_finite() && p[1].is_finite()),
                "caso {i}: NaN na rota {r:?} — um NaN contamina a bbox e o gizmo some"
            );
        }
    }
}

/// **Determinismo.** A mesma entrada dá a mesma rota, byte a byte — senão a CI que compara
/// hashes de replay quebra, e o undo passa a "mexer" o diagrama sozinho.
#[test]
fn the_same_input_always_yields_the_same_route() {
    let (input, obs) = side_by_side(5.0);
    let a = route(&RouteInput {
        obstacles: &obs,
        end: EndSpec {
            at: [9.0, 7.0],
            dir: Dir::West,
        },
        ..input
    });
    let b = route(&RouteInput {
        obstacles: &obs,
        end: EndSpec {
            at: [9.0, 7.0],
            dir: Dir::West,
        },
        ..input
    });
    assert_eq!(a, b, "o roteador nao e deterministico");
}

/// **O peso de dobra é adimensional.** Escalar o mundo inteiro por 100 tem de dar a MESMA
/// rota (escalada) — se o peso fosse absoluto (o `50` da libavoid), a rota grande viraria uma
/// reta e a pequena um zigue-zague. Era este o erro fatal a evitar.
#[test]
fn the_bend_penalty_is_scale_invariant() {
    let shape = |k: f64| {
        let a = Aabb::new([0.0, 0.0], [4.0 * k, 3.0 * k]);
        let b = Aabb::new([10.0 * k, 5.0 * k], [14.0 * k, 8.0 * k]);
        let obs = vec![a, b];
        route(&RouteInput {
            start: EndSpec {
                at: [4.0 * k, 1.5 * k],
                dir: Dir::East,
            },
            end: EndSpec {
                at: [10.0 * k, 6.5 * k],
                dir: Dir::West,
            },
            kind: RouteKind::Orthogonal,
            jetty: J * k,
            obstacles: &obs,
            spread: 0.0,
            self_loop: None,
        })
    };
    let small = shape(1.0);
    let big = shape(100.0);
    assert_eq!(
        small.len(),
        big.len(),
        "a mesma rota em escalas diferentes tem de ter o mesmo NUMERO DE DOBRAS \
         (pequena: {small:?}, grande: {big:?}) — o peso de dobra nao e adimensional"
    );
    for (s, b) in small.iter().zip(&big) {
        assert!(
            (s[0] * 100.0 - b[0]).abs() < 1e-6 && (s[1] * 100.0 - b[1]).abs() < 1e-6,
            "a rota grande tem de ser a pequena x100: {s:?} vs {b:?}"
        );
    }
}

/// O **laço** (uma forma que aponta para si mesma) sai e volta pela mesma face, e fica FORA
/// da forma.
#[test]
fn a_self_loop_leaves_and_returns_without_entering_the_shape() {
    let bbox = Aabb::new([0.0, 0.0], [4.0, 3.0]);
    let r = route(&RouteInput {
        start: EndSpec {
            at: [4.0, 1.5],
            dir: Dir::East,
        },
        end: EndSpec {
            at: [4.0, 1.5],
            dir: Dir::East,
        },
        kind: RouteKind::Orthogonal,
        jetty: J,
        obstacles: &[bbox],
        spread: 0.0,
        self_loop: Some(bbox),
    });
    assert!(r.len() >= 4, "o laco tem de dar a volta: {r:?}");
    assert_avoids(&r, &[bbox]);
    // Sai e volta pelo mesmo lado (o leste): todos os pontos estão à direita da forma.
    assert!(
        r.iter().all(|p| p[0] >= bbox.max[0] - 1e-6),
        "o laco saiu pela face leste: todo ponto fica em x >= {}: {r:?}",
        bbox.max[0]
    );
}
