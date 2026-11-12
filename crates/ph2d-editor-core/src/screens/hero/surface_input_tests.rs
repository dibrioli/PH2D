//! O botão primário que a shell anota ([`crate::documents::Documents::note_button`]) chega à aba de
//! família activa em [`crate::doc_surface::PointerInput::primary_held`], pelo carregar e pelo mover.

use crate::doc_surface::{
    Compose, DocTab, DocumentSurface, KeyInput, KeyOutcome, PointerInput, SurfaceTab,
};
use crate::screens::hero::HeroScreen;
use crate::screens::hero::board_view::Input;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_host::{Modifiers, PointerButton, PointerKind};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;
use std::any::Any;

/// Uma família com uma aba, que anota o `primary_held` de cada evento do rato que recebe.
#[derive(Default)]
struct Grava {
    vistos: Vec<bool>,
}

impl DocumentSurface for Grava {
    fn tabs(&self) -> Vec<DocTab> {
        vec![DocTab {
            id: 7,
            title: String::new(),
            dirty: false,
        }]
    }
    fn paint(&mut self, _: u64, _: Rect, _: &mut VectorScene, _: &mut TextSystem, _: &Theme) {}
    fn pointer(
        &mut self,
        _: u64,
        input: PointerInput<'_>,
        _: PointerKind,
        _: PointerButton,
        _: f32,
        _: f32,
    ) -> bool {
        self.vistos.push(input.primary_held);
        true
    }
    fn pointer_move(&mut self, _: u64, input: PointerInput<'_>, _: f32, _: f32) -> bool {
        self.vistos.push(input.primary_held);
        true
    }
    fn wheel(&mut self, _: u64, _: f32, _: f32, _: f32, _: bool) -> bool {
        false
    }
    fn key(&mut self, _: u64, _: &mut TextSystem, _: KeyInput<'_>) -> KeyOutcome {
        KeyOutcome::default()
    }
    fn compose(&mut self, _: u64, _: Compose<'_>) -> bool {
        false
    }
    fn close(&mut self, _: u64) {}
    fn undo(&mut self, _: u64, _: bool) -> bool {
        false
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

fn input(ts: &mut TextSystem) -> Input<'_> {
    Input {
        text: ts,
        mods: Modifiers::default(),
        now_ns: 0,
    }
}

/// ⭐ Premido na janela ⇒ a aba vê `true` no mover e no carregar; largado (mesmo fora dela) ⇒ o
/// mover seguinte vê `false` — é assim que a aba sabe que o largar se perdeu.
#[test]
fn o_botao_anotado_chega_a_aba_no_carregar_e_no_mover() {
    use PointerButton::Primary;
    use PointerKind::{Down, Up};
    let mut hero = HeroScreen::new(NodeId(1));
    let mut ts = TextSystem::without_system_fonts();
    hero.documents.install_surface(Box::<Grava>::default());
    hero.documents
        .activate_surface(SurfaceTab { family: 0, id: 7 });
    assert!(hero.documents.surface_active().is_some());

    hero.documents.note_button(Down, Primary);
    assert_eq!(
        super::pointer(&mut hero, input(&mut ts), Down, Primary, 1.0, 1.0),
        Some(true)
    );
    assert_eq!(
        super::pointer_move(&mut hero, input(&mut ts), 2.0, 2.0),
        Some(true)
    );
    hero.documents.note_button(Up, Primary);
    assert_eq!(
        super::pointer_move(&mut hero, input(&mut ts), 3.0, 3.0),
        Some(true)
    );
    assert_eq!(
        super::pointer(
            &mut hero,
            input(&mut ts),
            Down,
            PointerButton::Secondary,
            4.0,
            4.0
        ),
        Some(true)
    );

    let vistos = &hero
        .documents
        .surfaces()
        .find::<Grava>()
        .expect("a família instalada")
        .vistos;
    assert_eq!(
        vistos,
        &[true, true, false, false],
        "carregar premido, mover premido, mover largado, carregar de outro botão sem o primário"
    );
}

/// Uma aba com um menu a flutuar em `MENU` (por cima do chrome): conta os carregares e as rodas.
#[derive(Default)]
struct Flutua {
    carregares: usize,
    rodas: usize,
}

const MENU: Rect = Rect {
    x: 100.0,
    y: 100.0,
    w: 50.0,
    h: 50.0,
};

impl DocumentSurface for Flutua {
    fn tabs(&self) -> Vec<DocTab> {
        Grava::default().tabs()
    }
    fn paint(&mut self, _: u64, _: Rect, _: &mut VectorScene, _: &mut TextSystem, _: &Theme) {}
    fn pointer(
        &mut self,
        _: u64,
        _: PointerInput<'_>,
        _: PointerKind,
        _: PointerButton,
        _: f32,
        _: f32,
    ) -> bool {
        self.carregares += 1;
        true
    }
    fn pointer_move(&mut self, _: u64, _: PointerInput<'_>, _: f32, _: f32) -> bool {
        false
    }
    fn wheel(&mut self, _: u64, _: f32, _: f32, _: f32, _: bool) -> bool {
        self.rodas += 1;
        true
    }
    fn key(&mut self, _: u64, _: &mut TextSystem, _: KeyInput<'_>) -> KeyOutcome {
        KeyOutcome::default()
    }
    fn compose(&mut self, _: u64, _: Compose<'_>) -> bool {
        false
    }
    fn close(&mut self, _: u64) {}
    fn undo(&mut self, _: u64, _: bool) -> bool {
        false
    }
    fn floats_at(&self, _: u64, x: f32, y: f32) -> bool {
        MENU.contains(x, y)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// ⭐ Um carregar e uma roda por cima do CHROME vão à aba onde um menu dela flutua
/// ([`DocumentSurface::floats_at`]) e ficam no chrome fora dele.
#[test]
fn o_carregar_e_a_roda_sobre_o_chrome_vao_a_aba_so_onde_o_menu_dela_flutua() {
    use PointerButton::Primary;
    use PointerKind::{Down, Up};
    let mut hero = HeroScreen::new(NodeId(1));
    let mut ts = TextSystem::without_system_fonts();
    hero.documents.install_surface(Box::<Flutua>::default());
    hero.documents
        .activate_surface(SurfaceTab { family: 0, id: 7 });
    let board = crate::screens::hero::board_view::pointer;
    let (mx, my) = (MENU.x + 1.0, MENU.y + 1.0);
    let (fx, fy) = (MENU.x - 10.0, MENU.y - 10.0);
    // Sobre o chrome (`on_canvas` = false): no menu, a aba; fora, o chrome (o largar segue a descida).
    assert!(board(
        &mut hero,
        input(&mut ts),
        Down,
        Primary,
        mx,
        my,
        false
    ));
    assert!(board(&mut hero, input(&mut ts), Up, Primary, mx, my, false));
    assert!(!board(
        &mut hero,
        input(&mut ts),
        Down,
        Primary,
        fx,
        fy,
        false
    ));
    assert!(!board(
        &mut hero,
        input(&mut ts),
        Up,
        Primary,
        fx,
        fy,
        false
    ));
    // A roda sobre um painel (o chrome): a shell pergunta `chrome_owns_wheel`.
    assert!(
        !hero.documents.chrome_owns_wheel(true, mx, my),
        "no menu: da aba"
    );
    assert!(
        hero.documents.chrome_owns_wheel(true, fx, fy),
        "fora: do painel"
    );
    assert!(
        !hero.documents.chrome_owns_wheel(false, fx, fy),
        "fora do chrome: da aba"
    );
    assert!(crate::screens::hero::board_view::wheel(
        &mut hero, mx, my, 1.0
    ));
    let f = hero
        .documents
        .surfaces()
        .find::<Flutua>()
        .expect("a família");
    assert_eq!(
        (f.carregares, f.rodas),
        (2, 1),
        "a descida e o largar no menu; a roda"
    );
}
