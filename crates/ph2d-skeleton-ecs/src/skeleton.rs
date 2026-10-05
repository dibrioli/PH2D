//! ⭐⭐ **O ESQUELETO como OBJECTO** (A14, plano `docs/Skeleton/05_plano_o_esqueleto_e_um_objecto.md`;
//! a *Armature* do Blender).
//!
//! Marcador VAZIO: o tipo lê-se por presença (`ObjectKind::Skeleton`). Os ossos-raiz são filhos
//! (`ChildOf`) da entidade dele, e o `Transform` dela move o esqueleto inteiro — o mundo de um osso
//! já compõe a cadeia toda. Uma raiz continua a ser *«sem pai, ou pai que não é osso»*.

use bevy_ecs::component::Component;
use ph2d_ecs::SimComponent;
use serde::{Deserialize, Serialize};

/// O marcador do objecto esqueleto.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Skeleton;

impl SimComponent for Skeleton {}
