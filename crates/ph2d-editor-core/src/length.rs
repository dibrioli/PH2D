//! **Como este app IMPRIME um comprimento** — a porta única.
//!
//! O mundo é guardado em METROS (a convenção do `Transform`, do rapier e do
//! documento vetorial). O artista lê o que ele escolheu no menu Settings:
//! [`DisplayUnit::Pixels`] (o default, `× pixels_per_meter`) ou
//! [`DisplayUnit::Meters`]. Isto é a **fronteira de display** do doc 88, aqui
//! aplicada a comprimentos de MUNDO em vez de params de nó.
//!
//! # Por que uma porta, e não um `format!` em cada sítio
//!
//! ⚠️ Antes desta wave havia **duas respostas divergentes na tela ao mesmo
//! tempo**, e nenhuma sabia da outra:
//!
//! - o painel **Grid Snap** converte (`grid_snap::inspect`, o `NumberInput` do
//!   passo lê `display_unit.from_meters`) ⇒ com os defaults (100 px/m, Pixels)
//!   o artista digita **100**;
//! - a **RÉGUA** não convertia — `paint_rulers` nem sequer RECEBIA as settings,
//!   então ela **não conseguia** —, e rotulava a mesma linha de grade com **1**.
//!
//! Um app que diz `100` e `1` para a mesma distância, em duas superfícies que
//! o artista vê lado a lado, não tem um bug de rótulo: tem duas portas para uma
//! pergunta. O rótulo de distância dos smart guides (plano 25 §9, a W6) seria a
//! **terceira**, e é por isso que ele chega junto com esta porta.
//!
//! # O que ela decide, e o que ela NÃO decide
//!
//! Ela decide **o número e as casas**. Não decide onde o texto pousa, com que
//! corpo, nem se há sufixo — isso é de quem desenha, e as duas superfícies
//! respondem diferente de propósito: a régua imprime o número NU (uma régua é
//! entendida pela faixa em que ela vive) e o rótulo flutuante carrega o sufixo
//! (ele paira sobre a arte, sem eixo nenhum para o explicar).

use crate::project::{DisplayUnit, ProjectSettings};

/// A régua pela qual um comprimento de mundo vira o número que o artista lê.
///
/// Cópia barata (dois campos escalares) de propósito: ela é passada por valor
/// para todo pintor, e um empréstimo do `ProjectSettings` inteiro prenderia o
/// painel a um estado que ele não usa.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LengthDisplay {
    pub unit: DisplayUnit,
    pub pixels_per_meter: f32,
}

impl Default for LengthDisplay {
    /// Os mesmos defaults do projeto — para que uma fixture que não fala de
    /// unidade meça o que o app mede ao abrir.
    fn default() -> Self {
        Self::of(&ProjectSettings::default())
    }
}

impl LengthDisplay {
    /// A régua VIVA do projeto. **Toda** superfície que imprime comprimento
    /// nasce daqui — nunca de um `DisplayUnit` solto ao lado de um `f32` de
    /// escala, que é a forma de os dois se separarem.
    #[must_use]
    pub fn of(p: &ProjectSettings) -> Self {
        Self {
            unit: p.display_unit,
            pixels_per_meter: p.pixels_per_meter,
        }
    }

    /// O número que o artista lê, dado um comprimento (ou uma coordenada) em
    /// metros de mundo.
    #[must_use]
    pub fn value(self, world: f64) -> f64 {
        self.unit.from_meters_f64(world, self.pixels_per_meter)
    }

    /// `"m"` ou `"px"` — o sufixo, para quem o desenha.
    #[must_use]
    pub fn suffix(self) -> &'static str {
        self.unit.suffix()
    }

    /// O texto de um comprimento, com as casas que a sua **RESOLUÇÃO** justifica.
    ///
    /// `resolution_world` é o menor incremento que este número pode
    /// significativamente tomar, em metros de mundo — e **cada superfície tem a
    /// sua**: a régua passa o passo dos traços (o rótulo dela pousa SOBRE um
    /// traço, logo o valor é sempre um múltiplo do passo) e a ficha flutuante
    /// passa **um pixel de tela** (o valor dela é o que o arrasto do artista
    /// produziu, e não é múltiplo de nada). Uma regra, dois argumentos.
    ///
    /// ⚠️ **A resolução entra em MUNDO e é convertida pela MESMA porta.**
    /// Converter só o valor imprimiria `100` com as casas de um passo de `0,5` —
    /// uma casa decimal que o número não tem resolução para honrar.
    #[must_use]
    pub fn text(self, world: f64, resolution_world: f64) -> String {
        format_value(
            self.value(world),
            decimals_for(self.value(resolution_world)),
        )
    }

    /// O texto com as casas que **UM PIXEL de tela** justifica — a régua de toda
    /// leitura que paira sobre a arte, sem faixa graduada ao lado.
    ///
    /// ⚠️ **A 1ª versão desta função emprestava a cadência de rótulos da RÉGUA
    /// (`ruler::label_step`), e o smoke a reprovou.** As duas respondem perguntas
    /// diferentes, e a diferença é grande: `label_step` responde *que números
    /// merecem ser IMPRESSOS numa faixa graduada*, uma pergunta de LAYOUT (dois
    /// rótulos não podem colidir, daí os 56 px de `ruler::MIN_LABEL_PX`); esta
    /// responde *quanta resolução este zoom DISTINGUE*, que é uma pergunta sobre
    /// **um** pixel. No zoom de trabalho (~100 px por metro) aquele passo vale
    /// **1 m**, então uma distância de 1,5 m era impressa como **`2`** — não
    /// grosseira, errada. Um pixel ali vale 1 cm, e a mesma distância imprime
    /// `1.50`.
    ///
    /// Para a RÉGUA as duas cadências coincidem por construção, e é por isso que
    /// ela fica como está: o rótulo dela senta num traço, logo o valor É múltiplo
    /// do passo e não há nada abaixo dele a perder. Emprestar o passo para um
    /// número ARBITRÁRIO joga fora tudo o que está abaixo dele.
    ///
    /// A regra nunca esconde um dígito que o artista possa ver; nos zooms que
    /// caem no meio de uma década ela mostra **um a mais** (uma resolução de
    /// 5 mm imprime milímetros), e isso é o lado certo para errar: é o que faz
    /// cada pixel de arrasto mexer no número em vez de o deixar gaguejar.
    #[must_use]
    pub fn text_at_zoom(self, world: f64, px_per_world: f64) -> String {
        self.text(world, world_per_pixel(px_per_world))
    }
}

/// Quanto MUNDO cabe num pixel de tela — a resolução que o olho tem neste zoom.
///
/// Zoom degenerado (não-finito ou ≤ 0) cai em `1.0`, o mesmo fallback do
/// [`crate::ruler::label_step`]: sem escala não há resolução a afirmar, e um
/// número redondo é a leitura honesta. Duas funções, um fallback — um zoom
/// degenerado não pode fazer a régua e a ficha discordarem.
#[must_use]
pub fn world_per_pixel(px_per_world: f64) -> f64 {
    if px_per_world.is_finite() && px_per_world > 0.0 {
        1.0 / px_per_world
    } else {
        1.0
    }
}

/// Quantas casas decimais uma resolução justifica — a regra que morava dentro do
/// `ruler::label_text`, agora com dois consumidores.
///
/// O argumento está em unidade de DISPLAY: meio metro é `0,5` em metros (uma
/// casa) e `50` em pixels (nenhuma), e o número de casas segue o que a tela
/// mostra, não o que a memória guarda.
#[must_use]
pub fn decimals_for(resolution_display: f64) -> usize {
    let s = resolution_display.abs();
    if s >= 1.0 {
        0
    } else if s >= 0.1 {
        1
    } else if s >= 0.01 {
        2
    } else {
        3
    }
}

/// Formata com `decimals` casas, normalizando o zero negativo.
///
/// ⚠️ `-0` é o mesmo lugar que `0` e lê como um erro numa régua — a régua já
/// carregava esta linha, e ela viaja junto com a regra que a produzia.
#[must_use]
pub fn format_value(v: f64, decimals: usize) -> String {
    let v = if v == 0.0 { 0.0 } else { v };
    format!("{v:.decimals$}")
}

#[cfg(test)]
#[path = "length_tests.rs"]
mod tests;
