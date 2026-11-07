//! **A câmera no canvas — o fio do ponteiro** (plano GEF 17 A3/A4, cura 17b). A lei vive em
//! [`ph2d_camera_frame`]; aqui só se projecta o ponteiro, se escreve e se devolve o clique à seleção.

use ph2d_camera_frame::{AGARRE_PX, Alca, Arrasto, alca_em, aplica, arrastar, encaixa, padrao};

impl crate::App {
    /// Agarra uma alça da câmera selecionada. `true` = agarrou (o `Down` não segue para a seleção).
    pub(crate) fn camera_frame_down(&mut self, sx: f32, sy: f32) -> bool {
        let curar = self.modifiers.shift_key()
            || self.modifiers.super_key()
            || self.modifiers.control_key();
        let estado = &mut self.undo.camera_no_canvas;
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let Some(hero) = gfx.hero_screen.as_ref() else {
            return false;
        };
        let Some(m) = estado.molduras.iter().find(|m| m.selecionada).cloned() else {
            return false;
        };
        if hero.view.palco_do_jogo || !estado.em_edicao {
            return false;
        }
        let win = crate::scene_mapping::janela(hero.view.center_split, gfx.surface.size());
        let aspecto = hero.project.game_format.aspect();
        let p = gfx.camera.screen_to_world((sx, sy), win);
        let q = gfx.camera.screen_to_world((sx + 1.0, sy), win);
        let tol = (q[0] - p[0]).hypot(q[1] - p[1]) * AGARRE_PX;
        // Com modificador o artista está a curar a seleção: o miolo não a segura.
        let Some(alca) = alca_em(&m, p, tol).filter(|a| !(curar && *a == Alca::Dentro)) else {
            return false;
        };
        // GEF 20 C4: o 2.º press na MESMA alça devolve-a à fábrica (os limites ao contorno do nível).
        if estado
            .ultimo
            .duplo(crate::App::timestamp_ns(), m.camera, alca)
        {
            let contorno = ph2d_camera_frame::contorno_do_nivel(gfx.sim.world_mut());
            if let (Some(e), Some(hero)) = (padrao(&m, alca, contorno), gfx.hero_screen.as_mut()) {
                aplica(gfx.sim.world_mut(), &mut hero.bus, m.camera, e);
            }
            return true;
        }
        let e = ph2d_ecs::Entity::from_bits(m.camera);
        let pose = gfx
            .sim
            .world()
            .get::<ph2d_ecs::Transform>(e)
            .map_or([0.0, 0.0], |t| [t.translation.x, t.translation.y]);
        let mut a = Arrasto::novo(&m, alca, p, aspecto, pose, (sx, sy));
        // Filha de outro objecto: o miolo anda no MUNDO e a local sai da inversa do pai (rodado, escalado).
        if gfx.sim.world().get::<ph2d_ecs::ChildOf>(e).is_some() {
            a = a.com_pai(ph2d_ecs::parent_world_transform(gfx.sim.world(), e));
        }
        estado.arrasto = Some(a);
        estado.vista_do_down = Some(gfx.camera);
        true
    }

    /// Arrasta a alça agarrada. `true` = consumiu o movimento (o miolo por armar também o consome).
    pub(crate) fn camera_frame_move(&mut self, sx: f32, sy: f32) -> bool {
        let shift = self.modifiers.shift_key();
        let estado = &mut self.undo.camera_no_canvas;
        let Some(a) = estado.arrasto.as_mut() else {
            return false;
        };
        if !a.arma((sx, sy)) {
            return true;
        }
        let Some(gfx) = self.gfx.as_mut() else {
            return true;
        };
        let Some(hero) = gfx.hero_screen.as_mut() else {
            return true;
        };
        let win = crate::scene_mapping::janela(hero.view.center_split, gfx.surface.size());
        let vista = estado.vista_do_down.unwrap_or(gfx.camera);
        let escrita = arrastar(a, vista.screen_to_world((sx, sy), win));
        // GEF 20 C4: o Shift encaixa no passo do campo que a alça mexe.
        let escrita = if shift { encaixa(escrita) } else { escrita };
        aplica(gfx.sim.world_mut(), &mut hero.bus, a.camera, escrita);
        true
    }

    /// GEF 20 C4 — o `Esc` a meio de um arrasto da câmera devolve a alça ao valor do `Down`. `true` = consumido.
    pub(crate) fn camera_frame_escape(&mut self) -> bool {
        // GEF 20 C2: o Esc desarma o Eyedropper do Target.
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut())
            && hero.view.camera_canvas.escolhe_alvo.take().is_some()
        {
            return true;
        }
        let estado = &mut self.undo.camera_no_canvas;
        let Some(a) = estado.arrasto.take_if(|a| !a.era_um_clique()) else {
            return false;
        };
        estado.vista_do_down = None;
        if let Some(gfx) = self.gfx.as_mut()
            && let Some(hero) = gfx.hero_screen.as_mut()
        {
            aplica(gfx.sim.world_mut(), &mut hero.bus, a.camera, a.restaura());
        }
        true
    }

    /// O Eyedropper do Target do Follow está armado (GEF 20 C2).
    pub(crate) fn camera_target_armed(&self) -> bool {
        self.gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .is_some_and(|h| h.view.camera_canvas.escolhe_alvo.is_some())
    }

    /// O cursor da alça sob o ponteiro (ou da que está agarrada) — o gesto que ela promete (17b A10). Com o
    /// Eyedropper do Target armado, a cruz de escolher.
    pub(crate) fn camera_frame_cursor(&self) -> Option<winit::window::CursorIcon> {
        use winit::window::CursorIcon as C;
        if self.camera_target_armed()
            && self.pointer_on_canvas(self.last_pointer.0, self.last_pointer.1)
        {
            return Some(C::Crosshair);
        }
        let estado = &self.undo.camera_no_canvas;
        let alca = match estado.arrasto.as_ref() {
            Some(a) => a.alca,
            None => {
                let gfx = self.gfx.as_ref()?;
                let hero = gfx.hero_screen.as_ref()?;
                let m = estado.molduras.iter().find(|m| m.selecionada)?;
                let (sx, sy) = self.last_pointer;
                if hero.view.palco_do_jogo || !estado.em_edicao || !self.pointer_on_canvas(sx, sy) {
                    return None;
                }
                let win = crate::scene_mapping::janela(hero.view.center_split, gfx.surface.size());
                let p = gfx.camera.screen_to_world((sx, sy), win);
                let q = gfx.camera.screen_to_world((sx + 1.0, sy), win);
                alca_em(m, p, (q[0] - p[0]).hypot(q[1] - p[1]) * AGARRE_PX)?
            }
        };
        Some(match alca {
            Alca::Canto(i) | Alca::LimiteCanto(i) if i % 2 == 0 => C::NeswResize,
            Alca::Canto(_) | Alca::LimiteCanto(_) => C::NwseResize,
            Alca::LimiteBorda(0 | 1) | Alca::ZonaMorta(0 | 1) => C::EwResize,
            Alca::LimiteBorda(_) | Alca::ZonaMorta(_) => C::NsResize,
            Alca::Dentro | Alca::LimiteCentro => C::Move,
        })
    }

    /// As tarjas do formato do jogo no palco (plano GEF 17): ali o canvas é a janela inteira.
    pub(crate) fn tarjas_do_palco(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let Some(hero) = gfx.hero_screen.as_ref() else {
            return;
        };
        let w = gfx.surface.size();
        #[allow(clippy::cast_precision_loss)]
        let canvas = [0.0, 0.0, w.width as f32, w.height as f32];
        let caixa = ph2d_app_game::encaixe::caixa_do_jogo(canvas, hero.project.game_format);
        ph2d_camera_frame::tinta::tarjas(&mut gfx.vector_scene, canvas, caixa, gfx.theme);
    }

    /// Larga a alça; o desfazer pós-quadro fecha o gesto num passo. `true` = havia um arrasto. ⭐ O miolo que
    /// nunca armou era um CLIQUE (17b A1): escolhe o objecto de cima sob o `Down` que não é a câmera; sem nenhum,
    /// a câmera fica.
    pub(crate) fn camera_frame_up(&mut self) -> bool {
        self.undo.camera_no_canvas.vista_do_down = None;
        let Some(a) = self.undo.camera_no_canvas.arrasto.take() else {
            return false;
        };
        if a.era_um_clique() {
            self.clique_para_a_selecao(a.camera, a.ecra0());
        }
        true
    }

    /// O clique que uma edição no canvas não usou: o objecto de cima sob ele que não é o `dono`; o clique no
    /// próprio dono, ou sem ninguém, deixa-o ficar.
    pub(crate) fn clique_para_a_selecao(&mut self, dono: u64, ecra: (f32, f32)) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let Some(hero) = gfx.hero_screen.as_mut() else {
            return;
        };
        let mut pw = crate::hover_highlight::PickWorld {
            window_size: crate::scene_mapping::janela(hero.view.center_split, gfx.surface.size()),
            sim: &gfx.sim,
            vec_scene: &gfx.vec_scene,
            flip: &gfx.flip,
            frame_order: &gfx.frame_order,
            present: &mut gfx.present,
            camera: &gfx.camera,
            pixels_per_meter: hero.project.pixels_per_meter,
            camera_icon_px: hero.project.camera_icon_px,
            camera_frames: &self.undo.camera_no_canvas.molduras,
            motion: Some(ph2d_app_motion::motion_pick::Alvo::new(
                &gfx.motion,
                gfx.surface.gpu(),
                &mut self.motion_shell.clique,
            )),
        };
        let mut hits = crate::hover_highlight::pick_objects_at(
            &mut pw,
            &self.vec.entities,
            &self.vec.view_derived,
            &self.vec.live_drawn,
            &self.flip_state.entities,
            ecra,
        );
        if hits.first() == Some(&dono) {
            return;
        }
        hits.retain(|b| *b != dono);
        ph2d_pick_order::descendants_first(gfx.sim.world(), &mut hits);
        if let Some(&b) = hits.first() {
            ph2d_editor_core::screens::hero::mode_drive::chosen(hero, Some(b), false);
            hero.gizmo.replace_selection(Some(b));
        }
    }
}
