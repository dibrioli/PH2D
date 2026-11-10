//! **Os gates da aba Input Map** (plano `docs/InputMap/01_plano.md` §3): o [`Mesa`] liga a família a um
//! `HeroScreen` de teste e dirige-a pelas portas reais — o clique do menu, o `frame`, o `follow_surfaces`, a
//! pintura e o ponteiro da aba activa. As ondas seguintes (A · List, B · Keyboard, C · Try it) usam-no.

mod abrir;
mod acabamento_r12;
mod acabamento_r5b;
mod acabamento_r6a;
mod auditoria_fios;
mod auditoria_r9a;
mod cenas_do_smoke;
mod lista;
mod lista_acabamento;
mod lista_censo;
mod lista_r7a;
mod mutacao_r13;
mod polimento_b;
mod polimento_c;
mod polimento_w3;
mod teclado;
mod teclado_censo;
mod teclado_r7b;
mod teclado_r9b;
mod teclado_w2;
mod teclas_w1;
mod try_it;
mod try_it_censo;
mod try_it_r10b;
mod try_it_r11a;
mod try_it_r9c;

use ph2d_a11y::NodeId;
use ph2d_app_input_map::{InputMapTab, TAB_ID, View};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::doc_surface::PointerInput;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, ids};
use ph2d_host::{Modifiers, PointerButton, PointerKind};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// A janela do teste (a área central da aba).
pub(super) const AREA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0, // LITERAL-PX-OK: a janela do teste (plano §8)
    h: 1000.0, // LITERAL-PX-OK: a janela do teste (plano §8)
};

pub(super) struct Mesa {
    pub sim: SimWorld,
    pub hero: HeroScreen,
    pub ts: TextSystem,
    /// Há um comando ligado (o que a shell passa ao `frame`).
    pub gamepad: bool,
    /// O `InputState` da shell (o comando ao vivo; o estado do jogo com o Play a correr).
    pub live: ph2d_input::InputState,
    /// A sessão do jogador da shell (o mapa que o Play lê).
    pub jogador: ph2d_app_input_map::ph2d_player_controls::PlaySession,
}

impl Mesa {
    /// O herói com a família instalada na vista das prefs; a aba fechada.
    pub fn new(view: View) -> Self {
        let mut hero = HeroScreen::new(NodeId(1));
        ph2d_app_input_map::install(&mut hero, view);
        Self {
            sim: SimWorld::new(),
            hero,
            ts: TextSystem::without_system_fonts(),
            gamepad: false,
            live: ph2d_input::InputState::default(),
            jogador: Default::default(),
        }
    }

    /// A mesa com a aba aberta pelo menu e pintada uma vez.
    pub fn aberta(view: View) -> Self {
        let mut m = Self::new(view);
        m.menu();
        m
    }

    /// Settings ▸ Input Map…, seguido do quadro (o `frame` da família e o `follow_surfaces` do hero).
    pub fn menu(&mut self) {
        let menu = WidgetEvent::Click(ids::CTX_MENU_SETTINGS_INPUT_MAP);
        assert!(self.hero.apply_event(menu), "o menu tem de ser tomado");
        self.frame();
    }

    /// Um quadro: a família, o hero a seguir as abas, a pintura da aba activa.
    pub fn frame(&mut self) {
        let live = (&self.live, &self.jogador);
        ph2d_app_input_map::frame(self.sim.world(), &mut self.hero, self.gamepad, live);
        self.hero.documents.follow_surfaces();
        self.paint();
    }

    pub fn paint(&mut self) {
        self.paint_in(AREA);
    }

    /// Pinta a aba activa numa área à escolha (o censo a 1280×720).
    pub fn paint_in(&mut self, area: Rect) {
        let mut scene = VectorScene::new();
        let ts = &mut self.ts;
        if let Some((f, id)) = self.hero.documents.active_surface_mut() {
            f.paint(id, area, &mut scene, ts, &Theme::default());
        }
    }

    pub fn tab_mut(&mut self) -> &mut InputMapTab {
        self.hero
            .documents
            .surfaces_mut()
            .find_mut::<InputMapTab>()
            .expect("a família instalada")
    }

    /// O ponteiro passa em `(x, y)` (sem botão), seguido do quadro.
    pub fn hover(&mut self, (x, y): (f32, f32)) {
        let ts = &mut self.ts;
        if let Some((f, id)) = self.hero.documents.active_surface_mut() {
            let input = PointerInput {
                text: ts,
                mods: Modifiers::default(),
                now_ns: 0,
            };
            f.pointer_move(id, input, x, y);
        }
        self.frame();
    }

    /// O botão solta, seguido do quadro.
    pub fn release(&mut self) {
        let ts = &mut self.ts;
        if let Some((f, id)) = self.hero.documents.active_surface_mut() {
            let input = PointerInput {
                text: ts,
                mods: Modifiers::default(),
                now_ns: 0,
            };
            f.pointer(id, input, PointerKind::Up, PointerButton::Primary, 0.0, 0.0);
        }
        self.frame();
    }

    /// Uma tecla pela porta `key` da aba (o caminho do texto), seguida do quadro.
    pub fn key(&mut self, key: ph2d_editor_core::doc_surface::Key, text: Option<&str>) -> bool {
        let input = ph2d_editor_core::doc_surface::KeyInput {
            key,
            mods: Modifiers::default(),
            pressed: true,
            text,
            paste: None,
        };
        let ts = &mut self.ts;
        let took = self
            .hero
            .documents
            .active_surface_mut()
            .is_some_and(|(f, id)| f.key(id, ts, input).consumed);
        self.frame();
        took
    }

    /// Escreve `s` letra a letra no campo focado.
    pub fn type_text(&mut self, s: &str) {
        for ch in s.chars() {
            let t = ch.to_string();
            self.key(ph2d_editor_core::doc_surface::Key::Char(ch), Some(&t));
        }
    }

    /// Uma tecla FÍSICA como a shell a entrega: primeiro a escuta partilhada, depois o `raw_key` da aba.
    pub fn physical(&mut self, k: ph2d_input::Key) -> bool {
        let took = if let Some(evt) =
            ph2d_editor_core::interaction::capture_if_listening(&mut self.hero.store, k.0)
        {
            self.hero.apply_event(evt);
            true
        } else {
            ph2d_app_input_map::surface_raw_key(&mut self.hero, k.0, true, None)
        };
        self.frame();
        took
    }

    /// O centro de um alvo pintado.
    pub fn at(&self, id: NodeId) -> (f32, f32) {
        let r = self.tab().doc.rect_of(id).expect("o alvo pintado");
        (r.x + r.w * 0.5, r.y + r.h * 0.5)
    }

    pub fn tab(&self) -> &InputMapTab {
        self.hero
            .documents
            .surfaces()
            .find::<InputMapTab>()
            .expect("a família instalada")
    }

    /// A aba Input Map é a activa?
    pub fn active(&self) -> bool {
        self.hero.documents.surface_active().is_some_and(|t| {
            t.id == TAB_ID
                && self
                    .hero
                    .documents
                    .surfaces()
                    .tabs()
                    .iter()
                    .any(|(s, d)| *s == t && d.id == TAB_ID)
        }) && self.tab().is_open()
    }

    /// Um clique primário INTEIRO em `(x, y)` (Down, quadro, Up no mesmo ponto, quadro): o controlo do registo dispara
    /// no Up dentro (plano §13.3). Devolve se a aba tomou o Down.
    pub fn click(&mut self, at: (f32, f32)) -> bool {
        let took = self.down(at);
        self.button(PointerKind::Up, at);
        self.frame();
        took
    }

    /// Só o Down em `(x, y)`, seguido do quadro: começa um arrasto (`hover` move, `release` larga).
    pub fn down(&mut self, at: (f32, f32)) -> bool {
        let took = self.button(PointerKind::Down, at);
        self.frame();
        took
    }

    /// O Up em `(x, y)`, seguido do quadro (fora do controlo premido = cancela).
    pub fn up(&mut self, at: (f32, f32)) -> bool {
        let took = self.button(PointerKind::Up, at);
        self.frame();
        took
    }

    fn button(&mut self, kind: PointerKind, (x, y): (f32, f32)) -> bool {
        let ts = &mut self.ts;
        let Some((f, id)) = self.hero.documents.active_surface_mut() else {
            return false;
        };
        let input = PointerInput {
            text: ts,
            mods: Modifiers::default(),
            now_ns: 0,
        };
        f.pointer(id, input, kind, PointerButton::Primary, x, y)
    }

    /// O centro do segmento da vista `v` (da última pintura).
    pub fn segment(&self, v: View) -> (f32, f32) {
        let r = self.tab().doc.segment_rect(v).expect("o segmento pintado");
        (r.x + r.w * 0.5, r.y + r.h * 0.5)
    }

    /// O valor que a shell grava em `input_map_view=`.
    pub fn pref(&self) -> &'static str {
        ph2d_app_input_map::view(&self.hero).pref_key()
    }
}

/// Corre `f` no modo de texto dos testes e no de FÁBRICA do app (`CrispHeavyPlus`, com o −0,01 em de espaçamento):
/// um censo só no `Default` tem larguras exactas em f32 e não via o «F1» que a C4 cortava.
/// ⚠️ O `CrispHeavy` fica de fora: nele a dica do painel do jogador (`ph2d-player-controls`) corta-se a 304 px.
pub(super) fn em_cada_modo_de_texto(mut f: impl FnMut()) {
    use ph2d_tokens::TextRendering as R;
    for modo in [R::Default, R::CrispHeavyPlus] {
        eprintln!("modo de texto: {modo:?}");
        ph2d_text::set_active_text_rendering(modo);
        f();
    }
    ph2d_text::set_active_text_rendering(R::Default);
}
