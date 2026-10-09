//! ⭐⭐ **MOTION ▸ EDIT** — o modo de um objecto Motion é editar o grafo dele (doc ObjectsAndComponents
//! 01 §13, F5): o seletor oferece Object · Edit (o Edit do Blender, sobre nós em vez de vértices).
//!
//! - **Entrar** liga a ferramenta Motion com este objecto como dono em edição — o editor de nós mostra
//!   só o grafo dele (`motion_objeto::view`).
//! - **A ferramenta que chega à mão sem o modo** (o botão Motion da barra de cima) pede o Edit do
//!   objecto em edição ([`ModeFamily::wants`]); sem objecto nenhum, fica na mão: o 1.º nó que o artista
//!   criar faz nascer um objecto Motion (a migração), e aí o modo segue.
//! - **O conteúdo** é a linha «Nodes (n)» sob o objecto, com os nós dele; o clique entra no Edit.

use ph2d_component_desc::ObjectKind;
use ph2d_editor_core::hier_rows::{ContentGroup, ContentItem, ContentMap, ContentOpens};
use ph2d_editor_core::icons::IconId;
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
    fn content(&mut self, _: &mut ToolRegistry, out: &mut ContentMap) {
        for (sid, dono) in &self.motion.objeto.donos {
            let items: Vec<ContentItem> = self
                .motion
                .doc
                .nodes_of(*sid)
                .into_iter()
                .filter_map(|n| {
                    let inst = self.motion.doc.graph.node(n)?;
                    let ui = self.motion.registry.ui_manifest(inst.type_id());
                    Some(ContentItem {
                        key: u64::from(n.0),
                        name: ph2d_node_registry::card_title(
                            self.motion.doc.graph.label(n),
                            ui.map(|u| ph2d_i18n::tr(u.display_key)),
                            &inst.type_name,
                        ),
                        depth: 0,
                        active: false,
                    })
                })
                .collect();
            if items.is_empty() {
                continue;
            }
            out.entry(dono.entity).or_default().push(ContentGroup {
                label_key: "hier_rows.content.nodes",
                icon: IconId::MotionNodes,
                item_icon: IconId::MotionNodes,
                opens: ContentOpens::Mode(ObjectMode::Edit),
                items,
            });
        }
    }
    fn open_content(
        &mut self,
        mode: ObjectMode,
        entity: u64,
        _key: u64,
        tools: &mut ToolRegistry,
    ) -> bool {
        self.enter(mode, entity, tools)
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

    /// ⭐ A linha «Nodes (n)» sob o objecto Motion, com os nós DELE, abre o Edit.
    #[test]
    fn the_motion_object_declares_its_nodes_as_content() {
        let mut m = motion_com_dois_nos();
        let mut tools = ToolRegistry::new();
        let mut out = ContentMap::default();
        Family::new(&mut m).content(&mut tools, &mut out);
        let grupos = out.get(&42).expect("o objecto 42 tem conteúdo");
        assert_eq!(grupos.len(), 1);
        assert_eq!(grupos[0].label_key, "hier_rows.content.nodes");
        assert_eq!(grupos[0].items.len(), 2);
        assert_eq!(grupos[0].opens, ContentOpens::Mode(ObjectMode::Edit));
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
