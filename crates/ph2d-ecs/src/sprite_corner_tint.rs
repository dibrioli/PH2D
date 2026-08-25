//! `SpriteCornerTint` — o **gradiente bilinear de 4 paradas** de um sprite, uma cor por canto
//! (ADR-0164 F1 passo 6 / ADR-0166).
//!
//! # Porque sai do `Sprite`
//!
//! São **64 bytes** (`[[f32; 4]; 4]`) que a esmagadora maioria dos sprites carrega em branco — mas
//! o motivo do corte não é o tamanho ([ADR-0166](../../../docs/architecture/decisions/0166-the-inspector-shows-what-the-object-has-and-components-attach-through-one-palette-filtered-by-object-type.md)):
//! é que um *campo* de um componente que todo objeto-imagem tem **não pode não aparecer** no
//! Inspector. Um degradê por cantos é uma escolha do artista (o `setTint` de quatro cores do
//! Phaser), não parte do que uma imagem É.
//!
//! # ⚠️ Ele é UM dos QUATRO canais multiplicativos, e os outros três FICAM
//!
//! O [ADR-0071](../../../docs/architecture/decisions/0071-tint-channels-multiplicative.md) declara
//! quatro canais de tinta canónicos e o gate `tint_channel_count` toca em cada um **pelo nome**,
//! para que renomear ou remover seja erro de compilação. Os outros três continuam campos do
//! `Sprite` e a decisão é deliberada:
//!
//! | canal | onde vive | porquê |
//! |---|---|---|
//! | `tint` | `Sprite` | modulate herdado — todo objeto-imagem tem um, default branco |
//! | `self_tint` | `Sprite` | o `self_modulate` do Godot; o par com `tint` é a base, não uma escolha |
//! | `opacity` | `Sprite` | visibilidade final — universal |
//! | **`per_corner_tint`** | **aqui** | um degradê é uma feature, e a ausência dele é branco |
//!
//! ⇒ o gate do ADR-0071 passa a ler o quarto canal **deste componente**, e continua a ser um gate
//! sobre o CONJUNTO de canais (nenhum se perdeu; um mudou de casa).
//!
//! # Ausência = identidade
//!
//! Sem o componente os quatro cantos são **branco** — o que multiplica por 1 e não se vê. Um
//! projeto que nunca tocou no degradê é byte-idêntico ao que era.
//!
//! ⚠️ **A ordem dos cantos é o contrato:** `[TopLeft, TopRight, BottomLeft, BottomRight]`, a mesma
//! que a `RenderInstance.per_corner_tint` sobe para o vertex shader. Trocá-la espelharia o degradê
//! de toda cena já autorada, em silêncio.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// Índice do canto superior esquerdo em [`SpriteCornerTint::0`].
pub const TOP_LEFT: usize = 0;
/// Índice do canto superior direito.
pub const TOP_RIGHT: usize = 1;
/// Índice do canto inferior esquerdo.
pub const BOTTOM_LEFT: usize = 2;
/// Índice do canto inferior direito.
pub const BOTTOM_RIGHT: usize = 3;

/// Uma cor RGBA por canto, na ordem `[TopLeft, TopRight, BottomLeft, BottomRight]`.
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpriteCornerTint(pub [[f32; 4]; 4]);

impl SpriteCornerTint {
    /// Os quatro cantos brancos — o que a **ausência** do componente significa.
    pub const IDENTITY: Self = Self([[1.0; 4]; 4]);

    /// `true` quando este degradê é indistinguível da ausência do componente.
    ///
    /// ⚠️ Comparação **por bits de `f32`**, e de propósito: a pergunta é *"isto foi autorado?"*,
    /// não *"isto é visualmente branco"*. Um `0.999_999` que o artista digitou é autoria, e uma
    /// migração que o descartasse por «arredonda para branco» apagaria trabalho.
    #[must_use]
    pub fn is_identity(&self) -> bool {
        self.0 == Self::IDENTITY.0
    }
}

impl Default for SpriteCornerTint {
    /// ⚠️ **`IDENTITY`, não `zeroed`.** O `register_default` do registo constrói por `Default`
    /// antes de preencher com os bytes do disco; nascer em preto-transparente faria uma falha de
    /// leitura pintar o sprite de preto em vez de o deixar como estava.
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl SimComponent for SpriteCornerTint {}
