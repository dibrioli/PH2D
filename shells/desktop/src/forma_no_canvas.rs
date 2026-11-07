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

    /// O miolo que não armou era um clique: volta à seleção normal, como o da câmera (17b A1).
    pub(crate) fn forma_up(&mut self) -> bool {
        let clique = self.gfx.as_ref().and_then(|g| {
            let w = g.sim.world();
            ph2d_shape_handles::clique_parado(w).zip(ph2d_shape_handles::editando(w))
        });
        let havia = self.forma_porta().is_some_and(|mut p| p.sobe());
        if let Some((s, dono)) = clique {
            self.clique_para_a_selecao(dono, s);
        }
        havia
    }

    pub(crate) fn forma_esc(&mut self) -> bool {
        let w = self.gfx.as_ref().map(|g| g.sim.world());
        if w.is_some_and(ph2d_shape_handles::a_meio) {
            self.autokey.desiste_do_gesto(&mut self.timeline);
        }
        self.forma_porta().is_some_and(|mut p| p.esc())
    }

    /// O cursor da alça agarrada ou sob o ponteiro.
    pub(crate) fn forma_cursor(&self) -> Option<winit::window::CursorIcon> {
        use ph2d_shape_handles::CursorDaAlca as K;
        use winit::window::CursorIcon as C;
        let g = self.gfx.as_ref()?;
        let hero = g.hero_screen.as_ref()?;
        let (s, w) = (self.last_pointer, g.sim.world());
        let agarrada = w
            .get_resource::<ph2d_shape_handles::FormaNoCanvas>()?
            .arrasto;
        if agarrada.is_none() && !self.pointer_on_canvas(s.0, s.1) {
            return None;
        }
        let win = crate::scene_mapping::janela(hero.view.center_split, g.surface.size());
        Some(match ph2d_shape_handles::cursor_em(w, &g.camera, win, s)? {
            K::Mover => C::Move,
            K::Horizontal => C::EwResize,
            K::Vertical => C::NsResize,
            K::Sobe => C::NeswResize,
            K::Desce => C::NwseResize,
        })
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
            Some(P::Clica(b)) => {
                use ph2d_panel_inspector::ids as i;
                use ph2d_shape_handles::Botao as B;
                let e = ph2d_ecs::Entity::from_bits(alvo);
                let pose = ph2d_shape_handles::Pose::de(g.sim.world(), e);
                g.camera.center = pose.map_or(g.camera.center, |p| p.t);
                let id = match b {
                    B::EditShape => i::INSP_PHYS_EDIT_SHAPE,
                    B::FitToPicture => i::INSP_PHYS_FIT_PICTURE,
                    B::Largura => i::INSP_PHYS_HALF_X,
                };
                if let Some((x, y)) = self.smoke_find_widget(id) {
                    self.smoke_pointer_down(x, y);
                    self.smoke_pointer_up();
                }
            }
            Some(P::Escreve(t)) => {
                self.smoke_type(t);
                self.smoke_key_enter();
            }
            Some(P::Esc) => {
                use winit::{
                    event::ElementState::Pressed, keyboard::KeyCode, keyboard::PhysicalKey,
                };
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
                eprintln!(
                    "[forma-mao] {s} · curvas={chaves} · passos={}",
                    self.undo.depth()
                );
            }
            None => {}
        }
    }
}
