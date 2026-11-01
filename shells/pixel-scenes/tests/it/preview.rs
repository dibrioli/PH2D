//! **A janela Preview pelo caminho real** (P5, Trial 1.3.18 `ronda3b.preview_s0`), sobre a cena 16
//! (`tl.ase`: 6 quadros 100…600 ms): Shift+Enter toca SÓ a Preview — o quadro activo do documento
//! não anda —, parar volta ao activo, parada ela segue o activo, e o canal da imagem só publica com
//! a janela aberta.

use ph2d_app_pixel::{PixelApp, input, mode, panel};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::ToastQueue;
use ph2d_editor_core::object_mode::ObjectMode;
use ph2d_editor_core::pixel_panel::{PixelPanelInfo, PixelRequest};
use ph2d_editor_core::screens::hero::{HeroScreen, PANEL_PIXEL_PREVIEW};
use ph2d_panel_pixel_preview::current;
use ph2d_render::Camera2d;
use winit::keyboard::KeyCode;

use crate::comum::{PPM, WINDOW, spawn_root};

struct Palco {
    app: PixelApp,
    sim: SimWorld,
    hero: HeroScreen,
    toasts: ToastQueue,
    bits: u64,
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
        let ids = app
            .doc(key)
            .expect("doc")
            .sprite
            .frames()
            .iter()
            .map(|f| f.id.0)
            .collect();
        let mut p = Self {
            app,
            sim,
            hero,
            toasts: ToastQueue::new(),
            bits,
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

    fn tecla(&mut self, k: KeyCode, shift: bool) {
        let a = input::key(
            &mut self.app,
            self.sim.world(),
            Some(self.bits),
            k,
            false,
            shift,
        );
        assert!(a.is_some(), "{k:?} é da família");
        self.quadro();
    }

    fn aberta(&self) -> bool {
        self.hero
            .panel_visibility
            .get(PANEL_PIXEL_PREVIEW)
            .copied()
            .unwrap_or(false)
    }

    fn activo(&mut self) -> u32 {
        self.quadro().active_frame.expect("activo")
    }

    /// O frame que a Preview mostra (o canal).
    fn mostra(&self) -> u32 {
        current().expect("a Preview publica").frame
    }
}

/// Shift+Enter abre a Preview e toca-a; o relógio anda a Preview e NÃO o documento; o segundo
/// Shift+Enter pára-a e ela volta ao quadro activo.
#[test]
fn shift_enter_toca_so_a_preview_e_o_activo_do_documento_nao_anda() {
    let mut p = Palco::novo();
    assert!(!p.aberta(), "a Preview nasce fechada");
    let a = p.activo();
    p.tecla(KeyCode::Enter, true);
    assert!(p.aberta(), "Shift+Enter abre a Preview");
    let f = current().expect("publica com a janela aberta");
    assert!(f.playing, "e toca-a");
    assert_eq!(f.frame, a, "a partir do activo");
    let mut vistos = vec![f.frame];
    for _ in 0..8 {
        p.app.tick(150);
        p.quadro();
        assert_eq!(p.activo(), a, "o quadro activo do documento NÃO anda");
        assert!(!p.quadro().playing, "o editor fica parado");
        vistos.push(p.mostra());
    }
    assert!(
        vistos.iter().any(|&v| v != a),
        "a Preview andou: {vistos:?} (ids {:?})",
        p.ids
    );
    p.tecla(KeyCode::Enter, true);
    let f = current().expect("ainda aberta");
    assert!(!f.playing, "o segundo Shift+Enter pára-a");
    assert_eq!(f.frame, a, "parada, volta ao activo");
}

/// Com a Preview parada, mudar o activo (o clique num cabeçalho) muda o que ela mostra.
#[test]
fn com_a_preview_parada_ela_segue_o_quadro_activo() {
    let mut p = Palco::novo();
    p.tecla(KeyCode::F7, false);
    assert!(p.aberta(), "F7 abre a Preview");
    for pos in [2, 4, 0] {
        p.app.pede(PixelRequest::FrameSelect(p.ids[pos]));
        p.quadro();
        assert_eq!(
            p.mostra(),
            p.ids[pos],
            "a Preview segue o activo (posição {pos})"
        );
    }
}

/// O canal só publica com a janela aberta: fechada (o `×`, o F7), a imagem sai e o tocar pára.
#[test]
fn o_canal_da_preview_so_publica_com_a_janela_aberta() {
    let mut p = Palco::novo();
    assert_eq!(current(), None, "fechada não compõe nada");
    p.tecla(KeyCode::Enter, true);
    let f = current().expect("aberta publica");
    assert_eq!((f.w as usize * f.h as usize * 4), f.rgba.len());
    p.hero.panel_visibility.insert(PANEL_PIXEL_PREVIEW, false);
    p.quadro();
    assert_eq!(current(), None, "o × fecha a imagem");
    p.tecla(KeyCode::F7, false);
    assert!(
        !current().expect("reaberta").playing,
        "fechar parou o tocar"
    );
    p.tecla(KeyCode::F7, false);
    assert!(!p.aberta() && current().is_none(), "F7 fecha-a outra vez");
}
