//! **Fase do quadro: A PELE, OS ESTADOS, O Z-INDEX E O LAYOUT DA SELECÇÃO** — o que o painel vectorial mostra da
//! selecção: a pele por-widget, os estados de Morph e de UI, o z-index global, a Resize Box e o layout (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! **Corte um nível abaixo:** o bloco do painel vectorial é UM statement de 467 linhas; esta fase é um grupo dos statements do CORPO dele, e a chamada fica dentro do bloco, no sítio do corpo.

use super::*;

/// O pedido de Resize Box que o dreno do barramento recolheu neste quadro.
pub(super) struct ResizeBoxIntents {
    pub(super) pending_resize_box: bool,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_vector_selection_states_panel(
        &mut self,
        intents: ResizeBoxIntents,
        sel: Vec<ph2d_vec_scene::VecPathId>,
    ) -> Option<Vec<ph2d_vec_scene::VecPathId>> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            ui_states,
            ui_machines,
            sim,
            vec_scene,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let hero = hero_screen.as_mut()?;
        let ResizeBoxIntents { pending_resize_box } = intents;
        // **A PELE por-widget** (plano UI/UX W6.2) — que controle do catálogo esta forma
        // veste. Publicada pela MESMA porta que o clique honra, e para qualquer forma
        // única (vestida ou não): uma seção que só existisse onde já há pele tornaria a
        // feature alcançável apenas onde ela já foi usada.
        let skin = crate::vec_widget_edit::publish(sim, &self.vec.entities, &sel);
        let skin_beyond = skin.as_ref().map_or(0, |(_, b)| *b);
        ph2d_panel_vector::state::set_widget_skin_state(skin.map(|(s, _)| s), skin_beyond);
        // **OS ESTADOS de UI** (plano UI/UX W7) — que poses esta forma tem, e qual delas a
        // cena mostra AGORA. O `live` sai da MESMA máquina que escreve o mundo: um
        // readout derivado noutro lugar diria um papel e a cena mostraria outro.
        // ⭐ **A seção MORPH STATES** (plano 32 W4/W8) — as transições da máquina e qual
        // delas a cena percorre; ou, sem máquina, quantas formas a seleção tem prontas a
        // virar um conjunto. As acções vêm do Input Map do projecto: elas são o
        // vocabulário das condições, e lê-las no painel seria uma segunda leitura.
        ph2d_panel_vector::state::set_morph_states_state(crate::vec_morph_edit::publish(
            sim,
            vec_scene,
            &self.vec.entities,
            &sel,
            self.morph_preview,
            hero.input_map
                .actions()
                .iter()
                .map(|a| a.name.clone())
                .collect(),
        ));
        ph2d_panel_vector::state::set_ui_states_state(crate::vec_ui_state_edit::publish(
            sim,
            vec_scene,
            &self.vec.entities,
            &sel,
            ui_states,
            // ⚠️ **Pelo HOSPEDEIRO, não pelo primeiro da seleção.** O readout diz *que
            // papel a cena mostra*, e a máquina está pendurada no hospedeiro — com uma
            // seleção múltipla, `sel.first()` é um operando qualquer e a busca falha em
            // silêncio: a seção mostraria as poses de um objeto e o readout o estado de
            // outro (ou de nenhum).
            crate::render_loop::ui_state_bridge::live_role(
                ui_machines,
                crate::vec_ui_state_edit::host_of_selection(
                    sim,
                    vec_scene,
                    &self.vec.entities,
                    &sel,
                ),
            ),
            self.ui_preview.is_on(),
            self.ui_states_move_all,
        ));
        // **O Z-INDEX da seleção** (Enio, 2026-08-04) — o número GLOBAL que sobrepõe a
        // ordem da hierarquia. Publicado pela MESMA porta que o campo escreve e que os
        // botões Arrange movem, para o número que o artista lê ser o que ele edita.
        ph2d_panel_vector::state::set_z_index(
            sel.first()
                .and_then(|id| {
                    ph2d_vec_entities::entities::zorder::authored_z(sim, &self.vec.entities, *id)
                })
                .map(|z| z as f32),
        );
        // **Resize Box** (plano UI/UX W3b): honrar e so' depois publicar, a mesma ordem
        // dos irmaos acima — publicar antes deixaria a caixa a mostrar o estado ANTERIOR
        // por um frame, e o artista veria o clique "nao pegar".
        if pending_resize_box {
            crate::vec_resize_box_edit::toggle_resize_box(sim, &self.vec.entities, &sel);
        }
        ph2d_panel_vector::state::set_resize_box(crate::vec_resize_box_edit::selected_resize_box(
            sim,
            &self.vec.entities,
            &sel,
        ));
        // ⚠️ **Os DEZ comprimentos do fluxo cruzam a fronteira; `columns` NÃO.** O
        // `flow_in_display` é a porta, e o `selected_flow` continua a falar mundo — o
        // nome dele descreve o que a cena TEM, e converter lá dentro o faria mentir.
        ph2d_panel_vector::state::set_layout_flow(
            crate::vec_layout_edit::selected_flow(sim, &self.vec.entities, &sel).map(|f| {
                crate::vec_layout_edit::flow_in_display(
                    f,
                    ph2d_editor_core::LengthDisplay::of(&hero.project),
                )
            }),
        );
        ph2d_panel_vector::state::set_layout_item(crate::vec_layout_edit::selected_item(
            sim,
            &self.vec.entities,
            &sel,
        ));
        Some(sel)
    }
}
