//! `SpriteSheetFrame` — **este retângulo é uma folha de sprites**: a "imagem virtual" onde as
//! peças são montadas.
//!
//! ## Não é um tipo novo de objeto
//!
//! Gêmeo exato do [`crate::VecFrame`], e pela mesma razão que aquele doc escreve: a entidade é a
//! mesma que a ferramenta de forma produz — um retângulo vivo (`VecShape::Param { Rectangle }`) —
//! e este componente só acrescenta **o que ela FAZ com os filhos**. Como consequência de ser um
//! retângulo, saem **de graça**:
//!
//! | O que o artista faz | De onde vem |
//! |---|---|
//! | vê uma área transparente | o *fill* do retângulo |
//! | pega no gizmo da sprite | ADR-0111: uma forma vetorial publica `GizmoView` |
//! | põe uma sombra decorativa | a pilha de efeitos (o **Drop Shadow** já existe) |
//! | encontra-a na hierarquia | é uma entidade com [`crate::Name`] |
//! | esconde-a | [`crate::Visibility`] |
//! | move / redimensiona | `Transform` + o `w`/`h` do [`crate::VecShape`] |
//! | duplica | duplicação de entidade + subárvore |
//! | desfaz / salva | é componente: viaja no `WorldSnapshot` |
//!
//! Zero linhas de render, de hit-test ou de undo foram escritas para nada disto.
//!
//! ## ⚠️ Ele NÃO tem `size`, e isso é a decisão inteira
//!
//! A recusa é herdada do [`crate::VecFrame`], palavra por palavra: *"dois tamanhos divergem no
//! primeiro arrasto de alça, e o modo de falha é o pior que existe: o desenho concorda com um e o
//! layout com o outro, e nada parece errado."* O tamanho da folha **é** o `w`/`h` do `VecShape` da
//! entidade. Guardar uma cópia aqui seria construir essa divergência de propósito.
//!
//! ## As peças são FILHOS, e é isso que dispensa uma superfície de arrasto
//!
//! Os sprites que compõem a folha são filhos desta entidade. Logo **arrastar uma peça é mover um
//! filho** — com o gizmo, o snap e o undo que o app já tem. Não há um editor de folha a construir:
//! *a representação apaga o caso especial*. O botão de auto-arranjo apenas **propõe** poses; quem
//! decide é o artista, com o mouse.
//!
//! ## Densidade, não resolução
//!
//! [`SpriteSheetFrame::pixels_per_meter`] é a **densidade** do bake, e é ela que fica guardada
//! porque é o que sobrevive a um redimensionamento: esticar a folha dá mais espaço com o mesmo
//! detalhe. O tamanho em **pixels** que o painel mostra é derivado (`w` × densidade) — *uma fonte,
//! duas leituras*. Guardar a resolução em vez da densidade faria a folha perder nitidez sempre que
//! o artista a alargasse, sem lhe dizer porquê.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// Este retângulo é uma folha de sprites; os filhos dele são as peças.
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpriteSheetFrame {
    /// Densidade do bake, em **pixels por metro de mundo**.
    ///
    /// Semeada a partir do `ProjectSettings.pixels_per_meter` quando a folha nasce, para que o
    /// caso comum (assar no mesmo detalhe em que os sprites foram importados) não exija toque
    /// nenhum. Alterá-la é literalmente *"rasterizar com resolução diferente"*.
    pub pixels_per_meter: f32,
    /// Pixels transparentes deixados entre peças vizinhas ao assar.
    ///
    /// ⚠️ **Não é estética.** A amostragem bilinear lê meio texel para fora da borda, então duas
    /// peças coladas sangram uma na outra ao mínimo zoom — e a folha exportada vai ser lida por
    /// engines que não têm o nosso `region_filter_clip`.
    pub padding: u32,
}

impl SpriteSheetFrame {
    /// O padding que o Aseprite e o TexturePacker oferecem por omissão.
    pub const DEFAULT_PADDING: u32 = 2;

    /// Uma folha nova à densidade do projeto.
    pub fn at_density(pixels_per_meter: f32) -> Self {
        Self {
            pixels_per_meter,
            padding: Self::DEFAULT_PADDING,
        }
    }

    /// O tamanho em pixels que um lado de `meters` metros terá no bake.
    ///
    /// ⚠️ **Derivado, nunca guardado** — vide o cabeçalho. Piso de 1 px: uma folha de área zero
    /// ainda tem de produzir um buffer válido em vez de um `vec![]` que o consumidor lê como
    /// sucesso.
    pub fn pixels_for(&self, meters: f32) -> u32 {
        let ppm = if self.pixels_per_meter.is_finite() && self.pixels_per_meter > 0.0 {
            self.pixels_per_meter
        } else {
            1.0
        };
        ((meters.abs() * ppm).round() as i64).clamp(1, u32::MAX as i64) as u32
    }
}

impl SimComponent for SpriteSheetFrame {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixels_are_derived_from_density_and_size() {
        let f = SpriteSheetFrame::at_density(100.0);
        assert_eq!(f.pixels_for(2.0), 200);
        assert_eq!(f.pixels_for(0.5), 50);
    }

    /// ⚠️ A razão de guardar DENSIDADE e não resolução: esticar a folha dá mais espaço **com o
    /// mesmo detalhe**. Se a resolução fosse o guardado, alargar perderia nitidez em silêncio.
    #[test]
    fn stretching_the_sheet_buys_room_not_blur() {
        let f = SpriteSheetFrame::at_density(64.0);
        assert_eq!(f.pixels_for(1.0), 64);
        assert_eq!(f.pixels_for(4.0), 256, "4x a largura, 4x os pixels");
    }

    /// Uma densidade absurda não pode produzir um buffer vazio que o consumidor leia como sucesso.
    #[test]
    fn a_degenerate_density_still_yields_at_least_one_pixel() {
        for bad in [0.0, -5.0, f32::NAN, f32::INFINITY] {
            let f = SpriteSheetFrame {
                pixels_per_meter: bad,
                padding: 0,
            };
            assert!(f.pixels_for(2.0) >= 1, "densidade {bad}");
        }
        assert!(SpriteSheetFrame::at_density(100.0).pixels_for(0.0) >= 1);
    }
}
