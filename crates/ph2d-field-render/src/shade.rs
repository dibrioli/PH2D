//! **O SOMBREAMENTO** — do G-buffer aos pixels. Irmão de [`super`] pelo mesmo teto de LOC.
//!
//! ⚠️ Aqui não há decisão de geometria nenhuma: o traçador entrega máscara e normal, e este arquivo
//! é a única coisa do módulo que sabe o que é uma **cor**. A fronteira é a razão de o traçador não
//! conhecer matcap nenhum (ver o doc de [`super`]).

use super::*;

/// Os texels de um matcap, **em linear**, lado × lado, RGB.
///
/// ⚠️ Quem os fornece é o chamador — ver a nota do `Cargo.toml` sobre não arrastar o `wgpu` para
/// dentro de um traçador de CPU.
pub struct Matcap<'a> {
    pub side: u32,
    /// `side * side * 3` valores lineares.
    pub rgb_linear: &'a [f32],
}

impl Matcap<'_> {
    /// Amostra **bilinear**, em linear.
    ///
    /// ⚠️ Bilinear e não vizinho-mais-próximo: a normal varia **continuamente** sobre uma superfície
    /// curva, e o vizinho-mais-próximo transforma essa rampa contínua nos degraus da grelha do
    /// matcap. O sintoma é **banda** numa barriga lisa.
    ///
    /// ⚠️ **Quanto isto morde depende do matcap, e não foi medido no asset da casa** (749², onde a
    /// grelha é fina o bastante para o efeito ser pequeno). Está aqui por ser o certo em qualquer
    /// tamanho — não por ter sido a causa de um sintoma relatado. *Uma correção certa não precisa
    /// de reivindicar um bug que ninguém provou que ela cura.*
    #[must_use]
    fn sample(&self, u: f32, v: f32) -> [f32; 3] {
        let side = self.side as usize;
        // −0,5 porque o texel é uma ÁREA e a coordenada dele é o CENTRO dela: sem isso a imagem
        // desloca-se meio texel e as bordas do matcap espelham-se erradas.
        let fx = (u * side as f32 - 0.5).clamp(0.0, side as f32 - 1.0);
        let fy = (v * side as f32 - 0.5).clamp(0.0, side as f32 - 1.0);
        let (x0, y0) = (fx.floor() as usize, fy.floor() as usize);
        let (x1, y1) = ((x0 + 1).min(side - 1), (y0 + 1).min(side - 1));
        let (tx, ty) = (fx - x0 as f32, fy - y0 as f32);
        let texel = |x: usize, y: usize| -> [f32; 3] {
            let t = (y * side + x) * 3;
            [
                self.rgb_linear[t],
                self.rgb_linear[t + 1],
                self.rgb_linear[t + 2],
            ]
        };
        let (a, b, c, d) = (texel(x0, y0), texel(x1, y0), texel(x0, y1), texel(x1, y1));
        let mut out = [0.0f32; 3];
        for k in 0..3 {
            let top = a[k] + (b[k] - a[k]) * tx;
            let bot = c[k] + (d[k] - c[k]) * tx;
            out[k] = top + (bot - top) * ty;
        }
        out
    }

    /// A cor de uma normal de vista. A lei de amostragem é a do matcap: `uv = n.xy * 0.5 + 0.5`.
    #[must_use]
    fn colour(&self, n: [f32; 3]) -> [f32; 3] {
        let u = (n[0] * 0.5 + 0.5).clamp(0.0, 1.0);
        let v = (1.0 - (n[1] * 0.5 + 0.5)).clamp(0.0, 1.0);
        self.sample(u, v)
    }
}

/// Colore o G-buffer com um matcap e devolve RGBA8 **pré-multiplicado**.
///
/// A lei de amostragem é a do matcap: `uv = n.xy * 0.5 + 0.5`, com `n` em espaço de vista. É por
/// isso que ela mora aqui, ao lado de quem produz essa normal — e não do outro lado do repositório,
/// onde a convenção teria de ser re-afirmada num comentário.
///
/// # ⚠️ Pré-multiplicado, e a resolução é em LINEAR
///
/// Duas escolhas que não são gosto:
///
/// - **Pré-multiplicado** porque a imagem vai ser **filtrada** ao ser desenhada, e num alfa direto o
///   filtro mistura a cor de pixels transparentes — cuja cor não significa nada. O sintoma é a
///   auréola escura à volta da peça, e é *o* bug clássico de compor imagem com borda macia.
/// - **Média em linear**, nunca em bytes sRGB. Metade de branco com metade de preto não é cinza-127
///   — é cinza-188. Fazer a média em sRGB escurece toda borda, que é o outro bug clássico e o mais
///   difícil de ver porque parece só "um contorno".
#[must_use]
pub fn shade(g: &Gbuffer, m: &Matcap<'_>, background: [u8; 4]) -> Vec<u8> {
    let bg_a = f32::from(background[3]) / 255.0;
    // O fundo, já pré-multiplicado e em linear — é ele que entra na média de um pixel de borda.
    let bg = [
        ph2d_color::srgb::srgb_to_linear_byte(background[0]) * bg_a,
        ph2d_color::srgb::srgb_to_linear_byte(background[1]) * bg_a,
        ph2d_color::srgb::srgb_to_linear_byte(background[2]) * bg_a,
        bg_a,
    ];
    let write = |px: &mut [u8], c: [f32; 4]| {
        px[0] = ph2d_color::srgb::linear_to_srgb_byte(c[0]);
        px[1] = ph2d_color::srgb::linear_to_srgb_byte(c[1]);
        px[2] = ph2d_color::srgb::linear_to_srgb_byte(c[2]);
        px[3] = (c[3].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    };

    let mut out = vec![0u8; (g.width as usize) * (g.height as usize) * 4];
    if m.side == 0 {
        for px in out.chunks_exact_mut(4) {
            px.copy_from_slice(&background);
        }
        return out;
    }

    for (i, px) in out.chunks_exact_mut(4).enumerate() {
        if g.hit[i] {
            let rgb = m.colour(g.normal[i]);
            write(px, [rgb[0], rgb[1], rgb[2], 1.0]);
        } else {
            // ⚠️ Copiado, e não passado pela conversão: um pixel de fundo puro tem de sair
            // **exatamente** com os bytes que o chamador pediu. Levá-lo pela ida-e-volta sRGB faria
            // a cor do fundo depender da precisão de uma tabela — e um fundo que quase bate é a
            // costura mais difícil de ver que existe.
            px.copy_from_slice(&background);
        }
    }

    // As bordas, resolvidas em COR — e não pela média das normais.
    //
    // ⚠️ A diferença aparece exatamente onde uma superfície passa à frente de outra: ali as duas
    // normais podem ser quase opostas, e a média delas aponta para um sítio do matcap que não é
    // nenhuma das duas cores. Média de normais é interpolar a GEOMETRIA; o que se quer é interpolar
    // o que se vê.
    for e in &g.edges {
        let i = e.pixel as usize;
        let mut acc = [0.0f32; 4];
        for k in 0..4 {
            let c = if e.hit[k] {
                let rgb = m.colour(e.normal[k]);
                [rgb[0], rgb[1], rgb[2], 1.0]
            } else {
                bg
            };
            for j in 0..4 {
                acc[j] += c[j] * 0.25;
            }
        }
        write(&mut out[i * 4..i * 4 + 4], acc);
    }
    out
}
