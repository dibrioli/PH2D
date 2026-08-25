//! `SpriteRegion` — **este sprite amostra um sub-retângulo da textura**, não a textura inteira
//! (ADR-0164 F1 passo 6 / ADR-0166).
//!
//! # ⭐ A PRESENÇA substitui o `region_enabled`, e isso apaga um estado impossível
//!
//! O `Sprite` v4 guardava `region_enabled: bool` **ao lado** de `region_rect: [f32; 4]`, e a
//! combinação `enabled = false` com um retângulo autorado era estado que ninguém conseguia ler:
//! *"há região ou não há?"* tinha duas respostas e a segunda era invisível. Um componente responde
//! a pergunta **por existir** — *a representação apaga o caso especial*, o mesmo movimento que o
//! [`crate::SpriteGrid`] faz com a grelha de uma célula.
//!
//! # ⚠️ O `filter_clip` tinha um default CONDICIONAL, e era um defeito documentado
//!
//! No `Sprite` v4 o `region_filter_clip` valia `true` para sprites de **Atlas** (anti-bleed entre
//! regiões vizinhas) e `false` para **Individual** — e o `#[serde(default)]` dele devolvia sempre o
//! valor do Atlas, que é **o errado para Individual**. O próprio campo trazia a nota: *"a escolha
//! condicional vive em `migrate_v3_to_v4` / `Sprite::individual`, NÃO neste default"*.
//!
//! Aqui a escolha é do **construtor** ([`SpriteRegion::for_atlas`] / [`SpriteRegion::individual`]),
//! que é onde ela sempre pertenceu: quem cria a região sabe de que fonte ela é. O `Default` existe
//! só porque o registo o exige (ver a nota nele) e escolhe o valor **conservador**.
//!
//! # ⚠️ A folha HAND-PACKED depende deste componente
//!
//! [`crate::SpriteSheetRef`] é construído por cima da região — o doc dele diz, por escrito, que
//! *"o retângulo vive no `Sprite.region_rect` (cozido a partir da folha, no load e no import)"* e
//! que com ele *"o caminho de render não muda uma linha"*. ⇒ **Todo sprite com `SpriteSheetRef`
//! tem de ter um `SpriteRegion`**, e a migração honra-o. ⛔ Não «simplifique» retirando a região de
//! um sprite de folha: ele passaria a amostrar a folha INTEIRA, que é a textura de todas as peças.
//!
//! # Ausência = a textura inteira
//!
//! Sem o componente o sprite amostra a textura toda — exatamente o que `region_enabled = false`
//! significava.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// O sub-retângulo da textura que este sprite amostra.
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpriteRegion {
    /// `[x, y, w, h]` em **pixels de textura**. `w`/`h` mantidos `>= 0` pelo setter.
    pub rect: [f32; 4],
    /// Prende o sampler ao [`Self::rect`] para o texel vizinho não sangrar pela borda.
    ///
    /// ⚠️ O valor CERTO depende da fonte dos pixels — ver o cabeçalho do módulo. Use
    /// [`Self::for_atlas`] / [`Self::individual`] em vez de escrever o bool à mão.
    pub filter_clip: bool,
}

impl SpriteRegion {
    /// Uma região de um sprite de **atlas partilhado**: `filter_clip` LIGADO.
    ///
    /// No atlas as regiões são vizinhas na mesma textura, então a interpolação bilinear na borda
    /// puxa o texel do sprite do lado — a franja que este clamp existe para matar.
    #[must_use]
    pub const fn for_atlas(rect: [f32; 4]) -> Self {
        Self {
            rect,
            filter_clip: true,
        }
    }

    /// Uma região de uma textura **própria** (`Individual`): `filter_clip` DESLIGADO.
    ///
    /// Não há vizinho de quem sangrar — a textura é só deste sprite —, e prender o sampler
    /// custaria nitidez na borda sem comprar nada.
    #[must_use]
    pub const fn individual(rect: [f32; 4]) -> Self {
        Self {
            rect,
            filter_clip: false,
        }
    }
}

impl Default for SpriteRegion {
    /// ⚠️ **Existe pelo `register_default` do registo**, que constrói por `Default` antes de
    /// preencher com os bytes do disco — não é um default de produto. Escolhe o valor
    /// **conservador** (`filter_clip` ligado): errar para o lado do clamp custa nitidez de meio
    /// texel; errar para o outro lado põe o vizinho dentro do sprite, que é visível.
    fn default() -> Self {
        Self {
            rect: [0.0; 4],
            filter_clip: true,
        }
    }
}

impl SimComponent for SpriteRegion {}
