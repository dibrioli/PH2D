//! `SpriteSheetRef` — este sprite é **uma região de uma folha hand-packed**.
//!
//! ## Por que é uma componente e não um variante de `SpriteSource`
//!
//! Foi medido antes de escolher (2026-08-19). `SpriteSource` é casado exaustivamente em **25
//! sítios de 16 arquivos**, e a lição registada para exatamente esta situação é
//! [[feedback_widely_constructed_type_favors_optional_component_over_appended_field]]: *tipo
//! construído em N sítios → componente opcional*. Um variante novo custaria os 25 sítios, um
//! parâmetro a mais no extract e uma tabela de resolução — para exprimir algo que a **composição
//! já exprime**:
//!
//! > uma folha é *uma textura partilhada* + *um retângulo por sprite*.
//!
//! O `IndividualTextureStore` já guarda uma textura por id **com refcount**, e
//! `Sprite::region_rect` + `region_enabled` + `region_subrect()` já convertem um retângulo em
//! pixels para UV, com oito testes. Com esta componente o caminho de render **não muda uma
//! linha** — e o `HandPackedAtlasStore` que o [ADR-0026] previa nunca é construído, o que também
//! evita uma segunda resposta à pergunta *"que textura este id nomeia?"* (o `renderer_draw` diz
//! por escrito que ela tem uma porta só).
//!
//! ## O que ela guarda, e por que é durável
//!
//! O par `(sheet, region)` é **autoria**: `sheet` é o id estável do documento
//! (`ph2d_sprite_sheet::AuthoredSheet::id`) e `region` é o índice na lista de regiões daquela
//! folha, **ordenada por nome** — nenhum dos dois morre com o processo. O `Sprite.source` continua
//! a carregar o `texture_id` de runtime (que morre), e é esta componente que o repõe no load,
//! exatamente como o [`crate::PaintedDoc`] faz pelo documento pintado.
//!
//! ⚠️ **Ela e o [`crate::SpritePixels`] são mutuamente exclusivos** — pixels próprios *ou* uma
//! região de uma folha partilhada. Quem converte um sprite de uma coisa para a outra retira a
//! componente da anterior, senão o arquivo grava duas verdades sobre o mesmo sprite.
//!
//! ## O que ela NÃO faz
//!
//! Não guarda pixels nem retângulos. O retângulo vive no `Sprite.region_rect` (cozido a partir da
//! folha, no load e no import) e os pixels no documento — aqui viaja só *de que região de que
//! folha este sprite é*. É a mesma separação fonte-≠-cozido do ADR-0132 no vetor.
//!
//! [ADR-0026]: ../../../docs/architecture/decisions/0026-sprite-source-strategies.md

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// De que região de que folha hand-packed este sprite é.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteSheetRef {
    /// Id estável da folha (`ph2d_sprite_sheet::AuthoredSheet::id`).
    pub sheet: u32,
    /// Índice na lista de regiões da folha — que é ordenada **por nome**, e é isso que torna o
    /// índice uma referência estável entre importações.
    pub region: u32,
}

impl SimComponent for SpriteSheetRef {}
