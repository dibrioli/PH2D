//! **O ALPHA POR IMAGEM** — os pixels que um padrão autorado carrega.
//!
//! ⚠️ **Módulo irmão do [`super::alpha`], e o corte é o mesmo do
//! [`super::alpha_frame`]:** lá moram *as nove fórmulas* — funções puras, sem um
//! byte de estado —, aqui mora *a única que traz DADOS consigo*. Uma imagem não é
//! uma décima fórmula: ela é a mesma pergunta respondida por uma tabela em vez de
//! por aritmética, e o que ela precisa (memória, uma fonte, um ciclo de vida) não
//! se parece com nada que o `alpha.rs` conhece.
//!
//! ## A LEI: imagem → peso
//!
//! `peso = luminância(rgb) × alfa`.
//!
//! ⚠️ **A luminância não foi escolhida aqui — ela é a convenção do repo e da
//! indústria**, e responder de novo seria a segunda porta: o slot **Shape** do
//! Painter 2D já diz, no próprio tipo, *"an imported image's **luminance**"*
//! ([`ph2d_painter_brush::texture::ImageMask`]), e é a mesma lei do ZBrush, do
//! Photoshop e do Blender — **branco é cheio**.
//!
//! ⚠️ **E o `× alfa` é a metade que o Painter não precisava:** a fonte aqui é um
//! **sprite da cena**, que tem canal alfa, e um texel transparente não tem tinta
//! nenhuma — ler o RGB dele seria ler o que o formato deixou lá. Com a
//! multiplicação a lei é UMA e reduz **exatamente** à luminância pura numa imagem
//! opaca (`a = 255` ⇒ `× 1.0`), que é o caso do Painter.
//!
//! ⚠️ **Consequência NOMEADA, não defeito:** um desenho de tinta PRETA sobre
//! transparência vira um alpha **vazio** — a tinta tem luminância zero. É o que
//! acontece no ZBrush pela mesma razão, e é por isso que a convenção existe:
//! *branco é cheio*. Quem quer o desenho como padrão o desenha claro.
//!
//! ## A memória é a da FONTE, e isso é o teto
//!
//! Um texel guarda **um byte**. Não é um teto de conforto: o sprite é RGBA8, então
//! converter para `f32` na carga custaria **4×** a memória por **zero** informação
//! nova — o número que sai da tabela não pode ter mais precisão do que a fonte
//! tinha. A amostragem é bilinear, e é ela que devolve o contínuo entre texels.
//!
//! ## E ela é `libm`-free, como as nove
//!
//! Nada aqui chama transcendental: a amostra é `floor`, subtração, multiplicação
//! e soma. A promessa que o cabeçalho do `alpha.rs` faz vale para a décima.

/// **Uma IMAGEM autorada como padrão** — um peso por texel, `0..=255`.
///
/// ⚠️ **Os campos são privados e não há construtor literal:** a única porta é
/// [`AlphaImage::from_rgba`], que é onde a lei mora. Um `AlphaImage { .. }`
/// montado à mão por fora poderia ter `px.len() != w * h`, e a amostra leria
/// memória de outra linha — o `debug_assert` não alcança um build de release do
/// artista.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlphaImage {
    w: u32,
    h: u32,
    /// Peso por texel, row-major, `w * h` bytes. Ver a LEI no cabeçalho.
    px: Box<[u8]>,
}

/// Os coeficientes de luminância do Rec. 709 — os MESMOS que a `ph2d-light` usa
/// para dizer que o piso do ambiente preserva a média.
///
/// ⚠️ **Eles são o padrão do repo, não uma escolha deste módulo.** Um segundo
/// trio aqui faria duas partes do app discordarem sobre *"quão claro é este
/// cinza?"*, que é a pergunta que a palavra "luminância" já responde.
const LUM: [f32; 3] = [0.2126, 0.7152, 0.0722];

impl AlphaImage {
    /// **A PORTA** — pixels RGBA8 (row-major, straight) viram pesos.
    ///
    /// `None` quando as dimensões não descrevem o buffer, e a recusa é
    /// deliberada: uma imagem meio-lida é um padrão que desenha lixo, e o
    /// chamador que a lê de um sprite tem um caminho de erro para mostrar.
    ///
    /// ⚠️ **A conversão acontece UMA vez, na carga.** Ela é `O(texels)` e o
    /// caminho quente é a amostra — um `weight_at` que fizesse luminância por
    /// vértice pagaria três multiplicações a mais em cada um dos milhões que um
    /// dab toca, para chegar ao número que esta função já sabia.
    #[must_use]
    pub fn from_rgba(w: u32, h: u32, rgba: &[u8]) -> Option<Self> {
        let n = (w as usize).checked_mul(h as usize)?;
        if n == 0 || rgba.len() < n.checked_mul(4)? {
            return None;
        }
        let mut px = Vec::with_capacity(n);
        for t in rgba.chunks_exact(4).take(n) {
            let a = f32::from(t[3]) * (1.0 / 255.0);
            let l = LUM[0].mul_add(
                f32::from(t[0]),
                LUM[1].mul_add(f32::from(t[1]), LUM[2] * f32::from(t[2])),
            );
            // `l` já está em `0..=255`; o alfa é a fração que sobrevive.
            px.push((l * a) as u8);
        }
        Some(Self {
            w,
            h,
            px: px.into_boxed_slice(),
        })
    }

    /// Largura em texels.
    #[must_use]
    pub fn width(&self) -> u32 {
        self.w
    }

    /// Altura em texels.
    #[must_use]
    pub fn height(&self) -> u32 {
        self.h
    }

    /// **O peso em `[0, 1]` na coordenada `(u, v)`**, em unidades de TILE: `1.0`
    /// é uma imagem inteira.
    ///
    /// ⚠️ **LADRILHADO, e não recortado — e é a diferença que o separa do slot
    /// Shape do Painter.** Lá uma imagem É uma ponta finita, e fora dela a
    /// cobertura é zero; aqui ela é um **padrão**, irmão dos nove que cobrem a
    /// superfície inteira. Recortá-la faria o pincel pintar um retângulo em vez
    /// de uma textura, e o artista descobriria isso pela borda.
    ///
    /// ⚠️ **Bilinear com WRAP nos dois vizinhos**, não só na origem: o texel à
    /// direita do último é o primeiro, senão a última coluna de cada ladrilho
    /// interpolaria contra ela mesma e o padrão ganharia uma **costura** de um
    /// texel em cada emenda — visível exatamente onde o olho procura repetição.
    #[must_use]
    pub fn sample(&self, u: f32, v: f32) -> f32 {
        if !u.is_finite() || !v.is_finite() {
            return 0.0;
        }
        let (fw, fh) = (self.w as f32, self.h as f32);
        // Coordenada de texel com o centro em `+0.5`: a amostra de `u = 0` cai no
        // MEIO do primeiro texel, não na fronteira dele. É a mesma convenção do
        // `sample_shape` do Painter, e a que evita o meio-texel de deslocamento
        // que o repo já pagou noutro lugar.
        let x = u.mul_add(fw, -0.5);
        let y = v.mul_add(fh, -0.5);
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (x - x0, y - y0);
        let ix = wrap(x0, self.w);
        let iy = wrap(y0, self.h);
        let ix1 = if ix + 1 == self.w { 0 } else { ix + 1 };
        let iy1 = if iy + 1 == self.h { 0 } else { iy + 1 };

        let at = |cx: u32, cy: u32| -> f32 {
            let i = cy as usize * self.w as usize + cx as usize;
            f32::from(self.px[i]) * (1.0 / 255.0)
        };
        let top = at(ix, iy) + (at(ix1, iy) - at(ix, iy)) * fx;
        let bot = at(ix, iy1) + (at(ix1, iy1) - at(ix, iy1)) * fx;
        top + (bot - top) * fy
    }
}

/// Índice de texel de uma coordenada já em `floor`, envolvida no domínio `0..n`.
///
/// ⚠️ **O `rem_euclid` é o que torna a coordenada NEGATIVA correta**, e ela
/// acontece: o padrão é lido em espaço de objeto, cuja origem fica no meio da
/// peça, então metade da malha tem `u < 0`. Um `%` cru devolveria negativo ali e
/// o `as u32` o dobraria num índice enorme.
fn wrap(v: f32, n: u32) -> u32 {
    // `n >= 1` por construção (o `from_rgba` recusa `n == 0`).
    let m = v.rem_euclid(n as f32);
    // O `min` é a rede contra o único caso que a aritmética permite: `rem_euclid`
    // pode devolver exatamente `n` quando `v` é um negativo minúsculo.
    (m as u32).min(n - 1)
}

#[cfg(test)]
#[path = "alpha_image_tests.rs"]
mod tests;
