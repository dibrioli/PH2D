//! **Fase do quadro: OS CONTROLOS AUTORADOS E OS ESTADOS DE UI** — a posição dos controlos autorados nas duas direcções, a edição de widget, os estados de
//! UI e o sinal que move a cena (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct AuthoredControlsAndUiStatesIntents {
    pub(super) pending_widget_edit: Option<crate::vec_widget_edit::WidgetEdit>,
    pub(super) pending_ui_state: Option<crate::vec_ui_state_edit::UiStateEdit>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_authored_controls_and_ui_states(
        &mut self,
        intents: AuthoredControlsAndUiStatesIntents,
    ) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            ui_states,
            ui_machines,
            sim,
            vec_scene,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        let AuthoredControlsAndUiStatesIntents {
            pending_widget_edit,
            pending_ui_state,
        } = intents;
        // **A POSIÇÃO dos controles autorados** (W8b.4): o store e o mundo de acordo, nas
        // duas direções. ⚠️ ANTES do resolvedor dos drives (mais abaixo, no passe de desenho)
        // — um valor que acabou de chegar de um load tem de mexer na arte NESTE frame, senão a
        // cena abre com a arte antiga por um quadro e pisca.
        if crate::vec_widget_value::reconcile(
            sim,
            &self.vec.entities,
            &mut hero.store,
            &mut self.vec.widget_applied,
        ) {
            self.any_input_this_frame = true;
        }
        if let Some(verb) = pending_widget_edit {
            let sel: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
            crate::vec_widget_edit::apply(sim, &self.vec.entities, &sel, verb);
            // **Bind Shape** ARMA o conta-gotas (W8b.3) — quem resolve é o clique seguinte
            // (`vec_path_pick_click`), pela guarda modal que precede o picking/gizmo. É o
            // mesmo desenho do **Swap Main**, e reusá-lo é o que dá Escape, realce de hover e
            // desistência-no-vazio sem uma linha a mais.
            if verb == crate::vec_widget_edit::WidgetEdit::Bind
                && let Some(&at) = sel.first()
            {
                self.vec.path_pick = Some(crate::vec_pick::PathPick::WidgetBind(at));
            }
        }
        // OS ESTADOS de UI (W7). ⚠️ O **Show** não escreve pose aqui: ele DEVOLVE o pedido, e
        // quem o honra é a máquina — uma escrita direta seria a segunda porta para *"pôr a
        // cena nesta pose"*, e a diferença entre as duas é o tween que o artista autorou.
        if let Some(verb) = pending_ui_state {
            let sel: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
            if let Some((host, role)) = crate::vec_ui_state_edit::apply(
                sim,
                vec_scene,
                &self.vec.entities,
                &sel,
                ui_states,
                verb,
            ) {
                crate::render_loop::ui_state_bridge::request(ui_machines, ui_states, host, role);
            }
        }
        // ⭐ **O SINAL MOVE A CENA** — o consumidor da tabela de ligação (item 4 do estudo dos
        // contêineres). A saída é a MESMA do R0: a timeline, a física e um controle autorado
        // publicam nomes, e quem escuta casa numa string sem perguntar a origem (ADR-0143).
        //
        // ⚠️ **O cursor anda SEMPRE e a ação só corre na PREVIEW**, e a assimetria não é
        // gosto — é a lei que a própria preview escreveu, aplicada a um produtor novo:
        //
        // - fora dela **não há restauração**, então um sinal que chegasse enquanto o artista
        //   desenha **moveria o desenho dele** e ficaria assim;
        // - fora dela **o undo regista**, e um sinal de física a 60 Hz seria um passo de undo
        //   por quadro.
        //
        // ⚠️ **E isto NÃO contradiz o botão Show**, que escreve o mundo fora da preview: a
        // diferença é *quem pediu*. Uma pose que o artista pediu com um clique custa um passo
        // de undo e ele sabe porquê; uma pose que **chega sozinha** não pode cobrar nada.
        //
        // ⚠️ **Ler fora da preview é o que impede o salto de entrada** — ver o doc do
        // `signal_readers.ui`. Sem o `let _`, o `read` devolve um iterador preguiçoso e **nada
        // é consumido**: o cursor não andaria, e o gate que o prova é o da entrada limpa.
        {
            let acting = self.ui_preview.is_on();
            let moves: Vec<(ph2d_vec_scene::VecPathId, ph2d_ui_state::StateRole)> = self
                .signals
                .read(&mut self.signal_readers.ui)
                .filter(|_| acting)
                .flat_map(|sig| ui_states.targets(&sig.name).collect::<Vec<_>>())
                .collect();
            for (host, role) in moves {
                crate::render_loop::ui_state_bridge::request(ui_machines, ui_states, host, role);
            }
        }
    }
}
