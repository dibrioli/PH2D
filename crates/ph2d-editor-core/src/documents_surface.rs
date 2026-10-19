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
