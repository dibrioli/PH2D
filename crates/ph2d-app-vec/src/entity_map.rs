//! ⭐⭐ **O MAPA `caminho ⟺ entidade`, e só ele** — a peça de `vec_entities` que 18 ficheiros
//! desta família precisavam e que os prendia todos à shell.
//!
//! # Porque só o TIPO veio, e não o módulo
//!
//! Medido na Fase B: dos 30 ficheiros movidos que referiam `crate::vec_entities`, **20 usavam
//! apenas o `VecEntityMap`** — um alias de UMA linha sobre tipos de crates — e os outros 10 usavam
//! `sync`, que está preso a três predicados de **outras** famílias (`name_unique::unique_name`,
//! `render_loop::off_canvas::is_off_canvas`, `morph_set::is_set_member`).
//!
//! ⇒ trazer o alias custou uma linha e destravou 18 ficheiros; trazer o `sync` exigiria tomar
//! código de outra família, que o [HOWTO §1.2] proíbe (*duas famílias que partilham código
//! partilham uma FOLHA, nunca uma delas à outra*). ⚠️ **O `name_unique` é o MESMO bloqueador que a
//! `line/app-physics` nomeou** como folha partilhada de linha própria — esta linha confirma-o
//! independentemente, a partir de outra família.
//!
//! [HOWTO §1.2]: ../../../docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md

use std::collections::BTreeMap;

use ph2d_vec_scene::VecPathId;

/// De um caminho do documento para os bits da entidade que o representa na cena.
///
/// ⚠️ **`BTreeMap`, nunca `HashMap`** — a ordem de iteração entra em somas de `f32` e no snapshot
/// do undo; um mapa com ordem de hash faz o mesmo estado dar bytes diferentes (CLAUDE.md §5.1,
/// a espinha do determinismo).
pub type VecEntityMap = BTreeMap<VecPathId, u64>;

/// A profundidade máxima que um passeio pela árvore percorre antes de desistir.
pub const MAX_DEPTH: usize = 64;
