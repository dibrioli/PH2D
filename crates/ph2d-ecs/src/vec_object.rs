//! `VecObject` — o **OBJECTO VETORIAL**, o contentor das formas (spec/06 F3 ▸ Vector; escolha do
//! dono, 04/10: *«as shapes são filhas do objeto vetorial vazio»*).
//!
//! Uma entidade SEM geometria própria (nem `VecPathRef`) cujas filhas `ChildOf` são as formas. É o
//! que a Hierarquia mostra como «Vector», o que o gizmo move em Object e o alvo do Edit. Toda forma
//! vive dentro de um: a solta ganha o seu (`ph2d_vec_entities::entities::object`).

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// O marcador do `ObjectKind::Vector` — tamanho zero de propósito: as formas são as filhas, e o documento continua na `VecScene`.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VecObject;

impl SimComponent for VecObject {}
