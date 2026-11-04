//! A geometria de uma cena repetida noutro sítio (`append_transformed`) só se vê nas transformações: os
//! `draw_data` não a têm.

use crate::MockPanelHost;
use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::Panel;
use ph2d_editor_core::screens::HeroLayout;
use ph2d_editor_core::zones::Rect;

impl MockPanelHost {
    /// Pinta e devolve os alvos e a translação de cada transformação da cena.
    pub fn paint_with_translations<P: Panel>(
        &mut self,
        state: &mut P::State,
        viewport: Rect,
    ) -> (Vec<(NodeId, Rect)>, Vec<[f32; 2]>) {
        let scene = self.paint_scene::<P>(state, HeroLayout::for_viewport(viewport), viewport);
        let translacoes = scene
            .inner()
            .encoding()
            .transforms
            .iter()
            .map(|t| t.translation)
            .collect();
        (self.hit_index.iter_registrations().collect(), translacoes)
    }
}
