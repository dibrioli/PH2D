//! **P6 pelo caminho real**: a cena 9, as ferramentas de selecção com o ponteiro que a shell
//! entrega (e os modificadores), as teclas do Aseprite, a moldura solta, a área de transferência, o
//! menu Sprite e os diálogos pelo [`panel::frame`]. A pergunta de cada teste é a do smoke do dono:
//! *o que muda — e é UM passo que o Ctrl+Z desfaz?*

use ph2d_app_pixel::compose::Mods;
use ph2d_app_pixel::input::{self, Button, KeyAnswer, Press};
use ph2d_app_pixel::smoke_selecao as cena;
use ph2d_app_pixel::{PixelApp, Tool, mode, panel};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::ToastQueue;
use ph2d_editor_core::interaction::ContextMenuKind;
use ph2d_editor_core::object_mode::ObjectMode;
use ph2d_editor_core::pixel_panel::{
    PixelDialogChoice, PixelDialogKind, PixelEditAction as E, PixelOptions, PixelPanelInfo,
    PixelRequest, PixelResample, PixelSelectAction as S, PixelSelectMode, PixelSizeMethod,
    PixelSpriteAction as Sp, PixelTransformField as F, pixel_anchor_borders,
};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_pixel_core::affine::Resample;
use ph2d_pixel_core::canvas::{Anchor, anchor_borders};
use ph2d_pixel_core::selection::{self, Mask, SelectMode, Wand};
use ph2d_pixel_core::transform::{self, Image};
use ph2d_pixel_core::{Rect, Sprite};
use ph2d_render::Camera2d;
use winit::keyboard::KeyCode;

use crate::edit::{PPM, WINDOW, at, canvas, spawn_root};

const SHIFT: Mods = Mods {
    alt: false,
    shift: true,
    ctrl: false,
};
const ALT_SHIFT: Mods = Mods {
    alt: true,
    shift: true,
    ctrl: false,
};
const CTRL_SHIFT: Mods = Mods {
    alt: false,
    shift: true,
    ctrl: true,
};
const CTRL: Mods = Mods {
    alt: false,
    shift: false,
    ctrl: true,
};

struct Palco {
    app: PixelApp,
    sim: SimWorld,
    cam: Camera2d,
    hero: HeroScreen,
    toasts: ToastQueue,
    bits: u64,
    key: u32,
}

impl Palco {
    fn novo() -> Self {
        let mut app = PixelApp::default();
        let mut sim = SimWorld::new();
        assert!(
            ph2d_app_pixel::smoke::stage(&mut app, &mut sim, 9, spawn_root),
            "a cena 9 monta-se"
        );
        let (bits, key) = mode::sprites(sim.world())[0];
        let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
        hero.gizmo.mode.enter(bits, ObjectMode::Draw);
        let mut cam = Camera2d::default();
        app.frame(&mut sim, &mut cam, WINDOW, PPM, Some(bits), spawn_root);
        let mut p = Self {
            app,
            sim,
            cam,
            hero,
            toasts: ToastQueue::new(),
            bits,
            key,
        };
        p.quadro();
        p
    }

    /// A cena 9 sem selecção nem história (um palco limpo para um gesto).
    fn limpo() -> Self {
        let mut p = Self::novo();
        p.pede(PixelRequest::Select(S::Deselect));
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

    fn tecla_com(&mut self, k: KeyCode, ctrl: bool, shift: bool) -> Option<KeyAnswer> {
        let a = input::key(
            &mut self.app,
            self.sim.world(),
            Some(self.bits),
            k,
            ctrl,
            shift,
        );
        self.quadro();
        a
    }

    fn tecla(&mut self, k: KeyCode, ctrl: bool, shift: bool) {
        assert!(
            self.tecla_com(k, ctrl, shift).is_some(),
            "{k:?} é da família"
        );
    }

    fn sprite(&self) -> &Sprite {
        &self.app.doc(self.key).expect("doc").sprite
    }

    fn passos(&self) -> usize {
        self.app.doc(self.key).expect("doc").history.undo_len()
    }

    fn rgba(&self, x: i32, y: i32) -> [u8; 4] {
        let d = self.app.doc(self.key).expect("doc");
        d.sprite
            .rgba_of_value(d.sprite.value_at((d.layer, d.frame), x, y))
    }

    fn mascara(&self) -> Option<Mask> {
        self.sprite().mask().cloned()
    }

    fn press(&mut self, c: (i32, i32), m: Mods) {
        let p = at(&self.app, &self.sim, &self.cam, self.bits, c.0, c.1);
        let r = input::press(
            &mut self.app,
            &canvas(&self.sim, &self.cam, self.bits),
            p,
            Button::Primary,
            m,
        );
        assert_eq!(r, Press::Done);
    }

    fn move_to(&mut self, c: (i32, i32), m: Mods) {
        let p = at(&self.app, &self.sim, &self.cam, self.bits, c.0, c.1);
        assert!(input::moved(
            &mut self.app,
            &canvas(&self.sim, &self.cam, self.bits),
            p,
            m
        ));
    }

    fn release(&mut self) {
        // (um premir que acabou no próprio premir — a varinha, o clique que assenta — larga em vazio)
        let _ = input::release(&mut self.app);
        self.quadro();
    }

    fn arrasta(&mut self, a: (i32, i32), b: (i32, i32), m: Mods) {
        self.press(a, m);
        self.move_to(b, m);
        self.release();
    }

    fn ferramenta(&mut self, t: Tool) {
        self.pede(PixelRequest::Tool(t));
    }

    fn opcoes(&mut self, f: impl FnOnce(&mut PixelOptions)) {
        let mut o = self.app.options;
        f(&mut o);
        self.pede(PixelRequest::Options(o));
    }

    /// UM passo, e o Ctrl+Z devolve o documento de antes (células E selecção).
    fn um_passo_que_desfaz(&mut self, o_que: &str, f: impl FnOnce(&mut Self)) {
        let antes = self.sprite().clone();
        let n = self.passos();
        f(self);
        assert_eq!(self.passos(), n + 1, "{o_que}: UM passo");
        assert_ne!(*self.sprite(), antes, "{o_que}: mudou alguma coisa");
        self.tecla(KeyCode::KeyZ, true, false);
        assert_eq!(*self.sprite(), antes, "{o_que}: o Ctrl+Z devolve tudo");
        self.tecla(KeyCode::KeyZ, true, true);
        self.tecla(KeyCode::KeyZ, true, false);
    }
}

fn floating_image(at: (i32, i32), img: &Image, s: &Sprite) -> bool {
    (0..img.h).all(|y| {
        (0..img.w).all(|x| {
            let v = img.get(x, y);
            let (cx, cy) = (at.0 + x as i32, at.1 + y as i32);
            v == s.transparent_value()
                || s.value_at((s.layers()[0].id, s.frames()[0].id), cx, cy) == v
        })
    })
}

#[test]
fn scene_9_shows_each_thing_of_the_wave() {
    let p = Palco::novo();
    let s = p.sprite().clone();
    let key = (s.layers()[0].id, s.frames()[0].id);
    assert_eq!((s.width(), s.height()), cena::SIZE);
    assert_eq!(p.app.tool, Tool::RectMarquee, "abre com a marquee na mão");
    // a selecção activa são as quatro peças, e cada uma tem a sua borda (formigas)
    let m = s.mask().expect("abre com selecção activa").clone();
    assert_eq!(m, cena::mask(&s, key));
    for (x, y) in [(5, 5), (34, 11), (56, 12), (78, 12)] {
        assert!(m.get(x, y), "a peça em ({x}, {y}) está seleccionada");
    }
    let edges = ph2d_pixel_render::mask_edges(&m);
    let toca = |r: Rect| {
        edges
            .iter()
            .any(|&((x0, y0), _)| r.contains(x0, y0) || r.contains(x0 - 1, y0 - 1))
    };
    for r in [
        cena::RECT,
        Rect::new(27, 4, 16, 16),
        Rect::new(46, 2, 21, 19),
        Rect::new(70, 4, 17, 17),
    ] {
        assert!(toca(r), "a peça {r:?} tem formigas");
    }
    // a varinha apanhou EXACTAMENTE o losango
    let v = selection::wand(&s, key, cena::WAND.0, Wand::default());
    for y in 0..64 {
        for x in 0..128 {
            assert_eq!(
                v.get(x, y),
                cena::in_wand_blob(x, y),
                "varinha em ({x}, {y})"
            );
        }
    }
    // o traço atravessou de 0 a STROKE_X1 e só pintou dentro do retângulo
    for x in 0..=cena::STROKE_X1 {
        let dentro = cena::RECT.contains(x, cena::STROKE_Y);
        assert_eq!(
            p.rgba(x, cena::STROKE_Y) == cena::STROKE,
            dentro,
            "traço em x={x}"
        );
    }
    // as cópias são as leis
    let fast = cena::rotated(cena::FAST_AT, Resample::Fast).expect("fast");
    let rots = cena::rotated(cena::ROTSPRITE_AT, Resample::RotSprite).expect("rotsprite");
    assert!(
        floating_image((fast.x, fast.y), &fast.image, &s),
        "a Fast está assentada"
    );
    assert!(
        floating_image((rots.x, rots.y), &rots.image, &s),
        "a RotSprite está assentada"
    );
    assert_ne!(
        fast.image, rots.image,
        "Fast e RotSprite DIFEREM lado a lado"
    );
    assert!(floating_image(cena::ORIGINAL_AT, &cena::figure(), &s));
    assert!(
        floating_image(cena::FLIP_AT, &transform::flip_h(&cena::figure()), &s),
        "o espelho"
    );
    let tile = transform::offset_wrap(&cena::tile(), cena::SHIFT.0, cena::SHIFT.1);
    assert!(
        floating_image(cena::TILE_AT, &tile, &s),
        "o Shift com volta"
    );
    assert_ne!(tile, cena::tile());
    let near = transform::resize_nearest(&cena::small(), cena::SCALED, cena::SCALED);
    let bil = transform::resize_bilinear(&cena::small(), cena::SCALED, cena::SCALED);
    assert!(floating_image(cena::NEAREST_AT, &near, &s), "Nearest");
    assert!(floating_image(cena::BILINEAR_AT, &bil, &s), "Bilinear");
    assert_ne!(near, bil, "Nearest e Bilinear DIFEREM lado a lado");
}

#[test]
fn the_selection_tools_answer_their_aseprite_keys() {
    let mut p = Palco::limpo();
    for (k, shift, t) in [
        (KeyCode::KeyM, false, Tool::RectMarquee),
        (KeyCode::KeyM, true, Tool::EllipseMarquee),
        (KeyCode::KeyQ, false, Tool::Lasso),
        (KeyCode::KeyQ, true, Tool::PolygonLasso),
        (KeyCode::KeyW, false, Tool::MagicWand),
        (KeyCode::KeyV, false, Tool::Move),
    ] {
        p.tecla(k, false, shift);
        assert_eq!(p.app.tool, t, "{k:?} shift={shift}");
    }
}

#[test]
fn a_marquee_drag_is_one_step_and_a_click_deselects() {
    let mut p = Palco::limpo();
    p.ferramenta(Tool::RectMarquee);
    p.um_passo_que_desfaz("marquee", |p| p.arrasta((10, 30), (3, 22), Mods::default()));
    p.arrasta((10, 30), (3, 22), Mods::default());
    assert_eq!(p.mascara(), Some(Mask::marquee(128, 64, (10, 30), (3, 22))));
    p.um_passo_que_desfaz("clique parado", |p| {
        p.press((50, 50), Mods::default());
        p.release();
    });
    p.press((50, 50), Mods::default());
    p.release();
    assert_eq!(p.mascara(), None, "um clique sem arrastar desselecciona");
    p.ferramenta(Tool::EllipseMarquee);
    p.arrasta((0, 0), (9, 5), Mods::default());
    assert_eq!(p.mascara(), Some(Mask::ellipse(128, 64, (0, 0), (9, 5))));
}

#[test]
fn shift_alt_shift_and_ctrl_shift_add_subtract_and_intersect() {
    let mut p = Palco::limpo();
    p.ferramenta(Tool::RectMarquee);
    let (w, h) = (128, 64);
    let a = Mask::marquee(w, h, (0, 0), (9, 9));
    let b = Mask::marquee(w, h, (5, 5), (14, 14));
    p.arrasta((0, 0), (9, 9), Mods::default());
    p.arrasta((5, 5), (14, 14), SHIFT);
    assert_eq!(p.mascara(), Some(a.combine(&b, SelectMode::Add)));
    p.arrasta((0, 0), (9, 9), Mods::default());
    p.arrasta((5, 5), (14, 14), ALT_SHIFT);
    assert_eq!(p.mascara(), Some(a.combine(&b, SelectMode::Subtract)));
    p.arrasta((0, 0), (9, 9), Mods::default());
    p.arrasta((5, 5), (14, 14), CTRL_SHIFT);
    assert_eq!(p.mascara(), Some(a.combine(&b, SelectMode::Intersect)));
    // o botão da barra é o caminho inteiro sem modificadores
    p.opcoes(|o| o.select_mode = PixelSelectMode::Add);
    p.arrasta((0, 0), (9, 9), Mods::default());
    p.opcoes(|o| o.select_mode = PixelSelectMode::Subtract);
    p.arrasta((5, 5), (14, 14), Mods::default());
    assert_eq!(p.mascara(), Some(a.combine(&b, SelectMode::Subtract)));
}

#[test]
fn the_lassos_and_the_wand_select_by_their_laws() {
    let mut p = Palco::limpo();
    let (w, h) = (128, 64);
    // laço livre: as amostras do arrasto
    p.ferramenta(Tool::Lasso);
    p.press((2, 30), Mods::default());
    for c in [(20, 30), (20, 50), (2, 50)] {
        p.move_to(c, Mods::default());
    }
    p.release();
    let m = p.mascara().expect("laço");
    assert!(m.get(10, 40) && !m.get(30, 40));
    // polígono: cada clique um vértice, clicar o último fecha — UM passo no fim
    p.ferramenta(Tool::PolygonLasso);
    let n = p.passos();
    for c in [(30, 30), (50, 30), (40, 50)] {
        p.press(c, Mods::default());
        p.release();
    }
    assert_eq!(p.passos(), n, "o polígono aberto não é passo");
    p.press((40, 50), Mods::default());
    p.release();
    assert_eq!(p.passos(), n + 1);
    assert_eq!(
        p.mascara(),
        Some(Mask::polygon(w, h, &[(30, 30), (50, 30), (40, 50)]))
    );
    // o Enter fecha também
    for c in [(60, 30), (80, 30), (70, 50)] {
        p.press(c, Mods::default());
        p.release();
    }
    p.tecla(KeyCode::Enter, false, false);
    assert_eq!(
        p.mascara(),
        Some(Mask::polygon(w, h, &[(60, 30), (80, 30), (70, 50)]))
    );
    // a varinha: um clique, com as opções do painel
    p.ferramenta(Tool::MagicWand);
    p.press(cena::WAND.0, Mods::default());
    p.release();
    let s = p.sprite().clone();
    let key = (s.layers()[0].id, s.frames()[0].id);
    assert_eq!(
        p.mascara(),
        Some(selection::wand(&s, key, cena::WAND.0, Wand::default()))
    );
    p.opcoes(|o| {
        o.wand_contiguous = false;
        o.wand_tolerance = 255;
    });
    p.press((0, 63), Mods::default());
    p.release();
    let opt = Wand {
        tolerance: 255,
        contiguous: false,
        ..Wand::default()
    };
    assert_eq!(p.mascara(), Some(selection::wand(&s, key, (0, 63), opt)));
}

#[test]
fn each_select_and_edit_shortcut_is_one_step_that_undo_restores() {
    let mut p = Palco::novo();
    for (k, ctrl, shift, o_que) in [
        (KeyCode::KeyD, true, false, "Ctrl+D"),
        (KeyCode::KeyI, true, true, "Ctrl+Shift+I"),
        (KeyCode::KeyH, false, true, "Shift+H"),
        (KeyCode::KeyV, false, true, "Shift+V"),
        (KeyCode::Delete, false, false, "Delete"),
        (KeyCode::KeyX, true, false, "Ctrl+X"),
        (KeyCode::Escape, false, false, "Esc"),
    ] {
        p.um_passo_que_desfaz(o_que, |p| p.tecla(k, ctrl, shift));
    }
    p.tecla(KeyCode::KeyD, true, false);
    p.um_passo_que_desfaz("Ctrl+Shift+D", |p| p.tecla(KeyCode::KeyD, true, true));
    p.um_passo_que_desfaz("Ctrl+A", |p| p.tecla(KeyCode::KeyA, true, false));
    // o menu pelos botões: cada pedido um passo
    p.tecla(KeyCode::KeyA, true, false);
    for r in [
        E::Rotate180,
        E::Rotate90Cw,
        E::Rotate90Ccw,
        E::ShiftLeft,
        E::ShiftDown,
    ] {
        p.um_passo_que_desfaz(&format!("{r:?}"), |p| {
            p.pede(PixelRequest::Edit(r));
        });
    }
    // flip 2× = identidade
    let antes = p.sprite().clone();
    p.tecla(KeyCode::KeyH, false, true);
    p.tecla(KeyCode::KeyH, false, true);
    assert_eq!(p.sprite().layers(), antes.layers());
    let k = (antes.layers()[0].id, antes.frames()[0].id);
    assert_eq!(
        p.sprite().read_rect(k, antes.bounds()),
        antes.read_rect(k, antes.bounds())
    );
}

#[test]
fn painting_outside_the_selection_changes_nothing() {
    let mut p = Palco::limpo();
    p.ferramenta(Tool::RectMarquee);
    p.arrasta((40, 30), (50, 40), Mods::default());
    p.ferramenta(Tool::Pencil);
    let n = p.passos();
    p.arrasta((60, 50), (70, 55), Mods::default());
    assert_eq!(p.passos(), n, "fora da selecção o traço não é passo");
    p.arrasta((30, 35), (60, 35), Mods::default());
    assert_eq!(p.passos(), n + 1);
    assert_eq!(p.rgba(45, 35), [255, 255, 255, 255]);
    assert_eq!(p.rgba(35, 35), [0; 4], "fora não pinta");
}

#[test]
fn the_floating_transform_commits_as_one_step_and_esc_or_ctrl_z_drop_it() {
    let mut p = Palco::limpo();
    let r = Rect::new(
        cena::ORIGINAL_AT.0,
        cena::ORIGINAL_AT.1,
        cena::FIGURE,
        cena::FIGURE,
    );
    let m = Mask::from_rect(128, 64, r);
    p.ferramenta(Tool::RectMarquee);
    p.arrasta((r.x, r.y), (r.x + 13, r.y + 13), Mods::default());
    let base = p.sprite().clone();
    let n = p.passos();
    // Ctrl+T levanta: os campos aparecem, o documento NÃO muda
    p.tecla(KeyCode::KeyT, true, false);
    let info = p.quadro();
    let t = info.transform.expect("os campos só com células levantadas");
    assert_eq!((t.x, t.y, t.w, t.h, t.angle), (r.x, r.y, 14, 14, 0));
    // o ângulo 90 (anti-horário) em RotSprite é o giro exacto de 90
    p.opcoes(|o| o.resample = PixelResample::RotSprite);
    p.pede(PixelRequest::TransformField {
        field: F::Angle,
        value: 90,
    });
    assert_eq!(*p.sprite(), base, "a moldura solta não toca o documento");
    assert_eq!(p.passos(), n);
    p.tecla(KeyCode::Enter, false, false);
    assert_eq!(p.passos(), n + 1, "o Enter assenta UM passo");
    let k = (base.layers()[0].id, base.frames()[0].id);
    let fig = transform::lift(&base, k, Some(&m)).expect("figura");
    let quer = transform::rot90ccw(&fig.image);
    let (ox, oy) = ph2d_pixel_core::edit::rot90_origin(r);
    assert!(
        floating_image((ox, oy), &quer, p.sprite()),
        "RotSprite@90 = rot90"
    );
    // Esc larga sem passo; Ctrl+Z com a moldura larga sem desfazer o passo de antes
    p.tecla(KeyCode::KeyT, true, false);
    p.pede(PixelRequest::TransformField {
        field: F::X,
        value: 0,
    });
    p.tecla(KeyCode::Escape, false, false);
    assert!(!p.app.floating());
    assert_eq!(p.passos(), n + 1);
    p.tecla(KeyCode::KeyT, true, false);
    p.tecla(KeyCode::KeyZ, true, false);
    assert!(!p.app.floating());
    assert_eq!(p.passos(), n + 1, "o Ctrl+Z largou a moldura, não desfez");
}

#[test]
fn the_move_tool_and_ctrl_drag_lift_the_selection_and_enter_drops_it() {
    let mut p = Palco::limpo();
    let r = Rect::new(
        cena::ORIGINAL_AT.0,
        cena::ORIGINAL_AT.1,
        cena::FIGURE,
        cena::FIGURE,
    );
    p.ferramenta(Tool::RectMarquee);
    p.arrasta((r.x, r.y), (r.x + 13, r.y + 13), Mods::default());
    let base = p.sprite().clone();
    let n = p.passos();
    // Ctrl com a marquee = Move: arrasta as células 3 para a direita
    p.arrasta((r.x + 5, r.y + 5), (r.x + 8, r.y + 5), CTRL);
    assert!(p.app.floating(), "largar o arrasto deixa a moldura solta");
    assert_eq!(p.passos(), n);
    p.tecla(KeyCode::Enter, false, false);
    assert_eq!(p.passos(), n + 1);
    let k = (base.layers()[0].id, base.frames()[0].id);
    let fig = transform::lift(&base, k, Some(&Mask::from_rect(128, 64, r))).expect("figura");
    assert!(
        floating_image((r.x + 3, r.y), &fig.image, p.sprite()),
        "moveu 3 células"
    );
    assert_eq!(
        p.mascara(),
        Some(fig.mask(128, 64).offset(3, 0)),
        "a selecção vai com elas"
    );
    // a ferramenta Move faz o mesmo; um clique fora da moldura assenta
    p.ferramenta(Tool::Move);
    p.arrasta((r.x + 5, r.y + 5), (r.x + 5, r.y + 7), Mods::default());
    p.press((120, 5), Mods::default());
    p.release();
    assert!(!p.app.floating());
    assert_eq!(p.passos(), n + 2, "o clique fora assentou UM passo");
}

#[test]
fn copy_cut_and_paste_float_at_the_copied_place() {
    let mut p = Palco::limpo();
    let info = p.quadro();
    assert!(!info.has_clip);
    p.ferramenta(Tool::RectMarquee);
    let r = cena::RECT;
    p.arrasta(
        (r.x, r.y),
        (r.x + r.w as i32 - 1, r.y + r.h as i32 - 1),
        Mods::default(),
    );
    let n = p.passos();
    p.tecla(KeyCode::KeyC, true, false);
    assert_eq!(p.passos(), n, "o Copy não é passo");
    assert!(p.quadro().has_clip);
    p.tecla(KeyCode::KeyV, true, false);
    assert!(p.app.floating(), "o Paste fica solto");
    let t = p.quadro().transform.expect("solto");
    assert_eq!((t.x, t.y), (r.x, r.y), "na posição copiada");
    p.pede(PixelRequest::TransformField {
        field: F::Y,
        value: 40,
    });
    p.tecla(KeyCode::Enter, false, false);
    assert_eq!(p.passos(), n + 1);
    assert_eq!(
        p.rgba(r.x + 2, 40 + cena::STROKE_Y - r.y),
        cena::STROKE,
        "colou em y=40"
    );
    // Cut = copiar + limpar, UM passo
    p.arrasta(
        (r.x, r.y),
        (r.x + r.w as i32 - 1, r.y + r.h as i32 - 1),
        Mods::default(),
    );
    p.um_passo_que_desfaz("Cut", |p| p.tecla(KeyCode::KeyX, true, false));
    // Copy Merged sem selecção recusa e diz porquê
    p.tecla(KeyCode::KeyD, true, false);
    p.tecla(KeyCode::KeyC, true, true);
    assert!(!p.toasts.is_empty(), "sem selecção o Copy Merged avisa");
}

#[test]
fn the_sprite_menu_and_its_dialogs_reach_the_document() {
    let mut p = Palco::novo();
    p.um_passo_que_desfaz("Rotate Canvas 90", |p| {
        p.pede(PixelRequest::Sprite(Sp::RotateCanvas90Cw));
    });
    p.pede(PixelRequest::Sprite(Sp::RotateCanvas90Cw));
    assert_eq!((p.sprite().width(), p.sprite().height()), (64, 128));
    for a in [
        Sp::RotateCanvas90Ccw,
        Sp::FlipCanvasH,
        Sp::FlipCanvasV,
        Sp::RotateCanvas180,
        Sp::Trim,
    ] {
        p.um_passo_que_desfaz(&format!("{a:?}"), |p| {
            p.pede(PixelRequest::Sprite(a));
        });
    }
    p.pede(PixelRequest::Sprite(Sp::RotateCanvas90Ccw));
    p.pede(PixelRequest::Select(S::All));
    p.ferramenta(Tool::RectMarquee);
    p.arrasta((10, 10), (29, 19), Mods::default());
    p.pede(PixelRequest::Sprite(Sp::Crop));
    assert_eq!(
        (p.sprite().width(), p.sprite().height()),
        (20, 10),
        "Crop à selecção"
    );
    p.tecla(KeyCode::KeyZ, true, false);
    // o C abre o Canvas Size; o OK é UM passo
    p.tecla(KeyCode::KeyC, false, false);
    let kind = p.hero.store.context_menu().map(|r| r.kind);
    assert_eq!(kind, Some(ContextMenuKind::PixelEditDialog));
    assert_eq!(
        p.hero.store.pixel_edit_dialog().kind,
        PixelDialogKind::CanvasSize
    );
    p.hero.store.close_context_menu();
    p.um_passo_que_desfaz("Canvas Size", |p| {
        p.app.aplica_dialogo(PixelDialogChoice::CanvasSize {
            left: 2,
            top: 1,
            right: 3,
            bottom: 0,
        });
    });
    p.app.aplica_dialogo(PixelDialogChoice::CanvasSize {
        left: 2,
        top: 1,
        right: 3,
        bottom: 0,
    });
    assert_eq!((p.sprite().width(), p.sprite().height()), (133, 65));
    p.app.aplica_dialogo(PixelDialogChoice::SpriteSize {
        width: 266,
        height: 130,
        method: PixelSizeMethod::Nearest,
    });
    assert_eq!((p.sprite().width(), p.sprite().height()), (266, 130));
    // Color Range e Modify pelo OK
    p.pede(PixelRequest::Select(S::ColorRange));
    assert_eq!(
        p.hero.store.pixel_edit_dialog().kind,
        PixelDialogKind::ColorRange
    );
    p.hero.store.close_context_menu();
    p.app.fg = cena::WAND_COLOR;
    p.app
        .aplica_dialogo(PixelDialogChoice::ColorRange { tolerance: 0 });
    let m = p.mascara().expect("color range");
    assert_eq!(
        m.count(),
        (0..266 * 130)
            .filter(|i| {
                let (x, y) = ((i % 266) as i32, (i / 266) as i32);
                p.rgba(x, y) == cena::WAND_COLOR
            })
            .count()
    );
    let antes = m.count();
    p.app.aplica_dialogo(PixelDialogChoice::Modify {
        kind: ph2d_editor_core::pixel_panel::PixelModify::Expand,
        n: 1,
        brush: ph2d_editor_core::pixel_panel::PixelModifyBrush::Square,
    });
    assert!(p.mascara().expect("expand").count() > antes);
}

#[test]
fn replace_color_and_layer_via_copy_respect_the_real_selection() {
    let mut p = Palco::limpo();
    p.ferramenta(Tool::RectMarquee);
    // o losango verde, só a metade esquerda seleccionada
    p.arrasta((70, 4), (77, 20), Mods::default());
    p.app.fg = cena::WAND_COLOR;
    p.app.bg = [10, 20, 30, 255];
    p.pede(PixelRequest::ReplaceColor);
    assert_eq!(p.rgba(74, 12), [10, 20, 30, 255], "dentro troca");
    assert_eq!(p.rgba(82, 12), cena::WAND_COLOR, "fora fica");
    // Layer via Copy com uma elipse: só as células da elipse vão para a camada nova
    p.ferramenta(Tool::EllipseMarquee);
    p.arrasta((92, 4), (103, 15), Mods::default());
    let n = p.passos();
    p.pede(PixelRequest::LayerAction(
        ph2d_editor_core::pixel_panel::PixelLayerAction::ViaCut,
    ));
    assert_eq!(p.passos(), n + 1, "via Cut é UM passo");
    let s = p.sprite();
    let nova = s.layers().last().expect("camada nova").id;
    let f = s.frames()[0].id;
    let e = Mask::ellipse(128, 64, (92, 4), (103, 15));
    assert_ne!(
        s.value_at((nova, f), 97, 9),
        s.transparent_value(),
        "o miolo foi"
    );
    assert_eq!(
        s.value_at((nova, f), 92, 4),
        s.transparent_value(),
        "o canto fora da elipse não"
    );
    assert!(!e.get(92, 4));
    assert_eq!(
        s.value_at((s.layers()[0].id, f), 97, 9),
        s.transparent_value(),
        "o Cut abriu o buraco"
    );
}

/// O diálogo Canvas Size (chrome, sem ver o núcleo) faz a MESMA conta da âncora que o núcleo.
#[test]
fn the_canvas_size_dialog_anchor_law_is_the_cores() {
    for (w, h, nw, nh) in [
        (16, 16, 21, 9),
        (5, 7, 2, 30),
        (128, 64, 128, 65),
        (3, 3, 3, 3),
    ] {
        for col in 0..3u8 {
            for row in 0..3u8 {
                let b = anchor_borders(w, h, nw, nh, Anchor { col, row });
                let quer = [b.left, b.top, b.right, b.bottom];
                assert_eq!(
                    pixel_anchor_borders(w, h, nw, nh, (col, row)),
                    quer,
                    "{w}x{h}→{nw}x{nh} ({col},{row})"
                );
            }
        }
    }
}

impl Palco {
    /// O ponto de ecrã do CANTO `(x, y)` das células (não o centro).
    fn canto(&self, x: f64, y: f64) -> (f32, f32) {
        let d = self.app.doc(self.key).expect("doc");
        let m = input::to_screen(self.sim.world(), self.bits, d, &self.cam, WINDOW, PPM).expect("afim");
        let p = m * ph2d_vector::Point::new(x, y);
        (p.x as f32, p.y as f32)
    }

    fn arrasta_ecra(&mut self, a: (f32, f32), b: (f32, f32), m: Mods) {
        let c = canvas(&self.sim, &self.cam, self.bits);
        assert_eq!(input::press(&mut self.app, &c, a, Button::Primary, m), Press::Done);
        assert!(input::moved(&mut self.app, &c, b, m));
        let _ = input::release(&mut self.app);
        self.quadro();
    }
}

#[test]
fn the_handles_scale_rotate_and_the_inside_moves_the_floating_frame() {
    let mut p = Palco::limpo();
    let r = Rect::new(cena::ORIGINAL_AT.0, cena::ORIGINAL_AT.1, cena::FIGURE, cena::FIGURE);
    p.ferramenta(Tool::RectMarquee);
    p.arrasta((r.x, r.y), (r.x + 13, r.y + 13), Mods::default());
    p.tecla(KeyCode::KeyT, true, false);
    let (x1, y1) = (f64::from(r.x + 14), f64::from(r.y + 14));
    // a alça ↘ arrastada 7 células: 21 × 21
    let a = p.canto(x1, y1);
    let b = p.canto(x1 + 7.0, y1 + 7.0);
    p.arrasta_ecra(a, b, Mods::default());
    let t = p.quadro().transform.expect("solta");
    assert_eq!((t.w, t.h, t.angle), (21, 21, 0), "{t:?}");
    // a alça → com Shift guarda a proporção
    let a = p.canto(x1 + 7.0, f64::from(r.y) + 10.5);
    let b = p.canto(x1 + 14.0, f64::from(r.y) + 10.5);
    p.arrasta_ecra(a, b, SHIFT);
    let t = p.quadro().transform.expect("solta");
    assert_eq!(t.w, t.h, "Shift guarda a proporção: {t:?}");
    // fora de um canto gira (Shift: degraus de 15°)
    p.tecla(KeyCode::Escape, false, false);
    p.tecla(KeyCode::KeyT, true, false);
    let c = (f64::from(r.x) + 7.0, f64::from(r.y) + 7.0);
    // 12 px de ecrã para fora do canto ↘: além da alça (6 px), dentro da zona de giro (24 px)
    let k = p.canto(x1, y1);
    let a = (k.0 + 12.0, k.1 + 12.0);
    let b = p.canto(c.0 - 10.0, c.1 + 12.0);
    p.arrasta_ecra(a, b, SHIFT);
    let t = p.quadro().transform.expect("solta");
    assert!(t.angle != 0 && t.angle % 15 == 0, "giro com Shift em degraus de 15°: {t:?}");
    // dentro move, e o Enter assenta UM passo
    p.tecla(KeyCode::Escape, false, false);
    p.tecla(KeyCode::KeyT, true, false);
    let n = p.passos();
    let a = p.canto(f64::from(r.x) + 3.5, f64::from(r.y) + 3.5);
    let b = p.canto(f64::from(r.x) + 13.5, f64::from(r.y) + 3.5);
    p.arrasta_ecra(a, b, Mods::default());
    let t = p.quadro().transform.expect("solta");
    assert_eq!((t.x, t.y), (r.x + 10, r.y));
    p.tecla(KeyCode::Enter, false, false);
    assert_eq!(p.passos(), n + 1);
}
