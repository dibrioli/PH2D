//! ⭐ **A Slice Tool pelo ponteiro real** (P11b): a porta que a shell chama ([`PixelApp::pointer`],
//! [`PixelApp::pointer_moved`], [`PixelApp::keyboard`]), do `Shift+C` ao `Ctrl+Z`.

use ph2d_app_pixel::compose::{Mods, View};
use ph2d_app_pixel::input;
use ph2d_app_pixel::{PixelApp, Tool, panel};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::ToastQueue;
use ph2d_editor_core::object_mode::ObjectMode;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_host::{PointerButton, PointerKind};
use ph2d_pixel_core::slice;
use ph2d_pixel_core::{Op, Rect, SliceId, SliceShape, Sprite};
use ph2d_render::Camera2d;
use ph2d_vector::Point;
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::comum::{PPM, WINDOW, born};

struct Mesa {
    app: PixelApp,
    sim: SimWorld,
    cam: Camera2d,
    toasts: ToastQueue,
    hero: HeroScreen,
    bits: u64,
    key: u32,
}

impl Mesa {
    fn nova() -> Self {
        let (app, sim, cam, bits) = born();
        let key = input::key_of(sim.world(), bits).expect("marcador");
        let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
        hero.gizmo.mode.enter(bits, ObjectMode::Draw);
        Self {
            app,
            sim,
            cam,
            toasts: ToastQueue::default(),
            hero,
            bits,
            key,
        }
    }

    fn ecra(&self, x: f64, y: f64) -> (f32, f32) {
        let m = input::to_screen(
            self.sim.world(),
            self.bits,
            self.app.doc(self.key).expect("doc"),
            &self.cam,
            WINDOW,
            PPM,
        )
        .expect("afim");
        let p = m * Point::new(x, y);
        (p.x as f32, p.y as f32)
    }

    /// Um evento de ponteiro pela porta da shell.
    fn ponteiro(&mut self, kind: PointerKind, c: (f64, f64), mods: Mods) {
        let p = self.ecra(c.0, c.1);
        let mut v = View {
            world: self.sim.world(),
            camera: &mut self.cam,
            toasts: &mut self.toasts,
            window: WINDOW,
            ppm: PPM,
            editing: Some(self.bits),
            typing: false,
            conta_gotas: false,
        };
        let _ = self
            .app
            .pointer(&mut v, kind, PointerButton::Primary, p, mods, true);
    }

    fn arrasta(&mut self, a: (f64, f64), b: (f64, f64), mods: Mods) {
        self.ponteiro(PointerKind::Down, a, mods);
        let q = self.ecra(b.0, b.1);
        let v = View {
            world: self.sim.world(),
            camera: &mut self.cam,
            toasts: &mut self.toasts,
            window: WINDOW,
            ppm: PPM,
            editing: Some(self.bits),
            typing: false,
            conta_gotas: false,
        };
        assert!(self.app.pointer_moved(&v, q, mods), "o arrasto é consumido");
        self.ponteiro(PointerKind::Up, b, mods);
    }

    fn tecla(&mut self, code: KeyCode, m: Mods) -> bool {
        let mut v = View {
            world: self.sim.world(),
            camera: &mut self.cam,
            toasts: &mut self.toasts,
            window: WINDOW,
            ppm: PPM,
            editing: Some(self.bits),
            typing: false,
            conta_gotas: false,
        };
        self.app
            .keyboard(&mut v, PhysicalKey::Code(code), true, m, None)
    }

    /// O quadro da família: os pedidos das teclas aplicam-se.
    fn quadro(&mut self) {
        panel::frame(
            &mut self.app,
            self.sim.world(),
            &mut self.hero,
            &mut self.toasts,
        );
    }

    fn sprite(&self) -> &Sprite {
        &self.app.doc(self.key).expect("doc").sprite
    }

    fn frame(&self) -> u32 {
        let d = self.app.doc(self.key).expect("doc");
        d.sprite.frame_index(d.frame).expect("quadro") as u32
    }

    fn forma(&self, id: SliceId) -> SliceShape {
        self.sprite()
            .slice(id)
            .expect("slice")
            .shape_at(self.frame())
            .expect("key")
    }

    /// Passos que a família faz sem ponteiro: o centro e o pivot ligam-se no cartão (por vir).
    fn liga(&mut self, id: SliceId, center: Rect, pivot: (i32, i32)) {
        let mut k = self.forma(id);
        k.center = Some(center);
        k.pivot = Some(pivot);
        let op: Op = slice::set_shape(self.sprite(), id, 0, k).expect("muda");
        self.app.commit(self.key, op).expect("aplica");
    }
}

const NADA: Mods = Mods {
    alt: false,
    shift: false,
    ctrl: false,
};
const SHIFT: Mods = Mods {
    alt: false,
    shift: true,
    ctrl: false,
};
const CTRL: Mods = Mods {
    alt: false,
    shift: false,
    ctrl: true,
};

/// O centro do píxel.
fn px(x: i32, y: i32) -> (f64, f64) {
    (f64::from(x) + 0.5, f64::from(y) + 0.5)
}

#[test]
fn the_slice_tool_end_to_end_create_select_move_resize_center_pivot_delete_undo() {
    let mut t = Mesa::nova();
    let (w, h) = (t.sprite().width() as i32, t.sprite().height() as i32);
    assert!(w >= 32 && h >= 32, "{w}×{h}");

    assert!(t.tecla(KeyCode::KeyC, SHIFT), "Shift+C");
    assert_eq!(t.app.tool, Tool::Slice);

    // Criar: a slice nasce sem seleção; os pixels não mudam.
    let imagem = t.sprite().clone();
    t.arrasta(px(4, 6), px(24, 16), NADA);
    let id = t.sprite().slices()[0].id;
    assert_eq!(t.sprite().slices()[0].name, "Slice 1");
    assert_eq!(t.forma(id).bounds, Rect::new(4, 6, 20, 10));
    assert!(t.app.slices_selected(t.key).is_empty());
    assert_eq!(t.app.doc(t.key).unwrap().history.undo_len(), 1);
    assert_eq!(
        t.sprite().value_at(
            (
                t.app.doc(t.key).unwrap().layer,
                t.app.doc(t.key).unwrap().frame
            ),
            10,
            10
        ),
        imagem.value_at(
            (
                t.app.doc(t.key).unwrap().layer,
                t.app.doc(t.key).unwrap().frame
            ),
            10,
            10
        ),
        "a ferramenta não pinta"
    );

    // Premir numa não selecionada: seleciona e move (T1).
    t.arrasta(px(10, 10), px(12, 13), NADA);
    assert_eq!(t.forma(id).bounds, Rect::new(6, 9, 20, 10));
    assert_eq!(t.app.slices_selected(t.key), [id]);

    // A borda direita e a de baixo, separadas (não quadrada).
    t.arrasta(px(25, 12), px(21, 12), NADA);
    assert_eq!(t.forma(id).bounds, Rect::new(6, 9, 16, 10));
    t.arrasta(px(12, 18), px(12, 22), NADA);
    assert_eq!(t.forma(id).bounds, Rect::new(6, 9, 16, 14));
    // A esquerda anda a origem.
    t.arrasta(px(6, 12), px(8, 12), NADA);
    assert_eq!(t.forma(id).bounds, Rect::new(8, 9, 14, 14));

    // O centro: a linha da esquerda, e a caixa fica.
    t.liga(id, Rect::new(4, 3, 6, 8), (3, 2));
    t.arrasta(px(12, 16), px(14, 16), NADA);
    assert_eq!(t.forma(id).center, Some(Rect::new(6, 3, 4, 8)));
    assert_eq!(t.forma(id).bounds, Rect::new(8, 9, 14, 14));

    // O pivot sai da caixa; a caixa e o centro não andam.
    let antes = t.forma(id);
    t.arrasta(px(11, 11), px(30, 28), NADA);
    let k = t.forma(id);
    assert_eq!(k.pivot, Some((22, 19)), "fora da caixa");
    assert_eq!(k.bounds, antes.bounds, "a caixa não anda");
    assert_eq!(k.center, antes.center);

    // Redimensionar não mexe no pivot, mesmo fora (G4).
    t.arrasta(px(21, 16), px(19, 16), NADA);
    assert_eq!(t.forma(id).bounds.w, 12);
    assert_eq!(t.forma(id).pivot, Some((22, 19)));

    // Ctrl+Z volta 1 gesto inteiro.
    assert!(t.tecla(KeyCode::KeyZ, CTRL));
    assert_eq!(t.forma(id).bounds.w, 14);

    // Delete apaga a slice inteira; Ctrl+Z traz a key e a seleção não fica com um fantasma.
    let toda = t.sprite().slices()[0].clone();
    assert!(t.tecla(KeyCode::Delete, NADA));
    assert!(t.sprite().slices().is_empty());
    assert!(t.app.slices_selected(t.key).is_empty());
    assert!(t.tecla(KeyCode::KeyZ, CTRL));
    assert_eq!(t.sprite().slices()[0], toda, "as keys voltam");

    // Selecionada e desfeita a criação: a seleção poda-se.
    t.arrasta(px(10, 12), px(10, 12), NADA);
    assert_eq!(t.app.slices_selected(t.key), [id]);
    while t.sprite().slice(id).is_some() {
        assert!(t.tecla(KeyCode::KeyZ, CTRL), "desfaz até à criação");
    }
    assert!(t.app.slices_selected(t.key).is_empty(), "podada");
}

#[test]
fn esc_clears_the_selection_and_arrow_keys_change_frame_and_the_key_is_only_on_that_frame() {
    let mut t = Mesa::nova();
    assert!(t.tecla(KeyCode::KeyC, SHIFT));
    t.arrasta(px(4, 4), px(14, 10), NADA);
    let id = t.sprite().slices()[0].id;

    // Dois quadros: a edição do 2.º não chega ao 1.º.
    let d = t.app.foca(t.key).expect("doc");
    let src = d.frame;
    let (op, _) = ph2d_pixel_core::frames::insert(
        &mut d.sprite,
        1,
        src,
        ph2d_pixel_core::frames::NewFrame::Copy,
        100,
    )
    .expect("quadro");
    t.app.commit(t.key, op).expect("aplica");
    assert_eq!(t.frame(), 0);

    t.arrasta(px(6, 6), px(7, 6), NADA);
    assert_eq!(t.app.slices_selected(t.key), [id]);
    assert_eq!(t.forma(id).bounds.x, 5);
    assert!(t.tecla(KeyCode::ArrowRight, NADA));
    t.quadro();
    assert_eq!(t.frame(), 1, "a seta muda de quadro");
    assert_eq!(
        t.app.slices_selected(t.key),
        [id],
        "sobrevive ao quadro (T5)"
    );

    t.arrasta(px(6, 6), px(16, 6), NADA);
    let c = t.sprite().slice(id).unwrap();
    assert_eq!(c.keys.iter().map(|k| k.frame).collect::<Vec<_>>(), [0, 1]);
    assert_eq!(c.shape_at(0).unwrap().bounds.x, 5);
    assert_eq!(c.shape_at(1).unwrap().bounds.x, 15);

    assert!(t.tecla(KeyCode::Escape, NADA));
    assert!(t.app.slices_selected(t.key).is_empty());
}

#[test]
fn shift_drag_is_a_marquee_and_a_plain_click_on_empty_clears() {
    let mut t = Mesa::nova();
    assert!(t.tecla(KeyCode::KeyC, SHIFT));
    t.arrasta(px(2, 2), px(8, 6), NADA);
    t.arrasta(px(16, 2), px(22, 6), NADA);
    let (a, b) = (t.sprite().slices()[0].id, t.sprite().slices()[1].id);
    t.arrasta(px(1, 1), px(18, 10), SHIFT);
    assert_eq!(t.app.slices_selected(t.key), [a, b]);
    // Com as duas: a borda de `b` leva o mesmo delta a ambas (§10), 1 desfazer.
    let passos = t.app.doc(t.key).unwrap().history.undo_len();
    t.arrasta(px(21, 4), px(19, 4), NADA);
    assert_eq!(t.forma(a).bounds, Rect::new(2, 2, 4, 4));
    assert_eq!(t.forma(b).bounds, Rect::new(16, 2, 4, 4));
    assert_eq!(t.app.doc(t.key).unwrap().history.undo_len(), passos + 1);
    t.arrasta(px(28, 28), px(28, 28), NADA);
    assert!(t.app.slices_selected(t.key).is_empty());
    assert_eq!(t.sprite().slices().len(), 2, "o clique não cria");
}

/// `Esc` a meio do arrasto: nada muda e nenhum passo. Com outra ferramenta o `Delete` não apaga slices.
#[test]
fn esc_mid_drag_gives_no_step_and_delete_with_another_tool_keeps_the_slice() {
    let mut t = Mesa::nova();
    assert!(t.tecla(KeyCode::KeyC, SHIFT));
    t.arrasta(px(4, 4), px(14, 10), NADA);
    let id = t.sprite().slices()[0].id;
    let passos = t.app.doc(t.key).unwrap().history.undo_len();
    t.ponteiro(PointerKind::Down, px(6, 6), NADA);
    assert!(t.app.stroking(), "o gesto bloqueia as teclas de documento");
    assert!(t.tecla(KeyCode::Escape, NADA));
    t.ponteiro(PointerKind::Up, px(9, 9), NADA);
    assert_eq!(t.forma(id).bounds, Rect::new(4, 4, 10, 6));
    assert_eq!(t.app.doc(t.key).unwrap().history.undo_len(), passos);
    assert_eq!(t.app.slices_selected(t.key), [id]);

    assert!(t.tecla(KeyCode::KeyB, NADA));
    assert_eq!(t.app.tool, Tool::Pencil);
    let _ = t.tecla(KeyCode::Delete, NADA);
    assert!(t.sprite().slice(id).is_some());
}

/// O desenho chega à cena pela porta da shell: a slice com qualquer ferramenta, e a selecionada leva o
/// contorno duplo, as 8 alças e a ficha do nome, só com a ferramenta na mão.
#[test]
fn the_slices_reach_the_scene_with_any_tool_and_selected_adds_outline_handles_and_name() {
    let mut t = Mesa::nova();
    let mut text = ph2d_text::TextSystem::without_system_fonts();
    let mut caminhos = |t: &mut Mesa| {
        let mut scene = ph2d_vector::VectorScene::new();
        let ids = [t.bits];
        t.app.draw_of(
            t.sim.world(),
            &t.cam,
            WINDOW,
            &t.hero,
            &ids,
            (&mut scene, &mut text),
        );
        scene.inner_mut().encoding().n_paths
    };
    let vazio = caminhos(&mut t);
    assert!(t.tecla(KeyCode::KeyC, SHIFT));
    t.arrasta(px(4, 4), px(14, 10), NADA);
    let uma = caminhos(&mut t);
    assert_eq!(uma, vazio + 1, "o contorno na cor dela");
    t.arrasta(px(6, 6), px(6, 6), NADA);
    let selecionada = caminhos(&mut t);
    assert!(
        selecionada >= uma + 1 + 16 + 1,
        "contorno duplo, 8 alças com anel, ficha: {uma} -> {selecionada}"
    );
    assert!(t.tecla(KeyCode::KeyB, NADA));
    assert_eq!(caminhos(&mut t), uma, "com o lápis, a slice sem a seleção");
}
