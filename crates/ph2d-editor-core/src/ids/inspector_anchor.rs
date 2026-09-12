//! **Os ids da §12 Sockets / Named Anchors** ([ADR-0072], spec
//! [`07_named_anchors.md`](../../../../docs/Sprite_projeto/07_named_anchors.md) §7.5).
//!
//! ⚠️ **Irmão de [`super::inspector`] por CAP de LOC** — mesmo padrão do
//! [`super::inspector_slice`] e do [`super::inspector_sampling`].
//!
//! # LISTA + EDITOR, e não uma ficha por âncora
//!
//! A spec §7.5 desenha uma **ficha por âncora**, empilhadas. Fichas empilhadas exigem ids
//! pré-alocados por LINHA: com o cap de 64 âncoras e ~12 campos cada, seriam **768 ids** — e um
//! painel de 320 px a rolar por 64 fichas abertas não é navegável.
//!
//! Aqui é uma **lista de nomes** (um id por linha, 64) mais **um editor** do que está
//! selecionado (12 ids). 76 no total, e o gesto é o de todo editor com listas longas.
//! ⚠️ A informação que a ficha da spec mostrava de relance — o que cada âncora **é** — não se
//! perde: cada linha traz o seu tipo derivado (`Socket` · `Slice` · `Region`) ao lado do nome.
//!
//! [ADR-0072]: ../../../../docs/architecture/decisions/0072-named-anchor-unification.md

use super::*;

/// §12 — o cabeçalho colapsável. Entra em [`super::LIVE_SECTIONS`] emparelhado com o ponto.
pub const INSP_LIVE_ANCHOR_SECTION: NodeId = hash_node_id("insp_live_anchor_section");
/// §12 — ponto de cor do cabeçalho.
pub const INSP_LIVE_ANCHOR_COLOR: NodeId = hash_node_id("insp_live_anchor_color");
