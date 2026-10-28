//! ⭐ O `DocumentSurface` da família: o hero dá a área central à aba activa e entrega-lhe rato, roda,
//! teclado e composição. Os comandos da família («New Kanban», «Kanbans») entram no menu File com
//! qualquer aba activa.

use crate::docs::{HOME, KanbanDocs};
use crate::home::Tile;
use ph2d_a11y::NodeId;
use ph2d_editor_core::doc_surface::{
    Compose, DocCommand, DocMenu, DocTab, DocumentSurface, KeyInput, KeyOutcome, PointerInput,
    PointerLook,
};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerKind};
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_tool_registry::hash_node_id;
use ph2d_vector::VectorScene;
use std::any::Any;

pub const CMD_NEW: NodeId = hash_node_id("kanban_cmd_new");
pub const CMD_HOME: NodeId = hash_node_id("kanban_cmd_home");

impl DocumentSurface for KanbanDocs {
    fn tabs(&self) -> Vec<DocTab> {
        let set = self.set.borrow();
        let home = self.home.as_ref().map(|_| DocTab {
            id: HOME,
            title: tr("kanban.tab.home").to_owned(),
            dirty: false,
        });
        home.into_iter()
            .chain(self.tabs.iter().map(|t| {
                DocTab {
                    id: t.id(),
                    title: set
                        .get(t.view.kanban)
                        .map_or_else(String::new, |k| k.header.name.clone()),
                    dirty: false,
                }
            }))
            .collect()
    }

    fn paint(
        &mut self,
        id: u64,
        area: Rect,
        scene: &mut VectorScene,
        text: &mut TextSystem,
        theme: &Theme,
    ) {
        if id == HOME {
            let names = self.names();
            if let Some(h) = self.home.as_mut() {
                crate::home::paint(h, &names, area, scene, text, *theme);
            }
            return;
        }
        self.paint_tab(id, area, scene, text, *theme);
    }

    fn chrome_over(&mut self, _id: u64, over: Option<Rect>) {
        self.over = over;
    }

    fn pointer(
        &mut self,
        id: u64,
        input: PointerInput<'_>,
        kind: PointerKind,
        button: PointerButton,
        x: f32,
        y: f32,
    ) -> bool {
        if id == HOME {
            return self.home_pointer(kind, x, y);
        }
        match kind {
            PointerKind::Down => crate::pointer::down(self, id, input.text, button, x, y),
            PointerKind::Up => crate::pointer::up(self, id, input.text, input.now_ns, x, y),
            PointerKind::Move => crate::pointer::moved(self, id, input.text, x, y),
        }
    }

    fn pointer_move(&mut self, id: u64, input: PointerInput<'_>, x: f32, y: f32) -> bool {
        if id == HOME {
            return self.home_pointer(PointerKind::Move, x, y);
        }
        crate::pointer::moved(self, id, input.text, x, y)
    }

    fn wheel(&mut self, id: u64, x: f32, y: f32, dy: f32, shift: bool) -> bool {
        id != HOME && crate::pointer::wheel(self, id, x, y, dy, shift)
    }

    fn key(&mut self, id: u64, _ts: &mut TextSystem, input: KeyInput<'_>) -> KeyOutcome {
        if id == HOME {
            return KeyOutcome::default();
        }
        crate::keys::key(self, id, input)
    }

    fn compose(&mut self, id: u64, c: Compose<'_>) -> bool {
        let Some(f) = self.tab_mut(id).and_then(|t| t.field.as_mut()) else {
            return false;
        };
        match c {
            Compose::Commit(ch) => {
                f.insert(ch.encode_utf8(&mut [0; 4]));
                true
            }
            Compose::Preedit(..) | Compose::DeadKey(_) => false,
        }
    }

    fn close(&mut self, id: u64) {
        if id == HOME {
            self.home = None;
            return;
        }
        self.leave(id);
        self.tabs.retain(|t| t.id() != id);
    }

    fn undo(&mut self, id: u64, redo: bool) -> bool {
        self.with_doc(id, |tab, doc| {
            tab.field = None;
            doc.undo(redo)
        })
        .unwrap_or(false)
    }

    fn a11y(&self, id: u64) -> Vec<(NodeId, ph2d_a11y::Node)> {
        let title = self
            .tabs()
            .into_iter()
            .find(|t| t.id == id)
            .map(|t| t.title)
            .unwrap_or_default();
        let a = self.area;
        let node = ph2d_a11y::NodeBuilder::new(ph2d_a11y::Role::Document)
            .label(title)
            .bounds(
                f64::from(a.x),
                f64::from(a.y),
                f64::from(a.w),
                f64::from(a.h),
            )
            .build();
        vec![(NodeId(crate::A11Y_SALT ^ id), node)]
    }

    fn take_focus_request(&mut self) -> Option<u64> {
        self.focus.take()
    }

    fn family_commands(&self) -> Vec<DocCommand> {
        vec![
            DocCommand {
                id: CMD_NEW,
                label: tr("kanban.cmd.new").to_owned(),
                menu: DocMenu::File,
            },
            DocCommand {
                id: CMD_HOME,
                label: tr("kanban.cmd.home").to_owned(),
                menu: DocMenu::File,
            },
        ]
    }

    fn family_command(&mut self, cmd: NodeId) -> bool {
        if cmd == CMD_NEW {
            self.create();
            true
        } else if cmd == CMD_HOME {
            self.open_home();
            true
        } else {
            false
        }
    }

    fn pointer_look(&self, id: u64, x: f32, y: f32) -> Option<PointerLook> {
        if !self.area.contains(x, y) {
            return None;
        }
        if id == HOME {
            let over = self.home.as_ref().and_then(|h| crate::home::hit(h, x, y));
            return Some(if over.is_some() {
                PointerLook::Hand
            } else {
                PointerLook::Arrow
            });
        }
        let t = self.tabs.iter().find(|t| t.id() == id)?;
        Some(match t.hover {
            Some(crate::tab::Hit::Field) => PointerLook::Text,
            Some(crate::tab::Hit::Board) | None => PointerLook::Arrow,
            Some(_) => PointerLook::Hand,
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl KanbanDocs {
    fn home_pointer(&mut self, kind: PointerKind, x: f32, y: f32) -> bool {
        let Some(h) = self.home.as_mut() else {
            return false;
        };
        let over = crate::home::hit(h, x, y);
        match kind {
            PointerKind::Move => h.hover = over,
            PointerKind::Down => h.pressed = over,
            PointerKind::Up => {
                let pressed = h.pressed.take();
                if pressed.is_some() && pressed == over {
                    match over {
                        Some(Tile::Open(k)) => self.open(k),
                        Some(Tile::Create) => {
                            self.create();
                        }
                        None => {}
                    }
                }
            }
        }
        self.area.contains(x, y)
    }
}
