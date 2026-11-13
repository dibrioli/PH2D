//! A `DocumentSurface` da aba Joint Editor: uma aba só, que guarda o estado mesmo fechada.

use ph2d_a11y::NodeId;
use ph2d_editor_core::doc_surface::{
    Compose, DocCommand, DocMenu, DocTab, DocumentSurface, DropAt, Dropping, KeyInput, KeyOutcome,
    PointerInput, PointerLook,
};
use ph2d_editor_core::registry::hash_node_id;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerKind};
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

use crate::doc::{Hit, JointEditorDoc};

pub const TAB_ID: u64 = 1;
/// File ▸ Joint Editor… (e na paleta `Ctrl+K`).
pub const CMD_OPEN: NodeId = hash_node_id("joint_editor.cmd.open");
const DOC_NODE: NodeId = hash_node_id("joint_editor.document");

#[derive(Default)]
pub struct JointEditorTab {
    pub doc: JointEditorDoc,
    open: bool,
    focus: Option<u64>,
    claim: bool,
    /// A faixa do HUD pintada POR CIMA (`chrome_over`): a aba pára acima dela.
    over: Option<Rect>,
    /// A cena de smoke já abriu a aba (uma vez só).
    pub(crate) smoke_done: bool,
}

impl JointEditorTab {
    pub fn open(&mut self) {
        self.open = true;
        self.focus = Some(TAB_ID);
    }

    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.open
    }
}

/// A área sem a faixa do HUD que a tapa por baixo (a mesma lei da aba Input Map).
fn above(area: Rect, over: Option<Rect>) -> Rect {
    let Some(o) = over.filter(|o| o.x < area.x + area.w && o.x + o.w > area.x) else {
        return area;
    };
    let bottom = o.y.clamp(area.y, area.y + area.h);
    Rect::new(area.x, area.y, area.w, bottom - area.y)
}

/// O id de acessibilidade de um alvo: estável entre quadros (o mesmo alvo, o mesmo nó).
fn node_of(h: &Hit) -> NodeId {
    let s = format!("{h:?}");
    let mut x: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.bytes() {
        x = (x ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3);
    }
    NodeId(DOC_NODE.0 ^ x)
}

impl DocumentSurface for JointEditorTab {
    fn tabs(&self) -> Vec<DocTab> {
        if !self.open {
            return Vec::new();
        }
        vec![DocTab {
            id: TAB_ID,
            title: tr("joint_editor.title").to_owned(),
            dirty: false,
        }]
    }

    fn paint(
        &mut self,
        _id: u64,
        area: Rect,
        scene: &mut VectorScene,
        text: &mut TextSystem,
        theme: &Theme,
    ) {
        crate::paint::paint(&mut self.doc, above(area, self.over), scene, text, *theme);
    }

    fn chrome_over(&mut self, _id: u64, over: Option<Rect>) {
        self.over = over;
    }

    fn pointer(
        &mut self,
        _id: u64,
        _input: PointerInput<'_>,
        kind: PointerKind,
        button: PointerButton,
        x: f32,
        y: f32,
    ) -> bool {
        match (kind, button) {
            (PointerKind::Down, PointerButton::Primary) => {
                self.claim = self.doc.press(x, y);
                self.claim
            }
            (PointerKind::Up, _) => {
                let was = std::mem::take(&mut self.claim);
                self.doc.release(x, y) || was
            }
            (PointerKind::Move, _) => self.doc.motion(x, y),
            _ => false,
        }
    }

    fn pointer_move(&mut self, _id: u64, _input: PointerInput<'_>, x: f32, y: f32) -> bool {
        self.doc.motion(x, y) && (self.claim || self.doc.area.contains(x, y))
    }

    fn pointer_look(&self, _id: u64, x: f32, y: f32) -> Option<PointerLook> {
        if !self.doc.area.contains(x, y) {
            return None;
        }
        Some(match self.doc.hit_at(x, y) {
            _ if self.doc.drag.is_some() => PointerLook::Grabbing,
            Some(Hit::Handle(_)) => PointerLook::Grab,
            Some(Hit::Field(_)) => PointerLook::Text,
            Some(Hit::OpenMenu(_)) if self.doc.menu.is_some() => PointerLook::Arrow,
            Some(_) => PointerLook::Hand,
            None => PointerLook::Arrow,
        })
    }

    fn a11y(&self, _id: u64) -> Vec<(NodeId, ph2d_a11y::Node)> {
        let b = |r: Rect| {
            (
                f64::from(r.x),
                f64::from(r.y),
                f64::from(r.w),
                f64::from(r.h),
            )
        };
        let hits: Vec<&(Rect, Hit, String)> = self
            .doc
            .hits
            .iter()
            .filter(|(_, _, l)| !l.is_empty())
            .collect();
        let (x, y, w, h) = b(self.doc.area);
        let root = ph2d_a11y::NodeBuilder::new(ph2d_a11y::Role::Document)
            .label(tr("joint_editor.title"))
            .bounds(x, y, w, h)
            .children(hits.iter().map(|(_, hit, _)| node_of(hit)))
            .build();
        let mut out = vec![(DOC_NODE, root)];
        for (r, hit, label) in hits {
            let role = match hit {
                Hit::Field(_) => ph2d_a11y::Role::SpinButton,
                Hit::Flag(_) => ph2d_a11y::Role::CheckBox,
                Hit::Handle(_) => ph2d_a11y::Role::Slider,
                Hit::Row(_) | Hit::Kind(_) | Hit::Template(_) => ph2d_a11y::Role::ListItem,
                Hit::Scope(_) | Hit::Drive(_) => ph2d_a11y::Role::Tab,
                _ => ph2d_a11y::Role::Button,
            };
            let (x, y, w, h) = b(*r);
            let node = ph2d_a11y::NodeBuilder::new(role)
                .label(label.clone())
                .bounds(x, y, w, h)
                .action(ph2d_a11y::Action::Click)
                .build();
            out.push((node_of(hit), node));
        }
        out
    }

    fn wants_frame(&self) -> bool {
        self.open && (self.doc.drag.is_some() || self.doc.typing.is_some())
    }

    fn wheel(&mut self, _id: u64, x: f32, y: f32, dy: f32, _shift: bool) -> bool {
        self.doc.wheel(x, y, dy)
    }

    fn key(&mut self, _id: u64, _ts: &mut TextSystem, input: KeyInput<'_>) -> KeyOutcome {
        KeyOutcome {
            consumed: self.doc.key(input.key, input.pressed, input.text),
            copy: None,
        }
    }

    fn compose(&mut self, _id: u64, _c: Compose<'_>) -> bool {
        false
    }

    fn close(&mut self, _id: u64) {
        self.doc.finish_typing();
        self.open = false;
    }

    fn undo(&mut self, _id: u64, _redo: bool) -> bool {
        false
    }

    fn undo_is_the_projects(&self) -> bool {
        true
    }

    fn take_focus_request(&mut self) -> Option<u64> {
        self.focus.take()
    }

    fn family_commands(&self) -> Vec<DocCommand> {
        vec![DocCommand {
            id: CMD_OPEN,
            label: tr("joint_editor.cmd.open").to_owned(),
            menu: DocMenu::File,
        }]
    }

    fn family_command(&mut self, cmd: NodeId) -> bool {
        if cmd != CMD_OPEN {
            return false;
        }
        self.open();
        true
    }

    fn drop_hover(&mut self, _id: u64, at: Option<DropAt<'_>>) -> bool {
        let at = at.and_then(|a| match a.what {
            Dropping::Objects(names) => Some((a.x, a.y, names)),
            _ => None,
        });
        self.doc.drop_hover(at)
    }

    fn drop(&mut self, _id: u64, at: DropAt<'_>) -> bool {
        match at.what {
            Dropping::Objects(names) => self.doc.drop_at(at.x, at.y, names),
            _ => false,
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
