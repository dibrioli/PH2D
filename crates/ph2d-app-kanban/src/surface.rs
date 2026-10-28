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
/// W6 — «New Kanban from…» (um Kanban a partir de outro: o modelo de quadro do Trello).
pub const CMD_FROM: NodeId = hash_node_id("kanban_cmd_new_from");
/// J1 — «Join a shared Kanban»: o «Share» da aba activa, no campo do link.
pub const CMD_JOIN: NodeId = hash_node_id("kanban_cmd_join");

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
        crate::io::paint(id, area, scene, text, *theme);
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
        let taken = self.route_pointer(id, input, kind, button, x, y);
        crate::hooks::settle(self);
        taken
    }

    fn pointer_move(&mut self, id: u64, input: PointerInput<'_>, x: f32, y: f32) -> bool {
        let taken = self.route_move(id, input, x, y);
        crate::hooks::settle(self);
        taken
    }

    fn wheel(&mut self, id: u64, x: f32, y: f32, dy: f32, shift: bool) -> bool {
        let taken = self.route_wheel(id, x, y, dy, shift);
        crate::hooks::settle(self);
        taken
    }

    fn key(&mut self, id: u64, ts: &mut TextSystem, input: KeyInput<'_>) -> KeyOutcome {
        let out = self.route_key(id, ts, input);
        crate::hooks::settle(self);
        out
    }

    fn compose(&mut self, id: u64, c: Compose<'_>) -> bool {
        let taken = crate::keys::compose(self, id, c);
        crate::hooks::settle(self);
        taken
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
        // As reações pendentes primeiro (são do gesto anterior); o desfazer em si não reage.
        crate::hooks::settle(self);
        // Um gesto que tocou vários Kanbans (mover para outro, espelhar) desfaz-se inteiro.
        crate::w6::gesture::undo(self, id, redo)
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
                id: CMD_FROM,
                label: tr("kanban.w6.cmd.new_from").to_owned(),
                menu: DocMenu::File,
            },
            DocCommand {
                id: CMD_HOME,
                label: tr("kanban.cmd.home").to_owned(),
                menu: DocMenu::File,
            },
            DocCommand {
                id: CMD_JOIN,
                label: tr("kanban.share.join_caption").to_owned(),
                menu: DocMenu::File,
            },
            DocCommand {
                id: Self::CMD_IMPORT,
                label: tr("kanban.import.cmd_import").to_owned(),
                menu: DocMenu::File,
            },
            DocCommand {
                id: Self::CMD_EXPORT,
                label: tr("kanban.import.cmd_export").to_owned(),
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
        } else if cmd == CMD_FROM {
            self.new_from();
            true
        } else if cmd == CMD_JOIN {
            self.open_join();
            true
        } else if cmd == Self::CMD_IMPORT {
            self.import_from_trello();
            true
        } else if cmd == Self::CMD_EXPORT {
            self.export_kanban();
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
        self.tabs.iter().find(|t| t.id() == id).map(look_of)
    }

    fn drop_files(&mut self, id: u64, x: f32, y: f32, paths: &[std::path::PathBuf]) -> bool {
        if id == HOME || !self.area.contains(x, y) {
            return false;
        }
        let Some(tab) = self.tabs.iter().find(|t| t.id() == id) else {
            return false;
        };
        // Com o verso aberto, o que se larga é do cartão dele (o modal do Trello); senão, do cartão
        // sob o rato.
        let card = match tab.back.as_deref() {
            Some(b) => Some(b.card),
            None => crate::pointer::card_under(tab, x, y),
        };
        let Some(card) = card else {
            return false;
        };
        crate::back::attach::attach_paths(self, id, card, paths) > 0 || !self.notices.is_empty()
    }

    fn wants_frame(&self) -> bool {
        crate::front::cover::pending() || crate::back::attach::loading()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl KanbanDocs {
    /// «Join a shared Kanban»: a aba activa (ou a primeira) com o «Share» aberto no campo do link.
    fn open_join(&mut self) {
        let tab = self
            .active
            .filter(|a| *a != HOME && self.tabs.iter().any(|t| t.id() == *a))
            .or_else(|| self.tabs.first().map(crate::tab::Tab::id));
        let Some(id) = tab else {
            self.open_home();
            return;
        };
        self.focus = Some(id);
        if self.tab_mut(id).is_some_and(|t| t.w6.over != Some(crate::w6::Over::Share)) {
            crate::w6::toggle_over(self, id, crate::w6::Over::Share);
        }
        if let Some(t) = self.tab_mut(id) {
            t.w6.focus(crate::w6::Boxed::Link, "");
        }
    }

    fn route_pointer(
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
        if let Some(taken) = crate::io::pointer(id, kind, x, y) {
            return taken;
        }
        if let Some(taken) = crate::w6::pointer(self, id, kind, x, y) {
            return taken;
        }
        if crate::back::is_open(self, id) {
            return match kind {
                PointerKind::Down => {
                    crate::back::down(self, id, input.text, (x, y), input.mods, input.now_ns)
                }
                PointerKind::Up => crate::back::up(self, id, x, y),
                PointerKind::Move => crate::back::moved(self, id, input.text, x, y),
            };
        }
        match kind {
            PointerKind::Down => crate::pointer::down(self, id, input.text, button, x, y),
            PointerKind::Up => crate::pointer::up(self, id, input.text, input.now_ns, x, y),
            PointerKind::Move => crate::pointer::moved(self, id, input.text, x, y),
        }
    }

    fn route_move(&mut self, id: u64, input: PointerInput<'_>, x: f32, y: f32) -> bool {
        if id == HOME {
            return self.home_pointer(PointerKind::Move, x, y);
        }
        if let Some(taken) = crate::io::pointer(id, PointerKind::Move, x, y) {
            return taken;
        }
        if let Some(taken) = crate::w6::pointer(self, id, PointerKind::Move, x, y) {
            return taken;
        }
        if crate::back::is_open(self, id) {
            return crate::back::moved(self, id, input.text, x, y);
        }
        crate::pointer::moved(self, id, input.text, x, y)
    }

    fn route_wheel(&mut self, id: u64, x: f32, y: f32, dy: f32, shift: bool) -> bool {
        if crate::io::wheel(id, crate::pointer::wheel_px(dy)) {
            return true;
        }
        if id != HOME && crate::w6::wheel(self, id, x, y, crate::pointer::wheel_px(dy), shift) {
            return true;
        }
        if id != HOME && crate::back::is_open(self, id) {
            return crate::back::wheel(self, id, crate::pointer::wheel_px(dy));
        }
        id != HOME && crate::pointer::wheel(self, id, x, y, dy, shift)
    }

    fn route_key(&mut self, id: u64, ts: &mut TextSystem, input: KeyInput<'_>) -> KeyOutcome {
        if id == HOME {
            return KeyOutcome::default();
        }
        if let Some(out) = crate::io::key(id, &input) {
            return out;
        }
        crate::keys::key(self, id, ts, input)
    }

    /// «New Kanban from…»: o diálogo na aba activa (sem nenhuma, o início, onde se cria um).
    fn new_from(&mut self) {
        let tab = self
            .active
            .filter(|a| *a != HOME && self.tabs.iter().any(|t| t.id() == *a))
            .or_else(|| self.tabs.first().map(crate::tab::Tab::id));
        let Some(id) = tab else {
            self.open_home();
            return;
        };
        self.focus = Some(id);
        let anchor = self
            .tab_mut(id)
            .map(|t| t.layout.board_menu)
            .unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0));
        crate::w6::copy_move::open(self, id, crate::w6::DialogKind::NewKanbanFrom, anchor);
    }

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

/// O ponteiro que uma vista quer onde o rato a viu da última vez (a aba e o elemento do Quadro).
pub(crate) fn look_of(t: &crate::tab::Tab) -> PointerLook {
    use crate::edit::Target;
    use crate::tab::Hit;
    // O título que se escreve no lugar é texto (o I), não o botão que o abre.
    let editing = t.field.as_ref().map(|f| f.target);
    match t.hover {
        Some(Hit::ListTitle(l)) if editing == Some(Target::ListTitle(l)) => PointerLook::Text,
        Some(Hit::KanbanTitle) if editing == Some(Target::KanbanName) => PointerLook::Text,
        Some(crate::tab::Hit::Field) => PointerLook::Text,
        Some(crate::tab::Hit::W6(crate::w6::Act::Inside)) => PointerLook::Arrow,
        Some(crate::tab::Hit::Board) | None => PointerLook::Arrow,
        Some(_) => PointerLook::Hand,
    }
}
