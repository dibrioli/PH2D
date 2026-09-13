//! **Fase do quadro: A TRANSIÇÃO DO HOSPEDEIRO** — o hospedeiro da selecção e os gestos da tabela sinal → papel, a duração, a mola e a curva (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct UiHostTransitionIntents {
    pub(super) pending_ui_state_duration: Option<f64>,
    pub(super) pending_ui_spring_toggle: bool,
    pub(super) pending_ui_spring_knob: Option<(bool, f64)>,
    pub(super) pending_ui_easing: Option<crate::vec_ui_state_edit::EasingPick>,
    pub(super) pending_ui_signal_edit: Option<crate::vec_ui_state_edit::SignalEdit>,
    pub(super) pending_ui_signal_name: Option<(usize, String)>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_ui_host_transition(&mut self, intents: UiHostTransitionIntents) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            ui_states,
            sim,
            vec_scene,
            ..
        } = FrameGfx::of(gfx);
        let UiHostTransitionIntents {
            pending_ui_state_duration,
            pending_ui_spring_toggle,
            pending_ui_spring_knob,
            pending_ui_easing,
            pending_ui_signal_edit,
            pending_ui_signal_name,
        } = intents;
        // ⭐ **Os gestos da TABELA SINAL → PAPEL.** Eles correm DEPOIS do consumidor acima
        // e é indiferente — a tabela lida por ele é a deste frame, e uma ligação criada agora
        // responde ao próximo sinal. O que NÃO seria indiferente é o inverso do consumidor
        // com o `dispatch`, e essa ordem está fixada mais abaixo.
        // ⭐ **O HOSPEDEIRO DO QUADRO, calculado UMA vez** (auditoria de 2026-08-23).
        //
        // ⚠️ Cada gesto desta seção respondia por si a *"quem é o hospedeiro?"*, com um
        // `if let [host] = selected_paths()` próprio — **cinco portas** para o mesmo fato, e
        // nenhuma delas era a que o `publish` usa para PINTAR a seção. Desde que o hospedeiro
        // passou a ser derivado da seleção, isso é uma discordância garantida: o painel
        // mostraria as poses da forma que governa a seleção e o knob escreveria noutro sítio
        // (ou em sítio nenhum). Uma pergunta, uma resposta.
        let ui_host = crate::vec_ui_state_edit::host_of_selection(
            sim,
            vec_scene,
            &self.vec.entities,
            self.vec.pen.selected_paths(),
        );
        if let Some(edit) = pending_ui_signal_edit {
            crate::vec_ui_state_edit::apply_signal_edit(
                sim,
                vec_scene,
                &self.vec.entities,
                ui_states,
                self.vec.pen.selected_paths(),
                edit,
            );
        }
        if let Some((row, name)) = pending_ui_signal_name
            && let Some(host) = ui_host
        {
            ui_states.set_binding_name(host, row, name);
        }
        if let Some(secs) = pending_ui_state_duration
            && let Some(host) = ui_host
        {
            ui_states.set_duration(host, secs);
        }
        // **A MOLA** (W7m) — a mesma guarda de hospedeiro único da duração e da curva.
        //
        // ⚠️ Ligar SEMEIA com o default; desligar guarda `None` e **não apaga** a duração nem
        // a curva, que o artista recupera com o mesmo clique.
        if pending_ui_spring_toggle && let Some(host) = ui_host {
            let next = ui_states
                .spring(host)
                .is_none()
                .then(ph2d_ui_state::Spring::default);
            ui_states.set_spring(host, next);
        }
        if let Some((stiff, v)) = pending_ui_spring_knob
            && let Some(host) = ui_host
        {
            // ⚠️ Arrastar um knob de mola num hospedeiro que ainda não a tem **liga-a**: o
            // slider só é pintado no modo mola, então este caminho só corre com ela ligada —
            // e o `unwrap_or_default` é o que impede um `None` de engolir o gesto em silêncio
            // se um dia ele passar a ser alcançável.
            let mut sp = ui_states.spring(host).unwrap_or_default();
            if stiff {
                sp.stiffness = v;
            } else {
                sp.damping = v;
            }
            ui_states.set_spring(host, Some(sp));
        }
        // **A CURVA** (W7) — a outra metade do *como este hospedeiro transita*, e por isso
        // honrada ao lado da duracao e pela mesma guarda de hospedeiro unico.
        //
        // O pick e' uma METADE (familia ou direcao), entao ele e' aplicado sobre a curva que o
        // documento tem: `set_easing` recebe sempre um `Easing` completo, e quem o compoe e' a
        // porta unica `easing_with`.
        if let Some(pick) = pending_ui_easing
            && let Some(host) = ui_host
        {
            let cur = ui_states.timing(host).1;
            ui_states.set_easing(host, crate::vec_ui_state_edit::easing_with(cur, pick));
        }
    }
}
