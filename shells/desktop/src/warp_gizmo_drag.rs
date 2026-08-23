//! **A costura do gizmo dos deformadores de quadrilátero com o PONTEIRO** — a ponta que
//! faz as alças deixarem de ser decoração.
//!
//! A geometria vive em [`crate::render_loop::warp_gizmo`], pura e testada; o desenho no
//! `warp_overlay`. Aqui mora só o que precisa do mundo: onde está o cursor, que alça ele
//! agarrou, e por que porta a edição sai.
//!
//! # ⚠️ A edição sai pela MESMA porta do painel
//!
//! O arrasto escreve `Graph::set_param` — a mesma função que o slider chama. Não há um
//! segundo caminho de escrita, então o commit, o undo e o que o painel mostra **não
//! podem divergir**. *Duas portas para o mesmo estado é como duas portas divergem.*
//!
//! # ⚠️ Um arrasto é UM passo de undo, e ninguém aqui trata disso
//!
//! O `post_frame_undo` suprime o registo enquanto um botão está premido, então o gesto
//! inteiro fecha num passo. É a mesma lei do arrasto da âncora, e a razão de este ficheiro
//! não ter máquina de transacção nenhuma.

use crate::render_loop::warp_gizmo::{self, WarpGizmoView, WarpHandle};

/// O arrasto em curso: o retrato congelado no `Down`, a alça agarrada, e de onde ela
/// partiu.
///
/// ⚠️ **A caixa e os offsets de partida são CONGELADOS**, e isso é a correcção do gesto:
/// mover uma alça move as peças, o que move a caixa envolvente do nó de baixo — mas os
/// params deste nó referem-se à caixa de ENTRADA, que não muda durante o arrasto. Reler a
/// caixa a cada movimento faria o gizmo perseguir a própria cauda.
#[derive(Clone, Copy, Debug)]
pub(crate) struct WarpGizmoDrag {
    pub(crate) view: WarpGizmoView,
    pub(crate) handle: WarpHandle,
    /// Os dois offsets do param no instante do `Down`.
    pub(crate) start: [f32; 2],
    /// O ponto de MUNDO em que o dedo pousou.
    pub(crate) anchor: [f32; 2],
}

impl crate::App {
    /// A janela da CENA (o sub-retângulo do split) — a mesma que o gizmo de field usa,
    /// e a mesma com que o render faz o `set_viewport`. Uma janela cheia aqui daria o
    /// *drift* crónico que aquele arquivo documenta.
    fn warp_scene_window(&self) -> Option<ph2d_host::WindowSize> {
        let gfx = self.gfx.as_ref()?;
        let hero = gfx.hero_screen.as_ref()?;
        // ⚠️ **A MESMA porta que a tinta usa** — ver `warp_gizmo::scene_window`, que
        // carrega o relato do defeito em que as duas discordaram.
        Some(crate::render_loop::warp_gizmo::scene_window(
            hero.view.center_split,
            gfx.surface.size(),
        ))
    }

    /// Quantas unidades de MUNDO vale um pixel de tela, na câmara da cena.
    fn warp_world_per_px(&self) -> Option<f32> {
        let win = self.warp_scene_window()?;
        let cam = &self.gfx.as_ref()?.camera;
        // Dois pontos de tela a um pixel de distância, medidos no mundo.
        let a = cam.screen_to_world((0.0, 0.0), win);
        let b = cam.screen_to_world((1.0, 0.0), win);
        let d = (b[0] - a[0]).hypot(b[1] - a[1]);
        (d.is_finite() && d > 0.0).then_some(d)
    }

    /// O ponto de tela em coordenadas de MUNDO, na câmara da cena.
    fn warp_world_at(&self, sx: f32, sy: f32) -> Option<[f32; 2]> {
        let win = self.warp_scene_window()?;
        Some(self.gfx.as_ref()?.camera.screen_to_world((sx, sy), win))
    }

    /// Tenta agarrar uma alça. `true` = agarrou, e o `Down` **não** deve seguir para o
    /// pick de sprite nem para o gizmo genérico.
    pub(crate) fn warp_gizmo_down(&mut self, sx: f32, sy: f32) -> bool {
        let Some(view) = warp_gizmo::view() else {
            return false;
        };
        let Some(world) = self.warp_world_at(sx, sy) else {
            return false;
        };
        let Some(wpp) = self.warp_world_per_px() else {
            return false;
        };
        let Some(gfx) = self.gfx.as_ref() else {
            return false;
        };
        let port = warp_gizmo::param_port(&gfx.motion, view.node);
        let (hs, n) = warp_gizmo::view_handles(&view, &port);
        let Some(i) = warp_gizmo::hit(&hs[..n], world, wpp) else {
            return false;
        };
        let h = hs[i];
        self.warp_drag = Some(WarpGizmoDrag {
            view,
            handle: h,
            start: [port(h.param[0]), port(h.param[1])],
            anchor: world,
        });
        true
    }

    /// Move a alça agarrada. `true` = consumiu o movimento.
    pub(crate) fn warp_gizmo_move(&mut self, sx: f32, sy: f32) -> bool {
        let Some(drag) = self.warp_drag else {
            return false;
        };
        let Some(world) = self.warp_world_at(sx, sy) else {
            return true; // agarrado, mas sem câmara: consome e não escreve lixo
        };
        let delta = [world[0] - drag.anchor[0], world[1] - drag.anchor[1]];
        let Some(edits) = warp_gizmo::edits(
            &drag.handle,
            drag.start,
            delta,
            drag.view.warp,
            drag.view.down,
        ) else {
            return true;
        };
        if let Some(gfx) = self.gfx.as_mut() {
            for (name, value) in edits {
                gfx.motion.doc.graph.set_param(drag.view.node, name, value);
            }
            gfx.motion.pump.mark_dirty();
        }
        true
    }

    /// Larga a alça. `true` = havia um arrasto.
    pub(crate) fn warp_gizmo_up(&mut self) -> bool {
        self.warp_drag.take().is_some()
    }
}
