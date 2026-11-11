//! **O tecto do contorno (W0, plano f2): acima dele o Detail SOBE, nunca corta a arte.**
//! Números e recurso de cada tecto: `README.md` §Ceilings.

use ph2d_outline_trace::{MAX_PIECES, MAX_POINTS, Mask, Params, Q4, trace, trace_fitted};

/// Um pente de `dentes` dentes para baixo (barra 12 px, dente 3 px × 12 px, vão 3 px).
fn pente(dentes: u32) -> Mask {
    let (dente, vao, barra, perna) = (3u32, 3u32, 12u32, 12u32);
    let w = dentes * (dente + vao) + vao;
    let h = barra + perna;
    let bits = (0..h)
        .flat_map(|y| (0..w).map(move |x| u8::from(y < barra || x % (dente + vao) >= vao)))
        .collect();
    Mask { w, h, bits }
}

fn asked(detail_q4: u16) -> Params {
    Params {
        detail_q4,
        min_island_px: 0,
    }
}

/// Todo pixel opaco tem o centro dentro do contorno.
fn cobre_a_arte(m: &Mask, o: &ph2d_outline_trace::Outline) -> bool {
    (0..m.h).all(|y| {
        (0..m.w).all(|x| {
            let centro = [x as i32 * Q4 + Q4 / 2, y as i32 * Q4 + Q4 / 2];
            !m.get(i64::from(x), i64::from(y)) || o.contains_q4(centro)
        })
    })
}

#[test]
fn acima_do_tecto_o_detail_sobe_ao_menor_que_cabe() {
    let m = pente(200);
    let pedido = asked(2 * Q4 as u16);
    let cru = trace(&m, pedido);
    assert!(
        cru.vertex_count() > MAX_POINTS,
        "a fixtura tem de estar ACIMA do tecto: {} pontos",
        cru.vertex_count()
    );
    let f = trace_fitted(&m, pedido);
    let pontos = f.outline.vertex_count();
    eprintln!(
        "pente 200: {} → {pontos} pontos, {} peças, Detail {} → {} q4",
        cru.vertex_count(),
        f.pieces.len(),
        pedido.detail_q4,
        f.params.detail_q4
    );
    assert!(f.raised(pedido), "o Detail não subiu");
    assert!(pontos <= MAX_POINTS && f.pieces.len() <= MAX_PIECES);
    assert!(cobre_a_arte(&m, &f.outline), "subir o Detail cortou a arte");
    // O MENOR que cabe: um passo abaixo já não cabe.
    let abaixo = trace(&m, asked(f.params.detail_q4 - 1));
    let pecas_abaixo = ph2d_outline_trace::convex_pieces(&abaixo, f.params.detail_q4 - 1).len();
    assert!(
        abaixo.vertex_count() > MAX_POINTS || pecas_abaixo > MAX_PIECES,
        "Detail {} também cabia: subiu demais",
        f.params.detail_q4 - 1
    );
}

#[test]
fn abaixo_do_tecto_nada_muda() {
    let m = pente(16);
    let pedido = asked(2 * Q4 as u16);
    let f = trace_fitted(&m, pedido);
    assert!(!f.raised(pedido));
    assert_eq!(f.outline, trace(&m, pedido));
}

#[test]
fn ilhas_demais_sobem_o_ignore_specks_depois_do_detail() {
    // 300 pontos de 1 px, isolados: cada um é um anel de 4 pontos que nenhum Detail junta.
    let (w, h) = (60u32, 40u32);
    let mut bits = vec![0u8; (w * h) as usize];
    for k in 0..300u32 {
        let (x, y) = ((k % 30) * 2, (k / 30) * 4);
        bits[(y * w + x) as usize] = 1;
    }
    // E uma peça grande que não pode sumir.
    for y in 20..h {
        for x in 0..w {
            bits[(y * w + x) as usize] = 1;
        }
    }
    let m = Mask { w, h, bits };
    let pedido = asked(2 * Q4 as u16);
    let f = trace_fitted(&m, pedido);
    eprintln!(
        "pontinhos: Detail {} q4, Ignore specks under {} px, {} pontos",
        f.params.detail_q4,
        f.params.min_island_px,
        f.outline.vertex_count()
    );
    assert!(f.params.min_island_px > 0, "os pontinhos não saíram");
    assert!(f.outline.vertex_count() <= MAX_POINTS && f.pieces.len() <= MAX_PIECES);
    let base = [w as i32 * Q4 / 2, 30 * Q4];
    assert!(f.outline.contains_q4(base), "a peça grande sumiu");
}

/// Ruído determinístico `n × n` (um píxel em três transparente): ilhas demais em qualquer Detail.
fn ruido(n: u32) -> Mask {
    let mut s = 0x9e37_79b9_7f4a_7c15_u64;
    let bits = (0..n * n)
        .map(|_| {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            u8::from(s % 3 != 0)
        })
        .collect();
    Mask { w: n, h: n, bits }
}

/// ⭐ (GEF02 5.3) **O traço com tecto é o traço, ou nada**: abaixo do tecto os MESMOS bytes do [`trace`];
/// acima, `None`. Em pentes abaixo e acima do tecto, ruído, e todo Detail de 0 a 8 px.
#[test]
fn o_traco_com_tecto_e_o_traco_ou_nada() {
    use ph2d_outline_trace::trace_within;
    let (mut abaixo, mut acima) = (0, 0);
    for m in [pente(50), pente(130), pente(200), ruido(48)] {
        for d in (0..=128).step_by(8) {
            let cru = trace(&m, asked(d));
            let cabe = cru.vertex_count() <= MAX_POINTS;
            if cabe {
                abaixo += 1;
            } else {
                acima += 1;
            }
            assert_eq!(
                trace_within(&m, asked(d), MAX_POINTS),
                cabe.then_some(cru),
                "{}×{} a {d} q4",
                m.w,
                m.h
            );
        }
    }
    assert!(
        abaixo > 10 && acima > 10,
        "os dois lados: {abaixo} abaixo, {acima} acima"
    );
}

/// ⭐ (GEF02 5.3) **Acima do tecto a tentativa desiste ANTES da parte cara**: os pontos que a simplificação
/// guarda só crescem, logo passar de [`MAX_POINTS`] já depois do 1.º corte é o veredito — nenhuma ronda de
/// conflitos corre. Medido (README §Ceilings): o `trace_fitted` de um ruído 256² levava 1,18 s.
/// CONTROLO: o [`trace`] sem tecto corre as rondas.
#[test]
fn acima_do_tecto_a_tentativa_desiste_antes_da_varredura() {
    use ph2d_outline_trace::{sweeps_on_this_thread, trace_within};
    let m = ruido(96);
    let p = asked(2 * Q4 as u16);
    let antes = sweeps_on_this_thread();
    assert!(
        trace(&m, p).vertex_count() > MAX_POINTS,
        "a fixtura acima do tecto"
    );
    assert!(
        sweeps_on_this_thread() > antes,
        "CONTROLO: sem tecto as rondas correm"
    );
    let antes = sweeps_on_this_thread();
    assert_eq!(trace_within(&m, p, MAX_POINTS), None);
    assert_eq!(
        sweeps_on_this_thread(),
        antes,
        "acima do tecto nenhuma ronda corre"
    );
}

/// (GEF02 5.3) Uma tentativa acima do tecto: com tecto (o produto) × o traço inteiro (o de antes),
/// intercalados no mesmo processo, mínimo de 7; e o `trace_fitted` inteiro. `README.md` §Ceilings.
#[test]
#[ignore = "timing: --profile smoke, run by name"]
fn tempo_da_tentativa_acima_do_tecto() {
    use ph2d_outline_trace::trace_within;
    let p = asked(2 * Q4 as u16);
    for (nome, m) in [
        ("pente 200", pente(200)),
        ("pente 512", pente(512)),
        ("ruído 256²", ruido(256)),
        ("ruído 512²", ruido(512)),
    ] {
        let mut best = [u128::MAX; 3];
        for volta in 0..7 {
            for v in 0..2 {
                let com_tecto = (v == 0) == (volta % 2 == 0);
                let t = std::time::Instant::now();
                if com_tecto {
                    std::hint::black_box(trace_within(&m, p, MAX_POINTS));
                } else {
                    std::hint::black_box(trace(&m, p));
                }
                let s = &mut best[usize::from(!com_tecto)];
                *s = (*s).min(t.elapsed().as_micros());
            }
            let t = std::time::Instant::now();
            std::hint::black_box(trace_fitted(&m, p));
            best[2] = best[2].min(t.elapsed().as_micros());
        }
        println!(
            "{nome}: tentativa com tecto {} us · inteira {} us · trace_fitted {} us",
            best[0], best[1], best[2]
        );
    }
}
