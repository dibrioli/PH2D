//! A `DocumentSurface` da aba Input Map: uma aba só, que guarda a vista mesmo fechada.

use ph2d_editor_core::doc_surface::{
    Compose, DocTab, DocumentSurface, KeyInput, KeyOutcome, PointerInput, PointerLook,
};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerKind};
pub use ph2d_input_map_editor::layout::above_the_hud;
use ph2d_input_map_editor::{InputMapDoc, View};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

pub const TAB_ID: u64 = 1;

pub struct InputMapTab {
    pub doc: InputMapDoc,
    open: bool,
    focus: Option<u64>,
    /// A faixa do HUD que o hero pinta POR CIMA (`chrome_over`); o rect pintado e o Down tomado — o Move fora dele
    /// só é da aba com o botão preso nela (senão matava a borda das colunas, dono 10/11).
    over: Option<Rect>,
    claim: (Rect, bool),
}

impl InputMapTab {
    #[must_use]
    pub fn new(view: View) -> Self {
        Self {
            doc: InputMapDoc::new(view),
            open: false,
            focus: None,
            over: None,
            claim: (Rect::default(), false),
        }
    }

    pub fn open(&mut self) {
        self.open = true;
        self.focus = Some(TAB_ID);
    }

    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.open
    }
}

impl DocumentSurface for InputMapTab {
    fn tabs(&self) -> Vec<DocTab> {
        if !self.open {
            return Vec::new();
        }
        vec![DocTab {
            id: TAB_ID,
            title: ph2d_i18n::tr("input_map_tab.title").to_string(),
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
        let area = above_the_hud(area, self.over);
        self.claim.0 = area;
        ph2d_input_map_editor::paint::paint_doc(&mut self.doc, area, scene, text, *theme);
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
        match (kind, button) {
            (PointerKind::Down, PointerButton::Primary) => {
                self.claim.1 = self.doc.press(x, y);
                self.claim.1
            }
            (PointerKind::Up, _) => {
                self.claim.1 = false;
                self.doc.release_at(x, y)
            }
            (PointerKind::Move, _) => self.pointer_move(id, input, x, y),
            _ => false,
        }
    }

    fn pointer_move(&mut self, _id: u64, _input: PointerInput<'_>, x: f32, y: f32) -> bool {
        self.doc.motion(x, y) && (self.claim.1 || self.claim.0.contains(x, y))
    }

    fn pointer_look(&self, _id: u64, x: f32, y: f32) -> Option<PointerLook> {
        self.doc.look_at(x, y)
    }

    fn a11y(&self, _id: u64) -> ph2d_input_map_editor::interact::A11yNodes {
        self.doc.a11y()
    }

    fn wants_frame(&self) -> bool {
        self.open && self.doc.wants_frame()
    }

    fn wheel(&mut self, _id: u64, _x: f32, _y: f32, dy: f32, _shift: bool) -> bool {
        self.doc.wheel(dy)
    }

    fn key(&mut self, _id: u64, _ts: &mut TextSystem, input: KeyInput<'_>) -> KeyOutcome {
        KeyOutcome {
            consumed: self.doc.key(&input),
            copy: None,
        }
    }

    fn raw_key(
        &mut self,
        _id: u64,
        key: ph2d_input::Key,
        pressed: bool,
        writes_text: bool,
    ) -> bool {
        self.doc.raw_key(key, pressed, writes_text)
    }

    fn compose(&mut self, _id: u64, _c: Compose<'_>) -> bool {
        false
    }

    fn close(&mut self, _id: u64) {
        self.open = false;
        self.doc.stop_listening();
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
