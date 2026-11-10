use super::*;

/// An `w×h` RGBA8 buffer: left half `a`, right half `b`.
fn split(w: usize, h: usize, a: [u8; 4], b: [u8; 4]) -> Vec<u8> {
    let mut px = vec![0u8; w * h * 4];
    for y in 0..h {
        for x in 0..w {
            let o = (y * w + x) * 4;
            px[o..o + 4].copy_from_slice(if x < w / 2 { &a } else { &b });
        }
    }
    px
}

fn rgb(px: &[u8], w: usize, x: usize, y: usize) -> [u8; 3] {
    let o = (y * w + x) * 4;
    [px[o], px[o + 1], px[o + 2]]
}

#[test]
fn fills_the_connected_region_and_stops_at_a_colour_boundary() {
    let (w, h) = (8, 8);
    let mut px = split(w, h, [200, 0, 0, 255], [0, 0, 200, 255]);
    let rect = flood_fill(&mut px, w, h, (1, 1), [0, 255, 0], 10, None).expect("filled");
    for y in 0..h {
        for x in 0..w {
            if x < 4 {
                assert_eq!(
                    rgb(&px, w, x, y),
                    [0, 255, 0],
                    "red half → green at ({x},{y})"
                );
            } else {
                assert_eq!(
                    rgb(&px, w, x, y),
                    [0, 0, 200],
                    "blue half untouched at ({x},{y})"
                );
            }
        }
    }
    assert_eq!((rect.x, rect.y, rect.w, rect.h), (0, 0, 4, 8));
}

#[test]
fn tolerance_bridges_small_differences() {
    // Two near-identical reds (Δ = 8) meeting a far blue. tol 0 fills only the exact seed colour;
    // tol 20 bridges the near-red into one region; neither crosses into blue.
    let (w, h) = (6, 1);
    let mut px = vec![0u8; w * h * 4];
    let cols = [
        [200, 0, 0, 255],
        [200, 0, 0, 255],
        [208, 0, 0, 255],
        [208, 0, 0, 255],
        [0, 0, 200, 255],
        [0, 0, 200, 255],
    ];
    for (x, c) in cols.iter().enumerate() {
        px[x * 4..x * 4 + 4].copy_from_slice(c);
    }
    let mut tight = px.clone();
    flood_fill(&mut tight, w, h, (0, 0), [0, 255, 0], 0, None);
    assert_eq!(rgb(&tight, w, 0, 0), [0, 255, 0]);
    assert_eq!(
        rgb(&tight, w, 2, 0),
        [208, 0, 0],
        "Δ=8 not bridged at tol 0"
    );
    let mut loose = px.clone();
    flood_fill(&mut loose, w, h, (0, 0), [0, 255, 0], 20, None);
    assert_eq!(rgb(&loose, w, 2, 0), [0, 255, 0], "Δ=8 bridged at tol 20");
    assert_eq!(rgb(&loose, w, 4, 0), [0, 0, 200], "far blue never crossed");
}

#[test]
fn out_of_bounds_seed_returns_none() {
    let mut px = vec![255u8; 4 * 4 * 4];
    assert!(flood_fill(&mut px, 4, 4, (9, 9), [1, 2, 3], 0, None).is_none());
}

#[test]
fn no_change_returns_none() {
    // Seed region already the fill colour ⇒ nothing changes ⇒ None.
    let (w, h) = (4, 4);
    let mut px = vec![0u8; w * h * 4];
    for c in px.as_chunks_mut::<4>().0.iter_mut() {
        c.copy_from_slice(&[0, 255, 0, 255]);
    }
    assert!(flood_fill(&mut px, w, h, (1, 1), [0, 255, 0], 0, None).is_none());
}

/// **B48.1 — a pilha guarda uma entrada por CORRIDA da linha vizinha, não por texel.** Numa tela
/// uniforme o flood desce linha a linha; empilhar cada texel deixava `w-1` entradas velhas por linha
/// (pico ≈ `w·h`: ~268 MB a 4096²). O tecto `4·h` é folgado para a tela uniforme, onde a cura dá ~1.
#[test]
fn o_balde_empilha_uma_entrada_por_corrida_nao_por_texel() {
    for (w, h) in [(512usize, 512usize), (2048, 2048)] {
        for seed in [(0, 0), (w / 2, h / 2), (w - 1, h - 1)] {
            let mut px = vec![255u8; w * h * 4];
            let (r, pico, _) = flood_fill_medido(&mut px, w, h, seed, [0, 255, 0], 0, None);
            let r = r.expect("a tela inteira muda");
            assert_eq!((r.w, r.h), (w as u32, h as u32));
            eprintln!("B48.1 {w}x{h} semente {seed:?}: pico da pilha = {pico} entradas");
            assert!(
                pico <= 4 * h,
                "{w}x{h} semente {seed:?}: pico {pico} > 4·h = {} (uma entrada por texel)",
                4 * h
            );
        }
    }
}

/// O `flood_fill` de ANTES do B48.1, verbatim — o oráculo da paridade ao bit.
#[allow(clippy::too_many_arguments)]
fn flood_fill_antigo(
    px: &mut [u8],
    w: usize,
    h: usize,
    seed: (usize, usize),
    fill: [u8; 3],
    tol: u8,
    mut mask: Option<&mut [u8]>,
) -> Option<Region> {
    if w == 0 || h == 0 || px.len() != w * h * 4 {
        return None;
    }
    let (sx, sy) = seed;
    if sx >= w || sy >= h {
        return None;
    }
    let si = (sy * w + sx) * 4;
    let seed_c = [px[si], px[si + 1], px[si + 2], px[si + 3]];
    let fillpx = [fill[0], fill[1], fill[2], 255];
    let mut visited = vec![false; w * h];
    let mut stack: Vec<(usize, usize)> = vec![seed];
    let (mut minx, mut miny, mut maxx, mut maxy) = (sx, sy, sx, sy);
    let mut changed = false;
    while let Some((x, y)) = stack.pop() {
        if visited[y * w + x] {
            continue;
        }
        let mut lx = x;
        while lx > 0
            && !visited[y * w + lx - 1]
            && color_match(px, (y * w + lx - 1) * 4, seed_c, tol)
        {
            lx -= 1;
        }
        let mut rx = x;
        while rx + 1 < w
            && !visited[y * w + rx + 1]
            && color_match(px, (y * w + rx + 1) * 4, seed_c, tol)
        {
            rx += 1;
        }
        for xx in lx..=rx {
            let idx = y * w + xx;
            visited[idx] = true;
            if let Some(m) = mask.as_deref_mut()
                && let Some(cell) = m.get_mut(idx)
            {
                *cell = 255;
            }
            let o = idx * 4;
            if px[o..o + 4] != fillpx {
                changed = true;
            }
            px[o..o + 4].copy_from_slice(&fillpx);
        }
        minx = minx.min(lx);
        maxx = maxx.max(rx);
        miny = miny.min(y);
        maxy = maxy.max(y);
        for ny in [y.checked_sub(1), (y + 1 < h).then_some(y + 1)]
            .into_iter()
            .flatten()
        {
            for xx in lx..=rx {
                if !visited[ny * w + xx] && color_match(px, (ny * w + xx) * 4, seed_c, tol) {
                    stack.push((xx, ny));
                }
            }
        }
    }
    changed.then(|| Region {
        x: minx as u32,
        y: miny as u32,
        w: (maxx - minx + 1) as u32,
        h: (maxy - miny + 1) as u32,
    })
}

/// As seis telas do oráculo (`w×h` RGBA8, não quadradas de propósito).
fn telas(w: usize, h: usize) -> Vec<(&'static str, Vec<u8>)> {
    let pinta = |f: &dyn Fn(usize, usize) -> [u8; 4]| {
        let mut px = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                px[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(&f(x, y));
            }
        }
        px
    };
    let (a, b) = ([255, 255, 255, 255], [30, 60, 200, 255]);
    let mut s = 0x2545_f491_u32;
    let mut ruido = vec![0u8; w * h * 4];
    for c in &mut ruido {
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        *c = (s >> 24) as u8;
    }
    let (cx, cy) = (w as i64 / 2, h as i64 / 2);
    vec![
        ("uniforme", pinta(&|_, _| a)),
        (
            "xadrez",
            pinta(&|x, y| if (x + y) % 2 == 0 { a } else { b }),
        ),
        (
            "xadrez3",
            pinta(&|x, y| if (x / 3 + y / 3) % 2 == 0 { a } else { b }),
        ),
        ("metade", pinta(&|x, _| if x < w / 2 { a } else { b })),
        (
            "anel",
            pinta(&|x, y| {
                let d2 = (x as i64 - cx).pow(2) + (y as i64 - cy).pow(2);
                if (100..400).contains(&d2) { b } else { a }
            }),
        ),
        ("ruido", ruido),
    ]
}

/// **B48.1 ao bit:** a pilha por corrida devolve a MESMA região, os mesmos bytes e a mesma máscara que o
/// flood de antes, em seis telas × três tolerâncias × várias sementes.
#[test]
fn o_flood_por_corrida_e_o_antigo_ao_bit() {
    let (w, h) = (61usize, 47usize);
    for (nome, tela) in telas(w, h) {
        for tol in [0u8, 40, 255] {
            for seed in [
                (0, 0),
                (w / 2, h / 2),
                (w - 1, h - 1),
                (7, 30),
                (w / 2 + 12, h / 2),
            ] {
                let (mut novo, mut velho) = (tela.clone(), tela.clone());
                let (mut mn, mut mv) = (vec![0u8; w * h], vec![0u8; w * h]);
                let rn = flood_fill(&mut novo, w, h, seed, [9, 200, 90], tol, Some(&mut mn));
                let rv =
                    flood_fill_antigo(&mut velho, w, h, seed, [9, 200, 90], tol, Some(&mut mv));
                let caso = format!("{nome} tol {tol} semente {seed:?}");
                assert_eq!(
                    rn.map(|r| (r.x, r.y, r.w, r.h)),
                    rv.map(|r| (r.x, r.y, r.w, r.h)),
                    "{caso}: região"
                );
                assert!(novo == velho, "{caso}: bytes da tela");
                assert!(mn == mv, "{caso}: máscara do conjunto");
            }
        }
    }
}
