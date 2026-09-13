//! **O `HeroLive`** — o estado por-quadro da ponte viva do editor (o mapa entidade↔nó e as duas
//! leituras da árvore: a do prólogo, que o painel publica, e a da projeção de z). Irmão de
//! `app_state.rs` pelo tecto de 600 LOC da shell.
//!
//! Corte mecânico: a struct saiu inteira, verbatim. O caminho `crate::HeroLive` não muda — o
//! `app_state.rs` re-exporta-a e o `main.rs` re-exporta o `app_state`.

use super::*;

/// Per-frame state owned by the live editor bridge (ADR-0025 M14.4a).
pub(crate) struct HeroLive {
    pub(crate) bridge: hero_bridge::EntityNodeMap,
    pub(crate) walk_state: HierarchyWalkState,
    /// Scratch buffer for `build_hierarchy_snapshot`'s DFS stack.
    /// Preserved across frames so HR-3 zero-alloc invariant holds.
    pub(crate) walk_scratch: Vec<(ph2d_ecs::Entity, u8, Option<ph2d_ecs::Entity>)>,
    /// Reused per-frame snapshot. `build_hierarchy_snapshot` clears
    /// the inner Vec without releasing capacity.
    pub(crate) snapshot: HierarchySnapshot,
    /// **A árvore relida no momento da projeção de z** (ADR-0110, BUGS #15).
    ///
    /// A trinca acima é a PUBLICAÇÃO do painel, feita no prólogo do frame — ou seja,
    /// **antes** de `vec_entities::sync` dar entidade à forma recém-criada. A ordem de z
    /// é a projeção da árvore e precisa lê-la **depois** do `sync`; então ela lê de novo,
    /// com a MESMA função (`build_hierarchy_snapshot`) e scratch próprio.
    ///
    /// O scratch é próprio, e isso não é arrumação: sobrescrever a trinca do painel no
    /// meio do frame descasaria a `bridge`, que já foi sincronizada com o snapshot do
    /// prólogo. Duas leituras da mesma árvore, em instantes diferentes — não duas fontes.
    pub(crate) z_walk_state: HierarchyWalkState,
    pub(crate) z_walk_scratch: Vec<(ph2d_ecs::Entity, u8, Option<ph2d_ecs::Entity>)>,
    pub(crate) z_snapshot: HierarchySnapshot,
}
