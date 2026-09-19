//! ⭐⭐⭐ **OS BOTÕES DA CAMADA DE ESTILO, no painel** (`docs/Render3d/03`, a `W8`).
//!
//! # ⭐ A tabela é DERIVADA da arrumação, e não escrita ao lado dela
//!
//! O [`ph2d_field::Param::Style`] carrega a **posição na arrumação** do [`ph2d_style::wgsl::pack`],
//! e é isso que faz uma escrita ser uma linha: *desempacota, escreve a posição, empacota*. ⛔ Uma
//! numeração própria seria a segunda resposta à tabela do `pack`, e a que envelhece no dia em que
//! nascer um botão.
//!
//! ⇒ o que este ficheiro declara é só **quais posições são LINHAS** e o que cada uma se chama; os
//! valores, a ordem e o número de campos vêm todos de lá.
//!
//! # ⚠️ Porque as linhas só aparecem no modo RENDER
//!
//! O estilo é direcção de arte sobre um pipeline fisicamente honesto — no matcap ele **não corre**
//! (ver o censo do `estilo_tests`), e *uma affordance que não pode ser honrada é pior do que
//! nenhuma*, que é a lei que o [`ph2d_panel_model3d::ParamRow::inert`] já escreve para as fileiras
//! do material.

use ph2d_field::{Bound, Param};
use ph2d_style::{Style, wgsl};

/// Uma linha da secção do estilo — **a posição na arrumação**, o nome, e de que natureza ela é.
struct Linha {
    /// A posição no [`ph2d_style::wgsl::pack`]. Numa COR, ela é a do primeiro canal.
    slot: u8,
    /// A chave i18n do rótulo. ⚠️ Uma **chave**, nunca um rótulo pronto (HR-15).
    key: &'static str,
    /// `None` ⇒ é uma cor (três canais consecutivos, amostra). `Some(teto)` ⇒ é um número.
    teto: Option<f32>,
}

/// ⭐⭐⭐ **AS DEZ LINHAS** — cinco cores e cinco números, na ordem em que o artista as lê.
///
/// ⚠️ **Os tectos NÃO são escolhidos por conforto** (`CLAUDE.md` §0.0), e cada um diz de que é:
///
/// | linha | tecto | de que recurso ele é |
/// |---|---|---|
/// | a força do contorno | `4` | é uma RADIÂNCIA acrescentada, e acima de `4` ela satura a vista antes de o expoente ter forma |
/// | a largura do contorno | [`ph2d_style::Rim::MAX_WIDTH`] | **a lei declara-o**, e a razão está lá: acima dele a banda é mais fina que um pixel a `1080p` |
/// | a nitidez da curvatura | `8` | é um GANHO sobre `H·raio`; a `8` uma zona `8×` mais curva que a peça já satura o corte em `±1`, e acima disso o botão deixa de mover a imagem |
/// | o pivô das zonas | `4` | é uma LUMINÂNCIA de cena, e `4` é `~4,5` paragens acima do cinzento médio — o topo do que uma cena exposta a `0` entrega |
/// | a saturação da indirecta | `4` | é um multiplicador de crominância; acima de `4` a parcela indirecta satura o gamut em toda cena medida |
///
/// ⏳ **Os três `4` são tectos de PRODUTO por medir** — eles nomeiam o recurso e a wave que os mede
/// é a `W9` (a avaliação, que o dono pôs ao fim da fila). *Um tecto que diz de que é e ainda não tem
/// tabela é uma dívida nomeada; um que só diz «por segurança» é um palpite.*
const LINHAS: [Linha; 10] = [
    Linha {
        slot: 0,
        key: "panel.model3d.style.rim_color",
        teto: None,
    },
    Linha {
        slot: 3,
        key: "panel.model3d.style.rim_strength",
        teto: Some(4.0),
    },
    Linha {
        slot: 15,
        key: "panel.model3d.style.rim_width",
        teto: Some(ph2d_style::Rim::MAX_WIDTH),
    },
    Linha {
        slot: 4,
        key: "panel.model3d.style.convex",
        teto: None,
    },
    Linha {
        slot: 8,
        key: "panel.model3d.style.concave",
        teto: None,
    },
    Linha {
        slot: 7,
        key: "panel.model3d.style.sharpness",
        teto: Some(8.0),
    },
    Linha {
        slot: 12,
        key: "panel.model3d.style.shadow_tint",
        teto: None,
    },
    Linha {
        slot: 16,
        key: "panel.model3d.style.highlight_tint",
        teto: None,
    },
    Linha {
        slot: 11,
        key: "panel.model3d.style.pivot",
        teto: Some(4.0),
    },
    Linha {
        slot: 19,
        key: "panel.model3d.style.saturation",
        teto: Some(4.0),
    },
];

/// A chave i18n do cabeçalho da secção.
pub(crate) const SECCAO: &str = "panel.model3d.section.style";

/// ⭐⭐⭐ **AS LINHAS DO ESTILO** — vazio fora do modo *Render*, e a ausência é a lei.
///
/// ⚠️ **`entity` é `0` e ninguém o lê** — ver [`ph2d_field::Param::Style`]. O dreno decide o sujeito
/// pela FAMÍLIA, e o sujeito do estilo é a cena.
#[must_use]
pub fn rows(style: Style, render: bool) -> Vec<ph2d_panel_model3d::ParamRow> {
    if !render {
        return Vec::new();
    }
    let v = wgsl::pack(&style);
    LINHAS
        .iter()
        .enumerate()
        .map(|(i, l)| ph2d_panel_model3d::ParamRow {
            entity: 0,
            param: Param::Style(l.slot),
            key: l.key,
            value: v[l.slot as usize],
            lo: 0.0,
            // ⚠️ **`Soft` e não `Hard`**: nenhum destes tectos é uma parede do documento — a lei
            // aceita qualquer número finito, e o que eles limitam é o GESTO. Ver a tabela do
            // [`LINHAS`] para de que recurso é cada um.
            bound: Bound::Soft(l.teto.unwrap_or(1.0)),
            inert: None,
            integral: false,
            // ⛔ Nenhuma destas é uma ESCOLHA — são cinco cores e cinco números contínuos.
            choices: &[],
            section: (i == 0).then_some(SECCAO),
            // ⭐ **A amostra é o que faz uma cor ser UMA linha** e não três — ver
            // [`ph2d_panel_model3d::ParamRow::swatch`].
            swatch: l.teto.is_none().then(|| {
                crate::materials::colour_srgb8([
                    v[l.slot as usize],
                    v[l.slot as usize + 1],
                    v[l.slot as usize + 2],
                ])
            }),
            subject: None,
        })
        .collect()
}

/// ⭐⭐⭐ **A ESCRITA DE UM NÚMERO** — desempacota, escreve a posição, empacota.
///
/// ⚠️ **É uma linha porque o índice é o da arrumação**, e é esse o ponto de o [`Param::Style`] o
/// carregar. ⛔ Fora do alcance devolve o estilo intacto: *uma recusa é informação, e o retrato
/// publicado a seguir devolve o controlo ao valor que ficou* — a mesma lei do dreno das dimensões.
#[must_use]
pub fn with_number(style: Style, slot: u8, value: f32) -> Style {
    let mut v = wgsl::pack(&style);
    let Some(alvo) = v.get_mut(slot as usize) else {
        return style;
    };
    *alvo = value;
    wgsl::unpack(&v)
}

/// ⭐ **A escrita de uma COR** — os três canais no mesmo passo, como a do material.
///
/// ⚠️ **A âncora é o primeiro canal e os outros dois saem dela**, pela porta
/// [`ph2d_field::Param::colour_channels`] — a mesma que o material usa. *Um `+1`/`+2` escrito aqui
/// seria a segunda cópia de uma lei que já tem três leitores.*
#[must_use]
pub fn with_colour(style: Style, anchor: u8, srgb: [u8; 3]) -> Style {
    let cor = crate::materials::colour_from_srgb8(srgb);
    let Some(canais) = Param::Style(anchor).colour_channels() else {
        return style;
    };
    let mut fora = style;
    for (k, c) in canais.into_iter().enumerate() {
        if let Param::Style(slot) = c {
            fora = with_number(fora, slot, cor[k]);
        }
    }
    fora
}

#[cfg(test)]
#[path = "estilo_painel_tests.rs"]
mod tests;
