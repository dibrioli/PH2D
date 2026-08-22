//! **O degrau de FX raster** — o `FxOp` e o catálogo que o descreve, sem ECS e sem GPU.
//!
//! # Por que uma folha, e não o `ph2d-ecs` onde isto nasceu
//!
//! O tipo vive em **três** lugares do editor: o documento o guarda num componente
//! (`ph2d_ecs::VecFilter`), o device o consome ao desenhar (`ph2d_render::fx_stack`), e — desde
//! 2026-08-21 — **uma POSE de estado de UI o carrega** (`ph2d_ui_state::ObjectPose`), para um
//! blur ou um glow poderem diferir entre *Default* e *Hover*.
//!
//! Foi o terceiro consumidor que obrigou a mudança de casa. A `ph2d-ui-state` é folha de
//! propósito — *"não há relógio, não há ECS e não há UI"* — e depender da fundação para alcançar
//! um `struct` de números seria pagar o ECS inteiro por *plain data*.
//!
//! ⚠️ **O precedente é literal e está a dois passos daqui:** a `ph2d-stroke-width` existe pela
//! mesma razão e com o mesmo formato (o perfil de largura mora numa folha, o `VecStrokeProfile` o
//! embrulha, e a pose o carrega por si). Quando um canal precisa de estar no documento **e** numa
//! pose, a casa do tipo é uma folha; o componente é só o embrulho.
//!
//! # O que ficou para trás, de propósito
//!
//! O `VecFilter` — a pilha como **componente** — continua no `ph2d-ecs`, porque é lá que um
//! componente pertence. Esta crate não conhece `bevy_ecs`, e é isso que a torna utilizável dos
//! dois lados da fronteira que ela existe para atravessar.

mod kinds;
mod mix;
mod new;
mod op;

// ⚠️ A superfície é o DEGRAU e o catálogo que o descreve. `SPECS` e `BLANK` ficam internos:
// a porta pública da tabela é `FxOp::SPECS`, e o degrau em branco é detalhe do construtor.
pub use kinds::FALLOFF_MODES;
pub use mix::mix_stacks;
pub use op::{FxKindSpec, FxOp};
