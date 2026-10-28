//! **A câmera no canvas — o fio do ponteiro** (plano GEF 17 A3/A4). A lei vive em [`ph2d_app_game::moldura`];
//! aqui só se projecta o ponteiro, se pergunta quem mais está por baixo e se escreve.

use ph2d_app_game::moldura::{AGARRE_PX, Alca, Arrasto, Escrita, alca_em, arrastar};
use ph2d_editor_core::CameraFieldEdit;
use ph2d_editor_core::action_bus::EditorAction;

impl crate::App {
    /// Agarra uma alça da câmera selecionada. `true` = agarrou (o `Down` não segue para a seleção).
    pub(crate) fn camera_frame_down(&mut self, sx: f32, sy: f32) -> bool {
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let Some(hero) = gfx.hero_screen.as_mut() else {
            return false;
        };
        let Some(m) = self.camera_no_canvas.molduras.iter().find(|m| m.selecionada) else {
            return false;
        };
        if hero.view.palco_do_jogo {
            return false;
        }
        let win = crate::scene_mapping::janela(hero.view.center_split, gfx.surface.size());
        let p = gfx.camera.screen_to_world((sx, sy), win);
        let q = gfx.camera.screen_to_world((sx + 1.0, sy), win);
        let tol = (q[0] - p[0]).hypot(q[1] - p[1]) * AGARRE_PX;
        let mut alca = alca_em(m, p, tol, false);
        if alca == Some(Alca::Dentro) {
            let mut pw = crate::hover_highlight::PickWorld {
                window_size: win,
                sim: &gfx.sim,
                vec_scene: &gfx.vec_scene,
                flip: &gfx.flip,
                present: &mut gfx.present,
                camera: &gfx.camera,
                pixels_per_meter: hero.project.pixels_per_meter,
            };
            let hits = crate::hover_highlight::pick_objects_at(
                &mut pw,
                &self.vec.entities,
                &self.vec.view_derived,
                &self.vec.live_drawn,
                &self.flip_state.entities,
                (sx, sy),
            );
            alca = alca_em(m, p, tol, hits.iter().any(|b| *b != m.camera));
        }
        let Some(alca) = alca else {
            return false;
        };
        let e = ph2d_ecs::Entity::from_bits(m.camera);
        let pose = gfx
            .sim
            .world()
            .get::<ph2d_ecs::Transform>(e)
            .map_or([0.0, 0.0], |t| [t.translation.x, t.translation.y]);
        let aspecto = hero.project.game_format.aspect();
        self.camera_no_canvas.arrasto = Some(Arrasto::novo(m, alca, p, aspecto, pose));
        true
    }

    /// Arrasta a alça agarrada. `true` = consumiu o movimento.
    pub(crate) fn camera_frame_move(&mut self, sx: f32, sy: f32) -> bool {
        let Some(a) = self.camera_no_canvas.arrasto.as_ref() else {
            return false;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return true;
        };
        let Some(hero) = gfx.hero_screen.as_mut() else {
            return true;
        };
        let win = crate::scene_mapping::janela(hero.view.center_split, gfx.surface.size());
        let p = gfx.camera.screen_to_world((sx, sy), win);
        let entity_bits = a.camera;
        let mut edita = |edit| {
            hero.bus
                .push(EditorAction::InspectorCameraEdit { entity_bits, edit });
        };
        match arrastar(a, p) {
            Escrita::Altura(h) => edita(CameraFieldEdit::Height(h)),
            Escrita::Limites(min, max) => {
                edita(CameraFieldEdit::LimitMin(min));
                edita(CameraFieldEdit::LimitMax(max));
            }
            Escrita::Pose(xy) => {
                let e = ph2d_ecs::Entity::from_bits(entity_bits);
                if let Some(mut t) = gfx.sim.world_mut().get_mut::<ph2d_ecs::Transform>(e) {
                    t.translation.x = xy[0];
                    t.translation.y = xy[1];
                }
            }
        }
        true
    }

    /// Larga a alça; o desfazer pós-quadro fecha o gesto num passo. `true` = havia um arrasto.
    pub(crate) fn camera_frame_up(&mut self) -> bool {
        self.camera_no_canvas.arrasto.take().is_some()
    }
}
