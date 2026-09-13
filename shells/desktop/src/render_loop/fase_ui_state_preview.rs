//! **Fase do quadro: A PRÉVIA DOS ESTADOS E MOVER COM TODOS OS ESTADOS** — o modo de prévia dos estados de UI e da máquina de morph, a máquina viva no quadro e
//! mover o widget carregando todos os estados (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct UiStatePreviewIntents {
    pub(super) pending_ui_preview_toggle: bool,
    pub(super) pending_ui_move_all_toggle: bool,
    pub(super) pending_morph_preview_toggle: bool,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_ui_state_preview(
        &mut self,
        intents: UiStatePreviewIntents,
        report: ph2d_core::FixedStepReport,
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
            ..
        } = FrameGfx::of(gfx);
        let UiStatePreviewIntents {
            pending_ui_preview_toggle,
            pending_ui_move_all_toggle,
            pending_morph_preview_toggle,
        } = intents;
        // **O MODO DE PREVIEW** (W7r) — o interruptor e a saída por Esc, na MESMA porta: um
        // `leave` escrito num segundo sítio seria a segunda resposta a *"como se devolve o
        // mundo?"*, e a que esquecesse um plano deixaria a cena numa pose que ninguém autorou.
        //
        // ⚠️ **`preview_frame` é lido ANTES de qualquer coisa acontecer**, e ele é o que
        // suprime o undo no quadro da SAÍDA: `leave` escreve poses de volta no mundo, e um
        // diff tirado depois disso registraria *"o artista mexeu na cena"* por ele ter
        // olhado. Ligar não precisa (entrar só captura), mas custa uma disjunção e cobre o
        // caso de alguém pôr uma escrita no `enter` um dia.
        let preview_frame = self.ui_preview.is_on();
        if pending_ui_preview_toggle {
            // ⭐ **TOGGLE** (D1) — um interruptor que muda de estado. ⚠️ Aqui e não no ramo do
            // `ui_preview_leave`: aquele também dispara pelo **Esc** e pelo fim de um modo, e
            // um som ali anunciaria o que o app decidiu em vez de confirmar o que a mão fez.
            self.pending_ui_sound = Some(crate::ui_sound::UiSound::Toggle);
        }
        // ⭐⭐ **A PRÉ-VISUALIZAÇÃO da máquina de Morph** (plano 32 W9) — o modo em que o
        // teclado é da máquina. ⚠️ **Não há `enter`/`leave` a capturar mundo**, ao contrário do
        // irmão acima: aqui a restauração já é do ledger (`preview_drive`), que repõe o valor
        // AUTORADO na captura. Ligar e desligar é só o interruptor.
        if pending_morph_preview_toggle {
            self.morph_preview = !self.morph_preview;
            // ⭐ **TOGGLE** (D1), pela mesma razão do irmão: aqui e não no ramo do `leave`, que
            // também dispara pelo Esc — um som ali anunciaria o que o app decidiu.
            self.pending_ui_sound = Some(crate::ui_sound::UiSound::Toggle);
        }
        if std::mem::take(&mut self.morph_preview_leave) {
            self.morph_preview = false;
        }
        if pending_ui_preview_toggle || std::mem::take(&mut self.ui_preview_leave) {
            if self.ui_preview.is_on() {
                self.ui_preview
                    .leave(ui_machines, sim, vec_scene, &self.vec.entities);
            } else if pending_ui_preview_toggle {
                self.ui_preview
                    .enter(ui_machines, ui_states, sim, vec_scene, &self.vec.entities);
            }
        }
        // ⚠️ O relógio é o do FRAME — os ticks que o `FixedStep` de facto entregou, e não um
        // relógio próprio. É a lição W4.T7 do Motion, onde o `MotionTransport` morreu: dois
        // relógios divergem, e o modo de falha é a UI a andar noutra velocidade que a cena.
        #[allow(clippy::cast_precision_loss)]
        let ui_state_dt = report.ticks as f64 * self.fixed_step.fixed_dt();
        // ⚠️ A supressão do undo cobre a preview INTEIRA, e não só as máquinas em voo: uma
        // máquina PARADA num hover deixa o mundo fora da pose autorada, e o diff registraria
        // esse mundo como trabalho do artista. Os três termos são *estava ligada · está
        // ligada · alguma máquina anda*, e a disjunção é o que fecha o quadro da saída.
        self.ui_state_live = preview_frame
            | self.ui_preview.is_on()
            | crate::render_loop::ui_state_bridge::dispatch(
                ui_machines,
                ui_states,
                sim,
                vec_scene,
                &self.vec.entities,
                ui_state_dt,
                &mut self.ui_cooked,
            );
        if pending_ui_move_all_toggle {
            self.ui_states_move_all = !self.ui_states_move_all;
        }
        // **MOVER O WIDGET CARREGANDO TODOS OS ESTADOS** (Enio, 2026-08-07).
        //
        // Um estado grava a sub-árvore, e o hospedeiro está nela sempre que ele próprio é uma
        // forma desenhada ⇒ a translação dele fica congelada em cada estado, e relocar o
        // widget faz o Show seguinte **devolvê-lo ao lugar antigo**. Marcado, o deslocamento
        // do hospedeiro é aplicado à pose dele em TODOS os estados.
        //
        // ⚠️ **O ancoradouro é re-escrito em TODO quadro, aplique-se ou não** — e é isso que
        // impede a realimentação: um Show deixa a forma noutro lugar, e sem re-ancorar o
        // quadro seguinte leria essa diferença como um arrasto do artista e deslocaria todos
        // os estados por uma distância que ninguém percorreu. É a lição do `expr_owed` e do
        // `skip` do autokey, aqui.
        //
        // ⚠️ E o gesto é detectado pelo `Transform`, não pelo gizmo: assim o arrasto, a seta
        // do teclado, o campo numérico e o align entram todos pela mesma porta.
        {
            let host = match self.vec.pen.selected_paths() {
                [only] => Some(*only),
                _ => None,
            };
            let live = host.and_then(|h| {
                self.vec
                    .entities
                    .get(&h)
                    .map(|&bits| ph2d_ecs::Entity::from_bits(bits))
                    .and_then(|e| sim.world().get::<ph2d_ecs::Transform>(e))
                    .map(|t| [t.translation.x, t.translation.y])
            });
            if let (Some(h), Some(now)) = (host, live) {
                if self.ui_states_move_all
                    && !self.ui_state_live
                    && let Some((prev_h, prev)) = self.ui_states_anchor
                    && prev_h == h
                {
                    let d = [f64::from(now[0] - prev[0]), f64::from(now[1] - prev[1])];
                    crate::vec_ui_state_edit::shift_host_in_all_states(ui_states, h, d);
                }
                self.ui_states_anchor = Some((h, now));
            } else {
                self.ui_states_anchor = None;
            }
        }
    }
}
