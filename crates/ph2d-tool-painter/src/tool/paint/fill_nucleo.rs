//! O núcleo PURO do balde (★B52.R10): o flood por corridas e a componente da seleção sob a semente.
//! Sem `PainterTool` — entra a tela, sai a região; o gesto, o recorte e o commit vivem em `fill.rs`.

use crate::tool::paint::Region;

/// Max per-channel (RGBA) difference from the seed colour for a pixel to be filled, at a given tolerance.
#[inline]
fn color_match(px: &[u8], i: usize, seed: [u8; 4], tol: u8) -> bool {
    let d = px[i]
        .abs_diff(seed[0])
        .max(px[i + 1].abs_diff(seed[1]))
        .max(px[i + 2].abs_diff(seed[2]))
        .max(px[i + 3].abs_diff(seed[3]));
    d <= tol
}

/// Scanline flood-fill `px` (straight RGBA8, `w*h`) from `seed`, replacing the connected region within
/// `tol` of the seed colour with opaque `fill`. Returns the affected bounding box, or `None` if the seed
/// is out of bounds or nothing changed. A `visited` bitmap makes it correct even when `fill` equals the
/// seed colour (no infinite loop).
/// ⚠️ **`mask` é o CONJUNTO preenchido, não um segundo desenho.** O balde escreve pixels duros e o
/// clip da seleção os compõe depois; o corpo do Impasto precisa saber *onde* o flood chegou, e a
/// varredura já mantém exatamente esse conjunto no `visited`.
#[cfg(test)]
pub(in crate::tool::paint) fn flood_fill(
    px: &mut [u8],
    w: usize,
    h: usize,
    seed: (usize, usize),
    fill: [u8; 3],
    tol: u8,
    mask: Option<&mut [u8]>,
) -> Option<Region> {
    flood_fill_medido(px, w, h, seed, fill, tol, mask).0
}

/// [`flood_fill`] sobre um `visited` do chamador (todo `false` à entrada; à saída `true` no conjunto
/// preenchido, dentro da CAIXA devolvida) — o tique do balde (B48.2) não aloca `w·h`.
pub(in crate::tool::paint) fn flood_fill_caixa(
    px: &mut [u8],
    w: usize,
    h: usize,
    seed: (usize, usize),
    fill: [u8; 3],
    tol: u8,
    visited: &mut [bool],
) -> (Option<Region>, Option<Region>) {
    let (r, _, caixa) = flood_fill_em(px, w, h, seed, fill, tol, None, visited);
    (r, caixa)
}

/// [`flood_fill_caixa`] SÓ DE LEITURA (B17.6): o mesmo conjunto no `visited` e as mesmas duas caixas, sem
/// escrever `px` — o flood só lê texels ainda não visitados, que ele nunca escreveu. Quem chama pinta o
/// conjunto depois com [`pinta_o_conjunto`] (o fork da tela já sabe a caixa).
pub(in crate::tool::paint) fn flood_visita_caixa(
    px: &[u8],
    w: usize,
    h: usize,
    seed: (usize, usize),
    fill: [u8; 3],
    tol: u8,
    visited: &mut [bool],
) -> (Option<Region>, Option<Region>) {
    let (r, _, caixa) = flood_visita_em(px, w, h, seed, fill, tol, None, visited);
    (r, caixa)
}

/// Escreve `fill` opaco em todo texel de `caixa` marcado no `visited` — a metade que escreve do flood.
pub(in crate::tool::paint) fn pinta_o_conjunto(
    px: &mut [u8],
    w: usize,
    caixa: Region,
    visited: &[bool],
    fill: [u8; 3],
) {
    let fillpx = [fill[0], fill[1], fill[2], 255];
    for y in caixa.y as usize..(caixa.y + caixa.h) as usize {
        for x in caixa.x as usize..(caixa.x + caixa.w) as usize {
            let idx = y * w + x;
            if visited[idx] {
                px[idx * 4..idx * 4 + 4].copy_from_slice(&fillpx);
            }
        }
    }
}

/// [`flood_fill`] mais o PICO da pilha (entradas) — a régua do B48.1.
#[cfg(test)]
fn flood_fill_medido(
    px: &mut [u8],
    w: usize,
    h: usize,
    seed: (usize, usize),
    fill: [u8; 3],
    tol: u8,
    mask: Option<&mut [u8]>,
) -> (Option<Region>, usize, Option<Region>) {
    if w == 0 || h == 0 || px.len() != w * h * 4 {
        return (None, 0, None);
    }
    flood_fill_em(px, w, h, seed, fill, tol, mask, &mut vec![false; w * h])
}

#[expect(clippy::too_many_arguments, reason = "o flood + o seu visited")]
fn flood_fill_em(
    px: &mut [u8],
    w: usize,
    h: usize,
    seed: (usize, usize),
    fill: [u8; 3],
    tol: u8,
    mask: Option<&mut [u8]>,
    visited: &mut [bool],
) -> (Option<Region>, usize, Option<Region>) {
    let r = flood_visita_em(px, w, h, seed, fill, tol, mask, visited);
    if let Some(caixa) = r.2 {
        pinta_o_conjunto(px, w, caixa, visited, fill);
    }
    r
}

#[expect(clippy::too_many_arguments, reason = "o flood + o seu visited")]
fn flood_visita_em(
    px: &[u8],
    w: usize,
    h: usize,
    seed: (usize, usize),
    fill: [u8; 3],
    tol: u8,
    mut mask: Option<&mut [u8]>,
    visited: &mut [bool],
) -> (Option<Region>, usize, Option<Region>) {
    if w == 0 || h == 0 || px.len() != w * h * 4 || visited.len() != w * h {
        return (None, 0, None);
    }
    let (sx, sy) = seed;
    if sx >= w || sy >= h {
        return (None, 0, None);
    }
    let si = (sy * w + sx) * 4;
    let seed_c = [px[si], px[si + 1], px[si + 2], px[si + 3]];
    let fillpx = [fill[0], fill[1], fill[2], 255];
    let mut stack: Vec<(usize, usize)> = vec![seed];
    let mut pico = 1usize;
    let (mut minx, mut miny, mut maxx, mut maxy) = (sx, sy, sx, sy);
    let mut changed = false;
    while let Some((x, y)) = stack.pop() {
        if visited[y * w + x] {
            continue;
        }
        // Grow the horizontal span [lx, rx] around x over matching, unvisited pixels.
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
        // Fill the span.
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
        }
        minx = minx.min(lx);
        maxx = maxx.max(rx);
        miny = miny.min(y);
        maxy = maxy.max(y);
        // Seed the rows above and below: ONE entry per contiguous run (B48.1) — the span that pops it
        // grows over the whole run, so the rest of the run would only be stale entries.
        for ny in [y.checked_sub(1), (y + 1 < h).then_some(y + 1)]
            .into_iter()
            .flatten()
        {
            let linha = ny * w;
            let mut em_corrida = false;
            for xx in lx..=rx {
                let entra = !visited[linha + xx] && color_match(px, (linha + xx) * 4, seed_c, tol);
                if entra && !em_corrida {
                    stack.push((xx, ny));
                }
                em_corrida = entra;
            }
        }
        pico = pico.max(stack.len());
    }
    let caixa = Region {
        x: minx as u32,
        y: miny as u32,
        w: (maxx - minx + 1) as u32,
        h: (maxy - miny + 1) as u32,
    };
    (changed.then_some(caixa), pico, Some(caixa))
}

/// The connected COMPONENT (4-connected, `≥128` inside) of the selection `mask` that contains texel
/// `(sx, sy)` — `255` inside that one region, `0` elsewhere. Lets a ColorDrop fill only the selection
/// area the colour was dropped on, not every disjoint area (Enio 2026-07-04). All-`0` when the drop is
/// outside the selection. O tique usa [`componente_da_selecao_em`]; esta fica para o oráculo.
#[cfg(test)]
pub(in crate::tool::paint) fn componente_da_selecao(
    mask: &[u8],
    sx: usize,
    sy: usize,
    w: usize,
    h: usize,
) -> Vec<u8> {
    let mut comp = vec![0u8; w * h];
    componente_da_selecao_em(mask, sx, sy, w, h, &mut comp);
    comp
}

/// [`componente_da_selecao`] escrita em `comp` (todo `0` à entrada); devolve a caixa da componente.
pub(in crate::tool::paint) fn componente_da_selecao_em(
    mask: &[u8],
    sx: usize,
    sy: usize,
    w: usize,
    h: usize,
    comp: &mut [u8],
) -> Option<Region> {
    if mask.len() != w * h || comp.len() != w * h || sx >= w || sy >= h || mask[sy * w + sx] < 128 {
        return None; // drop outside the selection → fills nothing
    }
    let mut stack = vec![(sx, sy)];
    comp[sy * w + sx] = 255;
    let (mut x0, mut y0, mut x1, mut y1) = (sx, sy, sx, sy);
    while let Some((x, y)) = stack.pop() {
        (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
        for (dx, dy) in [(-1i32, 0), (1, 0), (0, -1), (0, 1)] {
            let (nx, ny) = (x as i32 + dx, y as i32 + dy);
            if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                continue;
            }
            let idx = ny as usize * w + nx as usize;
            if comp[idx] == 0 && mask[idx] >= 128 {
                comp[idx] = 255;
                stack.push((nx as usize, ny as usize));
            }
        }
    }
    Some(Region {
        x: x0 as u32,
        y: y0 as u32,
        w: (x1 - x0 + 1) as u32,
        h: (y1 - y0 + 1) as u32,
    })
}

#[cfg(test)]
#[path = "fill_nucleo_tests.rs"]
mod tests;
