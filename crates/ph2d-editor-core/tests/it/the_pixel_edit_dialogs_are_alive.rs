//! **Os quatro diálogos do Pixel P6 (Color Range, Modify, Canvas Size, Sprite Size), do rato e do
//! teclado ao pedido** — o caminho REAL dos irmãos `the_pixel_color_mode_dialog_is_alive` e
//! `the_new_sprite_dialog_is_alive`: o hero pinta, o ponteiro carrega no meio do que se pintou, as
//! teclas entram pelo `handle_key`/`handle_text_input` e cada evento passa pelo `apply_event`.
//!
//! O que se prova, por diálogo: todo controlo pintado e VIVO; OK e Enter armam o pedido EXACTO com
//! o que se escreveu e clicou; Cancel e Esc não armam nada; a sincronia dos campos (Canvas Size:
//! a lei `left = trunc(dw/2)`, `top = dh − trunc(dh/2)` e a âncora de canto, a borda que muda a
//! largura; Sprite Size: px ↔ % e o Lock Ratio); nenhum rótulo se corta na largura da caixa.

use bumpalo::Bump;
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{ContextMenuKind, KEY_ENTER, KEY_ESCAPE, KEY_KEY_A};
use ph2d_editor_core::pixel_panel::{
    PixelDialogChoice, PixelDialogKind, PixelModify, PixelModifyBrush, PixelSizeMethod,
};
use ph2d_editor_core::screens::hero::{
    HERO_VIEWPORT_H, HERO_VIEWPORT_W, HeroScreen, paint_hero_screen,
};
use ph2d_editor_core::text_elide::elisao;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{
    KeyEvent, KeyKind, Modifiers, PointerButton, PointerEvent, PointerKind, PointerSource,
};
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

const MODIFIES: [PixelModify; 3] = [
    PixelModify::Expand,
    PixelModify::Contract,
    PixelModify::Border,
];

struct Ecra {
    hero: HeroScreen,
    text: TextSystem,
}

impl Ecra {
    fn aberto(kind: PixelDialogKind, size: (u32, u32)) -> Self {
        let mut e = Self {
            hero: HeroScreen::new(NodeId(1)),
            text: TextSystem::without_system_fonts(),
        };
        e.hero.store.open_pixel_edit_dialog(kind, size);
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

    fn valor(&self, id: NodeId) -> f64 {
        self.hero.store.number_value(id).expect("campo registado")
    }

    fn aberto_ainda(&self) -> bool {
        matches!(
            self.hero.store.context_menu().map(|r| r.kind),
            Some(ContextMenuKind::PixelEditDialog)
        )
    }

    fn pedido(&mut self) -> Option<PixelDialogChoice> {
        self.hero.store.take_pixel_edit_request()
    }
}

fn kinds() -> Vec<PixelDialogKind> {
    let mut v = vec![PixelDialogKind::ColorRange];
    v.extend(MODIFIES.map(PixelDialogKind::Modify));
    v.extend([PixelDialogKind::CanvasSize, PixelDialogKind::SpriteSize]);
    v
}

/// Os controlos que o diálogo `kind` pinta.
fn controlos(kind: PixelDialogKind) -> Vec<NodeId> {
    let mut v = vec![ids::PIXEL_EDIT_OK, ids::PIXEL_EDIT_CANCEL];
    match kind {
        PixelDialogKind::ColorRange => v.push(ids::PIXEL_EDIT_TOLERANCE),
        PixelDialogKind::Modify(_) => {
            v.push(ids::PIXEL_EDIT_MODIFY_N);
            v.extend(ids::PIXEL_EDIT_BRUSHES.iter().map(|(_, id, _)| *id));
        }
        PixelDialogKind::CanvasSize => {
            v.extend([ids::PIXEL_EDIT_CANVAS_W, ids::PIXEL_EDIT_CANVAS_H]);
            v.extend(ids::PIXEL_EDIT_BORDERS);
            v.extend(ids::PIXEL_EDIT_ANCHORS.iter().map(|(_, id, _)| *id));
        }
        PixelDialogKind::SpriteSize => {
            v.extend([
                ids::PIXEL_EDIT_LOCK,
                ids::PIXEL_EDIT_SPRITE_W,
                ids::PIXEL_EDIT_SPRITE_H,
                ids::PIXEL_EDIT_SPRITE_WPCT,
                ids::PIXEL_EDIT_SPRITE_HPCT,
            ]);
            v.extend(ids::PIXEL_EDIT_METHODS.iter().map(|(_, id, _)| *id));
        }
    }
    v
}

fn ancora(a: (u8, u8)) -> NodeId {
    ids::PIXEL_EDIT_ANCHORS
        .iter()
        .find(|(x, ..)| *x == a)
        .map(|(_, id, _)| *id)
        .expect("a âncora está na tabela")
}

#[test]
fn todo_controlo_de_cada_dialogo_esta_pintado_e_vivo() {
    for kind in kinds() {
        let e = Ecra::aberto(kind, (32, 24));
        assert!(e.aberto_ainda(), "{kind:?}: abre");
        assert_eq!(e.hero.store.pixel_edit_dialog().kind, kind);
        for id in controlos(kind) {
            assert!(e.centro(id).is_some(), "{kind:?}: {id:?} sem clique");
            assert!(
                e.hero.store.get(id).is_some(),
                "{kind:?}: {id:?} sem estado: MORTO"
            );
            assert!(ids::pixel_edit_dialog_owns(id), "{id:?} fora do dono");
        }
    }
}

#[test]
fn color_range_ok_e_enter_com_a_tolerancia_escrita_e_lembrada() {
    let mut e = Ecra::aberto(PixelDialogKind::ColorRange, (16, 16));
    assert_eq!(e.valor(ids::PIXEL_EDIT_TOLERANCE), 0.0, "Aseprite: 0");
    e.escreve(ids::PIXEL_EDIT_TOLERANCE, "17");
    e.carrega(ids::PIXEL_EDIT_OK);
    assert!(!e.aberto_ainda(), "o OK fecha");
    assert_eq!(
        e.pedido(),
        Some(PixelDialogChoice::ColorRange { tolerance: 17 })
    );
    assert_eq!(e.pedido(), None, "uma vez só");
    e.hero
        .store
        .open_pixel_edit_dialog(PixelDialogKind::ColorRange, (16, 16));
    assert_eq!(e.valor(ids::PIXEL_EDIT_TOLERANCE), 17.0, "lembrada");
    e.pinta();
    e.escreve(ids::PIXEL_EDIT_TOLERANCE, "300");
    e.tecla(KEY_ENTER, false);
    assert!(!e.aberto_ainda(), "o Enter fecha");
    assert_eq!(
        e.pedido(),
        Some(PixelDialogChoice::ColorRange { tolerance: 255 })
    );
}

#[test]
fn modify_ok_e_enter_com_n_e_pincel_por_cada_tipo() {
    for kind in MODIFIES {
        let mut e = Ecra::aberto(PixelDialogKind::Modify(kind), (16, 16));
        e.carrega(ids::PIXEL_EDIT_BRUSHES[1].1);
        assert!(e.aberto_ainda(), "escolher não fecha");
        e.escreve(ids::PIXEL_EDIT_MODIFY_N, "3");
        e.carrega(ids::PIXEL_EDIT_OK);
        assert_eq!(
            e.pedido(),
            Some(PixelDialogChoice::Modify {
                kind,
                n: 3,
                brush: PixelModifyBrush::Circle
            })
        );
        // Lembrados; o Enter é o OK e o n tem tecto.
        e.hero
            .store
            .open_pixel_edit_dialog(PixelDialogKind::Modify(kind), (16, 16));
        e.pinta();
        assert_eq!(e.valor(ids::PIXEL_EDIT_MODIFY_N), 3.0);
        e.carrega(ids::PIXEL_EDIT_BRUSHES[0].1);
        e.escreve(ids::PIXEL_EDIT_MODIFY_N, "500");
        e.tecla(KEY_ENTER, false);
        assert_eq!(
            e.pedido(),
            Some(PixelDialogChoice::Modify {
                kind,
                n: 100,
                brush: PixelModifyBrush::Square
            })
        );
    }
}

#[test]
fn canvas_size_bordas_seguem_o_tamanho_e_a_ancora_e_a_borda_muda_o_tamanho() {
    let mut e = Ecra::aberto(PixelDialogKind::CanvasSize, (10, 7));
    assert_eq!(e.valor(ids::PIXEL_EDIT_CANVAS_W), 10.0);
    // Centro, dw = 5: left = trunc(5/2) = 2, right = 3.
    e.escreve(ids::PIXEL_EDIT_CANVAS_W, "15");
    e.carrega(ids::PIXEL_EDIT_CANVAS_H);
    assert_eq!(
        (
            e.valor(ids::PIXEL_EDIT_LEFT),
            e.valor(ids::PIXEL_EDIT_RIGHT)
        ),
        (2.0, 3.0)
    );
    // Centro, dh = 3: top = 3 − trunc(3/2) = 2, bottom = 1.
    e.escreve(ids::PIXEL_EDIT_CANVAS_H, "10");
    e.carrega(ids::PIXEL_EDIT_CANVAS_W);
    assert_eq!(
        (
            e.valor(ids::PIXEL_EDIT_TOP),
            e.valor(ids::PIXEL_EDIT_BOTTOM)
        ),
        (2.0, 1.0)
    );
    // Centro a encolher, dw = −3: left = trunc(−3/2) = −1, right = −2.
    e.escreve(ids::PIXEL_EDIT_CANVAS_W, "7");
    e.carrega(ids::PIXEL_EDIT_CANVAS_H);
    assert_eq!(
        (
            e.valor(ids::PIXEL_EDIT_LEFT),
            e.valor(ids::PIXEL_EDIT_RIGHT)
        ),
        (-1.0, -2.0)
    );
    e.escreve(ids::PIXEL_EDIT_CANVAS_W, "15");
    e.carrega(ids::PIXEL_EDIT_CANVAS_H);
    // Âncora de canto (baixo-direita): tudo cresce para a esquerda e para cima.
    e.carrega(ancora((2, 2)));
    assert_eq!(e.hero.store.pixel_edit_dialog().anchor, (2, 2));
    let bordas = ids::PIXEL_EDIT_BORDERS.map(|id| e.valor(id));
    assert_eq!(bordas, [5.0, 3.0, 0.0, 0.0]);
    // A borda editada muda a largura: 10 + (−2) + 0 = 8.
    e.escreve(ids::PIXEL_EDIT_LEFT, "-2");
    e.carrega(ids::PIXEL_EDIT_TOP);
    assert_eq!(e.valor(ids::PIXEL_EDIT_CANVAS_W), 8.0);
    e.carrega(ids::PIXEL_EDIT_OK);
    assert_eq!(
        e.pedido(),
        Some(PixelDialogChoice::CanvasSize {
            left: -2,
            top: 3,
            right: 0,
            bottom: 0
        })
    );
    // O Enter com a largura a meio de escrever: as bordas refazem-se ANTES do OK.
    let mut e = Ecra::aberto(PixelDialogKind::CanvasSize, (10, 7));
    assert_eq!(
        e.hero.store.pixel_edit_dialog().anchor,
        (1, 1),
        "volta ao centro"
    );
    e.escreve(ids::PIXEL_EDIT_CANVAS_W, "20");
    e.tecla(KEY_ENTER, false);
    assert_eq!(
        e.pedido(),
        Some(PixelDialogChoice::CanvasSize {
            left: 5,
            top: 0,
            right: 5,
            bottom: 0
        })
    );
}

#[test]
fn sprite_size_px_e_percentagem_e_o_lock_ratio() {
    let mut e = Ecra::aberto(PixelDialogKind::SpriteSize, (20, 10));
    assert!(
        e.hero.store.pixel_edit_dialog().lock_ratio,
        "Lock Ratio ligado"
    );
    assert_eq!(e.valor(ids::PIXEL_EDIT_SPRITE_WPCT), 100.0);
    // Lock: a largura em px leva as duas percentagens e a altura.
    e.escreve(ids::PIXEL_EDIT_SPRITE_W, "40");
    e.carrega(ids::PIXEL_EDIT_SPRITE_H);
    assert_eq!(e.valor(ids::PIXEL_EDIT_SPRITE_WPCT), 200.0);
    assert_eq!(e.valor(ids::PIXEL_EDIT_SPRITE_HPCT), 200.0);
    assert_eq!(e.valor(ids::PIXEL_EDIT_SPRITE_H), 20.0);
    // Sem lock: a percentagem da altura só mexe na altura.
    e.carrega(ids::PIXEL_EDIT_LOCK);
    assert!(!e.hero.store.pixel_edit_dialog().lock_ratio);
    e.escreve(ids::PIXEL_EDIT_SPRITE_HPCT, "50");
    e.carrega(ids::PIXEL_EDIT_SPRITE_W);
    assert_eq!(e.valor(ids::PIXEL_EDIT_SPRITE_H), 5.0);
    assert_eq!(e.valor(ids::PIXEL_EDIT_SPRITE_W), 40.0);
    e.carrega(ids::PIXEL_EDIT_METHODS[2].1);
    e.carrega(ids::PIXEL_EDIT_OK);
    assert_eq!(
        e.pedido(),
        Some(PixelDialogChoice::SpriteSize {
            width: 40,
            height: 5,
            method: PixelSizeMethod::RotSprite
        })
    );
    // Lembrados: o lock desligado e o método; ligar o lock faz a altura seguir a largura.
    let mut e2 = Ecra::aberto(PixelDialogKind::SpriteSize, (20, 10));
    e2.hero.store.set_pixel_edit_lock_ratio(false);
    e2.escreve(ids::PIXEL_EDIT_SPRITE_WPCT, "50");
    e2.carrega(ids::PIXEL_EDIT_SPRITE_H);
    assert_eq!(e2.valor(ids::PIXEL_EDIT_SPRITE_W), 10.0);
    assert_eq!(e2.valor(ids::PIXEL_EDIT_SPRITE_H), 10.0);
    e2.carrega(ids::PIXEL_EDIT_LOCK);
    assert_eq!(e2.valor(ids::PIXEL_EDIT_SPRITE_H), 5.0, "ao ligar, segue");
    e2.tecla(KEY_ENTER, false);
    assert_eq!(
        e2.pedido(),
        Some(PixelDialogChoice::SpriteSize {
            width: 10,
            height: 5,
            method: PixelSizeMethod::Nearest
        })
    );
    e.hero
        .store
        .open_pixel_edit_dialog(PixelDialogKind::SpriteSize, (20, 10));
    let d = e.hero.store.pixel_edit_dialog();
    assert_eq!(
        (d.lock_ratio, d.method),
        (false, PixelSizeMethod::RotSprite)
    );
}

#[test]
fn cancel_e_esc_nao_armam_nada() {
    for kind in kinds() {
        let mut e = Ecra::aberto(kind, (8, 8));
        e.carrega(ids::PIXEL_EDIT_CANCEL);
        assert!(!e.aberto_ainda(), "{kind:?}: o Cancel fecha");
        assert_eq!(e.pedido(), None);
        let mut e = Ecra::aberto(kind, (8, 8));
        e.tecla(KEY_ESCAPE, false);
        assert!(!e.aberto_ainda(), "{kind:?}: o Esc fecha");
        assert_eq!(e.pedido(), None);
    }
}

/// Os rótulos de cada diálogo (as chaves que o pintor mede).
fn rotulos(kind: PixelDialogKind) -> Vec<&'static str> {
    let mut v = vec!["pixel.new_sprite.ok", "pixel.new_sprite.cancel"];
    match kind {
        PixelDialogKind::ColorRange => v.extend([
            "pixel.edit_dialog.color_range.title",
            "pixel.edit_dialog.color_range.note",
            "pixel.edit_dialog.tolerance",
        ]),
        PixelDialogKind::Modify(m) => {
            v.extend(match m {
                PixelModify::Expand => [
                    "pixel.edit_dialog.modify.expand",
                    "pixel.edit_dialog.modify.expand_by",
                ],
                PixelModify::Contract => [
                    "pixel.edit_dialog.modify.contract",
                    "pixel.edit_dialog.modify.contract_by",
                ],
                PixelModify::Border => [
                    "pixel.edit_dialog.modify.border",
                    "pixel.edit_dialog.modify.border_width",
                ],
            });
            v.push("pixel.edit_dialog.brush");
            v.extend(ids::PIXEL_EDIT_BRUSHES.iter().map(|(_, _, k)| *k));
        }
        PixelDialogKind::CanvasSize => {
            v.extend([
                "pixel.edit_dialog.canvas.title",
                "pixel.edit_dialog.width",
                "pixel.edit_dialog.height",
                "pixel.edit_dialog.left",
                "pixel.edit_dialog.top",
                "pixel.edit_dialog.right",
                "pixel.edit_dialog.bottom",
                "pixel.edit_dialog.anchor",
            ]);
            v.extend(ids::PIXEL_EDIT_ANCHORS.iter().map(|(_, _, k)| *k));
        }
        PixelDialogKind::SpriteSize => {
            v.extend([
                "pixel.edit_dialog.sprite.title",
                "pixel.edit_dialog.lock_ratio",
                "pixel.edit_dialog.width",
                "pixel.edit_dialog.height",
                "pixel.edit_dialog.width_scale",
                "pixel.edit_dialog.height_scale",
                "pixel.edit_dialog.interpolation",
            ]);
            v.extend(ids::PIXEL_EDIT_METHODS.iter().map(|(_, _, k)| *k));
        }
    }
    v
}

/// ⭐ **Nenhum rótulo se corta** na largura da caixa — o censo das elisões do pintor; e o controlo
/// positivo: todo rótulo esperado tem de ter sido medido (e traduzido: a chave crua não conta).
#[test]
fn nenhum_rotulo_de_nenhum_dialogo_se_corta() {
    for kind in kinds() {
        let esperados: Vec<&str> = rotulos(kind).into_iter().map(ph2d_i18n::tr).collect();
        for (k, t) in rotulos(kind).iter().zip(&esperados) {
            assert_ne!(k, t, "{kind:?}: «{k}» sem tradução");
        }
        let mut e = Ecra::aberto(kind, (32, 24));
        let ((), medidos) = elisao::medindo(|| e.pinta());
        let mut vistos = Vec::new();
        for md in medidos
            .iter()
            .filter(|md| esperados.contains(&md.texto.as_str()))
        {
            assert!(
                md.coube(),
                "{kind:?}: «{}» cortado em «{}» ({:.1} px)",
                md.texto,
                md.pintado,
                md.largura
            );
            vistos.push(md.texto.clone());
        }
        for t in esperados {
            assert!(
                vistos.iter().any(|v| v == t),
                "{kind:?}: «{t}» nunca foi medido"
            );
        }
    }
}
