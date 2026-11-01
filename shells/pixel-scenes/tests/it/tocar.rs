//! **P5 pelo caminho real**: as opções do tocar (a velocidade do documento, *Play Once*, *Rewind
//! on Stop*), os saltos dentro da tag e o *Set Loop Section* chegam ao documento pelo pedido do
//! painel, sobre a cena 16 (`tl.ase` do oráculo: 6 quadros 100…600 ms, tag `walk` nas posições
//! 1..=3), com o relógio do host passado à mão (`PixelApp::tick`).

use ph2d_app_pixel::{PixelApp, mode, panel};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::ToastQueue;
use ph2d_editor_core::object_mode::ObjectMode;
use ph2d_editor_core::pixel_panel::{
    PixelFrameAction as A, PixelPanelInfo, PixelPlayback, PixelRange, PixelRangeKind as K,
    PixelRequest, PixelSpeed,
};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_render::Camera2d;

use crate::comum::{PPM, WINDOW, spawn_root};

struct Palco {
    app: PixelApp,
    sim: SimWorld,
    hero: HeroScreen,
    toasts: ToastQueue,
    ids: Vec<u32>,
}

impl Palco {
    fn novo() -> Self {
        let mut app = PixelApp::default();
        let mut sim = SimWorld::new();
        assert!(ph2d_app_pixel::smoke::stage(
            &mut app, &mut sim, 16, spawn_root
        ));
        let (bits, key) = mode::sprites(sim.world())[0];
        let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
        hero.gizmo.mode.enter(bits, ObjectMode::Draw);
        let mut cam = Camera2d::default();
        app.frame(&mut sim, &mut cam, WINDOW, PPM, Some(bits), spawn_root);
        let s = &app.doc(key).expect("doc").sprite;
        let ids = s.frames().iter().map(|f| f.id.0).collect();
        let mut p = Self {
            app,
            sim,
            hero,
            toasts: ToastQueue::new(),
            ids,
        };
        p.quadro();
        p
    }

    fn quadro(&mut self) -> PixelPanelInfo {
        panel::frame(
            &mut self.app,
            self.sim.world(),
            &mut self.hero,
            &mut self.toasts,
        )
    }

    fn pede(&mut self, r: PixelRequest) -> PixelPanelInfo {
        self.app.pede(r);
        self.quadro()
    }

    /// A posição (base 0) do quadro activo.
    fn onde(&mut self) -> usize {
        let a = self.quadro().active_frame.expect("activo");
        self.ids.iter().position(|&i| i == a).expect("da cena")
    }

    fn vai(&mut self, pos: usize) {
        let _ = self.pede(PixelRequest::FrameSelect(self.ids[pos]));
    }

    fn opcoes(&mut self, f: impl FnOnce(&mut PixelPlayback)) {
        let mut o = self.quadro().playback;
        f(&mut o);
        let _ = self.pede(PixelRequest::Playback(o));
    }
}

/// *Play Once* fora da tag toca a linha inteira e PÁRA no quadro de partida; dentro da `walk`
/// (para sempre) salta para o início dela e volta ao de partida no fim.
#[test]
fn play_once_ends_on_the_frame_it_started_from() {
    let mut p = Palco::novo();
    p.opcoes(|o| o.once = true);
    p.vai(4);
    let info = p.pede(PixelRequest::FrameAction(A::Play));
    assert!(info.playing);
    assert_eq!(p.onde(), 0, "a linha inteira começa no 1.º");
    p.app.tick(100 + 200 + 300 + 400 + 500 + 600);
    let info = p.quadro();
    assert!(!info.playing, "uma passada só");
    assert_eq!(p.onde(), 4, "volta ao quadro de partida");

    p.vai(2);
    let _ = p.pede(PixelRequest::FrameAction(A::Play));
    assert_eq!(p.onde(), 1, "a raiz é a `walk`: começa no `from` dela");
    p.app.tick(200 + 300 + 400);
    assert!(!p.quadro().playing);
    assert_eq!(p.onde(), 2);
}

/// *Rewind on Stop*: parar volta ao quadro onde o tocar começou; desligado, fica onde ia. A
/// velocidade é do documento: a 2×, o quadro de 100 ms dura 50.
#[test]
fn rewind_on_stop_and_the_speed_of_the_document() {
    let mut p = Palco::novo();
    p.vai(0);
    p.opcoes(|o| o.speed = PixelSpeed::Two);
    assert_eq!(p.quadro().playback.speed, PixelSpeed::Two);
    let _ = p.pede(PixelRequest::FrameAction(A::Play));
    p.app.tick(49);
    assert_eq!(p.onde(), 0);
    p.app.tick(1);
    assert_eq!(p.onde(), 1, "100 ms ÷ 2");
    let _ = p.pede(PixelRequest::FrameAction(A::Play));
    assert_eq!(p.onde(), 1, "sem Rewind fica onde ia");

    p.opcoes(|o| o.rewind = true);
    p.vai(0);
    let _ = p.pede(PixelRequest::FrameAction(A::Play));
    p.app.tick(200);
    assert_ne!(p.onde(), 0);
    let info = p.pede(PixelRequest::FrameAction(A::Play));
    assert!(!info.playing);
    assert_eq!(p.onde(), 0, "Rewind on Stop");
}

/// *Jump to First/Last Frame in Tag*: as pontas da tag do activo; fora de tags, as da linha.
#[test]
fn first_and_last_in_tag_jump_to_the_ends_of_the_innermost_tag() {
    let mut p = Palco::novo();
    p.vai(2);
    let _ = p.pede(PixelRequest::FrameAction(A::FirstInTag));
    assert_eq!(p.onde(), 1);
    let _ = p.pede(PixelRequest::FrameAction(A::LastInTag));
    assert_eq!(p.onde(), 3);
    p.vai(5);
    let _ = p.pede(PixelRequest::FrameAction(A::FirstInTag));
    assert_eq!(p.onde(), 0);
    let _ = p.pede(PixelRequest::FrameAction(A::LastInTag));
    assert_eq!(p.onde(), 5);
}

/// *Set Loop Section* (F2): a tag «Loop» nasce sobre o intervalo, chamá-lo de novo MOVE-a, e sem
/// intervalo nada muda (o Trial).
#[test]
fn set_loop_section_creates_then_moves_the_loop_tag() {
    let mut p = Palco::novo();
    let tags = p.quadro().tags.len();
    let range = |ids: &[u32]| PixelRange {
        kind: K::Frames,
        layers: Vec::new(),
        frames: ids.to_vec(),
    };
    let _ = p.pede(PixelRequest::Range(range(&[p.ids[3], p.ids[4]])));
    let info = p.pede(PixelRequest::FrameAction(A::LoopSection));
    assert_eq!(info.tags.len(), tags + 1);
    let lp = info
        .tags
        .iter()
        .find(|t| t.name == "Loop")
        .expect("a tag Loop");
    assert_eq!((lp.from, lp.to, lp.repeat), (3, 4, 0));
    let _ = p.pede(PixelRequest::Range(range(&[p.ids[5]])));
    let info = p.pede(PixelRequest::FrameAction(A::LoopSection));
    assert_eq!(info.tags.len(), tags + 1, "a mesma tag, movida");
    let lp = info
        .tags
        .iter()
        .find(|t| t.name == "Loop")
        .expect("a tag Loop");
    assert_eq!((lp.from, lp.to), (5, 5));
    let _ = p.pede(PixelRequest::Range(PixelRange::default()));
    let antes = p.quadro().tags;
    let info = p.pede(PixelRequest::FrameAction(A::LoopSection));
    assert_eq!(info.tags, antes, "sem intervalo, nada");
}

/// A cena 17 (a entrada `s1` do Trial) toca pelo relógio do host a sequência do vídeo do Trial
/// (`s1_on_f0`: `0 1 2 3 4 3 2 5 6 7 | 0`, base 0) — o que o dono vê no smoke.
#[test]
fn scene_17_plays_the_nested_tags_as_the_trial_did() {
    let mut app = PixelApp::default();
    let mut sim = SimWorld::new();
    assert!(ph2d_app_pixel::smoke::stage(
        &mut app, &mut sim, 17, spawn_root
    ));
    let (bits, key) = mode::sprites(sim.world())[0];
    let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
    hero.gizmo.mode.enter(bits, ObjectMode::Draw);
    let mut cam = Camera2d::default();
    app.frame(&mut sim, &mut cam, WINDOW, PPM, Some(bits), spawn_root);
    let ids: Vec<u32> = app
        .doc(key)
        .expect("doc")
        .sprite
        .frames()
        .iter()
        .map(|f| f.id.0)
        .collect();
    let mut toasts = ToastQueue::new();
    let mut onde = |app: &mut PixelApp| {
        let a = panel::frame(app, sim.world(), &mut hero, &mut toasts).active_frame;
        ids.iter().position(|&i| Some(i) == a).expect("da cena")
    };
    assert_eq!(onde(&mut app), 0);
    app.pede(PixelRequest::FrameAction(A::Play));
    let mut seq = vec![onde(&mut app)];
    for _ in 0..10 {
        app.tick(100);
        seq.push(onde(&mut app));
    }
    assert_eq!(seq, [0, 1, 2, 3, 4, 3, 2, 5, 6, 7, 0]);
}
