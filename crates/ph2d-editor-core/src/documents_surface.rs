//! As abas de FAMÍLIA dentro de `Documents` — ver [`crate::doc_surface`].

use super::Documents;
use crate::doc_surface::{DocumentSurface, PointerLook, SurfaceTab, Surfaces};

impl Documents {
    /// A shell instala cada família com documentos em aba, uma vez, no arranque.
    pub fn install_surface(&mut self, family: Box<dyn DocumentSurface>) {
        self.surfaces.families.push(family);
    }

    pub fn surfaces(&self) -> &Surfaces {
        &self.surfaces
    }

    pub fn surfaces_mut(&mut self) -> &mut Surfaces {
        &mut self.surfaces
    }

    /// A aba de família activa, se a há.
    #[must_use]
    pub fn surface_active(&self) -> Option<SurfaceTab> {
        self.surface_active
    }

    /// Um documento (quadro ou família) tapa a cena com o chão dele — menos a aba de família que
    /// cede a área à cena neste quadro ([`crate::doc_surface::DocumentSurface::yields_to_scene`]).
    pub fn paints_document_ground(&mut self) -> bool {
        let cede = self
            .active_surface_mut()
            .is_some_and(|(f, _)| f.yields_to_scene());
        (self.surface_active.is_some() && !cede) || self.active.is_some()
    }

    /// ⭐ A `Cena` é a aba activa (nem quadro, nem documento de família).
    #[must_use]
    pub fn scene_active(&self) -> bool {
        self.active.is_none() && self.surface_active.is_none()
    }

    /// Torna activa uma aba de família (sai do quadro, se havia um). Uma aba que a família já não
    /// tem cai na `Cena`.
    pub fn activate_surface(&mut self, tab: SurfaceTab) {
        self.activate(None);
        let exists = self
            .surfaces
            .families
            .get(usize::from(tab.family))
            .is_some_and(|f| f.tabs().iter().any(|t| t.id == tab.id));
        self.surface_active = exists.then_some(tab);
    }

    /// A shell anota CADA carregar e largar da janela, antes de qualquer desvio: o largar fora da aba
    /// não lhe chega, mas o movimento seguinte leva-o em [`PointerInput::primary_held`].
    ///
    /// [`PointerInput::primary_held`]: crate::doc_surface::PointerInput::primary_held
    pub fn note_button(&mut self, kind: ph2d_host::PointerKind, button: ph2d_host::PointerButton) {
        if button == ph2d_host::PointerButton::Primary {
            match kind {
                ph2d_host::PointerKind::Down => self.primary_held = true,
                ph2d_host::PointerKind::Up => self.primary_held = false,
                ph2d_host::PointerKind::Move => {}
            }
        }
    }

    /// O botão primário está premido na janela?
    #[must_use]
    pub fn primary_held(&self) -> bool {
        self.primary_held
    }

    /// A família e o id da aba activa, juntos, para um evento.
    pub fn active_surface_mut(&mut self) -> Option<(&mut (dyn DocumentSurface + 'static), u64)> {
        let tab = self.surface_active?;
        let f = self.surfaces.get(tab.family)?;
        Some((f, tab.id))
    }

    /// O ponteiro que a aba de família activa quer em `(x, y)`; `None` = nenhuma, ou fora dela.
    #[must_use]
    pub fn surface_pointer_look(&self, x: f32, y: f32) -> Option<PointerLook> {
        let tab = self.surface_active?;
        let f = self.surfaces.families.get(usize::from(tab.family))?;
        f.pointer_look(tab.id, x, y)
    }

    /// A aba de família activa tem algo flutuante em `(x, y)` ([`DocumentSurface::floats_at`]).
    #[must_use]
    pub fn surface_floats_at(&self, x: f32, y: f32) -> bool {
        let Some(tab) = self.surface_active else {
            return false;
        };
        let f = self.surfaces.families.get(usize::from(tab.family));
        f.is_some_and(|f| f.floats_at(tab.id, x, y))
    }

    /// A roda sobre o chrome (`over_chrome`) é dele, salvo onde a aba activa flutua por cima
    /// ([`Self::surface_floats_at`]: um menu dela sobre um painel rola o menu).
    #[must_use]
    pub fn chrome_owns_wheel(&self, over_chrome: bool, x: f32, y: f32) -> bool {
        over_chrome && !self.surface_floats_at(x, y)
    }

    /// Pinta o que a aba de família activa tem a flutuar sobre a janela inteira `window`.
    pub fn paint_floating(
        &mut self,
        window: crate::zones::Rect,
        scene: &mut ph2d_vector::VectorScene,
        text: &mut ph2d_text::TextSystem,
        theme: ph2d_tokens::Theme,
    ) {
        if let Some((f, id)) = self.active_surface_mut() {
            f.paint_floating(id, window, scene, text, &theme);
        }
    }

    /// ⭐ Corre em todo quadro antes de pintar: uma família que abriu um documento pede a aba; uma
    /// aba activa que a família fechou cai na `Cena`.
    pub fn follow_surfaces(&mut self) {
        let mut wants = None;
        for (i, f) in self.surfaces.families.iter_mut().enumerate() {
            if let Some(id) = f.take_focus_request() {
                wants = Some(SurfaceTab {
                    family: u8::try_from(i).unwrap_or(u8::MAX),
                    id,
                });
            }
        }
        if let Some(tab) = wants {
            self.activate_surface(tab);
        } else if let Some(tab) = self.surface_active {
            let alive = self
                .surfaces
                .families
                .get(usize::from(tab.family))
                .is_some_and(|f| f.tabs().iter().any(|t| t.id == tab.id));
            if !alive {
                self.surface_active = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use ph2d_host::PointerButton::{Primary, Secondary};
    use ph2d_host::PointerKind::{Down, Move, Up};

    /// Só o primário conta, e o largar dele limpa-o onde quer que tenha caído.
    #[test]
    fn o_botao_primario_e_anotado_ate_largar() {
        let mut d = super::Documents::default();
        assert!(!d.primary_held());
        d.note_button(Down, Primary);
        d.note_button(Move, Primary);
        d.note_button(Up, Secondary);
        assert!(d.primary_held(), "outro botão não o larga");
        d.note_button(Up, Primary);
        assert!(!d.primary_held());
        d.note_button(Down, Secondary);
        assert!(!d.primary_held(), "outro botão não o prime");
    }
}
