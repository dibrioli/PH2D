//! **Fase do quadro: O ESTILO DO TEXTO E O PAINEL DE TEXTO** — o estilo que o texto em edição herda do painel em tempo real, o espelho do painel de texto
//! e as prévias de fonte (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_text_panel(&mut self, vec_px_to_world: f64) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx { sim, vec_scene, .. } = FrameGfx::of(gfx);
        // Texto em edição herda o Style do painel em TEMPO REAL: o bridge acabou
        // de copiar Fill/Stroke/Width/Cap/Join do painel para o Pen; se mudou,
        // regenera os glyphs da sessão com o novo Paint (antes do `sync`, para as
        // entidades reconciliarem os paths novos neste mesmo frame). Sair do modo
        // Text (inclusive pelo botão do painel) COMMITA a sessão — senão o recolor
        // de multisseleção pegaria letras não-selecionadas e o gizmo sumiria.
        crate::vec_text::sync_active_text_style(
            &mut self.vec.text_edit,
            self.vec.draw_config.mode,
            &self.vec.pen,
            vec_px_to_world,
            vec_scene,
        );
        // Publica a string da sessão ativa para o painel exibir (read-only na
        // A2). `None` quando não há sessão de texto (mostra o hint).
        // As configs de TEXTO do painel agem sobre um ALVO: a sessão viva; sem ela, o
        // objeto de TEXTO selecionado — então a seção Text aparece e edita também na
        // ferramenta Select, enquanto o texto for texto (não-curva).
        #[cfg(feature = "panel-vector")]
        {
            let in_text_mode = self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Text;
            let sel: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
            let target = crate::vec_text::panel_text_target(
                sim,
                &self.vec.entities,
                &sel,
                self.vec.text_edit.as_ref(),
            );
            let visible = in_text_mode || target.is_some();
            ph2d_panel_vector::set_current_text_visible(visible);
            ph2d_panel_vector::set_current_text(target.as_ref().map(|t| t.text.clone()));
            // Família / alinhamento: do alvo; sem alvo (modo Text sem sessão), os
            // defaults correntes da shell (o que a próxima sessão vai usar).
            let family = target
                .as_ref()
                .map_or_else(|| self.vec.text.family.clone(), |t| t.family.clone());
            ph2d_panel_vector::set_current_text_font(
                visible.then(|| crate::vec_font::display_name(family.as_deref())),
            );
            ph2d_panel_vector::set_current_text_align(
                visible.then(|| target.as_ref().map_or(self.vec.text.align, |t| t.align)),
            );
            // ⚠️ A fileira Width lê o ALVO quando há um selecionado, e o default da shell
            // quando não há — a mesma regra do Align logo acima. Sem isto o painel mostraria
            // "Auto" sobre um texto que reflui, e o 1º clique em Fixed não mudaria nada.
            ph2d_panel_vector::set_current_text_wrap(
                target.as_ref().map_or(self.vec.text.wrap, |t| t.wrap_width),
            );
            // Semente dos sliders: só quando o ALVO muda (senão brigaria com o drag).
            let target_id = target.as_ref().map(|t| t.id);
            if target_id != self.vec.text_last_target {
                self.vec.text_last_target = target_id;
                ph2d_panel_vector::set_current_text_seed(target.as_ref().map(|t| t.sliders));
            }
            // ⛔ **E o FACTO que a fileira Weight precisava, e que ninguém publicava:** *esta
            // fonte expõe `wght`?* Sem `fvar` o `skrifa` ignora a localização de eixo, então
            // numa fonte ESTÁTICA aquele slider era pintado e **inerte**.
            //
            // ⚠️ **Ele NÃO é derivável dos `slots` abaixo**, e a tentação é exactamente o que
            // estava refutado: aquela lista é *"os eixos ALÉM do peso"*, então uma fonte
            // variável **só de peso** (a `Cantarell-VF` desta máquina, `fvar = ['wght']`)
            // publica-a vazia e ainda assim tem um Weight vivo. Duas perguntas, duas
            // publicações.
            //
            // ⚠️ Calculado **dentro do `visible`**, e junto dos eixos, porque
            // `has_weight_axis` resolve a família — e resolver uma família do sistema constrói
            // o catálogo do fontique (50–200 ms). Fora do modo Text isso seria pago por quadro
            // para responder a uma pergunta que a secção escondida não faz.
            ph2d_panel_vector::set_current_text_has_weight(
                visible && crate::vec_font::has_weight_axis(family.as_deref()),
            );
            // Eixos de variação da fonte do alvo (nome + range + valor).
            let slots = if visible {
                let descs = crate::vec_font::variation_axes(family.as_deref());
                let values: Vec<f32> = target.as_ref().map_or_else(
                    || self.vec.text.extra_axes.iter().map(|(_, v)| *v).collect(),
                    |t| t.axes.iter().map(|(_, v)| *v).collect(),
                );
                descs
                    .iter()
                    .zip(values)
                    .map(|(d, v)| ph2d_panel_vector::TextAxisSlot {
                        name: d.name.clone(),
                        min: f64::from(d.min),
                        max: f64::from(d.max),
                        value: f64::from(v),
                    })
                    .collect()
            } else {
                Vec::new()
            };
            ph2d_panel_vector::set_current_text_axes(slots);
        }
        // Dropdown de fonte: constrói as previews (nome de cada família na fonte
        // dela) SÓ quando o painel pede — i.e. na 1ª abertura do dropdown. Assim o
        // scan+parse das fontes do sistema é pago no open, nunca ao entrar no Text.
        #[cfg(feature = "panel-vector")]
        if self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Text
            && ph2d_panel_vector::take_want_font_previews()
        {
            ph2d_panel_vector::set_current_text_font_previews(
                crate::vec_font_preview::build_previews(),
            );
        }
    }
}
