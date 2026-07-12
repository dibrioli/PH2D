//! `FlipObjectRef` — a entidade que representa um objeto Flip na árvore do editor
//! (ADR-0110/0114).
//!
//! O documento Flip (`ph2d-flip`) é puro e não conhece ECS: ele é dono das
//! **camadas, frames e desenhos**. Esta componente é a única ponte, e carrega só
//! a identidade: o `FlipObjectId` do objeto lá dentro.
//!
//! Com ela, um objeto Flip é uma `Entity` como qualquer outra — então `ChildOf`,
//! `Children`, `RootOrder`, `Name`, `Visibility`, `Locked` e `GroupedChildren`
//! valem para ele exatamente como valem para um sprite ou um path vetorial. É o
//! que mantém a árvore única, com parentesco cruzado entre os quatro meios.
//!
//! Espelha `vec_path_ref.rs` (o vetor). O que ela NÃO faz: não põe geometria no
//! ECS. O `FlipDoc` continua dono, e a shell mantém o ciclo de vida (objeto
//! criado ⇒ entidade; objeto removido ⇒ entidade despawnada) num módulo só — ver
//! `shells/desktop/src/flip_entities.rs`.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// A entidade referencia o objeto `FlipObjectId` do `FlipDoc` da shell.
///
/// Um `u64` cru (e não um tipo do `ph2d-flip`) para manter `ph2d-ecs` sem
/// dependência do documento Flip — a direção da seta importa: a shell conhece os
/// dois, nenhum dos dois conhece o outro (idêntico ao `VecPathRef`).
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlipObjectRef(pub u64);

impl SimComponent for FlipObjectRef {}
