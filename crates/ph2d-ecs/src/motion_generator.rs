//! ⭐⭐ **O objecto MOTION** — o marcador que faz de uma entidade um gerador do Motion Nodes (doc
//! ObjectsAndComponents 01 §13, F5; escolha do dono 9.3: *«Motion vira objecto na árvore, com o que
//! cria como filhos; pode haver vários»*).
//!
//! Sem dados: os nós vivem no `MotionDoc` (um documento, cada nó com o `StableId` deste objecto como
//! dono), o lugar é o `Transform` dele (a origem do que os sinks geram) e os moldes são os filhos.
//! Registado ⇒ grava-se com a cena; o tipo `ObjectKind::Motion` é derivado DELE.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// A entidade é um objecto Motion.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MotionGenerator;

impl SimComponent for MotionGenerator {}
