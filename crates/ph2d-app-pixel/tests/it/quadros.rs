//! **P4 pelo caminho real**: a cena 8, os pedidos da linha do tempo pelo [`panel::frame`] da shell,
//! as teclas e o relógio do tocar. A pergunta de cada teste é a do smoke do dono: *o que muda?*

use ph2d_app_pixel::input::{self, KeyAnswer};
use ph2d_app_pixel::smoke_quadros::{
    COLORS, FRAMES, GROUND, HOLD_INDEX, HOLD_MS, PILLAR, ball_center,
};
use ph2d_app_pixel::{PixelApp, mode, panel};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::ToastQueue;
use ph2d_editor_core::object_mode::ObjectMode;
use ph2d_editor_core::pixel_panel::{PixelFrameAction as A, PixelPanelInfo, PixelRequest};
use ph2d_editor_core::screens::hero::{HeroScreen, PANEL_PIXEL_TIMELINE};
use ph2d_pixel_core::{Op, Patch, Purpose, Rect, compose_window};
use ph2d_render::Camera2d;
use winit::keyboard::KeyCode;

use crate::edit::{PPM, WINDOW, spawn_root};

struct Palco {
    app: PixelApp,
    sim: SimWorld,
    hero: HeroScreen,
    toasts: ToastQueue,
    bits: u64,
    key: u32,
}

impl Palco {
    fn novo() -> Self {
        let mut app = PixelApp::default();
        let mut sim = SimWorld::new();
        assert!(ph2d_app_pixel::smoke::stage(&mut app, &mut sim, 8, spawn_root), "a cena 8 monta-se");
        let (bits, key) = mode::sprites(sim.world())[0];
        let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
        hero.gizmo.mode.enter(bits, ObjectMode::Draw);
        let mut cam = Camera2d::default();
        app.frame(&mut sim, &mut cam, WINDOW, PPM, Some(bits), spawn_root);
        let mut p = Self { app, sim, hero, toasts: ToastQueue::new(), bits, key };
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

    fn tecla(&mut self, k: KeyCode) {
        let a = input::key(&mut self.app, self.sim.world(), Some(self.bits), k, false, false);
        assert!(matches!(a, Some(KeyAnswer::Quiet)), "{k:?} é da linha do tempo");
        self.quadro();
    }

    fn sprite(&self) -> &ph2d_pixel_core::Sprite {
        &self.app.doc(self.key).expect("doc").sprite
    }

    fn activo(&self) -> usize {
        let d = self.app.doc(self.key).expect("doc");
        d.sprite.frame_index(d.frame).expect("existe")
    }

    /// O RGBA da célula `(x, y)` composta no frame da posição `i`.
    fn px(&self, i: usize, x: i32, y: i32) -> [u8; 4] {
        let s = self.sprite();
        let v = compose_window(s, s.frames()[i].id, None, Rect::new(x, y, 1, 1), Purpose::View);
        [v[0], v[1], v[2], v[3]]
    }
}

#[test]
fn scene_8_shows_each_thing_of_the_wave() {
    let mut p = Palco::novo();
    let info = p.quadro();
    assert_eq!((info.frames.len(), info.tags.len()), (FRAMES, 4));
    assert!(info.onion && info.onion_tint, "o papel vegetal começa ligado, com tinta");
    assert_eq!(info.active_frame, Some(p.sprite().frames()[0].id.0));
    // o chão: UMA imagem nos 16 frames (a pílula da linha do tempo)
    let chao = p.sprite().layers()[0].id;
    let dono: Vec<u32> = info.cels.iter().filter(|c| c.layer == chao.0).map(|c| c.owner).collect();
    assert_eq!(dono.len(), FRAMES);
    assert!(dono.iter().all(|&o| o == dono[0]), "o chão é um cel ligado em todos");
    // o pilar tapa a bola na 2.ª posição, salvo no verde (z +1)
    let (cx, cy) = ball_center(1);
    assert!(PILLAR.contains(cx, cy));
    assert_eq!(p.px(1, cx, cy)[..3], [110, 110, 125], "vermelho: a bola por trás do pilar");
    assert_eq!(p.px(5, cx, cy), COLORS[1], "verde: a bola à frente do pilar");
    // o azul a 50 %, o vermelho opaco
    let (bx, by) = ball_center(0);
    assert_eq!(p.px(0, bx, by), COLORS[0]);
    assert_eq!(p.px(8, bx, by)[3], 128, "azul: o cel a 50 %");
    // o frame que segura
    assert_eq!(info.frames[HOLD_INDEX].duration_ms, HOLD_MS);
    // pintar o chão num frame muda-o em todos
    let antes = p.px(0, 1, GROUND.y);
    let f10 = p.sprite().frames()[10].id;
    let r = Rect::new(1, GROUND.y, 1, 1);
    let bbox = p.sprite().cel((chao, f10)).map(|c| c.bbox);
    p.app.commit(p.key, Op::Patch(Patch { layer: chao, frame: f10, bbox, rect: r, pixels: vec![0, 0, 0, 255] })).expect("pinta");
    assert_ne!(p.px(0, 1, GROUND.y), antes, "o chão ligado mudou também no frame 1");
}

#[test]
fn enter_plays_the_tags_with_their_passes_and_the_hold_by_the_host_clock() {
    let mut p = Palco::novo();
    p.tecla(KeyCode::Enter);
    assert!(p.quadro().playing);
    let mut seq = vec![p.activo()];
    for _ in 0..40 {
        let d = p.sprite().frames()[p.activo()].duration_ms;
        p.app.tick(d);
        seq.push(p.activo());
    }
    let want: Vec<usize> = [0, 1, 2, 3, 0, 1, 2, 3, 7, 6, 5, 4, 7, 6, 5, 4, 8, 9, 10, 11, 10, 9, 8, 15, 14, 13, 12, 13, 14, 15]
        .into_iter()
        .chain([14, 13, 12, 13, 14, 15, 14, 13, 12, 13, 14])
        .collect();
    assert_eq!(seq, want);
    // o frame que segura: 120 ms não chegam para sair dele
    let mut q = Palco::novo();
    q.tecla(KeyCode::End);
    q.tecla(KeyCode::Home);
    for _ in 0..HOLD_INDEX {
        q.tecla(KeyCode::ArrowRight);
    }
    q.tecla(KeyCode::Enter);
    q.app.tick(HOLD_MS - 1);
    assert_eq!(q.activo(), HOLD_INDEX, "ainda no frame que segura");
    q.tecla(KeyCode::Enter);
    assert!(!q.quadro().playing, "Enter de novo pára");
}

#[test]
fn frame_actions_are_one_undo_step_and_the_active_frame_stays_valid() {
    let mut p = Palco::novo();
    p.tecla(KeyCode::ArrowLeft);
    assert_eq!(p.activo(), FRAMES - 1, "← no primeiro vai para o último");
    p.tecla(KeyCode::Home);
    let n = p.sprite().frames().len();
    p.pede(PixelRequest::FrameAction(A::NewFrame));
    assert_eq!((p.sprite().frames().len(), p.activo()), (n + 1, 1), "o novo é o activo");
    p.pede(PixelRequest::FrameAction(A::Delete));
    assert_eq!(p.sprite().frames().len(), n);
    assert!(p.app.undo(p.key), "desfaz o apagar");
    assert!(p.app.undo(p.key), "desfaz o novo");
    assert_eq!(p.sprite().frames().len(), n);
    assert!(p.activo() < n, "o activo continua a existir");
    // Cel Properties: a opacidade vai à imagem partilhada
    p.tecla(KeyCode::Home);
    let ball = p.sprite().layers()[1].id;
    p.pede(PixelRequest::CelSelect { layer: ball.0, frame: p.sprite().frames()[0].id.0 });
    p.pede(PixelRequest::CelOpacity(64));
    assert_eq!(p.sprite().cel((ball, p.sprite().frames()[0].id)).map(|c| c.opacity), Some(64));
    p.pede(PixelRequest::FrameAction(A::Next));
    p.pede(PixelRequest::FrameAction(A::LinkPrevious));
    let (f0, f1) = (p.sprite().frames()[0].id, p.sprite().frames()[1].id);
    assert_eq!(p.sprite().owner((ball, f1)), (ball, f0), "ligado ao anterior");
    p.pede(PixelRequest::FrameAction(A::Unlink));
    assert_eq!(p.sprite().owner((ball, f1)), (ball, f1));
}

#[test]
fn f3_tab_and_the_draw_edge_drive_the_onion_and_the_timeline_panel() {
    let mut p = Palco::novo();
    let aberto = |p: &Palco| p.hero.panel_visibility.get(PANEL_PIXEL_TIMELINE).copied().unwrap_or(false);
    assert!(aberto(&p), "a linha do tempo abre ao entrar em Draw");
    p.tecla(KeyCode::Tab);
    assert!(!aberto(&p), "Tab esconde");
    p.tecla(KeyCode::Tab);
    assert!(aberto(&p), "Tab mostra");
    p.tecla(KeyCode::F3);
    assert!(!p.quadro().onion, "F3 desliga o papel vegetal");
    p.tecla(KeyCode::F3);
    assert!(p.quadro().onion);
}
