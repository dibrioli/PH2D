//! **O diálogo New Sprite (Add ▸ Pixel Sprite, ADR-0196), do rato e do teclado ao pedido** — pelo
//! caminho REAL do ecrã: o hero pinta, o ponteiro carrega no meio do que se pintou, as teclas entram
//! pelo `handle_key`/`handle_text_input`, e cada evento passa pelo `apply_event`.
//!
//! As metades que já falharam noutro modal: todo controlo VIVO sob o rato (o 4096 do New Image);
//! OK arma o pedido COM o que se escreveu e fecha; Cancel, Esc e o clique fora NÃO armam nada; a
//! escolha fica lembrada na sessão.

use bumpalo::Bump;
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{ContextMenuKind, KEY_ENTER, KEY_ESCAPE, KEY_KEY_A};
use ph2d_editor_core::pixel_panel::{NewSpriteBackground, NewSpriteChoice, NewSpriteMode};
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
    fn aberto() -> Self {
        let mut e = Self {
            hero: HeroScreen::new(NodeId(1)),
            text: TextSystem::without_system_fonts(),
        };
        e.hero.store.open_new_sprite_dialog();
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
        let mut hits = Vec::new();
        let mut y = vp.y;
        while y < vp.y + vp.h {
            let mut x = vp.x;
            while x < vp.x + vp.w {
                if self.hero.hit_index.hit(x, y) == Some(id) {
                    hits.push((x, y));
                }
                x += 2.0;
            }
            y += 2.0;
        }
        #[expect(clippy::cast_precision_loss, reason = "uma contagem de amostras")]
        let n = hits.len() as f32;
        (!hits.is_empty()).then(|| {
            let sx: f32 = hits.iter().map(|p| p.0).sum();
            let sy: f32 = hits.iter().map(|p| p.1).sum();
            (sx / n, sy / n)
        })
    }

    fn carrega_em(&mut self, (x, y): (f32, f32)) {
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

    fn carrega(&mut self, id: NodeId) {
        let at = self
            .centro(id)
            .unwrap_or_else(|| panic!("{id:?} não está pintado no diálogo"));
        self.carrega_em(at);
    }

    fn tecla(&mut self, keycode: u32, ctrl: bool) {
        let arena = Bump::new();
        let e = KeyEvent {
            keycode,
            modifiers: Modifiers {
                ctrl,
                ..Modifiers::default()
            },
            kind: KeyKind::Down,
            timestamp_ns: 0,
        };
        let ev = self.hero.handle_key(e, &arena).to_vec();
        for e in ev {
            self.hero.apply_event(e);
        }
        self.pinta();
    }

    /// Carrega no campo, selecciona tudo e escreve `s` (o que o artista faz).
    fn escreve(&mut self, campo: NodeId, s: &str) {
        self.carrega(campo);
        self.tecla(KEY_KEY_A, true);
        for ch in s.chars() {
            let arena = Bump::new();
            let ev = self.hero.handle_text_input(ch, &arena).to_vec();
            for e in ev {
                self.hero.apply_event(e);
            }
        }
        self.pinta();
    }

    fn aberto_ainda(&self) -> bool {
        matches!(
            self.hero.store.context_menu().map(|r| r.kind),
            Some(ContextMenuKind::NewSpriteDialog)
        )
    }
}

fn id_do_modo(m: NewSpriteMode) -> NodeId {
    ids::NEW_SPRITE_MODES
        .iter()
        .find(|(x, ..)| *x == m)
        .map(|(_, id, _)| *id)
        .expect("o modo está na tabela")
}

fn id_do_fundo(b: NewSpriteBackground) -> NodeId {
    ids::NEW_SPRITE_BGS
        .iter()
        .find(|(x, ..)| *x == b)
        .map(|(_, id, _)| *id)
        .expect("o fundo está na tabela")
}

#[test]
fn todo_controlo_do_dialogo_esta_pintado_e_vivo_sob_o_rato() {
    let e = Ecra::aberto();
    let todos = ids::NEW_SPRITE_FIELDS
        .into_iter()
        .chain(ids::NEW_SPRITE_MODES.iter().map(|(_, id, _)| *id))
        .chain(ids::NEW_SPRITE_BGS.iter().map(|(_, id, _)| *id))
        .chain([ids::NEW_SPRITE_OK, ids::NEW_SPRITE_CANCEL]);
    for id in todos {
        assert!(e.centro(id).is_some(), "{id:?} não se pintou");
        assert!(
            e.hero.store.get(id).is_some(),
            "{id:?} pintado sem estado no store: MORTO sob o rato"
        );
    }
    // Abre com o *File ▸ New* do Aseprite: 32 × 32, RGBA, transparente.
    assert_eq!(e.hero.store.new_sprite_choice(), NewSpriteChoice::default());
    assert_eq!(e.hero.store.number_value(ids::NEW_SPRITE_WIDTH), Some(32.0));
}

#[test]
fn ok_com_valores_mudados_arma_o_pedido_com_esses_valores_e_fecha() {
    let mut e = Ecra::aberto();
    e.carrega(id_do_modo(NewSpriteMode::Indexed));
    e.carrega(id_do_fundo(NewSpriteBackground::White));
    assert!(e.aberto_ainda(), "escolher não pode fechar o diálogo");
    e.escreve(ids::NEW_SPRITE_WIDTH, "64");
    e.escreve(ids::NEW_SPRITE_HEIGHT, "48");
    e.carrega(ids::NEW_SPRITE_OK);
    let pedido = NewSpriteChoice {
        width: 64,
        height: 48,
        mode: NewSpriteMode::Indexed,
        background: NewSpriteBackground::White,
    };
    assert_eq!(e.hero.store.take_new_sprite_request(), Some(pedido));
    assert_eq!(
        e.hero.store.take_new_sprite_request(),
        None,
        "drenado uma vez só"
    );
    assert!(!e.aberto_ainda(), "o OK fecha");
    // Lembrado na sessão: reabre com a última escolha.
    e.hero.store.open_new_sprite_dialog();
    assert_eq!(e.hero.store.new_sprite_choice(), pedido);
    assert_eq!(
        e.hero.store.number_value(ids::NEW_SPRITE_HEIGHT),
        Some(48.0)
    );
}

#[test]
fn cancel_esc_e_o_clique_fora_nao_armam_nada() {
    let mut e = Ecra::aberto();
    e.carrega(id_do_modo(NewSpriteMode::Gray));
    e.carrega(ids::NEW_SPRITE_CANCEL);
    assert!(!e.aberto_ainda(), "o Cancel fecha");
    assert_eq!(e.hero.store.take_new_sprite_request(), None);

    let mut e = Ecra::aberto();
    e.escreve(ids::NEW_SPRITE_WIDTH, "100");
    e.tecla(KEY_ESCAPE, false);
    assert!(!e.aberto_ainda(), "o Esc fecha");
    assert_eq!(e.hero.store.take_new_sprite_request(), None);

    let mut e = Ecra::aberto();
    e.carrega_em((2.0, HERO_VIEWPORT_H * 0.5));
    assert!(!e.aberto_ainda(), "o clique fora fecha");
    assert_eq!(e.hero.store.take_new_sprite_request(), None);
}

/// Enter = OK, com o campo que se estava a escrever DENTRO; e o lado prende-se a `1..=65535`.
#[test]
fn enter_e_o_ok_com_o_que_se_escreveu_e_o_lado_tem_tecto_e_piso() {
    let mut e = Ecra::aberto();
    e.escreve(ids::NEW_SPRITE_WIDTH, "70000");
    e.tecla(KEY_ENTER, false);
    let c = e
        .hero
        .store
        .take_new_sprite_request()
        .expect("o Enter é o OK");
    assert_eq!(c.width, 65_535);
    assert!(!e.aberto_ainda());

    let mut e = Ecra::aberto();
    e.escreve(ids::NEW_SPRITE_HEIGHT, "0");
    e.tecla(KEY_ENTER, false);
    let c = e
        .hero
        .store
        .take_new_sprite_request()
        .expect("o Enter é o OK");
    assert_eq!(c.height, 1);
}

/// ⭐ Dono 06/11: a predefinição enche Width e Height, acende-se, e o OK nasce desse tamanho; quadrada ou ecrã.
/// *Mutação: o clique não escrever os campos ⇒ o pedido sai 32 × 32.*
#[test]
fn a_predefinicao_enche_os_campos_acende_e_o_ok_nasce_desse_tamanho() {
    for (id, [w, h]) in [
        (ids::NEW_SPRITE_SIZES[0].1, [16, 16]),
        (ids::NEW_SPRITE_SIZES[7].1, [256, 256]),
        (ids::NEW_SPRITE_SCREENS[0].1, [160, 144]),
        (ids::NEW_SPRITE_SCREENS[3].1, [640, 360]),
    ] {
        let mut e = Ecra::aberto();
        e.carrega(id);
        assert_eq!(
            e.hero.store.number_value(ids::NEW_SPRITE_WIDTH),
            Some(f64::from(w))
        );
        assert_eq!(
            e.hero.store.number_value(ids::NEW_SPRITE_HEIGHT),
            Some(f64::from(h))
        );
        assert_eq!(e.hero.store.new_sprite_size_lit(), id);
        assert!(e.aberto_ainda(), "a predefinição não fecha o diálogo");
        e.carrega(ids::NEW_SPRITE_OK);
        let c = e.hero.store.take_new_sprite_request().expect("o OK arma");
        assert_eq!([c.width, c.height], [w, h]);
    }
}

/// ⭐ Dono 06/11: um valor sem predefinição acende o Custom, a cada dígito; voltar a uma igual apaga-o.
/// *Mutação: `new_sprite_size_lit` não cair no Custom ⇒ RED.*
#[test]
fn escrever_um_valor_sem_predefinicao_acende_o_custom() {
    let mut e = Ecra::aberto();
    assert_eq!(
        e.hero.store.new_sprite_size_lit(),
        ids::NEW_SPRITE_SIZES[2].1,
        "32 × 32"
    );
    e.escreve(ids::NEW_SPRITE_WIDTH, "33");
    assert_eq!(e.hero.store.new_sprite_size_lit(), ids::NEW_SPRITE_CUSTOM);
    e.carrega(ids::NEW_SPRITE_OK);
    let c = e.hero.store.take_new_sprite_request().expect("o OK arma");
    assert_eq!([c.width, c.height], [33, 32]);

    e.hero.store.open_new_sprite_dialog();
    e.pinta();
    e.escreve(ids::NEW_SPRITE_WIDTH, "32");
    assert_eq!(
        e.hero.store.new_sprite_size_lit(),
        ids::NEW_SPRITE_SIZES[2].1
    );
}

/// O Custom aceso ou não, carregar nele leva ao Width: quem carrega quer escrever.
#[test]
fn o_custom_leva_ao_campo_width() {
    let mut e = Ecra::aberto();
    e.carrega(ids::NEW_SPRITE_CUSTOM);
    assert_eq!(e.hero.store.focus_id(), Some(ids::NEW_SPRITE_WIDTH));
    assert!(e.aberto_ainda());
}

impl Ecra {
    fn new_image() -> Self {
        let mut e = Self {
            hero: HeroScreen::new(NodeId(1)),
            text: TextSystem::without_system_fonts(),
        };
        e.hero.store.open_new_image_dialog();
        e.pinta();
        e
    }
}

/// ⭐ Dono 06/11: o Custom do New Image revela Width e Height, e 300 × 200 pede 300 × 200, pelo rato e pelo Enter.
/// *Mutação: `request_new_image` ignorar o Custom ⇒ sai 512 × 512.*
#[test]
fn o_custom_do_new_image_pede_os_lados_escritos() {
    let mut e = Ecra::new_image();
    assert!(
        e.centro(ids::NEW_IMAGE_WIDTH).is_none(),
        "fechado: sem campos"
    );
    e.carrega(ids::CTX_MENU_NEW_IMAGE_CUSTOM);
    assert!(e.hero.store.new_image_is_custom());
    e.escreve(ids::NEW_IMAGE_WIDTH, "300");
    e.escreve(ids::NEW_IMAGE_HEIGHT, "200");
    e.carrega(ids::CTX_MENU_NEW_IMAGE_CREATE);
    assert_eq!(
        e.hero.store.take_new_image_request(),
        Some((300, 200, 0)),
        "o Create leva o que se escreveu"
    );

    let mut e = Ecra::new_image();
    e.carrega(ids::CTX_MENU_NEW_IMAGE_CUSTOM);
    e.escreve(ids::NEW_IMAGE_WIDTH, "640");
    e.tecla(KEY_ENTER, false);
    assert_eq!(e.hero.store.take_new_image_request(), Some((640, 512, 0)));
    assert!(e.hero.store.context_menu().is_none(), "o Enter fecha");
}

/// A predefinição sai do Custom e esconde os campos; o lado nunca passa do que o dispositivo publicou.
/// *Mutação: tirar o `.min(max)` de `request_new_image` ⇒ o pedido passa do dispositivo.*
#[test]
fn a_predefinicao_sai_do_custom_e_o_lado_para_no_dispositivo() {
    let mut e = Ecra::new_image();
    e.carrega(ids::CTX_MENU_NEW_IMAGE_CUSTOM);
    e.carrega(ids::CTX_MENU_NEW_IMAGE_SIZE_256);
    assert!(!e.hero.store.new_image_is_custom());
    assert!(e.centro(ids::NEW_IMAGE_WIDTH).is_none());
    e.carrega(ids::CTX_MENU_NEW_IMAGE_CREATE);
    assert_eq!(e.hero.store.take_new_image_request(), Some((256, 256, 0)));

    let mut e = Ecra::new_image();
    e.hero.store.set_new_image_max_side(1000);
    e.carrega(ids::CTX_MENU_NEW_IMAGE_CUSTOM);
    e.hero
        .store
        .set_number_range(ids::NEW_IMAGE_WIDTH, 1.0, 1e6, 1.0);
    e.escreve(ids::NEW_IMAGE_WIDTH, "5000");
    e.carrega(ids::CTX_MENU_NEW_IMAGE_CREATE);
    assert_eq!(e.hero.store.take_new_image_request(), Some((1000, 512, 0)));
}
