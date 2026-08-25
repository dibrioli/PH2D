//! `SpriteGrid` — **a folha INLINE**: a grelha `hframes × vframes` que divide a textura de um
//! sprite em células, e o índice da célula viva (ADR-0164 F1 passo 6 / ADR-0166).
//!
//! # Porque é um componente, e não três campos do `Sprite`
//!
//! A razão **não é tamanho** ([ADR-0166](../../../docs/architecture/decisions/0166-the-inspector-shows-what-the-object-has-and-components-attach-through-one-palette-filtered-by-object-type.md)):
//! enquanto o dado for *campo* de um componente que **todo** objeto-imagem tem, não há como não o
//! mostrar no Inspector. Um campo só pode desaparecer da vista quando é um componente que pode
//! estar **ausente**. O critério do corte é *"isto pertence ao objeto-imagem BASE, ou é uma escolha
//! que o artista faz?"* — e uma grelha é uma escolha: a esmagadora maioria dos sprites é uma imagem
//! só.
//!
//! # ⚠️ O NOME: porquê `SpriteGrid` e não `SpriteSheet` (o plano dizia `SpriteSheet`)
//!
//! Esta crate **já tem duas** componentes cujo nome começa por `SpriteSheet`, e as duas significam
//! a *folha HAND-PACKED*, que é outra coisa:
//!
//! - [`crate::SpriteSheetRef`] — *"sou a região R da folha autorada S"*;
//! - [`crate::SpriteSheetFrame`] — *"este retângulo É uma folha; os filhos dele são as peças"*.
//!
//! Um terceiro `SpriteSheet*` a significar **grelha inline** poria três nomes quase iguais para
//! duas ideias distintas, e a que se lê ao contrário é a que morde. *Grelha* já é a palavra que os
//! docs usam para esta ideia (§11 Animation: *"o pool de frames já existe — a **grelha**
//! `hframes × vframes`"*), então o componente chama-se pelo que ela é.
//!
//! # Ausência = default benigno
//!
//! Sem o componente a textura é **uma célula**: `hframes = vframes = 1`, `frame = 0` — exatamente o
//! que os defaults do `Sprite` v4 diziam. Criar um sprite continua a ser **um gesto**, e um projeto
//! sem grelha nenhuma é byte-idêntico ao que era.
//!
//! # Quem escreve o `frame`
//!
//! O [`crate::SpriteAnimator`] (§11 Animation) — é o **único sink vivo** do índice, e ele escreve-o
//! *só quando muda* (o undo regista por diff). ⚠️ Uma `AnimationTag` indexa **células**, então
//! animar quadros sem grelha é inexprimível por construção: sem este componente não há células.
//!
//! [ADR-0164]: ../../../docs/architecture/decisions/0164-instances-are-real-entities-linked-by-stableid-with-live-sync-and-incremental-undo.md

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// A grelha inline que divide a textura deste sprite em células.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteGrid {
    /// Colunas. `>= 1` — garantido pelos construtores e pelo setter do Inspector.
    pub hframes: u32,
    /// Linhas. `>= 1`.
    pub vframes: u32,
    /// Célula viva, `< hframes * vframes`.
    pub frame: u32,
}

impl SpriteGrid {
    /// A grelha de uma célula — o que a **ausência** do componente significa.
    ///
    /// Existe para que um leitor possa escrever `grid.copied().unwrap_or(SpriteGrid::SINGLE)` e
    /// ficar com a lei num sítio só, em vez de repetir `1, 1, 0` em cada `unwrap_or`.
    pub const SINGLE: Self = Self {
        hframes: 1,
        vframes: 1,
        frame: 0,
    };

    /// Quantas células a grelha tem. Satura em vez de estourar: `hframes`/`vframes` vêm de um
    /// campo digitável, e um `u32 * u32` a transbordar num painel seria um pânico por dedada.
    #[must_use]
    pub const fn cells(self) -> u32 {
        self.hframes.saturating_mul(self.vframes)
    }

    /// `true` quando esta grelha é indistinguível da ausência do componente.
    ///
    /// ⚠️ É a pergunta que a **migração** faz (não vale a pena anexar um componente que não diz
    /// nada) e a que o **save** faria se alguém quisesse podar. Mantida aqui, e não escrita duas
    /// vezes, porque «o que conta como vazio» é uma lei do tipo.
    #[must_use]
    pub const fn is_single(self) -> bool {
        self.hframes <= 1 && self.vframes <= 1 && self.frame == 0
    }
}

impl Default for SpriteGrid {
    /// ⚠️ **`SINGLE`, não `zeroed`.** Um `derive(Default)` daria `hframes = 0`, que é uma grelha
    /// sem células — e o `register_default` do registo **constrói por `Default`** ao inserir um
    /// componente vindo do disco antes de o preencher. Zero células é o estado inválido que os
    /// setters existem para impedir; nascer nele tornaria o inválido alcançável pela porta que
    /// menos se olha.
    fn default() -> Self {
        Self::SINGLE
    }
}

impl SimComponent for SpriteGrid {}
