//! ⭐⭐ **VECTOR ▸ EDIT** — o modo que esta família declara e como ele abre a ferramenta do vetor
//! SOBRE as formas da selecção (spec/06 F3; D6: *forma vetorial → Object · Edit, nós e alças*).
//!
//! - **Edit** = a ferramenta `vector` na mão com uma das que mexem na forma que já existe
//!   ([`DrawMode::EDIT_TOOLS`]), presa às formas do modo: só elas se agarram
//!   ([`ph2d_vec_scene::VecViewState::editing`], a porta única do clique). O painel só mostra as do
//!   modo em curso.
//! - **Object** = tudo o resto: as ferramentas que CRIAM (cada forma nova é um objecto) e as que
//!   trabalham sobre várias. Caneta, Lápis e Texto chegam pelo menu Add
//!   ([`crate::object_add::TOOLS`]), e o painel do vetor dá as restantes.
//! - ⭐ **Multi-objecto** ([`ModeFamily::joins`]): `Tab` com várias formas seleccionadas leva todas
//!   ao Edit — editar nós de várias formas (plano 25 §6, o laço, o soldar) já era do módulo.
//! - ⭐ **O ALVO** ([`EditTarget`]) mora no [`crate::state::VecState`], fora da `VecScene`: a cena
//!   entra no diff do undo, e entrar num modo seria um passo de histórico.
//!
//! **A porta antiga** ([`ModeFamily::wants`]): uma ferramenta do Edit que chega à mão sem o modo
//! (as cenas `PH2D_BUILD_SMOKE` que pedem o `Node`, o `Trim`, o `Fillet`…) pede o Edit sobre a forma
//! seleccionada; sem forma, espera por ela.

use ph2d_component_desc::ObjectKind;
use ph2d_editor_core::object_mode::{ActiveMode, ModeRequest, ObjectMode};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::screens::hero::mode_drive::ModeFamily;
use ph2d_editor_core::{ToolId, ToolRegistry};
use ph2d_tool_vector::{DrawMode, VectorTool};
use ph2d_vec_scene::VecPathId;

use crate::state::VecState;

/// O id da ferramenta do vetor no registo.
const VECTOR: &str = "vector";

/// ⭐ **O que o Edit tem em mãos.**
#[derive(Default)]
pub struct EditTarget {
    /// As formas do Edit — a do objecto activo à frente. Vazia em Object.
    pub paths: Vec<VecPathId>,
    /// O quadro anterior seguia um Edit do vetor.
    following: bool,
    /// A última ferramenta do Edit (o `Tab` volta a ela).
    last_tool: Option<DrawMode>,
    /// A ferramenta de criar que o menu Add pediu ([`crate::object_add::arm`]) — entra na mão
    /// quando a ferramenta `vector` lá chegar.
    pub armed: Option<DrawMode>,
}

/// ⭐ **A família**, construída em cada quadro com o estado do vetor que ela empresta.
pub struct Family<'a> {
    vec: &'a mut VecState,
}

impl<'a> Family<'a> {
    /// Empresta o estado deste quadro.
    pub fn new(vec: &'a mut VecState) -> Self {
        Self { vec }
    }

    fn path_of(&self, bits: u64) -> Option<VecPathId> {
        let (id, _) = self.vec.entities.iter().find(|(_, b)| **b == bits)?;
        Some(*id)
    }

    fn bits_of(&self, id: VecPathId) -> Option<u64> {
        self.vec.entities.get(&id).copied()
    }
}

/// A ferramenta do vetor, se é a que está na mão.
pub(crate) fn tool_mut(tools: &mut ToolRegistry) -> Option<&mut VectorTool> {
    tools
        .active_mut()
        .filter(|t| t.id() == ToolId::new(VECTOR))
        .and_then(|t| t.as_any_mut().downcast_mut::<VectorTool>())
}

/// O `DrawMode` na mão, se a ferramenta activa é a do vetor.
fn tool_in_hand(tools: &mut ToolRegistry) -> Option<DrawMode> {
    tool_mut(tools).map(|t| t.mode())
}

/// ⭐⭐ **A LEI do «tem em mãos»**, pura: a forma é o alvo e a ferramenta na mão é do Edit.
pub(crate) fn holds(mode: ObjectMode, is_target: bool, tool: Option<DrawMode>) -> bool {
    mode == ObjectMode::Edit && is_target && tool.is_some_and(|t| t.object_mode() == mode)
}

/// ⭐⭐ **A LEI da porta antiga**, pura: uma ferramenta do Edit na mão que o quadro anterior não
/// seguia pede o modo.
pub(crate) fn adopt(tool: Option<DrawMode>, following: bool) -> bool {
    !following && tool.is_some_and(|t| t.object_mode() == ObjectMode::Edit)
}

/// ⭐⭐ **A LEI do «a ferramenta segue o modo»**, pura: `true` = o Edit ACABOU neste quadro — a
/// ferramenta do Edit sai da mão. A que acabou de chegar por uma porta antiga espera pelo pedido
/// dela ([`adopt`]).
pub(crate) fn releases(ours_now: bool, following: bool) -> bool {
    !ours_now && following
}

impl ModeFamily for Family<'_> {
    fn modes(&self) -> &'static [(ObjectKind, ObjectMode)] {
        &[(ObjectKind::Vector, ObjectMode::Edit)]
    }

    fn holds(&mut self, mode: ObjectMode, entity: u64, tools: &mut ToolRegistry) -> bool {
        let is_target = self.path_of(entity).is_some()
            && self.path_of(entity) == self.vec.edit.paths.first().copied();
        holds(mode, is_target, tool_in_hand(tools))
    }

    fn enter(&mut self, mode: ObjectMode, entity: u64, tools: &mut ToolRegistry) -> bool {
        self.enter_with(mode, entity, &[], tools)
    }

    fn joins(&self, mode: ObjectMode) -> bool {
        mode == ObjectMode::Edit
    }

    fn enter_with(
        &mut self,
        mode: ObjectMode,
        entity: u64,
        joined: &[u64],
        tools: &mut ToolRegistry,
    ) -> bool {
        let Some(path) = self.path_of(entity).filter(|_| mode == ObjectMode::Edit) else {
            return false;
        };
        if tool_in_hand(tools).is_none() && !tools.set_active(&ToolId::new(VECTOR)) {
            return false;
        }
        let Some(tool) = tool_mut(tools) else {
            return false;
        };
        if tool.mode().object_mode() != ObjectMode::Edit {
            tool.set_mode(self.vec.edit.last_tool.unwrap_or(DrawMode::Node));
        }
        let mut paths = vec![path];
        paths.extend(joined.iter().filter_map(|b| self.path_of(*b)));
        self.vec.edit.paths = paths;
        true
    }

    fn leave(&mut self, _: ObjectMode, _: u64, tools: &mut ToolRegistry) {
        self.vec.edit.paths.clear();
        if tool_in_hand(tools).is_some() {
            tools.activate_default();
        }
    }

    fn follow(&mut self, current: Option<ActiveMode>, tools: &mut ToolRegistry) {
        if let Some(tool) = tool_mut(tools)
            && let Some(m) = self.vec.edit.armed.take()
        {
            tool.set_mode(m);
        }
        let ours = current
            .filter(|a| a.mode == ObjectMode::Edit)
            .and_then(|a| self.path_of(a.entity));
        let tool = tool_in_hand(tools);
        if let Some(path) = ours {
            if self.vec.edit.paths.first() != Some(&path) {
                self.vec.edit.paths = vec![path];
            }
            if let Some(t) = tool.filter(|t| t.object_mode() == ObjectMode::Edit) {
                self.vec.edit.last_tool = Some(t);
            }
        } else if releases(false, self.vec.edit.following) {
            self.vec.edit.paths.clear();
            // Só a ferramenta DO EDIT sai: a que o artista pegou no trilho (que acabou o modo) fica.
            if tool.is_some_and(|t| t.object_mode() == ObjectMode::Edit) {
                tools.activate_default();
            }
        }
        self.vec.edit.following = ours.is_some();
    }

    fn wants(&mut self, tools: &mut ToolRegistry) -> Option<(u64, ObjectMode)> {
        if !adopt(tool_in_hand(tools), self.vec.edit.following) {
            return None;
        }
        let path = self.vec.pen.selected_paths().last().copied()?;
        Some((self.bits_of(path)?, ObjectMode::Edit))
    }

    fn parts(&mut self, entity: u64) -> Option<Vec<u64>> {
        let target = self.vec.edit.paths.first().copied()?;
        (self.path_of(entity) == Some(target)).then(|| {
            let paths = &self.vec.edit.paths;
            paths.iter().filter_map(|p| self.bits_of(*p)).collect()
        })
    }
}

/// ⭐ **O smoke do modo** — `PH2D_OBJECT_MODE_SMOKE=6` acrescenta um rectângulo e uma elipse pelo
/// menu Add (nascem em Object, escolha do dono), entra no Edit da elipse e abre o seletor *Mode*: os
/// nós são só dela, o rectângulo à volta fica intocado. Corre uma vez.
pub fn smoke_step(vec: &VecState, hero: &mut HeroScreen) -> Option<ModeRequest> {
    use std::sync::atomic::{AtomicU8, Ordering};
    // 0 = por ler · 1 = o rectângulo · 2 = a elipse · 3 = o Edit · 4 = o seletor · 9 = feito.
    static STAGE: AtomicU8 = AtomicU8::new(0);
    let stage = match STAGE.load(Ordering::Relaxed) {
        0 => {
            let want = match std::env::var("PH2D_OBJECT_MODE_SMOKE").as_deref() {
                Ok("6") => 1,
                _ => 9,
            };
            STAGE.store(want, Ordering::Relaxed);
            want
        }
        s => s,
    };
    let pick = |hero: &mut HeroScreen, entry: ph2d_editor_core::object_add::AddEntry, next| {
        hero.store.set_command_pick(entry.id());
        STAGE.store(next, Ordering::Relaxed);
    };
    match stage {
        1 => pick(hero, crate::object_add::RECTANGLE, 2),
        2 if vec.entities.len() == 1 => pick(hero, crate::object_add::ELLIPSE, 3),
        3 if vec.entities.len() == 2 && hero.gizmo.selection.is_some() => {
            STAGE.store(4, Ordering::Relaxed);
            return Some(ModeRequest::Enter(ObjectMode::Edit));
        }
        4 if hero.gizmo.mode.current() == ObjectMode::Edit => {
            // O chip só se abre depois de pintado (o pulldown ancora-se no rect dele).
            let chip = ph2d_editor_core::ids::area_menu_button(0);
            if hero.hit_index.rect_for(chip).is_some() {
                STAGE.store(9, Ordering::Relaxed);
                hero.apply_event(ph2d_editor_core::interaction::WidgetEvent::Click(chip));
            }
        }
        _ => {}
    }
    None
}

#[cfg(test)]
#[path = "vector_mode_tests.rs"]
mod tests;
