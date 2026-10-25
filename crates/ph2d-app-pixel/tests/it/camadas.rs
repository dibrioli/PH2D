//! **P3 pelo caminho real**: a cena 7 e os pedidos das camadas chegam ao documento pelo
//! [`panel::frame`] da shell. ⭐ A pergunta de cada teste é a do smoke do dono: *o que muda quando
//! ele clica?* (na P2 a cena 3 não mostrava o Color Mode).

use ph2d_app_pixel::compose::Mods;
use ph2d_app_pixel::input::{self, Button, Press};
use ph2d_app_pixel::smoke_camadas::{
    GROUP_A, GROUP_B, HIDDEN_STAR, LOCKED_BOX, OPACITY_BOX, REFERENCE_BOX, stripe_rect,
};
use ph2d_app_pixel::{PixelApp, mode, panel};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::ToastQueue;
use ph2d_editor_core::object_mode::ObjectMode;
use ph2d_editor_core::pixel_panel::{PixelBlend, PixelLayerAction, PixelPanelInfo, PixelRequest};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_pixel_core::{Purpose, Rect, compose_window};
use ph2d_render::Camera2d;

use crate::edit::{PPM, WINDOW, at, canvas, spawn_root};

struct Palco {
    app: PixelApp,
    sim: SimWorld,
    hero: HeroScreen,
    toasts: ToastQueue,
    cam: Camera2d,
    bits: u64,
    key: u32,
}

impl Palco {
    fn novo() -> Self {
        let mut app = PixelApp::default();
        let mut sim = SimWorld::new();
        assert!(ph2d_app_pixel::smoke::stage(&mut app, &mut sim, 7, spawn_root), "a cena 7 monta-se");
        let (bits, key) = mode::sprites(sim.world())[0];
        let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
        hero.gizmo.mode.enter(bits, ObjectMode::Draw);
        let mut cam = Camera2d::default();
        app.frame(&mut sim, &mut cam, WINDOW, PPM, Some(bits), spawn_root);
        let mut p = Self { app, sim, hero, toasts: ToastQueue::new(), cam, bits, key };
        p.quadro();
        p
    }

    fn quadro(&mut self) -> PixelPanelInfo {
        panel::frame(&mut self.app, self.sim.world(), &mut self.hero, &mut self.toasts)
    }

    fn pede(&mut self, r: PixelRequest) -> PixelPanelInfo {
        self.app.pede(r);
        self.quadro()
    }

    fn janela(&self, r: Rect) -> Vec<u8> {
        let s = &self.app.doc(self.key).expect("doc").sprite;
        compose_window(s, s.frames()[0].id, None, r, Purpose::View)
    }

    fn id(&self, nome: &str) -> u32 {
        let s = &self.app.doc(self.key).expect("doc").sprite;
        s.layers().iter().find(|l| l.name == nome).unwrap_or_else(|| panic!("{nome}")).id.0
    }

    fn passos(&self) -> usize {
        self.app.doc(self.key).expect("doc").history.undo_len()
    }
}

#[test]
fn the_19_mode_stripes_are_pairwise_different_on_screen() {
    let p = Palco::novo();
    let faixas: Vec<Vec<u8>> = (0..19).map(|i| p.janela(stripe_rect(i))).collect();
    for i in 0..19 {
        for j in i + 1..19 {
            assert_ne!(faixas[i], faixas[j], "as faixas {i} e {j} mostram o mesmo — a cena não ensina");
        }
    }
    let info = Palco::novo().quadro();
    let grupo = info.layers.iter().find(|r| r.name == "19 modes").expect("o grupo");
    assert!(grupo.group && grupo.collapsed, "o grupo nasce recolhido");
    assert!(!info.layers.iter().any(|r| r.blend == PixelBlend::Divide), "recolhido esconde as 19 linhas");
}

#[test]
fn every_thing_in_the_bottom_band_changes_what_is_seen_when_clicked() {
    let mut p = Palco::novo();
    // a foto de 25/10: pintar a banda de cima cortava a Background — a de baixo tem de ser branca
    assert_eq!(p.janela(Rect::new(0, 95, 1, 1)), vec![255, 255, 255, 255], "o fundo branco em baixo");
    // a opacidade: 128 mostra metade; 255 mostra o azul inteiro
    let meio = p.janela(OPACITY_BOX);
    let info = p.pede(PixelRequest::LayerOpacity(255));
    assert_eq!(info.active_layer, Some(p.id("Opacity 50%")), "a activa é a do quadrado");
    assert_ne!(p.janela(OPACITY_BOX), meio, "a opacidade muda o quadrado");
    // o olho da estrela
    let sem = p.janela(HIDDEN_STAR);
    p.pede(PixelRequest::LayerVisible(p.id("Hidden star")));
    assert_ne!(p.janela(HIDDEN_STAR), sem, "o olho acende a estrela");
    // o olho do grupo apaga as duas formas
    let (a, b) = (p.janela(GROUP_A), p.janela(GROUP_B));
    p.pede(PixelRequest::LayerVisible(p.id("Group")));
    assert!(p.janela(GROUP_A) != a && p.janela(GROUP_B) != b, "o grupo escondido esconde as duas");
    // o Solo: só o quadrado; outra vez, tudo volta
    let antes = p.janela(Rect::new(0, 0, 190, 96));
    p.pede(PixelRequest::LayerSolo(p.id("Locked")));
    assert_ne!(p.janela(Rect::new(0, 0, 190, 96)), antes, "o Solo esconde as outras");
    assert_eq!(p.janela(LOCKED_BOX).len(), LOCKED_BOX.area() * 4);
    p.pede(PixelRequest::LayerSolo(p.id("Locked")));
    assert_eq!(p.janela(Rect::new(0, 0, 190, 96)), antes, "o segundo Solo devolve a vista");
    // a referência fica fora do Flatten e continua a ver-se
    let r = p.janela(REFERENCE_BOX);
    let n = p.passos();
    p.pede(PixelRequest::LayerAction(PixelLayerAction::Flatten));
    assert_eq!(p.passos(), n + 1, "o Flatten é UM passo");
    assert_eq!(p.janela(REFERENCE_BOX), r, "a referência continua a ver-se");
    let s = &p.app.doc(p.key).expect("doc").sprite;
    assert_eq!(s.layers().len(), 2, "a imagem achatada e a referência");
}

#[test]
fn painting_on_the_locked_layer_is_refused_and_says_why() {
    let mut p = Palco::novo();
    p.pede(PixelRequest::LayerSelect(p.id("Locked")));
    let antes = p.janela(LOCKED_BOX);
    let pt = at(&p.app, &p.sim, &p.cam, p.bits, LOCKED_BOX.x + 3, LOCKED_BOX.y + 3);
    let c = canvas(&p.sim, &p.cam, p.bits);
    assert_eq!(input::press(&mut p.app, &c, pt, Button::Primary, Mods::default()), Press::Done);
    let _ = input::release(&mut p.app);
    p.quadro();
    assert_eq!(p.janela(LOCKED_BOX), antes, "nada pintou");
    assert!(p.toasts.len() > 0, "e disse porquê");
}

#[test]
fn layer_actions_from_the_panel_are_one_undo_step_each() {
    let mut p = Palco::novo();
    for a in [
        PixelLayerAction::NewLayer,
        PixelLayerAction::Duplicate,
        PixelLayerAction::MoveDown,
        PixelLayerAction::MergeDown,
        PixelLayerAction::NewGroup,
        PixelLayerAction::Delete,
    ] {
        let n = p.passos();
        p.pede(PixelRequest::LayerAction(a));
        assert_eq!(p.passos(), n + 1, "{a:?} é UM passo");
    }
    // a cópia precisa de selecção, como no Aseprite
    let n = p.passos();
    p.pede(PixelRequest::LayerAction(PixelLayerAction::ViaCopy));
    assert_eq!(p.passos(), n, "sem selecção não copia");
    let info = p.pede(PixelRequest::LayerAction(PixelLayerAction::SelectAll));
    assert!(info.has_selection);
    p.pede(PixelRequest::LayerAction(PixelLayerAction::ViaCopy));
    assert_eq!(p.passos(), n + 1, "com selecção, via Copy é um passo");
    let info = p.pede(PixelRequest::LayerAction(PixelLayerAction::Copy));
    assert!(info.has_clip, "o Copy acende o Paste");
    p.pede(PixelRequest::LayerAction(PixelLayerAction::PasteAsReference));
    let info = p.quadro();
    assert!(info.layers.iter().any(|r| r.reference && r.active_check(info.active_layer)));
}

trait Activa {
    fn active_check(&self, a: Option<u32>) -> bool;
}

impl Activa for ph2d_editor_core::pixel_panel::PixelLayerRow {
    fn active_check(&self, a: Option<u32>) -> bool {
        a == Some(self.id)
    }
}

#[test]
fn renaming_and_mode_reach_the_document() {
    let mut p = Palco::novo();
    let id = p.id("Opacity 50%");
    let info = p.pede(PixelRequest::LayerRename { id, name: "  Moon  ".to_owned() });
    assert!(info.layers.iter().any(|r| r.id == id && r.name == "Moon"), "o nome aparado");
    let info = p.pede(PixelRequest::LayerBlend(PixelBlend::Difference));
    assert_eq!(info.layers.iter().find(|r| r.id == id).map(|r| r.blend), Some(PixelBlend::Difference));
    assert!(ph2d_app_pixel::camadas::blend_to(PixelBlend::Divide).code() == 18);
}
