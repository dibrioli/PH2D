//! O espelho por linha: hashes das linhas finas e de fluxo, e o mapa fina ⇄ fluxo.

use ph2d_wet_paint::grid::Grid;

/// Hash de uma linha: multiplica-e-roda sobre palavras de 8 B (não é cripto; a colisão de 64 bits é desprezável).
fn mistura(mut h: u64, d: &[u8]) -> u64 {
    let (palavras, resto) = d.as_chunks::<8>();
    for c in palavras {
        let w = u64::from_le_bytes(*c);
        h = (h ^ w).wrapping_mul(0x9E37_79B9_7F4A_7C15).rotate_left(31);
    }
    for &b in resto {
        h = (h ^ u64::from(b))
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            .rotate_left(31);
    }
    h
}

/// Os hashes das linhas `0..n` por `f(linha)`, em paralelo (fatias de linhas por núcleo).
pub(super) fn hashes(n: usize, f: &(dyn Fn(usize) -> u64 + Sync)) -> Vec<u64> {
    let mut v = vec![0u64; n];
    let nt = std::thread::available_parallelism()
        .map_or(4, |k| k.get())
        .min(16);
    let por = n.div_ceil(nt).max(1);
    std::thread::scope(|sc| {
        for (k, ch) in v.chunks_mut(por).enumerate() {
            sc.spawn(move || {
                for (j, h) in ch.iter_mut().enumerate() {
                    *h = f(k * por + j);
                }
            });
        }
    });
    v
}

/// O hash de uma linha FINA de tudo o que a placa espelha.
pub(super) fn hash_fina(g: &Grid, y: usize) -> u64 {
    let (a, b) = (y * g.s, (y + 1) * g.s);
    let c = bytemuck::cast_slice::<f32, u8>;
    let mut h = mistura(y as u64, c(&g.film[a..b]));
    h = mistura(h, c(&g.susp[a..b]));
    h = mistura(h, bytemuck::cast_slice(&g.susp_rgb[a..b]));
    h = mistura(h, c(&g.sett[a..b]));
    h = mistura(h, bytemuck::cast_slice(&g.sett_rgb[a..b]));
    h = mistura(h, c(&g.paper[a..b]));
    h = mistura(h, &g.active[a..b]);
    h = mistura(h, &g.wet[a..b]);
    if !g.vela.is_empty() {
        h = mistura(h, c(&g.vela[a..b]));
    }
    if !g.aparencia.is_empty() {
        let v: Vec<f32> = (a..b)
            .map(|i| ph2d_wet_paint::grid::aparencia::vidrado(&g.aparencia, i))
            .collect();
        h = mistura(h, c(&v));
    }
    let tinta = crate::tinta::plano_do_hash(g, a, b);
    if !tinta.is_empty() {
        h = mistura(h, c(&tinta));
    }
    h
}

/// O hash de uma linha da grade de FLUXO (velocidade e fluxo).
pub(super) fn hash_fluxo(g: &Grid, y: usize) -> u64 {
    let (a, b) = (y * g.flow.s, (y + 1) * g.flow.s);
    let c = bytemuck::cast_slice::<f32, u8>;
    let mut h = mistura(y as u64, c(&g.vel_x[a..b]));
    h = mistura(h, c(&g.vel_y[a..b]));
    h = mistura(h, c(&g.flow_x[a..b]));
    mistura(h, c(&g.flow_y[a..b]))
}

/// As linhas de fluxo que cobrem as finas `y0..=y1` (com uma de folga de cada lado).
pub(super) fn fluxo_de(g: &Grid, y0: usize, y1: usize) -> (usize, usize) {
    let rf = g.flow.rf;
    (
        y0.saturating_sub(1) / rf,
        (y1 / rf + 1).min(g.flow.rows - 1),
    )
}

/// As linhas finas que uma linha de fluxo cobre (com folga).
pub(super) fn finas_de(g: &Grid, f: usize) -> (usize, usize) {
    let rf = g.flow.rf;
    (f.saturating_sub(1) * rf, ((f + 1) * rf).min(g.rows - 1))
}
