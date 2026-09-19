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
    /// ⭐⭐⭐ **QUANDO É QUE ESTA FILEIRA NÃO FAZ NADA** — e a razão que o artista lê.
    ///
    /// `None` ⇒ ela está sempre viva. `Some((condição, chave))` ⇒ com a condição verdadeira ela
    /// fica **à vista, apagada, com a frase ao lado** — a decisão do dono de 2026-09-18, que esta
    /// secção citava no cabeçalho e não cumpria.
    ///
    /// ⛔⛔ **Medido em 2026-09-19 e é a resposta ao report *«Zone pivot parece morto»*:** na
    /// configuração em que o painel ABRE, quatro fileiras desta secção são inertes **por
    /// construção** — e as dez shipavam `inert: None`. *Um controlo travado e sem razão à vista
    /// lê-se exactamente como um controlo morto.*
    apagada: Option<Apagada>,
}

/// ⭐ **A razão por que uma fileira está apagada** — a condição, e a chave da frase que o artista lê.
///
/// ⚠️ Um tipo com nome e não a tupla crua: o clippy pede-o, e ele é melhor — *a condição e a frase
/// são UMA coisa* (nunca faz sentido ter uma sem a outra), e nomeá-la é o que impede alguém de
/// acrescentar uma terceira posição sem pensar no que ela significa.
type Apagada = (fn(&Style) -> bool, &'static str);

/// **Uma tinta que ninguém mexeu** — o branco de fábrica, que na lei multiplica por `1`.
fn branca(c: [f32; 3]) -> bool {
    c == [1.0; 3]
}

/// ⭐⭐⭐ **AS DOZE LINHAS** — cinco cores e sete números, na ordem em que o artista as lê.
///
/// # ⛔⛔⛔ Os tectos, MEDIDOS na auditoria de 2026-09-19 (`docs/Render3d/11` §10.8)
///
/// Esta tabela dizia **«três `4` por medir»** e a `W9` havia de os medir. A auditoria mediu-os antes
/// dela, e **três dos cinco eram palpites** — um deles com o número errado por `4×`:
///
/// | linha | tecto | de que recurso, e o número |
/// |---|---|---|
/// | força do contorno | `4` | ⚠️ tecto de **PRODUTO, com tabela**: o pico satura por volta de `1,5`–`2`, mas a ÁREA não satura (`18 270 → 47 626` px de `1` a `64`) — acima de `~2` deixa de ser um fio e vira lavagem |
/// | largura do contorno | [`ph2d_style::Rim::MAX_WIDTH`] | ✅ **MEDIDO e correcto**: o fio mede `0,535 px` a `1080p` no tecto, e o cruzamento de 1 px fica em `w ≈ 45–48` |
/// | **nitidez de aresta / de cova** | **`2`** | ⛔ **era `8` e a medição diz `2`**: `s = 1` entrega `99,0 %` do que `s = 8` entrega ⇒ **`87,5 %` do curso comprava `1 %` do efeito**. *É isto o «sem ajustes finos» do report* |
/// | **suavidade da curvatura** | [`ph2d_style::Curvature::MAX_SOFTNESS`] | ✅ **a lei declara-o e nomeia o recurso**: a FEIÇÃO MAIS PEQUENA que ainda se quer ver — acima dele as covas deixam de ser côncavas e a tinta delas morre |
/// | pivô das zonas | `4` | ⚠️ o contraste da grade **pica em `0,5`** e já está a cair no tecto; a imagem ainda move `100 %` dos píxeis a `4` ⇒ tecto de gosto, com tabela |
/// | saturação da indirecta | `4` | ⚠️ o efeito **cresce até `≥ 16`** (pior byte `13 · 36 · 66 · 110 · 119` a `2 · 4 · 8 · 16 · 32`) ⇒ tecto de gosto, com tabela |
///
/// ⚠️ **Três destes são multiplicativos numa pista LINEAR** e metade do efeito vive nos primeiros
/// `3`–`7 %` do curso. A porta para o curar já existe
/// (`ph2d_editor_core::…::link_slider_number_curved`) e é wave própria.
const LINHAS: [Linha; 12] = [
    Linha {
        slot: 0,
        key: "panel.model3d.style.rim_color",
        teto: None,
        apagada: Some((|s| s.rim.strength == 0.0, "field.inert.rim_is_off")),
    },
    Linha {
        slot: 3,
        key: "panel.model3d.style.rim_strength",
        teto: Some(4.0),
        apagada: None,
    },
    Linha {
        slot: 15,
        key: "panel.model3d.style.rim_width",
        teto: Some(ph2d_style::Rim::MAX_WIDTH),
        apagada: Some((|s| s.rim.strength == 0.0, "field.inert.rim_is_off")),
    },
    Linha {
        slot: 4,
        key: "panel.model3d.style.convex",
        teto: None,
        apagada: None,
    },
    Linha {
        slot: 8,
        key: "panel.model3d.style.concave",
        teto: None,
        apagada: None,
    },
    Linha {
        slot: 7,
        key: "panel.model3d.style.edge_sharpness",
        teto: Some(2.0),
        apagada: Some((|s| branca(s.curvature.convex), "field.inert.no_edge_tint")),
    },
    Linha {
        slot: 20,
        key: "panel.model3d.style.cavity_sharpness",
        teto: Some(2.0),
        apagada: Some((
            |s| branca(s.curvature.concave),
            "field.inert.no_cavity_tint",
        )),
    },
    Linha {
        slot: 21,
        key: "panel.model3d.style.softness",
        teto: Some(ph2d_style::Curvature::MAX_SOFTNESS),
        apagada: Some((|s| !s.reads_curvature(), "field.inert.no_curvature_tint")),
    },
    Linha {
        slot: 12,
        key: "panel.model3d.style.shadow_tint",
        teto: None,
        apagada: None,
    },
    Linha {
        slot: 16,
        key: "panel.model3d.style.highlight_tint",
        teto: None,
        apagada: None,
    },
    Linha {
        slot: 11,
        key: "panel.model3d.style.pivot",
        teto: Some(4.0),
        apagada: Some((
            |s| s.zones.shadow == s.zones.highlight,
            "field.inert.zones_are_the_same",
        )),
    },
    Linha {
        slot: 19,
        key: "panel.model3d.style.saturation",
        teto: Some(4.0),
        apagada: None,
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
            // ⭐⭐⭐ **A RAZÃO, quando a fileira não faz nada** — ver [`Linha::apagada`]. É a cura do
            // report *«Zone pivot não sei para que serve mas parece morto»*: ele é inerte **por
            // construção** enquanto as duas tintas de zona forem a mesma cor, e nada o dizia.
            inert: l
                .apagada
                .and_then(|(quando, porque)| quando(&style).then_some(porque)),
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
