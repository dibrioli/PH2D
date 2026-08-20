//! **A precisão de uma imagem, e as duas conversões entre elas** — W1 do plano
//! [`docs/Sprite_projeto/18`](../../../docs/Sprite_projeto/18_precisao_de_16_bits_nas_sprites.md).
//!
//! Enio, 2026-08-20: *"vamos corrigir a UI da sprite no painel com as duas opções e com os botões
//! que a partir de agora devem converter a imagem."*
//!
//! # As duas precisões, e por que uma delas é meio-float
//!
//! | | armazenamento | espaço | bytes/px | textura |
//! |---|---|---|---|---|
//! | [`Precision::Rgba8`] | `u8`×4 | **sRGB-encoded** | 4 | `Rgba8UnormSrgb` |
//! | [`Precision::Rgba16`] | `u16`×4 (bits de meio-float) | **linear** | 8 | `Rgba16Float` |
//!
//! ⚠️ **`Rgba16Float` e não `Rgba16Unorm`, e isto é medição, não gosto:** o `Rgba16Unorm` exige
//! `Features::TEXTURE_FORMAT_16BIT_NORM` (wgpu-types 27, `lib.rs:3341`), que o
//! [`ph2d_gpu`](../../ph2d-gpu/src/context.rs) pede **mascarada pelo adapter** — ou seja, pode não
//! existir. O `Rgba16Float` é baseline do WebGPU, filtrável, e já é a moeda do resto da engine
//! (`GameRt`, `fx_stack`, `walk_gpu`, `mesh-render`).
//!
//! ⚠️ **O 16 bits guarda LINEAR, e isto é a armadilha desta wave.** Não existe variante sRGB de
//! formato de 16 bits algum. Hoje toda textura de sprite é `Rgba8UnormSrgb` e o **hardware** faz o
//! sRGB→linear na amostragem; um `Rgba16Float` **não faz**. Converter e continuar a tratar os
//! valores como sRGB renderiza a sprite visivelmente mais clara que a gémea de 8 bits.
//!
//! # O que a conversão NÃO faz
//!
//! ⛔ **Não mexe em premultiplicação.** A associação de alfa continua a ser dita pelo
//! `Sprite::premultiplied`, como sempre foi — misturar as duas leis aqui faria uma conversão de
//! precisão mudar silenciosamente a composição. Precisão e espaço de cor entram; alfa sai como
//! entrou.
//!
//! ⛔ **`Rgba8` → `Rgba16` não cria informação.** Uma imagem que nasceu de 8 bits continua com 256
//! degraus por canal depois de convertida; o que ela ganha é **não requantizar nas edições
//! seguintes**. A UI que oferece o botão tem de dizer isto (plano 18 §3.2).
//!
//! # A lei que este módulo deve, e que é exaustiva
//!
//! `8 → 16 → 8` devolve **o byte original, para os 256 valores**, e o gate
//! [`tests::the_round_trip_is_exact_for_every_byte`] varre-os todos. Não é um argumento sobre
//! precisão relativa: é uma varredura. Se um dia um valor falhar, é o gate que o nomeia.

use ph2d_color::srgb::{linear_to_srgb_byte, srgb_to_linear_byte};

/// A precisão com que uma imagem é guardada.
///
/// ⚠️ **É um FACTO da imagem, e ao mesmo tempo uma ESCOLHA do autor** — ao contrário do que a linha
/// `Format` do Inspector dizia antes do plano 18, em que era literal `true` e não estado nenhum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Precision {
    /// 8 bits por canal, sRGB-encoded. O default de todo o produto.
    #[default]
    Rgba8,
    /// 16 bits por canal (meio-float), linear.
    Rgba16,
}

impl Precision {
    /// Bytes por pixel — a conta de memória que a UI mostra antes de converter.
    #[must_use]
    pub const fn bytes_per_pixel(self) -> usize {
        match self {
            Self::Rgba8 => 4,
            Self::Rgba16 => 8,
        }
    }

    /// O nome que aparece na UI. ⚠️ Inglês, como toda a UI do app.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Rgba8 => "RGBA8",
            Self::Rgba16 => "RGBA16",
        }
    }
}

/// **`f32` → bits de meio-float**, com arredondamento *round-to-nearest-even* (o do IEEE-754, e o
/// que a GPU faz).
///
/// ⚠️ Escrito à mão de propósito: o `f16` nativo continua **instável** no toolchain fixado
/// (`rustc 1.95.0`, issue #116909 — sondado em 2026-08-20), e uma dependência nova por 40 linhas de
/// aritmética paga `deny`/`machete`/auditoria para sempre.
///
/// O domínio desta wave é **cor**, mas a função é total: infinitos e NaN sobrevivem, o overflow
/// satura em infinito, e o underflow desce por subnormais até zero.
#[must_use]
pub fn f32_to_half(value: f32) -> u16 {
    let bits = value.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let exp = ((bits >> 23) & 0xff) as i32;
    let mantissa = bits & 0x007f_ffff;

    // Infinito ou NaN. ⚠️ Um NaN tem de continuar NaN: truncar a mantissa para zero transformá-lo-ia
    // em infinito, que é a diferença entre "não é número" e "é o maior de todos".
    if exp == 0xff {
        let m = (mantissa >> 13) as u16;
        let m = if mantissa != 0 { m.max(1) } else { 0 };
        return sign | 0x7c00 | m;
    }

    // Expoente re-enviesado: f32 tem bias 127, meio-float tem 15.
    let half_exp = exp - 127 + 15;

    if half_exp >= 0x1f {
        return sign | 0x7c00; // satura em infinito
    }

    if half_exp <= 0 {
        // Subnormal do meio-float, ou pequeno demais até para isso. O corte em `-11` está 15 ordens
        // de grandeza abaixo do menor valor que uma cor pode ter (o byte sRGB 1 dá ~3e-4).
        if half_exp < -11 {
            return sign;
        }
        // Repõe o 1 implícito e desloca para a posição subnormal.
        let m = mantissa | 0x0080_0000;
        let shift = (14 - half_exp) as u32;
        let kept = m >> shift;
        let dropped = m & ((1u32 << shift) - 1);
        let halfway = 1u32 << (shift - 1);
        let mut out = kept;
        if dropped > halfway || (dropped == halfway && (kept & 1) == 1) {
            out += 1;
        }
        // ⚠️ Um `out` que transborda 0x3ff vira **exatamente** o menor normal (expoente 1, mantissa
        // 0) — o transporte para o campo do expoente é a resposta certa, não um bug a mascarar.
        return sign | out as u16;
    }

    // Normal. Largam-se os 13 bits de baixo da mantissa, arredondando ao par.
    let kept = mantissa >> 13;
    let dropped = mantissa & 0x1fff;
    let mut out = ((half_exp as u32) << 10) | kept;
    if dropped > 0x1000 || (dropped == 0x1000 && (kept & 1) == 1) {
        // O transporte entra no expoente de propósito; se chegar a 0x1f, o resultado é infinito,
        // que é o arredondamento correto de um valor acima do maior meio-float.
        out += 1;
    }
    sign | out as u16
}

/// **Bits de meio-float → `f32`.** Exata nos dois ramos: todo meio-float cabe num `f32`.
#[must_use]
pub fn half_to_f32(half: u16) -> f32 {
    let sign = ((half & 0x8000) as u32) << 16;
    let exp = ((half >> 10) & 0x1f) as u32;
    let mantissa = (half & 0x03ff) as u32;

    if exp == 0x1f {
        // Infinito (mantissa 0) ou NaN — a mantissa sobe para o campo do f32.
        return f32::from_bits(sign | 0x7f80_0000 | (mantissa << 13));
    }
    if exp == 0 {
        if mantissa == 0 {
            return f32::from_bits(sign); // ±0
        }
        // Subnormal: valor = mantissa × 2⁻²⁴. As duas parcelas são exatas em f32, e o produto
        // também — por isso a aritmética direta é preferível a re-normalizar por deslocamento.
        let magnitude = mantissa as f32 / 16_777_216.0;
        return f32::from_bits(sign | magnitude.to_bits());
    }
    // Normal: 127 - 15 = 112.
    f32::from_bits(sign | ((exp + 112) << 23) | (mantissa << 13))
}

/// **`Rgba8` sRGB → `Rgba16` linear.** Entrada empacotada RGBA; saída com o mesmo número de canais.
///
/// ⚠️ **O alfa NÃO passa pela curva sRGB** — ele já é linear por definição. Aplicar-lhe a curva é o
/// erro clássico desta conversão, e escurece toda borda macia da imagem.
#[must_use]
pub fn rgba8_to_rgba16(bytes: &[u8]) -> Vec<u16> {
    bytes
        .chunks_exact(4)
        .flat_map(|px| {
            [
                f32_to_half(srgb_to_linear_byte(px[0])),
                f32_to_half(srgb_to_linear_byte(px[1])),
                f32_to_half(srgb_to_linear_byte(px[2])),
                f32_to_half(f32::from(px[3]) / 255.0),
            ]
        })
        .collect()
}

/// **`Rgba16` linear → `Rgba8` sRGB.** ⚠️ **Este sentido PERDE**, e é o único dos dois que perde.
///
/// O alfa volta por escala direta, pela mesma razão de [`rgba8_to_rgba16`].
#[must_use]
pub fn rgba16_to_rgba8(halves: &[u16]) -> Vec<u8> {
    halves
        .chunks_exact(4)
        .flat_map(|px| {
            let alpha = (half_to_f32(px[3]).clamp(0.0, 1.0) * 255.0).round() as u8;
            [
                linear_to_srgb_byte(half_to_f32(px[0])),
                linear_to_srgb_byte(half_to_f32(px[1])),
                linear_to_srgb_byte(half_to_f32(px[2])),
                alpha,
            ]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Valores canónicos do IEEE-754 binary16, conferidos contra a tabela do formato.
    #[test]
    fn the_half_encoding_matches_the_ieee_754_table() {
        for (value, bits) in [
            (0.0_f32, 0x0000_u16),
            (-0.0, 0x8000),
            (1.0, 0x3c00),
            (-1.0, 0xbc00),
            (2.0, 0x4000),
            (0.5, 0x3800),
            (65504.0, 0x7bff),      // o maior meio-float finito
            (6.1035156e-5, 0x0400), // o menor normal
            (5.9604645e-8, 0x0001), // o menor subnormal
            (f32::INFINITY, 0x7c00),
            (f32::NEG_INFINITY, 0xfc00),
        ] {
            assert_eq!(
                f32_to_half(value),
                bits,
                "f32_to_half({value}) devia dar {bits:#06x}"
            );
            assert_eq!(
                half_to_f32(bits),
                value,
                "half_to_f32({bits:#06x}) devia dar {value}"
            );
        }
    }

    /// **O MODO de arredondamento, que nenhum outro gate aqui guarda.**
    ///
    /// ⚠️ Este teste existe porque uma prova de mutação o exigiu (2026-08-20): trocar o
    /// *round-to-nearest-even* por truncagem pura deixou **verdes** tanto a tabela IEEE (os seus
    /// valores são meios-floats exatos, sem bits a largar) como a varredura exaustiva da
    /// ida-e-volta (a folga entre a precisão do meio-float e o espaçamento sRGB absorve o erro).
    /// *Um mutante sobrevivente não é ruído — é o nome do gate que falta.*
    ///
    /// Os quatro casos são os quatro ramos da regra: acima do meio, abaixo do meio, e os dois
    /// empates — o que desce por o vizinho de baixo ser par, e o que sobe por ele ser ímpar.
    #[test]
    fn the_rounding_is_nearest_even_and_not_truncation() {
        // ⚠️ **Os empates DERIVAM-SE dos próprios vizinhos**, em vez de serem soletrados em
        // decimal. Um literal como `1.000_488_281_25` é o mesmo `f32` que `1.000_488_3` (o
        // `excessive_precision` do clippy diz isso, e tem razão) — mas nenhum dos dois DIZ que é o
        // ponto médio exato entre dois meios-floats vizinhos, que é a única propriedade que este
        // teste mede. Derivar torna a intenção verificável em vez de acreditada.
        let midpoint = |a: u16, b: u16| (half_to_f32(a) + half_to_f32(b)) * 0.5;
        // Perto de 1.0 o passo do meio-float é 2⁻¹⁰: 0x3c00 = 1.0 · 0x3c01 · 0x3c02.
        let (even_lo, odd_lo, odd_hi) = (0x3c00_u16, 0x3c01_u16, 0x3c02_u16);
        for (value, expected, why) in [
            (1.0006_f32, odd_lo, "acima do meio arredonda para cima"),
            (1.0004, even_lo, "abaixo do meio arredonda para baixo"),
            (
                midpoint(even_lo, odd_lo),
                even_lo,
                "empate com vizinho de baixo PAR desce (nearest-EVEN)",
            ),
            (
                midpoint(odd_lo, odd_hi),
                odd_hi,
                "empate com vizinho de baixo IMPAR sobe (nearest-EVEN)",
            ),
        ] {
            assert_eq!(
                f32_to_half(value),
                expected,
                "f32_to_half({value}) devia dar {expected:#06x} — {why}"
            );
        }
    }

    /// ⚠️ **Um NaN tem de sobreviver como NaN.** Truncar a mantissa para zero fá-lo-ia virar
    /// infinito — e um pixel "não é número" a virar "o mais brilhante possível" é exatamente o
    /// género de defeito que aparece uma vez por ano numa imagem só.
    #[test]
    fn a_nan_survives_as_a_nan() {
        assert!(half_to_f32(f32_to_half(f32::NAN)).is_nan());
    }

    /// **Overflow satura em infinito, não dá a volta.**
    #[test]
    fn a_value_above_the_range_saturates_instead_of_wrapping() {
        assert_eq!(f32_to_half(1.0e30), 0x7c00);
        assert_eq!(f32_to_half(-1.0e30), 0xfc00);
        // 65520 é o ponto médio entre o maior finito e o primeiro que já não cabe: arredonda para
        // cima, para infinito. É o comportamento do IEEE, e é o que a GPU faz.
        assert_eq!(f32_to_half(65520.0), 0x7c00);
        // Logo abaixo dele ainda é finito.
        assert_eq!(f32_to_half(65503.0), 0x7bff);
    }

    /// **A LEI desta wave, varrida e não argumentada:** `8 → 16 → 8` devolve o byte original para
    /// **todos** os 256 valores, em qualquer canal.
    ///
    /// ⚠️ Isto não é óbvio: o meio-float tem 11 bits de mantissa, e a conversão passa por sRGB→
    /// linear→sRGB. O que a torna verdadeira é a folga entre a precisão relativa do meio-float
    /// (~0,05%) e o espaçamento entre códigos sRGB vizinhos (≥0,9%, e muito maior nos escuros).
    /// *Uma folga medida é um facto; uma folga estimada é uma esperança.*
    #[test]
    fn the_round_trip_is_exact_for_every_byte() {
        let original: Vec<u8> = (0..=255u8).flat_map(|b| [b, b, b, b]).collect();
        let back = rgba16_to_rgba8(&rgba8_to_rgba16(&original));
        assert_eq!(back.len(), original.len());
        let mut broken = Vec::new();
        for (i, (a, b)) in original.iter().zip(back.iter()).enumerate() {
            if a != b {
                broken.push(format!("  canal {} do byte {}: {a} -> {b}", i % 4, i / 4));
            }
        }
        assert!(
            broken.is_empty(),
            "a ida-e-volta 8->16->8 deixou de ser exata:\n{}",
            broken.join("\n")
        );
    }

    /// **O alfa não atravessa a curva sRGB.** Controle direto do erro clássico: se alguém aplicar
    /// `srgb_to_linear_byte` ao alfa, o meio-canal 128 desce de ~0,502 para ~0,216 e este teste
    /// nomeia-o.
    #[test]
    fn alpha_is_linear_and_does_not_cross_the_srgb_curve() {
        let px = [0u8, 0, 0, 128];
        let half = rgba8_to_rgba16(&px);
        let alpha = half_to_f32(half[3]);
        assert!(
            (alpha - 128.0 / 255.0).abs() < 1e-3,
            "o alfa saiu {alpha}, e devia ser ~{}. Alguem aplicou a curva sRGB a ele.",
            128.0 / 255.0
        );
    }

    /// ⚠️ **Um canal de COR, esse, atravessa mesmo.** Sem este par, o teste acima passaria numa
    /// implementação que não converte nada.
    #[test]
    fn colour_channels_do_cross_the_srgb_curve() {
        let px = [128u8, 128, 128, 255];
        let half = rgba8_to_rgba16(&px);
        let red = half_to_f32(half[0]);
        assert!(
            (red - 0.2158).abs() < 1e-3,
            "o canal R saiu {red}, e o linear de sRGB 128 e' ~0,2158. A conversao nao aconteceu."
        );
    }

    #[test]
    fn the_byte_cost_is_the_one_the_ui_shows() {
        assert_eq!(Precision::Rgba8.bytes_per_pixel(), 4);
        assert_eq!(Precision::Rgba16.bytes_per_pixel(), 8);
        assert_eq!(Precision::default(), Precision::Rgba8);
    }
}
