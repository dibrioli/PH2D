//! Os testes da COSTURA hero → aba do Planner: o menu File pelo `apply_event`, o rato pelo
//! `board_view`, o teclado pelo `board_keys`, o desfazer pelo `surface_input::undo`; o efeito lê-se no
//! DOCUMENTO (`hero.documents.kanbans()`). Cada quadro corre como a shell: o `frame` do Planner,
//! `follow_surfaces`, pintar (a família Kanban não entra: as famílias não se conhecem).

mod calendario;
mod destino;
mod fotos;
mod ics;
mod pessoal;
mod serie;

use ph2d_app_planner::{Hit, PlannerDocs};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::screens::hero::board_keys::{self, BoardKey};
use ph2d_editor_core::screens::hero::board_view::{self, Input};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{NodeId, ToastQueue};
use ph2d_host::{Modifiers, PointerButton, PointerKind};
use ph2d_kanban_dates::Zone;
use ph2d_kanban_model::{Card, KanbanId, KanbanSet, Moment};
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

pub const AREA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1200.0,
    h: 760.0,
};

const QUADRO_NS: u128 = 16_666_667;

pub struct Mesa {
    pub hero: HeroScreen,
    pub ts: TextSystem,
    pub agora: u128,
    pub toasts: ToastQueue,
}

impl Mesa {
    /// Um hero com a família do Planner (como a shell, `init.rs`), o relógio fixo e o fuso UTC.
    pub fn new() -> Self {
        let mut hero = HeroScreen::new(NodeId(1));
        ph2d_app_planner::install(&mut hero);
        let mut m = Self {
            hero,
            ts: TextSystem::without_system_fonts(),
            agora: 1_000_000_000_000,
            toasts: ToastQueue::new(),
        };
        let now = ph2d_app_planner::wall_clock();
        let d = m.docs();
        d.pinned = Some(now);
        d.set_zone(Zone::utc());
        m
    }

    /// A cena do smoke (o projecto rico), com a aba no Calendário.
    pub fn cena() -> (Self, KanbanId) {
        let mut m = Self::new();
        let k = ph2d_app_planner::smoke::stage(&mut m.hero, 1).expect("cena 1");
        m.quadros(2);
        m.foca_planner();
        (m, k)
    }

    /// Volta a pôr a aba do Planner à frente (a cena abre também a do Kanban).
    pub fn foca_planner(&mut self) {
        self.hero
            .apply_event(ph2d_editor_core::interaction::WidgetEvent::Click(
                ph2d_app_planner::CMD_CALENDAR,
            ));
        self.quadros(2);
    }

    pub fn quadro(&mut self) {
        self.agora += QUADRO_NS;
        self.hero.store.note_clock(self.agora);
        ph2d_app_planner::frame(&mut self.hero, &mut self.toasts);
        ph2d_editor_core::screens::hero::document_tabs::follow_surfaces(&mut self.hero);
        let theme = self.hero.theme;
        let mut scene = VectorScene::new();
        if let Some((f, id)) = self.hero.documents.active_surface_mut() {
            f.paint(id, AREA, &mut scene, &mut self.ts, &theme);
        }
    }

    pub fn quadros(&mut self, n: usize) {
        for _ in 0..n {
            self.quadro();
        }
    }

    pub fn docs(&mut self) -> &mut PlannerDocs {
        self.hero
            .documents
            .surfaces_mut()
            .find_mut::<PlannerDocs>()
            .expect("família instalada")
    }

    /// A aba activa é a do Planner?
    pub fn planner_activo(&mut self) -> bool {
        let Some(tab) = self.hero.documents.surface_active() else {
            return false;
        };
        self.hero
            .documents
            .surfaces_mut()
            .get(tab.family)
            .is_some_and(|f| f.as_any().is::<PlannerDocs>())
    }

    pub fn set(&self) -> KanbanSet {
        self.hero.documents.kanbans().clone()
    }

    pub fn cartao(&self, k: KanbanId, titulo: &str) -> Card {
        self.set()
            .get(k)
            .and_then(|kb| kb.cards.values().find(|c| c.title == titulo).cloned())
            .unwrap_or_else(|| panic!("cartão {titulo}"))
    }

    pub fn agora(&mut self) -> Moment {
        self.docs().pinned.expect("relógio fixo")
    }

    fn rato(&mut self, kind: PointerKind, x: f32, y: f32) -> bool {
        let input = Input {
            text: &mut self.ts,
            mods: Modifiers::default(),
            now_ns: self.agora,
        };
        board_view::pointer(&mut self.hero, input, kind, PointerButton::Primary, x, y, true)
    }

    pub fn move_(&mut self, x: f32, y: f32) -> bool {
        let input = Input {
            text: &mut self.ts,
            mods: Modifiers::default(),
            now_ns: self.agora,
        };
        board_view::pointer_move(&mut self.hero, input, x, y)
    }

    /// Um clique e um quadro.
    pub fn clica(&mut self, (x, y): (f32, f32)) {
        assert!(self.rato(PointerKind::Down, x, y), "o carregar em ({x}, {y}) não foi tomado");
        assert!(self.rato(PointerKind::Up, x, y), "o largar em ({x}, {y}) não foi tomado");
        self.quadro();
    }

    /// Clica no sítio `hit` da moldura (o rect pintado no último quadro).
    pub fn clica_em(&mut self, hit: Hit) {
        let r = self
            .docs()
            .probe_hit(hit)
            .unwrap_or_else(|| panic!("{hit:?} não está pintado"));
        self.clica(centro(r));
    }

    /// Um arrasto de `de` para `para`, um quadro entre cada passo, e largar.
    pub fn arrasta(&mut self, de: (f32, f32), para: (f32, f32)) {
        assert!(self.rato(PointerKind::Down, de.0, de.1), "o carregar não foi tomado");
        #[allow(clippy::cast_precision_loss)]
        for i in 1..=8 {
            let t = i as f32 / 8.0;
            self.move_(de.0 + (para.0 - de.0) * t, de.1 + (para.1 - de.1) * t);
            self.quadro();
        }
        assert!(self.rato(PointerKind::Up, para.0, para.1), "o largar não foi tomado");
        self.quadro();
    }

    pub fn tecla(&mut self, k: BoardKey, text: Option<&str>) {
        let _ = board_keys::key(&mut self.hero, &mut self.ts, k, Modifiers::default(), true, text, None);
        let _ = board_keys::key(&mut self.hero, &mut self.ts, k, Modifiers::default(), false, None, None);
    }

    pub fn escreve(&mut self, s: &str) {
        for c in s.chars() {
            let k = if c == ' ' {
                BoardKey::Space
            } else if c.is_ascii_alphanumeric() {
                BoardKey::Char(c.to_ascii_lowercase())
            } else {
                BoardKey::Other
            };
            let t = c.to_string();
            self.tecla(k, Some(&t));
        }
        self.quadro();
    }

    pub fn enter(&mut self) {
        self.tecla(BoardKey::Enter, Some("\r"));
        self.quadro();
    }

    pub fn desfaz(&mut self) -> Option<bool> {
        let r = ph2d_editor_core::screens::hero::surface_input::undo(&mut self.hero, false);
        self.quadro();
        r
    }
}

pub fn centro(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}
