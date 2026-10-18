//! **A cópia do motor** — o instantâneo do tocador de uma macro (`docs/Macros/02_plano_W5.md` §3) parte
//! dela. Todo o estado é dado e copia-se inteiro; o gancho de sonda [`Engine::on_dab`] não viaja (a cópia
//! nasce sem ele). O padrão sem `..` é a lei: um campo novo não compila aqui até alguém decidir se ele
//! viaja.

use super::Engine;

impl Clone for Engine {
    fn clone(&self) -> Self {
        let Engine {
            width,
            height,
            tuning,
            sim,
            layers,
            active_layer,
            paper_preset,
            paper_sheet,
            paper_tile,
            paper_dirty,
            paper_tooth,
            brush_tex,
            trail,
            stroke,
            stroke_events,
            tool,
            sliders,
            color,
            km_glaze,
            show_wet,
            stroke_down,
            has_prev_dab,
            prev_dab_x,
            prev_dab_y,
            stroke_tool_backup,
            lane_trails,
            silhouette_reach,
            on_dab: _,
            dirty,
            flow_ratio,
        } = self;
        Engine {
            width: *width,
            height: *height,
            tuning: tuning.clone(),
            sim: sim.clone(),
            layers: layers.clone(),
            active_layer: *active_layer,
            paper_preset: *paper_preset,
            paper_sheet: *paper_sheet,
            paper_tile: paper_tile.clone(),
            paper_dirty: *paper_dirty,
            paper_tooth: *paper_tooth,
            brush_tex: brush_tex.clone(),
            trail: trail.clone(),
            stroke: stroke.clone(),
            stroke_events: stroke_events.clone(),
            tool: *tool,
            sliders: *sliders,
            color: *color,
            km_glaze: *km_glaze,
            show_wet: *show_wet,
            stroke_down: *stroke_down,
            has_prev_dab: *has_prev_dab,
            prev_dab_x: *prev_dab_x,
            prev_dab_y: *prev_dab_y,
            stroke_tool_backup: *stroke_tool_backup,
            lane_trails: lane_trails.clone(),
            silhouette_reach: *silhouette_reach,
            on_dab: None,
            dirty: *dirty,
            flow_ratio: *flow_ratio,
        }
    }
}

/// Os bytes de um `Vec`, pelo comprimento (o que uma cópia aloca).
fn de<T>(v: &[T]) -> usize {
    std::mem::size_of_val(v)
}

impl crate::grid::Grid {
    /// **O que uma cópia deste grid retém** (o rascunho do solver nasce vazio nela). Sem `..`: um campo
    /// novo não compila aqui até alguém dizer quanto pesa.
    #[must_use]
    pub fn bytes_retidos(&self) -> usize {
        let crate::grid::Grid {
            w: _,
            h: _,
            s: _,
            rows: _,
            cells: _,
            film,
            susp,
            susp_rgb,
            sett,
            sett_rgb,
            relevo,
            altura,
            vela,
            flow: _,
            vel_x,
            vel_y,
            flow_x,
            flow_y,
            frow_lo,
            frow_hi,
            wet,
            paper,
            active,
            bloom,
            bx0: _,
            by0: _,
            bx1: _,
            by1: _,
            has_fluid: _,
            row_lo,
            row_hi,
            live_lo,
            live_hi,
            spans_enabled: _,
            scratch: _,
            paper_preset: _,
            paper_sheet: _,
        } = self;
        de(film)
            + de(susp)
            + de(susp_rgb)
            + de(sett)
            + de(sett_rgb)
            + de(relevo)
            + de(altura)
            + de(vela)
            + de(vel_x)
            + de(vel_y)
            + de(flow_x)
            + de(flow_y)
            + de(frow_lo)
            + de(frow_hi)
            + de(wet)
            + de(paper)
            + de(active)
            + de(bloom)
            + de(row_lo)
            + de(row_hi)
            + de(live_lo)
            + de(live_hi)
    }
}

impl crate::grid::GridSnapshot {
    #[must_use]
    pub fn bytes_retidos(&self) -> usize {
        de(&self.film)
            + de(&self.susp)
            + de(&self.susp_rgb)
            + de(&self.sett)
            + de(&self.sett_rgb)
            + de(&self.relevo)
            + de(&self.altura)
            + de(&self.vela)
            + de(&self.vel_x)
            + de(&self.vel_y)
            + de(&self.wet)
            + de(&self.active)
            + de(&self.bloom)
            + de(&self.row_lo)
            + de(&self.row_hi)
    }
}

impl Engine {
    /// **O que uma cópia deste motor retém**: as grades das camadas e os seus históricos, o papel e o
    /// grão. O rasto do pincel (uma janela do tamanho do dab) fica de fora — abaixo de 1 % a 1024²; o
    /// gate `medida_instantaneo_memoria` compara o total com o que o alocador vê.
    #[must_use]
    pub fn bytes_retidos(&self) -> usize {
        let camadas: usize = self
            .layers
            .iter()
            .map(|l| {
                l.grid.bytes_retidos()
                    + l.undo
                        .iter()
                        .chain(&l.redo)
                        .map(crate::grid::GridSnapshot::bytes_retidos)
                        .sum::<usize>()
            })
            .sum();
        camadas + de(&self.paper_tile) + self.brush_tex.as_deref().map_or(0, de)
    }
}
