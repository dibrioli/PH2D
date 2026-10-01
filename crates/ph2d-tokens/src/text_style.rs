//! ⭐⭐⭐ **O ESTILO DO TEXTO DA INTERFACE — a fonte, o peso e o tamanho que o ARTISTA escolhe**
//! (ordem do dono, 2026-10-01: *«pelo menos mais 2 tipos de fonts e 2 tipos de peso das fontes
//! assim como ajuste do tamanho das fonts. Veja como o blender e a godot fazem»*).
//!
//! ⭐ **Os dois oráculos foram CORRIDOS, não lembrados** (Blender 5.2.2 LTS por `bpy`, Godot 4.7.2
//! por um plugin que lê as `EditorSettings`): os dois separam a FONTE, o PESO, o TAMANHO e a
//! NITIDEZ em controlos independentes — o Blender com `font_path_ui` · `character_weight` (100..900)
//! · `points` (6..32) · `ui_scale` (0,5..6); a Godot com `main_font` · `main_font_bold` ·
//! `main_font_size` (8..48) · `display_scale` (75 %..200 %). ⇒ três eixos aqui, ao lado do
//! [`crate::TextRendering`], que continua a ser só a nitidez.
//!
//! ⚠️ **Nenhum número desta folha é escolhido: cada degrau é UM passo da escada que já existe.**
//! O tamanho `Small`/`Large` é o degrau de baixo/de cima da escala de tipo ([`TypeToken::Sm`] ·
//! [`TypeToken::Md`] sobre o [`TypeToken::Base`]) e o peso `Strong` é o degrau seguinte da escada de
//! pesos (`SemiBold − Medium`). ⛔ **E o tamanho PÁRA no `Large` por um RECURSO medido:** as linhas
//! dos painéis têm `ROW_H_PX` fixo (`22 px`) e o corpo a `15 px` ainda cabe nelas com a linha da
//! Inter (`1,21 em`); acima disso as letras encostam nas linhas vizinhas, e o que o Blender e a
//! Godot fazem nesse regime é a ESCALA DA INTERFACE inteira — outra obra.
//!
//! ⚠️ **O valor de fábrica** é `Inter` · `Normal` · `Normal`: escala `1,0` exacta e reforço `0`. A
//! FONTE muda em relação ao que esta máquina mostrava antes — [`UiFont::Inter`] diz porquê.

use crate::typography::{FontWeight, TypeToken};

/// ⭐ **A fonte da interface.** As três são variáveis no peso (o reforço de [`UiWeight`] é contínuo
/// nelas) e as três vão DENTRO do app, com a licença ao lado (SIL OFL 1.1 — `ph2d-text/fonts/`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum UiFont {
    /// ⭐ **A de FÁBRICA: a família que o design system declara**, desenhada para ecrã e com o eixo de
    /// tamanho óptico. ⚠️ Até 2026-10-01 o app **não a desenhava**: a pilha pedia `InterVariable`
    /// e, com as fontes do sistema carregadas, caía no `sans-serif` — nesta máquina a
    /// `NotoSans-Medium.ttf` instalada (`619 976` bytes, medido). Ela é a fábrica e não a Noto
    /// porque a régua é a mesma em toda máquina: **135** construções de teste medem a Inter, e as
    /// **4** que mediam a fonte do sistema mediam a fonte DESTA máquina (numa sem Noto mediriam
    /// outra). ⇒ quem quiser a aparência de antes escolhe [`Self::NotoSans`].
    #[default]
    Inter,
    /// Neutra, cobre latim, grego e cirílico inteiros — e é a aparência que o app tinha nesta
    /// máquina antes de 2026-10-01 (ver [`Self::Inter`]).
    NotoSans,
    /// Desenhada para leitura fácil: `l`, `I` e `1` não se confundem.
    AtkinsonHyperlegible,
}

impl UiFont {
    /// Todas, pela ordem do menu.
    pub const ALL: [Self; 3] = [Self::Inter, Self::NotoSans, Self::AtkinsonHyperlegible];

    /// O nome estável no ficheiro de preferências. Inverso de [`Self::from_wire`].
    #[must_use]
    pub const fn wire(self) -> &'static str {
        match self {
            Self::Inter => "inter",
            Self::NotoSans => "noto_sans",
            Self::AtkinsonHyperlegible => "atkinson_hyperlegible",
        }
    }

    /// Lê o nome do ficheiro de preferências; `None` para um nome que este build não conhece.
    #[must_use]
    pub fn from_wire(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|f| f.wire() == s)
    }

    /// A posição em [`Self::ALL`] — o índice da fonte registada no sistema de texto.
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// ⭐ **O peso da interface** — um desvio somado ao peso nominal de cada texto (o corpo é
/// `Medium`, os títulos `SemiBold`), para os dois andarem juntos e a hierarquia ficar.
///
/// ⚠️ Os dois desvios são o MESMO degrau da escada de pesos, um para cada lado — `Light` desce o
/// que `Strong` sobe (pedido do dono, 2026-10-01: *«uma opção mais delicada que normal»*). As três
/// fontes da casa são variáveis no peso e cobrem o degrau abaixo (Inter e Noto `100..900`,
/// Atkinson `200..800`), logo nenhuma delas o arredonda para o vizinho.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum UiWeight {
    /// Um degrau abaixo: o corpo em `Regular`, os títulos em `Medium`.
    Light,
    /// O peso de cada texto, como o desenho o pede.
    #[default]
    Normal,
    /// Um degrau acima na escada de pesos: o corpo em `SemiBold`, os títulos em `Bold`.
    Strong,
}

impl UiWeight {
    /// Os três, pela ordem do menu.
    pub const ALL: [Self; 3] = [Self::Light, Self::Normal, Self::Strong];

    /// O que se soma ao peso nominal (`0` no `Normal` — a identidade; negativo no `Light`).
    #[must_use]
    pub const fn boost(self) -> i16 {
        // Os pesos da escada são `100..=900`, logo cabem num `i16` sem perda.
        let degrau_abaixo = FontWeight::Regular.value() as i16 - FontWeight::Medium.value() as i16;
        let degrau_acima = FontWeight::Semibold.value() as i16 - FontWeight::Medium.value() as i16;
        match self {
            Self::Light => degrau_abaixo,
            Self::Normal => 0,
            Self::Strong => degrau_acima,
        }
    }

    /// O nome estável no ficheiro de preferências.
    #[must_use]
    pub const fn wire(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Normal => "normal",
            Self::Strong => "strong",
        }
    }

    /// Inverso de [`Self::wire`].
    #[must_use]
    pub fn from_wire(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|w| w.wire() == s)
    }
}

/// ⭐ **O tamanho do texto da interface** — um FACTOR sobre todo tamanho de texto, para a escala de
/// tipo inteira subir junta (o título continua maior que o corpo).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum UiTextSize {
    /// O corpo um degrau abaixo (`Sm` em vez de `Base`).
    Small,
    /// O de sempre.
    #[default]
    Normal,
    /// O corpo um degrau acima (`Md` em vez de `Base`) — o maior que as linhas de `ROW_H_PX` levam.
    Large,
}

impl UiTextSize {
    /// Os três, pela ordem do menu.
    pub const ALL: [Self; 3] = [Self::Small, Self::Normal, Self::Large];

    /// O factor sobre o tamanho nominal. ⚠️ `Normal` é `1,0` EXACTO (`Base / Base`), e é isso que
    /// faz o valor de fábrica não mexer um píxel.
    #[must_use]
    pub fn scale(self) -> f32 {
        let corpo = match self {
            Self::Small => TypeToken::Sm.px(),
            Self::Normal => TypeToken::Base.px(),
            Self::Large => TypeToken::Md.px(),
        };
        corpo / TypeToken::Base.px()
    }

    /// O nome estável no ficheiro de preferências.
    #[must_use]
    pub const fn wire(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Normal => "normal",
            Self::Large => "large",
        }
    }

    /// Inverso de [`Self::wire`].
    #[must_use]
    pub fn from_wire(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|z| z.wire() == s)
    }
}

/// ⭐⭐ **Os três eixos juntos** — o que o sistema de texto lê a cada layout.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct UiTextStyle {
    pub font: UiFont,
    pub weight: UiWeight,
    pub size: UiTextSize,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ **A fábrica é a identidade AO BIT** — sem isto, abrir o app mudava o texto de todo o mundo.
    #[test]
    fn the_factory_style_changes_nothing() {
        let s = UiTextStyle::default();
        assert_eq!(s.font, UiFont::Inter);
        assert_eq!(s.weight.boost(), 0);
        assert_eq!(s.size.scale().to_bits(), 1.0_f32.to_bits());
    }

    /// Os degraus andam no sentido do nome, e o peso `Strong` sobe mesmo.
    #[test]
    fn each_step_goes_the_way_its_name_says() {
        assert!(UiTextSize::Small.scale() < 1.0 && UiTextSize::Large.scale() > 1.0);
        assert!(UiWeight::Strong.boost() > 0);
        assert!(UiWeight::Light.boost() < 0);
        // O mesmo degrau para os dois lados — a hierarquia do corpo para o título fica igual.
        assert_eq!(UiWeight::Light.boost(), -UiWeight::Strong.boost());
    }

    /// O ficheiro de preferências lê o que escreveu — nos três eixos, para todo valor.
    #[test]
    fn every_value_survives_the_prefs_file() {
        for f in UiFont::ALL {
            assert_eq!(UiFont::from_wire(f.wire()), Some(f));
        }
        for w in UiWeight::ALL {
            assert_eq!(UiWeight::from_wire(w.wire()), Some(w));
        }
        for z in UiTextSize::ALL {
            assert_eq!(UiTextSize::from_wire(z.wire()), Some(z));
        }
        assert_eq!(UiFont::from_wire("comic"), None);
    }

    /// O índice de uma fonte é a posição dela na lista — é por ele que o sistema de texto a acha.
    #[test]
    fn the_font_index_is_its_place_in_the_list() {
        for (i, f) in UiFont::ALL.into_iter().enumerate() {
            assert_eq!(f.index(), i);
        }
    }
}
