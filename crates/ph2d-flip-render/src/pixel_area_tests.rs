//! Gates da [`super`] — a área do pixel sob uma união de semi-planos.
//!
//! ⚠️ **O oráculo forte é o de FORÇA BRUTA** (`by_sampling`): ele conta pontos dentro da união e não
//! sabe nada de recorte nem de sapateiro. Os gates de forma fechada (eixo, 45°, quina) existem
//! porque um oráculo por amostragem tem erro próprio e não distingue *exato* de *quase*.

use super::*;

/// A cobertura por FORÇA BRUTA: conta sub-amostras dentro de alguma passagem.
fn by_sampling(planes: &[OutsidePlane], n: u32) -> f32 {
    let mut dentro = 0_u32;
    for j in 0..n {
        for i in 0..n {
            let u = [
                -0.5 + (i as f32 + 0.5) / n as f32,
                -0.5 + (j as f32 + 0.5) / n as f32,
            ];
            // Coberto = dentro de ALGUM (o de fora de todos é o descoberto).
            if planes
                .iter()
                .any(|p| p.n[0] * u[0] + p.n[1] * u[1] + p.sd < 0.0)
            {
                dentro += 1;
            }
        }
    }
    dentro as f32 / (n * n) as f32
}

fn set(planes: &[OutsidePlane]) -> PlaneSet {
    let mut s = PlaneSet::default();
    for p in planes {
        s.offer(*p);
    }
    s
}

/// ⭐ **A LEI ANTIGA É UM CASO PARTICULAR, e este gate é o que torna a wave segura.** Com a borda
/// paralela a um eixo, a área exata **é** a rampa `0,5 − sd` — então todo pixel de traço horizontal
/// ou vertical continua byte a byte onde estava, e o que muda é só o que estava errado.
#[test]
fn an_axis_aligned_edge_is_exactly_the_old_ramp() {
    for eixo in [[0.0_f32, 1.0], [0.0, -1.0], [1.0, 0.0], [-1.0, 0.0]] {
        for k in -80..=80 {
            let sd = k as f32 / 100.0;
            let esperado = (0.5 - sd).clamp(0.0, 1.0);
            let obtido = set(&[OutsidePlane { n: eixo, sd }]).coverage();
            assert!(
                (obtido - esperado).abs() < 1e-6,
                "eixo {eixo:?} sd={sd}: {obtido} != {esperado}"
            );
        }
    }
}

/// A 45° a resposta certa é a QUADRÁTICA, e é aqui que a rampa devia os 9,72/255 medidos no
/// produto. A forma fechada: com `s = −sd·√2`, a área coberta é `(s+1)²/2` para `s ≤ 0` e
/// `1 − (1−s)²/2` para `s ≥ 0`.
#[test]
fn a_45_degree_edge_is_the_exact_quadratic() {
    let d = core::f32::consts::FRAC_1_SQRT_2;
    let (mut pior_novo, mut pior_rampa) = (0.0_f32, 0.0_f32);
    for k in -70..=70 {
        let sd = k as f32 / 100.0;
        let s = -sd * core::f32::consts::SQRT_2;
        let exato = if s <= 0.0 {
            (s + 1.0) * (s + 1.0) / 2.0
        } else {
            1.0 - (1.0 - s) * (1.0 - s) / 2.0
        };
        let obtido = set(&[OutsidePlane { n: [d, d], sd }]).coverage();
        pior_novo = pior_novo.max((obtido - exato).abs());
        pior_rampa = pior_rampa.max(((0.5 - sd).clamp(0.0, 1.0) - exato).abs());
    }
    assert!(pior_novo < 1e-6, "o recorte erra {pior_novo} a 45 graus");
    // E o número que a wave existe para remover, afirmado no MESMO gate: sem ele, "exato" não tem
    // com o que ser comparado.
    assert!(
        pior_rampa * 255.0 > 9.0,
        "a rampa devia 9,72/255 a 45 graus e agora deve {}",
        pior_rampa * 255.0
    );
}

/// ⭐ **A QUINA — o defeito de 63,75/255.** Duas bordas perpendiculares com o centro do pixel
/// exatamente sobre as duas: a união cobre ¾ e o `min` dos SDFs dizia ½.
#[test]
fn a_right_angle_corner_uncovers_a_quarter() {
    let quina = set(&[
        OutsidePlane {
            n: [1.0, 0.0],
            sd: 0.0,
        },
        OutsidePlane {
            n: [0.0, 1.0],
            sd: 0.0,
        },
    ]);
    assert!(
        (quina.coverage() - 0.75).abs() < 1e-6,
        "a quina cobre {} e nao 0,75",
        quina.coverage()
    );
}

/// O recorte concorda com a contagem de pontos em cenas arbitrárias — 1, 2 e 3 bordas, ângulos e
/// distâncias irregulares (nada de múltiplos de 45°, que são justamente onde uma implementação
/// errada tem chance de acertar por simetria).
#[test]
fn the_clip_agrees_with_brute_force() {
    const N: u32 = 256;
    let dirs = [
        [1.0_f32, 0.0],
        [0.6, 0.8],
        [-0.28, 0.96],
        [-0.8, -0.6],
        [0.96, -0.28],
    ];
    let mut pior = 0.0_f32;
    for (ia, a) in dirs.iter().enumerate() {
        for (ib, b) in dirs.iter().enumerate() {
            for k in -6..=6 {
                for m in -6..=6 {
                    let planes = [
                        OutsidePlane {
                            n: *a,
                            sd: k as f32 / 10.0,
                        },
                        OutsidePlane {
                            n: *b,
                            sd: m as f32 / 10.0,
                        },
                    ];
                    let d = (set(&planes).coverage() - by_sampling(&planes, N)).abs();
                    assert!(
                        d < 4.0 / N as f32,
                        "planos {ia}/{ib} sd={k}/{m}: recorte {} vs amostragem {}",
                        set(&planes).coverage(),
                        by_sampling(&planes, N)
                    );
                    pior = pior.max(d);
                }
            }
        }
    }
    assert!(pior < 4.0 / N as f32, "pior desvio {pior}");
}

/// Um plano longe demais não corta nada, e um plano que engole o quadrado cobre tudo. São as duas
/// bordas do [`PLANE_REACH`], e é o que torna seguro NÃO tratar o caso degenerado (`dist = 0`, sem
/// normal): ali `sd = −r`, muito além do alcance.
#[test]
fn a_plane_out_of_reach_decides_the_pixel_by_itself() {
    let longe = set(&[OutsidePlane {
        n: [0.6, 0.8],
        sd: 0.71,
    }]);
    assert!(
        longe.coverage() < 1e-6,
        "descoberto, mas deu {}",
        longe.coverage()
    );
    let engole = set(&[OutsidePlane {
        n: [0.6, 0.8],
        sd: -0.71,
    }]);
    assert!(
        (engole.coverage() - 1.0).abs() < 1e-6,
        "coberto, mas deu {}",
        engole.coverage()
    );
    // E um plano fora de alcance nem ocupa vaga: com ele adiante, os do miolo ainda entram.
    let mut s = PlaneSet::default();
    s.offer(OutsidePlane {
        n: [1.0, 0.0],
        sd: 9.0,
    });
    for k in 0..MAX_PLANES {
        s.offer(OutsidePlane {
            n: [0.0, 1.0],
            sd: 0.4 - k as f32 * 0.1,
        });
    }
    let menor = 0.4 - (MAX_PLANES - 1) as f32 * 0.1;
    assert!(
        (s.coverage() - (0.5 - menor)).abs() < 1e-6,
        "o plano descartavel roubou vaga: {}",
        s.coverage()
    );
}

/// Quando sobram planos, ficam os que mais recortam (menor `sd`) — e o resultado **não depende da
/// ordem de chegada**, que é o que o percurso não pode garantir.
#[test]
fn the_set_keeps_the_planes_that_clip_most_whatever_the_order() {
    let muitos: [OutsidePlane; 6] = [
        OutsidePlane {
            n: [1.0, 0.0],
            sd: 0.30,
        },
        OutsidePlane {
            n: [0.0, 1.0],
            sd: -0.10,
        },
        OutsidePlane {
            n: [-1.0, 0.0],
            sd: 0.45,
        },
        OutsidePlane {
            n: [0.0, -1.0],
            sd: 0.05,
        },
        OutsidePlane {
            n: [0.6, 0.8],
            sd: 0.60,
        },
        OutsidePlane {
            n: [-0.6, -0.8],
            sd: 0.20,
        },
    ];
    let direto = set(&muitos).coverage();
    let mut invertido: [OutsidePlane; 6] = muitos;
    invertido.reverse();
    assert!(
        (direto - set(&invertido).coverage()).abs() < 1e-6,
        "a ordem de chegada mudou a resposta"
    );
    // Os escolhidos são os de menor `sd` — e o oráculo é DERIVADO do `MAX_PLANES`, senão o gate
    // apodrece calado no dia em que a medição mover o teto (foi o que aconteceu: nasceu com 4).
    let mut ordenados = muitos;
    ordenados.sort_by(|a, b| a.sd.total_cmp(&b.sd));
    let esperado = set(&ordenados[..MAX_PLANES]).coverage();
    assert!(
        (direto - esperado).abs() < 1e-6,
        "escolheu outros quatro: {direto} != {esperado}"
    );
}
