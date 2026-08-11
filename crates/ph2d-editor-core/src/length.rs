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

    /// O texto de um comprimento, com as casas que o **passo** justifica.
    ///
    /// ⚠️ **O passo entra em MUNDO e é convertido pela MESMA porta.** Converter
    /// só o valor imprimiria `100` com as casas de um passo de `0,5` — três
    /// dígitos e uma casa decimal que o número não tem resolução para honrar.
    #[must_use]
    pub fn text(self, world: f64, step_world: f64) -> String {
        format_value(self.value(world), decimals_for(self.value(step_world)))
    }

    /// O texto com as casas que o **ZOOM** justifica.
    ///
    /// A cadência de rótulos da régua ([`crate::ruler::label_step`]) é a
    /// política de precisão deste app: ela responde *quanta resolução este zoom
    /// distingue*, que é exatamente a pergunta que um rótulo flutuante tem.
    /// Chamá-la aqui é o que impede a régua e o rótulo de mostrarem casas
    /// diferentes para a mesma distância no mesmo instante.
    #[must_use]
    pub fn text_at_zoom(self, world: f64, px_per_world: f64) -> String {
        self.text(world, crate::ruler::label_step(px_per_world))
    }
}

/// Quantas casas decimais um passo justifica — a regra que morava dentro do
/// `ruler::label_text`, agora com dois consumidores.
///
/// O argumento está em unidade de DISPLAY: um passo de meio metro é `0,5` em
/// metros (uma casa) e `50` em pixels (nenhuma), e o número de casas segue o
/// que a tela mostra, não o que a memória guarda.
#[must_use]
pub fn decimals_for(step_display: f64) -> usize {
    let s = step_display.abs();
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
