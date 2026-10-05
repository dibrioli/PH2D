//! **A ferramenta TRIM na shell** (plano 38) — o ponteiro e o quadro. A costura (o pedaço sob o
//! cursor, o corte, o realce) mora em [`ph2d_app_vec::trim`].

use ph2d_app_vec::trim::{TrimHit, hit_at, piece_world};

impl crate::App {
    /// ⭐⭐⭐ **Recalcula o pedaço sob o cursor** — uma vez por quadro, no topo, ao lado do realce de
    /// proveniência (que responde à mesma pergunta noutra tinta).
    ///
    /// ⚠️ **Fora do modo Trim ele é LIMPO**, e não simplesmente não-actualizado: um realce vermelho
    /// deixado a arder depois de trocar de ferramenta prometeria um corte que nenhum clique faria.
    pub(crate) fn refresh_trim_hover(&mut self, pointer: (f32, f32)) {
        if self.vec.draw_config.mode != ph2d_tool_vector::DrawMode::Trim {
            self.vec.trim_hit = None;
            self.vec.trim_piece.clear();
            return;
        }
        let Some(world) = self.vec_world_at(pointer) else {
            self.vec.trim_hit = None;
            self.vec.trim_piece.clear();
            return;
        };
        let Some(tol) = self.trim_tolerance() else {
            self.vec.trim_hit = None;
            self.vec.trim_piece.clear();
            return;
        };
        let achado = self.trim_hit_at(world, tol);
        self.vec.trim_piece = match (&achado, self.gfx.as_ref()) {
            (Some(h), Some(gfx)) => {
                let xf = ph2d_vec_entities::transform::build(&gfx.sim, &self.vec.entities);
                piece_world(&gfx.vec_scene, &xf, h)
            }
            _ => Vec::new(),
        };
        self.vec.trim_hit = achado;
    }

    /// **O RAIO DE CAPTURA em unidades de MUNDO** — o mesmo que as outras ferramentas de apontar
    /// curva usam, para o Trim não pegar a uma distância diferente das vizinhas.
    pub(crate) fn trim_tolerance(&self) -> Option<f64> {
        let gfx = self.gfx.as_ref()?;
        Some(ph2d_app_vec::vec_gizmo_view::stroke_hit_r(
            &gfx.camera,
            gfx.scene_window(),
        ))
    }

    /// **O pedaço no ponto de MUNDO `world`** — a porta única, com dois chamadores: o realce (por
    /// quadro) e o clique. ⛔ Duas rotas para *"o que está sob o cursor"* divergiriam no primeiro
    /// ajuste de tolerância, e o artista veria uma coisa e apagaria outra.
    pub(crate) fn trim_hit_at(&self, world: [f64; 2], tol: f64) -> Option<TrimHit> {
        let gfx = self.gfx.as_ref()?;
        let xf = ph2d_vec_entities::transform::build(&gfx.sim, &self.vec.entities);
        hit_at(&self.vec.pen, &gfx.vec_scene, &xf, world, tol)
    }
}
