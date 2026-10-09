//! ⭐⭐ **MOTION ▸ EDIT** — o modo de um objecto Motion é editar o grafo dele (doc ObjectsAndComponents
//! 01 §13, F5): o seletor oferece Object · Edit (o Edit do Blender, sobre nós em vez de vértices).
//!
//! - **Entrar** liga a ferramenta Motion com este objecto como dono em edição — o editor de nós mostra
//!   só o grafo dele (`motion_objeto::view`).
//! - **A ferramenta que chega à mão sem o modo** (o botão Motion da barra de cima) pede o Edit do
//!   objecto em edição ([`ModeFamily::wants`]); sem objecto nenhum, fica na mão: o 1.º nó que o artista
//!   criar faz nascer um objecto Motion (a migração), e aí o modo segue.
//! - ⛔ **Sem conteúdo na Hierarquia** (dono, 09/10: *«não é legal listar todos os nodes na hierarchy;
//!   nodes apenas nos grafos»*): os nós vivem só no editor de nós. Na árvore ficam o objecto e os
//!   moldes (os filhos dele).

use ph2d_component_desc::ObjectKind;
use ph2d_editor_core::object_mode::ObjectMode;
use ph2d_editor_core::screens::hero::mode_drive::ModeFamily;
use ph2d_editor_core::{ToolId, ToolRegistry};

use crate::motion_state::MotionState;

const MOTION: &str = "motion";

/// ⭐ **A família**, construída em cada quadro sobre o estado do Motion.
pub struct Family<'a> {
    motion: &'a mut MotionState,
}

impl<'a> Family<'a> {
    /// A família sobre o estado do Motion deste quadro.
    #[must_use]
    pub fn new(motion: &'a mut MotionState) -> Self {
        Self { motion }
    }

    /// O `StableId` do objecto Motion que é a entidade `bits`.
    fn sid_of(&self, bits: u64) -> Option<u64> {
        self.motion
            .objeto
            .donos
            .iter()
            .find(|(_, d)| d.entity == bits)
            .map(|(s, _)| *s)
    }
}

/// A ferramenta Motion está na mão?
#[must_use]
pub fn holds_the_tool(tools: &ToolRegistry) -> bool {
    tools
        .active()
        .is_some_and(|t| t.id() == ToolId::new(MOTION))
}

impl ModeFamily for Family<'_> {
    fn modes(&self) -> &'static [(ObjectKind, ObjectMode)] {
        &[(ObjectKind::Motion, ObjectMode::Edit)]
    }
    fn holds(&mut self, mode: ObjectMode, entity: u64, tools: &mut ToolRegistry) -> bool {
        mode == ObjectMode::Edit && holds_the_tool(tools) && self.sid_of(entity).is_some()
    }
    fn enter(&mut self, mode: ObjectMode, entity: u64, tools: &mut ToolRegistry) -> bool {
        let Some(sid) = self.sid_of(entity).filter(|_| mode == ObjectMode::Edit) else {
            return false;
        };
        self.motion.objeto.editing = Some(sid);
        holds_the_tool(tools) || tools.set_active(&ToolId::new(MOTION))
    }
    fn leave(&mut self, _: ObjectMode, _: u64, tools: &mut ToolRegistry) {
        if holds_the_tool(tools) {
            tools.activate_default();
        }
    }
    /// A ferramenta chegou à mão sem o modo (o botão da barra): o Edit do objecto em edição.
    fn wants(&mut self, tools: &mut ToolRegistry) -> Option<(u64, ObjectMode)> {
        if !holds_the_tool(tools) {
            return None;
        }
        let sid = self.motion.objeto.editing?;
        let dono = self.motion.objeto.donos.get(&sid)?;
        Some((dono.entity, ObjectMode::Edit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::motion_objeto::{Dono, IDENTIDADE};

    fn motion_com_dois_nos() -> MotionState {
        let mut m = MotionState::new();
        let a = m.doc.graph.add_node("motion.grid");
        let b = m.doc.graph.add_node("motion.output");
        m.doc.set_owner(a, 5);
        m.doc.set_owner(b, 5);
        m.objeto.donos.insert(
            5,
            Dono {
                entity: 42,
                visible: true,
                afim: IDENTIDADE,
                pose: ph2d_editor_core::TransformSnapshot::IDENTITY,
            },
        );
        m
    }

    /// ⛔ **Os nós não são linhas da Hierarquia** (dono, 09/10: «nodes apenas nos grafos»): o objecto
    /// Motion não declara conteúdo. (Mutação: devolver o grupo «Nodes (n)» ⇒ RED.)
    #[test]
    fn the_motion_object_lists_no_nodes_in_the_hierarchy() {
        let mut m = motion_com_dois_nos();
        let mut tools = ToolRegistry::new();
        let mut out = ph2d_editor_core::hier_rows::ContentMap::default();
        Family::new(&mut m).content(&mut tools, &mut out);
        assert!(out.is_empty(), "o objecto Motion listou conteúdo: {out:?}");
    }

    /// Entrar no Edit de um objecto Motion faz dele o dono em edição; outra entidade recusa.
    #[test]
    fn entering_edit_picks_the_object_as_the_one_being_edited() {
        let mut m = motion_com_dois_nos();
        let mut tools = ToolRegistry::new();
        let mut f = Family::new(&mut m);
        assert!(
            !f.enter(ObjectMode::Edit, 99, &mut tools),
            "não é objecto Motion"
        );
        let _ = f.enter(ObjectMode::Edit, 42, &mut tools);
        assert_eq!(m.objeto.editing, Some(5));
    }
}
