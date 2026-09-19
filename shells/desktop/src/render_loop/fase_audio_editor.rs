//! **As três fases-filhas do EDITOR DE ÁUDIO** — a ponte do painel `audio_editor`, por assunto: a entrada e a
//! saída (carregar, exportar, o transporte, o ML, a entrega), o rack de efeitos, e a preparação de asset (loop,
//! marcadores, variações). Chamadas pela [`super`] (`fase_audio_panels`) no sítio e pela ordem dos blocos (OBRA 3
//! da `line/render-bodies`); cada uma re-deriva o `self.audio`, que nenhuma delas troca.

use ph2d_i18n::tr;
impl crate::App {
    /// A entrada e a saída: carregar, exportar, LUFS em lote, peças e conjunto, o transporte e o loop, o ML, e a
    /// entrega com o preço das plataformas (só com a secção Delivery aberta).
    pub(super) fn fase_audio_editor_io(&mut self) {
        use ph2d_panel_audio_editor as ed;
        let Some(audio) = self.audio.as_mut() else {
            return;
        };
        if ed::take_load()
            && let Some(path) = rfd::FileDialog::new()
                .add_filter("audio", ph2d_audio_decode::decode_any::AUDIO_IMPORT_EXTS)
                .pick_file()
        {
            audio.editor_load(&path);
        }
        // One Export, driven by the Delivery section's codec: the file that
        // lands on disk is the one the panel just priced.
        if ed::take_export() && audio.editor_loaded() {
            let codec = audio.editor_codec();
            if let Some(path) = rfd::FileDialog::new()
                // ⚠️ O rótulo do filtro é o que o artista lê no diálogo NATIVO do sistema, logo ele
                //    vem da tabela; a EXTENSÃO ao lado nunca se traduz.
                .add_filter(ph2d_i18n::tr(codec.label_key()), &[codec.extension()])
                .set_file_name(format!("export.{}", codec.extension()))
                .save_file()
            {
                audio.editor_export_codec(&path);
            }
        }
        // Batch LUFS — pick a folder; normalize every audio file in it to
        // −16 LUFS, writing copies under `<folder>/normalized/`.
        if ed::take_batch_lufs()
            && let Some(dir) = rfd::FileDialog::new().pick_folder()
        {
            audio.editor_batch_lufs(&dir, -16.0); // LITERAL-PX-OK: −16 LUFS target
        }
        // Export Pieces (Delivery) — a Save dialog (name + folder), same as Export Set: the
        // pieces land as `<name>_01..NN`, exactly the naming the variation importer reads
        // back as one group. Not a folder picker (see Export Set below for why).
        if ed::take_export_pieces()
            && audio.editor_loaded()
            && let Some(path) = rfd::FileDialog::new()
                .set_file_name(audio.editor_default_stem())
                .save_file()
        {
            audio.editor_export_pieces(&path);
        }
        // Export Set (Delivery) — one file per shipping target, each conformed to that
        // target's own format first, named `<name>.<platform>.<ext>`. A **Save** dialog
        // (name + folder), NOT a folder picker: the native folder chooser makes you confirm
        // a highlighted folder and a double-click enters it, so "pick a folder" turns into
        // "keep opening folders". Save is one confirm and mirrors Export WAV.
        if ed::take_export_set()
            && audio.editor_loaded()
            && let Some(path) = rfd::FileDialog::new()
                .set_file_name(audio.editor_default_stem())
                .save_file()
        {
            audio.editor_export_set(&path);
        }
        // Cache the loop crossfade the panel asks for BEFORE Play reads it —
        // Play plays the click-free region when Loop is on and a region is set.
        audio.editor_set_pending_xfade(audio.editor_xfade_frames(ed::xfade_norm()));
        if ed::take_play_pause() {
            audio.editor_toggle_play(ed::looping());
        }
        if ed::take_stop() {
            audio.editor_stop();
        }
        if let Some(cmd) = ed::take_edit_cmd() {
            audio.editor_apply(cmd);
        }
        // Live Loop toggle — takes effect on the sounding preview immediately.
        audio.editor_set_looping(ed::looping());
        audio.editor_poll();
        // The AI Denoise (W7) runs off the UI thread — this is where its result comes
        // home, and where the bar of a just-started one is handed to the app-wide
        // queue. Both are per-frame and both are no-ops when nothing is running.
        #[cfg(feature = "audio-ml")]
        {
            audio.editor_poll_ml();
            if let Some(progress) = audio.editor_take_started_job()
                && let Some(gfx) = self.gfx.as_mut()
            {
                gfx.jobs.push(progress);
            }
        }
        audio.editor_publish_delivery();
        // **Pricing the shipping targets is EXPORT work** (ADR-0125). It costs three
        // conforms and three real encodes of the whole clip — 1549 ms on a 3-minute take —
        // so it is gated on someone actually looking at the rows, and when they are, it
        // runs on a worker. The section ships FOLDED, so the usual answer here is "no".
        //
        // Asked of the shell's own `HeroScreen`, which already owns both halves of the
        // question. Note `is_panel_visible` is not enough on its own: the panel can be open
        // with this section folded away, which is the default.
        let delivery_open = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .is_some_and(|h| {
                h.is_panel_visible("audio_editor")
                    && !h
                        .store
                        .is_collapsed(ph2d_panel_audio_editor::AEDIT_SEC_DELIVERY)
            });
        audio.editor_publish_platforms(delivery_open);
        audio.editor_publish_spectral();
        ed::set_playing(audio.editor_playing());
        ed::set_loaded(audio.editor_loaded());
        ed::set_position_secs(audio.editor_position_secs());
        ed::set_duration_secs(audio.editor_duration_secs());
        ed::set_clip_name(audio.editor_name());
        ed::set_can_undo(audio.editor_can_undo());
        ed::set_can_redo(audio.editor_can_redo());
        ed::set_has_selection(audio.editor_selection().is_some());
        ed::set_has_clipboard(audio.editor_has_clipboard());
        // How many pieces the clip is in — what dims Move, Clear Cuts and Export Pieces.
        ph2d_panel_audio_editor::tool_state::set_pieces(audio.editor_piece_count());
        // Whether a crossfade bake would do anything (needs a loop, a crossfade, and audio
        // before the loop start to fade from).
        ph2d_panel_audio_editor::loop_state::set_can_bake(audio.editor_can_bake_crossfade());
    }

    /// O rack de efeitos: a tabela de tipos, a sala (IR), os presets, a audição ao vivo, o A/B e o mono.
    pub(super) fn fase_audio_editor_rack(&mut self) {
        use ph2d_panel_audio_editor as ed;
        let Some(audio) = self.audio.as_mut() else {
            return;
        };
        // Effects rack (W3 blocks 3a/3b): the panel owns the effect CHAIN as
        // kind indices + raw 0..1 slider positions; the shell owns the real
        // DSP ranges. Publish the kind table (names + each kind's NEUTRAL
        // normals, so the panel can seed a fresh stage transparent) BEFORE
        // reading the chain — `fx_chain()` materializes its first stage from
        // exactly those defaults.
        use ph2d_app_audio::{fx_params, fx_presets};
        ed::set_fx_kind_names(&fx_params::kind_names());
        ed::set_fx_kind_defaults(&fx_params::all_default_norms());
        let (kind, norms) = ed::fx_sel_stage();
        ed::set_fx_param_views(&fx_params::views(kind, &norms));
        // Does the selected stage want a ROOM? Derived from the effect the table
        // builds, not from a new column in it: "needs an impulse response" is a fact
        // ABOUT the effect, not a knob on it. Then drain the request.
        ed::set_fx_ir(
            fx_params::needs_ir(kind),
            &ph2d_app_audio::editor::ir::readout(),
        );
        if ed::take_load_ir()
            && let Some(path) = rfd::FileDialog::new()
                // IR keeps its own list (an impulse response is normally lossless), but if
                // ogg is allowed then opus is too — both lossy, both decodable here.
                .add_filter(
                    tr("shell.fase_audio_editor.impulse_response"),
                    &["wav", "flac", "aiff", "aif", "ogg", "opus"],
                )
                .pick_file()
        {
            ph2d_app_audio::editor::ir::load(&path);
        }

        // Chain presets. Publish the factory names for the selector, then
        // drain its three one-shots: Apply loads a factory preset into the
        // chain (it auditions like any edit); Save / Load are user preset
        // FILES via a native dialog. `set_fx_chain` marks the chain dirty, so
        // the audition + Apply-to-commit flow below carries them for free.
        ed::set_preset_names(&fx_presets::factory_names());
        if ed::take_apply_preset() {
            ed::set_fx_chain(fx_presets::factory_chain(ed::preset_sel()));
        }
        if ed::take_save_preset()
            && let Some(path) = rfd::FileDialog::new()
                .add_filter(tr("shell.fase_audio_editor.ph2d_audio_preset"), &["txt"])
                .set_file_name("chain-preset.txt")
                .save_file()
        {
            let text = fx_presets::serialize_chain(&ed::fx_chain());
            if let Err(e) = std::fs::write(&path, text) {
                eprintln!("audio: preset save failed for {}: {e}", path.display());
            }
        }
        if ed::take_load_preset()
            && let Some(path) = rfd::FileDialog::new()
                .add_filter(tr("shell.fase_audio_editor.ph2d_audio_preset"), &["txt"])
                .pick_file()
        {
            match std::fs::read_to_string(&path) {
                Ok(text) => ed::set_fx_chain(fx_presets::parse_chain(&text)),
                Err(e) => {
                    eprintln!("audio: preset load failed for {}: {e}", path.display())
                }
            }
        }
        // Live audition: once the user touches the rack, render the whole
        // chain over the (pristine) clip and hot-swap it into the sounding
        // preview, so it is heard while the sliders move. Change-gated
        // inside, so this is at most one render per parameter change, and
        // everything upstream of the edited stage is cached.
        // Apply commits that exact buffer as one undo step; Cancel drops it.
        if ed::fx_dirty() {
            audio.editor_fx_update(&ed::fx_chain(), ed::fx_sel());
        }
        // Global A/B — hear/see/export the dry clip without losing the chain.
        audio.editor_fx_set_bypass(ed::fx_bypass());
        ed::set_fx_auditioning(audio.editor_fx_auditioning());

        // Force-to-mono — a NON-destructive output toggle. Flip on click (which
        // rebuilds the view + live-switches the preview); otherwise keep the
        // mono view fresh vs. edits. Publish the state so the button lights.
        if ed::take_toggle_mono() {
            audio.editor_toggle_force_mono();
        } else {
            audio.editor_refresh_mono_view();
        }
        ed::set_mono_on(audio.editor_force_mono());
    }

    /// A preparação de asset: os pontos de loop, os marcadores e os contentores de variação.
    pub(super) fn fase_audio_editor_prep(&mut self) {
        use ph2d_panel_audio_editor as ed;
        let Some(audio) = self.audio.as_mut() else {
            return;
        };
        // Loop points (W6 — asset-prep). Set (from selection, auto-snapped) /
        // Clear the region; there is no separate Audition — Loop + Play plays
        // the region (handled above). While a region loops, hot-swap a fresh
        // crossfaded buffer when the Crossfade slider or the region moves.
        // Publish the region span for the panel readout.
        if ed::take_set_loop() {
            audio.editor_set_loop_from_selection();
        }
        if ed::take_clear_loop() {
            audio.editor_clear_loop();
        }
        ed::set_loop_span(audio.editor_loop_span());

        // Markers (W6): Add a cue at the playhead / Delete the nearest; publish
        // the count for the panel readout + Del enablement.
        if ed::take_add_marker() {
            audio.editor_add_marker();
        }
        if ed::take_del_marker() {
            audio.editor_del_marker();
        }
        ed::set_marker_count(audio.editor_marker_count());

        // Variation containers (W6): a set of clips the runtime plays one of per
        // trigger. Add opens a file picker (decode + cache); Play auditions the
        // next pick (strategy + jitter) through the preview voice; Save/Load are
        // manifest files. The panel owns the selected row + jitter sliders; the
        // shell owns the set, the decoded clips and the picker. Publish the row
        // labels + strategy name + count back each frame.
        if ed::take_add_variation()
            && let Some(path) = rfd::FileDialog::new()
                .add_filter("audio", ph2d_audio_decode::decode_any::AUDIO_IMPORT_EXTS)
                .pick_file()
        {
            audio.editor_add_variation(&path);
        }
        // Import by convention: a folder of `name_01..NN` → the whole set,
        // natural-sorted.
        if ed::take_add_variation_folder()
            && let Some(dir) = rfd::FileDialog::new().pick_folder()
        {
            audio.editor_add_variation_folder(&dir);
        }
        if ed::take_toggle_enabled() {
            audio.editor_toggle_variation_enabled();
        }
        ed::set_selected_enabled(audio.editor_variation_enabled());
        if ed::take_remove_variation() {
            audio.editor_remove_variation(ed::variation_sel());
        }
        let strategy_steps = ed::take_strategy_step();
        if strategy_steps != 0 {
            audio.editor_cycle_variation_strategy(strategy_steps);
        }
        let weight_steps = ed::take_weight_step();
        if weight_steps != 0 {
            audio.editor_bump_variation_weight(ed::variation_sel(), weight_steps);
        }
        audio.editor_set_variation_jitter(ed::pitch_jitter_norm(), ed::gain_jitter_norm());
        if ed::take_play_variation() {
            audio.editor_play_variation();
        }
        if ed::take_save_variation_set()
            && let Some(path) = rfd::FileDialog::new()
                .add_filter(tr("shell.fase_audio_editor.ph2d_variation_set"), &["txt"])
                .set_file_name("variations.txt")
                .save_file()
        {
            audio.editor_save_variation_set(&path);
        }
        if ed::take_load_variation_set()
            && let Some(path) = rfd::FileDialog::new()
                .add_filter(tr("shell.fase_audio_editor.ph2d_variation_set"), &["txt"])
                .pick_file()
        {
            audio.editor_load_variation_set(&path);
        }
        ed::set_variation_names(&audio.editor_variation_names());
        ed::set_strategy_name(audio.editor_variation_strategy());
    }
}
