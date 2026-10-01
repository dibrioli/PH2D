//! ⭐⭐ **A QUINA INTERIOR do traço esticado é EXACTA** (doc 121 §9.4) — a faixa contra a ÁREA
//! VERDADEIRA, e não contra o Vello.
//!
//! O traço de uma cópia não conforme é a união dos troços (rectângulos de meia largura `r` à volta de
//! cada segmento) e das juntas. Pelo lado de DENTRO de uma quina os dois rectângulos SOBREPÕEM-SE, e
//! uma rasterização por área que os SOMA (o Vello sobre o contorno do kurbo, que passa pelo vértice;
//! e o passe até 01/10, com uma peça por troço) conta a sobreposição DUAS vezes num pixel de borda —
//! o canto de dentro sai escuro de mais. A FAIXA acaba os dois troços na mesma bissectriz, logo não
//! há sobreposição e a área é a verdadeira.
//!
//! ⚠️ **É por isto que a paridade da estrela esticada contra o Vello subiu de `40` para `73` de alfa**
//! (`paridade_com_o_vello::barra`): os pixels que mudaram são estes cantos, e quem se afasta da área
//! verdadeira é o Vello. ⇒ a prova de que a divergência é a CURA e não um defeito vive aqui, com o
//! CONTROLO de que a fixtura contém o fenómeno: a soma dos troços lê o canto `≥ 0,15` acima da área
//! verdadeira. ⚠️ A fixtura é a cópia `8` da família da paridade, onde a réplica da conta do shader
//! achou o pior canto (`0,30` de sobreconta no pixel `(389, 299)`) — uma `V` feita à mão não o
//! continha (a régua leu `0,03`). E o TRAÇO FINO (`0,01`), cópia `59` da família dele: lá o canto lê
//! `0,693` verdadeiro, `0,694` na faixa e `1,0` somado (a paridade dele subiu de `41` para `67`).

use super::paridade_com_o_vello::{Copia, Forma, LADO, basis, esticadas, estrela, gpu, pelo_passe};
use ph2d_shape_gpu::FillRule;
use ph2d_vector::{Join, Stroke};

const LIMITE: f64 = 4.0;

type P = [f64; 2];

fn sub(a: P, b: P) -> P {
    [a[0] - b[0], a[1] - b[1]]
}

fn cruz(a: P, b: P) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}

/// O ponto está no rectângulo de meia largura `r` à volta de `a → b` (sem pontas)?
fn no_troco(p: P, a: P, b: P, r: f64) -> bool {
    let d = sub(b, a);
    let l = d[0].hypot(d[1]);
    let u = [d[0] / l, d[1] / l];
    let v = sub(p, a);
    let ao_longo = v[0] * u[0] + v[1] * u[1];
    (0.0..=l).contains(&ao_longo) && cruz(u, v).abs() <= r
}

/// A cunha da esquadria em `b` (`b`, as duas normais de FORA e a ponta), ou nada se ela passa do
/// limite — o mesmo teste do Vello.
fn cunha(a: P, b: P, c: P, r: f64) -> Option<[P; 4]> {
    let un = |v: P| {
        let l = v[0].hypot(v[1]);
        [v[0] / l, v[1] / l]
    };
    let (u, w) = (un(sub(b, a)), un(sub(c, b)));
    let dt = u[0] * w[0] + u[1] * w[1];
    if 2.0 > (1.0 + dt) * LIMITE * LIMITE {
        return None;
    }
    let lado = if cruz(u, w) > 0.0 { -1.0 } else { 1.0 };
    let n0 = [-u[1] * r * lado, u[0] * r * lado];
    let n1 = [-w[1] * r * lado, w[0] * r * lado];
    let m = [
        b[0] + (n0[0] + n1[0]) / (1.0 + dt),
        b[1] + (n0[1] + n1[1]) / (1.0 + dt),
    ];
    Some([
        b,
        [b[0] + n0[0], b[1] + n0[1]],
        m,
        [b[0] + n1[0], b[1] + n1[1]],
    ])
}

fn na_cunha(p: P, q: &[P; 4]) -> bool {
    let s: Vec<f64> = (0..4)
        .map(|i| cruz(sub(q[(i + 1) % 4], q[i]), sub(p, q[i])))
        .collect();
    s.iter().all(|&x| x >= 0.0) || s.iter().all(|&x| x <= 0.0)
}

#[test]
#[ignore = "precisa de adapter de GPU"]
fn a_quina_interior_e_a_area_verdadeira() {
    // (largura, a família da paridade, a cópia onde a réplica achou o pior canto)
    for (largura, n, semente, copia) in [(0.06, 40, 6, 8), (0.01, 60, 10, 59)] {
        if !uma_copia(
            largura,
            esticadas(n, 40.0, 220.0, semente).swap_remove(copia),
        ) {
            return;
        }
    }
}

/// Mede UMA cópia da estrela esticada; `false` sem adapter.
fn uma_copia(largura: f64, fonte: Copia) -> bool {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter — o gate não correu");
        return false;
    };
    let est = estrela();
    let forma = Forma {
        bp: &est,
        linha: None,
        regra: FillRule::NonZero,
        marcas: None,
        traco: Some((
            Stroke::new(largura).with_join(Join::Miter),
            [0.0, 0.0, 0.0, 1.0],
        )),
    };
    // Uma cópia SÓ, e o preenchimento TRANSPARENTE: o alfa do pixel é a cobertura do traço.
    let cs = [Copia {
        tint: [0.0, 0.0, 0.0, 0.0],
        ..fonte
    }];
    let c = &cs[0];
    let img = pelo_passe(&gpu, &forma, &cs, wgpu::TextureFormat::Rgba16Float);
    let alfa = |x: u32, y: u32| f64::from(img[((y * LADO + x) * 4 + 3) as usize]) / 255.0;

    let [b0, b1, b2, b3] = basis(c.ang).map(f64::from);
    let (sx, sy) = (f64::from(c.lado), f64::from(c.lado * c.aspecto));
    let no_ecra = |x: f64, y: f64| -> P {
        [
            b0 * sx * x + b2 * sy * y + f64::from(c.pos[0]),
            b1 * sx * x + b3 * sy * y + f64::from(c.pos[1]),
        ]
    };
    let pts: Vec<P> = est
        .elements()
        .iter()
        .filter_map(|e| e.end_point())
        .map(|p| no_ecra(p.x, p.y))
        .collect();
    let n = pts.len();
    let r = largura * 0.5 * (sx * sy).sqrt();
    let cunhas: Vec<[P; 4]> = (0..n)
        .filter_map(|i| cunha(pts[(i + n - 1) % n], pts[i], pts[(i + 1) % n], r))
        .collect();

    const N: u32 = 32;
    let n2 = f64::from(N * N);
    let mut medidos = 0;
    let mut maior_sobreconta: f64 = 0.0;
    let mut pior: f64 = 0.0;
    // Os vértices CÔNCAVOS da estrela (os ímpares): é do lado de fora da estrela que os troços se
    // sobrepõem.
    for v in (1..n).step_by(2).map(|i| pts[i]) {
        #[expect(clippy::cast_possible_truncation, reason = "pixels de um alvo de 512")]
        let (vx, vy, alc) = (v[0] as i64, v[1] as i64, (3.0 * r) as i64);
        for py in vy - alc..=vy + alc {
            for px in vx - alc..=vx + alc {
                let (mut uniao, mut soma) = (0u32, 0u32);
                for i in 0..N {
                    for j in 0..N {
                        #[expect(clippy::cast_precision_loss, reason = "pixels de um alvo de 512")]
                        let p = [
                            px as f64 + (f64::from(i) + 0.5) / f64::from(N),
                            py as f64 + (f64::from(j) + 0.5) / f64::from(N),
                        ];
                        let troços = (0..n)
                            .filter(|&k| no_troco(p, pts[k], pts[(k + 1) % n], r))
                            .count();
                        let em_cunha = cunhas.iter().filter(|q| na_cunha(p, q)).count();
                        let k = u32::try_from(troços + em_cunha).expect("poucas peças");
                        uniao += u32::from(k > 0);
                        soma += k;
                    }
                }
                let verdade = f64::from(uniao) / n2;
                let sobre = (f64::from(soma) / n2).min(1.0);
                // Só os pixels de BORDA onde as duas leis discordam — os únicos onde há sobreposição.
                if sobre - verdade < 0.05 {
                    continue;
                }
                medidos += 1;
                maior_sobreconta = maior_sobreconta.max(sobre - verdade);
                #[expect(clippy::cast_sign_loss, reason = "pixels dentro do alvo")]
                let g = alfa(px as u32, py as u32);
                pior = pior.max((g - verdade).abs());
            }
        }
    }
    eprintln!(
        "quina interior (largura {largura}): {medidos} pixels de borda · sobreconta da soma ate' {maior_sobreconta:.3} · pior desvio da faixa {pior:.3}"
    );
    assert!(
        medidos >= 2,
        "controlo: a fixtura tem de ter pixels de borda nas quinas ({medidos})"
    );
    assert!(
        maior_sobreconta >= 0.15,
        "controlo: a soma dos troços tem de SOBRECONTAR o canto (lê {maior_sobreconta:.3})"
    );
    assert!(
        pior <= 0.03,
        "a quina interior desvia {pior:.3} da área verdadeira — a faixa deixou de a fechar"
    );
    true
}
