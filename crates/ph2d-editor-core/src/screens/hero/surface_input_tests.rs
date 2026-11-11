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
