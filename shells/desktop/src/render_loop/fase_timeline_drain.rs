//! **Fase do quadro: O DRENO DA TIMELINE** — as intents do painel e do K entram no documento geral, a
//! timeline é aplicada à cena no relógio da vista (Keys, contêiner ou cena), o reset do transporte, o
//! toggle do motion path espelhado do objecto seleccionado, e os valores das faixas publicados DEPOIS de a
//! pose deste quadro estar escrita (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! ⚠️ Os quatro parâmetros são os que a `fase_timeline_view` publicou para ESTE quadro.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_timeline_drain(
        &mut self,
        container: Option<usize>,
        maos: &[u64],
        keys_mode: bool,
        selected_now: Option<u64>,
    ) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx { sim, .. } = FrameGfx::of(gfx);

        // General timeline (W1): drain pending panel/K intents into the app-general
        // document, then apply it to the scene. Three clocks, one chosen per view:
        // Keys drives the CLIP playhead (solo the clip); inside a container the
        // CONTAINER playhead (solo the interior); Arrange the timeline playhead (blend
        // the stack). Skipping the dragged entity. No-op while empty.
        let active_playhead = if keys_mode {
            &mut self.clip_playhead
        } else if container.is_some() {
            &mut self.container_playhead
        } else {
            &mut self.playhead
        };
        let timeline_reset = timeline_bridge::run(
            sim.world_mut(),
            &mut self.timeline,
            active_playhead,
            &mut self.timeline_intents,
            maos,
            &mut self.autokey,
            keys_mode,
            container,
            &mut self.timeline_signals,
            // ⚠️ **O terceiro membro da família pré-visualização↔documento** — o que as curvas
            // escrevem enquanto o playhead toca não é uma edição (`crate::timeline_preview`).
            &mut self.preview_drive,
        );
        if timeline_reset {
            // The document just went back to a fresh state (the last animated
            // object was deleted — Enio 2026-07-22, "a timeline precisa ser
            // resetada ao deletar o objeto"). The clocks and the panel's
            // container trail describe the OLD document, so they reset with it:
            // rewind + pause BOTH playheads (`rewind` deliberately preserves the
            // play state, so the pause is explicit), and drop the trail (its
            // steps are container indices into a document that no longer has
            // containers).
            self.playhead.rewind();
            self.playhead.pause();
            self.clip_playhead.rewind();
            self.clip_playhead.pause();
            self.container_playhead.rewind();
            self.container_playhead.pause();
            self.last_timeline_container = None;
            ph2d_panel_timeline::state::reset_trail();
        }
        // **PRODUTOR 1 — a timeline PUBLICA os sinais que o play cruzou** (ADR-0143).
        //
        // Ela não escolhe consumidor, e é essa a mudança: antes daqui saía um toast escrito à
        // mão, gêmeo de outro a oitenta linhas abaixo, ao lado do produtor da física — dois
        // lugares decidindo o que um sinal FAZ. Agora os dois publicam na MESMA saída e quem
        // escuta se serve depois, cada um com o seu cursor (ADR-0075: o produtor não chama).
        for sig in self.timeline_signals.out.drain(..) {
            self.signals
                .publish(ph2d_runtime::Signal::from_timeline(&sig.name, sig.t));
        }
        // The playhead has now moved: a transport jump queued last frame can
        // finally ask the panel to pan to it (the snapshot below carries the
        // new time, and `paint` reads both later this frame).
        if std::mem::take(&mut self.timeline_reveal_after_apply) {
            ph2d_panel_timeline::request_reveal_playhead();
        }
        // Publish the view snapshot the docked timeline panel paints (transport
        // state; tracks/keys from E3+). Rebuilt from the ACTIVE playhead (the clip
        // clock in Keys mode, the CONTAINER clock inside one, the timeline clock in
        // Arrange), so the whole transport reflects the clock the panel is showing.
        let active_playhead = if keys_mode {
            &self.clip_playhead
        } else if container.is_some() {
            &self.container_playhead
        } else {
            &self.playhead
        };
        self.timeline_view
            .rebuild(&mut self.timeline, active_playhead, keys_mode);
        // The Motion Path toggle reflects the SELECTED object's position mode, so
        // switching objects updates it (ADR-0141). Filled here because the document
        // has no selection; fresh / no single selection → Path (the default).
        self.timeline_view.position_is_path = selected_now.is_none_or(|e| {
            matches!(
                self.timeline.doc.position_key_mode(e, true),
                ph2d_timeline::PositionKeyMode::Path
            )
        });
        // ...e QUEM é cada objeto animado, por nome, pela mesma razão: o documento aponta
        // para os objetos por `wire_id` (o hash de um `Name`), nunca pelo nome. Sem isto o
        // dope-sheet rotula toda row como `#7294`.
        crate::timeline_persist::publish_object_names(&mut self.timeline_view, sim.world());
        // ...e QUANTO cada propriedade animada vale AGORA, para a row o mostrar (Enio, 2026-09-04).
        // ⚠️ **Depois do `timeline_bridge::run`**, que é quem escreve a pose deste quadro: lido
        // antes, o número atrasaria um quadro — e este é o sítio onde o mundo já está escrito.
        timeline_bridge::publish_track_values(&mut self.timeline_view, sim.world());
        ph2d_panel_timeline::set_current_timeline(Some(self.timeline_view.clone()));
    }
}
