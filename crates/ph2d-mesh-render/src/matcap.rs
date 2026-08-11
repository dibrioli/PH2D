//! **OS MATCAPS — a tabela, os pixels e a lei de espaço.**
//!
//! Um matcap é sombreamento que é função APENAS da normal em espaço de vista: a
//! luz viaja com a câmera, então orbitar não muda a leitura da forma. É por isso
//! que ele não é substituível pelo rig — o rig é do DOCUMENTO (a mesma lâmpada
//! acende a tinta ao lado), o matcap é do OLHO.
//!
//! # Isto SUBSTITUIU o matcap analítico, e a premissa que o defendia caiu
//!
//! Até 2026-08-10 o matcap era um punhado de cores e expoentes avaliados no
//! WGSL, e o doc daquele bloco dizia por quê: *"seria o caminho certo se houvesse
//! matcaps AUTORADOS para carregar. Não há: seriam assets novos, com licença,
//! num repo que não os tem."* A premissa era sobre LICENÇA, e caiu por medição —
//! os do Blender trazem um `license.txt` dizendo **CC0**. Com os assets em mãos,
//! o resto do argumento (*"uma textura sintetizada seria a MESMA função assada"*)
//! deixou de valer: estas imagens **não são** aquela função.
//!
//! # NADA é quantizado abaixo da precisão da FONTE
//!
//! ⚠️ **O primeiro corte desta wave guardava tudo em 8 bits, e era uma perda que
//! eu não tinha marcado como tal.** Eu medi *"algum valor passa de 1,0?"* (a
//! faixa HDR) e concluí que 8 bits bastavam — mas *"cabe em `[0,1]`"* e *"8 bits
//! chegam"* são perguntas diferentes. Medido de volta em LINEAR, que é o que o
//! shader recebe, a quantização para 8 bits erra **~1 nível de 255** (0,93 no
//! `Basic Bright`, 1,09 no `Basic Side`) contra **0,004** em 16 bits — 259×. Um
//! matcap é um gradiente liso sobre uma esfera, que é o caso clássico de banda
//! visível.
//!
//! Hoje cada fonte é guardada na precisão em que foi autorada, e a decodificação
//! entrega **meio-float linear** para uma textura `Rgba16Float`:
//!
//! | fonte | no disco | por quê |
//! |---|---|---|
//! | Blender | EXR RGB **meio-float**, ZIP | é a precisão do original, sem perda |
//! | SculptGL | PNG de 8 bits | o original é JPEG ⇒ isto é **bit-idêntico** |
//!
//! ⚠️ **Promover o PNG a float não acrescentaria informação nenhuma** — daria um
//! arquivo maior dizendo a mesma coisa. A conversão sRGB→linear acontece na
//! decodificação, pela porta do repo ([`ph2d_color::srgb::srgb_to_linear_byte`]), e
//! não por uma curva escrita aqui.
//!
//! ⚠️ **E os `.exr` do Blender NÃO são os originais:** o nosso decoder recusa
//! aqueles por **dois** motivos escritos no doc dele — *"custom channel layouts
//! beyond RGBA"* (eles têm `diffuse.*` e `specular.*`) e *"tile-based + DWA/DWB
//! compression"*. Nenhum dos dois é sobre precisão, e é por isso que re-embalar
//! (somar as camadas, renomear os canais, trocar DWAA por ZIP) devolve a mesma
//! informação por uma porta que já temos. O script está em
//! `docs/3D/ferramentas/cook_matcaps.sh`.
//!
//! # A tabela é UMA, e é isso que torna um nome sem pixels inexprimível
//!
//! O nome, a procedência, o lado e a imagem moram no MESMO registro. Uma linha
//! nova **não pode** nascer sem imagem: o `include_bytes!` recusa em tempo de
//! COMPILAÇÃO se o arquivo não existir.
//!
//! # O espaço, que é a única coisa que um render revela
//!
//! A coordenada é `uv = n.xy * 0.5 + 0.5` sobre o normal que o `canvas_normal`
//! devolve — o de TELA, com **`y` crescendo para BAIXO**. A linha 0 de uma
//! textura também é o topo, então os dois eixos já concordam e **não há flip**.
//!
//! ⚠️ **Os cantos da imagem NUNCA são amostrados**, e é geometria: `|n.xy| <= 1`
//! num vetor unitário, então o disco inscrito é todo o domínio. É por isso que
//! estes arquivos podem ter fundos diferentes entre si sem que isso signifique
//! nada — e por isso um matcap de 749² e um de 512² convivem sem se falarem.

use ph2d_imageio::{ImageImporter, ImportOpts};

/// **COMO os pixels de um matcap estão guardados no disco.**
///
/// ⚠️ Duas variantes porque as duas FONTES são autoradas em precisões
/// diferentes, e a lei é não quantizar abaixo do original. Um formato só
/// obrigaria a escolher entre inflar o JPEG ou truncar o meio-float.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Encoding {
    /// EXR RGB meio-float, LINEAR — os do Blender.
    ExrHalfLinear,
    /// PNG de 8 bits, **sRGB** — os do SculptGL, bit-idênticos ao JPEG de origem.
    PngSrgb8,
}

/// **UM matcap: o nome que o artista lê e os pixels que o acendem.**
///
/// ⚠️ Os bytes são o arquivo, não a imagem decodificada — decodificar os dez no
/// boot custaria dezenas de MB para mostrar UM. Quem decodifica é o
/// [`crate::MeshRenderer`], sob demanda, e só o escolhido.
pub struct Matcap {
    /// O que o chip do painel escreve.
    pub name: &'static str,
    /// De onde ele veio e sob que licença — ver `assets/matcaps/LICENSES.md`.
    pub credit: Credit,
    /// O lado da imagem, em texels.
    ///
    /// ⚠️ **Por-matcap, e não uma constante do módulo.** Os do Blender são 512²
    /// e os do SculptGL 749², e reamostrar para um lado comum seria a perda que
    /// esta wave existe para não pagar. Quem lida com isso é o
    /// `MeshRenderer::ensure_matcap`, que recria a textura quando o lado muda.
    pub side: u32,
    /// Como os bytes estão guardados.
    pub encoding: Encoding,
    /// O arquivo embarcado.
    pub bytes: &'static [u8],
}

/// A procedência de um matcap — ver `assets/matcaps/LICENSES.md`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Credit {
    /// Blender, `release/datafiles/studiolights/matcap/` — **CC0 / domínio
    /// público**, pelo `license.txt` do próprio diretório.
    Blender,
    /// **HazardousArts**, publicados no DeviantArt em 2014 e distribuídos pelo
    /// SculptGL (MIT) desde então.
    ///
    /// ⚠️ **Ele NÃO se chama `SculptGl`, e a distinção é o ponto:** o SculptGL os
    /// redistribui sob a licença do repositório dele, mas o nome do arquivo
    /// credita um terceiro e o README dele atribui a terceiros apenas os
    /// *environments*. Os termos exatos do autor **não estão documentados** — o
    /// `LICENSES.md` diz isso com todas as letras em vez de escrever "MIT" e
    /// deixar a ambiguidade escondida num campo.
    HazardousArts,
}

/// **A TABELA.** A ordem é a que o painel pinta, e o índice `0` é o default do
/// app (ver [`crate::DEFAULT_MATCAP`]).
///
/// ⚠️ **`const` e não `static`, e a diferença é load-bearing:** é ela que deixa
/// [`MATCAP_NAMES`] ser **derivado** desta lista em tempo de compilação (não se
/// lê um `static` em contexto `const`). Sem isso os nomes voltariam a ser uma
/// segunda lista que precisa concordar com esta por contagem.
pub const MATCAPS: [Matcap; 10] = [
    // ⚠️ **O default do app, e ele vem PRIMEIRO de propósito:** o índice 0 é o
    // que `Shade::default()` arma, então a ordem da lista e a escolha do default
    // são o MESMO fato em vez de dois números que precisam concordar.
    Matcap {
        name: "Skin Haz 2",
        credit: Credit::HazardousArts,
        side: 749,
        encoding: Encoding::PngSrgb8,
        bytes: include_bytes!("../assets/matcaps/skinHazardousarts2.png"),
    },
    Matcap {
        name: "Skin Haz",
        credit: Credit::HazardousArts,
        side: 749,
        encoding: Encoding::PngSrgb8,
        bytes: include_bytes!("../assets/matcaps/skinHazardousarts.png"),
    },
    Matcap {
        name: "Basic Bright",
        credit: Credit::Blender,
        side: 512,
        encoding: Encoding::ExrHalfLinear,
        bytes: include_bytes!("../assets/matcaps/basic_bright.exr"),
    },
    Matcap {
        name: "Basic Dark",
        credit: Credit::Blender,
        side: 512,
        encoding: Encoding::ExrHalfLinear,
        bytes: include_bytes!("../assets/matcaps/basic_dark.exr"),
    },
    Matcap {
        name: "Basic Gray",
        credit: Credit::Blender,
        side: 512,
        encoding: Encoding::ExrHalfLinear,
        bytes: include_bytes!("../assets/matcaps/basic_grey.exr"),
    },
    Matcap {
        name: "Basic Side",
        credit: Credit::Blender,
        side: 512,
        encoding: Encoding::ExrHalfLinear,
        bytes: include_bytes!("../assets/matcaps/basic_side.exr"),
    },
    Matcap {
        name: "Clay Brown",
        credit: Credit::Blender,
        side: 512,
        encoding: Encoding::ExrHalfLinear,
        bytes: include_bytes!("../assets/matcaps/clay_brown.exr"),
    },
    Matcap {
        name: "Clay Green",
        credit: Credit::Blender,
        side: 512,
        encoding: Encoding::ExrHalfLinear,
        bytes: include_bytes!("../assets/matcaps/clay_green.exr"),
    },
    Matcap {
        name: "Clay Warm",
        credit: Credit::Blender,
        side: 512,
        encoding: Encoding::ExrHalfLinear,
        bytes: include_bytes!("../assets/matcaps/clay_warm.exr"),
    },
    Matcap {
        name: "Red Wax",
        credit: Credit::Blender,
        side: 512,
        encoding: Encoding::ExrHalfLinear,
        bytes: include_bytes!("../assets/matcaps/red_wax.exr"),
    },
];

/// **OS NOMES, na ordem da tabela** — o que o painel pinta nos chips.
///
/// ⚠️ **DERIVADO de [`MATCAPS`], nunca escrito à mão.** É isto que torna
/// inexprimível um nome sem imagem (e uma imagem sem nome): a linha nova entra
/// numa lista só, e o `include_bytes!` recusa em tempo de compilação se o
/// arquivo não existir.
pub const MATCAP_NAMES: [&str; MATCAPS.len()] = {
    let mut out = [""; MATCAPS.len()];
    let mut i = 0;
    while i < MATCAPS.len() {
        out[i] = MATCAPS[i].name;
        i += 1;
    }
    out
};

/// **Os pixels do matcap `id`, prontos para `write_texture`** — RGBA de
/// **meio-float LINEAR**, `side²`.
///
/// ⚠️ **Meio-float e não 8 bits, e a diferença está medida** (ver o doc do
/// módulo): 8 bits erram ~1 nível de 255 de volta em linear, e um matcap é
/// exatamente o gradiente liso onde 1 nível vira banda.
///
/// ⚠️ **O erro é um PÂNICO, e é deliberado.** Estes dez arquivos são
/// `include_bytes!` — não vêm do disco do usuário, não têm caminho, e não podem
/// faltar num binário que compilou. Um `Result` obrigaria todo chamador a
/// inventar comportamento para um caso que não existe, e o mais provável seria
/// *"desenha sem matcap"*: a feature falhando em silêncio.
///
/// ⚠️ **Um índice fora da tabela é CLAMPADO** — a mesma política do
/// [`crate::ShadeRaw::pack`]: a entrada vem de uma escolha de UI que atravessou
/// um `u8`, não de uma corrupção.
#[must_use]
pub fn decode(id: usize) -> Vec<u8> {
    let m = &MATCAPS[id.min(MATCAPS.len() - 1)];
    let n = (m.side as usize) * (m.side as usize);
    let mut out: Vec<half::f16> = Vec::with_capacity(n * 4);

    match m.encoding {
        Encoding::ExrHalfLinear => {
            let img = ph2d_imageio_exr::ExrImporter
                .import(m.bytes, &ImportOpts::default())
                .unwrap_or_else(|e| panic!("o matcap `{}` é um EXR válido: {e}", m.name));
            let ph2d_imageio::DecodedImage::FlatHdr(buf) = img else {
                panic!("o matcap `{}` é uma imagem HDR plana", m.name)
            };
            assert_side(m, buf.width, buf.height);
            for p in &buf.pixels {
                // ⚠️ O alfa é forçado a 1: o EXR de origem trazia um canal de
                // alfa que vale 1 em toda parte (medido), e o disco amostrado
                // nunca alcança os cantos de qualquer forma.
                out.extend([
                    half::f16::from_f32(p.r()),
                    half::f16::from_f32(p.g()),
                    half::f16::from_f32(p.b()),
                    half::f16::ONE,
                ]);
            }
        }
        Encoding::PngSrgb8 => {
            let img = ph2d_imageio_png::PngImporter
                .import(m.bytes, &ImportOpts::default())
                .unwrap_or_else(|e| panic!("o matcap `{}` é um PNG válido: {e}", m.name));
            let ph2d_imageio::DecodedImage::Flat(buf) = img else {
                panic!("o matcap `{}` é uma imagem PLANA", m.name)
            };
            assert_side(m, buf.width, buf.height);
            for p in &buf.pixels {
                // ⚠️ **Pela porta do repo, e não por uma curva escrita aqui:** a
                // sRGB tem JOELHO, e um `x^2.2` escrito à mão erra justamente no
                // escuro — que é metade de um matcap de pele.
                out.extend([
                    half::f16::from_f32(ph2d_color::srgb::srgb_to_linear_byte(p.0[0])),
                    half::f16::from_f32(ph2d_color::srgb::srgb_to_linear_byte(p.0[1])),
                    half::f16::from_f32(ph2d_color::srgb::srgb_to_linear_byte(p.0[2])),
                    half::f16::ONE,
                ]);
            }
        }
    }

    // ⚠️ **`to_le_bytes` e não `bytemuck::cast_slice`:** o `half::f16` só é
    // `Pod` sob uma feature da `half` que este workspace não liga, e ligá-la
    // para poupar um `flat_map` seria mexer numa dep compartilhada por causa
    // desta função. O `wgpu` quer little-endian, que é o que isto escreve.
    out.iter().flat_map(|h| h.to_le_bytes()).collect()
}

fn assert_side(m: &Matcap, w: u32, h: u32) {
    assert_eq!(
        (w, h),
        (m.side, m.side),
        "o matcap `{}` mede {w}×{h} e a tabela diz {}²",
        m.name,
        m.side,
    );
}

#[cfg(test)]
#[path = "matcap_tests.rs"]
mod tests;
