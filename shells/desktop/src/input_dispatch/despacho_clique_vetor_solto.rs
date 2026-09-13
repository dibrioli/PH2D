//! **O clique, o SOLTAR da ferramenta vetorial** — ramos do `on_mouse_input` ([`super`]): o osso que nasce do
//! arrasto, os gestos que se fecham (Build, conector, gradiente, gaiola, região, Width, lápis) e o release da
//! caneta e da forma, com a solda. Os corpos MUDARAM-SE verbatim (`line/input-dispatch`, 2026-09-13) e correm no
//! sítio da chamada, pela mesma ordem.
//!
//! ⚠️ Cada ramo devolve `true` onde o braço fazia `return;` — e o braço, ao receber `true`, devolve. Um Up que
//! nenhum consome cai no chrome, como caía.

use super::*;

impl crate::App {
    /// O release da caneta (e o diagnóstico das quinas) ou da forma em arrasto — com a solda dos endpoints e a
    /// forma nova seleccionada.
    pub(super) fn ramo_vetor_solto_caneta(&mut self) -> bool {
        if shape_kind_for_mode(&self.vec.draw_config).is_none() {
            // Pen: the release ends a handle drag / grab.
            let consumed = self.vec.pen.on_release();
            // DIAGNÓSTICO (`PH2D_CORNER_LOG=1`): os raios LOGO APÓS o gesto. Com o
            // log do press, parte o report em dois — se aqui os raios anteriores já
            // sumiram, foi o GESTO; se estão inteiros e somem até o press seguinte,
            // foi um passe POR-FRAME entre os dois.
            if std::env::var_os("PH2D_CORNER_LOG").is_some()
                && self.vec.draw_config.mode.is_corner_tool()
                && let Some(gfx) = self.gfx.as_ref()
                && let Some(pid) = self.vec.pen.selected()
            {
                let shape = self.vec.entities.get(&pid).is_some_and(|&b| {
                    gfx.sim
                        .world()
                        .get::<ph2d_ecs::VecShape>(ph2d_ecs::Entity::from_bits(b))
                        .is_some()
                });
                let radii: Vec<f64> = gfx
                    .vec_scene
                    .path(pid)
                    .map(|p| p.verts_all().map(|v| v.corner_radius).collect())
                    .unwrap_or_default();
                eprintln!("[corner] RELEASE shape={shape} radii={radii:?}");
            }
            if consumed {
                return true;
            }
        } else if shape_up_consumes(self.vec.draw_config.mode, self.vec.shape.is_active()) {
            // A shape drag is in progress → finalize it. Commit if the
            // drag spanned a real size, else discard the stray click
            // (cancel the pending undo so it doesn't record a spurious
            // `next_id`-only step). ONLY consume the Up when a shape is
            // actually being drawn — otherwise (e.g. releasing over a
            // panel button while in a shape mode) the Up MUST fall
            // through to the chrome dispatch, else every panel click
            // (mode switch, boolean, close) is silently swallowed.
            let committed = if let Some(gfx) = self.gfx.as_mut() {
                let c = self.vec.shape.on_release(&mut gfx.vec_scene);
                if c {
                    // Solda os endpoints da forma recém-criada com nós
                    // vizinhos: basta ficarem próximos para se fundirem, e
                    // várias linhas/arcos fecham numa forma (Enio
                    // 2026-07-09). A forma nova ainda não tem entidade
                    // (o sync roda depois), então está na identidade; a
                    // geometria PRÉ-existente nunca se mexe (só a nova
                    // snapa nela). Ao fechar num laço, recebe o fill do
                    // estilo atual — como uma região desenhada pela pen.
                    if let Some(new_id) = self.vec.shape.selected() {
                        let fill = self.vec.pen.style().fill;
                        let fill_on_close =
                            (fill.a != 0).then(|| ph2d_vec_scene::Paint::solid(fill));
                        let xforms =
                            ph2d_vec_entities::transform::build(&gfx.sim, &self.vec.entities);
                        let win = gfx.surface.size();
                        let tol = crate::vec_gizmo_view::stroke_hit_r(&gfx.camera, win) * 1.5;
                        gfx.vec_scene
                            .weld_new_shape(new_id, &xforms, tol, fill_on_close);
                    }
                }
                c
            } else {
                false
            };
            if committed {
                // Seleciona a forma nova para edição imediata — a menos que
                // o weld a tenha fundido noutro objeto (o id sumiu).
                let sel = self.vec.shape.selected().filter(|id| {
                    self.gfx
                        .as_ref()
                        .is_some_and(|g| g.vec_scene.paths().iter().any(|p| p.id == *id))
                });
                self.vec.pen.select(sel);
            }
            return true;
        }
        // Shape mode but no active drag → fall through to chrome so the
        // panel buttons receive their Up.
        false
    }
}
