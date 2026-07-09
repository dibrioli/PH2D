//! `VecPathRef` — a entidade que representa um path vetorial na árvore do editor
//! (ADR-0110).
//!
//! O documento vetorial (`ph2d-vec-scene`) é puro e não conhece ECS: ele é dono da
//! **geometria e do estilo**. Esta componente é a única ponte, e carrega só a
//! identidade: o `VecPathId` do path lá dentro.
//!
//! Com ela, um path vetorial é uma `Entity` como qualquer outra — então
//! `ChildOf`, `Children`, `RootOrder`, `Name`, `Visibility`, `Locked` e
//! `GroupedChildren` valem para ele exatamente como valem para um sprite. É o que
//! torna possível uma árvore só, com **parentesco cruzado** entre tipos, e um grupo
//! que é apenas uma entidade comum com filhos.
//!
//! O que ela NÃO faz: não põe geometria no ECS. O `VecScene` continua sendo o dono,
//! e a shell mantém o ciclo de vida (path criado ⇒ entidade; path removido ⇒
//! entidade despawnada) num módulo só — ver `shells/desktop/src/vec_entities.rs`.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// A entidade referencia o path `VecPathId` do `VecScene` da shell.
///
/// Um `u64` cru (e não um tipo do `ph2d-vec-scene`) para manter `ph2d-ecs` sem
/// dependência do documento vetorial — a direção da seta importa: a shell conhece
/// os dois, nenhum dos dois conhece o outro.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VecPathRef(pub u64);

impl SimComponent for VecPathRef {}
