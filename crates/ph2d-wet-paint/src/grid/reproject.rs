//! ⭐⭐ **A ÁGUA MUDA DE VISTA** — a grade reconstruída a partir de outra
//! vista da MESMA superfície (report do dono, 29/09: *«rotacionar e pintar em
//! seguida está pausando a simulação»*).
//!
//! A grade vive nos píxeis de uma tela, e a tela de quem pinta uma peça 3D é o
//! ECRÃ. Rodar a peça e dar o traço seguinte muda o ecrã debaixo da água; sem
//! esta porta a única resposta era largar a sessão — a água parava.
//!
//! # A lei
//!
//! Quem chama diz, célula a célula da grade NOVA, **que célula da grade de
//! ANTES está no mesmo ponto da superfície** (`origem`) — ou `None` onde a
//! vista de antes não via aquele ponto. A célula nova COPIA a antiga, sem
//! interpolar: a cor, a água e o pigmento que ela carrega são os de uma célula
//! que o solver produziu. ⚠️ *Interpolar inventaria água que o solver não
//! produziu* (a recusa que o `grid_ratio` do host já escreve para a mudança de
//! razão); copiar só REPETE ou SALTA células que existiam.
//!
//! ⚠️ **O invariante é a APARÊNCIA, não a massa.** Uma célula de ecrã cobre
//! mais ou menos superfície conforme a vista, logo a massa de água por célula
//! não se conserva numa rotação — e é isso que está certo: a cor que o artista
//! vê num ponto da peça não muda por ele a ter rodado.
//!
//! ⚠️ **A velocidade RECOMEÇA do zero.** Ela é um vector no plano do ecrã, e o
//! ecrã rodou; copiá-la faria a água continuar a correr na direcção de antes,
//! que agora aponta para outro lado da peça. A gravidade volta a acelerá-la no
//! passo seguinte.

use super::{Grid, snapshot_grid};

/// **Reconstrói `g` a partir dele mesmo visto de outro sítio.** `origem(cx,
/// cy)` recebe uma célula INTERIOR da grade nova (1-based, a convenção do
/// motor) e devolve a célula interior da grade de antes no mesmo ponto da
/// superfície, ou `None` onde ela não existia.
///
/// Uma resposta fora do interior conta como `None` — a porta não confia em
/// quem chama para os limites.
pub fn reproject_grid(g: &mut Grid, origem: impl Fn(i32, i32) -> Option<(i32, i32)>) {
    let antes = snapshot_grid(g);
    let (w, h, s) = (g.w as i32, g.h as i32, g.s);
    g.film.fill(0.0);
    g.susp.fill(0.0);
    g.susp_rgb.fill([0.0; 3]);
    g.sett.fill(0.0);
    g.sett_rgb.fill([0.0; 3]);
    g.wet.fill(0);
    g.bloom.fill(0);
    let (mut x0, mut y0, mut x1, mut y1) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
    for cy in 1..=h {
        for cx in 1..=w {
            let Some((sx, sy)) = origem(cx, cy) else {
                continue;
            };
            if sx < 1 || sy < 1 || sx > w || sy > h {
                continue;
            }
            let i = cx as usize + cy as usize * s;
            let j = sx as usize + sy as usize * s;
            g.film[i] = antes.film[j];
            g.susp[i] = antes.susp[j];
            g.susp_rgb[i] = antes.susp_rgb[j];
            g.sett[i] = antes.sett[j];
            g.sett_rgb[i] = antes.sett_rgb[j];
            g.wet[i] = antes.wet[j];
            g.bloom[i] = antes.bloom[j];
            if g.film[i] > 0.0 || g.susp[i] > 0.0 {
                x0 = x0.min(cx);
                y0 = y0.min(cy);
                x1 = x1.max(cx);
                y1 = y1.max(cy);
            }
        }
    }
    g.vel_x.fill(0.0);
    g.vel_y.fill(0.0);
    g.flow_x.fill(0.0);
    g.flow_y.fill(0.0);
    // O estado DERIVADO recomeça do conteúdo: a máscara vazia, a faixa viva
    // aberta (um superconjunto é sempre válido — `active ⊆ faixa`), e o
    // rebuild do solver reaperta as duas a partir da água que ficou.
    g.active.fill(0);
    g.row_lo.fill(0);
    g.row_hi.fill(w + 1);
    g.empty_bbox();
    if x0 <= x1 {
        g.expand_bbox(x0, y0, x1, y1);
        crate::solver::rebuild_active_region(g);
    }
}

#[cfg(test)]
#[path = "reproject_tests.rs"]
mod tests;
