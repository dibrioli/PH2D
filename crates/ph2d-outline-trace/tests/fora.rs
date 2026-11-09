//! **A lei do dono (09/11): o contorno pode sair PARA FORA da arte até Detail px, nunca para dentro.**
//! Detail é o desvio máximo; numa arte redonda (moeda) Detail maior dá MENOS pontos. Números: `README.md`.

use ph2d_outline_trace::{Mask, Outline, Params, Q4, trace};
use sha2::{Digest, Sha256};

fn mask(w: u32, h: u32, f: impl Fn(i64, i64) -> bool) -> Mask {
    let bits = (0..i64::from(h))
        .flat_map(|y| (0..i64::from(w)).map(move |x| (x, y)))
        .map(|(x, y)| u8::from(f(x, y)))
        .collect();
    Mask { w, h, bits }
}

/// Distância² (Q4², ×4 para ficar inteira) do centro do píxel `(x, y)` a `(cx, cy)` em meios-píxeis.
fn d2(x: i64, y: i64, c: i64) -> i64 {
    (2 * x + 1 - 2 * c).pow(2) + (2 * y + 1 - 2 * c).pow(2)
}

/// Moeda: disco de raio 20 px — convexa e lisa, o caso onde a lei antiga não tirava ponto nenhum.
fn moeda() -> Mask {
    mask(48, 48, |x, y| d2(x, y, 24) <= 4 * 400)
}

/// Argola: disco de raio 20 com buraco de raio 8 — «fora da arte» é também PARA DENTRO do buraco.
fn argola() -> Mask {
    mask(48, 48, |x, y| (4 * 64..=4 * 400).contains(&d2(x, y, 24)))
}

/// Pente de 16 dentes para baixo (o de `tecto.rs`): reentrâncias que um desvio para fora não pode tapar
/// além de Detail.
fn pente() -> Mask {
    mask(99, 24, |x, y| y < 12 || x % 6 >= 3)
}

fn at(d_px: i32) -> Params {
    Params {
        detail_q4: u16::try_from(d_px * Q4).expect("q4"),
        min_island_px: 0,
    }
}

const DETAILS_PX: [i32; 4] = [0, 2, 4, 8];

/// Píxeis opacos cujo centro fica fora do contorno.
fn fora(m: &Mask, o: &Outline) -> usize {
    (0..i64::from(m.h))
        .flat_map(|y| (0..i64::from(m.w)).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            let c = [x as i32 * Q4 + Q4 / 2, y as i32 * Q4 + Q4 / 2];
            m.get(x, y) && !o.contains_q4(c)
        })
        .count()
}

/// Distância² (Q4²) de `p` ao quadrado opaco mais próximo.
fn dist2_arte(m: &Mask, p: [i64; 2]) -> i64 {
    let q = i64::from(Q4);
    (0..i64::from(m.h))
        .flat_map(|y| (0..i64::from(m.w)).map(move |x| (x, y)))
        .filter(|&(x, y)| m.get(x, y))
        .map(|(x, y)| {
            let dx = (x * q - p[0]).max(p[0] - (x + 1) * q).max(0);
            let dy = (y * q - p[1]).max(p[1] - (y + 1) * q).max(0);
            dx * dx + dy * dy
        })
        .min()
        .unwrap_or(i64::MAX)
}

/// O maior desvio (Q4) de um ponto do contorno à arte: cada vértice e 7 pontos por aresta (truncados,
/// < 1 Q4 de erro).
fn desvio(m: &Mask, o: &Outline) -> i64 {
    let mut worst = 0;
    for r in &o.rings {
        for i in 0..r.len() {
            let (a, b) = (r[i], r[(i + 1) % r.len()]);
            for t in 0..8 {
                let p = [
                    i64::from(a[0]) + (i64::from(b[0]) - i64::from(a[0])) * t / 8,
                    i64::from(a[1]) + (i64::from(b[1]) - i64::from(a[1])) * t / 8,
                ];
                worst = worst.max(dist2_arte(m, p));
            }
        }
    }
    worst.isqrt()
}

#[test]
fn na_moeda_detail_8_tem_menos_pontos_que_detail_2() {
    let m = moeda();
    let n: Vec<usize> = DETAILS_PX
        .iter()
        .map(|d| trace(&m, at(*d)).vertex_count())
        .collect();
    println!("moeda r=20: pontos a Detail {DETAILS_PX:?} px = {n:?}");
    assert!(n[3] < n[1], "Detail 8 ({}) × 2 ({})", n[3], n[1]);
    assert!(n[2] < n[1] && n[1] < n[0], "{n:?}");
}

#[test]
fn nenhum_pixel_da_arte_fica_fora_e_o_desvio_nao_passa_do_detail() {
    for (nome, m) in [("moeda", moeda()), ("argola", argola()), ("pente", pente())] {
        for d in DETAILS_PX {
            let o = trace(&m, at(d));
            let (f, dv) = (fora(&m, &o), desvio(&m, &o));
            println!(
                "{nome} Detail {d} px: anéis {} pontos {} fora {f} desvio {dv} q4",
                o.rings.len(),
                o.vertex_count()
            );
            assert_eq!(f, 0, "{nome} Detail {d}: cortou a arte");
            // +1 Q4: o vértice arredondado ao Q4 (≤ √2/2) e a amostra truncada (< 1).
            assert!(
                dv <= i64::from(d * Q4) + 1,
                "{nome} Detail {d}: desvio {dv} q4"
            );
        }
    }
    let o = trace(&argola(), at(8));
    assert_eq!(o.rings.len(), 2, "o buraco fica");
}

#[test]
fn o_contorno_da_moeda_e_byte_estavel() {
    let mut h = Sha256::new();
    for d in DETAILS_PX {
        h.update(trace(&moeda(), at(d)).to_bytes());
        h.update(trace(&argola(), at(d)).to_bytes());
    }
    let hex: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(
        hex,
        "199e67a162b2f6ebb0f751f797380d79d6e590b687a191ad110344f7027fd821"
    );
}

/// O retraço do pente do tecto (W0): o guarda dos centros de píxel pelas FILAS (o produto) × perguntando a
/// todo anel (a referência), intercalados no mesmo processo, `--profile smoke`, mínimo de 15. `README.md` §Ceilings.
#[test]
#[ignore = "timing: --profile smoke, run by name"]
fn tempo_do_pente_do_tecto() {
    use ph2d_outline_trace::{convex_pieces, trace_reference_guard};
    for dentes in [16u32, 64, 128, 256] {
        let m = mask(dentes * 6 + 3, 24, |x, y| y < 12 || x % 6 >= 3);
        let mut best = [u128::MAX; 2];
        let mut n = 0;
        for volta in 0..30 {
            let ref_primeiro = volta % 2 == 1;
            for v in 0..2 {
                let brute = (v == 0) == ref_primeiro;
                let t = std::time::Instant::now();
                let o = if brute {
                    trace_reference_guard(&m, at(2))
                } else {
                    trace(&m, at(2))
                };
                n = o.vertex_count();
                std::hint::black_box(convex_pieces(&o, 32));
                let s = &mut best[usize::from(brute)];
                *s = (*s).min(t.elapsed().as_micros());
            }
        }
        println!(
            "pente {dentes}: {n} pontos, trace + peças: filas {} us · todo anel {} us",
            best[0], best[1]
        );
    }
}

/// As filas dão os MESMOS bytes que a referência (todo anel), em toda máscara e todo Detail de 0 a 8 px.
#[test]
fn o_guarda_por_filas_e_byte_igual_ao_de_todo_anel() {
    use ph2d_outline_trace::trace_reference_guard;
    let mut n = 0;
    for m in [
        moeda(),
        argola(),
        pente(),
        mask(64 * 6 + 3, 24, |x, y| y < 12 || x % 6 >= 3),
    ] {
        for d in (0..=128).step_by(4) {
            let p = Params {
                detail_q4: d,
                min_island_px: 0,
            };
            assert_eq!(trace(&m, p), trace_reference_guard(&m, p), "Detail {d} q4");
            n += 1;
        }
    }
    println!("{n} traçados byte-iguais");
}
