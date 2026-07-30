//! **A grade do fluido não é a grade de PIXELS** — a porta única da conversão
//! (filho de [`super`], LOC cap).
//!
//! Até 2026-07-29 o motor nascia `Engine::new(w, h)` com `(w, h)` = o tamanho
//! do canvas, isto é **uma célula de fluido por pixel**. A 4096² isso são
//! 16,7 M células, e o custo do solver é **linear nas células vivas**
//! (medido: `b = 15,5 ns por célula-ativa`, contra os 16 ns/visita que o
//! ADR-0134 declara como o piso escalar desta física) ⇒ a água ficava em
//! **17-19 Hz** contra o nominal de 40 da SPEC, com o worker `busy 88%`.
//!
//! A razão **`grid_ratio`** (1..=30, autorada — o slider no topo da seção Wet
//! Paint) diz **quantos pixels de canvas medem uma célula de fluido**. `1` é o
//! mundo de sempre, **byte-idêntico por construção** (ver abaixo); `2` corta o
//! trabalho por ~3,3× e leva a água ao nominal.
//!
//! ⚠️ **O motor NÃO é tocado, e é isso que mantém o fingerprint do ADR-0134
//! intacto:** ele sempre foi agnóstico de dimensão (`Engine::new(gw, gh)` com
//! números menores é o MESMO código, e a suíte de aceitação roda em 900×450 /
//! 300×200 / 60×60 justamente porque a dimensão nunca foi parte da física).
//! O que a razão muda é **de quantos pixels o host fala com ele** — e essa
//! conversão vive TODA aqui.
//!
//! ## As quatro perguntas, e por que são uma família
//!
//! | pergunta | porta | quem chama |
//! |---|---|---|
//! | de que tamanho é a grade? | [`grid_dims`] | o nascimento da sessão |
//! | onde, em células, está este ponto de canvas? | [`px_to_cell`] | a rota do dab |
//! | que pixel de canvas é o centro desta célula? | [`cell_center_px`] / [`cell_center_texel`] | a silhueta, o grain, o papel |
//! | de que células sai este pixel? | [`SampleU::at`] | o composite |
//!
//! Elas TÊM de ser inversas uma da outra: se o dab pousa numa célula e a
//! silhueta é avaliada noutra, o carimbo sai deslocado de meia célula — a
//! doença de coordenada derivada que este repo já pagou várias vezes
//! (`feedback_derived_coordinate_seed_must_match_sample`). Escrever a
//! aritmética duas vezes é como isso acontece, então ela é escrita **uma**.
//!
//! ## A convenção, e a razão de ela ser exatamente esta
//!
//! A célula `c` (1-based, o pad ring é 0 e `n+1`) **cobre** os pixels de canvas
//! `[(c-1)·r, c·r)`. Logo o centro dela, em coordenada de canvas contínua
//! (onde o pixel `i` tem centro `i + 0,5`), é `(c-1)·r + r/2`.
//!
//! O motor recebe posições contínuas em que o centro da célula `c` vale
//! `c + 0,5` — é o que o `+ 1.0` que a rota do dab sempre somou significa
//! (`px_center = i + 0,5` ⇒ `i + 1,5 = (i+1) + 0,5`, e `i+1` é a célula). Com
//! a razão isso vira **`u = px / r + 1,0`**, e em `r = 1` a expressão é
//! *literalmente* `px + 1.0`: o mesmo `f64`, os mesmos bits, sem ramo. É por
//! isso que a razão 1 não pode divergir do que shipava.

/// Quantos pixels de canvas medem uma célula, aos limites do slider.
///
/// O teto é o do **slider que o Enio pediu** (1:1 a 1:30), não um limite de
/// recurso: a razão 30 a 4096² dá uma grade de 137×137, que é menor que a
/// fixture de 300×200 da suíte de aceitação. O que aperta primeiro é o
/// DESENHO (uma célula de 30 px é uma poça de 30 px), e isso é o olho do
/// artista, não a máquina.
pub(in crate::tool::paint) const MIN_RATIO: u8 = 1;
pub(in crate::tool::paint) const MAX_RATIO: u8 = 30;

/// A razão de fábrica: **1**, a grade de sempre.
///
/// ⚠️ Deliberadamente NÃO o valor rápido. Um default que muda a resolução do
/// fluido mudaria o desenho de toda arte já feita, e a escolha do ponto de
/// operação é do artista — o slider existe exatamente para isso. O que a
/// medição diz (e o smoke confirma) fica no doc 28 §5.41.
pub(in crate::tool::paint) const DEFAULT_RATIO: u8 = 1;

/// Clampa uma razão vinda de fora (o chip numérico do painel) à faixa viva.
pub(in crate::tool::paint) fn clamp_ratio(r: u8) -> u8 {
    r.clamp(MIN_RATIO, MAX_RATIO)
}

/// De que tamanho nasce a grade para um canvas `w × h`.
///
/// ⚠️ **Arredonda para CIMA** e o motivo é de correção, não de estética: com
/// `floor` a última fatia de pixels do canvas não teria célula nenhuma, e
/// tinta pousada ali seria descartada em silêncio — a borda direita/inferior
/// do documento deixaria de aceitar água. O `max(1)` cobre um canvas menor
/// que uma célula.
pub(in crate::tool::paint) fn grid_dims(w: usize, h: usize, ratio: u8) -> (usize, usize) {
    let r = usize::from(clamp_ratio(ratio));
    (w.div_ceil(r).max(1), h.div_ceil(r).max(1))
}

/// A coordenada de CÉLULA (contínua, como o motor a quer) de um ponto de
/// canvas — a porta da rota do dab.
///
/// Em `ratio == 1` isto é `px + 1.0` ao bit (ver o doc do módulo).
pub(in crate::tool::paint) fn px_to_cell(px: f64, ratio: u8) -> f64 {
    if ratio <= 1 {
        return px + 1.0;
    }
    px / f64::from(ratio) + 1.0
}

/// Um comprimento de canvas medido em células (raios de dab, cordas de
/// segmento). Em `ratio == 1` devolve o próprio número.
pub(in crate::tool::paint) fn px_len_to_cell(len: f64, ratio: u8) -> f64 {
    if ratio <= 1 {
        return len;
    }
    len / f64::from(ratio)
}

/// O centro da célula `c` em coordenada de canvas CONTÍNUA (o mesmo espaço em
/// que o pixel `i` tem centro `i + 0,5`) — a inversa de [`px_to_cell`].
///
/// Em `ratio == 1` é `(c - 1) as f32 + 0.5`, que é exatamente o que a
/// silhueta computava (`px as f32 + 0.5`, `px = c - 1`).
pub(in crate::tool::paint) fn cell_center_px(c: i32, ratio: u8) -> f32 {
    let r = f32::from(clamp_ratio(ratio));
    (c - 1) as f32 * r + r * 0.5
}

/// O TEXEL de canvas representativo da célula `c` — o inteiro que os
/// amostradores de imagem (Shape, Grain, Paper) recebem.
///
/// ⚠️ Um amostrador de imagem indexa uma grade de texels, então ele não pode
/// receber a coordenada contínua: ele recebe o texel onde o centro da célula
/// caiu. Em `ratio == 1` é `c - 1`, o que aqueles três chamadores já passavam.
pub(in crate::tool::paint) fn cell_center_texel(c: i32, ratio: u8) -> i64 {
    let r = i64::from(clamp_ratio(ratio));
    i64::from(c - 1) * r + r / 2
}

/// A janela de canvas que uma janela de CÉLULAS cobre (as duas inclusivas em
/// células 1-based; a de saída é meio-aberta em pixels e já clampada ao
/// documento).
///
/// É a tradução do dirty rect do motor para o retângulo que o composite
/// escreve e que o `mark_dirty` publica.
pub(in crate::tool::paint) fn cell_rect_to_px(
    (cx0, cy0, cx1, cy1): (usize, usize, usize, usize),
    ratio: u8,
    w: usize,
    h: usize,
) -> (usize, usize, usize, usize) {
    let r = usize::from(clamp_ratio(ratio));
    let x0 = (cx0 - 1) * r;
    let y0 = (cy0 - 1) * r;
    let x1 = (cx1 * r).min(w);
    let y1 = (cy1 * r).min(h);
    (x0, y0, x1, y1)
}

/// O amostrador do composite: de que células — e com que pesos — sai um pixel.
///
/// ⚠️ **É bilinear, e não vizinho-mais-próximo, por uma razão medida:** a
/// razão 30 que o slider oferece faz de cada célula um bloco de 30×30 px, e
/// *nearest* pintaria a água como um mosaico de quadrados de 900 px². O
/// campo de pigmento é suave por construção (o solver difunde), então
/// interpolá-lo é ler o que ele já diz entre os nós, não inventar detalhe.
///
/// ⚠️ **E interpola PREMULTIPLICADO.** O pigmento é *straight alpha*, e
/// interpolar cor straight entre um vizinho opaco e um transparente **puxa a
/// cor do vizinho vazio para dentro da tinta** (o halo escuro clássico). Em
/// premultiplicado a mistura é linear e correta; o composite desmultiplica no
/// fim, onde ele já dividia por `oa`.
#[derive(Clone, Copy)]
pub(in crate::tool::paint) struct SampleU {
    /// `1 / ratio`, pré-dividido (a divisão sairia por pixel).
    inv_r: f64,
    /// O maior índice de célula válido em cada eixo (o clamp da borda).
    max_x: usize,
    max_y: usize,
    /// Passo de linha do plano de pigmento, em bytes.
    stride: usize,
    /// `true` quando a razão é 1 — o caminho de cópia direta.
    identity: bool,
}

impl SampleU {
    /// `gw`/`gh` são as dimensões da GRADE (sem o pad ring).
    pub(in crate::tool::paint) fn new(ratio: u8, gw: usize, gh: usize) -> Self {
        Self {
            inv_r: 1.0 / f64::from(clamp_ratio(ratio)),
            max_x: gw,
            max_y: gh,
            stride: gw * 4,
            identity: clamp_ratio(ratio) == 1,
        }
    }

    /// Verdadeiro quando um pixel É uma célula (a rota byte-idêntica).
    pub(in crate::tool::paint) fn is_identity(self) -> bool {
        self.identity
    }

    /// A coordenada contínua de célula de um pixel, no eixo — a MESMA lei de
    /// [`px_to_cell`] aplicada ao centro do pixel (`px + 0,5`), que é onde a
    /// cor de um pixel é definida.
    fn u(self, px: usize) -> f64 {
        (px as f64 + 0.5) * self.inv_r + 0.5
    }

    /// Amostra o plano de pigmento (grade-shaped, RGBA8 *straight*) no pixel
    /// de canvas `(px, py)` e devolve **premultiplicado**, em 0..255.
    ///
    /// O chamador da rota identidade não passa por aqui — ele lê os quatro
    /// bytes direto, e é isso que faz a razão 1 não ter uma segunda resposta.
    pub(in crate::tool::paint) fn at(self, pig: &[u8], px: usize, py: usize) -> [f64; 4] {
        let (ux, uy) = (self.u(px), self.u(py));
        // `floor` e não `as usize`: a coordenada nunca é negativa aqui (o
        // pixel é >= 0 e o `+0.5` só empurra para cima), mas o `floor`
        // explícito é o que casa com o peso `frac` abaixo.
        let (fx0, fy0) = (ux.floor(), uy.floor());
        let (tx, ty) = (ux - fx0, uy - fy0);
        // Clamp de BORDA (o pixel do canto tem `u < 1`, e a célula 0 é pad).
        let cx0 = (fx0 as usize).clamp(1, self.max_x);
        let cy0 = (fy0 as usize).clamp(1, self.max_y);
        let cx1 = (cx0 + 1).min(self.max_x);
        let cy1 = (cy0 + 1).min(self.max_y);
        let mut out = [0.0f64; 4];
        // Quatro cantos, pesos bilineares, tudo premultiplicado.
        for (cy, wy) in [(cy0, 1.0 - ty), (cy1, ty)] {
            if wy == 0.0 {
                continue;
            }
            let row = (cy - 1) * self.stride;
            for (cx, wx) in [(cx0, 1.0 - tx), (cx1, tx)] {
                if wx == 0.0 {
                    continue;
                }
                let o = row + (cx - 1) * 4;
                let a = f64::from(pig[o + 3]);
                let w = wx * wy;
                out[0] += f64::from(pig[o]) * a * w;
                out[1] += f64::from(pig[o + 1]) * a * w;
                out[2] += f64::from(pig[o + 2]) * a * w;
                out[3] += a * w;
            }
        }
        // As três primeiras saem multiplicadas por alpha em 0..255 (ou seja,
        // por 255x o alpha normalizado); o composite divide pelo alpha
        // acumulado, então a escala cancela.
        out
    }
}

#[cfg(test)]
#[path = "grid_map_tests.rs"]
mod tests;
