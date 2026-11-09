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
