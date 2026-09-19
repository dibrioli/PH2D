//! **Fase do quadro: A PILHA DE FILTROS NO PAINEL** — os tipos, os nomes de mistura e as linhas da pilha de FX raster da selecção, publicados no
//! painel vectorial (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_selection_mirror_filters(&mut self, sel: Vec<ph2d_vec_scene::VecPathId>) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx { sim, .. } = FrameGfx::of(gfx);
        // Filters (a pilha de FX raster, plano 24): as MESMAS duas perguntas do Contour —
        // *"esta seleção pode receber um filtro?"* (há forma) e *"o que está armado?"*. O
        // painel lê a pilha do PRIMEIRO caminho selecionado que tenha uma; a seção some sem
        // seleção e sem pilha viva. `radius`/`offset`/`opacity` são MUNDO/normalizados —
        // sem conversão de escala (o raio de mundo é o número que o slider mostra).
        let filt = sel
            .iter()
            .find_map(|id| crate::fx_live::spec_of(sim, &self.vec.entities, *id));
        ph2d_panel_vector::set_current_filter_can_add(!sel.is_empty());
        // A TABELA dos tipos vem do MOTOR (o painel não alcança o `ph2d-ecs`) — uma
        // segunda tabela discordaria do `kind` na primeira adição, e com sete tipos o
        // modo de falha é um knob morto (ou um que falta) que nenhum gate vê.
        ph2d_panel_vector::set_filter_kinds(
            ph2d_ecs::FxOp::SPECS
                .iter()
                .map(|s| ph2d_panel_vector::FilterKindView {
                    name: s.name,
                    radius_label: s.radius_label,
                    offset_labels: s.offset_labels,
                    color_label: s.color_label,
                    color_b_label: s.color_b_label,
                    modes: s.modes,
                    takes_blend: s.takes_blend,
                    takes_ramp: s.takes_ramp,
                    noise_labels: s.noise_labels,
                    grow_label: s.grow_label,
                    adjust_labels: s.adjust_labels,
                })
                .collect(),
        );
        // …e os NOMES das leis de mistura, pela mesma porta e pelo mesmo motivo: quem
        // conhece o `BlendMode` é a shell, não o painel. As leis de COBERTURA (`Behind` /
        // `Clear`) ficam de fora — um degrau aplica a lei dele onde a cobertura já está
        // decidida, e oferecê-las seria a opção que despacha e mente.
        ph2d_panel_vector::set_filter_blend_names(
            (0..ph2d_ecs::FxOp::BLEND_KINDS)
                // ⚠️ `tr(label_key())` desde 2026-09-19: o motor publica a CHAVE, e quem resolve é
                //    quem já fala a tabela — aqui, a shell.
                .map(|m| ph2d_i18n::tr(ph2d_painter_effects::BlendMode::from_u8(m).label_key()))
                .collect(),
        );
        ph2d_panel_vector::set_current_filters(
            filt.map(|f| {
                f.ops
                    .iter()
                    .map(|op| ph2d_panel_vector::FilterRowView {
                        kind: op.kind,
                        mode: op.mode,
                        enabled: op.enabled,
                        radius: f64::from(op.radius),
                        offx: f64::from(op.offset[0]),
                        offy: f64::from(op.offset[1]),
                        color: crate::fx_live::colour_bytes(op.color),
                        color_b: crate::fx_live::colour_bytes(op.color_b),
                        opacity: f64::from(op.opacity),
                        blend: op.blend_code(),
                        scale: f64::from(op.scale),
                        // ⚠️ **`detail_clamped`, não `detail`** — a mesma metade de
                        // HONRAR que o produtor da GPU usa. O painel mostra o número que
                        // o dispositivo de fato soma.
                        detail: op.detail_clamped(),
                        seed: op.seed,
                        grow: f64::from(op.grow),
                        // ⚠️ **VOLTAS -> GRAUS na fronteira.** O modelo fala voltas (a
                        // unidade do `HsbParams` do Painter, que é a MESMA lei); o painel
                        // fala graus, porque é a unidade em que um artista pensa uma cor.
                        // A volta é feita aqui e no `apply`, em linhas que se leem juntas
                        // — o idioma que a §12 da física já usa com radianos.
                        hue: f64::from(op.hue) * 360.0,
                        sat: f64::from(op.sat),
                        bright: f64::from(op.bright),
                        stop_pos: op.stop_pos,
                        stop_colors: op.stops.map(crate::fx_live::colour_bytes),
                        stop_count: op.stop_count,
                        // ⚠️ **A rampa é amostrada AQUI, pela função que é o ORÁCULO dos
                        // gates de paridade** (`gradient_map_lut` do
                        // `ph2d-painter-effects`) — o bar é o que o artista lê para prever
                        // o render, então ele TEM de sair da mesma lei que o device honra.
                        // Um lerp de conveniência no painel divergiria em gama justo nos
                        // meios-tons, e o único lugar onde isso apareceria é uma
                        // screenshot. Medido: device vs esta função, **1 nível de byte**.
                        ramp_preview: crate::fx_live::ramp_preview(op),
                    })
                    .collect()
            })
            .unwrap_or_default(),
        );
    }
}
