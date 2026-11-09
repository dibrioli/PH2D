//! ⭐ **Trim, Padding e Make Square numa folha CORTADA em quadros** (dono 09/11, opção A): a mesma margem em cada
//! quadro, e o desenho de cada um fica onde estava. Na imagem inteira, `run.png` 288 × 96 em 6 × 2 virava 269 × 90
//! (células de 44,8 px) e cada quadro andava até 16 px (medido 09/11; doc 27 §3).
//!
//! A origem, as outras folhas, os filhos e a colisão não se mexem: só o Pivot da folha editada anda, o bastante para
//! cada píxel cair no mesmo sítio local.

use ph2d_ecs::{Entity, SpriteGrid, World};
use ph2d_tool_padding::PaddingSpec;

/// O corte da folha visível: mais de uma célula, todas com píxeis inteiros.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cut {
    pub cols: u32,
    pub rows: u32,
    /// Uma célula, em píxeis.
    pub cell: [u32; 2],
    /// Metros por píxel da folha: o tamanho de uma célula na tela ÷ [`Self::cell`].
    pub m_per_px: [f32; 2],
}

/// O que o verbo faz a CADA célula: o tamanho novo e onde o canto da célula velha cai nela (negativo = corte).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CellChange {
    pub cell: [u32; 2],
    pub at: [i32; 2],
}

/// O corte da folha `sheet` de `e` sobre a imagem `w × h`. `None` = uma célula só (a imagem é um Still e o verbo age
/// nela inteira) ou um corte que não cai em píxeis inteiros (o desenho já está partido entre células).
#[must_use]
pub fn cut_of(world: &World, e: Entity, sheet: u8, w: u32, h: u32) -> Option<Cut> {
    let (s, grid) = ph2d_render::sheet_geometry(world, e, sheet, 0)?;
    let g = grid.unwrap_or(SpriteGrid::SINGLE);
    let (cols, rows) = (g.hframes.max(1), g.vframes.max(1));
    if u64::from(cols) * u64::from(rows) <= 1 || !w.is_multiple_of(cols) || !h.is_multiple_of(rows)
    {
        return None;
    }
    let cell = [w / cols, h / rows];
    Some(Cut {
        cols,
        rows,
        cell,
        m_per_px: [s.size[0] / cell[0] as f32, s.size[1] / cell[1] as f32],
    })
}

/// A janela `[x, y, w, h]` da célula `(c, r)`.
const fn janela(cut: &Cut, c: u32, r: u32) -> [u32; 4] {
    [c * cut.cell[0], r * cut.cell[1], cut.cell[0], cut.cell[1]]
}

/// **Trim:** a união das caixas opacas de todas as células, o mesmo corte em cada quadro. `None` = nada a cortar (ou
/// a folha toda transparente).
#[must_use]
pub fn trim(rgba: &[u8], w: u32, cut: &Cut) -> Option<CellChange> {
    let mut u: Option<[u32; 4]> = None;
    for r in 0..cut.rows {
        for c in 0..cut.cols {
            if let Some(b) = ph2d_sprite_edit::pivot::caixa_opaca(rgba, w, janela(cut, c, r)) {
                u = Some(u.map_or(b, |a| {
                    [
                        a[0].min(b[0]),
                        a[1].min(b[1]),
                        a[2].max(b[2]),
                        a[3].max(b[3]),
                    ]
                }));
            }
        }
    }
    let [x0, y0, x1, y1] = u?;
    if [x0, y0, x1, y1] == [0, 0, cut.cell[0], cut.cell[1]] {
        return None;
    }
    Some(CellChange {
        cell: [x1 - x0, y1 - y0],
        at: [-(x0 as i32), -(y0 as i32)],
    })
}

/// **Make Square:** a moldura transparente que faz cada célula quadrada, com o arredondamento do Make Square (o resto
/// ímpar fica à direita e em baixo). `None` = já quadrada.
#[must_use]
pub fn square(cut: &Cut) -> Option<CellChange> {
    let [cw, ch] = cut.cell;
    if cw == ch {
        return None;
    }
    let side = cw.max(ch);
    Some(CellChange {
        cell: [side, side],
        at: [((side - cw) / 2) as i32, ((side - ch) / 2) as i32],
    })
}

/// **Padding:** o `spec` em cada célula. A geometria é a do `add_padding` sobre a 1.ª célula (o corte que deixa pelo
/// menos 1 px é dele; uma segunda cópia da conta teria de concordar para sempre). `None` = nada muda.
#[must_use]
pub fn padding(rgba: &[u8], w: u32, cut: &Cut, spec: PaddingSpec) -> Option<CellChange> {
    if spec.is_noop() {
        return None;
    }
    let primeira = remap(rgba, w, janela(cut, 0, 0), cut.cell, [0, 0]);
    let r = ph2d_tool_padding::add_padding(
        bytemuck::cast_slice(&primeira),
        cut.cell[0],
        cut.cell[1],
        spec,
    );
    r.changed.then_some(CellChange {
        cell: [r.width, r.height],
        at: [r.pivot_delta_x, r.pivot_delta_y],
    })
}

/// A janela `[x, y, w, h]` de `px` (4 canais, `w` por linha) posta numa imagem `novo` com o canto em `at`; o que sai
/// fica de fora, o que sobra é transparente.
fn remap<T: Copy + Default>(
    px: &[T],
    w: u32,
    [jx, jy, jw, jh]: [u32; 4],
    novo: [u32; 2],
    at: [i32; 2],
) -> Vec<T> {
    let mut out = vec![T::default(); novo[0] as usize * novo[1] as usize * 4];
    pousa(px, w, [jx, jy, jw, jh], &mut out, novo[0], [0, 0], novo, at);
    out
}

/// Copia a janela de `px` para `out` (largura `ow`) na caixa `[ox, oy]` de tamanho `caixa`, com o canto da janela em
/// `at` dentro da caixa. Recorta às duas.
#[allow(clippy::too_many_arguments)]
fn pousa<T: Copy>(
    px: &[T],
    w: u32,
    [jx, jy, jw, jh]: [u32; 4],
    out: &mut [T],
    ow: u32,
    [ox, oy]: [u32; 2],
    caixa: [u32; 2],
    at: [i32; 2],
) {
    let x0 = at[0].max(0);
    let x1 = (at[0] + jw as i32).min(caixa[0] as i32);
    if x1 <= x0 {
        return;
    }
    let n = (x1 - x0) as usize * 4;
    for y in at[1].max(0)..(at[1] + jh as i32).min(caixa[1] as i32) {
        let sy = (jy as i32 + y - at[1]) as usize;
        let sx = (jx as i32 + x0 - at[0]) as usize;
        let de = (sy * w as usize + sx) * 4;
        let para = ((oy as usize + y as usize) * ow as usize + ox as usize + x0 as usize) * 4;
        out[para..para + n].copy_from_slice(&px[de..de + n]);
    }
}

/// ⭐ **A folha refeita:** cada célula muda como `ch` diz, na mesma grelha. `T` = `u8` (RGBA8) ou `u16` (RGBA16): o
/// verbo não calcula valor de píxel nenhum, logo os 16 bits passam intactos. Devolve `(píxeis, largura, altura)`.
#[must_use]
pub fn recompose<T: Copy + Default>(
    px: &[T],
    w: u32,
    cut: &Cut,
    ch: CellChange,
) -> (Vec<T>, u32, u32) {
    let (nw, nh) = (ch.cell[0] * cut.cols, ch.cell[1] * cut.rows);
    let mut out = vec![T::default(); nw as usize * nh as usize * 4];
    for r in 0..cut.rows {
        for c in 0..cut.cols {
            let canto = [c * ch.cell[0], r * ch.cell[1]];
            pousa(
                px,
                w,
                janela(cut, c, r),
                &mut out,
                nw,
                canto,
                ch.cell,
                ch.at,
            );
        }
    }
    (out, nw, nh)
}

/// Quanto o Pivot da folha anda (metros locais, com o flip do `Sprite`, que vira os píxeis dentro do quad) para cada
/// píxel de cada quadro ficar no mesmo sítio.
#[must_use]
pub fn pivot_shift(cut: &Cut, ch: CellChange, [fx, fy]: [bool; 2]) -> [f32; 2] {
    let meio = |a: u32| a as f32 * 0.5;
    let dx = (meio(ch.cell[0]) - meio(cut.cell[0]) - ch.at[0] as f32) * cut.m_per_px[0];
    let dy = (meio(cut.cell[1]) - meio(ch.cell[1]) + ch.at[1] as f32) * cut.m_per_px[1];
    [if fx { -dx } else { dx }, if fy { -dy } else { dy }]
}

/// O tamanho que o funil de gravação espera: UMA célula na folha `0` (o `Sprite.size`), a imagem inteira numa folha
/// extra (o [`crate::shown_edit::set_image`] divide-a pela grelha).
#[must_use]
pub fn commit_size(cut: &Cut, ch: CellChange, sheet: u8) -> [f32; 2] {
    let celula = [
        ch.cell[0] as f32 * cut.m_per_px[0],
        ch.cell[1] as f32 * cut.m_per_px[1],
    ];
    if sheet == 0 {
        celula
    } else {
        [celula[0] * cut.cols as f32, celula[1] * cut.rows as f32]
    }
}

/// ⭐ Depois de gravar: o Pivot da folha `sheet` anda `d` (de [`pivot_shift`]) e nada mais se mexe. Numa folha extra o
/// Pivot é arte e sobe com ela ([`ph2d_ecs::linked_art_chain`]), como no [`crate::shown_edit::place`].
pub fn keep_drawing(world: &mut World, e: Entity, sheet: u8, d: [f32; 2]) {
    if sheet == 0 {
        if let Some(mut s) = world.get_mut::<ph2d_render::Sprite>(e) {
            s.anchor = [s.anchor[0] + d[0], s.anchor[1] + d[1]];
        }
        return;
    }
    for x in ph2d_ecs::linked_art_chain(world, e) {
        crate::shown_edit::edit_sheet(world, x, sheet, |s| {
            s.anchor = [s.anchor[0] + d[0], s.anchor[1] + d[1]];
        });
    }
}

#[cfg(test)]
#[path = "cell_edit_tests.rs"]
mod tests;
