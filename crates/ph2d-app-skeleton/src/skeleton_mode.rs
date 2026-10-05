//! ⭐⭐ **SKELETON ▸ EDIT · POSE** — os modos do objecto esqueleto (A14, a *Armature* do Blender;
//! plano `docs/Skeleton/05_plano_o_esqueleto_e_um_objecto.md`).
//!
//! - **Edit** = a ferramenta de osso na mão com o verbo *Create* (a pose de repouso: ossos novos,
//!   pontas); **Pose** = com *Transform* ou *Weight* (posar, IK, corrigir o peso). **Object** = a
//!   ferramenta sai da mão: o gizmo move o esqueleto inteiro.
//! - ⭐ **O ALVO** (`SkeletonState::target`) é escrito AQUI e só aqui.
//! - ⭐ **As partes são os ossos** ([`ModeFamily::parts`]): seleccionar um osso do esqueleto em Edit ou
//!   Pose não tropeça no cadeado; um osso seleccionado responde pelo esqueleto dono
//!   ([`ModeFamily::owner_of`]), e o `Tab` sobre ele entra no Edit do esqueleto.
//! - ⭐ **O modo é a verdade do verbo:** a ferramenta que chega à mão por outra porta (um segmento do
//!   painel Bones, uma cena) pede o modo do verbo dela ([`adopt`]).

use ph2d_component_desc::ObjectKind;
use ph2d_ecs::{Entity, World};
use ph2d_editor_core::object_mode::{ActiveMode, ObjectMode};
use ph2d_editor_core::screens::hero::mode_drive::ModeFamily;
use ph2d_editor_core::tool::ToolRegistry;
use ph2d_tool_bone::BoneAction;

use crate::bone_bridge;
use crate::state::SkeletonState;

/// ⭐⭐ **A LEI do verbo ↔ modo**, pura: *Create* é do Edit; *Transform* e *Weight* são do Pose.
#[must_use]
pub fn mode_of(action: BoneAction) -> ObjectMode {
    match action {
        BoneAction::Create => ObjectMode::Edit,
        BoneAction::Transform | BoneAction::Weight => ObjectMode::Pose,
    }
}

/// O verbo com que se ENTRA em `mode` (o Pose volta ao último que se usou nele).
#[must_use]
pub fn verb_of(mode: ObjectMode, last_pose: Option<BoneAction>) -> Option<BoneAction> {
    match mode {
        ObjectMode::Edit => Some(BoneAction::Create),
        ObjectMode::Pose => Some(last_pose.unwrap_or(BoneAction::Transform)),
        _ => None,
    }
}

/// ⭐⭐ **A LEI do «tem em mãos»**, pura: o esqueleto do modo é o alvo e o verbo na mão é do modo.
#[must_use]
pub fn holds(mode: ObjectMode, is_target: bool, tool: Option<BoneAction>) -> bool {
    matches!(mode, ObjectMode::Edit | ObjectMode::Pose)
        && is_target
        && tool.is_some_and(|a| mode_of(a) == mode)
}

/// ⭐⭐ **A LEI da porta antiga**, pura: o verbo na mão cujo modo o quadro anterior não seguia
/// pede-o.
#[must_use]
pub fn adopt(tool: Option<BoneAction>, following: Option<ObjectMode>) -> Option<ObjectMode> {
    let want = mode_of(tool?);
    (following != Some(want)).then_some(want)
}

/// ⭐ **A família**, construída por quadro com o que ela empresta: o estado do esqueleto, o mundo e
/// a selecção primária (é ela que escolhe o esqueleto quando há vários).
pub struct Family<'a> {
    state: &'a mut SkeletonState,
    world: &'a World,
    selected: Option<u64>,
}

impl<'a> Family<'a> {
    /// Empresta o estado, o mundo e a selecção deste quadro.
    pub fn new(state: &'a mut SkeletonState, world: &'a World, selected: Option<u64>) -> Self {
        Self {
            state,
            world,
            selected,
        }
    }

    fn is_skeleton(&self, bits: u64) -> bool {
        Entity::try_from_bits(bits)
            .is_some_and(|e| self.world.get::<ph2d_skeleton_ecs::Skeleton>(e).is_some())
    }

    fn skeleton_of(&self, bits: u64) -> Option<u64> {
        let e = Entity::try_from_bits(bits)?;
        self.world.get_entity(e).ok()?;
        ph2d_skeleton_ecs::skeleton_of(self.world, e).map(Entity::to_bits)
    }

    /// O esqueleto que a porta antiga abre: o da selecção, senão o último alvo, senão o primeiro.
    fn chosen(&self) -> Option<u64> {
        self.selected
            .and_then(|b| self.skeleton_of(b))
            .or_else(|| self.state.target.filter(|b| self.is_skeleton(*b)))
            .or_else(|| {
                self.world
                    .iter_entities()
                    .filter(|er| er.contains::<ph2d_skeleton_ecs::Skeleton>())
                    .map(|er| er.id().to_bits())
                    .min()
            })
    }
}

fn verb_in_hand(tools: &mut ToolRegistry) -> Option<BoneAction> {
    bone_bridge::in_hand(tools).then(|| bone_bridge::config(tools).action)
}

impl ModeFamily for Family<'_> {
    fn modes(&self) -> &'static [(ObjectKind, ObjectMode)] {
        &[
            (ObjectKind::Skeleton, ObjectMode::Edit),
            (ObjectKind::Skeleton, ObjectMode::Pose),
        ]
    }

    fn holds(&mut self, mode: ObjectMode, entity: u64, tools: &mut ToolRegistry) -> bool {
        holds(mode, self.state.target == Some(entity), verb_in_hand(tools))
    }

    fn enter(&mut self, mode: ObjectMode, entity: u64, tools: &mut ToolRegistry) -> bool {
        if !self.is_skeleton(entity) {
            return false;
        }
        let Some(verbo) = verb_of(mode, self.state.last_pose) else {
            return false;
        };
        // A ferramenta já na mão com um verbo deste modo fica com ele (o segmento que a trouxe).
        let na_mao = verb_in_hand(tools).filter(|a| mode_of(*a) == mode);
        if !bone_bridge::arm(tools, na_mao.unwrap_or(verbo)) {
            return false;
        }
        self.state.target = Some(entity);
        if self.state.born == Some(entity) {
            self.state.born = None;
        }
        true
    }

    fn leave(&mut self, _: ObjectMode, _: u64, tools: &mut ToolRegistry) {
        self.state.target = None;
        if bone_bridge::in_hand(tools) {
            tools.activate_default();
        }
    }

    fn follow(&mut self, current: Option<ActiveMode>, tools: &mut ToolRegistry) {
        let ours = current
            .filter(|a| matches!(a.mode, ObjectMode::Edit | ObjectMode::Pose))
            .filter(|a| self.is_skeleton(a.entity));
        let verbo = verb_in_hand(tools);
        if let Some(a) = ours {
            self.state.target = Some(a.entity);
            if let Some(v) = verbo.filter(|v| mode_of(*v) == ObjectMode::Pose) {
                self.state.last_pose = Some(v);
            }
        } else if self.state.following.is_some() {
            // O modo do esqueleto ACABOU: a ferramenta sai da mão (Object = o gizmo).
            self.state.target = None;
            if verbo.is_some() {
                tools.activate_default();
            }
        }
        self.state.following = ours.map(|a| a.mode);
    }

    fn wants(&mut self, tools: &mut ToolRegistry) -> Option<(u64, ObjectMode)> {
        if let Some(born) = self.state.born {
            if self.is_skeleton(born) {
                return Some((born, ObjectMode::Edit));
            }
            self.state.born = None;
        }
        let want = adopt(verb_in_hand(tools), self.state.following)?;
        Some((self.chosen()?, want))
    }

    fn parts(&mut self, entity: u64) -> Option<Vec<u64>> {
        let e = Entity::try_from_bits(entity)?;
        self.is_skeleton(entity).then(|| {
            ph2d_skeleton_ecs::bones_of(self.world, e)
                .into_iter()
                .map(Entity::to_bits)
                .collect()
        })
    }

    fn owner_of(&mut self, bits: u64) -> Option<u64> {
        self.skeleton_of(bits).filter(|s| *s != bits)
    }
}

#[cfg(test)]
#[path = "skeleton_mode_tests.rs"]
mod tests;
