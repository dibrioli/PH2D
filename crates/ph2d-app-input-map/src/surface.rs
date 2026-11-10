//! A `DocumentSurface` da aba Input Map: uma aba só, que guarda a vista mesmo fechada.

use ph2d_editor_core::doc_surface::{
    Compose, DocTab, DocumentSurface, KeyInput, KeyOutcome, PointerInput, PointerLook,
};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerKind};
use ph2d_input_map_editor::{InputMapDoc, View};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// O id da única aba da família.
pub const TAB_ID: u64 = 1;

pub struct InputMapTab {
    pub doc: InputMapDoc,
    open: bool,
    focus: Option<u64>,
    /// A faixa do HUD de baixo que o hero pinta POR CIMA da área (`chrome_over`).
    over: Option<Rect>,
}

impl InputMapTab {
    #[must_use]
    pub fn new(view: View) -> Self {
        Self {
            doc: InputMapDoc::new(view),
            open: false,
            focus: None,
            over: None,
        }
    }

    /// Abre a aba, ou trá-la à frente se já está aberta (uma só).
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
        ph2d_input_map_editor::paint::paint_doc(&mut self.doc, area, scene, text, *theme);
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
            (PointerKind::Down, PointerButton::Primary) => self.doc.press(x, y),
            (PointerKind::Up, _) => self.doc.release(),
            (PointerKind::Move, _) => self.doc.motion(x, y),
            _ => false,
        }
    }

    fn pointer_move(&mut self, _id: u64, _input: PointerInput<'_>, x: f32, y: f32) -> bool {
        self.doc.motion(x, y)
    }

    fn pointer_look(&self, _id: u64, _x: f32, _y: f32) -> Option<PointerLook> {
        self.doc.hover().map(|_| PointerLook::Hand)
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
    }

    fn undo(&mut self, _id: u64, _redo: bool) -> bool {
        false
    }

    /// O mapa é documento: o Ctrl+Z vai à fila do projecto (o mapa entra nela na R2).
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

/// A área sem a faixa do HUD que o hero pinta por cima (a 1280×720 tapava a última linha do pé, foto A1):
/// a aba pára no topo dele, como o SFX Lab e o Kanban.
#[must_use]
pub fn above_the_hud(area: Rect, over: Option<Rect>) -> Rect {
    let Some(o) = over.filter(|o| o.x < area.x + area.w && o.x + o.w > area.x) else {
        return area;
    };
    let bottom = o.y.clamp(area.y, area.y + area.h);
    Rect::new(area.x, area.y, area.w, bottom - area.y)
}
