//! **O dither da descida para 8 bits** — W6 do plano
//! [`docs/Sprite_projeto/18`](../../../docs/Sprite_projeto/18_precisao_de_16_bits_nas_sprites.md).
//!
//! # O defeito que isto cura
//!
//! Uma imagem de 16 bits tem ~2048 degraus por canal perto de 1.0; a de 8 bits tem 256. Quando a
//! primeira desce para a segunda, todo pixel de uma faixa contínua de valores cai no **mesmo** byte
//! — e a fronteira onde ele passa a cair no byte seguinte é uma **linha visível**. Num degradê
//! limpo isso lê-se como faixas; à volta de um brilho, como anéis.
//!
//! O dither substitui essa fronteira única por uma mistura fina dos dois bytes vizinhos, na
//! proporção do valor real. A informação que a quantização deitava fora passa a viver na
//! **densidade** em vez de no valor — é a mesma troca que a meia-tinta de impressão faz há um
//! século.
//!
//! ⛔ **Isto NÃO é o `Dither` do Color Equalization.** Aquele é um efeito **de estilo**: ele reduz
//! a imagem a N níveis de propósito e espalha o erro com Floyd–Steinberg, para dar o aspeto de arte
//! por pixels. Este é **técnico e invisível**: amplitude abaixo de meio passo, e nenhum pixel que já
//! estava certo se mexe. *Duas coisas com o mesmo nome, e só uma delas o utilizador quer ver.*
//!
//! # Por que ORDENADO (Bayer) e não difusão de erro
//!
//! | | Bayer ordenado | Floyd–Steinberg |
//! |---|---|---|
//! | função de | **só a posição do pixel** | todos os pixels anteriores |
//! | paralelizável | sim (por pixel) | não (serial por varrimento) |
//! | determinista | **por construção** | sim, mas só se a ordem for fixa |
//! | recorte / colagem | o padrão **não muda** | o erro entra diferente e a imagem muda |
//!
//! A última linha é a que decide. Este módulo serve uma engine em que a mesma imagem é recortada,
//! ladrilhada e desenhada em pedaços; um dither que dependesse do varrimento faria o mesmo pixel
//! sair diferente conforme o **recorte** em que ele calhou. E HR-5 (determinismo) proíbe RNG por
//! quadro: a matriz de Bayer não é aleatória nenhuma — é uma função pura de `(x, y)`.
//!
//! # A amplitude é MEDIDA, e é o que torna a ida-e-volta exata
//!
//! O dither move um valor no máximo meio passo — é a definição. Mas o valor que chega aqui **já não
//! está exatamente na grelha**: um byte que subiu a 16 bits e volta atravessa a mantissa de 11 bits
//! do meio-float e volta ligeiramente ao lado. Se o dither usasse os 0,5 passos inteiros, essa
//! folga somava-se a ele e um cinzento chapado de 8 bits **mudaria de byte**. Numa engine de sprites
//! isso não é ruído: é arte por pixels estragada.
//!
//! Então a amplitude é *meio passo menos a deriva medida*, e a deriva foi medida sobre os 256 bytes:
//!
//! | canal | deriva máxima | onde |
//! |---|---|---|
//! | cor (atravessa a curva sRGB) | 0,037231 LSB | byte 192 |
//! | alfa (escala direta) | **0,062012 LSB** | byte 239 |
//!
//! ⚠️ **É o ALFA que manda**, e isso é contra-intuitivo: ele não atravessa curva nenhuma, e por isso
//! parece o caminho seguro. Mas justamente por ser uma escala direta, o erro relativo do meio-float
//! (2⁻¹²) aparece **inteiro** em LSB no topo da faixa (255 × 2,44e-4 ≈ 0,062), enquanto na cor a
//! derivada da curva sRGB o **encolhe**. *O canal sem curva é o que tem menos folga.*
//!
//! O gate [`tests::the_round_trip_stays_exact_under_every_dither_cell`] varre os 256 bytes × as 64
//! células da matriz e prova que nenhum se mexe — 16 384 casos, não um argumento.

use crate::srgb::linear_to_srgb_byte_biased;

/// **A deriva MEDIDA do caminho 8 → 16 → 8**, em passos de 8 bits (a tabela está no cabeçalho).
///
/// ⚠️ Este número é o do **alfa**, que é o pior dos dois. Quem mudar a representação de 16 bits
/// (outro formato de meio-float, unorm16, f32) tem de o **remedir** — ele descreve a precisão da
/// representação, não uma preferência.
pub const HALF_ROUND_TRIP_DRIFT_LSB: f32 = 0.062_012;

/// **O intervalo que o dither varre**, em passos de 8 bits: um passo inteiro menos a deriva de cada
/// lado. Derivado, nunca escolhido.
///
/// ⚠️ **É `pub` para a SONDA, não para um segundo motor.** A descida do ecrã (o passe de tonemap)
/// **não** leva dither: foi construída, medida e recusada — a folga que o hardware deixa lá é de
/// `0,0283` LSB, 7% desta amplitude. A sonda que mediu isso
/// (`crates/ph2d-render/tests/tonemap_descent_gpu.rs`) compara-se com este número, e é o único
/// consumidor de fora.
///
/// ⛔ *Não* reconstrua o dither na GPU a partir daqui — o mecanismo está no cabeçalho de
/// `crates/ph2d-render/src/shaders/tonemap.wgsl`, e a resposta curta é que a tabela sRGB do hardware
/// não é a curva ideal, logo a folga é propriedade **da placa** e não cabe nesta constante.
pub const DITHER_SPAN_LSB: f32 = 1.0 - 2.0 * HALF_ROUND_TRIP_DRIFT_LSB;

/// O lado da matriz de Bayer.
pub const BAYER_SIDE: u32 = 8;

/// Quantos níveis distintos a matriz oferece.
const BAYER_LEVELS: f32 = (BAYER_SIDE * BAYER_SIDE) as f32;

/// **A matriz de Bayer 8×8, DERIVADA da recorrência que a define** — nunca digitada.
///
/// `M₁ = [0]` e cada duplicação é
///
/// ```text
///   M₂ₙ = ⎡ 4·Mₙ + 0   4·Mₙ + 2 ⎤
///         ⎣ 4·Mₙ + 3   4·Mₙ + 1 ⎦
/// ```
///
/// ⚠️ Escrever os 64 números à mão passaria despercebido se um deles estivesse trocado: a matriz
/// continuaria a **parecer** um dither, e só um degradê muito limpo mostraria a falha. Derivá-la faz
/// o compilador ser o revisor — e [`tests::the_matrix_is_a_permutation_of_every_level`] prova que os
/// 64 níveis aparecem exatamente uma vez cada.
pub const BAYER_8X8: [u8; 64] = bayer_8x8();

const fn bayer_8x8() -> [u8; 64] {
    let mut m = [0u8; 64];
    let mut size = 1usize;
    while size < BAYER_SIDE as usize {
        let mut y = 0usize;
        while y < size {
            let mut x = 0usize;
            while x < size {
                // O valor antigo tem de ser lido ANTES de qualquer escrita: os quatro quadrantes
                // derivam todos dele, e o de cima-esquerda escreve por cima da própria origem.
                let v = m[y * BAYER_SIDE as usize + x] * 4;
                m[y * BAYER_SIDE as usize + x] = v;
                m[y * BAYER_SIDE as usize + x + size] = v + 2;
                m[(y + size) * BAYER_SIDE as usize + x] = v + 3;
                m[(y + size) * BAYER_SIDE as usize + x + size] = v + 1;
                x += 1;
            }
            y += 1;
        }
        size *= 2;
    }
    m
}

/// **O viés do dither para um pixel**, em passos de 8 bits, no intervalo `±0,4312`.
///
/// A matriz é ladrilhada sobre a imagem, por isso a função é total em `(x, y)`.
///
/// ⚠️ **O `+ 0.5` no numerador é o que torna o intervalo simétrico e nunca exatamente `−0,5`.** Sem
/// ele o nível 0 daria o extremo inferior fechado, e um valor exatamente na fronteira desceria um
/// byte — que é precisamente o pixel que este módulo promete não mexer.
#[must_use]
pub fn dither_offset_lsb(x: u32, y: u32) -> f32 {
    let cell = BAYER_8X8[((y % BAYER_SIDE) * BAYER_SIDE + (x % BAYER_SIDE)) as usize];
    let unit = (f32::from(cell) + 0.5) / BAYER_LEVELS;
    (unit - 0.5) * DITHER_SPAN_LSB
}

/// **`Rgba16` linear → `Rgba8` sRGB, com dither ordenado.** A gémea de
/// [`crate::rgba16_to_rgba8`], para quando a descida é um ato **deliberado e destrutivo** do autor.
///
/// ⚠️ **As duas existem de propósito, e a escolha é do sítio que chama.** Ler pixels para inspecionar,
/// gravar ou reenviar tem de ser **fiel** — um `read` que devolvesse valores diferentes dos que
/// guardou não é um read. Converter *a sprite* para 8 bits para sempre, esse, é onde as faixas
/// nascem e onde o dither pertence.
///
/// # O viés é o MESMO nos quatro canais
///
/// Um cinzento neutro tem R = G = B; um viés por canal separá-los-ia e a neutralidade viraria uma
/// franja de cor. Com um viés por **pixel**, os três canais movem-se juntos e o cinzento continua
/// cinzento — o dither passa a ser de luminância, que é onde a banda mora.
///
/// # Panics
///
/// Se `width` for zero. Uma imagem sem largura não tem coordenadas de pixel, e devolver o vetor
/// vazio esconderia a chamada errada até alguém reparar na sprite em branco.
#[must_use]
pub fn rgba16_to_rgba8_dithered(halves: &[u16], width: u32) -> Vec<u8> {
    assert!(width > 0, "rgba16_to_rgba8_dithered: width = 0");
    halves
        .as_chunks::<4>()
        .0
        .iter()
        .enumerate()
        .flat_map(|(i, px)| {
            let i = i as u32;
            let bias = dither_offset_lsb(i % width, i / width);
            let alpha = (crate::half_to_f32(px[3]).clamp(0.0, 1.0) * 255.0 + 0.5 + bias)
                .clamp(0.0, 255.0) as u8;
            [
                linear_to_srgb_byte_biased(crate::half_to_f32(px[0]), bias),
                linear_to_srgb_byte_biased(crate::half_to_f32(px[1]), bias),
                linear_to_srgb_byte_biased(crate::half_to_f32(px[2]), bias),
                alpha,
            ]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{rgba8_to_rgba16, rgba16_to_rgba8};

    /// **Os 64 níveis, uma vez cada.** Uma matriz com um nível repetido continua a dither*ar* — só
    /// que com um degrau que nunca é cruzado, e a faixa que ele deixa parece um defeito da imagem.
    #[test]
    fn the_matrix_is_a_permutation_of_every_level() {
        let mut seen = BAYER_8X8;
        seen.sort_unstable();
        let expected: Vec<u8> = (0..64).collect();
        assert_eq!(seen.as_slice(), expected.as_slice());
    }

    /// **A recorrência, recalculada aqui e comparada com a constante.**
    ///
    /// ⚠️ O `const fn` acima é a implementação; este teste é a *definição* escrita outra vez, de
    /// forma independente (aloca, indexa por linhas, não é `const`). Se as duas concordarem, o
    /// erro teria de estar nas duas ao mesmo tempo.
    #[test]
    fn the_constant_matches_the_recurrence_that_defines_it() {
        let mut m: Vec<Vec<u8>> = vec![vec![0]];
        while m.len() < BAYER_SIDE as usize {
            let n = m.len();
            let mut next = vec![vec![0u8; n * 2]; n * 2];
            for (y, row) in m.iter().enumerate() {
                for (x, &cell) in row.iter().enumerate() {
                    let v = cell * 4;
                    next[y][x] = v;
                    next[y][x + n] = v + 2;
                    next[y + n][x] = v + 3;
                    next[y + n][x + n] = v + 1;
                }
            }
            m = next;
        }
        let flat: Vec<u8> = m.into_iter().flatten().collect();
        assert_eq!(flat.as_slice(), BAYER_8X8.as_slice());
    }

    /// **A amplitude é simétrica e cabe dentro do que a deriva deixa.**
    ///
    /// ⚠️ O gate exaustivo abaixo prova a consequência; este prova a *causa*, e por isso nomeia o
    /// número. Sem ele, alguém que subisse o `DITHER_SPAN_LSB` veria só «um teste de 16 384 casos
    /// ficou vermelho» e não saberia qual constante o tinha partido.
    #[test]
    fn the_amplitude_leaves_room_for_the_measured_drift() {
        let mut lo = f32::MAX;
        let mut hi = f32::MIN;
        for y in 0..BAYER_SIDE {
            for x in 0..BAYER_SIDE {
                let d = dither_offset_lsb(x, y);
                lo = lo.min(d);
                hi = hi.max(d);
            }
        }
        assert!(
            (lo + hi).abs() < 1e-6,
            "o dither devia ser simetrico, e vai de {lo} a {hi}"
        );
        let headroom = 0.5 - HALF_ROUND_TRIP_DRIFT_LSB;
        assert!(
            hi < headroom,
            "amplitude {hi} nao cabe nos {headroom} que a deriva medida deixa — um byte exato \
             mudaria de valor"
        );
    }

    /// **A LEI: `8 → 16 → 8` continua exata, célula a célula.** 256 bytes × 64 células.
    ///
    /// ⚠️ Esta é a promessa que separa este dither de ruído: quem clicou `RGBA16` e voltou a clicar
    /// `RGBA8` recebe **os seus bytes**, não uma aproximação. Se algum dia esta varredura ficar
    /// vermelha, o que mudou foi a representação de 16 bits — e o
    /// [`HALF_ROUND_TRIP_DRIFT_LSB`] tem de ser remedido, nunca afrouxado.
    #[test]
    fn the_round_trip_stays_exact_under_every_dither_cell() {
        // Uma linha com os 256 bytes em cada canal, repetida ao longo de Y para varrer as 8 linhas
        // da matriz; a largura 256 garante que cada byte vê as 8 colunas ao longo de X.
        let width = 256u32;
        let original: Vec<u8> = (0..BAYER_SIDE)
            .flat_map(|_| (0..=255u8).flat_map(|b| [b, b, b, b]))
            .collect();
        let back = rgba16_to_rgba8_dithered(&rgba8_to_rgba16(&original), width);
        assert_eq!(back.len(), original.len());
        let mut broken = Vec::new();
        for (i, (a, b)) in original.iter().zip(back.iter()).enumerate() {
            if a != b {
                let px = (i / 4) as u32;
                broken.push(format!(
                    "  byte {a} -> {b} (canal {}, celula de Bayer x={} y={})",
                    i % 4,
                    px % width % BAYER_SIDE,
                    px / width % BAYER_SIDE
                ));
            }
        }
        assert!(
            broken.is_empty(),
            "o dither moveu {} valores que ja' estavam na grelha de 8 bits:\n{}",
            broken.len(),
            broken
                .iter()
                .take(12)
                .cloned()
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    /// ⚠️ **Controle positivo: o dither TEM de mudar alguma coisa.** Sem isto, uma amplitude posta a
    /// zero por engano deixaria o gate acima verde e o produto sem cura nenhuma — o modo de falha
    /// mais provável deste módulo é ele parar de fazer efeito em silêncio.
    ///
    /// A fixture é um degradê **entre dois bytes vizinhos**: em 16 bits ele tem valores a meio
    /// caminho, e é exatamente onde a quantização sem dither faz a linha aparecer.
    #[test]
    fn a_value_between_two_bytes_becomes_a_mix_of_both() {
        // O meio exato entre sRGB 100 e 101, em linear.
        let lo = crate::srgb::srgb_to_linear_byte(100);
        let hi = crate::srgb::srgb_to_linear_byte(101);
        let mid = (lo + hi) * 0.5;
        let half = crate::f32_to_half(mid);
        let width = BAYER_SIDE;
        let halves: Vec<u16> = (0..width * BAYER_SIDE)
            .flat_map(|_| [half, half, half, crate::f32_to_half(1.0)])
            .collect();

        let plain = rgba16_to_rgba8(&halves);
        let dithered = rgba16_to_rgba8_dithered(&halves, width);

        let flat = plain.as_chunks::<4>().0.iter().all(|p| p[0] == plain[0]);
        assert!(flat, "sem dither, um valor uniforme tem de dar UM byte so'");

        let mut lows = 0usize;
        let mut highs = 0usize;
        for p in dithered.as_chunks::<4>().0 {
            match p[0] {
                100 => lows += 1,
                101 => highs += 1,
                other => panic!(
                    "o dither saiu da vizinhanca: byte {other} para um valor entre 100 e 101"
                ),
            }
        }
        assert!(
            lows > 0 && highs > 0,
            "o dither devia misturar os dois bytes vizinhos, e deu {lows} de 100 e {highs} de 101"
        );
        // Metade da matriz de cada lado: o valor está no meio exato, e o dither é simétrico.
        assert_eq!(
            lows, highs,
            "um valor no meio EXATO devia dar metade de cada byte"
        );
    }

    /// **Um cinzento continua cinzento.** O viés é por pixel, nunca por canal — se o fosse, R, G e B
    /// separavam-se e a neutralidade virava franja de cor.
    #[test]
    fn a_neutral_grey_stays_neutral() {
        let lo = crate::srgb::srgb_to_linear_byte(140);
        let hi = crate::srgb::srgb_to_linear_byte(141);
        let half = crate::f32_to_half((lo + hi) * 0.5);
        let width = BAYER_SIDE;
        let halves: Vec<u16> = (0..width * BAYER_SIDE)
            .flat_map(|_| [half, half, half, crate::f32_to_half(1.0)])
            .collect();
        for (i, p) in rgba16_to_rgba8_dithered(&halves, width)
            .as_chunks::<4>()
            .0
            .iter()
            .enumerate()
        {
            assert!(
                p[0] == p[1] && p[1] == p[2],
                "pixel {i} saiu {:?} — os canais separaram-se e o cinzento ganhou cor",
                &p[..3]
            );
        }
    }

    /// **O padrão é função da POSIÇÃO, e por isso um recorte sai igual ao pedaço correspondente do
    /// inteiro.** É a propriedade que a difusão de erro não tem, e a razão de a matriz ser ordenada.
    #[test]
    fn the_pattern_follows_the_pixel_and_not_the_scan() {
        for y in 0..BAYER_SIDE * 3 {
            for x in 0..BAYER_SIDE * 3 {
                assert_eq!(
                    dither_offset_lsb(x, y),
                    dither_offset_lsb(x + BAYER_SIDE, y + BAYER_SIDE),
                    "a matriz devia ladrilhar em ({x}, {y})"
                );
            }
        }
    }

    /// ⚠️ **A porta FIEL não pode ter ganho dither por arrasto.** As duas funções são gémeas e vivem
    /// a um `_dithered` de distância; um dia alguém vai «uniformizá-las».
    #[test]
    fn the_faithful_door_is_still_faithful() {
        let lo = crate::srgb::srgb_to_linear_byte(60);
        let hi = crate::srgb::srgb_to_linear_byte(61);
        let half = crate::f32_to_half((lo + hi) * 0.5);
        let halves: Vec<u16> = (0..64).flat_map(|_| [half, half, half, half]).collect();
        let plain = rgba16_to_rgba8(&halves);
        assert!(
            plain.as_chunks::<4>().0.iter().all(|p| p == &plain[..4]),
            "`rgba16_to_rgba8` tem de dar o MESMO byte para o mesmo valor, sempre"
        );
    }
}
