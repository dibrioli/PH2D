//! Os testes da COSTURA hero → aba do Kanban: cada evento entra pela porta real do hero (o menu
//! File pelo `apply_event`, o rato pelo `board_view`, o teclado pelo `board_keys`, o desfazer pelo
//! `surface_input::undo`) e o efeito lê-se no DOCUMENTO (`hero.documents.kanbans()`) ou nas sondas
//! da vista. Cada quadro corre como a shell o corre: `frame` da família → `follow_surfaces` → pintar.

mod arrastar;
mod canvas_entrada;
mod d9;
mod duas_janelas;
mod escrever;
mod foto_tutorial;
mod frente;
mod j1_fotos;
mod j1_partilha;
mod j1_vistas;
mod j2_auto;
mod j2_fotos;
mod j2_verso;
mod j3_ganchos;
mod j3_import;
mod j4_reconexao;
mod j5_botoes;
mod j6_verso_regras;
mod j8_fotos_ao_vivo;
mod medida;
mod t1_abrir_cartao;
mod teclado;
mod teto;
mod titulo;
mod verso;
mod verso_anexos;
mod escrever_no_verso;
mod verso_listas;
mod vista;
mod voo;
mod w6;
mod w6_b;

use ph2d_app_kanban::KanbanDocs;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::screens::hero::board_keys::{self, BoardKey};
use ph2d_editor_core::screens::hero::board_view::{self, Input};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{Modifiers, PointerButton, PointerKind};
use ph2d_kanban_model::{CardId, Kanban, KanbanId, ListId};
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

pub const AREA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1200.0,
    h: 700.0,
};

/// O passo do relógio por quadro (60 Hz).
pub const QUADRO_NS: u128 = 16_666_667;

pub struct Mesa {
    pub hero: HeroScreen,
    pub ts: TextSystem,
    pub agora: u128,
}

impl Default for Mesa {
    fn default() -> Self {
        Self::new()
    }
}

impl Mesa {
    /// Um hero com a família instalada, como a shell (`init.rs`).
    pub fn new() -> Self {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.documents
            .install_surface(Box::new(KanbanDocs::default()));
        Self {
            hero,
            ts: TextSystem::without_system_fonts(),
            agora: 1_000_000_000_000,
        }
    }

    /// A cena 1 do smoke («Short film — production»), aberta e pintada.
    pub fn cena1() -> (Self, KanbanId) {
        let mut m = Self::new();
        let id = ph2d_app_kanban::smoke::stage(&mut m.hero, 1).expect("cena 1");
        m.quadro();
        m.quadro();
        (m, id)
    }

    /// Um quadro como a shell: o relógio, o `frame` da família, a aba que pede o foco, a pintura.
    pub fn quadro(&mut self) {
        self.relogio();
        ph2d_app_kanban::frame(&mut self.hero);
        ph2d_editor_core::screens::hero::document_tabs::follow_surfaces(&mut self.hero);
        let mut scene = VectorScene::new();
        self.pinta(&mut scene);
    }

    pub fn relogio(&mut self) {
        self.agora += QUADRO_NS;
        self.hero.store.note_clock(self.agora);
    }

    /// A pintura da aba activa no rect que o hero lhe dá (`hero/paint.rs`).
    pub fn pinta(&mut self, scene: &mut VectorScene) {
        let theme = self.hero.theme;
        if let Some((f, id)) = self.hero.documents.active_surface_mut() {
            f.paint(id, AREA, scene, &mut self.ts, &theme);
        }
    }

    /// `n` quadros.
    pub fn quadros(&mut self, n: usize) {
        for _ in 0..n {
            self.quadro();
        }
    }

    /// Quadros até as molas pararem.
    pub fn assenta(&mut self, id: KanbanId) {
        for _ in 0..240 {
            self.quadro();
            if !self.docs().probe_moving(id.0) {
                return;
            }
        }
        panic!("as molas não assentaram em 4 s");
    }

    pub fn docs(&mut self) -> &mut KanbanDocs {
        self.hero
            .documents
            .surfaces_mut()
            .find_mut::<KanbanDocs>()
            .expect("família instalada")
    }

    /// A aba activa do hero é uma aba da família, com este id?
    pub fn activa(&mut self) -> Option<u64> {
        let tab = self.hero.documents.surface_active()?;
        let ours = self
            .hero
            .documents
            .surfaces_mut()
            .get(tab.family)
            .is_some_and(|f| f.as_any().is::<KanbanDocs>());
        ours.then_some(tab.id)
    }

    fn rato(&mut self, kind: PointerKind, x: f32, y: f32) -> bool {
        let input = Input {
            text: &mut self.ts,
            mods: Modifiers::default(),
            now_ns: self.agora,
        };
        board_view::pointer(
            &mut self.hero,
            input,
            kind,
            PointerButton::Primary,
            x,
            y,
            true,
        )
    }

    pub fn carrega(&mut self, x: f32, y: f32) -> bool {
        self.rato(PointerKind::Down, x, y)
    }

    pub fn larga(&mut self, x: f32, y: f32) -> bool {
        self.rato(PointerKind::Up, x, y)
    }

    pub fn move_(&mut self, x: f32, y: f32) -> bool {
        let input = Input {
            text: &mut self.ts,
            mods: Modifiers::default(),
            now_ns: self.agora,
        };
        board_view::pointer_move(&mut self.hero, input, x, y)
    }

    /// A roda em `(x, y)` (a porta do hero), e um quadro.
    pub fn roda(&mut self, x: f32, y: f32, dy: f32) -> bool {
        let taken = board_view::wheel(&mut self.hero, x, y, dy);
        self.quadro();
        taken
    }

    /// Um clique (carregar + largar no mesmo sítio) e um quadro.
    pub fn clica(&mut self, (x, y): (f32, f32)) {
        assert!(
            self.carrega(x, y),
            "o carregar em ({x}, {y}) não foi tomado"
        );
        assert!(self.larga(x, y), "o largar em ({x}, {y}) não foi tomado");
        self.quadro();
    }

    /// Um arrasto: carrega em `de`, anda até `para` em `passos` (um quadro entre cada), sem largar.
    pub fn arrasta(&mut self, de: (f32, f32), para: (f32, f32), passos: usize) {
        assert!(self.carrega(de.0, de.1), "o carregar não foi tomado");
        #[allow(clippy::cast_precision_loss)]
        for i in 1..=passos {
            let t = i as f32 / passos as f32;
            self.move_(de.0 + (para.0 - de.0) * t, de.1 + (para.1 - de.1) * t);
            self.quadro();
        }
    }

    /// Uma tecla premida pelo caminho da shell (`on_keyboard_input`): a composição vê-a primeiro e,
    /// se a tomou, o despacho pára; senão segue para a porta do hero (o `BoardKey`, com o texto).
    /// Report do dono 28/10: sem este primeiro passo os testes escreviam na descrição e ele não.
    pub fn tecla(&mut self, k: BoardKey, mods: Modifiers, text: Option<&str>) -> bool {
        if compoe_primeiro(&mut self.hero, k, mods) {
            return true;
        }
        let out = board_keys::key(&mut self.hero, &mut self.ts, k, mods, true, text, None);
        let _ = board_keys::key(&mut self.hero, &mut self.ts, k, mods, false, None, None);
        out.consumed
    }

    /// Escreve `s` como o teclado o escreveria (uma tecla por carácter).
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
            self.tecla(k, Modifiers::default(), Some(&t));
        }
    }

    pub fn enter(&mut self) {
        self.tecla(BoardKey::Enter, Modifiers::default(), Some("\r"));
        self.quadro();
    }

    /// O `Ctrl+Z` do menu Edit (`surface_input::undo`).
    pub fn desfaz(&mut self) -> Option<bool> {
        let r = ph2d_editor_core::screens::hero::surface_input::undo(&mut self.hero, false);
        self.quadro();
        r
    }

    /// O Kanban `id` do projecto (uma cópia).
    pub fn kanban(&self, id: KanbanId) -> Kanban {
        self.hero
            .documents
            .kanbans()
            .get(id)
            .expect("kanban")
            .clone()
    }

    pub fn bytes(&self) -> Vec<u8> {
        self.hero.documents.kanbans().to_bytes().expect("bytes")
    }

    pub fn lista(&self, id: KanbanId, titulo: &str) -> ListId {
        self.kanban(id)
            .lists
            .values()
            .find(|l| l.title == titulo)
            .unwrap_or_else(|| panic!("lista {titulo}"))
            .id
    }

    pub fn cartoes(&self, id: KanbanId, l: ListId) -> Vec<CardId> {
        self.kanban(id).cards_in(l).iter().map(|c| c.id).collect()
    }

    pub fn rect_cartao(&mut self, id: KanbanId, c: CardId) -> Rect {
        self.docs()
            .probe_card_rect(id.0, c)
            .expect("cartão na disposição")
    }

    /// Os botões do painel do arquivo aberto.
    pub fn painel(&mut self, id: KanbanId) -> Vec<(String, Rect)> {
        let mut ts = std::mem::replace(&mut self.ts, TextSystem::without_system_fonts());
        let rows = self.docs().probe_panel_rows(id.0, &mut ts);
        self.ts = ts;
        rows
    }

    pub fn rect_lista(&mut self, id: KanbanId, l: ListId) -> Rect {
        self.docs()
            .probe_list_rect(id.0, l)
            .expect("lista na disposição")
    }
}

/// Um ponto do cartão que só agarra o cartão (longe do lápis e dos rótulos).
/// O primeiro passo da shell para toda tecla premida: a composição (tecla morta) vê-a; o Ctrl de um
/// acorde é um evento à parte, antes da letra. `true` = tomada pela composição.
pub fn compoe_primeiro(hero: &mut HeroScreen, k: BoardKey, mods: Modifiers) -> bool {
    use ph2d_editor_core::ime::ComposeKey;
    if mods.ctrl || mods.shift {
        let _ = hero.handle_compose_key(ComposeKey::Modifier);
    }
    let ck = match k {
        BoardKey::Escape => ComposeKey::Escape,
        _ => ComposeKey::Other,
    };
    hero.handle_compose_key(ck)
}

pub fn pega(r: Rect) -> (f32, f32) {
    (r.x + 20.0, r.y + r.h - 6.0)
}

pub fn centro(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}
mod j7_porta;
