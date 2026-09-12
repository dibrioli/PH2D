//! ⭐⭐ **A forma vetorial é uma ENTIDADE da árvore do editor, com pose** (ADR-0110 + ADR-0111).
//!
//! O documento é dono da **geometria**; a entidade é dona da **identidade e do lugar na árvore** —
//! nome, visibilidade, trava, pai, ordem — e do **`Transform`** que a leva ao mundo. Os dois ADR
//! dizem a mesma frase de dois lados, e é por isso que são uma crate só.
//!
//! # ⛔ Por que isto é uma FOLHA, e não parte da `ph2d-app-vec`
//!
//! Três famílias consomem esta ponte — o `vec` em 32 ficheiros de produto, a `motion` em 3 e a
//! `flip` em 4 (medido no `main` de 2026-09-12) — e uma peça partilhada por duas famílias é uma
//! folha, nunca a casa de uma delas: pô-la dentro da crate do vetor faria a `motion` e a `flip`
//! dependerem do vetor inteiro, que é exactamente o acoplamento que o ADR-0075 existe para
//! impedir (HOWTO §1.2).
//!
//! ⭐ **O [`entity_map`] veio de volta da [`ph2d-app-vec`]**, onde tinha ficado na Fase B daquela
//! linha por ser o pedaço de que 18 ficheiros dela precisavam. Ele é a mesma peça partilhada, um
//! degrau acima: a `ph2d-app-vec` re-exporta-o hoje e os 18 ficheiros não mudaram uma linha.
//!
//! # ⚠️ O CICLO é real, e é por isso que os três chegam juntos
//!
//! ```text
//! entities ──is_set_member──▶ morph_set ──sync, group_entities──▶ entities
//! transform ──VecEntityMap──▶ entities
//! ```
//!
//! Os três saem juntos ou nenhum sai. ⭐ E o ciclo **não é um defeito de arrumação**: um conjunto
//! de estados do morph *é* uma manipulação da hierarquia (nasce um pai, os filhos reparentam-se e
//! escondem-se), logo é o mesmo assunto visto de outro ângulo.
//!
//! # ⚠️ Onde estão os gates
//!
//! Os que medem a LEI vieram (`entities_group_tests`, `zorder_arrange_tests`, `transform_tests`,
//! `transform_reparent_tests`, e os inline da selecção). Os **oito** que medem a lei *através da
//! shell* — o `input_dispatch`, o `hero_bridge`, o `undo`, o `vec_tree_settle`, o `morph_live`, o
//! `profile_live` — ficaram lá, porque em qualquer outra crate mediriam zero (HOWTO §1.2: *os
//! testes seguem o SUJEITO, não o ficheiro*).
//!
//! [`ph2d-app-vec`]: https://docs.rs/ph2d-app-vec

pub mod entities;
pub mod entity_map;
pub mod morph_set;
pub mod transform;

pub use entities::zorder;
