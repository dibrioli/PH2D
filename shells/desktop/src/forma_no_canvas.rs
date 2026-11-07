//! **As alças da forma no canvas** (ficha f1, onda 3): só o fio do ponteiro; a lei vive em [`ph2d_shape_handles`].

use ph2d_shape_handles::{Porta, Teclas};

impl crate::App {
    fn forma_porta(&mut self) -> Option<Porta<'_>> {
        let gfx = self.gfx.as_mut()?;
        let hero = gfx.hero_screen.as_mut()?;
        Some(Porta {
            window: crate::scene_mapping::janela(hero.view.center_split, gfx.surface.size()),
            camera: gfx.camera,
            world: gfx.sim.world_mut(),
            bus: &mut hero.bus,
        })
    }

    pub(crate) fn forma_down(&mut self, sx: f32, sy: f32) -> bool {
        self.forma_porta().is_some_and(|mut p| p.desce((sx, sy)))
    }

    pub(crate) fn forma_move(&mut self, sx: f32, sy: f32) -> bool {
        let teclas = Teclas {
            alt: self.modifiers.alt_key(),
            shift: self.modifiers.shift_key(),
        };
        self.forma_porta()
            .is_some_and(|mut p| p.move_((sx, sy), teclas))
    }

    pub(crate) fn forma_up(&mut self) -> bool {
        self.forma_porta().is_some_and(|mut p| p.sobe())
    }

    pub(crate) fn forma_esc(&mut self) -> bool {
        self.forma_porta().is_some_and(|mut p| p.esc())
    }

    /// A mão das fotos `fisica_forma_*`: cada passo pelas portas reais do botão e do ponteiro; `[forma-mao]` no `stderr`.
    pub(crate) fn mao_da_forma_do_smoke(&mut self) {
        use ph2d_shape_handles::{PassoDaMao as P, passo_da_mao};
        let Some(ph2d_app_game::smoke::Foto::Fisica(c)) = ph2d_app_game::smoke::foto() else {
            return;
        };
        let Some(plano) = ph2d_app_game::smoke::plano_da_forma(c) else {
            return;
        };
        let Some(g) = self.gfx.as_mut() else { return };
        let Some(hero) = g.hero_screen.as_mut() else {
            return;
        };
        let Some(alvo) = hero.gizmo.selection else {
            return;
        };
        let win = crate::scene_mapping::janela(hero.view.center_split, g.surface.size());
        match passo_da_mao(plano, g.sim.world(), alvo, &g.camera, win) {
            Some(P::Clica) => {
                // A foto mostra a forma no meio da vista; o botão é o do cartão, pelo ponteiro.
                let e = ph2d_ecs::Entity::from_bits(alvo);
                let pose = ph2d_shape_handles::Pose::de(g.sim.world(), e);
                g.camera.center = pose.map_or(g.camera.center, |p| p.t);
                let id = ph2d_panel_inspector::ids::INSP_PHYS_EDIT_SHAPE;
                if let Some((x, y)) = self.smoke_find_widget(id) {
                    self.smoke_pointer_down(x, y);
                    self.smoke_pointer_up();
                }
            }
            Some(P::Esc) => {
                use winit::{event::ElementState::Pressed, keyboard::KeyCode, keyboard::PhysicalKey};
                self.escape_key(PhysicalKey::Code(KeyCode::Escape), Pressed, false);
            }
            Some(P::Desce(x, y)) => self.smoke_pointer_down(x, y),
            Some(P::Move(x, y, t)) => {
                use winit::keyboard::ModifiersState as M;
                let alt = if t.alt { M::ALT } else { M::empty() };
                self.modifiers = alt | if t.shift { M::SHIFT } else { M::empty() };
                self.smoke_pointer_move(x, y);
            }
            Some(P::Sobe) => self.smoke_pointer_up(),
            Some(P::Desfaz) => self.undo_request = Some(false),
            Some(P::Relata(s)) => {
                let chaves = self.timeline.doc.bindings().iter();
                let chaves = chaves.filter(|b| b.entity == alvo).count();
                eprintln!("[forma-mao] {s} · curvas={chaves} · passos={}", self.undo.depth());
            }
            None => {}
        }
    }
}
