//! ⭐ **A família da aba «Collision Layers» no centro** (onda 9a, versão B do dono 07/11): a família de UMA aba ao
//! lado de Scene, pelo mesmo mecanismo das abas do Kanban, do Code e do editor de animações. A família só
//! dá o título, o fecho, o pedido de vir à frente e a ÁREA; a grade, a lista e os alvos pinta-os o painel
//! ([`super`]) por cima, para as células continuarem os ids de sempre (undo,
//! Auto Key, arrasto, losango).

use std::any::Any;

use crate::aba;
use ph2d_editor_core::doc_surface::{
    Compose, DocTab, DocumentSurface, KeyInput, KeyOutcome, PointerInput,
};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerKind};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// O id da única aba desta família.
pub const ABA: u64 = 1;

/// A família da aba das camadas.
#[derive(Debug, Default)]
pub struct AbaDasCamadas;

impl DocumentSurface for AbaDasCamadas {
    fn tabs(&self) -> Vec<DocTab> {
        if !aba::aberta() {
            return Vec::new();
        }
        vec![DocTab {
            id: ABA,
            title: ph2d_i18n::tr("card.physics.matrix_title").to_owned(),
            dirty: false,
        }]
    }

    fn paint(
        &mut self,
        _id: u64,
        area: Rect,
        _scene: &mut VectorScene,
        _text: &mut TextSystem,
        _theme: &Theme,
    ) {
        aba::poe_area(area);
    }

    // Os cliques da aba são alvos do painel (o índice de cliques vem antes); o que sobra é do fundo.
    fn pointer(
        &mut self,
        _id: u64,
        _input: PointerInput<'_>,
        kind: PointerKind,
        _button: PointerButton,
        x: f32,
        y: f32,
    ) -> bool {
        // As barras da grade (onda 9f): carregar agarra, largar solta.
        match kind {
            PointerKind::Down => aba::agarra(x, y),
            PointerKind::Up => aba::solta(),
            PointerKind::Move => aba::arrasta(x, y),
        }
    }

    fn pointer_move(&mut self, _id: u64, _input: PointerInput<'_>, x: f32, y: f32) -> bool {
        aba::arrasta(x, y)
    }

    fn wheel(&mut self, _id: u64, x: f32, y: f32, dy: f32, shift: bool) -> bool {
        aba::rola(x, y, dy, shift)
    }

    /// O deslizar de lado do trackpad: as colunas da grelha, como o Shift+roda.
    fn wheel_sideways(&mut self, _id: u64, x: f32, y: f32, dx: f32) -> bool {
        aba::rola(x, y, dx, true)
    }

    fn key(&mut self, _id: u64, _ts: &mut TextSystem, _input: KeyInput<'_>) -> KeyOutcome {
        KeyOutcome::default()
    }

    fn compose(&mut self, _id: u64, _c: Compose<'_>) -> bool {
        false
    }

    fn close(&mut self, _id: u64) {
        aba::fecha();
    }

    fn undo(&mut self, _id: u64, _redo: bool) -> bool {
        false
    }

    /// As células são edições do projeto: o Ctrl+Z com a aba à frente é o da cena.
    fn undo_is_the_projects(&self) -> bool {
        true
    }

    fn take_focus_request(&mut self) -> Option<u64> {
        aba::tira_pedido().then_some(ABA)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// O arranque da família.
pub fn install(hero: &mut HeroScreen) {
    hero.documents.install_surface(Box::new(AbaDasCamadas));
}
