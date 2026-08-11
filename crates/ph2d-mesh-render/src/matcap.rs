//! **OS MATCAPS — a tabela, os pixels e a lei de espaço.**
//!
//! Um matcap é sombreamento que é função APENAS da normal em espaço de vista: a
//! luz viaja com a câmera, então orbitar não muda a leitura da forma. É por isso
//! que ele não é substituível pelo rig — o rig é do DOCUMENTO (a mesma lâmpada
//! acende a tinta ao lado), o matcap é do OLHO.
//!
//! # Isto SUBSTITUIU o matcap analítico, e a substituição foi pedida
//!
//! Até 2026-08-10 o matcap era um punhado de cores e expoentes avaliados no
//! WGSL, e o doc daquele bloco dizia por quê: *"a forma canônica é uma imagem de
//! esfera amostrada por `n.xy * 0.5 + 0.5`, e ela seria o caminho certo se
//! houvesse matcaps AUTORADOS para carregar. Não há: seriam assets novos, com
//! licença, num repo que não os tem."*
//!
//! ⚠️ **A premissa era sobre LICENÇA, e ela caiu por medição, não por opinião:**
//! os matcaps do Blender trazem um `license.txt` no próprio diretório dizendo
//! *"These matcap images are licensed as CC0 or public domain"*, e o do SculptGL
//! vem de um repositório MIT cujos créditos atribuem a terceiros apenas os
//! *environments*. Com os assets em mãos, o argumento que sobrava do bloco
//! antigo (*"uma textura sintetizada na CPU seria a MESMA função avaliada uma
//! vez por texel"*) deixou de valer: estas imagens **não são** aquela função —
//! elas são autoradas, e nenhuma quantidade de lâmpadas analíticas as alcança.
//!
//! # A tabela é UMA, e é isso que torna um nome sem pixels inexprimível
//!
//! O nome e a imagem moram no MESMO registro. A forma anterior — uma lista de
//! nomes aqui e os valores no shader — mantinha duas listas que tinham de
//! concordar por contagem, e o gate do shell existe justamente porque uma
//! terceira (os ids dos chips) podia divergir das duas. Uma linha nova aqui
//! **não pode** nascer sem imagem, e o `include_bytes!` recusa em tempo de
//! COMPILAÇÃO se o arquivo não existir.
//!
//! # O espaço, que é a única coisa que um render revela
//!
//! A coordenada é `uv = n.xy * 0.5 + 0.5` sobre o normal que o
//! `canvas_normal` devolve — o de TELA, com **`y` crescendo para BAIXO**. A
//! linha 0 de uma textura também é o topo, então os dois eixos já concordam e
//! **não há flip**: o realce que a imagem traz em cima aparece em cima. Com um
//! flip a escultura acenderia por baixo enquanto a tinta ao lado, sob a mesma
//! lâmpada, acenderia por cima.
//!
//! ⚠️ **Os cantos da imagem NUNCA são amostrados**, e é geometria: `|n.xy| <= 1`
//! para um vetor unitário, então o disco inscrito é todo o domínio. É por isso
//! que estes arquivos podem ter fundos diferentes entre si (o `basic_side` é
//! preto, o `studio` é cinza) sem que isso signifique nada.

use ph2d_imageio::{ImageImporter, ImportOpts};

/// **UM matcap: o nome que o artista lê e os pixels que o acendem.**
///
/// ⚠️ Os bytes são o PNG, não a imagem decodificada — decodificar os nove no
/// boot custaria 9 MB de RAM para mostrar UM. Quem decodifica é o
/// [`crate::MeshRenderer`], sob demanda, e só o escolhido.
pub struct Matcap {
    /// O que o chip do painel escreve.
    pub name: &'static str,
    /// De onde ele veio e sob que licença — ver `assets/matcaps/LICENSES.md`.
    pub credit: Credit,
    /// O PNG embarcado: 512×512, RGB, **já em sRGB**.
    pub png: &'static [u8],
}

/// A procedência de um matcap. Duas fontes, duas licenças, e as duas
/// redistribuíveis — ver `assets/matcaps/LICENSES.md`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Credit {
    /// Blender, `release/datafiles/studiolights/matcap/` — **CC0 / domínio
    /// público**, pelo `license.txt` do próprio diretório.
    Blender,
    /// SculptGL (Stéphane Ginier) — **MIT**.
    SculptGl,
}

/// O lado de uma imagem de matcap, em texels.
///
/// ⚠️ **É a resolução da FONTE, e não uma escolha a defender:** os oito do
/// Blender e o do SculptGL nascem todos em 512². Reduzir foi MEDIDO — a 256 o
/// erro médio é ínfimo (0,16 a 0,47 níveis de 255) mas o **máximo do
/// `Basic Side` é 74 níveis** e o do `Studio` 45, exatamente nos dois cujo
/// trabalho é um terminador duro e um realce apertado. Amaciar o que eles
/// existem para mostrar não é uma economia, é uma regressão de LOOK.
pub const MATCAP_SIDE: u32 = 512;

/// **A TABELA.** A ordem é a que o painel pinta, e o índice `0` é o default do
/// app (ver [`crate::DEFAULT_MATCAP`]).
///
/// ⚠️ **`const` e não `static`, e a diferença é load-bearing:** é ela que deixa
/// [`MATCAP_NAMES`] ser **derivado** desta lista em tempo de compilação (não se
/// lê um `static` em contexto `const`). Sem isso os nomes voltariam a ser uma
/// segunda lista que precisa concordar com esta por contagem — exatamente o que
/// esta wave removeu.
pub const MATCAPS: [Matcap; 9] = [
    // ⚠️ **O default do app, e ele vem PRIMEIRO de propósito:** o índice 0 é o
    // que `Shade::default()` arma, então a ordem da lista e a escolha do default
    // são o MESMO fato em vez de dois números que precisam concordar.
    Matcap {
        name: "Studio",
        credit: Credit::SculptGl,
        png: include_bytes!("../assets/matcaps/sculptgl_fv.png"),
    },
    Matcap {
        name: "Basic Bright",
        credit: Credit::Blender,
        png: include_bytes!("../assets/matcaps/basic_bright.png"),
    },
    Matcap {
        name: "Basic Dark",
        credit: Credit::Blender,
        png: include_bytes!("../assets/matcaps/basic_dark.png"),
    },
    Matcap {
        name: "Basic Gray",
        credit: Credit::Blender,
        png: include_bytes!("../assets/matcaps/basic_grey.png"),
    },
    Matcap {
        name: "Basic Side",
        credit: Credit::Blender,
        png: include_bytes!("../assets/matcaps/basic_side.png"),
    },
    Matcap {
        name: "Clay Brown",
        credit: Credit::Blender,
        png: include_bytes!("../assets/matcaps/clay_brown.png"),
    },
    Matcap {
        name: "Clay Green",
        credit: Credit::Blender,
        png: include_bytes!("../assets/matcaps/clay_green.png"),
    },
    Matcap {
        name: "Clay Warm",
        credit: Credit::Blender,
        png: include_bytes!("../assets/matcaps/clay_warm.png"),
    },
    Matcap {
        name: "Red Wax",
        credit: Credit::Blender,
        png: include_bytes!("../assets/matcaps/red_wax.png"),
    },
];

/// **OS NOMES, na ordem da tabela** — o que o painel pinta nos chips.
///
/// ⚠️ **DERIVADO de [`MATCAPS`], nunca escrito à mão.** É isto que torna
/// inexprimível um nome sem imagem (e uma imagem sem nome): a linha nova entra
/// numa lista só, e o `include_bytes!` recusa em tempo de compilação se o
/// arquivo não existir. O painel recebe uma fatia disto; a igualdade com a
/// contagem de chips é gateada no shell.
pub const MATCAP_NAMES: [&str; MATCAPS.len()] = {
    let mut out = [""; MATCAPS.len()];
    let mut i = 0;
    while i < MATCAPS.len() {
        out[i] = MATCAPS[i].name;
        i += 1;
    }
    out
};

/// **Os pixels do matcap `id`, prontos para `write_texture`** — RGBA de 8 bits,
/// `MATCAP_SIDE²`, em sRGB.
///
/// ⚠️ **O erro é um PÂNICO, e é deliberado.** Estes nove arquivos são
/// `include_bytes!` — eles não vêm do disco do usuário, não têm caminho, e não
/// podem faltar num binário que compilou. Um `Result` aqui obrigaria todo
/// chamador a inventar um comportamento para um caso que não existe, e o mais
/// provável seria *"desenha sem matcap"* — que é a feature falhando em silêncio.
/// Se isto panicar, o repositório está corrompido, e é isso que a mensagem diz.
///
/// ⚠️ **Um índice fora da tabela é CLAMPADO, não é pânico** — é a mesma política
/// do [`crate::shade::ShadeRaw::pack`], que já prende o índice no último: a
/// entrada vem de um `u32` que atravessou um uniform, e o chamador errado é uma
/// escolha de UI, não uma corrupção.
#[must_use]
pub fn decode(id: usize) -> Vec<u8> {
    let m = &MATCAPS[id.min(MATCAPS.len() - 1)];
    let img = ph2d_imageio_png::PngImporter
        .import(m.png, &ImportOpts::default())
        .unwrap_or_else(|e| panic!("o matcap `{}` está embarcado e é um PNG válido: {e}", m.name));
    let ph2d_imageio::DecodedImage::Flat(buf) = img else {
        panic!("o matcap `{}` é uma imagem PLANA, não uma pilha de camadas", m.name)
    };
    assert_eq!(
        (buf.width, buf.height),
        (MATCAP_SIDE, MATCAP_SIDE),
        "o matcap `{}` mede {}×{} e a textura é {MATCAP_SIDE}²",
        m.name,
        buf.width,
        buf.height,
    );
    buf.pixels.iter().flat_map(|p| p.0).collect()
}

#[cfg(test)]
#[path = "matcap_tests.rs"]
mod tests;
