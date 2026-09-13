//! **Fase do quadro: O PASSO DA FÍSICA** — o mundo rapier anda na cabeça de leitura (é ele que PRODUZ os
//! contatos), o flash do estouro envelhece, o readout do player sai a cada meio segundo DEPOIS do dispatch,
//! e uma junta que cedeu anuncia-se com a carga a que partiu (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! ⚠️ **Antes de a física publicar no outbox**, que é a fase seguinte: publicar antes deste passo entregaria
//! os contatos do quadro ANTERIOR. O `player_input` é o da `fase_pointer_subjects` deste quadro (`Copy`).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_physics_step(&mut self, player_input: ph2d_physics_ecs::PlayerInput) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            sim,
            toasts,
            physics,
            ..
        } = FrameGfx::of(gfx);

        // Global rigid physics (ADR-0131 W1): step the rapier world at the
        // Playhead tick and read poses back into Transform, BEFORE
        // sim_extract so bodies render the same frame. Runtime-truth: play
        // = N sequential steps + readback; paused = settle to the authored
        // pose (read-only on Transform → no spurious undo step when idle).
        // ⚠️ ONE transport, two consumers — the curves and the rapier world.
        // Which of them the clock reaches is the artist's call, armed on the
        // transport bar and OFF by default (`TimelineFlags::simulate_physics`).
        let simulate_physics = self.timeline.flags.simulate_physics;
        ph2d_app_physics::bridge::dispatch::dispatch(
            physics,
            sim,
            &self.playhead,
            self.fixed_step.fixed_dt(),
            &mut self.timeline.doc,
            simulate_physics,
            // O dedo do jogador — **resolvido do `InputMap` do projecto** (plano 30 W5), e
            // entregue INTEIRO por uma porta só. Ele é calculado no topo do quadro, antes de
            // qualquer empréstimo de `self`.
            player_input,
            &mut self.player_tape,
            // ⚠️ **A pose que o solver escreve é pré-visualização** — o ledger que a separa do
            // documento (a folha `ph2d-preview-drive`, Enio 2026-08-23: *«corrigir o CtrlZ para ambas»*).
            &mut self.preview_drive,
        );
        // O flash do estouro envelhece uma vez por frame, aqui: ao lado do
        // dispatch da física, que é a fase em que o tempo do mundo anda. Um canal
        // PRÓPRIO, porque uma explosão é um impulso e não deixa estado no mundo
        // para uma marca derivada ler (o irmão exato do `ContactFlash`).
        ph2d_app_physics::body_grab::age_blast_flash(&mut self.blast_flash);
        // **O READOUT do player, a cada meio segundo** (`W-PlayerOut` A5) — a
        // metade do smoke que torna a afinação legível sem um `println` à mão.
        //
        // ⚠️ **Depois do dispatch, e a ordem é a lei:** a vista é escrita no fim
        // do laço de tiques, então ler antes dele imprimiria o tique anterior — e
        // um readout um tique atrasado é indistinguível de um readout certo,
        // menos exactamente no instante em que o artista está a olhar.
        if let Some(bits) = self.physics.player_readout_log {
            const EVERY: u64 = 30;
            let tick = self.fixed_step.tick_count();
            if tick.is_multiple_of(EVERY) {
                let e = ph2d_ecs::Entity::from_bits(bits);
                match physics.player_view(e) {
                    Some(v) => eprintln!(
                        "[player] {:?} facing {:+.0} vel ({:.2}, {:.2})                          ar {} dash {} agarrado {}",
                        v.footing,
                        v.facing,
                        v.velocity[0],
                        v.velocity[1],
                        v.air_jumps_left,
                        u8::from(v.dash_charged),
                        u8::from(v.ledging),
                    ),
                    // ⚠️ **A ausência é o outro readout**, e ela é a linha que
                    // ensina o toggle: sem `Physics` armado a lei não corre.
                    None => eprintln!("[player] (a fisica esta' desarmada)"),
                }
            }
        }
        // A joint that gave way announces itself (W-J7). The overlay shows where
        // and that; only the event carries the load it broke at.
        for msg in ph2d_app_physics::bridge::dispatch::break_reports(physics, sim) {
            toasts.push(Toast::warning(msg));
        }
    }
}
