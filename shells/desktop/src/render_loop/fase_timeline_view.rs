//! **Fase do quadro: A VISTA DA TIMELINE** — as animações das sprites amostradas na cabeça de leitura, as
//! intents que o painel e o menu de preset estacionaram, a tangente escolhida numa âncora do motion path,
//! e os espelhos que o painel publica e o transporte lê: a aba Keys, a lista de contêineres, o caminho
//! de edição e o contêiner aberto (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! ⚠️ Tudo isto é lido ANTES do dreno das intents, e é por isso que corre antes dele; os quatro locais
//! que ele e o relógio dos contêineres leem voltam pelo contexto [`TimelineView`].

use super::*;

/// **O CONTEXTO que a vista da timeline entrega ao resto do quadro** — os locais que ATRAVESSAM a
/// fronteira da fase, com os MESMOS nomes: o relógio dos contêineres e o dreno das intents leem-nos.
pub(super) struct TimelineView {
    /// O contêiner cujas faixas o transporte está a EDITAR neste quadro (só dentro de um).
    pub(super) container: Option<usize>,
    /// O que a MÃO segura neste quadro — o bridge não o aplica, para o documento não brigar com ela.
    /// São DUAS mãos: o gizmo de sprite e a ferramenta Bone ([`timeline_bridge::maos_do_quadro`]).
    pub(super) maos: Vec<u64>,
    /// A aba Keys do painel (o relógio próprio do clip solado).
    pub(super) keys_mode: bool,
    /// O objecto seleccionado agora (a borda que leva a timeline à aba Keys).
    pub(super) selected_now: Option<u64>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo. O `None` é inalcançável (os guardas correram na `fase_chrome_clock`).
    pub(super) fn fase_timeline_view(&mut self) -> Option<TimelineView> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            sim, hero_screen, ..
        } = FrameGfx::of(gfx);

        // General timeline (M0): sample every animated sprite's Clip at the
        // engine Playhead and write its Transform, BEFORE propagation/extract
        // read it. A no-op when nothing carries a SpriteAnimation; drives any
        // sprite carrying one (programmatic binds) in the real scene.
        ph2d_timeline::apply_sprite_animations(sim.world_mut(), self.playhead.time());
        // W2.E5b — fold the dope-sheet edits the panel raised from its surface
        // gestures last frame (key select / move / clear) into the intent
        // queue the bridge drains below (same channel as transport/K intents).
        self.timeline_intents
            .extend(ph2d_panel_timeline::drain_intents());
        // W3.E4 — the segment preset the user picked from a key's right-click
        // menu. editor-core parked an opaque `(item, mode)` on the hero (it
        // knows no easings); resolve it against the document here, upstream of
        // the bridge below, so the curve redraws on THIS frame.
        if let Some(pick) = hero_screen
            .as_mut()
            .and_then(|h| h.pending_timeline_interp.take())
        {
            let picked = ph2d_panel_timeline::presets::intents_for_pick(&self.timeline, pick);
            self.timeline_intents.extend(picked);
        }
        // The on-canvas motion-path anchor handle-type pick (ADR-0141): editor-core parked
        // `(target bits, anchor index, kind u8)` on the hero when the artist chose Corner /
        // Smooth / Symmetric from the anchor's right-click menu. Convert that anchor's
        // tangents here — upstream of the bridge, so the trajectory redraws THIS frame — in
        // its own undo step (the sibling of the drag's `history.begin`/`commit_if_changed`).
        if let Some((bits, i, kind)) = hero_screen
            .as_mut()
            .and_then(|h| h.pending_motion_path_handle.take())
        {
            let target = ph2d_timeline::AnimTarget::new(bits);
            let tk = match kind {
                0 => ph2d_timeline::TangentKind::Corner,
                2 => ph2d_timeline::TangentKind::Symmetric,
                _ => ph2d_timeline::TangentKind::Smooth,
            };
            self.timeline.history.begin(&self.timeline.doc);
            self.timeline
                .doc
                .set_path_tangent_kind(target, i as usize, tk);
            self.timeline.history.commit_if_changed(&self.timeline.doc);
        }
        // K (capture-the-pose): a one-shot AddKey on every bound track of the
        // selected sprite (its own undo step). AutoKey — the pose-following
        // that keys ANY UI edit — is a single pass AFTER all the frame's
        // Transform writes (`autokey_pass::run`, below the EditorAction drain),
        // so it observes the settled pose and cannot fight the apply.
        //
        // `maos` is still needed here: `timeline_bridge::run` skips what the hand holds
        // in the apply so the document never fights a live manipulation — the gizmo drag
        // AND the bone the Bone tool is posing (`timeline_bridge::maos_do_quadro`).
        let maos = timeline_bridge::maos_do_quadro(hero_screen.as_ref(), &self.skeleton, sim);
        // The panel's Keys tab drives a soloed clip on its OWN clock (the AE precomp
        // model). `keys_mode` is the panel's last-painted tab; it picks which
        // playhead the transport moves, whether the scene solos, and how K authors.
        // **Selecionar um objeto NOVO leva a timeline à aba Keys** (Enio, 2026-07-22):
        // selecionar é dizer "quero trabalhar NESTE", e as keys dele são onde esse
        // trabalho acontece. A decisão de borda é pura e testada
        // (`selection_jumps_to_keys`); o pedido viaja pelo canal do `F` e é consumido
        // (ou, painel oculto, descartado) pelo paint deste MESMO frame.
        let selected_now = hero_screen
            .as_ref()
            .and_then(|h| h.gizmo.iter_selected().next());
        if timeline_bridge::selection_jumps_to_keys(self.timeline_last_selected, selected_now) {
            ph2d_panel_timeline::state::request_keys_tab();
        }
        self.timeline_last_selected = selected_now;
        let keys_mode = ph2d_panel_timeline::state::keys_mode();
        // **The Containers LIST has no playback mode** (Enio, 2026-07-22) — the
        // panel publishes which view it painted (false while hidden), the shell
        // stamps it here, and every enforcement point (play-button refusal in
        // `intent_for_transport`, the spacebar arm, the bridge's pause backstop)
        // reads the ONE stamped field instead of re-deriving the view.
        self.timeline.containers_list = ph2d_panel_timeline::state::containers_list();
        // The active clip's loop is per-VIEW now (independent Keys/Arrange loops). The
        // shell owns both clocks, so it (a) stamps the mode `apply_intent` reads to pick
        // which loop an edit or a clip-switch targets, and (b) on a TAB switch — which is
        // NOT an intent — hands the now-active clock its own view's loop, so playback
        // wraps over the right range without waiting for the next edit.
        self.timeline.keys_mode = keys_mode;
        // **Where a stack edit lands** (ADR-0133 §5) — the same mirror, one line later: the
        // panel knows which container the animator entered, and the intents about to drain
        // have to land there. Read BEFORE the drain, exactly like `keys_mode`.
        //
        // A CHANGE in the path is the artist walking in or out of a container, and the
        // transport follows them (`timeline_bridge::on_nav_change`): in, the loop brackets
        // the entered instance; out, the document's own loop is re-installed. Edge-triggered
        // — the mirror of the `keys_mode` tab-switch sync right below.
        let edit_path = ph2d_panel_timeline::state::edit_path();
        self.timeline.edit_path = edit_path;
        // **Which container the transport is EDITING this frame** (Enio, 2026-07-22) —
        // `Some(c)` only while inside a container's lanes (not Keys, not the scene root),
        // stamped for the bridge, the autokey pass and the K flow to read. The active
        // playhead becomes the CONTAINER clock there (chosen just below), so playback,
        // the ruler and the loop are the container's, not the Arrange's.
        let container: Option<usize> = (!keys_mode)
            .then(|| self.timeline.edit_path.last().map(|s| s.container))
            .flatten();
        self.timeline.container_open = container;
        Some(TimelineView {
            container,
            maos,
            keys_mode,
            selected_now,
        })
    }
}
