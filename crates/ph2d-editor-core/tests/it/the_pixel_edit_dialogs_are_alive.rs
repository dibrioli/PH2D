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

pub(crate) struct Ecra {
    pub(crate) hero: HeroScreen,
    pub(crate) text: TextSystem,
    /// A janela onde o hero pinta (`None` = a de referência).
    pub(crate) janela: Option<Rect>,
}

impl Ecra {
    fn aberto(kind: PixelDialogKind, size: (u32, u32)) -> Self {
        let mut e = Self {
            hero: HeroScreen::new(NodeId(1)),
            text: TextSystem::without_system_fonts(),
            janela: None,
        };
        e.hero.store.open_pixel_edit_dialog(kind, size);
        e.pinta();
        e
    }

    pub(crate) fn pinta(&mut self) {
        let mut scene = VectorScene::new();
        let vp = self
            .janela
            .unwrap_or(Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H));
        paint_hero_screen(&mut self.hero, vp, &mut scene, &mut self.text);
    }

    pub(crate) fn centro(&self, id: NodeId) -> Option<(f32, f32)> {
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

    pub(crate) fn carrega(&mut self, id: NodeId) {
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

    pub(crate) fn tecla(&mut self, keycode: u32, ctrl: bool) {
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

    pub(crate) fn aberto_ainda(&self) -> bool {
        matches!(
            self.hero.store.context_menu().map(|r| r.kind),
            Some(ContextMenuKind::PixelEditDialog)
        )
    }

    pub(crate) fn pedido(&mut self) -> Option<PixelDialogChoice> {
        self.hero.store.take_pixel_edit_request()
    }
}

fn kinds() -> Vec<PixelDialogKind> {
    let mut v = vec![PixelDialogKind::ColorRange];
    v.extend(MODIFIES.map(PixelDialogKind::Modify));
    v.extend([PixelDialogKind::CanvasSize, PixelDialogKind::SpriteSize]);
    v.extend(FILTERS);
    v
}

/// P8: os diálogos de filtro.
const FILTERS: [PixelDialogKind; 6] = [
    PixelDialogKind::Invert,
    PixelDialogKind::Despeckle,
    PixelDialogKind::Outline,
    PixelDialogKind::BrightnessContrast,
    PixelDialogKind::HueSaturation,
    PixelDialogKind::ColorCurve,
];

/// Os controlos de um diálogo de filtro: canais e Preview sempre; Tiled fora do Invert.
fn controlos_filtro(kind: PixelDialogKind) -> Vec<NodeId> {
    let mut v: Vec<NodeId> = ids::PIXEL_FILTER_CHANNELS
        .iter()
        .map(|(_, id, _)| *id)
        .collect();
    if matches!(kind, PixelDialogKind::Despeckle | PixelDialogKind::Outline) {
        v.push(ids::PIXEL_FILTER_TILED);
    }
    if kind == PixelDialogKind::BrightnessContrast {
        v.extend([ids::PIXEL_FILTER_BRIGHTNESS, ids::PIXEL_FILTER_CONTRAST]);
    }
    if kind == PixelDialogKind::HueSaturation {
        v.extend(ids::PIXEL_FILTER_HUE_MODES.iter().map(|(_, id, _)| *id));
        v.extend(ids::PIXEL_FILTER_ADJUST_FIELDS[2..].iter().map(|f| f.1));
    }
    if kind == PixelDialogKind::ColorCurve {
        v.push(ids::PIXEL_FILTER_CURVE);
    }
    if kind == PixelDialogKind::Despeckle {
        v.extend([ids::PIXEL_FILTER_MEDIAN_W, ids::PIXEL_FILTER_MEDIAN_H]);
    }
    if kind == PixelDialogKind::Outline {
        v.extend(ids::PIXEL_FILTER_PLACES.iter().map(|(_, id, _)| *id));
        v.extend(ids::PIXEL_FILTER_MATRICES.iter().map(|(_, id, _)| *id));
        v.extend(ids::PIXEL_FILTER_BACKGROUNDS.iter().map(|(_, id, _)| *id));
    }
    v
}

/// Os rótulos de um diálogo de filtro.
fn rotulos_filtro(kind: PixelDialogKind) -> Vec<&'static str> {
    let mut v: Vec<&str> = ids::PIXEL_FILTER_CHANNELS
        .iter()
        .map(|(_, _, k)| *k)
        .collect();
    v.push("pixel.filter_dialog.apply");
    match kind {
        PixelDialogKind::Invert => v.push("pixel.filter_dialog.invert.title"),
        PixelDialogKind::Despeckle => v.extend([
            "pixel.filter_dialog.despeckle.title",
            "pixel.filter_dialog.tiled",
            "pixel.edit_dialog.width",
            "pixel.edit_dialog.height",
        ]),
        PixelDialogKind::BrightnessContrast => v.extend([
            "pixel.filter_dialog.brightness_contrast.title",
            "pixel.filter_dialog.brightness",
            "pixel.filter_dialog.contrast",
        ]),
        PixelDialogKind::HueSaturation => {
            v.extend([
                "pixel.filter_dialog.hue_saturation.title",
                "pixel.filter_dialog.hue",
                "pixel.filter_dialog.saturation",
                "pixel.filter_dialog.lightness",
            ]);
            v.extend(ids::PIXEL_FILTER_HUE_MODES.iter().map(|(_, _, k)| *k));
        }
        PixelDialogKind::ColorCurve => v.push("pixel.filter_dialog.curve.title"),
        _ => {
            v.extend([
                "pixel.filter_dialog.outline.title",
                "pixel.filter_dialog.tiled",
            ]);
            v.extend(ids::PIXEL_FILTER_PLACES.iter().map(|(_, _, k)| *k));
            v.extend(ids::PIXEL_FILTER_MATRICES.iter().map(|(_, _, k)| *k));
            v.extend(ids::PIXEL_FILTER_BACKGROUNDS.iter().map(|(_, _, k)| *k));
        }
    }
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
        k => v.extend(controlos_filtro(k)),
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
    // O «0» vai ao piso de 1 px e a proporção é a do tamanho original: subir devolve-a (04/11).
    e.escreve(ids::PIXEL_EDIT_SPRITE_W, "0");
    e.carrega(ids::PIXEL_EDIT_SPRITE_H);
    assert_eq!(e.valor(ids::PIXEL_EDIT_SPRITE_W), 1.0);
    assert_eq!(e.valor(ids::PIXEL_EDIT_SPRITE_H), 1.0);
    e.escreve(ids::PIXEL_EDIT_SPRITE_W, "40");
    e.carrega(ids::PIXEL_EDIT_SPRITE_H);
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
    let mut v = vec!["pixel.new_sprite.cancel"];
    if !kind.is_filter() {
        v.push("pixel.new_sprite.ok");
    }
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
        k => v.extend(rotulos_filtro(k)),
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
        for md in medidos.iter() {
            // Em cartões a unidade vai no rótulo: «Width (px)» é o «Width» da tabela.
            let nome = md.texto.split(" (").next().unwrap_or(&md.texto);
            if !esperados.contains(&nome) {
                continue;
            }
            assert!(
                md.coube(),
                "{kind:?}: «{}» cortado em «{}» ({:.1} px)",
                md.texto,
                md.pintado,
                md.largura
            );
            vistos.push(nome.to_owned());
        }
        for t in esperados {
            assert!(
                vistos.iter().any(|v| v == t),
                "{kind:?}: «{t}» nunca foi medido"
            );
        }
    }
}

/// **Trial 01/11** (`ph2d-pixel-core/tests/fixtures/oracle_trial_modify.json`): o `n` do Modify
/// tem tecto 100 (acima dá o mesmo que 100), os decimais cortam para BAIXO e um «%» no fim é
/// ignorado (`50%` = 50 px). O `n` esperado sai da geometria de cada caso da fixtura (Expand de
/// uma célula: `(x1 − x0) / 2`; Contract do sprite inteiro: `x0`).
#[test]
fn the_modify_field_caps_at_100_truncates_and_ignores_a_percent_sign_like_the_trial() {
    let casos = [
        ("2.5", 2),
        ("2.6", 2),
        ("3%", 3),
        ("50%", 50),
        ("63", 63),
        ("100", 100),
        ("101", 100),
        ("150", 100),
        ("999", 100),
    ];
    for kind in MODIFIES {
        for (texto, n) in casos {
            let mut e = Ecra::aberto(PixelDialogKind::Modify(kind), (320, 206));
            e.escreve(ids::PIXEL_EDIT_MODIFY_N, texto);
            e.tecla(KEY_ENTER, false);
            let got = match e.pedido() {
                Some(PixelDialogChoice::Modify { n, .. }) => n,
                o => panic!("{texto}: {o:?}"),
            };
            assert_eq!(got, n, "{kind:?} «{texto}»");
        }
    }
}

/// ⭐ P8: o diálogo de filtro pelo clique real — cada escolha muda o pedido que a família
/// pré-visualiza (`pixel_filter_preview`), o Preview desligado larga-o, o OK arma o MESMO pedido e
/// fecha, e a janela da mediana fica lembrada.
#[test]
fn the_filter_dialogs_choices_reach_the_preview_and_the_ok() {
    use ph2d_editor_core::pixel_panel::{PixelOutlineBg, PixelOutlineMatrix, PixelOutlinePlace};
    let mut e = Ecra::aberto(PixelDialogKind::Outline, (16, 16));
    let antes = e
        .hero
        .store
        .pixel_filter_preview()
        .expect("o filtro aberto mostra-se em tempo real");
    e.carrega(ids::PIXEL_FILTER_PLACES[1].1);
    e.carrega(ids::PIXEL_FILTER_MATRICES[1].1);
    e.carrega(ids::PIXEL_FILTER_BACKGROUNDS[1].1);
    e.carrega(ids::PIXEL_FILTER_TILED);
    e.carrega(ids::PIXEL_FILTER_CHANNELS[3].1);
    let quer = PixelDialogChoice::Outline {
        place: PixelOutlinePlace::Inside,
        matrix: PixelOutlineMatrix::Square,
        background: PixelOutlineBg::BackgroundColor,
        tiled: true,
        channels: [true, true, true, false],
        all: false,
    };
    assert_ne!(antes, quer);
    assert_eq!(e.hero.store.pixel_filter_preview(), Some(quer));
    e.carrega(ids::PIXEL_EDIT_OK);
    assert!(!e.aberto_ainda());
    assert_eq!(e.pedido(), Some(quer));
    assert_eq!(
        e.hero.store.pixel_filter_preview(),
        None,
        "fechado não pré-visualiza"
    );

    // Dono, 05/11: em tempo real também a ESCREVER, antes do Enter.
    let mut e = Ecra::aberto(PixelDialogKind::HueSaturation, (16, 16));
    e.escreve(ids::PIXEL_FILTER_HUE, "40");
    assert!(
        matches!(
            e.hero.store.pixel_filter_preview(),
            Some(PixelDialogChoice::HueSaturation { hue: 40, .. })
        ),
        "o efeito acompanha cada tecla"
    );

    let mut e = Ecra::aberto(PixelDialogKind::Despeckle, (16, 16));
    assert_eq!(e.valor(ids::PIXEL_FILTER_MEDIAN_W), 3.0, "Aseprite: 3×3");
    e.escreve(ids::PIXEL_FILTER_MEDIAN_W, "5");
    e.escreve(ids::PIXEL_FILTER_MEDIAN_H, "99");
    e.tecla(KEY_ENTER, false);
    assert!(
        matches!(
            e.pedido(),
            Some(PixelDialogChoice::Despeckle {
                width: 5,
                height: 16,
                ..
            })
        ),
        "a janela presa ao lado do sprite"
    );
    e.hero
        .store
        .open_pixel_edit_dialog(PixelDialogKind::Despeckle, (16, 16));
    assert_eq!(
        e.valor(ids::PIXEL_FILTER_MEDIAN_W),
        5.0,
        "lembrada na sessão"
    );
}

impl Ecra {
    /// Um premir/largar do rato em `(x, y)`, com o quadro da família (`drive_pixel_curve`) a cada
    /// passo, como o `panel::frame` do Pixel faz.
    fn rato(&mut self, (x, y): (f32, f32), kind: PointerKind, s: u64) {
        let arena = Bump::new();
        let ev = PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind,
            source: PointerSource::Mouse,
            button: PointerButton::Primary,
            // Segundos entre gestos: só o par do duplo-clique cai na janela dele.
            timestamp_ns: u128::from(s) * 1_000_000_000 + u128::from(kind == PointerKind::Up),
        };
        let ev = self.hero.handle_pointer(ev, &arena).to_vec();
        for e in ev {
            self.hero.apply_event(e);
        }
        let caixa = self.hero.hit_index.rect_for(ids::PIXEL_FILTER_CURVE);
        self.hero.store.drive_pixel_curve(caixa);
        self.pinta();
    }

    fn curva(&self) -> Vec<(u8, u8)> {
        self.hero
            .store
            .pixel_edit_dialog()
            .filter
            .curve
            .points()
            .to_vec()
    }
}

/// ⭐ P8 Adjustments pelo clique real: B/C e H/S escrevem o pedido preso às faixas do Aseprite; a
/// curva ganha um ponto no premir sobre a caixa, o arrasto move-o sem cruzar, o duplo-clique tira-o.
#[test]
fn the_adjustments_dialogs_reach_the_preview_and_the_curve_is_edited_by_the_mouse() {
    let mut e = Ecra::aberto(PixelDialogKind::BrightnessContrast, (16, 16));
    e.escreve(ids::PIXEL_FILTER_BRIGHTNESS, "-50");
    e.escreve(ids::PIXEL_FILTER_CONTRAST, "300");
    // Carregar noutro campo confirma o escrito (o Enter também).
    e.carrega(ids::PIXEL_FILTER_BRIGHTNESS);
    let quer = PixelDialogChoice::BrightnessContrast {
        brightness: -50,
        contrast: 100,
        channels: [true; 4],
        all: false,
    };
    assert_eq!(
        e.hero.store.pixel_filter_preview(),
        Some(quer),
        "preso a 100"
    );
    e.tecla(KEY_ENTER, false);
    assert_eq!(e.pedido(), Some(quer));

    let mut e = Ecra::aberto(PixelDialogKind::HueSaturation, (16, 16));
    e.carrega(ids::PIXEL_FILTER_HUE_MODES[3].1);
    e.escreve(ids::PIXEL_FILTER_HUE, "-400");
    e.escreve(ids::PIXEL_FILTER_LIGHTNESS, "25");
    e.escreve(ids::PIXEL_FILTER_ALPHA, "-150");
    e.carrega(ids::PIXEL_FILTER_TARGET);
    e.carrega(ids::PIXEL_FILTER_HUE);
    assert_eq!(
        e.hero.store.pixel_filter_preview(),
        Some(PixelDialogChoice::HueSaturation {
            mode: ph2d_editor_core::pixel_panel::PixelHueMode::HsvPlus,
            hue: -180,
            saturation: 0,
            lightness: 25,
            alpha: -100,
            channels: [true; 4],
            all: true,
        })
    );

    let mut e = Ecra::aberto(PixelDialogKind::ColorCurve, (16, 16));
    let b = e
        .hero
        .hit_index
        .rect_for(ids::PIXEL_FILTER_CURVE)
        .expect("a caixa da curva pintada");
    let (x, y) = (b.x + b.w * 0.25, b.y + b.h * 0.25);
    e.rato((x, y), PointerKind::Down, 0);
    e.rato((x, y), PointerKind::Up, 0);
    let pts = e.curva();
    assert_eq!(pts.len(), 1, "o premir numa zona vazia acrescenta");
    assert!(
        pts[0].0.abs_diff(64) <= 1 && pts[0].1.abs_diff(191) <= 1,
        "{pts:?}"
    );
    assert!(matches!(
        e.hero.store.pixel_filter_preview(),
        Some(PixelDialogChoice::ColorCurve { curve, .. }) if curve.points() == pts.as_slice()
    ));
    // O arrasto para lá da ponta direita prende o x antes dela.
    let longe = (b.x + b.w * 2.0, b.y - b.h);
    e.rato((x, y), PointerKind::Down, 10);
    e.rato(longe, PointerKind::Move, 10);
    e.rato(longe, PointerKind::Up, 10);
    assert_eq!(e.curva(), vec![(254, 255)]);
    // Duplo-clique sobre o ponto tira-o; as pontas ficam.
    let [px, py] = ph2d_editor_core::widget::curve_to_screen(b, (254, 255));
    // Um px para dentro da caixa (a ponta de cima é a borda dela).
    let dentro = (px - 1.0, py + 1.0);
    for _ in 0..2 {
        e.rato(dentro, PointerKind::Down, 20);
        e.rato(dentro, PointerKind::Up, 20);
    }
    assert!(e.curva().is_empty(), "o duplo-clique tira o ponto");
    // A capacidade da caixa pintada cabe no tipo que a guarda.
    assert!(
        ph2d_editor_core::widget::curve_capacity(b)
            <= ph2d_editor_core::pixel_panel::PIXEL_CURVE_MAX
    );
}
