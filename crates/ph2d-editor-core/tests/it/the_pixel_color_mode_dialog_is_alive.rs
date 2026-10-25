//! **O diálogo Color Mode do Pixel Art, do rato e do teclado ao pedido** — o mesmo caminho REAL do
//! irmão `the_new_sprite_dialog_is_alive`: o hero pinta, o ponteiro carrega no meio do que se
//! pintou, as teclas entram pelo `handle_key` e cada evento passa pelo `apply_event`.
//!
//! O que se prova: todo controlo do alvo escolhido está VIVO; o modo actual aparece mas não se
//! escolhe; OK e Enter armam o pedido com a escolha; Cancel e Esc não armam nada; a escolha fica.

use bumpalo::Bump;
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{ContextMenuKind, KEY_ENTER, KEY_ESCAPE};
use ph2d_editor_core::pixel_panel::{ColorModeChoice, NewSpriteMode, PixelDither, PixelToGray};
use ph2d_editor_core::screens::hero::{
    HERO_VIEWPORT_H, HERO_VIEWPORT_W, HeroScreen, paint_hero_screen,
};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{
    KeyEvent, KeyKind, Modifiers, PointerButton, PointerEvent, PointerKind, PointerSource,
};
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

struct Ecra {
    hero: HeroScreen,
    text: TextSystem,
}

impl Ecra {
    fn aberto(current: NewSpriteMode) -> Self {
        let mut e = Self {
            hero: HeroScreen::new(NodeId(1)),
            text: TextSystem::without_system_fonts(),
        };
        e.hero.store.open_color_mode_dialog(current);
        e.pinta();
        e
    }

    fn pinta(&mut self) {
        let mut scene = VectorScene::new();
        let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
        paint_hero_screen(&mut self.hero, vp, &mut scene, &mut self.text);
    }

    fn centro(&self, id: NodeId) -> Option<(f32, f32)> {
        let vp = self.hero.last_viewport;
        let (mut n, mut sx, mut sy) = (0u32, 0.0f32, 0.0f32);
        let mut y = vp.y;
        while y < vp.y + vp.h {
            let mut x = vp.x;
            while x < vp.x + vp.w {
                if self.hero.hit_index.hit(x, y) == Some(id) {
                    n += 1;
                    sx += x;
                    sy += y;
                }
                x += 2.0;
            }
            y += 2.0;
        }
        #[expect(clippy::cast_precision_loss, reason = "uma contagem de amostras")]
        let k = n as f32;
        (n > 0).then(|| (sx / k, sy / k))
    }

    fn carrega(&mut self, id: NodeId) {
        let (x, y) = self
            .centro(id)
            .unwrap_or_else(|| panic!("{id:?} não está pintado no diálogo"));
        let arena = Bump::new();
        let ptr = |kind| PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind,
            source: PointerSource::Mouse,
            button: PointerButton::Primary,
            timestamp_ns: 0,
        };
        let mut ev = self
            .hero
            .handle_pointer(ptr(PointerKind::Down), &arena)
            .to_vec();
        ev.extend_from_slice(self.hero.handle_pointer(ptr(PointerKind::Up), &arena));
        for e in ev {
            self.hero.apply_event(e);
        }
        self.pinta();
    }

    fn tecla(&mut self, keycode: u32) {
        let arena = Bump::new();
        let e = KeyEvent {
            keycode,
            modifiers: Modifiers::default(),
            kind: KeyKind::Down,
            timestamp_ns: 0,
        };
        let ev = self.hero.handle_key(e, &arena).to_vec();
        for e in ev {
            self.hero.apply_event(e);
        }
        self.pinta();
    }

    fn aberto_ainda(&self) -> bool {
        matches!(
            self.hero.store.context_menu().map(|r| r.kind),
            Some(ContextMenuKind::PixelColorModeDialog)
        )
    }
}

fn alvo(m: NewSpriteMode) -> NodeId {
    ids::COLOR_MODE_TARGETS
        .iter()
        .find(|(x, ..)| *x == m)
        .map(|(_, id, _)| *id)
        .expect("o alvo está na tabela")
}

#[test]
fn todo_controlo_do_alvo_esta_vivo_e_o_modo_actual_nao_se_escolhe() {
    let mut e = Ecra::aberto(NewSpriteMode::Rgba);
    assert_eq!(
        e.hero.store.color_mode_choice().target,
        NewSpriteMode::Indexed,
        "o alvo nunca é o modo actual"
    );
    let vivos = ids::COLOR_MODE_DITHERS
        .iter()
        .map(|(_, id, _)| *id)
        .chain([alvo(NewSpriteMode::Gray), alvo(NewSpriteMode::Indexed)])
        .chain([ids::COLOR_MODE_OK, ids::COLOR_MODE_CANCEL]);
    for id in vivos {
        assert!(e.centro(id).is_some(), "{id:?} não se pintou");
        assert!(e.hero.store.get(id).is_some(), "{id:?} sem estado: MORTO");
    }
    assert!(
        e.centro(alvo(NewSpriteMode::Rgba)).is_none(),
        "o modo actual não tem clique"
    );
    // Grayscale troca a fila dos dithers pela dos métodos.
    e.carrega(alvo(NewSpriteMode::Gray));
    assert!(e.aberto_ainda(), "escolher não fecha");
    for (_, id, _) in ids::COLOR_MODE_GRAYS {
        assert!(e.centro(id).is_some(), "{id:?} não se pintou");
    }
    assert!(e.centro(ids::COLOR_MODE_DITHERS[0].1).is_none());
}

#[test]
fn ok_e_enter_armam_o_pedido_com_a_escolha_e_ela_fica_lembrada() {
    let mut e = Ecra::aberto(NewSpriteMode::Rgba);
    e.carrega(ids::COLOR_MODE_DITHERS[7].1);
    e.carrega(ids::COLOR_MODE_OK);
    let quer = ColorModeChoice {
        target: NewSpriteMode::Indexed,
        dither: PixelDither::FloydSteinberg,
        gray: PixelToGray::Luminance,
    };
    assert_eq!(e.hero.store.take_color_mode_request(), Some(quer));
    assert_eq!(e.hero.store.take_color_mode_request(), None, "uma vez só");
    assert!(!e.aberto_ainda(), "o OK fecha");

    let mut e = Ecra::aberto(NewSpriteMode::Indexed);
    e.carrega(alvo(NewSpriteMode::Gray));
    e.carrega(ids::COLOR_MODE_GRAYS[1].1);
    e.tecla(KEY_ENTER);
    let c = e
        .hero
        .store
        .take_color_mode_request()
        .expect("o Enter é o OK");
    assert_eq!((c.target, c.gray), (NewSpriteMode::Gray, PixelToGray::Hsv));
    assert!(!e.aberto_ainda());
    // Lembrado: reabre com o método escolhido.
    e.hero.store.open_color_mode_dialog(NewSpriteMode::Rgba);
    assert_eq!(e.hero.store.color_mode_choice().gray, PixelToGray::Hsv);
}

#[test]
fn cancel_e_esc_nao_armam_nada() {
    let mut e = Ecra::aberto(NewSpriteMode::Rgba);
    e.carrega(ids::COLOR_MODE_CANCEL);
    assert!(!e.aberto_ainda(), "o Cancel fecha");
    assert_eq!(e.hero.store.take_color_mode_request(), None);

    let mut e = Ecra::aberto(NewSpriteMode::Rgba);
    e.tecla(KEY_ESCAPE);
    assert!(!e.aberto_ainda(), "o Esc fecha");
    assert_eq!(e.hero.store.take_color_mode_request(), None);
}
