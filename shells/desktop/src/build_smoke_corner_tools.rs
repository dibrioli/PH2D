//! A cena de smoke das ferramentas de QUINA (Fillet / Chamfer) — `PH2D_BUILD_SMOKE=16`.
//! Módulo irmão de `build_smoke` (teto de 600 LOC). Frame 3 monta a cena e arma o modo Fillet;
//! frame 4 (pós-`settle`) pré-seleciona o retângulo para as quinas já aparecerem.

use crate::build_smoke::shape;
use ph2d_vec_scene::ShapeKind;

impl crate::App {
    /// Frame 3: um retângulo de quinas RETAS + uma elipse de âncoras SUAVES, no modo Fillet.
    pub(crate) fn smoke_corner_tools_build(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
        let scene = &mut gfx.vec_scene;
        // Retângulo de quinas AFIADAS (radius 0) à esquerda.
        scene.push_path(shape(
            ShapeKind::Rectangle,
            [-3.4, -1.2],
            [-1.0, 1.2],
            &[],
            [90, 150, 220],
        ));
        // Elipse à direita — suas âncoras são SUAVES; o clique as vira quina primeiro.
        scene.push_path(shape(
            ShapeKind::Ellipse,
            [1.0, -1.2],
            [3.4, 1.2],
            &[],
            [200, 120, 80],
        ));
        self.vec_set_draw_mode(ph2d_tool_vector::DrawMode::Fillet);
    }

    /// Frame 4: pré-seleciona o retângulo (sem seleção o overlay de edição não desenha as
    /// âncoras que se vai clicar) e imprime as instruções do smoke.
    pub(crate) fn smoke_corner_tools_select(&mut self) {
        let first = self
            .gfx
            .as_ref()
            .expect("gfx")
            .vec_scene
            .paths()
            .first()
            .map(|p| p.id);
        if let Some(id) = first {
            self.vec_pen.select(Some(id));
        }
        eprintln!(
            "[smoke] Fillet/Chamfer tools: no rail, os pills **Fillet** e **Chamfer**. \
             Clique uma QUINA do retângulo e ARRASTE para dentro — ela arredonda (Fillet) \
             ou chanfra (Chamfer). Na ELIPSE, clicar uma âncora a transforma em quina \
             PRIMEIRO e então arredonda. Arrastar para FORA afia de volta."
        );
    }
}
