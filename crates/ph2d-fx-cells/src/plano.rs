//! **O PLANO das células** — que instância vai para que célula de que página do átlas, e a
//! translação INTEIRA de cada uma (camada → átlas). Puro: sem `wgpu`, prova-se inteiro na cabeça.
//!
//! Uma célula é a caixa da instância no ecrã (arredondada para fora ao pixel, mais um de folga de
//! anti-aliasing) com a [`Vala`] à volta: a saída dela é o conteúdo mais o que a pilha ESPALHA
//! (`A`), e cada pixel da saída LÊ no máximo `L` para cada lado — entre os conteúdos de duas
//! células há `A + L + 1` pixels, logo nenhuma leitura alcança a vizinha (doc 129 §3.4, doc 131
//! §7 C.10).

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

/// **A vala entre células, POR LADO** (`[l, t, r, b]`): o que a pilha ESPALHA para fora do conteúdo
/// ([`ph2d_render::stack_reach`]) e o que ela LÊ em volta de um pixel que escreve
/// ([`ph2d_render::stack_read_por_lado`]).
///
/// ⭐ Entre os conteúdos de duas células vizinhas em `x` bastam
/// `max(A_r + L_r, A_l + L_l, A_l + A_r) + 1`: a leitura de cada saída para o lado da vizinha, e as
/// duas saídas sem se tocarem (em `y`, o mesmo com `t`/`b`). Não `2·max(A, L) + 1` (o W3b: um degrau de DENTRO, `A = 0`, `L = 23`,
/// separava as formas por 47 px quando 24 bastam, doc 131 §7 C.10), nem `A + L + 1` com `L` igual
/// nos quatro lados: uma Drop Shadow lê pouco para o lado OPOSTO ao deslocamento (6/6: a célula de um
/// disco de 14 px `60×60 → 54×54`; a longa `14/0`: `76×76 → 62×48`, doc 132 §7) — e a pilha custa
/// pela ÁREA das páginas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vala {
    pub alcance: [u32; 4],
    pub leitura: [u32; 4],
}

impl Vala {
    /// A vala do W3b: `M = max(A, L)` dos quatro lados e para ler — a forma «dupla» (`2M + 1`).
    #[must_use]
    pub fn dupla(self) -> Self {
        let m = self
            .alcance
            .iter()
            .chain(&self.leitura)
            .copied()
            .fold(0, u32::max);
        Self {
            alcance: [m; 4],
            leitura: [m; 4],
        }
    }
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
///
/// Na célula, o conteúdo começa a `A` (o maior alcance) do canto e acaba a `max(L, A) + 1` do fim:
/// a saída cabe na célula e a vala até à vizinha é `A + max(L, A) + 1`.
#[must_use]
pub fn planeia(caixas: &[Caixa], janela: [u32; 2], vala: Vala, lado_max: u32) -> Vec<Pagina> {
    planeia_com_par(caixas, janela, vala, lado_max, 0)
}

/// [`planeia`] com a translação PAR nas instâncias `[0, par)` (as sprites): a célula ganha um pixel
/// de folga por eixo e o conteúdo anda um pixel dentro dela quando a translação sairia ímpar. O
/// sampler das sprites (trilinear, anisotropia 16) lê as derivadas do quadrado 2×2 de pixels, e uma
/// translação ímpar troca o quadrado de cada pixel da aresta: 75 texels da silhueta de cinco
/// sprites mudavam entre «cada forma» e «a camada inteira» (doc 132 §6, P6).
#[must_use]
pub fn planeia_com_par(
    caixas: &[Caixa],
    janela: [u32; 2],
    vala: Vala,
    lado_max: u32,
    par: u32,
) -> Vec<Pagina> {
    #[expect(clippy::cast_possible_wrap, reason = "uma margem cabe num i32")]
    let [al, at, ar, ab] = vala.alcance.map(|v| v as i32);
    // `(antes, depois)` por eixo: o conteúdo começa a `antes` do canto da célula e acaba a `depois`
    // do fim — a vala até à vizinha é `antes + depois`.
    #[expect(clippy::cast_possible_wrap, reason = "uma margem cabe num i32")]
    let [ll, lt, lr, lb] = vala.leitura.map(|v| v as i32);
    let vala_x = (ar + lr).max(al + ll).max(al + ar) + 1;
    let vala_y = (ab + lb).max(at + lt).max(at + ab) + 1;
    let (antes, depois) = ([al, at], [vala_x - al, vala_y - at]);
    let saida = |c: [i32; 4]| recorta([c[0] - al, c[1] - at, c[2] + ar, c[3] + ab], janela);
    let mut vivas: Vec<(u32, [i32; 4])> = Vec::new();
    for (i, c) in caixas.iter().enumerate() {
        let Some(r) = conteudo(c) else { continue };
        let s = saida(r);
        if s[0] < s[2] && s[1] < s[3] {
            #[expect(clippy::cast_possible_truncation, reason = "instâncias cabem num u32")]
            vivas.push((i as u32, r));
        }
    }
    let folga = |i: u32| i32::from(i < par);
    #[expect(clippy::cast_sign_loss, reason = "x1 ≥ x0")]
    let tamanhos: Vec<(u32, u32)> = vivas
        .iter()
        .map(|&(i, r)| {
            (
                ((r[2] - r[0]) + antes[0] + depois[0] + folga(i)) as u32,
                ((r[3] - r[1]) + antes[1] + depois[1] + folga(i)) as u32,
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
                        let f = folga(i);
                        let t = [
                            p.org[0] as i32 + antes[0] - r[0],
                            p.org[1] as i32 + antes[1] - r[1],
                        ];
                        t.map(|v| v + (v & f))
                    };
                    let c = &caixas[i as usize];
                    #[expect(clippy::cast_precision_loss, reason = "pixels de ecrã")]
                    Celula {
                        instancia: i,
                        t,
                        saida: saida(r),
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
