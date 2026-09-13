//! **Fase do quadro: O LATCH DA FORMA ARMADA E OS CAMPOS DE FORMA** — o «armado para desenhar» que a selecção desarma, o alvo dos campos de forma do painel e a semente
//! one-shot quando o par (alvo, tipo) muda (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_shape_fields(&mut self, vec_px_to_world: f64) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            sim,
            tools,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        // Live Shapes: o ALVO dos campos de forma do painel é a forma paramétrica
        // SELECIONADA — os campos DELA aparecem (mesmo na ferramenta Select) e a
        // editam. Sem alvo, valem os da forma ativa do catálogo (default do traço).
        #[cfg(feature = "panel-vector")]
        {
            let sel: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
            // ⭐⭐ **O LATCH de «armado para desenhar».** A tool publica o clique no catálogo;
            // a selecção que MUDA o apaga — e desenhar selecciona a forma nova, então o ciclo
            // Live Shape volta sozinho no gesto seguinte. Sem isto, *"armei o Polígono"* e
            // *"acabei de desenhar uma estrela"* leem-se iguais (os dois são `DrawMode::Shape`
            // com uma forma viva na selecção) e um dos dois fica errado, seja qual for a regra.
            // ⚠️ **O alvo CRU** (`panel_shape_target`, e não a porta): o latch tem de ver a
            // selecção real para saber quando se apagar. Ler a porta aqui seria um laço — ela
            // devolve `None` justamente porque o latch está aceso, e ele nunca mais cairia.
            let alvo_vivo =
                crate::vec_shape_params::panel_shape_target(sim, &self.vec.entities, &sel)
                    .map(|(id, ..)| id);
            // ⚠️ **O DESARME primeiro, o ARME depois.** Um clique é um EVENTO drenado; a
            // mudança de alvo é um NÍVEL comparado com o frame anterior. Se algum dia os dois
            // caírem no mesmo frame, quem tem de ganhar é o gesto que se sabe ter acontecido.
            if alvo_vivo != self.vec.shape_armed_target {
                self.vec.shape_armed_target = alvo_vivo;
                self.vec.shape_armed = false;
            }
            if vector_bridge::take_shape_armed(tools) {
                self.vec.shape_armed = true;
            }
            let target = crate::vec_shape_params::shape_field_target(
                sim,
                &self.vec.entities,
                &sel,
                self.vec.draw_config.mode,
                self.vec.shape_armed,
            );
            ph2d_panel_vector::set_current_shape_focus(target.as_ref().map(|(_, _, k, _)| *k));
            // Semente ONE-SHOT: só quando o alvo MUDA (senão brigaria com o arrasto).
            // Além dos campos, a TOOL adota os params — assim painel, tool e objeto
            // concordam, e a próxima forma desenhada herda (modelo Figma).
            // ⚠️ O gatilho é o PAR `(alvo, tipo)`. Só o alvo deixava *"nada selecionado,
            // catálogo em Star"* e *"…em Polygon"* comparando iguais (`None == None`), e
            // como os slots do store são por ÍNDICE — compartilhados por TODAS as formas —
            // os campos ficavam com os números da forma anterior.
            let catalog = vector_bridge::shape_catalog(tools);
            let focus = crate::vec_shape_params::shape_seed_focus(
                target.as_ref().map(|(id, _, k, _)| (*id, *k)),
                catalog.map(|(k, _)| k).unwrap_or_default(),
            );
            if Some(focus) != self.vec.shape_last_focus {
                self.vec.shape_last_focus = Some(focus);
                // A conversão para UI é UMA, aqui: os dois consumidores (o store que o
                // painel pinta e a tool que adota) leem o MESMO array. Fazê-la dentro do
                // `seed_shape_fields` a duplicaria.
                let ui = match target.as_ref() {
                    // Alvo vivo: os parâmetros DELE, que estão em mundo.
                    Some((_, _, kind, world)) => {
                        crate::vec_shape_params::ui_values_of(*kind, world, vec_px_to_world)
                    }
                    // Sem alvo: o que a tool guarda para aquele tipo — já em UI, e é o
                    // default do próximo desenho.
                    None => catalog.map(|(_, v)| v).unwrap_or_default(),
                };
                crate::vec_shape_params::seed_shape_fields(&mut hero.store, focus.1, &ui);
                vector_bridge::adopt_shape_values(tools, focus.1, ui);
            }
        }
    }
}
