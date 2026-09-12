//! **Os ids da §11 Animation** (spec
//! [`08_animation_inline.md`](../../../../docs/Sprite_projeto/08_animation_inline.md) §8.7).
//!
//! ⚠️ **Irmão de [`super::inspector`] por CAP de LOC** — mesmo padrão do
//! [`super::inspector_anchor`] e do [`super::inspector_slice`].
//!
//! # ⚠️ A linha da lista É a animação ATUAL — e isso difere da §12 de propósito
//!
//! Na §12 Sockets, clicar numa linha muda **só** a ficha aberta e **não** vai ao barramento: qual
//! âncora se edita é um facto da UI. Aqui é o contrário, e a diferença é do domínio: numa
//! biblioteca de animações, *a que se está a ver* e *a que toca* serem a mesma coisa é o que o
//! artista espera (é o que o `AnimationPlayer` do Godot faz), e separá-las obrigaria a um segundo
//! controlo — um seletor com as mesmas entradas da lista logo abaixo dele.
//!
//! ⇒ Clicar numa linha **é** uma edição da cena (escreve `SpriteAnimator::current`), e por isso
//! entra no undo. Isto poupa 65 ids e a máquina de popover de um seletor que duplicaria a lista.
//!
//! **A POSIÇÃO NO ARRAY É A TAG** nos segmentados de direção e de loop — o despacho deriva-a de
//! `position(|&o| o == id)`. ⛔ Nunca reordene esses arrays.

use super::*;

/// §11 Animation — o cabeçalho colapsável. Entra em [`super::LIVE_SECTIONS`] com o ponto de cor.
pub const INSP_LIVE_ANIM_SECTION: NodeId = hash_node_id("insp_live_anim_section");
/// §11 Animation — ponto de cor do cabeçalho.
pub const INSP_LIVE_ANIM_COLOR: NodeId = hash_node_id("insp_live_anim_color");

// ── O TOCADOR (o estado, e não a autoria) ────────────────────────────────────────────────────
