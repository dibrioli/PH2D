//! `Sculpt3dPieceRef` — a entidade que representa uma **peça da escultura** na árvore do
//! editor (ADR-0150), espelhando [`crate::flip_object_ref`] e [`crate::vec_path_ref`].
//!
//! A cena 3D (`Sculpt3dScene`, na shell) é dona da **geometria**: a pilha de multires, a pose,
//! o preview do pincel. Esta componente é a única ponte, e carrega só a identidade: o
//! `ObjectId` da peça lá dentro.
//!
//! Com ela, uma peça esculpida é uma `Entity` como qualquer outra — então `Name`, `ChildOf`,
//! `Children`, `RootOrder`, `Visibility`, `Locked` e `GroupedChildren` valem para ela
//! exactamente como valem para um sprite, um path vetorial ou um objeto Flip. É o que faz a
//! Hierarquia **listá-la** e os verbos de linha (apagar, duplicar, renomear) alcançarem-na sem
//! uma lei nova.
//!
//! # ⛔ O que ela NÃO faz
//!
//! Não põe geometria no ECS. A cena continua dona, e a shell mantém o ciclo de vida — peça
//! criada ⇒ entidade; peça removida ⇒ entidade despawnada; entidade apagada pela Hierarquia ⇒
//! peça removida — num módulo só: `shells/desktop/src/sculpt3d_entities.rs`.
//!
//! ⚠️ **Um `u32` cru, e não um tipo do módulo de escultura**, para manter `ph2d-ecs` sem
//! dependência da shell: a direcção da seta importa — a shell conhece os dois, nenhum dos dois
//! conhece o outro (idêntico ao `FlipObjectRef`).

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// A entidade referencia a peça `ObjectId` da cena 3D da shell.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sculpt3dPieceRef(pub u32);

impl SimComponent for Sculpt3dPieceRef {}
