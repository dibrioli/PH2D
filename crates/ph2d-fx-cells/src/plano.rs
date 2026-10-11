//! **O PLANO das células** — que instância vai para que célula de que página do átlas, e a
//! translação INTEIRA de cada uma (camada → átlas). Puro: sem `wgpu`, prova-se inteiro na cabeça.
//!
//! Uma célula é a caixa da instância no ecrã (arredondada para fora ao pixel, mais um de folga de
//! anti-aliasing), com a margem `M` da pilha dos quatro lados e um pixel de vala: entre o conteúdo
//! de duas células há `2M + 1` pixels — a pilha inteira lê no máximo `M` para cada lado de um pixel
//! que ela escreve dentro da margem de uma célula, logo nunca alcança a vizinha (doc 129 §3.4).

use ph2d_render::fx_atlas::pack_in_order;

/// A caixa de uma instância no ECRÃ (pixels da camada) e o pixel da âncora dela — o que os passes
/// de limites escrevem, dois `vec4` por instância. `lo > hi` = vazia (sem geometria).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Caixa {
    pub lo: [f32; 2],
    pub hi: [f32; 2],
    pub ancora: [f32; 2],
    pub _pad: [f32; 2],
}

/// Uma célula: a instância (índice na lista sprites ++ formas), a translação inteira camada → átlas,
/// o rectângulo da CAMADA que ela devolve (`x0, y0, x1, y1`, meio-aberto) e a âncora no átlas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Celula {
    pub instancia: u32,
    pub t: [i32; 2],
    pub saida: [i32; 4],
    pub org: [f32; 2],
}

/// Uma página do átlas: o tamanho que a pilha percorre e as células, na ordem de z.
#[derive(Clone, Debug, PartialEq)]
pub struct Pagina {
    pub tamanho: [u32; 2],
    pub celulas: Vec<Celula>,
}

/// O conteúdo de uma instância em pixels inteiros: a caixa para fora, mais um de folga.
fn conteudo(c: &Caixa) -> Option<[i32; 4]> {
    let ok =
        c.lo.iter().chain(&c.hi).all(|v| v.is_finite()) && c.lo[0] <= c.hi[0] && c.lo[1] <= c.hi[1];
    #[expect(clippy::cast_possible_truncation, reason = "pixels de ecrã")]
    ok.then(|| {
        [
            c.lo[0].floor() as i32 - 1,
            c.lo[1].floor() as i32 - 1,
            c.hi[0].ceil() as i32 + 1,
            c.hi[1].ceil() as i32 + 1,
        ]
    })
}

fn recorta(r: [i32; 4], janela: [u32; 2]) -> [i32; 4] {
    #[expect(clippy::cast_possible_wrap, reason = "uma janela cabe num i32")]
    let (w, h) = (janela[0] as i32, janela[1] as i32);
    [r[0].max(0), r[1].max(0), r[2].min(w), r[3].min(h)]
}

/// ⭐ **O plano**: as instâncias cuja saída alcança a `janela`, empacotadas EM ORDEM em páginas de
/// lado até `lado_max` (o mesmo empacotador do Vector, [`pack_in_order`]). Uma instância que não
/// cabe numa página corre SOZINHA numa página do tamanho da janela, sem translação — o caso
/// «sozinha» por construção.
#[must_use]
pub fn planeia(caixas: &[Caixa], janela: [u32; 2], margem: u32, lado_max: u32) -> Vec<Pagina> {
    #[expect(clippy::cast_possible_wrap, reason = "uma margem cabe num i32")]
    let m = margem as i32;
    let mut vivas: Vec<(u32, [i32; 4])> = Vec::new();
    for (i, c) in caixas.iter().enumerate() {
        let Some(r) = conteudo(c) else { continue };
        let s = recorta([r[0] - m, r[1] - m, r[2] + m, r[3] + m], janela);
        if s[0] < s[2] && s[1] < s[3] {
            #[expect(clippy::cast_possible_truncation, reason = "instâncias cabem num u32")]
            vivas.push((i as u32, r));
        }
    }
    #[expect(clippy::cast_sign_loss, reason = "x1 ≥ x0")]
    let tamanhos: Vec<(u32, u32)> = vivas
        .iter()
        .map(|(_, r)| {
            (
                ((r[2] - r[0]) + 2 * m + 1) as u32,
                ((r[3] - r[1]) + 2 * m + 1) as u32,
            )
        })
        .collect();
    pack_in_order(&tamanhos, lado_max)
        .into_iter()
        .map(|lote| {
            let sozinha = lote.cells.len() == 1 && {
                let (w, h) = tamanhos[lote.cells[0].index];
                w > lado_max || h > lado_max
            };
            let celulas = lote
                .cells
                .iter()
                .map(|p| {
                    let (i, r) = vivas[p.index];
                    #[expect(clippy::cast_possible_wrap, reason = "um átlas cabe num i32")]
                    let t = if sozinha {
                        [0, 0]
                    } else {
                        [p.org[0] as i32 + m - r[0], p.org[1] as i32 + m - r[1]]
                    };
                    let c = &caixas[i as usize];
                    #[expect(clippy::cast_precision_loss, reason = "pixels de ecrã")]
                    Celula {
                        instancia: i,
                        t,
                        saida: recorta([r[0] - m, r[1] - m, r[2] + m, r[3] + m], janela),
                        org: [c.ancora[0] + t[0] as f32, c.ancora[1] + t[1] as f32],
                    }
                })
                .collect();
            Pagina {
                tamanho: if sozinha { janela } else { [lote.w, lote.h] },
                celulas,
            }
        })
        .collect()
}

#[cfg(test)]
#[path = "plano_tests.rs"]
mod tests;
