//! Per-frame render orchestration.
//!
//! Wave 3.1 stage C — `App::render_frame`'s body lifted verbatim from
//! `main.rs` into this sibling. Wave 3.2 stage A splits the lifted
//! body further into per-phase siblings, each implemented as an
//! `impl crate::App` block on a sibling file (split-impl pattern,
//! same as Wave 3.1 used for the initial lift).
//!
//! Phases (called by `run_render_frame` in order):
//!  - `present.rs` — paint + 4 GPU passes + title refresh.
//!  - (more phases land as Wave 3.2 progresses.)
//!
// ph2d-loc-cap: frame orchestrator — heavy phases already extracted to
// siblings (present/image_edit/sim_extract/snapshots/bgremoval_preview/
// hierarchy); residual is the frame skeleton + EditorAction intent drain.
// FOLLOW-UP: extract the intent-drain match to a `intents.rs` sibling to
// drop back under the cap (2026-05-21: +SetPresentMode/RealSize tipped it).

#[cfg(feature = "panel-audio-editor")]
mod audio_overlay;
mod audio_pieces;
mod audio_spectrogram;
pub(crate) mod autokey_pass;
pub(crate) mod bgremoval_preview;
mod color_equalization_bridge;
mod cooked_texture_bridge;
mod equalize_sizes_bridge;
pub(crate) mod flip_bridge;
/// O anel do cursor do pincel do FLIP (ADR-0114 W5, smoke do Enio): o Size é absoluto
/// em px de tela, então o anel é px de tela — sem conversão de câmera.
pub(crate) mod flip_cursor;
mod flip_gap_overlay;
pub(crate) mod flip_pass;
mod flip_pass_cache;
mod flip_pass_ghosts;
mod flip_selection_overlay;
mod flip_tween_overlay;
mod gizmo_prune;
mod hierarchy;
mod image_edit;
mod inspector_commits;
pub(crate) mod inspector_joint;
/// The BREAK half (W-J7): the switch, the two thresholds, and the fact that
/// neither is converted. Its own file for the shell's LOC cap.
#[cfg(test)]
mod inspector_joint_break_tests;
/// The MOTOR half of that seam (W-J6): the mode, the two numbers, and the unit
/// each carries. Its own file for the shell's LOC cap.
#[cfg(test)]
mod inspector_joint_motor_tests;
/// The PAIR half (W-J8): the Active switch, Collide Connected, the Swap, and the
/// name a new joint is born with. Its own file for the shell's LOC cap, cut on
/// the same line the section draws — *which two, and how they treat each other*.
#[cfg(test)]
mod inspector_joint_pair_tests;
/// The §11 Physics Body seam's OTHER half: the Inspector click reached the
/// ECS and the sprite actually falls (panel-side proof lives in
/// ).
#[cfg(test)]
mod inspector_joint_tests;
mod inspector_ordering;
mod inspector_physics;
mod inspector_physics_apply;
mod inspector_physics_area;
#[cfg(test)]
mod inspector_physics_gesture_tests;
mod inspector_physics_markers;
#[cfg(test)]
mod inspector_physics_tests;
mod inspector_visibility;
/// MEASUREMENT scaffold: onde as fases PANEL e CHROME do `painter_bridge::dispatch` gastam um frame.
#[cfg(test)]
mod measure_bridge_phases;
pub(crate) mod motion_bridge;
/// A trajetória do objeto selecionado no canvas (ADR-0141, Fatia 3).
pub(crate) mod motion_path_overlay;
mod padding_bridge;
/// `PH2D_PAINT_PERF` aggregation (one summary line per window, not per frame).
///
/// `pub(crate)` porque a frente L do plano 26 carimba a chegada do evento de ponteiro lá do
/// `input_dispatch` (a latência começa na ENTREGA, não no frame).
pub(crate) mod paint_perf;
pub(crate) mod painter_bridge;
/// Brush-image import helpers (Grain/Shape file pickers), split from
/// `painter_bridge` for the HR-18 file-LOC cap.
pub(crate) mod painter_bridge_assets;
/// The brush-cursor ring, split from `painter_bridge_overlays` for the HR-18 file-LOC cap.
pub(crate) mod painter_bridge_brush_ring;
/// The Curve / Free Hand editor overlay (spine + control dots + tangent handles), split from
/// `painter_bridge_overlays` for the HR-18 file-LOC cap.
pub(crate) mod painter_bridge_curve_overlay;
/// The Fill (Bucket) ColorDrop cursor swatch overlay, split from `painter_bridge_overlays` for the
/// HR-18 file-LOC cap.
pub(crate) mod painter_bridge_fill_overlay;
/// Shared Sprite-style gizmo painting for the Curve + Stencil transform gizmos (theme tokens, darker).
pub(crate) mod painter_bridge_gizmo;
/// The Line polyline editor overlay (segments + corner dots + transform gizmo + Fillet/Chamfer handles),
/// split from `painter_bridge_overlays` for the HR-18 file-LOC cap.
pub(crate) mod painter_bridge_line_overlay;
/// Multi-shape op badges (`+`/`−`/`○` type-square glyph per shape + a frame for parked shapes).
pub(crate) mod painter_bridge_op_badges;
/// On-canvas editing chrome (brush ring + Curve/Circle/Polygon/Stencil overlays), split from
/// `painter_bridge` for the HR-18 file-LOC cap.
pub(crate) mod painter_bridge_overlays;
/// Live GPU preview of a brush Shape-source sprite (when not selected), split from `painter_bridge` for
/// the HR-18 file-LOC cap.
pub(crate) mod painter_bridge_shape_preview;
/// On-canvas wetness sheen veil (Watercolor render-path), split from `painter_bridge_overlays` for the
/// HR-18 file-LOC cap.
pub(crate) mod painter_bridge_wetness;
/// Display gates, producer-handoff half (upload-plan refusals + the CPU→GPU→CPU dance on real
/// hardware) — split from the pipeline tests for the HR-18 file-LOC cap.
#[cfg(test)]
mod painter_preview_handoff_tests;
/// Ownership gates: the shell's preview buffer is INDEPENDENT of the tool's canvas, so a plain stroke
/// stays footprint-bound (the tool keeps sole ownership) — split from the pipeline tests (HR-18).
#[cfg(test)]
mod painter_preview_ownership_tests;
/// Display gates: the preview slot (what the sprite shader samples) is held byte-equal to the
/// tool's composite across a stroke's whole life — phase D of the impasto smoke.
#[cfg(test)]
mod painter_preview_pipeline_tests;
/// O que a tela mostra DEPOIS de um undo — o report de resquício do smoke de 2026-07-25.
#[cfg(test)]
mod painter_preview_undo_tests;
pub(crate) mod physics_bake;
pub(crate) mod physics_bridge;
/// The transport's Physics toggle at the seam the frame loop runs — a `hold`
/// nobody calls is unit-green and dead in the product.
#[cfg(test)]
mod physics_bridge_tests;
pub(crate) mod physics_overlay;
mod physics_overlay_annotations;
mod physics_overlay_contacts;
mod physics_overlay_joint_glyphs;
mod physics_overlay_joint_readout;
mod physics_overlay_joints;
pub(crate) mod physics_panel_bridge;
/// Render-and-look probe for the Push phase (diagnostic, `#[ignore]`d — writes lit PNGs).
#[cfg(test)]
mod push_look_probe;
pub(crate) mod record_fit;
pub(crate) mod timeline_bridge;
pub(crate) mod timeline_onion;
mod timeline_presets;
/// Render-and-look da razão da grade do fluido (diagnóstica, `#[ignore]`d).
#[cfg(test)]
mod wet_grid_look_probe;
// `pub(crate)`: `apply_layer_reparent` is called from `input_dispatch` (outside
// render_loop) to route the W3.T3.8 layer drag-reparent through the allowlisted
// bridge-queries module instead of downcasting in central dispatch.
/// The Deform Transform gizmo (whole-region bounding box), split from `painter_bridge_overlays` (Wave 2).
pub(crate) mod painter_bridge_deform_gizmo;
pub(crate) mod painter_bridge_queries;
/// The isolated selection gizmos (ellipse / polygon / freehand), split from `painter_bridge_overlays`.
pub(crate) mod painter_bridge_selection_gizmos;
/// The Selection overlay (marching ants + deselected-area hatching), split from `painter_bridge_overlays`
/// for the HR-18 file-LOC cap.
pub(crate) mod painter_bridge_selection_overlay;
pub(crate) mod painter_gpu_flatten;
pub(crate) mod painter_gpu_preview;
/// The joint-anchor point gizmo's publish rule — extracted from `snapshots` so
/// "which entity gets a point handle" is gated headless.
pub(crate) mod point_gizmo;
mod present;
mod sim_extract;
mod snapshots;
mod upscale_bridge;
// ADR-0108 cutover: the single Vector-tool bridge (style sync + recolour).
// Rendering of `AppGfx.vec_scene` stays inline below (ph2d_vec_render).
// pub(crate): `set_mode` é chamado do `vec_text` (o `T` troca o modo pela allowlist
// de downcast deste bridge).
pub(crate) mod vector_bridge;

use crate::*;

use ph2d_editor::interaction::WidgetEvent;
use ph2d_editor::paint::PaintCtx;
use ph2d_editor::zones::Rect as EditorRect;
use ph2d_editor::{Layout as EditorLayout, RequestedSpriteStrategy, Toast, paint_hero_screen};
use std::time::Instant;

thread_local! {
    /// Frame-phase profiler (shares the `PH2D_FLUID_PROFILE` env so no new flag):
    /// `-1` = unread, else cached on/off. Splits the frame into CPU-encode (raw)
    /// vs the present/acquire stall, plus the painter bridge dispatch (CPU preview)
    /// — to pin a slowdown the `[fluid]` profiler proves is OUTSIDE the fluid drive.
    static FRAME_PROF_ON: std::cell::Cell<i8> = const { std::cell::Cell::new(-1) };
    static FRAME_PROF_N: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    static FRAME_PROF_DISPATCH_US: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    /// Active tool's `on_tick` µs (the watercolor heartbeat: soak pour + live recomposite) —
    /// the perf-audit phase the original split missed (2026-07-07, "grave FPS drop" hunt).
    /// Acumulado sobre a JANELA, nunca lido de um frame só (ver abaixo).
    /// ⚠️ **A JANELA, não uma amostra.** A linha `[frame]` sai a cada 120 frames e lia o valor do
    /// frame SORTEADO — e um traço inteiro cabe entre duas impressões, então `tool-tick` e `stamps`
    /// liam **0,00 por construção** enquanto o artista pintava (smoke do Enio, 2026-07-29: quatro
    /// amostras seguidas zeradas num app onde a água estava viva). É a mesma doença que o split de
    /// fases teve com a mediana (§4.8.2): *um custo intermitente é invisível num redutor que só
    /// olha um instante.* Estes três acumulam sobre a janela e zeram a cada impressão.
    static FRAME_PROF_TICK_SUM_US: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static FRAME_PROF_TICK_MAX_US: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static FRAME_PROF_TICK_N: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    /// Idem para o carimbo de dabs (`stamps`), que é o outro inquilino intermitente do frame.
    static FRAME_PROF_STAMP_SUM_US: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static FRAME_PROF_STAMP_MAX_US: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static FRAME_PROF_STAMP_N: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    /// `paint_hero_screen` µs (panel/chrome Vello encode — includes the Paper preview).
    static FRAME_PROF_HERO_US: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// Quiet frames after the last resize event before the present mode saved by the fluid-drag override is
/// restored (~0.5s at 60Hz) — long enough that a paused-then-resumed drag doesn't thrash reconfigures.
const RESIZE_SETTLE_FRAMES: u32 = 30;

fn frame_prof_on() -> bool {
    FRAME_PROF_ON.with(|c| {
        if c.get() < 0 {
            c.set(i8::from(
                std::env::var("PH2D_FLUID_PROFILE").is_ok_and(|v| v != "0"),
            ));
        }
        c.get() > 0
    })
}

/// `PH2D_PAINT_PERF=1` diagnostic (2026-07-24, the mask-path FPS report): the WHOLE-frame wall clock,
/// handed to [`paint_perf::end_frame`] on drop so it pairs with the per-dispatch info recorded by
/// `painter_bridge`. The aggregator prints ONE summary line per window (not one per frame — that
/// drowned the terminal), and the frame-vs-dispatch split says whether a slow frame's cost is IN the
/// painter preview production or OUTSIDE it (panel, sim_extract, present).
struct PaintFrameTimer(Option<std::time::Instant>);
impl Drop for PaintFrameTimer {
    fn drop(&mut self) {
        if let Some(t0) = self.0 {
            paint_perf::end_frame(t0.elapsed().as_secs_f64() as f32 * 1e3);
        }
    }
}

// The mixer panel is UI-only (no `ph2d-audio` dep); its sub-bus strips are
// index-aligned with `BusId::SUB_BUSES` by convention. This asserts the two
// counts agree at compile time, so adding a core bus without a panel strip (or
// vice-versa) is a build error, not a silent misroute.
#[cfg(feature = "panel-audio-mixer")]
const _: () = assert!(ph2d_audio::SUB_BUS_COUNT == ph2d_panel_audio_mixer::SUB_BUS_COUNT);

impl crate::App {
    pub(super) fn run_render_frame(&mut self) {
        // PH2D_PAINT_PERF: whole-frame timer (aggregated on scope exit, paired with the dispatch info).
        let _paint_frame_timer = PaintFrameTimer(paint_perf::on().then(std::time::Instant::now));
        // Phase 2.1: drop finished-sample Arcs on the main thread (HR-3).
        // Phase 2.3c: feed the mixer panel live levels + apply its Master mute.
        if let Some(audio) = self.audio.as_mut() {
            audio.poll();
            #[cfg(feature = "panel-audio-mixer")]
            {
                // Built-in test oscillator (panel footer Play Test).
                audio.set_test_playing(ph2d_panel_audio_mixer::play_test());
                // Master strip.
                ph2d_panel_audio_mixer::set_levels(audio.levels(), audio.rms());
                ph2d_panel_audio_mixer::set_loudness(audio.momentary_lufs());
                let muted = ph2d_panel_audio_mixer::master_muted();
                let gain = ph2d_panel_audio_mixer::master_gain_target();
                audio.set_master_gain(if muted { 0.0 } else { gain });
                audio.set_master_cutoff(ph2d_panel_audio_mixer::master_cutoff_target());
                audio.set_master_highpass(ph2d_panel_audio_mixer::master_lowcut_target());
                audio.set_master_pan(ph2d_panel_audio_mixer::master_pan_target());
                audio.set_master_limiter(ph2d_panel_audio_mixer::limiter());
                audio.set_reverb(
                    ph2d_panel_audio_mixer::reverb_on(),
                    ph2d_panel_audio_mixer::reverb_size(),
                    ph2d_panel_audio_mixer::reverb_mix(),
                );
                // Master 3-band EQ: the panel publishes 0..1 slider positions
                // (0.5 = flat); map each to ±12 dB for the engine.
                const EQ_DB_RANGE: f32 = 24.0; // ±12 dB across the 0..1 EQ slider
                let eq = ph2d_panel_audio_mixer::master_eq_target();
                audio.set_master_eq(
                    (eq[0] - 0.5) * EQ_DB_RANGE,
                    (eq[1] - 0.5) * EQ_DB_RANGE,
                    (eq[2] - 0.5) * EQ_DB_RANGE,
                );
                // Master delay/echo: Time slider position is seconds (0..1 s);
                // feedback + return mix are raw 0..1. The engine clamps time.
                audio.set_delay(
                    ph2d_panel_audio_mixer::delay_on(),
                    ph2d_panel_audio_mixer::delay_time(),
                    ph2d_panel_audio_mixer::delay_feedback(),
                    ph2d_panel_audio_mixer::delay_mix(),
                );
                // Sub-bus strips — index-aligned with `BusId::SUB_BUSES` (the
                // panel's strip index i maps to sub-bus i; count guarded below).
                ph2d_panel_audio_mixer::set_sub_levels(audio.bus_levels(), audio.bus_rms());
                let sub_muted = ph2d_panel_audio_mixer::sub_muted();
                let sub_soloed = ph2d_panel_audio_mixer::sub_soloed();
                let sub_gain = ph2d_panel_audio_mixer::sub_gain_target();
                let sub_pan = ph2d_panel_audio_mixer::sub_pan_target();
                let sub_tone = ph2d_panel_audio_mixer::sub_tone_target();
                let sub_lowcut = ph2d_panel_audio_mixer::sub_lowcut_target();
                let sub_send = ph2d_panel_audio_mixer::sub_send_target();
                let sub_delay_send = ph2d_panel_audio_mixer::sub_delay_send_target();
                let sub_comp = ph2d_panel_audio_mixer::sub_comp_target();
                // Sidechain ducking: every bus drops under the selected key bus.
                let duck_key = ph2d_panel_audio_mixer::ducking_key();
                let duck = audio.update_ducking(
                    ph2d_panel_audio_mixer::ducking(),
                    ph2d_panel_audio_mixer::duck_depth(),
                    duck_key,
                );
                // Solo overrides mute: when any bus is soloed, only soloed buses
                // sound; otherwise a bus sounds unless it's muted.
                let any_solo = sub_soloed.iter().any(|&s| s);
                for i in 0..ph2d_audio::SUB_BUS_COUNT {
                    let sounds = if any_solo {
                        sub_soloed[i]
                    } else {
                        !sub_muted[i]
                    };
                    // Every bus except the key ducks under it (the key itself
                    // stays at full level so it cuts through).
                    let ducks = i != duck_key;
                    let mut g = if sounds { sub_gain[i] } else { 0.0 };
                    if ducks {
                        g *= duck;
                    }
                    audio.set_bus_gain(i, g);
                    audio.set_bus_pan(i, sub_pan[i]);
                    audio.set_bus_cutoff(i, sub_tone[i]);
                    audio.set_bus_highpass(i, sub_lowcut[i]);
                    audio.set_bus_send(i, sub_send[i]);
                    audio.set_bus_delay_send(i, sub_delay_send[i]);
                    audio.set_bus_compressor(i, sub_comp[i]);
                }
            }
            // Audio Editor bridge (docs/Audio/, W1): drain the panel's one-shot
            // transport intents → drive the preview engine, then publish the live
            // position/duration/name back for the readout (+ overlay playhead).
            #[cfg(feature = "panel-audio-editor")]
            {
                use ph2d_panel_audio_editor as ed;
                if ed::take_load()
                    && let Some(path) = rfd::FileDialog::new()
                        .add_filter("audio", crate::audio::decode_any::AUDIO_IMPORT_EXTS)
                        .pick_file()
                {
                    audio.editor_load(&path);
                }
                // One Export, driven by the Delivery section's codec: the file that
                // lands on disk is the one the panel just priced.
                if ed::take_export() && audio.editor_loaded() {
                    let codec = audio.editor_codec();
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter(codec.name(), &[codec.extension()])
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
                ph2d_panel_audio_editor::loop_state::set_can_bake(
                    audio.editor_can_bake_crossfade(),
                );
                // Effects rack (W3 blocks 3a/3b): the panel owns the effect CHAIN as
                // kind indices + raw 0..1 slider positions; the shell owns the real
                // DSP ranges. Publish the kind table (names + each kind's NEUTRAL
                // normals, so the panel can seed a fresh stage transparent) BEFORE
                // reading the chain — `fx_chain()` materializes its first stage from
                // exactly those defaults.
                use crate::audio::{fx_params, fx_presets};
                ed::set_fx_kind_names(&fx_params::kind_names());
                ed::set_fx_kind_defaults(&fx_params::all_default_norms());
                let (kind, norms) = ed::fx_sel_stage();
                ed::set_fx_param_views(&fx_params::views(kind, &norms));
                // Does the selected stage want a ROOM? Derived from the effect the table
                // builds, not from a new column in it: "needs an impulse response" is a fact
                // ABOUT the effect, not a knob on it. Then drain the request.
                ed::set_fx_ir(
                    fx_params::needs_ir(kind),
                    &crate::audio::editor::ir::readout(),
                );
                if ed::take_load_ir()
                    && let Some(path) = rfd::FileDialog::new()
                        // IR keeps its own list (an impulse response is normally lossless), but if
                        // ogg is allowed then opus is too — both lossy, both decodable here.
                        .add_filter(
                            "impulse response",
                            &["wav", "flac", "aiff", "aif", "ogg", "opus"],
                        )
                        .pick_file()
                {
                    crate::audio::editor::ir::load(&path);
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
                        .add_filter("PH2D audio preset", &["txt"])
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
                        .add_filter("PH2D audio preset", &["txt"])
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
                        .add_filter("audio", crate::audio::decode_any::AUDIO_IMPORT_EXTS)
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
                        .add_filter("PH2D variation set", &["txt"])
                        .set_file_name("variations.txt")
                        .save_file()
                {
                    audio.editor_save_variation_set(&path);
                }
                if ed::take_load_variation_set()
                    && let Some(path) = rfd::FileDialog::new()
                        .add_filter("PH2D variation set", &["txt"])
                        .pick_file()
                {
                    audio.editor_load_variation_set(&path);
                }
                ed::set_variation_names(&audio.editor_variation_names());
                ed::set_strategy_name(audio.editor_variation_strategy());
            }
        }
        // Coalesced painter Move: stamp the LATEST buffered canvas position ONCE this frame, replacing
        // the per-raw-CursorMoved whole-shape re-stamp storm that ran between frames (the FPS-drop /
        // "Raw rises" path — `HANDOFF_per_layer_color_perf_artifacts` §1.R). Done before `cpu_start` so
        // the re-stamp stays OUT of the encode window and "Raw" keeps its encode-only meaning.
        self.flush_pending_painter_move();
        // Snapshot + reset the per-frame input/stamp diagnostics for the HUD (input rate vs delivered
        // re-stamps — coalescing collapses a burst of events to one stamp here). `paint_stamp_us`
        // accumulates BOTH the coalesced flush (just now) and any incremental per-event stamps since the
        // last frame, so "paint ms" is the real per-frame painter cost (not just the flush).
        let diag_input_events = std::mem::take(&mut self.input_events_this_frame);
        let diag_paint_stamps = std::mem::take(&mut self.paint_stamps_this_frame);
        self.last_paint_stamp_us = std::mem::take(&mut self.paint_stamp_us_this_frame);
        // M14.7 polish (10.1): tag the start of CPU work for the
        // raw-fps measurement. Stopped after `queue.submit` (before
        // the present blocks on vsync) so the EWMA tracks pure
        // hardware capacity, independent of refresh rate.
        let cpu_start = Instant::now();
        // Pump gamepad events first so InputState reflects the latest
        // state by the time sim/extract run. Order: input → script
        // input snapshot → sim → extract → render.
        self.pump_gamepad();
        self.push_input_to_script();

        // M14.7 polish (7.3 fix): drain `pending_drops` atomically
        // BEFORE the render walks PresentWorld. Each path imports
        // exactly once, so a batch drop of N files always produces
        // exactly N sprites (winit's per-event timing no longer
        // matters). Clear the hover overlay here too — the gesture
        // is over the moment the first DroppedFile arrived.
        if !self.pending_drops.is_empty() {
            let paths = std::mem::take(&mut self.pending_drops);
            self.hovered_files.clear();
            if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
                hero.dragging_files = None;
            }
            self.handler.on_file_drop(&paths);
            self.handle_dropped_files(&paths);
        }

        // **Shape Builder:** abre/renova o arranjo das formas selecionadas. Roda ANTES do
        // borrow mutável do `gfx` (ele precisa ler a cena e escrever em `self`), e só
        // reconstrói quando a SELEÇÃO muda — refazê-lo por frame jogaria fora o memo do
        // arranjo e cada hover voltaria a pagar a booleana.
        self.build_smoke();
        self.stack_smoke();
        self.motion_path_smoke();
        self.harmony_smoke();
        self.timeline_onion_smoke();
        self.signal_smoke();
        self.timescale_smoke();
        self.stagger_smoke();
        self.buffer_smoke();
        self.extrap_smoke();
        self.expr_smoke();
        self.nest_smoke();
        self.physics_smoke();
        self.flip_pose_smoke();
        self.flip_edit_smoke();
        self.flip_fill_smoke();
        self.flip_colorize_smoke();
        self.flip_tween_smoke();
        self.flip_tween_pairs_smoke();
        self.flip_strip_smoke();
        self.flip_tween_phase_smoke();
        self.flip_tween_torsion_smoke();
        self.flip_tip_smoke();
        self.flip_multiplane_smoke();
        self.flip_self_overlap_smoke();
        self.flip_airbrush_smoke();
        self.flip_resample_smoke();
        self.flip_pressure_smoke();
        self.flip_selection_smoke();
        self.flip_segment_smoke();
        self.blend_smoke();
        self.motion_node_path_smoke();
        self.motion_delay_smoke();
        self.motion_fx_smoke();
        self.post_stack_smoke();
        self.value_curve_smoke();
        self.value_noise_smoke();
        self.value_mix_smoke();
        self.value_quantize_smoke();
        self.value_gain_smoke();
        self.value_step_smoke();
        self.value_normalize_smoke();
        self.value_unary_smoke();
        self.value_reduce_smoke();
        self.value_smooth_smoke();
        self.value_pattern_smoke();
        self.value_wrap_smoke();
        self.value_time_smoke();
        self.value_slope_smoke();
        self.value_median_smoke();
        self.value_percentile_smoke();
        self.value_wave_smoke();
        self.build_session_upkeep();
        // Tween v2: a sessão de correção de pares SEGUE o artista a um novo intervalo (no-op
        // se o intervalo é o mesmo, ou se a sessão está fechada).
        self.flip_tween_pairs_upkeep();
        // ADR-0114 C2: o Apply/Clear do Colorize marcado no frame anterior (o drain de painel
        // roda com `self.gfx` preso; aqui, antes de bindar `gfx`, `self` está livre).
        if std::mem::take(&mut self.pending_flip_colorize_apply) {
            self.flip_colorize_apply();
            // ⚠️ **Sem isto o Apply NÃO É DESFAZÍVEL.** O `post_frame_undo` só compara o
            // estado quando o frame teve INPUT (`had_input`) — é o proxy dele para "o
            // usuário fez algo que merece um passo". O clique aconteceu no frame ANTERIOR
            // (a deferral acima), então este frame normalmente não tem input nenhum: o diff
            // era pulado e a mutação nunca virava passo. O trabalho é do usuário e cai
            // NESTE frame, então o frame carrega trabalho — e o proxy volta a ser verdade.
            self.any_input_this_frame = true;
        }
        if std::mem::take(&mut self.pending_flip_colorize_clear) {
            self.flip_colorize_clear();
        }
        // ADR-0114 C2 (6º smoke): Trap/Bleed em tempo real depois do Apply. Se um dos dois
        // mudou desde a última rodada, re-roda o corte in-place (o "ajustar a última operação").
        self.flip_colorize_live_adjust();
        // Doc 06 §8: os helpers ao vivo do Gap Closure — mantém o worker sincronizado
        // com o desenho na tela e o alcance atual (só em modo Fill; no-op fora dele).
        self.flip_gap_helpers_tick();

        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let AppGfx {
            surface,
            renderer,
            sim,
            present,
            camera,
            asset_db,
            atlas_is_real: _,
            script,
            theme,
            zen,
            toasts,
            jobs,
            tools,
            layout,
            game_rt,
            motion_fx,
            post_stack,
            tonemap,
            compositor,
            vello_pass,
            vector_scene,
            vec_scene,
            // ADR-0114: cena Flip. A ponte objeto↔entidade é reconciliada todo
            // frame (abaixo, ao lado do vetor). O RENDER é no present phase.
            flip,
            // ADR-0114 W1: o rasterizador + a composição são usados no present.rs.
            flip_render: _,
            flip_compose: _,
            flip_composite: _,
            text_system,
            hero_screen,
            hero_arena,
            clipboard: _,
            prop_state,
            worklist,
            sort_scratch,
            sort_inputs,
            hero_live,
            next_import_cell,
            // A identidade estável do documento pintado é carimbada no SAVE (`project_painter`), que é
            // o único momento em que ela precisa existir — o loop de render não a lê.
            next_painted_doc: _,
            atlas_asset_map,
            logical_texture_map,
            component_registry,
            editor_queue,
            transform_type_id,
            visibility_type_id,
            name_type_id,
            sprite_type_id,
            image_edit_undo,
            // ADR-0054 W0.T6: registries held but not yet consumed
            // inside the render loop — W1 wires Open/Save user paths
            // through `imageio_importers.find_for(...)`.
            imageio_importers: _,
            imageio_exporters: _,
            // Motion Nodes: cooked per frame by `motion_bridge` (M0.T10) into its
            // reused instance buffer while the `motion` tool is active.
            motion,
            // Global rigid physics: stepped per frame by `physics_bridge`
            // (ADR-0131 W1) — reads RigidBody/Collider, writes Transform.
            physics,
        } = gfx;
        let Some(host) = self.host.as_ref() else {
            return;
        };

        // M12 per-frame ticks: ZenMode debounce cooldown + ToastQueue
        // TTL decay. Both are pure data-layer (no Vello paint here).
        zen.tick();
        let prev_toasts = toasts.len();
        toasts.tick();
        if toasts.len() != prev_toasts {
            self.title_dirty = true;
        }
        // Long-operation bars: drop the ones whose worker has stopped. Same once-per-frame
        // settle as the toasts above, and the same reason it lives here rather than at the
        // paint site — a queue that only prunes when someone draws it is a queue that leaks
        // on any frame that is skipped.
        jobs.tick();

        // M14.A: drive the NumberInput stepper continuous-hold. Each
        // frame we ask the dispatcher whether a held arrow should
        // fire one more `ValueChanged` (initial 250 ms delay, then
        // 30 ms repeat). Events drained through `hero.apply_event` so
        // a Transform field that's been incrementing flows through
        // the same commit path as a Enter/blur — the EditorCommand
        // pipeline drain below picks it up.
        if let Some(hero) = hero_screen.as_mut() {
            let tick_events: Vec<WidgetEvent> =
                ph2d_editor::dispatch_tick(hero_arena, &mut hero.store, Self::timestamp_ns())
                    .to_vec();
            for e in tick_events {
                let _ = hero.apply_event(e);
            }
        }

        // Impasto smoke (`PH2D_IMPASTO_SMOKE=1`): one-shot, on the first frame where the atlas plumbing
        // is in scope — spawn a white canvas and SEAT the selection on it, so the artist lands on a
        // ready surface instead of assembling one. The brush itself is armed in `painter_bridge`, when
        // the painter first binds the document.
        if let Some(hero) = hero_screen.as_mut()
            && crate::impasto_smoke::enabled()
            && !std::mem::replace(&mut self.impasto_smoke_done, true)
        {
            let ppm = hero.project.pixels_per_meter;
            let cell = *next_import_cell;
            if let Some(bits) = crate::impasto_smoke::spawn_if_enabled(
                sim,
                renderer,
                asset_db,
                cell,
                ppm,
                atlas_asset_map,
            ) {
                *next_import_cell = next_import_cell.saturating_add(1);
                hero.gizmo.replace_selection(Some(bits));
                hero.bus
                    .push(ph2d_editor::action_bus::EditorAction::SetViewFocus {
                        kind: ph2d_editor::ViewFocusKind::Selected,
                    });
                toasts.push(Toast::success(
                    "Impasto smoke: pick the Painter tool and drag".to_string(),
                ));
            }
        }

        // Mask smoke (`PH2D_MASK_SMOKE=1`): the same dance for the mask coverage law (doc 25 §13.9).
        // Nothing but the canvas is staged — the artist picks the rail chip, so the scene shows the
        // shipped default mask brush rather than a rigged one.
        if let Some(hero) = hero_screen.as_mut()
            && crate::mask_smoke::enabled()
            && !std::mem::replace(&mut self.mask_smoke_done, true)
        {
            let ppm = hero.project.pixels_per_meter;
            let cell = *next_import_cell;
            if let Some(bits) = crate::mask_smoke::spawn_if_enabled(
                sim,
                renderer,
                asset_db,
                cell,
                ppm,
                atlas_asset_map,
            ) {
                *next_import_cell = next_import_cell.saturating_add(1);
                hero.gizmo.replace_selection(Some(bits));
                hero.bus
                    .push(ph2d_editor::action_bus::EditorAction::SetViewFocus {
                        kind: ph2d_editor::ViewFocusKind::Selected,
                    });
                toasts.push(Toast::success(
                    "Mask smoke: paint some art, then the MASK chip — and SCRUB".to_string(),
                ));
            }
        }

        // Wet Paint smoke (`PH2D_WETPAINT_SMOKE=1`): the impasto smoke's exact dance for the fluid
        // mode (ADR-0134 W1) — spawn, seat the selection, arm in `painter_bridge`.
        if let Some(hero) = hero_screen.as_mut()
            && crate::wetpaint_smoke::enabled()
            && !std::mem::replace(&mut self.wetpaint_smoke_done, true)
        {
            let ppm = hero.project.pixels_per_meter;
            let cell = *next_import_cell;
            if let Some(bits) = crate::wetpaint_smoke::spawn_if_enabled(
                sim,
                renderer,
                asset_db,
                cell,
                ppm,
                atlas_asset_map,
            ) {
                *next_import_cell = next_import_cell.saturating_add(1);
                hero.gizmo.replace_selection(Some(bits));
                hero.bus
                    .push(ph2d_editor::action_bus::EditorAction::SetViewFocus {
                        kind: ph2d_editor::ViewFocusKind::Selected,
                    });
                toasts.push(Toast::success(
                    "Wet Paint smoke: pick the Painter tool and drag".to_string(),
                ));
            }
        }

        // New-image modal (Cmd/Ctrl+N) → spawn the chosen blank canvas. The modal's Create button set
        // `new_image_request`; service it here where `gfx` is destructured (sim/renderer/atlas access).
        if let Some(hero) = hero_screen.as_mut()
            && let Some((size, bg)) = hero.store.take_new_image_request()
        {
            let ppm = hero.project.pixels_per_meter;
            let cell = *next_import_cell;
            match crate::image_import::spawn_blank_canvas(
                sim,
                renderer,
                asset_db,
                cell,
                size,
                bg,
                ph2d_core::Vec2::new(0.0, 0.0),
                ppm,
                atlas_asset_map,
            ) {
                Ok((label, bits)) => {
                    *next_import_cell = next_import_cell.saturating_add(1);
                    hero.gizmo.replace_selection(Some(bits));
                    hero.bus
                        .push(ph2d_editor::action_bus::EditorAction::SetViewFocus {
                            kind: ph2d_editor::ViewFocusKind::Selected,
                        });
                    toasts.push(Toast::success(format!("New canvas · {label} ({size}²)")));
                }
                Err(e) => {
                    toasts.push(Toast::error(format!("New canvas failed: {e}")));
                }
            }
            self.title_dirty = true;
        }

        // M7 per-frame GC step. Cheap (p99 ≤ 10µs target per the M7
        // gate test) — keeps the Luau heap from accumulating between
        // future scripted ticks. Errors here are non-fatal; just log.
        if let Some(host) = script
            && let Err(e) = host.gc_step()
        {
            eprintln!("M7 gc_step error: {e}");
        }

        // ── Coalesced resize + FLUID-DRAG present mode (Enio 2026-07-05, take 2) ──
        // The full re-fit (layout + RT reallocs + rebinds) runs once per frame with the latest size —
        // everything stays exact-size, no stretching (a first "two-speed" attempt stretched between
        // re-fits and read as "truncado, sem fluidez"). The REAL live-drag jank lever is the PRESENT
        // MODE: under VSync (`Fifo`) every `surface.configure` discards the swapchain images and the
        // next acquire BLOCKS up to a full refresh — a drag reconfigures every frame, so the app ran at
        // a fraction of the refresh rate. While resize events stream we switch to the backend's best
        // NON-BLOCKING mode (Immediate, else Mailbox) and restore the configured mode a few quiet
        // frames after the drag settles.
        let resize_streaming = self.pending_resize.is_some();
        if resize_streaming {
            if self.resize_saved_present_mode.is_none() {
                let cur = surface.present_mode();
                let fast = surface.best_nonblocking_mode();
                if fast != cur {
                    self.resize_saved_present_mode = Some(cur);
                    surface.set_present_mode(fast);
                }
            }
            self.resize_settle_frames = RESIZE_SETTLE_FRAMES;
        } else if self.resize_settle_frames > 0 {
            self.resize_settle_frames -= 1;
            if self.resize_settle_frames == 0
                && let Some(mode) = self.resize_saved_present_mode.take()
            {
                surface.set_present_mode(mode);
            }
        }
        // Apply the coalesced resize once per frame.
        if let Some(size) = self.pending_resize.take() {
            surface.resize(size);
            // Layout + every offscreen RT in the pipeline must follow
            // surface size. M14.5: game_rt, tonemap output, vello
            // intermediate — all three; then the compositor's bind
            // group must be rebuilt against the new texture views.
            // Size every offscreen RT to the surface's CLAMPED size, not the
            // raw winit size. `surface.resize` clamps each dim to ≥1, and
            // `game_rt.ensure_size` REJECTS a 0 dim (keeps its old size). On a
            // transient 0-dimension frame (minimize/restore) the raw size
            // would diverge: game_rt stays old while the W3 §8 clip/mask
            // stencil sizes to `surface.size()` → a render pass pairing the
            // game_rt color attachment with a differently-sized stencil =
            // wgpu validation panic. Using the clamped size keeps color +
            // stencil extents equal every frame (audit MEDIUM fix).
            let clamped = surface.size();
            *layout = EditorLayout::new(clamped.width as f32, clamped.height as f32);
            let dim = (clamped.width, clamped.height);
            game_rt.ensure_size(surface.gpu(), dim);
            // doc 67: the Motion glow RT + blur chain track the surface too.
            motion_fx.ensure_size(surface.gpu(), dim);
            // ADR-0145: the post-stack scratch RT tracks the surface alongside game_rt.
            post_stack.ensure_size(surface.gpu(), dim);
            tonemap.ensure_size(surface.gpu(), dim);
            tonemap.rebind_game_view(
                surface.gpu(),
                game_rt
                    .texture()
                    .create_view(&wgpu::TextureViewDescriptor::default()),
            );
            vello_pass.ensure_size(surface.gpu(), dim);
            compositor.rebind(
                surface.gpu(),
                tonemap
                    .output_texture()
                    .create_view(&wgpu::TextureViewDescriptor::default()),
                vello_pass
                    .intermediate_texture()
                    .create_view(&wgpu::TextureViewDescriptor::default()),
            );
            self.handler.on_resize(clamped, host.scale_factor());
            self.title_dirty = true;
        }

        // Drive fixed-step accumulator.
        let now = Instant::now();
        let wall_dt = now.duration_since(self.last_frame).as_secs_f64();
        self.last_frame = now;
        // M14.4g: feed EWMA frame-time using the same `wall_dt` —
        // single source of truth for "how long did the last frame
        // take". α=0.1 smooths over jitter while still tracking
        // sustained changes in ~10 frames.
        let frame_ms_now = (wall_dt * 1000.0) as f32;
        const ALPHA: f32 = 0.1;
        self.frame_ms_ewma = ALPHA * frame_ms_now + (1.0 - ALPHA) * self.frame_ms_ewma;
        // **W15.1 (ADR-0040-amendment-2):** per-frame heartbeat on the ACTIVE tool,
        // with the real frame delta. Drives the watercolor live wet-on-wet diffusion
        // (ADR-0049 / ADR-0077 D11) so the wash keeps blooming + drying after pen-up;
        // a no-op default for every other tool, so this costs nothing elsewhere.
        if let Some(t) = tools.active_mut() {
            // Frame profiler: the heartbeat is where the watercolor live recomposite runs while the
            // brush is held (soak pour → apply_watercolor) — time it so a paint slowdown is attributable
            // (one Instant when profiling; zero cost otherwise).
            let tick_t0 = frame_prof_on().then(Instant::now);
            t.on_tick(frame_ms_now);
            if let Some(t0) = tick_t0 {
                let us = t0.elapsed().as_micros() as u64;
                // A janela: soma, pico e quantos frames de fato trabalharam.
                if us > 0 {
                    FRAME_PROF_TICK_SUM_US.with(|c| c.set(c.get() + us));
                    FRAME_PROF_TICK_MAX_US.with(|c| c.set(c.get().max(us)));
                    FRAME_PROF_TICK_N.with(|c| c.set(c.get() + 1));
                }
                let st = self.last_paint_stamp_us;
                if st > 0 {
                    FRAME_PROF_STAMP_SUM_US.with(|c| c.set(c.get() + st));
                    FRAME_PROF_STAMP_MAX_US.with(|c| c.set(c.get().max(st)));
                    FRAME_PROF_STAMP_N.with(|c| c.set(c.get() + 1));
                }
            }
            // Fill dwell gesture: a held-still ColorDrop fires the fill + enters live threshold-adjust
            // (see `input_dispatch::fill_drag`). `last_pointer` is disjoint from `self.gfx.tools`.
            crate::input_dispatch::fill_drag::fill_drag_tick(t, self.last_pointer, frame_ms_now);
        }
        let report = self.fixed_step.advance(wall_dt);
        if report.dropped_secs > 0.0 {
            eprintln!(
                "warn: dropped {:.3}s of sim time (max_substeps cap)",
                report.dropped_secs
            );
        }
        panic::set_frame_id(self.fixed_step.tick_count());
        // Advance the engine-wide timeline cursor by the ticks that ran this
        // frame. Every animatable system samples the Playhead for its current
        // value; while paused it holds. (General timeline, M0 time wire.)
        self.playhead.advance_ticks(report.ticks);
        // The Keys view's clip clock advances on the same ticks — it only moves
        // when ITS transport is playing, so advancing both is harmless and keeps
        // "play a clip's keys" working without a second tick source.
        self.clip_playhead.advance_ticks(report.ticks);
        // The Containers view's clock (Enio, 2026-07-22): editing a container's
        // lanes plays the CONTAINER's own interior, not the scene. A third clock,
        // like the clip's — it only advances while ITS transport plays.
        self.container_playhead.advance_ticks(report.ticks);

        // Sim tick + extract — extracted to sibling `sim_extract.rs`
        // (Wave 3.2 stage A). Runs the bouncing-motion sim tick and
        // the ADR-0021 / ADR-0025 propagate-transforms + sprite
        // emit pass.
        // Demo bouncing-motion integrates ONCE per render frame, so it
        // must use the real wall-clock delta — not the fixed timestep —
        // or its speed scales with the frame rate. That was invisible
        // under vsync (~60 fps) but the non-blocking `Immediate` present
        // mode (stutter fix, 2026-05-21) uncaps the loop to hundreds of
        // fps, which made the sprites race + jitter. `wall_dt` makes the
        // motion frame-rate-independent (real-time, smooth at any fps);
        // clamped so a hitch / debugger pause can't teleport a sprite.
        // (The proper fixed-step substep integration lands with the M10
        // gameplay sim; this is the M5 demo's stop-gap.)
        let dt = (wall_dt as f32).min(1.0 / 30.0);
        // Lens F (2026-05-26): the Background-Removal live preview no
        // longer suppresses the sprite + paints a Vello overlay on
        // top; instead it injects a synthetic `PreviewOverride` that
        // swaps the entity's `RenderInstance.texture_id` for a
        // transient `IndividualTextureStore` slot owning the preview
        // pixels. Same `Rgba8UnormSrgb` + sprite shader + premul
        // blend as Apply → byte-for-byte parity. The GPU slot is
        // populated by `bgremoval_preview::dispatch` LATER in the
        // frame, so reading `self.bgremoval_preview_gpu` here picks
        // up last frame's upload (1-frame lag is invisible — the
        // preview is a continuous animation).
        let bgremoval_preview_override: Option<sim_extract::PreviewOverride> = self
            .bgremoval_preview_gpu
            .map(|gpu| sim_extract::PreviewOverride {
                entity_bits: gpu.entity_bits,
                texture_id: gpu.texture_id,
                // Byte-space premul upload + Apply uses the same flag.
                premultiplied: true,
            });
        // W3 Painter sprite-suppression: the active sprite's `preview_override`.
        // The base layer composite REPLACES the source sprite in-place (no
        // overlay duplication) so its opacity/representation affects the whole
        // image.
        let painter_preview_override: Option<sim_extract::PreviewOverride> = self
            .painter_preview_gpu
            .map(|gpu| sim_extract::PreviewOverride {
                entity_bits: gpu.entity_bits,
                texture_id: gpu.texture_id,
                premultiplied: true,
            });
        // A sprite used as the brush Shape but NOT currently selected previews its OWN composite too, so
        // brush opacity/blend remote-control edits show on it in real time (its `IndividualTextureStore`
        // slot, driven by the painter bridge). Distinct entity from the active sprite ⇒ a SEPARATE override.
        let painter_shape_source_override: Option<sim_extract::PreviewOverride> = self
            .painter_shape_source_preview_gpu
            .map(|gpu| sim_extract::PreviewOverride {
                entity_bits: gpu.entity_bits,
                texture_id: gpu.texture_id,
                premultiplied: true,
            });
        // Several sprites can preview at once now (active + shape-source); `sim_extract` matches each
        // sprite to its own entry. Painter and BgRemoval are never active simultaneously (one active tool).
        let preview_overrides: Vec<sim_extract::PreviewOverride> = [
            painter_preview_override,
            bgremoval_preview_override,
            painter_shape_source_override,
        ]
        .into_iter()
        .flatten()
        .collect();
        // Project px/m for `Sprite::resolve_anchor` (intrinsic-px `offset` →
        // local meters). `None` only under the M5 demo / headless, whose sprites
        // use the centered/offset defaults so the value is inert; fall back to
        // the canonical default.
        let ppm = hero_screen
            .as_ref()
            .map(|h| h.project.pixels_per_meter)
            .unwrap_or(ph2d_editor::project::DEFAULT_PIXELS_PER_METER);
        // W3.T3.11: project-default sampling for all-Inherit sprites, from the
        // project image filter (PixelArt → Nearest, Smooth → Linear); repeat
        // defaults to clamp (Disabled).
        let default_filter = match hero_screen
            .as_ref()
            .map(|h| h.project.image_filter)
            .unwrap_or(ph2d_render::ImageFilterMode::Smooth)
        {
            ph2d_render::ImageFilterMode::PixelArt => ph2d_ecs::FilterMode::Nearest,
            ph2d_render::ImageFilterMode::Smooth => ph2d_ecs::FilterMode::Linear,
        };
        // W2.T4 cooked-texture loader: resolve + decode + upload every
        // `SpriteSource::CookedTexture` sprite's KTX2 (for the device tier,
        // descending the fallback ladder) BEFORE extract reads back the cached
        // `texture_id`. Idempotent + cheap after the first upload.
        cooked_texture_bridge::ensure_uploaded(sim, renderer, asset_db, logical_texture_map);
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
            let picked = timeline_presets::intents_for_pick(&self.timeline, pick);
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
        // `dragging_entity` is still needed here: `timeline_bridge::run` skips
        // it in the apply so the document never fights the live gizmo drag.
        let dragging_entity: Option<u64> = hero_screen
            .as_ref()
            .and_then(|h| h.gizmo.drag)
            .map(|d| d.entity_bits);
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
        // Entering / switching / leaving a container arms the CONTAINER clock with its
        // OWN loop (edge-triggered, the nav mirror of the `keys_mode` sync below). The
        // scene clock is deliberately NOT touched — the Arrange loop survives the visit.
        if container != self.last_timeline_container {
            self.last_timeline_container = container;
            timeline_bridge::on_container_nav_change(
                &self.timeline.doc,
                container,
                &mut self.container_playhead,
            );
        }
        if keys_mode != self.last_timeline_keys_mode {
            self.last_timeline_keys_mode = keys_mode;
            if keys_mode {
                ph2d_timeline::sync_transport_loop(
                    &self.timeline.doc,
                    &mut self.clip_playhead,
                    true,
                );
            } else {
                ph2d_timeline::sync_transport_loop(&self.timeline.doc, &mut self.playhead, false);
            }
        }
        if self.timeline_insert_key {
            self.timeline_insert_key = false;
            // The K clock is the clock the animator is watching: the container's
            // inside one, the scene's on Arrange. Keys solos and reads the clip
            // directly (no scratch). The value/time helpers below read the same clock.
            let k_time = if container.is_some() {
                self.container_playhead.time()
            } else {
                self.playhead.time()
            };
            // The scratch must describe THIS instant before the Arrange/container K asks
            // it where a key lands or whether the pose is reachable — ROOTED at whatever
            // stack the animator is looking at (the scene's, or the container's interior;
            // Enio 2026-07-22). Skip it in Keys (solo has no scratch). Without the prime
            // the resolve runs against the PREVIOUS frame's strip state.
            if !keys_mode {
                self.timeline.doc.prime_rooted(container, k_time);
            }
            if let Some(entity) = hero_screen
                .as_ref()
                .and_then(|h| h.gizmo.iter_selected().next())
            {
                let mut props: Vec<_> = self
                    .timeline
                    .doc
                    .bindings()
                    .iter()
                    .filter(|b| b.entity == entity)
                    .map(|b| b.prop)
                    .collect();
                // If the entity has no position animation yet, K creates it in the
                // toggle's mode (ADR-0141) — the same choice auto-key makes for a
                // fresh object, so K and auto-key agree on Position vs separate X/Y.
                // An entity already in one mode keeps it (the branch below routes
                // Position → anchor, X/Y → scalar), so this only injects the FIRST
                // position channel; the existing loop then authors it uniformly.
                let has_position = props.iter().any(|p| {
                    matches!(
                        p,
                        ph2d_timeline::PropKind::Position
                            | ph2d_timeline::PropKind::TranslationX
                            | ph2d_timeline::PropKind::TranslationY
                    )
                });
                if !has_position {
                    // Fresh object → the After Effects default (motion path); the
                    // per-object toggle marks it Separate otherwise.
                    match self.timeline.doc.position_key_mode(entity, true) {
                        ph2d_timeline::PositionKeyMode::Path => {
                            props.push(ph2d_timeline::PropKind::Position);
                        }
                        ph2d_timeline::PositionKeyMode::Separate => {
                            props.push(ph2d_timeline::PropKind::TranslationX);
                            props.push(ph2d_timeline::PropKind::TranslationY);
                        }
                    }
                }
                for prop in props {
                    // ⚠️ **Position não amostra um escalar: ele acrescenta uma ÂNCORA**
                    // (ADR-0141). "Capturar a pose" numa trajetória é uma edição da
                    // GEOMETRIA — a âncora nova muda o percurso, que é o que as outras
                    // keys medem —, então não há valor a computar aqui e o
                    // `sample_prop_value` recusa este kind de propósito. O tempo é o
                    // mesmo `k_time` das outras (o relógio da vista), e o intent leva o
                    // LUGAR; quem aplica é que sabe quanto ele vale em distância.
                    if prop == ph2d_timeline::PropKind::Position {
                        if let Some(xf) = sim
                            .world()
                            .get::<ph2d_ecs::Transform>(ph2d_ecs::Entity::from_bits(entity))
                        {
                            // ⚠️ O tempo vem das MESMAS portas das outras keys: em
                            // Keys, o relógio do clip cortado e remapeado
                            // (`solo_key_time`, a metade extraída do
                            // `key_authoring_solo`); em Arrange/container, o
                            // `key_insert_time`, que pode RECUSAR (o clip não toca
                            // aqui, ou toca duas vezes). Uma composição própria aqui
                            // seria a pose a saltar de volta.
                            let t = if keys_mode {
                                Some(timeline_bridge::solo_key_time(
                                    &self.timeline,
                                    entity,
                                    self.clip_playhead.time(),
                                ))
                            } else {
                                timeline_bridge::key_insert_time(
                                    &self.timeline,
                                    entity,
                                    prop,
                                    k_time,
                                )
                            };
                            if let Some(t) = t {
                                self.timeline_intents.push(
                                    ph2d_timeline::TimelineIntent::AddPathKey {
                                        entity,
                                        t,
                                        at: [xf.translation.x, xf.translation.y],
                                    },
                                );
                            }
                        }
                        continue;
                    }
                    // In KEYS mode the animator sees the active clip soloed, so the
                    // pose IS the clip's value and the time is the clip playhead's —
                    // stored directly, and K never refuses (there is no stack to be
                    // overridden by or to play twice). In ARRANGE mode the pose is a
                    // blend, so `key_value_for` inverts it and `key_insert_time` can
                    // REFUSE (the clip has no influence, or plays zero/two times).
                    let authored = if keys_mode {
                        timeline_bridge::key_authoring_solo(
                            sim.world(),
                            &self.timeline,
                            entity,
                            prop,
                            self.clip_playhead.time(),
                        )
                    } else {
                        // Arrange OR inside a container — both read the stack (rooted at
                        // the scene or the container respectively, primed above) at
                        // `k_time`. The value/time helpers are root-aware, so a container
                        // key lands on the container's clock and refuses on the
                        // container's terms, never the scene's.
                        timeline_bridge::key_value_for(
                            sim.world(),
                            &self.timeline,
                            entity,
                            prop,
                            k_time,
                        )
                        .zip(timeline_bridge::key_insert_time(
                            &self.timeline,
                            entity,
                            prop,
                            k_time,
                        ))
                    };
                    if let Some((value, t)) = authored {
                        self.timeline_intents
                            .push(ph2d_timeline::TimelineIntent::AddKey {
                                entity,
                                prop,
                                t,
                                value,
                                interp: timeline_bridge::default_interp(),
                            });
                    }
                }
            }
        }
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
            dragging_entity,
            &mut self.autokey,
            keys_mode,
            container,
            &mut self.timeline_signals,
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
        // Hand each signal the SCENE play crossed this frame to a consumer (ADR-0143).
        // v1 consumer: a toast — the visible proof the decoupled channel round-trips.
        // Audio/gameplay/Luau are the deferred cross-line consumers of the SAME outbox;
        // the timeline emits an event and never calls any of them (ADR-0075).
        for sig in self.timeline_signals.out.drain(..) {
            toasts.push(ph2d_editor::Toast::info(format!("Signal: {}", sig.name)));
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
        ph2d_panel_timeline::set_current_timeline(Some(self.timeline_view.clone()));
        // Global rigid physics (ADR-0131 W1): step the rapier world at the
        // Playhead tick and read poses back into Transform, BEFORE
        // sim_extract so bodies render the same frame. Runtime-truth: play
        // = N sequential steps + readback; paused = settle to the authored
        // pose (read-only on Transform → no spurious undo step when idle).
        // ⚠️ ONE transport, two consumers — the curves and the rapier world.
        // Which of them the clock reaches is the artist's call, armed on the
        // transport bar and OFF by default (`TimelineFlags::simulate_physics`).
        let simulate_physics = self.timeline.flags.simulate_physics;
        physics_bridge::dispatch(
            physics,
            sim,
            &self.playhead,
            self.fixed_step.fixed_dt(),
            &mut self.timeline.doc,
            simulate_physics,
        );
        // O flash do estouro envelhece uma vez por frame, aqui: ao lado do
        // dispatch da física, que é a fase em que o tempo do mundo anda. Um canal
        // PRÓPRIO, porque uma explosão é um impulso e não deixa estado no mundo
        // para uma marca derivada ler (o irmão exato do `ContactFlash`).
        crate::body_grab::age_blast_flash(&mut self.blast_flash);
        // A joint that gave way announces itself (W-J7). The overlay shows where
        // and that; only the event carries the load it broke at.
        for msg in physics_bridge::break_reports(physics, sim) {
            toasts.push(Toast::warning(msg));
        }
        sim_extract::run(
            dt,
            sim,
            present,
            renderer,
            prop_state,
            worklist,
            sort_scratch,
            sort_inputs,
            &preview_overrides,
            ppm,
            camera.cull_mask,
            default_filter,
            ph2d_ecs::RepeatMode::Disabled,
        );

        // Sprite-layer clear color = backdrop visible in the canvas
        // area through the transparent regions of `vello_rt`. Live
        // editor mode wants a static neutral surface so it doesn't
        // pulse rainbow under the chrome.
        //
        // Why 0.047 instead of the theme-canonical 0.012:
        // pre-M14.5 the chrome AA edges were rendered against a
        // backdrop of `Bg1` painted by Vello as sRGB byte ~12,
        // which the legacy wgpu blitter sampled as 12/255 ≈ 0.047
        // *treated as linear* (the documented vello-blitter gamma
        // confusion in `vello_pass.rs`). Anti-aliased chrome edges
        // in `ph2d-tokens` are calibrated against that 0.047
        // backdrop. Setting `game_rt` clear to the theme's true
        // linear 0.012 would make the AA halos contrast strongly
        // against the now-much-darker dst — that's exactly the
        // "pixelated borders" regression seen in M14.5 round 2.
        // Match the legacy backdrop value here; the chrome edges
        // composite identically.
        let (r, g, b) = if hero_live.is_some() {
            // The post-stack smoke needs a BRIGHT, uniform canvas so its vignette reads
            // as darkened EDGES (on the dark 0.047 backdrop the darkening is invisible,
            // and a lift would only brighten the centre — the confusion Enio reported).
            // Gated on the smoke env, so the calibrated editor backdrop is untouched.
            if crate::post_stack_smoke::is_active() {
                (0.55, 0.55, 0.55)
            } else {
                (0.047, 0.047, 0.055)
            }
        } else {
            let t = self.fixed_step.tick_count() as f64 * self.fixed_step.fixed_dt();
            (
                (t.sin() * 0.05 + 0.05).clamp(0.0, 1.0),
                ((t + 2.094).sin() * 0.05 + 0.05).clamp(0.0, 1.0),
                ((t + 4.188).sin() * 0.05 + 0.05).clamp(0.0, 1.0),
            )
        };

        let window_size = surface.size();
        // M11: build the widget scene up-front (no GPU work yet — just
        // VectorScene encoding). Done outside acquire_frame so an
        // Occluded/Timeout doesn't waste the encoder.
        let viewport = EditorRect::new(
            0.0,
            0.0,
            window_size.width as f32,
            window_size.height as f32,
        );
        vector_scene.reset();
        let mut paint_ctx = PaintCtx {
            theme: *theme,
            viewport,
            text: text_system,
        };

        // Default editor mode: AppGfx owns a HeroScreen with a
        // retained WidgetStore (ADR-0024). Paint reads + writes its
        // hit_index each frame; pointer/key events are forwarded to
        // it from window_event handlers via `hero_screen.handle_*`.
        // `hero_screen` is `None` only under `PH2D_M5_DEMO=1`.
        if let Some(hero) = hero_screen.as_mut() {
            // Snapshot publication phase — extracted to sibling
            // `snapshots.rs` as a free fn taking explicit refs (Wave
            // 3.2 stage A). Reads PresentWorld + SimWorld + AssetDb,
            // writes onto the HeroScreen (live_hierarchy, grid_view,
            // stats, gizmo_view, inspector_*) so the paint pass
            // honors the HR-8 / ADR-0021 boundary.
            snapshots::publish(
                hero,
                hero_live,
                sim,
                present,
                camera,
                asset_db,
                atlas_asset_map,
                renderer,
                window_size,
                self.last_pointer,
                self.frame_ms_ewma,
                self.frame_cpu_ms_ewma,
                diag_input_events,
                diag_paint_stamps,
                self.paint_ms_ewma,
                // Deform Transform live ⇒ the sprite gizmo is suppressed for the frame (its corner
                // handles share the deform gizmo's screen corners on a whole-image transform).
                painter_bridge_queries::deform_transform_gizmo_active(tools),
                vec_scene,
                // O gizmo da forma só existe fora da ferramenta vetorial, ou no modo
                // Select dela (ADR-0112).
                !tools
                    .active()
                    .is_some_and(|t| t.id() == ph2d_editor::ToolId::new("vector"))
                    || self.vec_draw_config.mode == ph2d_tool_vector::DrawMode::Select,
                flip,
                // Idem para o objeto Flip: gizmo fora da tool Flip, ou no modo Select
                // dela — em Draw/Erase ele comeria o clique do canvas (ADR-0112 parity).
                !tools
                    .active()
                    .is_some_and(|t| t.id() == ph2d_editor::ToolId::new("flip"))
                    || matches!(
                        self.flip_style.map(|s| s.mode),
                        Some(ph2d_tool_flip::FlipMode::Select)
                    ),
                // W4: the range the §11 Bake button covers, resolved HERE
                // because the shell owns both the document and the clock, and
                // shown on the button so the artist never has to guess it.
                // Two numbers now — the loop's start is honoured (W-BakeRange),
                // so a `[2s, 5s]` loop bakes `[2s, 5s]` and the button says so.
                {
                    let (bs, be) = physics_bake::bake_range(&self.timeline.doc, &self.playhead);
                    (bs as f32, be as f32)
                },
                // Which pose channels the Bake selector shows as chosen.
                self.bake_channels.tag(),
                // The pending join KIND the §11 selector shows as chosen.
                self.join_kind,
                // The armed §12 joint-body eyedropper, so the waiting slot's
                // picker paints pressed.
                self.joint_body_pick,
                // W-J4: o gesto de desenhar está armado?
                self.joint_draw_armed,
                // W-J2/W-J2b: every grabbable joint anchor. Resolved HERE
                // because `publish` does not take the bridge, and through the
                // SAME door `sync_joint_pivots` uses for the A pivot — two
                // derivations of "where is this anchor" is how two dots would
                // come to disagree. Rest-only (the rule lives in the callee):
                // during play the overlay draws the SOLVER's anchors, and these
                // authored ones would describe a pose the artist is not editing.
                {
                    // As DUAS famílias numa lista só: as âncoras (sempre) e os
                    // grips de parâmetro (só com o overlay de joints na tela —
                    // eles agarram a geometria DELE).
                    let at_rest = !self.playhead.is_playing();
                    let mut hs = point_gizmo::joint_anchor_handles(sim, physics, at_rest);
                    hs.extend(point_gizmo::joint_param_handles(
                        physics,
                        camera,
                        window_size,
                        self.show_colliders,
                        at_rest,
                    ));
                    hs
                },
                // The candidate a live anchor drag has caught (the crosshair).
                self.joint_anchor_drag.and_then(|d| d.snap),
            );
            // Flip W7.5/§4.A: os gizmos do modo Edit — só na tool Flip em modo Edit. Os
            // dois campos próprios no `GizmoStateGroup` (append-only) são MUTUAMENTE
            // EXCLUSIVOS por `is_instanced`: a `pose_view` só publica quando o quadro
            // visível é uma INSTÂNCIA (rotate/escala da pose), a `selection_view` só
            // quando é arte EXCLUSIVA com seleção (rotate/escala assado na geometria).
            // O painter os desenha keyed (`FlipPose`/`FlipSelection`), sem interior — a
            // seleção de traço do Edit continua dona do canvas.
            let flip_edit_mode = tools
                .active()
                .is_some_and(|t| t.id() == ph2d_editor::ToolId::new("flip"))
                && matches!(
                    self.flip_style.map(|s| s.mode),
                    Some(ph2d_tool_flip::FlipMode::Edit)
                );
            hero.gizmo.pose_view = flip_edit_mode
                .then(|| {
                    crate::flip_pose_gizmo::pose_view(
                        sim,
                        flip,
                        &self.flip_entities,
                        crate::flip_pose_gizmo::PoseViewInputs {
                            playhead: &self.playhead,
                            active_layer: self.flip_active_layer,
                            last_pointer: self.last_pointer,
                        },
                        camera,
                        window_size,
                    )
                })
                .flatten();
            hero.gizmo.selection_view = flip_edit_mode
                .then(|| {
                    crate::flip_selection_gizmo::selection_view(
                        sim,
                        flip,
                        &self.flip_entities,
                        crate::flip_selection_gizmo::SelectionViewInputs {
                            playhead: &self.playhead,
                            active_layer: self.flip_active_layer,
                            last_pointer: self.last_pointer,
                        },
                        camera,
                        window_size,
                    )
                })
                .flatten();
            // Motion Nodes: o gizmo de canvas de um FIELD ESPACIAL (`field.box`, …) — só com
            // a tool Motion ativa + um field espacial selecionado no grafo. Slot próprio
            // (`field_view`), desenhado keyed (`MotionField`) ⇒ o gizmo de sprite (`view`)
            // fica INTOCADO e os dois nunca coexistem por modalidade da tool. `tools` é o
            // local (não `self.motion_tool_active()`, que re-emprestaria `self.gfx`), espelho
            // do `flip_edit_mode` acima.
            let motion_tool_active = tools
                .active()
                .is_some_and(|t| t.id() == ph2d_editor::ToolId::new("motion"));
            // As dims da CENA (o sub-retângulo do split, `CenterSplit::scene_viewport`) — o
            // gizmo é pintado e arrastado com ELAS, casando com o `set_viewport` do render
            // (present.rs). É o fix do drift crônico: sob o split a cena renderiza na banda
            // e o chrome projetava a janela cheia. Fora do split = janela cheia (no-op).
            let (scene_w, scene_h) =
                crate::field_gizmo::scene_window_wh(hero.view.center_split, window_size);
            hero.gizmo.field_view = motion_tool_active
                .then(|| {
                    crate::field_gizmo::field_view(
                        motion,
                        camera,
                        scene_w,
                        scene_h,
                        self.last_pointer,
                    )
                })
                .flatten();
            // ─────────────────────────────────────────────────────────
            // Wave 2.5 PR 11.8 closeout — consolidated bus drain.
            // ─────────────────────────────────────────────────────────
            //
            // Previously, each of the 18 EditorAction variants had its
            // own filter-and-replace block (one per drain site, ~20 LOC
            // of "drain, capture this variant, push others back" each).
            // Now we drain the bus ONCE at the top of this section,
            // categorize every variant into per-kind locals, and the
            // dispatch sites further down just read `if let Some(x) = X`.
            //
            // First-wins for most variants (matches the old
            // `found.is_none()` short-circuit). Latest-wins for
            // `InspectorNameEdit` (preserves the pre-bus Option
            // coalescing that drained at most one SetComponent per
            // frame). `Bgremoval` is NOT categorized here — it keeps
            // a separate filter-and-replace at its original site so
            // its `bgremoval_active` gate runs AFTER any same-frame
            // `ActivateTool { tool_id: "bgremoval" }` fires (1-frame
            // defer edge case).
            //
            // Audit 2026-05-26 F1: 6 flags hardcoded per-tool (`activate_bgremoval`
            // etc.) substituídas por uma única option `pending_image_tool_activation`.
            // O drain único abaixo usa `installed_registry().cluster("image_tools")`
            // + `Tool::label()` para dispatch data-driven. Painter + os 5 image-tools
            // pré-existentes flow pelo mesmo canal — anti-padrão Image Tools Bugs
            // §2.b fechado neste ponto da render loop.
            let mut pending_image_tool_activation: Option<&'static str> = None;
            let mut visibility_toggle_row: Option<NodeId> = None;
            let mut lock_toggle_row: Option<NodeId> = None;
            let mut group_toggle_row: Option<NodeId> = None;
            let mut reparent_intent: Option<ph2d_editor::screens::hero::HierReparentIntent> = None;
            let mut duplicate_row: Option<NodeId> = None;
            // Set by `hierarchy::dispatch` to `(source_bits, new_bits)` when a sprite is duplicated, so
            // we can fork the copy onto its own texture (independent object) post-dispatch.
            let mut duplicate_made: Option<(u64, u64)> = None;
            let mut add_child_row: Option<NodeId> = None;
            let mut reset_transform_row: Option<NodeId> = None;
            let mut delete_row: Option<NodeId> = None;
            // Enio 2026-05-27: right-click → Merge Sprites in Hierarchy.
            // Carries the clicked row's `NodeId` (the merged sprite
            // adopts that row's parent for Hierarchy placement); the
            // drain reads the full multi-selection at apply time.
            let mut merge_sprites_row: Option<NodeId> = None;
            let mut use_as_brush_texture_row: Option<NodeId> = None;
            let mut use_as_brush_shape_row: Option<NodeId> = None;
            let mut use_as_paper_row: Option<NodeId> = None;
            let mut use_as_granulation_row: Option<NodeId> = None;
            let mut hierarchy_row_click: Option<NodeId> = None;
            let mut hierarchy_select_intent: Option<hierarchy::HierarchySelectIntent> = None;
            let mut rename_seed_row: Option<NodeId> = None;
            let mut rename_commit: Option<(NodeId, String)> = None;
            let mut view_focus_kind: Option<ph2d_editor::ViewFocusKind> = None;
            let mut reimport_entity: Option<u64> = None;
            // Fase 0e: per-sprite tools collect a Vec<u64> instead of
            // Option<u64> so a multi-select OneShotImageOp broadcast
            // applies the bake to every selected sprite (legacy
            // single-select still works — the Vec just carries one
            // entry). image_edit::dispatch iterates each Vec.
            let mut trim_entities: Vec<u64> = Vec::new();
            let mut make_square_entities: Vec<u64> = Vec::new();
            let mut real_size_entities: Vec<u64> = Vec::new();
            let mut rasterize_entities: Vec<u64> = Vec::new();
            let mut undo_image_edit = false;
            // ADR-0108 Fase 1: a Boolean button (Union/Subtract/Intersect) in the
            // docked Vector panel forwards a `ToolPanelEvent::Click`; the op acts
            // on the DOCUMENT (shell-owned `vec_scene`), not the tool's Style, so
            // capture it here and apply after the drain (mirror of the U/I/D
            // hotkeys, next to the vector render).
            let mut pending_vec_bool: Option<ph2d_vec_boolean::BoolOp> = None;
            let mut pending_vec_expand: Option<crate::vec_expand::Expand> = None;
            // **A ESCALA da seleção de nós** (plano 25 §6, W3b): os dois alcances que o retângulo
            // não dá. Não são edições de documento — só mudam QUEM está selecionado —, então não
            // abrem passo de undo (o `post_frame_undo` compara o ESTADO, e a seleção não é dele).
            let mut pending_vec_select_subpath = false;
            let mut pending_vec_select_same = false;
            // O índice do perfil nomeado que o clique pediu (W2b), se algum.
            let mut pending_width_preset: Option<usize> = None;
            // ADR-0128: o botão "Blend" cria um Blend Object VIVO da seleção; o slider Steps
            // ajusta o blend selecionado ao vivo. (O destrutivo `vec_blend::apply` sobrevive só
            // para os smokes — o painel não o alcança mais.)
            let mut pending_create_blend = false;
            let mut pending_reset_spine = false;
            let mut pending_expand_blend = false;
            let mut pending_release_blend = false;
            let mut pending_blend_steps: Option<u32> = None;
            let mut pending_create_morph = false;
            let mut pending_morph_t: Option<f32> = None;
            // ADR-0129: o botão "Envelope" envolve a seleção numa gaiola (container); Expand
            // materializa a deformada e Release ressuscita a fonte autorada — os dois dissolvem.
            let mut pending_create_envelope = false;
            let mut pending_textpath: Option<crate::vec_text_ride::TextPathCmd> = None;
            let mut pending_textpath_offset: Option<f64> = None;
            // Pattern on Path (plano 23): o comando de vínculo + os dois sliders, drenados como os
            // do texto (o motivo é o PRIMÁRIO, o guia é o outro selecionado).
            let mut pending_patternpath: Option<crate::pattern_live::PatternPathCmd> = None;
            let mut pending_pp_spacing: Option<f64> = None;
            let mut pending_pp_start: Option<f64> = None;
            let mut pending_pp_end: Option<f64> = None;
            let mut pending_pp_slide: Option<f64> = None;
            let mut pending_pp_offset: Option<f64> = None;
            // Contour (pesquisa `20_*` #9): os três comandos + os três sliders + os dois trios
            // exclusivos. `Add`/`Remove` são portas do MODELO (armam/tiram o componente),
            // `Expand` materializa; os knobs só editam o que já existe.
            let mut pending_contour: Option<crate::contour_live::ContourCmd> = None;
            let mut pending_contour_steps: Option<f64> = None;
            let mut pending_contour_d: Option<f64> = None;
            let mut pending_contour_accel: Option<f64> = None;
            let mut pending_contour_join: Option<u8> = None;
            let mut pending_contour_side: Option<u8> = None;
            // Filters (FX raster, plano 24). `Some(Some(k))` arma o tipo `k`; `Some(None)` remove.
            // Filters (a PILHA de FX raster, plano 24): um comando (Add/✕/↑/↓/👁) e um valor de
            // slider por frame, decodificados pela porta única `fx_live::hit_of`.
            let mut pending_filter_cmd: Option<crate::fx_live::FilterHit> = None;
            // O arrasto de um punho da rampa: `(linha, índice de AUTORIA do stop, posição 0..1)`.
            // Um `pending`, como o `FilterHit`, e pelo MESMO motivo: a edição do documento mora no
            // bloco que tem o `sim` em mãos, e o drain do barramento não o tem.
            let mut pending_filter_stop: Option<(usize, u8, f32)> = None;
            let mut pending_filter_val: Option<(crate::fx_live::FilterHit, f64)> = None;
            let mut pending_pp_rotation: Option<f64> = None;
            // O Picker de guia (Enio 2026-07-23): o botão só ARMA — a shell captura a fonte e o
            // clique seguinte no canvas escolhe o guia. Um por feature; a fonte é resolvida no drain.
            let mut pending_pp_pick = false;
            let mut pending_text_pick = false;
            let mut pending_expand_envelope = false;
            let mut pending_release_envelope = false;
            // O GESTO do envelope (ADR-0129 Fatias D+E): Perspective (projetivo) · Mesh (Coons) ·
            // Pins (MLS). Um enum e nao um bool desde que o 3o gesto entrou.
            let mut pending_envelope_kind: Option<ph2d_ecs::EnvelopeKind> = None;
            let mut pending_clear_pins = false;
            // O PRESET de gaiola (ADR-0129 Fatia C): indice em `EnvelopeWarp::ALL`, e o Bend.
            let mut pending_envelope_preset: Option<usize> = None;
            let mut pending_envelope_bend: Option<f64> = None;
            // ADR-0132: a pilha de efeitos. Um clique num BOTAO (add/remove/up/down/toggle) e
            // um arrasto num slider -- os dois enderecados por (linha, parametro), sem que este
            // arquivo saiba que efeitos existem.
            let mut pending_fx_add: Option<usize> = None;
            let mut pending_fx_button: Option<(usize, crate::fx_bridge_dispatch::FxRowAction)> =
                None;
            let mut pending_fx_param: Option<(usize, usize, f64)> = None;
            // ADR-0132: o "Apply" assa a pilha de efeitos no cozido e a esvazia (Expand Appearance).
            let mut pending_fx_apply = false;
            // ADR-0108 Fase 1: a Vertex button (Corner/Smooth/Symmetric) retypes
            // the selected vertex — a document edit, applied after the drain.
            let mut pending_vec_vertex_kind: Option<ph2d_vec_scene::VertexKind> = None;
            // ADR-0108 Fase 1: "Delete Node" button removes the selected vertex.
            let mut pending_vec_delete_vertex = false;
            // ADR-0108: Arrange buttons — z-order restack + Duplicate + Flip H/V —
            // act on the selected path (document ops), applied after the drain.
            let mut pending_vec_reorder: Option<ph2d_vec_scene::ZOrder> = None;
            let mut pending_vec_duplicate = false;
            let mut pending_vec_flip: Option<ph2d_vec_scene::FlipAxis> = None;
            let mut pending_vec_rotate: Option<ph2d_vec_scene::Rotate90> = None;
            let mut pending_vec_path_shape: Option<crate::input_dispatch::VecPathShapeOp> = None;
            let mut pending_vec_toggle_closed = false;
            let mut pending_vec_pivot_edit = false;
            let mut pending_vec_fill_kind: Option<crate::input_dispatch::VecFillKind> = None;
            // Linear-gradient angle (degrees) from the Angle slider (track·360).
            let mut pending_vec_grad_angle: Option<f64> = None;
            let mut pending_vec_grad_add = false;
            let mut pending_vec_grad_remove = false;
            // Multi-point Influence slider (track·4).
            let mut pending_vec_grad_influence: Option<f64> = None;
            let mut pending_vec_grad_jitter: Option<f64> = None;
            let mut pending_vec_grad_add_stop = false;
            let mut pending_vec_grad_remove_stop = false;
            let mut pending_vec_align: Option<crate::input_dispatch::VecAlign> = None;
            let mut pending_vec_distribute: Option<crate::input_dispatch::VecDistribute> = None;
            // Make (true) / Release (false) Compound over the selection.
            let mut pending_vec_compound: Option<bool> = None;
            // Fill rule of the selected compound path: even-odd (true) or non-zero.
            let mut pending_vec_fill_rule: Option<bool> = None;
            // Snap section: encaixar em formas (a grade é do painel de Grid).
            let mut pending_vec_snap_on: Option<bool> = None;

            // Numeric Transform field edit (X/Y/W/H) — a SetValue document command.
            let mut pending_vec_transform: Option<(crate::input_dispatch::VecTransformField, f64)> =
                None;
            // Transform Angle field (R) — a relative rotation delta (degrees).
            let mut pending_vec_rotate_by: Option<f64> = None;
            // Slider de parâmetro de forma (Sides/Points/Inner/Radius/Turns/Degrees):
            // `(id, track 0..1)`. A tool já o consome como default de desenho; aqui ele
            // também edita a forma VIVA selecionada (Live Shape).
            let mut pending_vec_shape_param: Option<(ph2d_editor::NodeId, f64)> = None;
            // Campo do CONECTOR (Route / Jetty / Spread): `(id, valor)`. Não é Style da tool
            // — é a RELAÇÃO, que mora no `VecConnector` de cada conector SELECIONADO (todos
            // eles: é assim que se calibra o diagrama inteiro de uma vez).
            let mut pending_vec_connector: Option<(ph2d_editor::NodeId, f64)> = None;
            // Text Size slider (world units) — updates the active session + the
            // size a new session starts at.
            let mut pending_vec_text_size: Option<f64> = None;
            // Text Weight slider (`wght` axis) — updates the active session + the
            // weight a new session starts at.
            let mut pending_vec_text_weight: Option<f32> = None;
            // Paragraph: line-height (× size), tracking (em), and alignment (L/C/R).
            let mut pending_vec_text_line_height: Option<f64> = None;
            let mut pending_vec_text_tracking: Option<f64> = None;
            let mut pending_vec_text_align: Option<ph2d_tool_vector::TextAlign> = None;
            // Variation-axis field edit: (slot index into the font's non-wght axes, value).
            let mut pending_vec_text_axis: Option<(usize, f64)> = None;
            // Text font-family cycle (`<` = -1 / `>` = +1) from the panel picker.
            let mut pending_vec_font_cycle: Option<i32> = None;
            // Font dropdown option pick — index into `vec_font::pickable_families()`.
            let mut pending_vec_font_pick: Option<usize> = None;
            // "Import Font…" button — opens a native picker for a .ttf/.otf.
            let mut pending_vec_font_import = false;
            // "Convert to Curves" — bake the selected live shape(s) into raw paths.
            let mut pending_vec_convert = false;
            let mut transform_edit: Option<ph2d_editor::InspectorTransformInfo> = None;
            let mut visibility_edit: Option<ph2d_editor::InspectorVisibilityInfo> = None;
            let mut sprite_source_change: Option<(u64, RequestedSpriteStrategy)> = None;
            // Sprite field edits (flip/region/sheet/tint/…) — a Vec so a
            // bulk edit that touches several fields in one frame all apply.
            let mut sprite_edits: Vec<(u64, ph2d_editor::SpriteFieldEdit)> = Vec::new();
            // §7 ordering edits (W3) — optional-component edits, fanned out
            // to the selection like sprite edits.
            let mut ordering_edits: Vec<(u64, ph2d_editor::OrderingFieldEdit)> = Vec::new();
            let mut sampling_edits: Vec<(u64, ph2d_editor::SamplingFieldEdit)> = Vec::new();
            let mut blend_edits: Vec<(u64, ph2d_editor::BlendFieldEdit)> = Vec::new();
            let mut physics_edits: Vec<(u64, ph2d_editor::PhysicsFieldEdit)> = Vec::new();
            // §12 joints (W3). Kept out of `inspector_commits::dispatch`: that
            // signature is already the length its own doc-comment warns about,
            // and these two are applied in one short block below.
            let mut joint_edits: Vec<(u64, ph2d_editor::JointFieldEdit)> = Vec::new();
            // The pair to join, at most one per frame — it is a click, not a
            // per-entity edit.
            let mut bake_request: Option<Vec<u64>> = None;
            // W-J4: a rota por SELEÇÃO virou "ligue a sequência" (2 corpos = um
            // joint; N = uma corrente de N−1), então o pedido é um booleano — a
            // ordem vem da própria seleção, que o `join_selected_chain` lê.
            let mut join_chain = false;
            let mut join_draw_arm = false;
            let mut visibility_section_edits: Vec<(u64, ph2d_editor::VisibilityFieldEdit)> =
                Vec::new();
            let mut name_edit: Option<ph2d_editor::InspectorNameInfo> = None;
            let mut bgremoval_leftover: Vec<ph2d_editor::action_bus::EditorAction> = Vec::new();
            // Painter Apply leftover — same shape as bgremoval (drained
            // back into the bus so `image_edit::dispatch`'s
            // `painter_active` gate runs AFTER any same-frame
            // ActivateTool resolution). Day-7 ship.
            let mut painter_leftover: Vec<ph2d_editor::action_bus::EditorAction> = Vec::new();
            // BulkSelect (T2.0): the live selection (primary + extras),
            // captured before the drain so an Inspector sprite edit can
            // fan out to every selected sprite. Only allocated for a
            // MULTI-selection; single-select takes the empty path and the
            // edit's own `entity_bits` (no per-frame alloc — audit D-5).
            let inspector_selection: Vec<u64> = if hero.gizmo.selected_len() > 1 {
                hero.gizmo.iter_selected().collect()
            } else {
                Vec::new()
            };
            for action in hero.bus.drain() {
                use ph2d_editor::action_bus::EditorAction;
                match action {
                    // ADR-0040 TG-A: generic activation. Per-tool flags
                    // preserve the existing mode_on gating / activation
                    // side effects after the drain.
                    // ADR-0040 TG-A: generic activation. Audit F1 (2026-05-26):
                    // data-driven via cluster lookup no drain abaixo; sem
                    // per-tool flag flooding.
                    EditorAction::ActivateTool { tool_id } => {
                        pending_image_tool_activation = Some(tool_id);
                    }
                    // ADR-0040 TG-B: generic panel→tool channel. Route the
                    // event to the active tool's `handle_panel_event` —
                    // semantic mapping (slider id → typed UI edit) lives on
                    // the tool, not here.
                    EditorAction::ToolPanelEvent(ev) => {
                        // Vector Boolean + Vertex buttons are DOCUMENT commands,
                        // not Style edits — capture them (by ref, PanelEvent isn't
                        // Copy) to apply after the drain; still forward to the tool
                        // (which ignores those ids) so mode/width/etc. flow.
                        if let ph2d_editor::tool::PanelEvent::Click(id) = &ev {
                            if *id == ph2d_editor::ids::VECTOR_BLEND_RUN {
                                // ADR-0128: cria o Blend Object VIVO da seleção (não o destrutivo).
                                pending_create_blend = true;
                            } else if *id == ph2d_editor::ids::VECTOR_BLEND_RESET_SPINE {
                                // ADR-0128 C2b: volta o spine editado ao automático.
                                pending_reset_spine = true;
                            } else if *id == ph2d_editor::ids::VECTOR_BLEND_EXPAND {
                                // ADR-0128 D: materializa os passos e descarta o objeto vivo.
                                pending_expand_blend = true;
                            } else if *id == ph2d_editor::ids::VECTOR_BLEND_RELEASE {
                                // ADR-0128 D: desfaz o blend; as fontes ficam.
                                pending_release_blend = true;
                            } else if *id == ph2d_editor::ids::VECTOR_MORPH_RUN {
                                // O irmão animável do blend: UMA forma, com o `t` keyável.
                                pending_create_morph = true;
                            } else if *id == ph2d_editor::ids::VECTOR_ENVELOPE_RUN {
                                // ADR-0129: envolve a seleção (1..N) num container com gaiola.
                                pending_create_envelope = true;
                            } else if *id == ph2d_editor::ids::VECTOR_ENVELOPE_EXPAND {
                                // ADR-0129: a deformada vira o desenho; a gaiola morre.
                                pending_expand_envelope = true;
                            } else if *id == ph2d_editor::ids::VECTOR_ENVELOPE_RELEASE {
                                // ADR-0129: a fonte autorada volta; a gaiola morre.
                                pending_release_envelope = true;
                            } else if *id == ph2d_editor::ids::VECTOR_ENVELOPE_PERSPECTIVE {
                                // ADR-0129 Fatia D: a homografia -- lados RETOS.
                                pending_envelope_kind = Some(ph2d_ecs::EnvelopeKind::Perspective);
                            } else if *id == ph2d_editor::ids::VECTOR_ENVELOPE_MESH {
                                // ADR-0129 Fatia D: o patch de Coons -- os lados DOBRAM.
                                pending_envelope_kind = Some(ph2d_ecs::EnvelopeKind::Mesh);
                            } else if *id == ph2d_editor::ids::VECTOR_ENVELOPE_PINS {
                                // ADR-0129 Fatia E: o puppet warp (MLS-rigid).
                                pending_envelope_kind = Some(ph2d_ecs::EnvelopeKind::Pins);
                            } else if *id == ph2d_editor::ids::VECTOR_TEXTPATH_LINK {
                                // Plano 22: prende o texto da seleção à outra forma dela.
                                pending_textpath = Some(crate::vec_text_ride::TextPathCmd::Link);
                            } else if *id == ph2d_editor::ids::VECTOR_TEXTPATH_PICK {
                                // Picker: arma; a fonte (o texto em foco) é capturada no drain.
                                pending_text_pick = true;
                            } else if *id == ph2d_editor::ids::VECTOR_TEXTPATH_DETACH {
                                pending_textpath = Some(crate::vec_text_ride::TextPathCmd::Detach);
                            } else if *id == ph2d_editor::ids::VECTOR_TEXTPATH_FLIP {
                                pending_textpath =
                                    Some(crate::vec_text_ride::TextPathCmd::Flip(true));
                            } else if *id == ph2d_editor::ids::VECTOR_TEXTPATH_FLIP_OFF {
                                pending_textpath =
                                    Some(crate::vec_text_ride::TextPathCmd::Flip(false));
                            } else if *id == ph2d_editor::ids::VECTOR_PATTERNPATH_LINK {
                                pending_patternpath =
                                    Some(crate::pattern_live::PatternPathCmd::Link);
                            } else if *id == ph2d_editor::ids::VECTOR_PATTERNPATH_PICK {
                                // Picker: arma; a fonte (o motivo selecionado) é capturada no drain.
                                pending_pp_pick = true;
                            } else if *id == ph2d_editor::ids::VECTOR_PATTERNPATH_DETACH {
                                pending_patternpath =
                                    Some(crate::pattern_live::PatternPathCmd::Detach);
                            } else if *id == ph2d_editor::ids::VECTOR_PATTERNPATH_FLIP {
                                pending_patternpath =
                                    Some(crate::pattern_live::PatternPathCmd::Flip(true));
                            } else if *id == ph2d_editor::ids::VECTOR_PATTERNPATH_FLIP_OFF {
                                pending_patternpath =
                                    Some(crate::pattern_live::PatternPathCmd::Flip(false));
                            } else if *id == ph2d_editor::ids::VECTOR_CONTOUR_ADD {
                                pending_contour = Some(crate::contour_live::ContourCmd::Add);
                            } else if *id == ph2d_editor::ids::VECTOR_CONTOUR_REMOVE {
                                pending_contour = Some(crate::contour_live::ContourCmd::Remove);
                            } else if *id == ph2d_editor::ids::VECTOR_CONTOUR_EXPAND {
                                pending_contour = Some(crate::contour_live::ContourCmd::Expand);
                            } else if let Some(code) = crate::contour_live::join_code_of_id(*id) {
                                pending_contour_join = Some(code);
                            } else if let Some(code) = crate::contour_live::side_code_of_id(*id) {
                                pending_contour_side = Some(code);
                            } else if let Some(hit) = crate::fx_live::hit_of(*id) {
                                pending_filter_cmd = Some(hit);
                            } else if let Some(hit) = crate::fx_bridge_dispatch::classify_click(*id)
                            {
                                match hit {
                                    crate::fx_bridge_dispatch::FxClick::Add(k) => {
                                        pending_fx_add = Some(k);
                                    }
                                    crate::fx_bridge_dispatch::FxClick::Row(r, a) => {
                                        pending_fx_button = Some((r, a));
                                    }
                                    crate::fx_bridge_dispatch::FxClick::Apply => {
                                        pending_fx_apply = true;
                                    }
                                }
                            } else if *id == ph2d_editor::ids::VECTOR_ENVELOPE_CLEAR_PINS {
                                pending_clear_pins = true;
                            } else if let Some(i) = (0..ph2d_editor::ids::MAX_ENVELOPE_PRESETS)
                                .find(|&i| *id == ph2d_editor::ids::vector_envelope_preset_id(i))
                            {
                                // ADR-0129 Fatia C: carimba o preset `i` na gaiola.
                                pending_envelope_preset = Some(i);
                            } else if let Some(i) = (0..ph2d_editor::ids::MAX_WIDTH_PRESETS)
                                .find(|&i| *id == ph2d_editor::ids::vector_width_preset_id(i))
                            {
                                // W2b: escolhe a FORMA da largura (o catálogo de perfis).
                                pending_width_preset = Some(i);
                            } else if let Some(op) = crate::input_dispatch::vec_bool_op_for_id(*id)
                            {
                                pending_vec_bool = Some(op);
                            } else if let Some(cmd) = crate::vec_expand::expand_for_id(*id) {
                                pending_vec_expand = Some(cmd);
                            } else if let Some(kind) =
                                crate::input_dispatch::vec_vertex_kind_for_id(*id)
                            {
                                pending_vec_vertex_kind = Some(kind);
                            } else if *id == ph2d_editor::ids::VECTOR_VERT_DELETE {
                                pending_vec_delete_vertex = true;
                            } else if *id == ph2d_editor::ids::VECTOR_VERT_SEL_SUBPATH {
                                pending_vec_select_subpath = true;
                            } else if *id == ph2d_editor::ids::VECTOR_VERT_SEL_SAME {
                                pending_vec_select_same = true;
                            } else if let Some(order) =
                                crate::input_dispatch::vec_reorder_for_id(*id)
                            {
                                pending_vec_reorder = Some(order);
                            } else if *id == ph2d_editor::ids::VECTOR_ARRANGE_DUPLICATE {
                                pending_vec_duplicate = true;
                            } else if let Some(axis) = crate::input_dispatch::vec_flip_for_id(*id) {
                                pending_vec_flip = Some(axis);
                            } else if let Some(dir) = crate::input_dispatch::vec_rotate_for_id(*id)
                            {
                                pending_vec_rotate = Some(dir);
                            } else if let Some(op) =
                                crate::input_dispatch::vec_path_shape_for_id(*id)
                            {
                                pending_vec_path_shape = Some(op);
                            } else if *id == ph2d_editor::ids::VECTOR_PIVOT_EDIT {
                                pending_vec_pivot_edit = true;
                            } else if *id == ph2d_editor::ids::VECTOR_PATH_CLOSE {
                                pending_vec_toggle_closed = true;
                            } else if let Some(k) = crate::input_dispatch::vec_fill_kind_for_id(*id)
                            {
                                pending_vec_fill_kind = Some(k);
                            } else if *id == ph2d_editor::ids::VECTOR_GRAD_ADD_POINT {
                                pending_vec_grad_add = true;
                            } else if *id == ph2d_editor::ids::VECTOR_GRAD_REMOVE_POINT {
                                pending_vec_grad_remove = true;
                            } else if *id == ph2d_editor::ids::VECTOR_GRAD_ADD_STOP {
                                pending_vec_grad_add_stop = true;
                            } else if *id == ph2d_editor::ids::VECTOR_GRAD_REMOVE_STOP {
                                pending_vec_grad_remove_stop = true;
                            } else if let Some(a) = crate::input_dispatch::vec_align_for_id(*id) {
                                pending_vec_align = Some(a);
                            } else if let Some(d) =
                                crate::input_dispatch::vec_distribute_for_id(*id)
                            {
                                pending_vec_distribute = Some(d);
                            } else if *id == ph2d_editor::ids::VECTOR_COMPOUND_MAKE {
                                pending_vec_compound = Some(true);
                            } else if *id == ph2d_editor::ids::VECTOR_COMPOUND_RELEASE {
                                pending_vec_compound = Some(false);
                            } else if *id == ph2d_editor::ids::VECTOR_FILL_RULE_NONZERO {
                                pending_vec_fill_rule = Some(false);
                            } else if *id == ph2d_editor::ids::VECTOR_FILL_RULE_EVENODD {
                                pending_vec_fill_rule = Some(true);
                            } else if *id == ph2d_editor::ids::VECTOR_SNAP_OFF {
                                pending_vec_snap_on = Some(false);
                            } else if *id == ph2d_editor::ids::VECTOR_SNAP_ON {
                                pending_vec_snap_on = Some(true);
                            } else if *id == ph2d_editor::ids::VECTOR_TEXT_FONT_PREV {
                                pending_vec_font_cycle = Some(-1);
                            } else if *id == ph2d_editor::ids::VECTOR_TEXT_FONT_NEXT {
                                pending_vec_font_cycle = Some(1);
                            } else if *id == ph2d_editor::ids::VECTOR_TEXT_FONT_IMPORT {
                                pending_vec_font_import = true;
                            } else if *id == ph2d_editor::ids::VECTOR_TEXT_ALIGN_LEFT {
                                pending_vec_text_align = Some(ph2d_tool_vector::TextAlign::Left);
                            } else if *id == ph2d_editor::ids::VECTOR_TEXT_ALIGN_CENTER {
                                pending_vec_text_align = Some(ph2d_tool_vector::TextAlign::Center);
                            } else if *id == ph2d_editor::ids::VECTOR_TEXT_ALIGN_RIGHT {
                                pending_vec_text_align = Some(ph2d_tool_vector::TextAlign::Right);
                            } else if *id == ph2d_editor::ids::VECTOR_CONVERT_TO_CURVES {
                                pending_vec_convert = true;
                            }
                        }
                        // Transform fields (X/Y/W/H) are numeric SetValue document
                        // commands (not tool Style) — capture; the tool ignores them.
                        if let ph2d_editor::tool::PanelEvent::SetValue(id, v) = &ev {
                            if let Some(field) =
                                crate::input_dispatch::vec_transform_field_for_id(*id)
                            {
                                pending_vec_transform = Some((field, *v));
                            } else if *id == ph2d_editor::ids::VECTOR_TRANSFORM_R {
                                pending_vec_rotate_by = Some(*v);
                            } else if *id == ph2d_editor::ids::VECTOR_GRAD_ANGLE {
                                // Slider carries the track 0..1 → 0..360°.
                                pending_vec_grad_angle = Some(*v * 360.0);
                            } else if *id == ph2d_editor::ids::VECTOR_GRAD_INFLUENCE {
                                // Track 0..1 → influence 0..4.
                                pending_vec_grad_influence = Some(*v * 4.0);
                            } else if *id == ph2d_editor::ids::VECTOR_GRAD_JITTER {
                                // Track 0..1 → jitter 0..1 (already a fraction).
                                pending_vec_grad_jitter = Some(*v);
                            } else if *id == ph2d_editor::ids::VECTOR_TEXT_SIZE {
                                // Track 0..1 → glyph size (world units); shared mapping.
                                pending_vec_text_size =
                                    Some(ph2d_tool_vector::params::slider_to_text_size(*v as f32));
                            } else if *id == ph2d_editor::ids::VECTOR_TEXT_WEIGHT {
                                // Track 0..1 → font weight (wght); shared mapping.
                                pending_vec_text_weight = Some(
                                    ph2d_tool_vector::params::slider_to_text_weight(*v as f32)
                                        as f32,
                                );
                            } else if *id == ph2d_editor::ids::VECTOR_TEXT_LINE_HEIGHT {
                                // Track 0..1 → line height (× size); shared mapping.
                                pending_vec_text_line_height = Some(
                                    ph2d_tool_vector::params::slider_to_text_line_height(*v as f32),
                                );
                            } else if *id == ph2d_editor::ids::VECTOR_TEXT_TRACKING {
                                // Track 0..1 → tracking (em fraction); shared mapping.
                                pending_vec_text_tracking = Some(
                                    ph2d_tool_vector::params::slider_to_text_tracking(*v as f32),
                                );
                            } else if crate::vec_connector_panel::is_connector_field_id(*id) {
                                // Os três campos do conector: a shell os aplica em TODOS os
                                // conectores selecionados (a tool os ignora — não são Style).
                                pending_vec_connector = Some((*id, *v));
                            } else if crate::vec_shape_params::is_shape_field_id(*id) {
                                // Sliders de forma: a tool os toma como default de
                                // desenho (abaixo, no forward) E eles editam a forma
                                // VIVA selecionada — o track cru vai junto, porque a
                                // conversão depende da variante da forma.
                                pending_vec_shape_param = Some((*id, *v));
                            } else if *id == ph2d_editor::ids::VECTOR_BLEND_STEPS {
                                // ADR-0128: arrastar Steps ajusta o blend selecionado AO VIVO.
                                pending_blend_steps =
                                    Some(ph2d_tool_vector::params::blend_steps_from_track(*v));
                            } else if *id == ph2d_editor::ids::VECTOR_TEXTPATH_OFFSET {
                                // Plano 22: FRAÇÃO do comprimento do caminho, ja' no dominio do
                                // documento (o painel nao converte -- track e valor coincidem).
                                pending_textpath_offset = Some(*v);
                            } else if *id == ph2d_editor::ids::VECTOR_PATTERNPATH_SPACING {
                                // Plano 23: ja' convertido pelo event.rs do painel para o dominio do
                                // documento (multiplos da largura do motivo) -- aqui e' valor.
                                pending_pp_spacing = Some(*v);
                            } else if *id == ph2d_editor::ids::VECTOR_PATTERNPATH_START {
                                // FRAÇÃO do comprimento (track == valor, como o Offset do texto).
                                pending_pp_start = Some(*v);
                            } else if *id == ph2d_editor::ids::VECTOR_PATTERNPATH_END {
                                // FRAÇÃO do comprimento -- o fim do trecho `[Start, End]`.
                                pending_pp_end = Some(*v);
                            } else if *id == ph2d_editor::ids::VECTOR_PATTERNPATH_SLIDE {
                                // O CENTRO do trecho -- o drain re-centra a janela (move Start+End).
                                pending_pp_slide = Some(*v);
                            } else if *id == ph2d_editor::ids::VECTOR_PATTERNPATH_ROTATION {
                                // A ORIENTAÇÃO do motivo sobre a guia, em GRAUS -- o event.rs do painel
                                // ja' converteu o track bipolar (`-180..180`); aqui e' valor.
                                pending_pp_rotation = Some(*v);
                            } else if *id == ph2d_editor::ids::VECTOR_PATTERNPATH_OFFSET {
                                // Desvio perpendicular (unidades de mundo), ja' bipolar (`-2..2`)
                                // convertido pelo event.rs do painel -- aqui e' valor.
                                pending_pp_offset = Some(*v);
                            } else if *id == ph2d_editor::ids::VECTOR_CONTOUR_STEPS {
                                // Quantos aneis -- o `event.rs` do painel ja arredondou ao inteiro.
                                pending_contour_steps = Some(*v);
                            } else if *id == ph2d_editor::ids::VECTOR_CONTOUR_OFFSET {
                                // A distancia POR PASSO, em FRACAO do tamanho da forma: o painel
                                // fala fracao (um rotulo em unidades de mundo mentiria a cada troca
                                // de selecao) e o componente guarda MUNDO. A conversao e' do `arm`
                                // e do drain, com a MESMA `offset_scale` que o Offset usa.
                                pending_contour_d = Some(*v);
                            } else if *id == ph2d_editor::ids::VECTOR_CONTOUR_ACCEL {
                                // A aceleracao da progressao -- o painel ja aplicou o mapa
                                // GEOMETRICO do trilho; aqui e' valor.
                                pending_contour_accel = Some(*v);
                            } else if let Some(hit) = crate::fx_live::hit_of(*id) {
                                pending_filter_val = Some((hit, *v));
                            } else if *id == ph2d_editor::ids::VECTOR_ENVELOPE_BEND {
                                // ADR-0129 Fatia C: o `event.rs` do painel ja converteu o track
                                // bipolar para o dominio do documento (`-1..1`) -- aqui e' valor.
                                pending_envelope_bend = Some(*v);
                            } else if let Some((r, prm)) =
                                crate::fx_bridge_dispatch::classify_param(*id)
                            {
                                // O painel entrega o TRACK normalizado; a faixa real e' do
                                // efeito e a ponte a aplica.
                                pending_fx_param = Some((r, prm, *v));
                            } else if *id == ph2d_editor::ids::VECTOR_MORPH_T {
                                // Arrastar o `t` move a forma pelo caminho AO VIVO — e é assim que
                                // o artista a estaciona onde ela fica bem, antes do K.
                                #[allow(clippy::cast_possible_truncation)]
                                let t = *v as f32;
                                pending_morph_t = Some(t);
                            } else {
                                // Variation-axis field carries the axis VALUE directly
                                // (not a 0..1 track): match the slot to its font axis.
                                for i in 0..ph2d_editor::ids::MAX_TEXT_VARIATION_AXES {
                                    if *id == ph2d_editor::ids::vector_text_axis_id(i) {
                                        pending_vec_text_axis = Some((i, *v));
                                        break;
                                    }
                                }
                            }
                        }
                        // Font dropdown pick: `SelectOption(chip, "<index>")` → the
                        // family index into `vec_font::pickable_families()`.
                        if let ph2d_editor::tool::PanelEvent::SelectOption(id, val) = &ev
                            && *id == ph2d_editor::ids::VECTOR_TEXT_FONT_DD
                        {
                            pending_vec_font_pick = val.parse::<usize>().ok();
                        }
                        // O punho de um stop da rampa: `SelectOption(trilho, "linha:idx:x")` — o
                        // dispatch de 2D já converteu o ponteiro contra a barra, então o `x` chega
                        // normalizado. O formato espelha o do editor de falloff do Painter.
                        if let ph2d_editor::tool::PanelEvent::SelectOption(id, val) = &ev
                            && (0..ph2d_editor::ids::MAX_FILTER_ROWS)
                                .any(|r| *id == ph2d_editor::ids::filter_ramp_id(r))
                        {
                            let mut parts = val.split(':');
                            if let (Some(Ok(row)), Some(Ok(idx)), Some(Ok(x))) = (
                                parts.next().map(str::parse::<usize>),
                                parts.next().map(str::parse::<u8>),
                                parts.next().map(str::parse::<f32>),
                            ) {
                                pending_filter_stop = Some((row, idx, x));
                                // ⚠️ **Diagnóstico de UM elo, atrás de env.** O report *"não é
                                // possível arrastar os pontos de cor"* não reproduz headless — o
                                // gate de seam dirige o gesto REAL e chega ao barramento —, então o
                                // que falta medir é o que só o app vivo tem: se esta linha imprime,
                                // o painel entregou e o defeito está a jusante; se não imprime, o
                                // evento nunca chegou (e o `[hero] unhandled event` o dirá).
                                if std::env::var_os("PH2D_FX_RAMP_DIAG").is_some() {
                                    eprintln!(
                                        "[ramp] painel entregou: linha {row} stop {idx} -> x {x:.4}"
                                    );
                                }
                            }
                        }
                        // ADR-0114 C2: Colorize Apply/Clear — mexem no buffer de rabiscos do
                        // shell + no doc, e o `self.gfx` está preso pelo borrow deste bloco;
                        // marca-se um pending no `self` (campo disjunto) e aplica-se no topo
                        // do PRÓXIMO frame, com `self` livre (latência de 1 frame, imperceptível
                        // num botão).
                        if let ph2d_editor::tool::PanelEvent::Click(id) = &ev {
                            if *id == ph2d_editor::ids::FLIP_COLORIZE_APPLY {
                                self.pending_flip_colorize_apply = true;
                            } else if *id == ph2d_editor::ids::FLIP_COLORIZE_CLEAR {
                                self.pending_flip_colorize_clear = true;
                            }
                        }
                        // ADR-0114 W2: Flip layer ops (add/delete/select/visibility/
                        // lock/reorder/opacity/blend) are DOCUMENT edits — apply to
                        // `gfx.flip` + the active-layer pointer (mirror of the vector
                        // Boolean/Arrange capture). No-op for non-Flip ids. Still
                        // forward `ev` to the tool below (it ignores layer ids).
                        crate::flip_layers::apply_panel_event(
                            &ev,
                            flip,
                            &mut self.flip_active_layer,
                            &self.playhead,
                            matches!(
                                self.flip_style.map(|s| s.edit_domain),
                                Some(ph2d_tool_flip::EditDomain::Point)
                            ),
                        );
                        // ADR-0114 W3: e os eventos da TIRA (transporte, ops de
                        // chave, exposição, tween, ciclo, Ghost Frames) — documento
                        // + playhead, aplicados aqui pelo mesmo drain.
                        // O `add` (Shift/Ctrl) vem do SHELL, não do evento: o
                        // `WidgetEvent::Click` não carrega modificadores e o `PanelEvent`
                        // está CONGELADO em 4 variantes (ADR-0040). O drain roda no MESMO
                        // frame do clique, então o estado da tecla ainda é o do gesto — e
                        // nenhum contrato precisa ser tocado para a tira ganhar
                        // multisseleção (W7).
                        crate::flip_strip::apply_panel_event(
                            &ev,
                            flip,
                            self.flip_active_layer,
                            &mut self.playhead,
                            &mut self.flip_strip,
                            self.modifiers.shift_key()
                                || self.modifiers.super_key()
                                || self.modifiers.control_key(),
                        );
                        if let Some(t) = tools.active_mut() {
                            t.handle_panel_event(ev);
                        }
                    }
                    // docs/Timeline W2.E2: the docked timeline panel is not a
                    // tool — translate its transport PanelEvents into
                    // `TimelineIntent`s (id → intent; the timeline semantics live
                    // here, editor-core stays timeline-agnostic) and queue them
                    // for `timeline_bridge::run` to apply this frame.
                    EditorAction::TimelinePanelEvent(ev) => {
                        // "+Track <prop>" binds the selected sprite's property
                        // (the panel doesn't know the selection; the shell does).
                        if let ph2d_editor::tool::PanelEvent::Click(id) = &ev
                            && let Some(prop) = timeline_bridge::prop_for_addprop_id(*id)
                        {
                            if let Some(entity) = hero.gizmo.iter_selected().next() {
                                self.timeline_intents
                                    .push(ph2d_timeline::TimelineIntent::Bind { entity, prop });
                            }
                        } else if let ph2d_editor::tool::PanelEvent::Toggle(id, on) = &ev
                            && *id == ph2d_editor::ids::TIMELINE_MOTION_PATH
                        {
                            // The Motion Path toggle is PER OBJECT (like +Track, the
                            // panel doesn't know the selection): convert THIS object's
                            // position to a trajectory (`on`) or separate X/Y — Convert
                            // to Motion Path / to Separate Axes (ADR-0141).
                            if let Some(entity) = hero.gizmo.iter_selected().next() {
                                self.timeline_intents.push(
                                    ph2d_timeline::TimelineIntent::ConvertPositionMode {
                                        entity,
                                        to_path: *on,
                                    },
                                );
                            }
                        } else if let ph2d_editor::tool::PanelEvent::Click(id) = &ev
                            && *id == ph2d_editor::ids::TIMELINE_ONION_SETTINGS
                        {
                            // Open the onion settings card (hero chrome), seeded from the current
                            // onion. Shell-side because the card lives in `hero.store`, out of the
                            // panel's reach (mirror of the Motion Path case above). `OnionSettings`
                            // is `Copy`, so this holds no borrow while `hero.store` is written; the
                            // count↔slider + rgb↔u8 mappings live in `crate::onion_modal`.
                            let o = self.timeline.onion;
                            let (ax, ay) = hero
                                .hit_index
                                .rect_for(*id)
                                .map_or((120.0, 120.0), |r| (r.x - 90.0, r.y - 236.0));
                            hero.store.open_onion_modal(
                                ax,
                                ay,
                                o.opacity,
                                crate::onion_modal::count_to_frac(o.frames_before),
                                crate::onion_modal::count_to_frac(o.frames_after),
                                crate::onion_modal::rgb_to_u8(o.color_before),
                                crate::onion_modal::rgb_to_u8(o.color_after),
                            );
                        } else if let Some(intent) = timeline_bridge::intent_for_transport(
                            &ev,
                            &self.timeline,
                            &self.playhead,
                        ) {
                            self.timeline_intents.push(intent);
                            // A jump to an absolute time may land outside the
                            // visible span; pan the dope sheet after it (the
                            // panel page-follows only while playing). Deferred
                            // to the apply — see `timeline_reveal_after_apply`.
                            self.timeline_reveal_after_apply |=
                                timeline_bridge::jumps_the_playhead(&ev);
                        }
                    }
                    // ADR-0040 TG-B/TG-C: generic "cancel the active modal
                    // tool". Switch back to the default tool and tear down
                    // any image-tool shell-side preview caches. Bg Removal +
                    // Padding panels both raise this; the bgremoval cleanup
                    // is a no-op when padding (or any non-bgremoval tool)
                    // was active. Padding's shell-side state is purely
                    // tool-internal (no shell-cached preview), so no
                    // padding-specific cleanup is needed here.
                    EditorAction::CancelActiveTool => {
                        // ADR-0108: end any in-progress Vector draw cleanly when
                        // the tool is toggled off. The Pen lives on the shell, so
                        // the partial path PERSISTS in `vec_scene` (open) — no
                        // discard, no warning; `finish` just leaves drawing mode
                        // (a cheap no-op for any other tool being cancelled).
                        self.vec_pen.finish();
                        if let Some(default_id) = tools.default_tool_id()
                            && tools.set_active(&default_id)
                        {
                            self.last_bgremoval_pushed_entity = None;
                            self.bgremoval_preview = None;
                            self.title_dirty = true;
                        }
                    }
                    EditorAction::UndoImageEdit => undo_image_edit = true,
                    // Os botões Undo/Redo da barra: MESMO caminho do Ctrl+Z. O despacho
                    // espera o fim do frame (`post_frame_undo`) porque `undo_or_redo`
                    // precisa de `&mut self` e o `gfx` está emprestado aqui.
                    EditorAction::UndoStep { redo } => self.undo_button = Some(redo),
                    EditorAction::HierToggleVisibility { row } => {
                        visibility_toggle_row.get_or_insert(row);
                    }
                    EditorAction::HierToggleLock { row } => {
                        lock_toggle_row.get_or_insert(row);
                    }
                    EditorAction::HierToggleGroup { row } => {
                        group_toggle_row.get_or_insert(row);
                    }
                    EditorAction::HierReparent(intent) => {
                        reparent_intent.get_or_insert(intent);
                    }
                    EditorAction::HierDuplicate { row } => {
                        duplicate_row.get_or_insert(row);
                    }
                    EditorAction::HierAddChild { row } => {
                        add_child_row.get_or_insert(row);
                    }
                    EditorAction::HierResetTransform { row } => {
                        reset_transform_row.get_or_insert(row);
                    }
                    EditorAction::HierDelete { row } => {
                        delete_row.get_or_insert(row);
                    }
                    EditorAction::HierMergeSprites { row } => {
                        merge_sprites_row.get_or_insert(row);
                    }
                    EditorAction::HierUseAsBrushTexture { row } => {
                        use_as_brush_texture_row.get_or_insert(row);
                    }
                    EditorAction::HierUseAsBrushShape { row } => {
                        use_as_brush_shape_row.get_or_insert(row);
                    }
                    EditorAction::HierUseAsPaper { row } => {
                        use_as_paper_row.get_or_insert(row);
                    }
                    EditorAction::HierUseAsGranulation { row } => {
                        use_as_granulation_row.get_or_insert(row);
                    }
                    EditorAction::HierRowClick { row } => {
                        hierarchy_row_click.get_or_insert(row);
                    }
                    // Fase 0e: multi-select-aware hierarchy click +
                    // shift-range. Collect into a single latest-wins
                    // intent — the dispatch resolves row → entity_bits
                    // and applies the matching `GizmoStateGroup`
                    // mutation. Range overrides Row when both arrive
                    // in the same frame (the user can only be in one
                    // selection-gesture at a time).
                    EditorAction::HierSelectRow { row, modifier }
                        if !matches!(
                            hierarchy_select_intent,
                            Some(hierarchy::HierarchySelectIntent::Range { .. })
                        ) =>
                    {
                        hierarchy_select_intent =
                            Some(hierarchy::HierarchySelectIntent::Row { row, modifier });
                    }
                    EditorAction::HierRangeSelect { row } => {
                        hierarchy_select_intent =
                            Some(hierarchy::HierarchySelectIntent::Range { row });
                    }
                    // Fase 0e: canvas-side select via the bus (reserved
                    // for callers that don't have direct hero access —
                    // input_dispatch.rs:435 mutates hero.gizmo directly
                    // because it already holds the borrow).
                    EditorAction::SelectSprite {
                        entity_bits,
                        modifier,
                    } => match modifier {
                        ph2d_editor::action_bus::SelectModifier::Replace => {
                            hero.gizmo.replace_selection(Some(entity_bits));
                        }
                        ph2d_editor::action_bus::SelectModifier::Add => {
                            hero.gizmo.add_to_selection(entity_bits);
                        }
                        ph2d_editor::action_bus::SelectModifier::Toggle => {
                            hero.gizmo.toggle_in_selection(entity_bits);
                        }
                    },
                    EditorAction::ClearSelection => {
                        hero.gizmo.clear_all_selection();
                    }
                    EditorAction::HierRenameSeed { row } => {
                        rename_seed_row.get_or_insert(row);
                    }
                    EditorAction::HierRenameCommit { row, new_name } if rename_commit.is_none() => {
                        rename_commit = Some((row, new_name));
                    }
                    EditorAction::SetViewFocus { kind } => {
                        view_focus_kind.get_or_insert(kind);
                    }
                    EditorAction::Reimport { entity_bits } => {
                        reimport_entity.get_or_insert(entity_bits);
                    }
                    // ADR-0040 TG-A: generic one-shot image-op dispatch.
                    // Trim/MakeSquare/RealSize collect into per-tool Option<u64>
                    // for the existing per-tool drain functions; bgremoval bake
                    // is deferred via leftover (must run AFTER ActivateTool
                    // has switched the tool active, image_edit.rs:184 picks it up).
                    oneshot @ EditorAction::OneShotImageOp {
                        tool_id,
                        entity_bits,
                    } => match tool_id {
                        "trim_transparency" => {
                            trim_entities.push(entity_bits);
                        }
                        "make_square" => {
                            make_square_entities.push(entity_bits);
                        }
                        "real_size" => {
                            real_size_entities.push(entity_bits);
                        }
                        "rasterize" => {
                            rasterize_entities.push(entity_bits);
                        }
                        "bgremoval" => {
                            bgremoval_leftover.push(oneshot);
                        }
                        "painter" => {
                            painter_leftover.push(oneshot);
                        }
                        _ => {}
                    },
                    EditorAction::InspectorTransformEdit(info) => {
                        transform_edit.get_or_insert(info);
                    }
                    EditorAction::InspectorVisibilityEdit(info) => {
                        visibility_edit.get_or_insert(info);
                    }
                    EditorAction::InspectorSpriteSourceChange {
                        entity_bits,
                        strategy,
                    } => {
                        sprite_source_change.get_or_insert((entity_bits, strategy));
                    }
                    EditorAction::InspectorSpriteEdit { entity_bits, edit } => {
                        // BulkSelect: apply to EVERY selected sprite, not
                        // just the dispatching (primary) entity. The Vec
                        // includes the primary first; single-select pushes
                        // one. Fall back to the edit's own entity if the
                        // selection snapshot is empty (stale dispatch).
                        if inspector_selection.is_empty() {
                            sprite_edits.push((entity_bits, edit));
                        } else {
                            for &t in &inspector_selection {
                                sprite_edits.push((t, edit));
                            }
                        }
                    }
                    EditorAction::InspectorOrderingEdit { entity_bits, edit } => {
                        // BulkSelect fan-out, same shape as the sprite edit.
                        if inspector_selection.is_empty() {
                            ordering_edits.push((entity_bits, edit));
                        } else {
                            for &t in &inspector_selection {
                                ordering_edits.push((t, edit));
                            }
                        }
                    }
                    EditorAction::InspectorSamplingEdit { entity_bits, edit } => {
                        if inspector_selection.is_empty() {
                            sampling_edits.push((entity_bits, edit));
                        } else {
                            for &t in &inspector_selection {
                                sampling_edits.push((t, edit));
                            }
                        }
                    }
                    EditorAction::InspectorBlendEdit { entity_bits, edit } => {
                        if inspector_selection.is_empty() {
                            blend_edits.push((entity_bits, edit));
                        } else {
                            for &t in &inspector_selection {
                                blend_edits.push((t, edit));
                            }
                        }
                    }
                    // §11 Physics Body. Fans out over a BulkSelect like its
                    // siblings — "make all of these physical" is the gesture
                    // an artist actually performs.
                    EditorAction::InspectorPhysicsEdit { entity_bits, edit } => {
                        // ⚠️ **Join does NOT fan out.** Every other §11 edit is
                        // per-entity ("make all of these static"), but joining
                        // is one gesture over a PAIR — fanned out it would
                        // create one joint per selected body, i.e. two joints
                        // between the same two objects, on the very click that
                        // is supposed to make one.
                        if matches!(edit, ph2d_editor::PhysicsFieldEdit::Join) {
                            // ⚠️ **2 ou MAIS** (W-J4): três corpos marcados
                            // fazem uma CORRENTE de N−1 joints, na ordem da
                            // seleção. Não é fan-out (isso criaria um joint por
                            // corpo, entre os mesmos dois) — é UMA operação
                            // sobre a sequência, que a `join_selected_chain`
                            // executa depois do laço.
                            if inspector_selection.len() >= 2 {
                                join_chain = true;
                            }
                        } else if matches!(edit, ph2d_editor::PhysicsFieldEdit::JoinDraw) {
                            // ARMA o gesto de canvas (sem operando, como os
                            // eyedroppers do §12): quem nomeia os dois corpos é
                            // o press e o release, não a seleção. Armado aqui e
                            // honrado no `input_dispatch`.
                            join_draw_arm = true;
                        } else if matches!(edit, ph2d_editor::PhysicsFieldEdit::Bake) {
                            // WARNING: **Bake does not fan out either**, and the
                            // cost of getting it wrong is bigger than Join's:
                            // ONE bake runs the whole simulation once and writes
                            // every selected body's curves from that single run.
                            // Fanned out it would re-simulate the entire scene
                            // once per selected body - same numbers, N times the
                            // work - and file a separate undo step for each, so
                            // undoing "the bake" would take as many Ctrl+Z
                            // presses as there were objects.
                            bake_request = Some(if inspector_selection.is_empty() {
                                vec![entity_bits]
                            } else {
                                inspector_selection.clone()
                            });
                        } else if let ph2d_editor::PhysicsFieldEdit::BakeChannels(tag) = edit {
                            // A GLOBAL bake option, not a per-body edit (like
                            // Bake itself): it says how the NEXT bake behaves.
                            // No fan-out, no Collider write — just the app state
                            // the Bake button reads.
                            self.bake_channels = physics_bake::BakeChannels::from_tag(tag);
                        } else if let ph2d_editor::PhysicsFieldEdit::JoinKind(tag) = edit {
                            // The pending join KIND, the same class as BakeChannels:
                            // an app-state option the Join gesture reads, not a
                            // per-body edit. No fan-out, no Collider write.
                            self.join_kind = tag;
                        } else if inspector_selection.is_empty() {
                            physics_edits.push((entity_bits, edit));
                        } else {
                            for &t in &inspector_selection {
                                physics_edits.push((t, edit));
                            }
                        }
                    }
                    // §12 Physics Joint. No fan-out either, and for a simpler
                    // reason: the section only ever describes one joint object.
                    EditorAction::InspectorJointEdit { entity_bits, edit } => {
                        // The eyedropper ARMS a canvas pick (shell state), it is
                        // not a component edit — handled here where `self` is
                        // freely mutable, exactly like `Join` sets `join_request`.
                        // The next canvas click resolves it (`input_dispatch`).
                        match edit {
                            ph2d_editor::JointFieldEdit::PickBodyA => {
                                self.joint_body_pick = Some((entity_bits, false));
                            }
                            ph2d_editor::JointFieldEdit::PickBodyB => {
                                self.joint_body_pick = Some((entity_bits, true));
                            }
                            _ => joint_edits.push((entity_bits, edit)),
                        }
                    }
                    EditorAction::InspectorVisibilitySectionEdit { entity_bits, edit } => {
                        // BulkSelect fan-out, same shape as the sampling edit.
                        if inspector_selection.is_empty() {
                            visibility_section_edits.push((entity_bits, edit));
                        } else {
                            for &t in &inspector_selection {
                                visibility_section_edits.push((t, edit));
                            }
                        }
                    }
                    EditorAction::InspectorNameEdit(info) => {
                        // Latest-wins (Option-coalesce parity).
                        name_edit = Some(info);
                    }
                    EditorAction::SetImageFilter { mode } => {
                        // Single global image-filter toggle. Rebuilds the
                        // atlas + individual samplers and their bind groups
                        // so EVERY sprite samples with the new mode; no
                        // texture re-upload. The Vello BG-Removal preview
                        // reads `hero.project.image_filter` directly (set by
                        // the editor before this action), so both stay in
                        // sync.
                        renderer.set_filter_mode(mode);
                    }
                    EditorAction::SetPresentMode { vsync } => {
                        // Config → Display toggle. VSync (Fifo) = smooth
                        // hardware-paced motion; Immediate = non-blocking
                        // (no mouse-stutter). Reconfigures the swap chain
                        // in place. Both modes are available on this
                        // backend (boot log confirms); Fifo is the
                        // universal fallback.
                        surface.set_present_mode(if vsync {
                            wgpu::PresentMode::Fifo
                        } else {
                            wgpu::PresentMode::Immediate
                        });
                    }
                    EditorAction::Transport(cmd) => {
                        // TopBar Play/Pause/Reset drive the ONE clock
                        // (`Playhead`, W4.T7). Physics, Motion, Timeline and
                        // Flip all ride it, so one click moves every
                        // time-based subsystem at once. The single door
                        // `transport::apply` is unit-tested headless. NOTE:
                        // physics scrub-back — the ball flying back up — is
                        // W1.5; here Reset only returns the clock to 0.
                        crate::transport::apply(cmd, &mut self.playhead);
                    }
                    // (Bgremoval bake leftover handled inside the
                    // `OneShotImageOp` arm above — defers to the
                    // image_edit drain site so `bgremoval_active` is
                    // observed AFTER any same-frame ActivateTool fires.)
                    // EditorAction is `#[non_exhaustive]`. A future
                    // variant landing in `ph2d-editor` shouldn't break
                    // the shell — drop it silently here until a
                    // dispatch site is wired up.
                    _ => {}
                }
            }
            for a in bgremoval_leftover {
                hero.bus.push(a);
            }
            for a in painter_leftover {
                hero.bus.push(a);
            }
            // Drain the `EditorAction::ActivateTool { tool_id: "bgremoval" }`
            // intent raised by clicking the Bg Removal pill. The hero can't reach
            // `gfx.tools` so the activation round-trips via the bus.
            // Same force-refresh of the snapshot push state as the
            // Digit3 shortcut below so the next snapshot push fires
            // against the current selection.
            // Data-driven activation of any stateful image-tool (audit F1
            // 2026-05-26 — substitui 6 drain blocks hardcoded per-tool).
            // Gated on `mode_on`: image tools are only reachable while Image
            // Tools toggle is on (the pills only exist then; the Digit3
            // shortcut must also respect the mode). The reconcile below is
            // the safety net, but gating here avoids a 1-frame
            // activate→deactivate flicker + a spurious toast.
            //
            // Cluster lookup via `installed_registry()` resolve o handler kind
            // (Stateful vs OneShot) e o label canônico (`Tool::label()`); zero
            // hardcoded id no dispatch. Tools dropped via fan-out drop-crate
            // (incluindo Painter T1.1) flow pelo mesmo canal automaticamente.
            //
            // Legacy débito: `last_bgremoval_pushed_entity = None` reset é
            // bgremoval-specific shell cache. Em T-N.X (refactor cache-per-tool
            // map) substituído por `HashMap<ToolId, ShellCache>` ou hook em
            // `Tool::on_activate` (ADR-0041). Por hoje, mantido inline.
            if let Some(tool_id) = pending_image_tool_activation.take() {
                // Look up the activating tool's cluster + Stateful gate.
                // W1.T1.7 generalization: was "image_tools" only; now also
                // accepts "vector_tools" (Pen tool ship). When a third
                // cluster appears, add it here OR extract a generic
                // `find_activatable_stateful_tool` helper.
                let activating_cluster: Option<&'static str> = ph2d_editor::installed_registry()
                    .and_then(|reg| {
                        ["image_tools", "vector_tools", "motion_tools", "flip_tools"]
                            .into_iter()
                            .find(|&cluster_name| {
                                reg.cluster(cluster_name).iter().any(|m| {
                                    m.id == tool_id
                                        && matches!(
                                            m.handler,
                                            ph2d_tool_registry::ToolHandler::Stateful { .. }
                                        )
                                })
                            })
                    });
                // Per-cluster activation gate. "image_tools" requires
                // the IMG mode toggle; "vector_tools" / "motion_tools" have no
                // toggle so they're always-on (the pill is direct-activate).
                let gate_on = match activating_cluster {
                    Some("image_tools") => hero.image_edit.mode_on,
                    Some("vector_tools") | Some("motion_tools") | Some("flip_tools") => true,
                    _ => false,
                };
                // O pill de um cluster direct-activate ALTERNA: clicar na ferramenta
                // já ativa sai dela e volta para a default (move). É o que faz uma
                // forma vetorial voltar a se comportar como qualquer objeto — o
                // gizmo de sprite a move, o clique a seleciona (ADR-0111). Os
                // `image_tools` ficam de fora: quem manda neles é o toggle IMG.
                let already_active = tools.active().map(ph2d_editor::Tool::id)
                    == Some(ph2d_editor::ToolId::new(tool_id));
                let toggles_off = matches!(
                    activating_cluster,
                    Some("vector_tools" | "motion_tools" | "flip_tools")
                );
                if gate_on && already_active && toggles_off {
                    tools.activate_default();
                    self.title_dirty = true;
                    if let Some(active) = tools.active() {
                        toasts.push(Toast::info(format!("Tool · {}", active.label())));
                    }
                } else if gate_on && tools.set_active(&ph2d_editor::ToolId::new(tool_id)) {
                    self.title_dirty = true;
                    if tool_id == "bgremoval" {
                        self.last_bgremoval_pushed_entity = None;
                    }
                    if let Some(active) = tools.active() {
                        toasts.push(Toast::info(format!("Tool · {}", active.label())));
                    }
                    // (R4: Pen activation no longer needs a sprite —
                    // network IS the asset, world-coords throughout.)
                }
            }
            // Image Tools OFF is AUTHORITATIVE over the active tool. The
            // TopBar Image Tools toggle (`image_edit.mode_on`) and the
            // ToolRegistry's active tool are otherwise decoupled: a
            // stateful image tool (Bg Removal / Padding) activated while
            // the mode was on stays active — panel + on-canvas preview and
            // all — after the mode is toggled off, since nothing
            // deactivated it. Reconcile here every frame, BEFORE the
            // panel/preview bridges run: when the mode is off, no
            // image-edit tool may remain active, so switch back to the
            // default tool and drop the Bg-Removal preview. This is the
            // single invariant that makes "Image Tools off ⟹ every image
            // tool off & inaccessible" hold no matter how the tool became
            // active (toggle-off, a stale path, the Digit3 shortcut).
            if !hero.image_edit.mode_on {
                let active_is_image_tool = tools
                    .active()
                    .map(|t| crate::is_image_edit_tool(&t.id()))
                    .unwrap_or(false);
                if active_is_image_tool
                    && let Some(default_id) = tools.default_tool_id()
                    && tools.set_active(&default_id)
                {
                    self.bgremoval_preview = None;
                    self.last_bgremoval_pushed_entity = None;
                    self.title_dirty = true;
                }
            }
            // Mirror the active image-edit tool's canonical id into the hero
            // state so editor-core chrome (the left rail's Painter face) can
            // react without a dependency on the concrete tool crates (ADR-0040).
            // Runs AFTER the mode-off reconciliation above, so it reflects the
            // frame's final active tool. `ToolId` holds a runtime `String`; the
            // rail only needs to recognise the Painter, so intern to the
            // `&'static str` literal the `ActivateTool { tool_id: "painter" }`
            // action already uses. Gated on `mode_on` (no image tool is
            // reachable with Image Tools off).
            hero.image_edit.active_tool_id = tools
                .active()
                .filter(|_| hero.image_edit.mode_on)
                .and_then(|t| match t.id().0.as_str() {
                    "painter" => Some("painter"),
                    _ => None,
                });
            // Reconcile Image Tools pill ButtonState ↔ active tool. Each pill
            // whose manifest id matches `tools.active()` is forced to Pressed;
            // pills holding a stale Pressed (tool no longer active) drop back
            // to Normal. Hovered/click-transient states are preserved (we only
            // touch the Normal↔Pressed transitions).
            //
            // Data-driven via `installed_registry().cluster("image_tools")` —
            // zero hardcoded tool id (anti-padrão Image Tools Bugs §2.b
            // fechado em T1.2). New tools dropped via fan-out drop-crate
            // inherit the highlight wiring automatically.
            {
                let active_id_string: Option<String> = tools.active().map(|t| t.id().0.clone());
                if let Some(reg) = ph2d_editor::installed_registry() {
                    // W1.T1.7 R3: iterate both image_tools (existing)
                    // AND vector_tools (Pen pill ship) so the Pressed-
                    // highlight reconcile picks up the Pen pill when
                    // the Vector Pen tool activates. Each pill's
                    // NodeId is computed via `hash_node_id(manifest.id)`
                    // — for Pen this matches `TOPBAR_VECTOR_PEN` only
                    // because `TOPBAR_VECTOR_PEN = hash_node_id("vector_pen")`
                    // (image-action pill convention; see ids.rs).
                    for cluster_name in ["image_tools", "vector_tools"] {
                        for manifest in reg.cluster(cluster_name) {
                            let pill_id = ph2d_tool_registry::hash_node_id(manifest.id);
                            let should_press = active_id_string.as_deref() == Some(manifest.id);
                            if let Some(ph2d_editor::InteractiveState::Button { state }) =
                                hero.store.get_mut(pill_id)
                            {
                                use ph2d_editor::widget::ButtonState;
                                match (*state, should_press) {
                                    (ButtonState::Normal, true) => *state = ButtonState::Pressed,
                                    (ButtonState::Pressed, false) => *state = ButtonState::Normal,
                                    _ => {} // preserve Hovered + already-consistent
                                }
                            }
                        }
                    }
                }
            }
            // Padding panel ⟷ tool bridge — publishes the snapshot, draws
            // the live (non-destructive) canvas-bounds preview, and returns
            // the (selection, spec, pivot mode) to bake on Apply. Panel
            // events themselves are routed earlier in the frame via
            // `EditorAction::ToolPanelEvent` → `Tool::handle_panel_event`
            // (ADR-0040 TG-C). Sibling `padding_bridge.rs`.
            let padding_apply =
                padding_bridge::dispatch(hero, tools, sim, camera, window_size, vector_scene);
            // Bg Removal panel ⟷ tool bridge + on-canvas live preview
            // — extracted to sibling `bgremoval_preview.rs` (HR-18 LOC).
            // Panel events now flow through `EditorAction::ToolPanelEvent`
            // (drained above into `handle_panel_event` → `apply_ui_edit`);
            // the canvas-preview cache is gated on `BgRemovalTool::take_params_dirty`
            // instead of a per-frame edits vector (ADR-0040 TG-B).
            let bgremoval_apply_committed = bgremoval_preview::dispatch(
                hero,
                tools,
                sim,
                renderer,
                asset_db,
                atlas_asset_map,
                camera,
                window_size,
                vector_scene,
                &mut self.last_bgremoval_pushed_entity,
                &mut self.bgremoval_preview,
                &mut self.bgremoval_preview_gpu,
                toasts,
            );
            // Color Equalization panel ⟷ tool bridge: drives panel
            // visibility, refreshes the tool's source bitmap when the
            // primary changes, publishes the snapshot the panel paints,
            // and returns the multi-selection on Apply for the bake.
            let color_equalization_apply = color_equalization_bridge::dispatch(
                hero,
                tools,
                sim,
                renderer,
                asset_db,
                atlas_asset_map,
                camera,
                window_size,
                vector_scene,
                &mut self.last_color_equalization_pushed_entity,
                &mut self.color_equalization_previews,
                toasts,
            );
            // Equalize Sizes panel ⟷ tool bridge — multi-sprite, no
            // per-frame on-canvas preview (the visual effect is the
            // Apply bake; an interim transform-only preview is future
            // work). Returns the full `iter_selected()` on Apply for
            // the cross-sprite `run_full_resolution_multi` bake.
            let equalize_sizes_apply = equalize_sizes_bridge::dispatch(hero, tools);
            // Upscale panel ⟷ tool bridge — sabor 3 with on-canvas
            // live preview (algo + scale apply each frame the user
            // moves the slider). Mirror of `color_equalization_bridge`.
            let upscale_apply = upscale_bridge::dispatch(
                hero,
                tools,
                sim,
                renderer,
                asset_db,
                atlas_asset_map,
                camera,
                window_size,
                vector_scene,
                &mut self.last_upscale_pushed_entity,
                &mut self.upscale_preview,
            );
            // ── Persist painter work BEFORE the bridge rebinds / right after a deferred deactivation
            // (Enio 2026-06-24: paint must survive deselect / object-switch / closing painter mode).
            // Done HERE (not in the bridge) because the bake needs `&mut sim` and must run before the
            // bridge's source-push replaces the working canvas. ──
            {
                let painter_id = ph2d_editor::ToolId::new("painter");
                let painter_active = tools.active().map(|t| t.id()) == Some(painter_id.clone());
                if painter_active {
                    // Selection moved off the bound sprite (incl. deselect) → bake it now.
                    let sel = hero.gizmo.selection;
                    if let Some(old) = self.last_painter_pushed_entity
                        && sel != Some(old)
                        && let Some(painter) = tools.active_mut().and_then(|t| {
                            t.as_any_mut()
                                .downcast_mut::<ph2d_tool_painter::PainterTool>()
                        })
                        && painter.has_unbaked_edits()
                    {
                        crate::hero_intents::auto_commit_painter(
                            old,
                            sim,
                            renderer,
                            asset_db,
                            atlas_asset_map,
                            painter,
                        );
                        self.last_painter_pushed_entity = None; // bridge re-pushes the new selection
                    }
                } else if let Some(old) = self.last_painter_pushed_entity {
                    if let Some(painter) = tools.tool_by_id_mut(&painter_id).and_then(|t| {
                        t.as_any_mut()
                            .downcast_mut::<ph2d_tool_painter::PainterTool>()
                    }) && painter.take_deferred_bake()
                    {
                        // The painter deactivated with unbaked edits → bake the kept canvas, then
                        // finish the teardown its `on_deactivate` deferred.
                        crate::hero_intents::auto_commit_painter(
                            old,
                            sim,
                            renderer,
                            asset_db,
                            atlas_asset_map,
                            painter,
                        );
                        (painter as &mut dyn ph2d_editor::tool::RasterEditTool).deactivate();
                    }
                    // ⚠️ Cleared whether or not there was a bake to defer. The tool is not active, so
                    // nothing is bound — and this memo is read downstream as "the doc the painter is
                    // working on" (`on_active_doc` in the image-edit intents). Leaving it set on the
                    // no-edits path left it naming a sprite the painter had already torn down: the
                    // same stale-second-copy that made the canvas unreachable (Enio 2026-07-22).
                    self.last_painter_pushed_entity = None;
                }
            }
            // Painter panel ⟷ tool bridge (W1 T1.5) — source push +
            // current_preview drain + pending_commit capture; on-canvas
            // overlay paints the canvas RGBA over the sprite footprint.
            // Sidebar Procreate-style lands in W2 (ph2d-panel-painter).
            let painter_dispatch_t0 = Instant::now();
            let painter_apply_committed = painter_bridge::dispatch(
                hero,
                tools,
                sim,
                renderer,
                asset_db,
                atlas_asset_map,
                camera,
                window_size,
                vector_scene,
                paint_ctx.text,
                self.last_pointer,
                &mut self.last_painter_pushed_entity,
                &mut self.painter_preview,
                &mut self.painter_preview_gpu,
                &mut self.painter_gpu_preview,
                &mut self.painter_commit_requested,
                &mut self.painter_undo_requested,
                &mut self.painter_redo_requested,
                toasts,
            );
            // Live-preview a non-selected sprite used as the brush Shape (so its opacity/blend remote-
            // control edits show in real time), into a SECOND preview slot/override.
            painter_bridge_shape_preview::drive_shape_source_preview(
                tools,
                renderer,
                &mut self.painter_shape_source_preview_gpu,
                toasts,
            );
            // Always measure (one Instant/frame) so the HUD's "paint ms" gauge is live, not gated on
            // the frame profiler. EWMA the painter CPU per frame = this frame's preview dispatch +
            // the coalesced re-stamp flush; publish reads it (1-frame lag — fine for a smoothed gauge).
            self.last_dispatch_us = painter_dispatch_t0.elapsed().as_micros() as u64;
            const PAINT_ALPHA: f32 = 0.1;
            let paint_ms_now = (self.last_dispatch_us + self.last_paint_stamp_us) as f32 / 1000.0;
            self.paint_ms_ewma =
                PAINT_ALPHA * paint_ms_now + (1.0 - PAINT_ALPHA) * self.paint_ms_ewma;
            if frame_prof_on() {
                FRAME_PROF_DISPATCH_US.with(|c| c.set(self.last_dispatch_us));
            }
            // ADR-0108 cutover: the Vector drawing tool. `AppGfx.vec_scene` is
            // document artwork — render it into the shared Vello scene EVERY
            // frame (not gated on the active tool; no per-tool branch). The
            // `vector_bridge` reflects the active tool's Style into the shell
            // Pen + recolours the selection; the edit gizmos draw ONLY while the
            // Vector tool is active (mirror of how the pen input is gated).
            let vector_active = tools
                .active()
                .is_some_and(|t| t.id() == ph2d_editor::ToolId::new("vector"));
            // World units per screen pixel (1px delta) — lets the bridge convert
            // the tool's px stroke width into the selected path's world width.
            let vw0 = camera.screen_to_world((0.0, 0.0), window_size);
            let vw1 = camera.screen_to_world((1.0, 0.0), window_size);
            let vec_px_to_world =
                (((vw1[0] - vw0[0]).powi(2) + (vw1[1] - vw0[1]).powi(2)).sqrt()) as f64;
            // Apply a Boolean button press (drained above) to the document before
            // the bridge/render so the result selects + renders this frame
            // (mirror of the U/I/D hotkeys' `vec_boolean`).
            // ADR-0128: o botão "Blend" cria o Blend Object VIVO sobre as formas fechadas
            // selecionadas (2..=5, em z). `create` empurra o spine e devolve o componente; o
            // `sync`/`upkeep`/`recook` do frame dão vida a ele. Seleciona o OBJETO (o spine) para
            // o slider Steps passar a mirar nele.
            if pending_create_blend {
                let xf = crate::vec_transform::build(sim, &self.vec_entities);
                // Os passos vêm do slider do painel — a fonte da verdade é o widget, não uma
                // cópia no shell (uma cópia driftaria do que o artista está VENDO).
                let steps = hero
                    .store
                    .slider(ph2d_editor::ids::VECTOR_BLEND_STEPS)
                    .map_or(ph2d_tool_vector::params::BLEND_STEPS_DEFAULT, |(_, v)| {
                        ph2d_tool_vector::params::blend_steps_from_track(f64::from(v))
                    });
                // A ORDEM da cadeia: no modo Pick Shapes, a de CLIQUE (a lista escolhida a dedo);
                // fora dele, a de z da seleção (ADR-0128 C2b). O Pick é o "escolher a ordem" do Enio.
                let picking = self.vec_draw_config.mode == ph2d_tool_vector::DrawMode::PickBlend;
                let sources = if picking && self.vec_blend_picks.len() >= 2 {
                    self.vec_blend_picks.clone()
                } else {
                    crate::blend_live::selected_closed_in_z(vec_scene, &self.vec_pen)
                };
                if let Some((spine, blend)) =
                    crate::blend_live::create(vec_scene, &xf, &sources, steps)
                {
                    self.vec_pen.select_many(&[spine]);
                    self.vec_blend_pending = Some((spine, blend));
                    self.vec_blend_picks.clear();
                    // Feito o blend, volta ao Select — o objeto novo está selecionado e o gizmo
                    // manda (o modo Pick já cumpriu o papel de juntar a lista). Inline do
                    // `vec_set_draw_mode` (que re-borrowaria o `gfx` já destructurado): a tool é a
                    // dona do modo, `vec_draw_config` é o espelho lido no mesmo frame.
                    crate::render_loop::vector_bridge::set_mode(
                        tools,
                        ph2d_tool_vector::DrawMode::Select,
                    );
                    self.vec_draw_config.mode = ph2d_tool_vector::DrawMode::Select;
                    eprintln!(
                        "[ph2d-vec] blend: objeto vivo sobre {} formas, {steps} passos/elo",
                        sources.len()
                    );
                } else {
                    eprintln!("[ph2d-vec] blend: selecione de 2 a 5 formas FECHADAS");
                }
            }
            // **MORPH** — o irmão animável do blend: UMA forma entre DUAS, com o `t` keyável.
            // Mesma mecânica do `create` acima (`push` do path + componente; o
            // `sync`/`upkeep`/`recook` do frame lhe dão vida), e a mesma escolha de fontes: no
            // Pick Shapes a ordem de CLIQUE, fora dele a ordem de z.
            if pending_create_morph {
                let picking = self.vec_draw_config.mode == ph2d_tool_vector::DrawMode::PickBlend;
                let sources = if picking && self.vec_blend_picks.len() >= 2 {
                    self.vec_blend_picks.clone()
                } else {
                    crate::blend_live::selected_closed_in_z(vec_scene, &self.vec_pen)
                };
                // DUAS, e exatamente duas: o morph é um `t` sobre UM par. Uma cadeia de 3+ formas
                // é o Blend — e recusar aqui em voz alta é melhor do que morfar as duas primeiras
                // e deixar o artista a descobrir sozinho quais foram escolhidas.
                if let [a, b] = sources[..] {
                    let (id, morph) = crate::morph_live::create(vec_scene, a, b);
                    self.vec_pen.select_many(&[id]);
                    self.vec_morph_pending = Some((id, morph));
                    self.vec_blend_picks.clear();
                    crate::render_loop::vector_bridge::set_mode(
                        tools,
                        ph2d_tool_vector::DrawMode::Select,
                    );
                    self.vec_draw_config.mode = ph2d_tool_vector::DrawMode::Select;
                    eprintln!("[ph2d-vec] morph: objeto vivo entre 2 formas (t animável)");
                } else {
                    eprintln!(
                        "[ph2d-vec] morph: selecione exatamente 2 formas FECHADAS (tem {})",
                        sources.len()
                    );
                }
            }
            // Arrastar o slider `t` move o morph SELECIONADO pelo caminho, ao vivo.
            if let Some(t) = pending_morph_t {
                for id in self.vec_pen.selected_paths() {
                    let Some(&bits) = self.vec_entities.get(id) else {
                        continue;
                    };
                    let e = ph2d_ecs::Entity::from_bits(bits);
                    if let Some(mut m) = sim.world_mut().get_mut::<ph2d_ecs::VecMorph>(e) {
                        m.t = t;
                    }
                }
            }
            // ADR-0128 Fase D: **Expand** — materializa os passos VIRTUAIS em formas REAIS e
            // descarta o objeto vivo. A sequência de z que ele pede espera em `vec_restack`: as
            // entidades dos passos só nascem no `sync`, e quem manda no z é a ÁRVORE (ADR-0110).
            if pending_expand_blend {
                let xf = crate::vec_transform::build(sim, &self.vec_entities);
                let runs = crate::blend_live::expand(
                    sim,
                    vec_scene,
                    &self.vec_entities,
                    &xf,
                    &mut self.vec_pen,
                );
                if runs.is_empty() {
                    eprintln!("[ph2d-vec] blend: selecione um blend (a linha, ou uma forma dele)");
                } else {
                    let n: usize = runs.iter().map(Vec::len).sum();
                    eprintln!("[ph2d-vec] blend: expandido em {n} forma(s)");
                    self.vec_restack.extend(runs);
                }
            }
            // ADR-0128 Fase D: **Release** — desfaz o blend (os passos somem, as fontes ficam).
            if pending_release_blend
                && crate::blend_live::release(sim, vec_scene, &self.vec_entities, &mut self.vec_pen)
            {
                eprintln!("[ph2d-vec] blend: solto (as formas-fonte ficam)");
            }
            // ADR-0129: **Envelope** — envolve a seleção (1..N formas) num container com a gaiola em
            // repouso. Síncrono (as formas já existem; o container não tem path), então age já.
            // Plano 22: prender / soltar / trocar o lado. Um comando só por frame (é um
            // clique), e todos passam pelas portas do `vec_text_ride` — que re-cozinham pela
            // porta de sempre, para não haver uma segunda resposta a "como um texto vira
            // geometria".
            if let Some(v) = pending_textpath_offset {
                let sel = self.vec_pen.selected_paths().to_vec();
                crate::vec_text_ride::edit(sim, vec_scene, &self.vec_entities, &sel, |l| {
                    l.start_offset = v as f32;
                });
            }
            if let Some(cmd) = pending_textpath {
                let sel = self.vec_pen.selected_paths().to_vec();
                let done = match cmd {
                    crate::vec_text_ride::TextPathCmd::Link => {
                        crate::vec_text_ride::link(sim, vec_scene, &self.vec_entities, &sel)
                    }
                    crate::vec_text_ride::TextPathCmd::Detach => {
                        crate::vec_text_ride::detach(sim, vec_scene, &self.vec_entities, &sel)
                    }
                    crate::vec_text_ride::TextPathCmd::Flip(v) => {
                        crate::vec_text_ride::edit(sim, vec_scene, &self.vec_entities, &sel, |l| {
                            l.flip = v;
                        })
                    }
                };
                if !done {
                    eprintln!(
                        "[ph2d-vec] text on path: selecione o TEXTO e um caminho (ou um texto \
                         ja' preso, para soltar)"
                    );
                }
            }
            // Picker do texto (Enio 2026-07-23): o botão só ARMOU; aqui capturamos a FONTE — o texto
            // em foco — para o clique seguinte no canvas escolher o guia. Capturamos o id agora porque
            // esse clique pode mudar a seleção (ele ESCOLHE o guia, não deve virar a fonte).
            if pending_text_pick {
                let sel = self.vec_pen.selected_paths().to_vec();
                if let Some((text, _, _)) =
                    crate::vec_text_object::selected_text_object(sim, &self.vec_entities, &sel)
                {
                    self.vec_path_pick = Some(crate::vec_pick::PathPick::TextObject(text));
                    eprintln!(
                        "[ph2d-vec] text on path: pick armado -- clique no CAMINHO-guia (vazio = \
                         desiste)"
                    );
                }
            }
            // Contour (pesquisa `20_*` #9): os comandos e os knobs, todos pela porta única
            // `contour_live`. O `recook` do frame seguinte redesenha os anéis.
            //
            // ⚠️ **Add/Remove/Expand agem sobre a SELEÇÃO inteira** e os knobs também: uma seleção
            // de duas formas ganha dois contours, e mexer no slider afina os dois. É o mesmo
            // desenho do Offset vivo — e o oposto do Join da física, que precisa de fan-out
            // BLOQUEADO porque criaria um objeto por par. Aqui cada forma tem o seu, e um efeito
            // por forma é exatamente o que o artista pediu ao selecionar duas.
            {
                let sel: Vec<ph2d_vec_scene::VecPathId> = self.vec_pen.selected_paths().to_vec();
                match pending_contour {
                    Some(crate::contour_live::ContourCmd::Add) => {
                        let xf = crate::vec_transform::build(sim, &self.vec_entities);
                        let scale = crate::vec_expand::offset_scale(vec_scene, &self.vec_pen, &xf);
                        let n = crate::contour_live::arm(sim, &self.vec_entities, &sel, scale);
                        eprintln!("[ph2d-vec] contour: armado em {n} forma(s)");
                    }
                    Some(crate::contour_live::ContourCmd::Remove) => {
                        let n = crate::contour_live::remove(sim, &self.vec_entities, &sel);
                        eprintln!("[ph2d-vec] contour: removido de {n} forma(s)");
                    }
                    Some(crate::contour_live::ContourCmd::Expand) => {
                        let xf = crate::vec_transform::build(sim, &self.vec_entities);
                        let runs =
                            self.contour_live
                                .expand(sim, vec_scene, &self.vec_entities, &xf, &sel);
                        if runs.is_empty() {
                            eprintln!(
                                "[ph2d-vec] contour: nada a expandir (selecione uma forma com contour)"
                            );
                        } else {
                            let n: usize = runs.iter().map(|r| r.len().saturating_sub(1)).sum();
                            eprintln!("[ph2d-vec] contour: expandido em {n} anel(is)");
                            self.vec_restack.extend(runs);
                        }
                    }
                    None => {}
                }
                if let Some(v) = pending_contour_steps {
                    let steps = v.max(1.0) as u16;
                    crate::contour_live::edit(sim, &self.vec_entities, &sel, |c| c.steps = steps);
                }
                if let Some(frac) = pending_contour_d {
                    // FRAÇÃO → MUNDO na fronteira, com a MESMA escala do `arm` e do Offset.
                    let xf = crate::vec_transform::build(sim, &self.vec_entities);
                    let d = frac * crate::vec_expand::offset_scale(vec_scene, &self.vec_pen, &xf);
                    crate::contour_live::edit(sim, &self.vec_entities, &sel, |c| c.d = d);
                }
                if let Some(v) = pending_contour_accel {
                    #[allow(clippy::cast_possible_truncation)]
                    let accel = v as f32;
                    crate::contour_live::edit(sim, &self.vec_entities, &sel, |c| c.accel = accel);
                }
                if let Some(code) = pending_contour_join {
                    crate::contour_live::edit(sim, &self.vec_entities, &sel, |c| c.join = code);
                }
                if let Some(code) = pending_contour_side {
                    crate::contour_live::edit(sim, &self.vec_entities, &sel, |c| c.side = code);
                }
                // A COR-ALVO vem do picker OKLCH partilhado, lido de volta como o Stroke e o Fill
                // fazem no `vector_bridge` — mas AQUI, porque o alvo da escrita é um componente
                // ECS e este é o bloco que tem `sim` e o mapa em mãos. A swatch é marcada como
                // picker-swatch no `paint.rs` do painel; o Down abre o picker por dispatch
                // genérico, e o que chega cá é só a escolha.
                if hero.store.picker_target() == Some(ph2d_editor::ids::VECTOR_CONTOUR_TO)
                    && let Some((value, _, _, _)) = hero
                        .store
                        .blender_picker(ph2d_editor::ids::INSP_BLENDER_PICKER)
                {
                    let to = value.rgba;
                    crate::contour_live::edit(sim, &self.vec_entities, &sel, |c| c.to = to);
                }
            }
            // Filters (a PILHA de FX raster, plano 24): "Add" empurra um degrau, os ícones do card
            // reordenam/desarmam/apagam, e os sliders + o picker afinam a linha. Tudo pela porta
            // única `fx_live`; o `recook` do frame seguinte re-produz as imagens. Age sobre a
            // SELEÇÃO inteira — uma pilha por forma, o mesmo desenho do Contour/Offset.
            //
            // ⚠️ **Remover a última linha REMOVE o componente** — e isso mora dentro do
            // `fx_live::edit`, não aqui: uma regra escrita no chamador é uma regra que o próximo
            // chamador nasce sem.
            {
                use crate::fx_live::FilterHit;
                let sel: Vec<ph2d_vec_scene::VecPathId> = self.vec_pen.selected_paths().to_vec();
                // O punho arrastado move APENAS a posição — a cor é da swatch, e o índice é o de
                // AUTORIA (quem ordena é o consumidor), então arrastar por cima do vizinho não
                // re-liga o dedo a outro stop.
                if let Some((row, idx, x)) = pending_filter_stop {
                    let diag = std::env::var_os("PH2D_FX_RAMP_DIAG").is_some();
                    if diag {
                        eprintln!(
                            "[ramp] shell aplica: {} forma(s) selecionada(s), linha {row} stop {idx}",
                            sel.len()
                        );
                    }
                    crate::fx_live::edit(sim, &self.vec_entities, &sel, |f| {
                        if let Some(op) = f.ops.get_mut(row)
                            && usize::from(idx) < usize::from(op.stop_count)
                            && let Some(slot) = op.stop_pos.get_mut(usize::from(idx))
                        {
                            *slot = x.clamp(0.0, 1.0);
                            if diag {
                                eprintln!("[ramp] stop_pos escrito: {:?}", op.stop_pos);
                            }
                        } else if diag {
                            eprintln!(
                                "[ramp] RECUSADO: ops={} stop_count={:?}",
                                f.ops.len(),
                                f.ops.get(row).map(|o| o.stop_count)
                            );
                        }
                    });
                }
                if let Some(cmd) = pending_filter_cmd {
                    match cmd {
                        // Um "Add" numa forma SEM pilha cria a pilha; com pilha, empilha no fim
                        // (o topo visual). O degrau nasce com defaults VISÍVEIS.
                        FilterHit::Add(kind) => {
                            for id in &sel {
                                let one = std::slice::from_ref(id);
                                match crate::fx_live::spec_of(sim, &self.vec_entities, *id) {
                                    Some(mut f) if f.has_room() => {
                                        f.ops.push(ph2d_ecs::FxOp::new(kind));
                                        crate::fx_live::set_filter(
                                            sim,
                                            &self.vec_entities,
                                            one,
                                            Some(f),
                                        );
                                    }
                                    Some(_) => {}
                                    None => {
                                        crate::fx_live::set_filter(
                                            sim,
                                            &self.vec_entities,
                                            one,
                                            Some(ph2d_ecs::VecFilter::single(ph2d_ecs::FxOp::new(
                                                kind,
                                            ))),
                                        );
                                    }
                                }
                            }
                        }
                        FilterHit::Remove(row) => {
                            crate::fx_live::edit(sim, &self.vec_entities, &sel, |f| {
                                if row < f.ops.len() {
                                    f.ops.remove(row);
                                }
                            });
                        }
                        FilterHit::Up(row) => {
                            crate::fx_live::edit(sim, &self.vec_entities, &sel, |f| {
                                f.move_up(row);
                            });
                        }
                        FilterHit::Down(row) => {
                            crate::fx_live::edit(sim, &self.vec_entities, &sel, |f| {
                                f.move_down(row);
                            });
                        }
                        FilterHit::Hide(row) => {
                            crate::fx_live::edit(sim, &self.vec_entities, &sel, |f| {
                                if let Some(op) = f.ops.get_mut(row) {
                                    op.enabled = !op.enabled;
                                }
                            });
                        }
                        // O MODO é a LEI do degrau, não a intensidade dele — e um clique num modo
                        // que o TIPO não oferece é recusado aqui (o painel não o pinta, mas a
                        // recusa mora onde o valor é escrito).
                        FilterHit::Mode(row, mode) => {
                            crate::fx_live::edit(sim, &self.vec_entities, &sel, |f| {
                                if let Some(op) = f.ops.get_mut(row)
                                    && (mode as usize) < ph2d_ecs::FxOp::spec(op.kind).modes.len()
                                {
                                    op.mode = mode;
                                }
                            });
                        }
                        // A LEI DE MISTURA — *como a cor deste degrau encosta na que já está ali*.
                        // Mesma recusa do MODO, pela porta única `FxOp::takes_blend`: um clique
                        // numa lei que o TIPO não toma é recusado ONDE o valor é escrito, e não só
                        // onde ele é pintado. (O popover nem chega a existir num tipo que não a
                        // oferece — mas a segunda metade é o que impede um arquivo, ou um teste,
                        // de instalar um número órfão.)
                        FilterHit::Blend(row, blend) => {
                            crate::fx_live::edit(sim, &self.vec_entities, &sel, |f| {
                                if let Some(op) = f.ops.get_mut(row)
                                    && op.takes_blend()
                                    && blend < ph2d_ecs::FxOp::BLEND_KINDS
                                {
                                    op.blend = blend;
                                }
                            });
                        }
                        // A swatch só ABRE o picker (o `register_picker_swatch` faz isso); a cor
                        // é lida abaixo, do alvo do picker.
                        // O trilho da rampa: `+` põe um stop no maior vão com a cor que já está
                        // ali (não muda o desenho), `−` tira o SELECIONADO com piso em dois.
                        FilterHit::StopAdd(row) => {
                            crate::fx_live::edit(sim, &self.vec_entities, &sel, |f| {
                                if let Some(op) = f.ops.get_mut(row) {
                                    crate::fx_live::add_stop(op);
                                }
                            });
                        }
                        FilterHit::StopRemove(row) => {
                            let sel_stop = usize::from(ph2d_panel_vector::selected_stop(row));
                            crate::fx_live::edit(sim, &self.vec_entities, &sel, |f| {
                                if let Some(op) = f.ops.get_mut(row) {
                                    crate::fx_live::remove_stop(op, sel_stop);
                                }
                            });
                        }
                        FilterHit::Color(_)
                        | FilterHit::StopColor(_)
                        | FilterHit::ColorB(_)
                        | FilterHit::Radius(_)
                        | FilterHit::OffX(_)
                        | FilterHit::OffY(_)
                        | FilterHit::Opacity(_)
                        | FilterHit::Scale(_)
                        | FilterHit::Detail(_)
                        | FilterHit::Seed(_)
                        | FilterHit::Grow(_)
                        | FilterHit::Hue(_)
                        | FilterHit::Sat(_)
                        | FilterHit::Bright(_) => {}
                    }
                }
                if let Some((hit, v)) = pending_filter_val {
                    #[allow(clippy::cast_possible_truncation)]
                    let x = v as f32;
                    crate::fx_live::edit(sim, &self.vec_entities, &sel, |f| {
                        let row = match hit {
                            FilterHit::Radius(r)
                            | FilterHit::OffX(r)
                            | FilterHit::OffY(r)
                            | FilterHit::Opacity(r)
                            | FilterHit::Scale(r)
                            | FilterHit::Detail(r)
                            | FilterHit::Seed(r)
                            | FilterHit::Grow(r)
                            | FilterHit::Hue(r)
                            | FilterHit::Sat(r)
                            | FilterHit::Bright(r) => r,
                            _ => return,
                        };
                        let Some(op) = f.ops.get_mut(row) else { return };
                        match hit {
                            FilterHit::Radius(_) => op.radius = x,
                            FilterHit::OffX(_) => op.offset[0] = x,
                            FilterHit::OffY(_) => op.offset[1] = x,
                            FilterHit::Opacity(_) => op.opacity = x,
                            FilterHit::Scale(_) => op.scale = x,
                            // As duas CONTAGENS chegam já arredondadas da fronteira do painel; o
                            // clamp aqui é a recusa de um número órfão (um arquivo, um teste),
                            // não uma segunda régua.
                            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                            FilterHit::Detail(_) => {
                                op.detail = (x.round() as u8).clamp(1, ph2d_ecs::FxOp::MAX_DETAIL);
                            }
                            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                            FilterHit::Seed(_) => op.seed = x.clamp(0.0, 255.0).round() as u8,
                            FilterHit::Grow(_) => op.grow = x,
                            // ⚠️ **GRAUS -> VOLTAS**, o inverso exacto da linha que publica o
                            // snapshot. As duas conversões são as ÚNICAS do eixo, e é por isso
                            // que ficam nomeadas uma na outra.
                            FilterHit::Hue(_) => op.hue = x / 360.0,
                            FilterHit::Sat(_) => op.sat = x,
                            FilterHit::Bright(_) => op.bright = x,
                            _ => {}
                        }
                    });
                }
                // A cor do halo vem do MESMO picker OKLCH partilhado (lido como o Contour, aqui,
                // porque o alvo é um componente ECS). Qual LINHA? A que o alvo do picker nomeia.
                if let Some(target) = hero.store.picker_target()
                    && let Some((row, slot)) = crate::fx_live::colour_target(target)
                    && let Some((value, _, _, _)) = hero
                        .store
                        .blender_picker(ph2d_editor::ids::INSP_BLENDER_PICKER)
                {
                    let c = value.rgba;
                    let col = [
                        f32::from(c[0]) / 255.0,
                        f32::from(c[1]) / 255.0,
                        f32::from(c[2]) / 255.0,
                        f32::from(c[3]) / 255.0,
                    ];
                    // ⚠️ **A rota mora numa função PURA** (`apply_picked_colour`), e não aqui: a
                    // decisão de QUAL cor recebe a escolha é o que um arch-gate sobre o fonte NÃO
                    // consegue provar — a mutação que dobrava o stop na ponta escura manteve o nome
                    // do slot num braço inalcançável e passou verde. Lá ela é observável.
                    let sel_stop = usize::from(ph2d_panel_vector::selected_stop(row));
                    crate::fx_live::edit(sim, &self.vec_entities, &sel, |f| {
                        if let Some(op) = f.ops.get_mut(row) {
                            crate::fx_live::apply_picked_colour(op, slot, sel_stop, col);
                        }
                    });
                }
            }
            // Pattern on Path (plano 23): os sliders afinam o vínculo do MOTIVO — que é o caminho
            // LINKADO da seleção (`linked_motif`), não o primário: depois de prender, o primário
            // pode ser o GUIA. O comando prende/solta/vira. Tudo pela porta única `pattern_live`, e
            // o `recook` do frame seguinte redesenha as cópias.
            let pp_motif = crate::pattern_live::linked_motif(
                sim,
                &self.vec_entities,
                self.vec_pen.selected_paths(),
            );
            if let Some(v) = pending_pp_spacing
                && let Some(motif) = pp_motif
            {
                crate::pattern_live::edit(sim, &self.vec_entities, motif, |l| l.spacing = v as f32);
            }
            if let Some(v) = pending_pp_start
                && let Some(motif) = pp_motif
            {
                crate::pattern_live::edit(sim, &self.vec_entities, motif, |l| {
                    l.start_offset = v as f32;
                });
            }
            if let Some(v) = pending_pp_end
                && let Some(motif) = pp_motif
            {
                crate::pattern_live::edit(sim, &self.vec_entities, motif, |l| {
                    l.end_offset = v as f32;
                });
            }
            if let Some(v) = pending_pp_offset
                && let Some(motif) = pp_motif
            {
                crate::pattern_live::edit(sim, &self.vec_entities, motif, |l| l.offset = v as f32);
            }
            // A rotação tem porta PRÓPRIA (`set_rotation`) e não o `edit`: ela vive num componente
            // separado, para não bumpar o `PROJECT_SCHEMA` -- e essa porta destaca no neutro.
            if let Some(v) = pending_pp_rotation
                && let Some(motif) = pp_motif
            {
                crate::pattern_live::set_rotation(sim, &self.vec_entities, motif, v as f32);
            }
            // Slide re-centra o trecho `[Start, End]` PRESERVANDO o comprimento: move as duas
            // âncoras juntas (o pedido do Enio). O centro é clampado para a janela caber em [0,1].
            if let Some(v) = pending_pp_slide
                && let Some(motif) = pp_motif
            {
                crate::pattern_live::edit(sim, &self.vec_entities, motif, |l| {
                    let half = (f64::from(l.end_offset) - f64::from(l.start_offset)) * 0.5;
                    let c = v.clamp(half, 1.0 - half);
                    l.start_offset = (c - half) as f32;
                    l.end_offset = (c + half) as f32;
                });
            }
            if let Some(cmd) = pending_patternpath {
                let sel = self.vec_pen.selected_paths().to_vec();
                let done = match cmd {
                    // O guia é o caminho de MAIOR extensão dos dois (independe da ordem de clique)
                    // — a correção do "escolhendo a si mesmo" (Enio).
                    crate::pattern_live::PatternPathCmd::Link => {
                        crate::pattern_live::link_candidate(vec_scene, &sel).is_some_and(
                            |(motif, guide)| {
                                crate::pattern_live::link(sim, &self.vec_entities, motif, guide)
                            },
                        )
                    }
                    crate::pattern_live::PatternPathCmd::Detach => pp_motif
                        .is_some_and(|m| crate::pattern_live::detach(sim, &self.vec_entities, m)),
                    crate::pattern_live::PatternPathCmd::Flip(v) => pp_motif.is_some_and(|m| {
                        crate::pattern_live::edit(sim, &self.vec_entities, m, |l| l.flip = v)
                    }),
                };
                if !done {
                    eprintln!(
                        "[ph2d-vec] pattern on path: selecione o MOTIVO e um caminho (ou um motivo \
                         ja' preso, para soltar/afinar)"
                    );
                }
            }
            // Picker do motivo (Enio 2026-07-23): o botão só ARMOU; a FONTE é o motivo selecionado (a
            // `can_pick` já garantiu um só, ainda solto). O clique seguinte no canvas escolhe o guia.
            if pending_pp_pick && let Some(motif) = self.vec_pen.selected() {
                self.vec_path_pick = Some(crate::vec_pick::PathPick::PatternMotif(motif));
                eprintln!(
                    "[ph2d-vec] pattern on path: pick armado -- clique no CAMINHO-guia (vazio = \
                     desiste)"
                );
            }
            if pending_create_envelope {
                let ids: Vec<ph2d_vec_scene::VecPathId> = self.vec_pen.selected_paths().to_vec();
                match crate::envelope_live::create(sim, vec_scene, &self.vec_entities, &ids) {
                    Some(_) => {
                        // O artista SELECIONOU a forma e SÓ ENTÃO clicou Envelope: enveloparr
                        // re-parenteia o filho sem tocar o pen, então o `sync_selection` deste
                        // frame não reroda a promoção filho→container (nem o pen mudou, nem o
                        // conjunto vetorial do gizmo) — e o gizmo ficaria no FILHO, sem gaiola para
                        // desenhar (alças de nó em vez da gaiola). Invalidar a memória do sync força
                        // a promoção no `sync_selection` logo abaixo. Gate:
                        // `enveloping_a_selected_shape_promotes_the_gizmo_to_the_container`.
                        self.vec_sel.invalidate();
                        eprintln!(
                            "[ph2d-vec] envelope: {} forma(s) envolvida(s) -- va' para o modo Node \
                             e arraste os CANTOS da gaiola",
                            ids.len()
                        );
                    }
                    None => eprintln!("[ph2d-vec] envelope: selecione ao menos UMA forma"),
                }
            }
            // ADR-0129: **Expand** (a deformada vira o desenho) e **Release** (a fonte autorada
            // volta). O MESMO `dissolve` — muda só QUAL geometria fica.
            if pending_expand_envelope
                && crate::envelope_live::dissolve(
                    sim,
                    vec_scene,
                    &self.vec_entities,
                    &mut self.vec_pen,
                    crate::envelope_live::Keep::Deformed,
                )
            {
                eprintln!("[ph2d-vec] envelope: expandido (a deformacao virou o desenho)");
            }
            if pending_release_envelope
                && crate::envelope_live::dissolve(
                    sim,
                    vec_scene,
                    &self.vec_entities,
                    &mut self.vec_pen,
                    crate::envelope_live::Keep::Authored,
                )
            {
                eprintln!("[ph2d-vec] envelope: solto (a forma original voltou)");
            }
            // ADR-0129 Fatia D: trocar o GESTO da gaiola. O container vem da MESMA porta que
            // decide a selecao e alimenta os chips -- o clique nunca acerta outro envelope que
            // nao o desenhado. Trocar re-cozinha no frame seguinte: em repouso os dois mapas
            // coincidem, entao numa gaiola intocada a troca nao move um pixel.
            if pending_envelope_kind.is_some() || pending_clear_pins {
                let sel: Vec<u64> = self
                    .vec_pen
                    .selected_paths()
                    .iter()
                    .filter_map(|id| self.vec_entities.get(id).copied())
                    .collect();
                if let Some(bits) = crate::envelope_live::sole_container(sim, &sel) {
                    if let Some(kind) = pending_envelope_kind
                        && crate::envelope_gesture::set_kind(sim, bits, kind)
                    {
                        eprintln!("[ph2d-vec] envelope: gesto {kind:?}");
                    }
                    if pending_clear_pins && crate::envelope_gesture::clear_pins(sim, bits) {
                        eprintln!("[ph2d-vec] envelope: pinos apagados");
                    }
                }
            }
            // ADR-0129 Fatia C: o preset carimba a gaiola inteira; o Bend re-carimba o preset ATIVO.
            // Os dois passam pela MESMA `apply_preset`, entao clicar "Arc" e arrastar o Bend nao
            // podem produzir gaiolas diferentes para os mesmos numeros.
            if pending_envelope_preset.is_some() || pending_envelope_bend.is_some() {
                let sel: Vec<u64> = self
                    .vec_pen
                    .selected_paths()
                    .iter()
                    .filter_map(|id| self.vec_entities.get(id).copied())
                    .collect();
                if let Some(bits) = crate::envelope_live::sole_container(sim, &sel)
                    && let Some((cur_warp, cur_bend)) = crate::envelope_gesture::warp_of(sim, bits)
                {
                    let warp = pending_envelope_preset
                        .and_then(|i| ph2d_ecs::EnvelopeWarp::ALL.get(i).copied())
                        .or(cur_warp);
                    let bend = pending_envelope_bend.unwrap_or(cur_bend);
                    // Sem preset na mao E sem preset ativo, o Bend nao tem o que re-carimbar --
                    // e' o caso da gaiola promovida a manual pelo arrasto.
                    if let Some(warp) = warp
                        && crate::envelope_live::apply_preset(sim, bits, warp, bend)
                    {
                        eprintln!("[ph2d-vec] envelope: preset {warp:?} bend {bend:.2}");
                    }
                }
            }
            // ADR-0132: a pilha de efeitos do caminho selecionado. Os dois passam pela MESMA
            // `sole_path`, entao o que a secao PINTA e o que o clique ESCREVE nao podem divergir.
            if pending_fx_add.is_some()
                || pending_fx_button.is_some()
                || pending_fx_param.is_some()
                || pending_fx_apply
            {
                let sel = self.vec_pen.selected_paths().to_vec();
                if let Some(pid) = crate::fx_bridge::sole_path(&sel) {
                    crate::fx_bridge_dispatch::apply(
                        vec_scene,
                        pid,
                        pending_fx_add,
                        pending_fx_button,
                        pending_fx_param,
                        pending_fx_apply,
                    );
                }
            }
            // ADR-0128 C2b: Reset Spine — volta o(s) blend(s) selecionado(s) ao spine automático.
            if pending_reset_spine
                && crate::blend_live::reset_spine(
                    sim,
                    &self.vec_entities,
                    &self.vec_pen,
                    &mut self.vec_blend_spines,
                )
            {
                eprintln!("[ph2d-vec] blend: spine resetado ao automático");
            }
            // Arrastar o slider Steps retuna o blend SELECIONADO ao vivo (o recook lê
            // `VecBlend.steps`). Sem blend selecionado, é o valor de criação do próximo Blend.
            if let Some(steps) = pending_blend_steps {
                crate::blend_live::set_selected_steps(
                    sim,
                    &self.vec_entities,
                    &self.vec_pen,
                    steps,
                );
            }
            if let Some(op) = pending_vec_bool {
                let xf = crate::vec_transform::build(sim, &self.vec_entities);
                crate::input_dispatch::apply_vec_boolean(
                    vec_scene,
                    &mut self.vec_history,
                    &mut self.vec_pen,
                    &xf,
                    op,
                );
            }
            // ── Offset AO VIVO ───────────────────────────────────────────────────
            // *"os botões Miter, Round e Bevel são previsualizações em tempo real dos efeitos,
            // mas para consolidar a curva deve-se apertar Apply Offset ou Convert to Curves"*
            // (Enio, 2026-07-21). O documento guarda a curva AUTORADA o tempo todo — o que se
            // vê é a geometria derivada, cozida por `offset_live::recook` e desenhada no z da
            // forma. Aqui só se ARMA a relação (`ph2d_ecs::VecOffset`): o slider dá o `d`, os
            // chips de Corner/Side dão a quina e o lado.
            {
                let knobs = crate::vec_expand::expand_knobs();
                let sel: Vec<ph2d_vec_scene::VecPathId> = self.vec_pen.selected_paths().to_vec();
                // **O painel espelha o que está SELECIONADO.** Sem isto, escolher uma forma com
                // offset vivo mostraria os knobs globais do painel e o chip mentiria sobre a
                // forma que está na tela. A borda é a SELEÇÃO (não o clique), e ela corre ANTES
                // da borda dos chips — publicar depois faria o espelho parecer um clique novo e
                // reescreveria a forma com os valores que acabaram de sair dela.
                let mirror = (sel.len() == 1).then(|| sel[0]).filter(|id| {
                    crate::offset_live::spec_of(sim, &self.vec_entities, *id).is_some()
                });
                let knobs = if mirror != self.vec_offset_mirrored {
                    self.vec_offset_mirrored = mirror;
                    match mirror
                        .and_then(|id| crate::offset_live::spec_of(sim, &self.vec_entities, id))
                    {
                        Some(spec) => {
                            ph2d_panel_vector::set_expand_join(spec.join);
                            ph2d_panel_vector::set_expand_side(spec.side);
                            let scale =
                                crate::vec_expand::offset_scale(vec_scene, &self.vec_pen, &{
                                    crate::vec_transform::build(sim, &self.vec_entities)
                                });
                            hero.store.set_slider_value(
                                ph2d_editor::ids::VECTOR_EXPAND_OFFSET,
                                ph2d_tool_vector::params::offset_frac_to_slider(spec.d / scale),
                            );
                            (spec.join, spec.side)
                        }
                        None => knobs,
                    }
                } else {
                    knobs
                };
                let offset_grabbed = matches!(
                    hero.store.active_id(),
                    Some(id) if id == ph2d_editor::ids::VECTOR_EXPAND_OFFSET
                );
                // O slider fala FRAÇÃO do tamanho da forma (−100%..+100%); o `d` de mundo nasce
                // de `fração × escala` (a porta única `vec_expand::offset_scale`, `ada45fac`).
                // ⚠️ A escala NÃO precisa mais ser congelada no grab: o preview deixou de
                // churnar a cena, então a bbox das FONTES não se move durante o arrasto.
                let frac = hero
                    .store
                    .slider(ph2d_editor::ids::VECTOR_EXPAND_OFFSET)
                    .map_or(ph2d_tool_vector::params::OFFSET_DEFAULT_FRAC, |(_, v)| {
                        ph2d_tool_vector::params::slider_to_offset_frac(v)
                    });
                if offset_grabbed && !sel.is_empty() {
                    let xf = crate::vec_transform::build(sim, &self.vec_entities);
                    let d = frac * crate::vec_expand::offset_scale(vec_scene, &self.vec_pen, &xf);
                    crate::offset_live::arm(sim, &self.vec_entities, &sel, d, knobs.0, knobs.1);
                    self.vec_offset_mirrored = (sel.len() == 1).then(|| sel[0]);
                }
                // Um chip de Corner/Side clicado RETUNA os offsets vivos da seleção — e só
                // eles: sem offset armado, o chip arma o próximo arrasto e não inventa
                // geometria de lugar nenhum. Comparar com o quadro anterior é o que distingue
                // "o artista clicou" de "o painel está no valor de sempre".
                if knobs != self.vec_expand_knobs {
                    self.vec_expand_knobs = knobs;
                    crate::offset_live::retune(sim, &self.vec_entities, &sel, knobs);
                }
            }
            // ── A LARGURA VIVA (ADR-0148) ────────────────────────────────────────
            // Os quatro sliders `W Start/Mid/End/Pos` deixaram de ser parâmetros de um comando
            // e passaram a AUTORAR um perfil vivo: o traço engrossa e afina enquanto o slider
            // anda, e o botão *Power Stroke* MATERIALIZA — o mesmo par que o Offset já tinha.
            {
                let sel: Vec<ph2d_vec_scene::VecPathId> = self.vec_pen.selected_paths().to_vec();
                // **O painel espelha o que está SELECIONADO** — a mesma lei (e a mesma ordem) do
                // offset acima: a borda é a SELEÇÃO, e ela corre ANTES de o arrasto ser lido.
                let mirror = (sel.len() == 1).then(|| sel[0]).filter(|id| {
                    crate::profile_live::spec_of(sim, &self.vec_entities, *id).is_some()
                });
                if mirror != self.vec_profile_mirrored {
                    self.vec_profile_mirrored = mirror;
                    if let Some(p) = mirror
                        .and_then(|id| crate::profile_live::spec_of(sim, &self.vec_entities, id))
                        .as_ref()
                        .and_then(crate::profile_live::preset_of)
                    {
                        crate::profile_live::write_preset_to_store(&mut hero.store, &p);
                    }
                }
                // **O catálogo de perfis** (W2b) — escolher uma FORMA pelo nome. Corre ANTES do
                // arrasto de propósito: escrever os quatro sliders é o que faz a fileira acender
                // e os knobs mostrarem o que a forma passou a ser, e o armamento abaixo o
                // relê pela mesma porta. As duas metades — os sliders e o documento — têm de
                // andar juntas, senão o painel diria uma coisa e a tela outra.
                if let Some(p) = pending_width_preset
                    .and_then(|i| ph2d_vec_scene::WIDTH_PRESETS.get(i))
                    .filter(|_| !sel.is_empty())
                {
                    crate::profile_live::write_preset_to_store(&mut hero.store, &p.profile);
                    crate::profile_live::arm(sim, &self.vec_entities, &sel, &p.profile.to_stops());
                    // O espelho da seleção acabou de ser ESCRITO por nós: sem isto o bloco do
                    // frame seguinte veria o `mirror` inalterado e não reescreveria nada — mas
                    // com uma seleção de uma forma só ele passaria a divergir na primeira troca.
                    self.vec_profile_mirrored = (sel.len() == 1).then(|| sel[0]);
                }
                let grabbed = matches!(
                    hero.store.active_id(),
                    Some(id) if id == ph2d_editor::ids::VECTOR_EXPAND_W_START
                        || id == ph2d_editor::ids::VECTOR_EXPAND_W_MID
                        || id == ph2d_editor::ids::VECTOR_EXPAND_W_END
                        || id == ph2d_editor::ids::VECTOR_EXPAND_W_POS
                );
                if grabbed && !sel.is_empty() {
                    let stops = crate::profile_live::preset_from_store(&hero.store).to_stops();
                    crate::profile_live::arm(sim, &self.vec_entities, &sel, &stops);
                    self.vec_profile_mirrored = (sel.len() == 1).then(|| sel[0]);
                }
            }
            if let Some(cmd) = pending_vec_expand {
                let xf = crate::vec_transform::build(sim, &self.vec_entities);
                // **Apply Offset MATERIALIZA o offset vivo** — é o único momento em que os
                // vértices do offset passam a existir no documento (Enio, 2026-07-21). Cada
                // forma é assada com o `VecOffset` DELA, e não com o slider: duas formas podem
                // carregar offsets diferentes, e o botão tem de honrar o que está na TELA.
                // Passa pela MESMA porta do caminho numérico (`expand_selection`), senão
                // haveria uma 2ª maneira de a geometria do offset entrar na cena.
                // **O botão Power Stroke MATERIALIZA o perfil vivo** (ADR-0148) — o espelho
                // exato do Apply Offset logo abaixo. Sem perfil armado na seleção devolve
                // `false`, e o clique segue pelo caminho numérico (que lê os sliders).
                let sel_now: Vec<ph2d_vec_scene::VecPathId> =
                    self.vec_pen.selected_paths().to_vec();
                if matches!(cmd, crate::vec_expand::Expand::PowerStroke { .. })
                    && crate::profile_live::materialise(
                        vec_scene,
                        sim,
                        &mut self.vec_pen,
                        &mut self.vec_history,
                        &self.vec_entities,
                        &xf,
                        &sel_now,
                    )
                {
                    // A forma nova não tem perfil vivo, e knobs parados num afinamento sobre ela
                    // mentiriam sobre o que está na cena — o mesmo argumento do slider de Offset.
                    self.vec_profile_mirrored = None;
                    crate::profile_live::write_preset_to_store(
                        &mut hero.store,
                        &ph2d_vec_scene::WidthProfile::UNIFORM,
                    );
                    return;
                }
                let materialised = matches!(cmd, crate::vec_expand::Expand::Offset { .. }) && {
                    let ids: Vec<ph2d_vec_scene::VecPathId> =
                        self.vec_pen.selected_paths().to_vec();
                    crate::offset_live::materialise(
                        vec_scene,
                        sim,
                        &mut self.vec_pen,
                        &mut self.vec_history,
                        &self.vec_entities,
                        &xf,
                        &ids,
                    )
                };
                if materialised {
                    self.vec_offset_mirrored = None;
                    // O slider volta ao zero: a forma nova não tem offset vivo, e um slider
                    // parado em +40% sobre ela mentiria sobre o que está na cena.
                    hero.store.set_slider_value(
                        ph2d_editor::ids::VECTOR_EXPAND_OFFSET,
                        ph2d_tool_vector::params::offset_frac_to_slider(0.0),
                    );
                } else {
                    // O caminho NUMÉRICO (sem offset vivo armado): a distância vem do slider —
                    // a MESMA fonte que o chip mostra —, fração × escala da seleção atual.
                    let d = hero
                        .store
                        .slider(ph2d_editor::ids::VECTOR_EXPAND_OFFSET)
                        .map_or(ph2d_tool_vector::params::OFFSET_DEFAULT_FRAC, |(_, v)| {
                            ph2d_tool_vector::params::slider_to_offset_frac(v)
                        })
                        * crate::vec_expand::offset_scale(vec_scene, &self.vec_pen, &xf);
                    // ⚠️ O PERFIL também vem dos sliders — a mesma fonte que o chip mostra. O
                    // `expand_for_id` devolve o comando com o perfil UNIFORME (ele não tem o
                    // store), e é aqui que ele é preenchido; um default cravado lá seria um 2º
                    // lugar decidindo o que o artista já arrastou.
                    let cmd = match cmd {
                        crate::vec_expand::Expand::PowerStroke { .. } => {
                            crate::vec_expand::Expand::PowerStroke {
                                stops: crate::profile_live::preset_from_store(&hero.store)
                                    .to_stops(),
                            }
                        }
                        other => other,
                    };
                    crate::vec_expand::apply_vec_expand(
                        vec_scene,
                        &mut self.vec_history,
                        &mut self.vec_pen,
                        &xf,
                        cmd.clone(),
                        d,
                    );
                    // O botão de Offset (caminho numérico: arrastar sem seleção viva e clicar)
                    // recentra o slider — cada aplicação offseta pelo valor mostrado e zera.
                    if matches!(cmd, crate::vec_expand::Expand::Offset { .. }) {
                        hero.store.set_slider_value(
                            ph2d_editor::ids::VECTOR_EXPAND_OFFSET,
                            ph2d_tool_vector::params::offset_frac_to_slider(0.0),
                        );
                    }
                }
            }
            if let Some(make) = pending_vec_compound {
                crate::input_dispatch::apply_vec_compound(
                    vec_scene,
                    &mut self.vec_history,
                    &mut self.vec_pen,
                    make,
                );
            }
            if let Some(even_odd) = pending_vec_fill_rule {
                crate::input_dispatch::apply_vec_fill_rule(
                    vec_scene,
                    &mut self.vec_history,
                    &self.vec_pen,
                    even_odd,
                );
            }
            // Snap settings are TOOL state, not document state — no undo step.
            if let Some(on) = pending_vec_snap_on {
                self.vec_snap.on = on;
            }
            if let Some(kind) = pending_vec_vertex_kind {
                crate::input_dispatch::apply_vec_vertex_kind(
                    vec_scene,
                    &mut self.vec_history,
                    &mut self.vec_pen,
                    kind,
                );
            }
            if pending_vec_delete_vertex {
                crate::input_dispatch::apply_vec_delete_vertex(
                    vec_scene,
                    &mut self.vec_history,
                    &mut self.vec_pen,
                );
            }
            // ⚠️ **Sem `vec_history.begin`**: os dois só mudam QUEM está selecionado, e a seleção
            // não é estado de documento — abrir um passo de undo por eles poria uma linha na fila
            // que o Ctrl+Z não teria o que desfazer.
            if pending_vec_select_subpath {
                self.vec_pen.select_subpath_verts(vec_scene);
            }
            if pending_vec_select_same {
                self.vec_pen.select_verts_of_same_kind(vec_scene);
            }
            if let Some(order) = pending_vec_reorder {
                crate::input_dispatch::apply_vec_reorder(
                    vec_scene,
                    &mut self.vec_history,
                    &self.vec_pen,
                    order,
                );
            }
            if pending_vec_duplicate {
                // Offset the clone by a fixed SCREEN distance (px → world) so it's
                // visibly separated at any zoom.
                const OFFSET_PX: f64 = 12.0;
                let off = OFFSET_PX * vec_px_to_world;
                crate::input_dispatch::apply_vec_duplicate(
                    vec_scene,
                    &mut self.vec_history,
                    &mut self.vec_pen,
                    off,
                    off,
                );
            }
            if let Some(axis) = pending_vec_flip {
                crate::input_dispatch::apply_vec_flip(
                    vec_scene,
                    &mut self.vec_history,
                    &self.vec_pen,
                    axis,
                );
            }
            if let Some(dir) = pending_vec_rotate {
                crate::input_dispatch::apply_vec_rotate(
                    vec_scene,
                    &mut self.vec_history,
                    &self.vec_pen,
                    dir,
                );
            }
            // O afim de cada path, para as operações que falam MUNDO (align, distribute,
            // campos X/Y/W/H). O mapa é o do frame passado — os paths envolvidos já
            // existem, então basta.
            let vec_xf_ops = crate::vec_transform::build(sim, &self.vec_entities);
            if let Some((field, target)) = pending_vec_transform {
                crate::input_dispatch::apply_vec_transform(
                    vec_scene,
                    &mut self.vec_history,
                    &self.vec_pen,
                    &vec_xf_ops,
                    field,
                    target,
                );
            }
            if let Some(deg) = pending_vec_rotate_by {
                crate::input_dispatch::apply_vec_rotate_by(
                    vec_scene,
                    &mut self.vec_history,
                    &self.vec_pen,
                    deg,
                );
            }
            // Configs de texto: aplicam na SESSÃO viva; sem sessão, no objeto de TEXTO
            // SELECIONADO (o texto segue editável no Select até virar curva). O
            // `vec_text_sel` é a seleção corrente para o caminho do objeto.
            let vec_text_sel: Vec<ph2d_vec_scene::VecPathId> =
                self.vec_pen.selected_paths().to_vec();
            // **O conector, pelo painel.** Editar um campo FIXA o valor (`None` → `Some`) em
            // TODOS os conectores selecionados — é o que permite calibrar o diagrama inteiro
            // de uma vez, em vez de linha por linha. A geometria não é escrita aqui: ela é
            // função pura da relação, e o `connector_live::recook` deste mesmo frame (mais
            // abaixo) a refaz. O undo global pega a mudança pelo diff do mundo ECS.
            if let Some((id, v)) = pending_vec_connector {
                crate::vec_connector_panel::edit_selected_connectors(
                    sim,
                    &self.vec_entities,
                    &vec_text_sel,
                    id,
                    v,
                );
            }
            // Live Shapes: os sliders de forma editam a forma VIVA selecionada — muda o
            // parâmetro e RE-COZINHA in-place (id/estilo/pose preservados). Sem forma
            // viva na seleção, o slider só moveu o default de desenho (a tool já o
            // guardou) — é o que fecha o ciclo paramétrico: um polígono de 5 lados vira
            // de 7 depois de desenhado.
            if let Some((id, v)) = pending_vec_shape_param {
                crate::vec_shape_params::edit_selected_shape(
                    sim,
                    vec_scene,
                    &self.vec_entities,
                    &vec_text_sel,
                    |kind, values| {
                        crate::vec_shape_params::apply_shape_field(
                            kind,
                            values,
                            id,
                            v,
                            vec_px_to_world,
                        )
                    },
                );
            }
            let editing_session = self.vec_text_edit.is_some();
            if let Some(size) = pending_vec_text_size {
                crate::vec_text::apply_text_size(
                    &mut self.vec_text_edit,
                    &mut self.vec_text_size,
                    vec_scene,
                    size,
                );
                if !editing_session {
                    crate::vec_text::edit_selected_text(
                        sim,
                        vec_scene,
                        &self.vec_entities,
                        &vec_text_sel,
                        |p| p.size = size,
                    );
                }
            }
            if let Some(weight) = pending_vec_text_weight {
                crate::vec_text::apply_text_weight(
                    &mut self.vec_text_edit,
                    &mut self.vec_text_weight,
                    vec_scene,
                    weight,
                );
                if !editing_session {
                    crate::vec_text::edit_selected_text(
                        sim,
                        vec_scene,
                        &self.vec_entities,
                        &vec_text_sel,
                        |p| p.weight = weight,
                    );
                }
            }
            if let Some(lh) = pending_vec_text_line_height {
                crate::vec_text::apply_text_line_height(
                    &mut self.vec_text_edit,
                    &mut self.vec_text_line_height,
                    vec_scene,
                    lh,
                );
                if !editing_session {
                    crate::vec_text::edit_selected_text(
                        sim,
                        vec_scene,
                        &self.vec_entities,
                        &vec_text_sel,
                        |p| p.line_height = lh,
                    );
                }
            }
            if let Some(tr) = pending_vec_text_tracking {
                crate::vec_text::apply_text_tracking(
                    &mut self.vec_text_edit,
                    &mut self.vec_text_tracking,
                    vec_scene,
                    tr,
                );
                if !editing_session {
                    crate::vec_text::edit_selected_text(
                        sim,
                        vec_scene,
                        &self.vec_entities,
                        &vec_text_sel,
                        |p| p.tracking = tr,
                    );
                }
            }
            if let Some((i, v)) = pending_vec_text_axis
                && !editing_session
            {
                crate::vec_text::edit_selected_text(
                    sim,
                    vec_scene,
                    &self.vec_entities,
                    &vec_text_sel,
                    |p| {
                        if let Some(a) = p.axes.get_mut(i) {
                            a.1 = v as f32;
                        }
                    },
                );
            }
            if let Some(align) = pending_vec_text_align {
                if !editing_session {
                    crate::vec_text::edit_selected_text(
                        sim,
                        vec_scene,
                        &self.vec_entities,
                        &vec_text_sel,
                        |p| p.align = crate::vec_text::align_to_u8(align),
                    );
                }
                crate::vec_text::apply_text_align(
                    &mut self.vec_text_edit,
                    &mut self.vec_text_align,
                    vec_scene,
                    align,
                );
            }
            // A família "corrente" para o ciclo `<`/`>` é a do ALVO: o objeto de texto
            // selecionado (sem sessão) ou o default da shell.
            let cur_family = if editing_session {
                self.vec_text_family.clone()
            } else {
                crate::vec_text::selected_text_object(sim, &self.vec_entities, &vec_text_sel)
                    .map_or_else(|| self.vec_text_family.clone(), |(_, _, p)| p.family)
            };
            if let Some(dir) = pending_vec_font_cycle {
                let next = crate::vec_font::cycle_family(cur_family.as_deref(), dir);
                if !editing_session {
                    crate::vec_text::set_selected_text_font(
                        sim,
                        vec_scene,
                        &self.vec_entities,
                        &vec_text_sel,
                        next.clone(),
                    );
                }
                crate::vec_text::set_text_font(
                    &mut self.vec_text_edit,
                    &mut self.vec_text_family,
                    &mut self.vec_text_extra_axes,
                    vec_scene,
                    next,
                );
            }
            if let Some(i) = pending_vec_font_pick {
                // Índice na MESMA lista que gerou as previews → família escolhida.
                let family = crate::vec_font::pickable_families()
                    .get(i)
                    .cloned()
                    .flatten();
                if !editing_session {
                    crate::vec_text::set_selected_text_font(
                        sim,
                        vec_scene,
                        &self.vec_entities,
                        &vec_text_sel,
                        family.clone(),
                    );
                }
                crate::vec_text::set_text_font(
                    &mut self.vec_text_edit,
                    &mut self.vec_text_family,
                    &mut self.vec_text_extra_axes,
                    vec_scene,
                    family,
                );
            }
            if let Some((index, value)) = pending_vec_text_axis {
                crate::vec_text::apply_text_axis(
                    &mut self.vec_text_edit,
                    &mut self.vec_text_extra_axes,
                    vec_scene,
                    index,
                    value,
                );
            }
            if pending_vec_font_import {
                let imported = crate::vec_text::import_text_font(
                    &mut self.vec_text_edit,
                    &mut self.vec_text_family,
                    &mut self.vec_text_extra_axes,
                    vec_scene,
                );
                // Sem sessão, a fonte importada vai para o objeto de texto SELECIONADO.
                if imported && !editing_session {
                    let fam = self.vec_text_family.clone();
                    crate::vec_text::set_selected_text_font(
                        sim,
                        vec_scene,
                        &self.vec_entities,
                        &vec_text_sel,
                        fam,
                    );
                }
                // A fonte importada entra no dropdown: reconstrói as previews agora.
                #[cfg(feature = "panel-vector")]
                if imported {
                    ph2d_panel_vector::set_current_text_font_previews(
                        crate::vec_font_preview::build_previews(),
                    );
                }
                #[cfg(not(feature = "panel-vector"))]
                let _ = imported;
            }
            if let Some(op) = pending_vec_path_shape {
                crate::input_dispatch::apply_vec_path_shape(
                    vec_scene,
                    &mut self.vec_history,
                    &self.vec_pen,
                    op,
                );
            }
            if pending_vec_toggle_closed {
                crate::input_dispatch::apply_vec_toggle_closed(
                    vec_scene,
                    &mut self.vec_history,
                    &self.vec_pen,
                );
            }
            if let Some(kind) = pending_vec_fill_kind {
                crate::input_dispatch::apply_vec_set_fill_kind(
                    vec_scene,
                    &mut self.vec_history,
                    &self.vec_pen,
                    kind,
                );
                // The old handle no longer addresses the new fill kind — reset the
                // gradient selection so the overlay highlight + panel don't cling to it.
                self.vec_grad_selected = None;
                self.vec_grad_drag = None;
            }
            if let Some(deg) = pending_vec_grad_angle {
                crate::input_dispatch::apply_vec_set_grad_angle(
                    vec_scene,
                    &mut self.vec_history,
                    &self.vec_pen,
                    deg,
                );
            }
            if pending_vec_grad_add {
                crate::input_dispatch::apply_vec_grad_add_point(
                    vec_scene,
                    &mut self.vec_history,
                    &self.vec_pen,
                );
            }
            if pending_vec_grad_remove {
                self.vec_grad_selected = crate::input_dispatch::apply_vec_grad_remove_point(
                    vec_scene,
                    &mut self.vec_history,
                    &self.vec_pen,
                    self.vec_grad_selected
                        .and_then(ph2d_vec_render::GradHandle::point),
                )
                .map(ph2d_vec_render::GradHandle::Point);
            }
            if let Some(v) = pending_vec_grad_influence {
                crate::input_dispatch::apply_vec_grad_influence(
                    vec_scene,
                    &mut self.vec_history,
                    &self.vec_pen,
                    self.vec_grad_selected
                        .and_then(ph2d_vec_render::GradHandle::point),
                    v,
                );
            }
            if let Some(v) = pending_vec_grad_jitter {
                crate::input_dispatch::apply_vec_grad_jitter(
                    vec_scene,
                    &mut self.vec_history,
                    &self.vec_pen,
                    self.vec_grad_selected
                        .and_then(ph2d_vec_render::GradHandle::point),
                    v,
                );
            }
            if pending_vec_grad_add_stop {
                self.vec_grad_selected = crate::input_dispatch::apply_vec_grad_add_stop(
                    vec_scene,
                    &mut self.vec_history,
                    &self.vec_pen,
                )
                .map(ph2d_vec_render::GradHandle::Stop)
                .or(self.vec_grad_selected);
            }
            if let Some(a) = pending_vec_align {
                crate::input_dispatch::apply_vec_align(
                    vec_scene,
                    &mut self.vec_history,
                    &self.vec_pen,
                    &vec_xf_ops,
                    a,
                );
            }
            if let Some(d) = pending_vec_distribute {
                crate::input_dispatch::apply_vec_distribute(
                    vec_scene,
                    &mut self.vec_history,
                    &self.vec_pen,
                    &vec_xf_ops,
                    d,
                );
            }
            if pending_vec_grad_remove_stop
                && let Some(si) = self
                    .vec_grad_selected
                    .and_then(ph2d_vec_render::GradHandle::stop)
            {
                // Only an interior stop can be removed; a no-op otherwise keeps the
                // current selection (endpoint handles aren't removable stops).
                self.vec_grad_selected = crate::input_dispatch::apply_vec_grad_remove_stop(
                    vec_scene,
                    &mut self.vec_history,
                    &self.vec_pen,
                    Some(si),
                )
                .map(ph2d_vec_render::GradHandle::Stop);
            }
            if pending_vec_pivot_edit {
                // Arma "Set Center": a próxima pressão no canvas põe a ORIGEM ali.
                self.vec_pivot_edit = true;
            }
            let vec_cfg = vector_bridge::dispatch(
                hero,
                tools,
                vec_scene,
                &mut self.vec_pen,
                &mut self.vec_shape,
                &mut self.vec_pencil,
                &mut self.vec_history,
                vec_px_to_world,
                self.vec_grad_selected,
                &vec_xf_ops,
                sim,
                &self.vec_entities,
                self.vec_pivot_edit,
                self.vec_snap.on,
            );
            // Motion Nodes M0.T10: same phase as vector_bridge (AFTER the
            // ActivateTool drain, so a freshly-activated tool is seen this frame;
            // BEFORE paint + present, so the split/panel visibility it sets and
            // the instances it cooks both land this frame). Cooks the graph into
            // `motion.instances` (present injects them via `render_with_extra`)
            // and drives the center split + docked-panel visibility.
            // The drawn shapes go into the cook BEFORE it runs (doc 65): every named vector path
            // becomes an external the graph can walk (`motion.path`). Here, because this is the
            // one place the document, the world, the entity map and the transforms are all in
            // hand at once.
            motion_bridge::publish_shapes(motion, sim, vec_scene, &self.vec_entities, &vec_xf_ops);
            motion_bridge::dispatch(
                hero,
                tools,
                motion,
                &mut self.playhead,
                self.fixed_step.fixed_dt(),
                self.last_pointer,
                toasts,
                surface.gpu(),
            );
            // Mirror the tool's mode + shape params for the input dispatch's
            // pen-vs-shape routing (the downcast lives in the bridge).
            self.vec_draw_config = vec_cfg;

            // ADR-0114 W2 T2.17 (ready-to-smoke): ativar a tool Flip num documento
            // VAZIO cria um objeto inicial (1 camada) pra desenhar na hora — sem
            // ele o `bake_stroke` sai (não há objeto). Só na borda de ativação;
            // um doc já povoado (ex. PH2D_FLIP_DEMO) não é tocado.
            {
                let now_active = tools
                    .active()
                    .is_some_and(|t| t.id() == ph2d_editor::ToolId::new("flip"));
                if now_active && !self.flip_active && flip.is_empty() {
                    let oid = flip.push_object("Flip");
                    if let Some(obj) = flip.object_mut(oid) {
                        self.flip_active_layer = Some(obj.add_layer("Layer 1"));
                    }
                }
            }
            // ADR-0114 W2: espelha o estado da tool Flip (ativa + estilo de brush)
            // pro input_dispatch decidir/assar o desenho sem downcast (o downcast
            // vive no flip_bridge, allowlistado).
            // Physics WORLD panel (ADR-0131 D8 / W2b). Same phase as the other
            // panel bridges — after the ActivateTool drain, before paint — but
            // deliberately NOT tool-gated: this panel belongs to the document,
            // so the artist owns its visibility and this call never writes it.
            //
            // ⚠️ Distinct from `physics_bridge::dispatch` far above, which steps
            // the SIMULATION at the Playhead tick. Two bridges, two phases.
            self.show_colliders = physics_panel_bridge::dispatch(
                hero,
                physics,
                self.show_colliders,
                &mut self.interaction,
            );
            // O ARRASTO da tira (mover a chave / esticar o hold): o painel enfileirou o
            // pedido no pen-up do frame anterior; aqui ele vira documento — ANTES do
            // publish, senão o snapshot deste frame descreveria a tira de antes do gesto e
            // a célula piscaria de volta por um frame.
            // (o retorno diz se o documento mudou; ninguém precisa dele aqui — o undo é
            // GLOBAL e por DIFF: `post_frame_undo` compara o `ProjectState`, do qual o
            // `FlipDoc` faz parte. É o mesmo motivo pelo qual o drain do `PanelEvent`
            // logo acima também ignora o seu.)
            let _ = crate::flip_strip_drag::apply_strip_intents(
                flip,
                self.flip_active_layer,
                &mut self.flip_strip,
            );
            let (flip_active, flip_style) = flip_bridge::publish(
                hero,
                tools,
                flip,
                self.flip_active_layer,
                &self.playhead,
                &self.flip_strip,
            );
            self.flip_active = flip_active;
            self.flip_style = flip_style;
            // O anel do pincel (W5): mostra no canvas o tamanho do que vai acontecer.
            // Depois do publish (o estilo do frame já está no cache) e na cena de
            // overlay, como o anel do Painter.
            flip_cursor::draw_flip_cursor(
                flip_active,
                flip_style,
                hero,
                vector_scene,
                self.last_pointer,
                // §4.C.6: o Size mede o MUNDO — o anel se projeta pelo zoom, como a tinta.
                f64::from(window_size.height as f32 / camera.height_world.max(f32::EPSILON)),
            );
            // O contorno dos colliders: um sprite é um QUAD e um collider é
            // invisível, então sem isto "que forma isto tem, fisicamente?"
            // não tem resposta na tela (Enio, 2026-07-18). No-op sem corpos.
            // Joints too — a joint is a RELATIONSHIP with no geometry at all,
            // so two objects pinned together look exactly like two that merely
            // touch. The anchors come from the solver, not from the joint
            // entity's Transform (see `physics_overlay::joint_marks`).
            //
            // W-J1: a VIEW, not just the anchor pair — the drawing reads the
            // very `JointDesc` the solver was handed (kind, limits, motor,
            // length) plus the live poses, so the glyph cannot describe a joint
            // the solver is not enforcing.
            let joint_views: Vec<ph2d_physics_ecs::JointView> = physics.joint_views().collect();
            // A corda frouxa pendura para onde as coisas caem — a MESMA fonte
            // que decide a superfície de uma poça (W-Buoyancy).
            let joint_gravity = {
                let s = physics.settings();
                [s.gravity_x, s.gravity_y]
            };
            // Sensors that have a body inside them THIS frame — the overlay
            // lights them up. A sensor with nothing reading its overlaps would
            // be a dead flag (W7), so the visible reaction lives here.
            let triggered = physics.triggered_sensors();
            // The initial-velocity arrow is only truthful before the sim steps:
            // once a body has moved, its live velocity is no longer the authored
            // launch. `last_stepped() == 0` is exactly "the bodies are at their
            // authored rest", the same fact the bridge uses to decide a respawn.
            let velocity_at_rest = physics.last_stepped() == 0;
            // Where bodies actually TOUCH, and how hard they press (W-Contacts). A
            // contact exists only while two shapes meet, and nothing else on screen
            // says whether two objects are resting on each other or just overlapping
            // in the artist's eye.
            let contacts = physics.contacts().to_vec();
            // Onde o cursor está, em mundo — a âncora da mira das ferramentas de
            // ponto (W-Hand). Derivada aqui e não guardada: o `last_pointer` é a
            // única fonte, e uma cópia dela desenharia a mira onde o mouse ESTAVA.
            let pointer_world = camera.screen_to_world(self.last_pointer, window_size);
            // The begin-flashes (`×`) — the visible half of the contact-events channel,
            // a separate list from the standing `+` crosses because a flash marks a
            // BEGINNING and outlives the tick it was born in (W-TickContacts).
            let flashes = physics.contact_flashes().to_vec();
            // Onde a água está. O empuxo calcula essa superfície todo frame e, até
            // isto, nada na tela a mostrava — o artista posicionava o que boia no olho.
            let waterlines = physics.waterlines();
            physics_overlay::draw(
                self.show_colliders,
                velocity_at_rest,
                sim,
                &joint_views,
                joint_gravity,
                // W-J3: o limite que o arrasto está posando AGORA, para o
                // fantasma de B. Lido do componente (o arrasto já escreveu nele
                // neste frame), então a silhueta e o arco mostram o mesmo número.
                self.joint_anchor_drag.and_then(|d| d.posed_limit(sim)),
                // W-J4: a banda elástica, se um gesto de criar está em voo (e o
                // corpo A ainda existe — apagá-lo sob o gesto o invalida).
                crate::joint_draw::body_alive(sim, self.joint_draw)
                    .then(|| crate::joint_draw::band(self.joint_draw))
                    .flatten(),
                // W-Grab: a mola da mão, lida do ÚNICO dono do fato (a ponte);
                // o ponto de pega é derivado da pose VIVA do corpo, então o
                // zigzag acompanha o que a mola está de fato puxando.
                physics.grab_marks(),
                // W-Hand: a MIRA da ferramenta de ponto em mãos. `aim_radius` é
                // `None` para a mão; e o gesto só é oferecido com o relógio
                // ANDANDO e a física ARMADA, então a mira honra as MESMAS duas
                // condições que `body_grab::poke_at` — uma mira que promete o que
                // o clique não faz é pior que mira nenhuma.
                (self.playhead.is_playing() && self.timeline.flags.simulate_physics)
                    .then(|| self.interaction.aim_radius())
                    .flatten()
                    .map(|r| (pointer_world, r)),
                // O campo VIVO, do ÚNICO dono do fato (a ponte).
                physics.attract_marks(),
                // E o último estouro, enquanto o flash dura.
                self.blast_flash.map(|(c, r, _)| (c, r)),
                &contacts,
                &flashes,
                &waterlines,
                &triggered,
                // W-J7b: o joint selecionado ganha readout mesmo sem teto armado
                // — é preciso ler a carga ANTES de escolher um número.
                hero.gizmo
                    .iter_selected()
                    .map(ph2d_ecs::Entity::from_bits)
                    .find(|e| {
                        sim.world()
                            .get::<ph2d_physics_ecs::PhysicsJoint>(*e)
                            .is_some()
                    }),
                camera,
                window_size,
                vector_scene,
                // ⚠️ Reborrow por `paint_ctx`, não o binding cru: o `text_system`
                // já está emprestado por ele desde o começo do frame, e um
                // segundo empréstimo direto não compila. O reborrow morre com a
                // chamada, que é exatamente o tempo de vida que o rótulo precisa.
                paint_ctx.text,
            );
            // A TRAJETÓRIA do objeto selecionado (ADR-0141): um binding Position guarda
            // uma curva, e sem desenhá-la o artista vê o objeto aparecer noutro lugar a
            // cada frame sem ter onde pegar o caminho. Os PONTOS são um por quadro, e o
            // espaçamento entre eles é a velocidade. No-op sem seleção ou sem Position.
            motion_path_overlay::draw(
                true,
                &self.timeline.doc,
                hero.gizmo.iter_selected().next(),
                camera,
                window_size,
                vector_scene,
            );
            // O ONION da timeline (ADR-0142): as poses-fantasma do objeto selecionado em
            // t±k, cozidas AQUI (temos sim/present/doc/seleção) e desenhadas pelo passe de
            // sprite em `run_present_phase` — o padrão do Motion (cozinha numa fase,
            // desenha noutra). No-op quando desligado / sem seleção animada.
            // Onion settings modal (ADR-0142 W3b): while the card is open, read its slider/swatch
            // values back into the onion each frame — live edits on the canvas (the ghost pass
            // below re-reads `self.timeline.onion`). No-op when closed. The store is the shared
            // blackboard; `enabled`/`mode` stay owned by the transport toggles.
            crate::onion_modal::read_into(&hero.store, &mut self.timeline.onion);
            self.onion_ghosts.clear();
            timeline_onion::collect_onion_ghosts(
                &self.timeline.onion,
                sim.world(),
                present,
                &self.timeline.doc,
                hero.gizmo.iter_selected().next(),
                self.playhead.time(),
                &mut self.onion_ghosts,
            );
            // O realce da seleção (W6): uma seleção que não se VÊ não existe. Overlay
            // (chrome), nunca render de traço — ver o cabeçalho do módulo.
            {
                let l2w = flip
                    .objects()
                    .first()
                    .map(|o| o.id)
                    .and_then(|oid| self.flip_entities.get(&oid).copied())
                    .map(ph2d_ecs::Entity::from_bits)
                    .filter(|e| sim.world().get_entity(*e).is_ok())
                    .map_or(ph2d_vec_scene::Xform::IDENTITY, |e| {
                        crate::flip_transform::object_xform(sim, e)
                    });
                // W8/§4.C: o realce fala a linguagem do DOMÍNIO — halo de traço (Stroke),
                // dots (Point), ou halo do PEDAÇO + preview de hover (Segment).
                let overlay_domain = match flip_style.map(|s| s.edit_domain) {
                    Some(ph2d_tool_flip::EditDomain::Point) => {
                        flip_selection_overlay::OverlayDomain::Point
                    }
                    Some(ph2d_tool_flip::EditDomain::Segment) => {
                        flip_selection_overlay::OverlayDomain::Segment
                    }
                    _ => flip_selection_overlay::OverlayDomain::Stroke,
                };
                let hover = self
                    .flip_segment_hover
                    .as_ref()
                    .map(|(si, pts)| (*si, pts.as_slice()));
                flip_selection_overlay::draw_flip_selection(
                    flip_active,
                    matches!(
                        flip_style.map(|s| s.mode),
                        Some(ph2d_tool_flip::FlipMode::Edit)
                    ),
                    overlay_domain,
                    hover,
                    flip,
                    &self.playhead,
                    self.flip_active_layer,
                    &l2w,
                    camera,
                    surface.size(),
                    vector_scene,
                );
                // A caixa do marquee (W6.1) — em px de tela, como o realce.
                flip_selection_overlay::draw_flip_marquee(self.flip_edit_gesture, vector_scene);

                // Tween v2 — a correção de pares: os dois desenhos-chave sobrepostos + as
                // linhas de par (pela confiança) + órfãos, no MESMO `l2w` do objeto. Só
                // desenha com a sessão Pairs aberta.
                flip_tween_overlay::draw(
                    flip_active && self.flip_strip.tween_correct.is_some(),
                    self.flip_strip.tween_correct.as_ref(),
                    &l2w,
                    camera,
                    surface.size(),
                    vector_scene,
                );

                // Gap Closure (doc 06 §8): os helpers ao vivo — cada vão que o alcance
                // atual fecha, desenhado onde o clique vai fechá-lo. Os segmentos vêm do
                // worker (`flip_gap_live`, coords de ARTE); a pergunta do modo é a MESMA
                // porta do tick, e a projeção é a MESMA cadeia do render (l2w ∘ pose).
                flip_gap_overlay::draw(
                    crate::flip_gap_live::wants_gap_helpers(flip_active, flip_style),
                    &self.flip_gap.segments,
                    &l2w,
                    // A MESMA pose que a autoria dobra (`flip_transform::active_pose`) —
                    // função livre porque aqui `self.gfx` está destruturado.
                    crate::flip_transform::active_pose(
                        flip,
                        self.flip_active_layer,
                        &self.playhead,
                    ),
                    camera,
                    surface.size(),
                    vector_scene,
                );
            }

            // Texto em edição herda o Style do painel em TEMPO REAL: o bridge acabou
            // de copiar Fill/Stroke/Width/Cap/Join do painel para o Pen; se mudou,
            // regenera os glyphs da sessão com o novo Paint (antes do `sync`, para as
            // entidades reconciliarem os paths novos neste mesmo frame). Sair do modo
            // Text (inclusive pelo botão do painel) COMMITA a sessão — senão o recolor
            // de multisseleção pegaria letras não-selecionadas e o gizmo sumiria.
            crate::vec_text::sync_active_text_style(
                &mut self.vec_text_edit,
                self.vec_draw_config.mode,
                &self.vec_pen,
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
                let in_text_mode = self.vec_draw_config.mode == ph2d_tool_vector::DrawMode::Text;
                let sel: Vec<ph2d_vec_scene::VecPathId> = self.vec_pen.selected_paths().to_vec();
                let target = crate::vec_text::panel_text_target(
                    sim,
                    &self.vec_entities,
                    &sel,
                    self.vec_text_edit.as_ref(),
                );
                let visible = in_text_mode || target.is_some();
                ph2d_panel_vector::set_current_text_visible(visible);
                ph2d_panel_vector::set_current_text(target.as_ref().map(|t| t.text.clone()));
                // Família / alinhamento: do alvo; sem alvo (modo Text sem sessão), os
                // defaults correntes da shell (o que a próxima sessão vai usar).
                let family = target
                    .as_ref()
                    .map_or_else(|| self.vec_text_family.clone(), |t| t.family.clone());
                ph2d_panel_vector::set_current_text_font(
                    visible.then(|| crate::vec_font::display_name(family.as_deref())),
                );
                ph2d_panel_vector::set_current_text_align(
                    visible.then(|| target.as_ref().map_or(self.vec_text_align, |t| t.align)),
                );
                // Semente dos sliders: só quando o ALVO muda (senão brigaria com o drag).
                let target_id = target.as_ref().map(|t| t.id);
                if target_id != self.vec_text_last_target {
                    self.vec_text_last_target = target_id;
                    ph2d_panel_vector::set_current_text_seed(target.as_ref().map(|t| t.sliders));
                }
                // Eixos de variação da fonte do alvo (nome + range + valor).
                let slots = if visible {
                    let descs = crate::vec_font::variation_axes(family.as_deref());
                    let values: Vec<f32> = target.as_ref().map_or_else(
                        || self.vec_text_extra_axes.iter().map(|(_, v)| *v).collect(),
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
            if self.vec_draw_config.mode == ph2d_tool_vector::DrawMode::Text
                && ph2d_panel_vector::take_want_font_previews()
            {
                ph2d_panel_vector::set_current_text_font_previews(
                    crate::vec_font_preview::build_previews(),
                );
            }

            // ADR-0110 — a árvore do editor é a Hierarquia. Reconcilia documento e
            // entidades (path novo ⇒ entidade; entidade apagada ⇒ path), projeta a
            // ordem de z da árvore na pilha, e lê visibilidade/trava herdadas.
            crate::vec_entities::sync(sim, vec_scene, &mut self.vec_entities);
            // ADR-0114: idem para os objetos Flip (objeto novo ⇒ entidade; entidade
            // apagada ⇒ objeto). No W0 é no-op (nenhuma tool cria objetos ainda); a
            // tool do W2 passa a populá-lo.
            crate::flip_entities::sync(sim, flip, &mut self.flip_entities);
            // Live Shapes: mantém o `VecShape::Text` na entidade do texto ativo (a
            // entidade já existe pós-sync) para o objeto lembrar que é texto — re-cook,
            // painel, Convert e save/undo. Idempotente; só com sessão viva.
            if let Some(edit) = self.vec_text_edit.as_ref() {
                crate::vec_text::upsert_text_shape(sim, &self.vec_entities, edit);
            }
            // Live Shapes: a forma recém-desenhada NASCE VIVA — geometria re-cozida
            // centrada (pivô no centro), pose no `Transform`, `VecShape` na entidade.
            // Antes do `settle` (que pula formas vivas). Idempotente.
            crate::vec_shape_live::make_committed_shape_live(
                sim,
                vec_scene,
                &self.vec_entities,
                &mut self.vec_shape,
            );
            // "Convert to Curves": assa a(s) forma(s) viva(s) selecionada(s) em paths
            // crus — o TEXTO explode num grupo por-letra; as PARAMÉTRICAS descartam o
            // `VecShape` (a geometria já é a forma); e a pilha de EFEITOS é assada no cozido
            // (ADR-0132). A porta única (`vec_convert::to_curves`) usa o MESMO bake do botão
            // "Apply" da seção Effects. Re-seleciona o resultado.
            if pending_vec_convert {
                let sel: Vec<ph2d_vec_scene::VecPathId> = self.vec_pen.selected_paths().to_vec();
                let xf = crate::vec_transform::build(sim, &self.vec_entities);
                let new_sel = crate::vec_convert::to_curves(
                    sim,
                    vec_scene,
                    &mut self.vec_entities,
                    &mut self.vec_pen,
                    &mut self.vec_history,
                    &xf,
                    &sel,
                );
                self.vec_pen.select_many(&new_sel);
            }
            // Habilita "Convert to Curves" pela porta ÚNICA (`vec_convert::is_convertible`) — a
            // MESMA que o conversor honra. Enumerar as fontes aqui foi o que apodreceu duas
            // vezes: o botão ficava desligado num caminho só-efeitos e depois num só-quinas,
            // sempre sem erro nenhum. [[feedback_a_condition_that_enumerates_its_readers_rots]]
            #[cfg(feature = "panel-vector")]
            {
                let convertible = self.vec_pen.selected_paths().iter().any(|id| {
                    crate::vec_convert::is_convertible(sim, &self.vec_entities, vec_scene, *id)
                });
                ph2d_panel_vector::set_current_convertible(convertible);
                // ADR-0129: Expand/Release só são OFERECIDOS quando a seleção é de fato um
                // envelope. A pergunta é a MESMA porta que decide a seleção (selecionar-só-o-
                // container) e executa o dissolve — três consumidores, uma resposta.
                let sel_bits: Vec<u64> = self
                    .vec_pen
                    .selected_paths()
                    .iter()
                    .filter_map(|id| self.vec_entities.get(id).copied())
                    .collect();
                let env_container = crate::envelope_live::sole_container(sim, &sel_bits);
                ph2d_panel_vector::set_current_has_envelope(env_container.is_some());
                // Text on Path (plano 22): as duas perguntas que só a shell sabe responder —
                // *"esta seleção permite prender?"* (um texto + um caminho) e *"o texto em foco
                // já cavalga alguma coisa, e com que valores?"*. A primeira usa a MESMA porta
                // que o clique honra (`link_candidate`), senão o botão apareceria e recusaria.
                let sel = self.vec_pen.selected_paths().to_vec();
                ph2d_panel_vector::set_current_textpath_can_link(
                    crate::vec_text_ride::link_candidate(sim, &self.vec_entities, &sel).is_some(),
                );
                let ride = crate::vec_text_ride::current(sim, &self.vec_entities, &sel);
                ph2d_panel_vector::set_current_textpath(
                    ride.is_some(),
                    ride.map_or(0.0, |r| f64::from(r.start_offset)),
                    ride.is_some_and(|r| r.flip),
                );
                // Pattern on Path (plano 23): as MESMAS duas perguntas — *"esta seleção permite
                // prender?"* (dois caminhos) e *"o motivo em foco já cavalga algo, com que
                // valores?"*. A 1ª usa a MESMA porta que o clique honra (`link_candidate`), a 2ª
                // acha o motivo LINKADO na seleção (não o primário — depois de prender ele pode ser
                // o guia).
                ph2d_panel_vector::set_current_patternpath_can_link(
                    crate::pattern_live::link_candidate(vec_scene, &sel).is_some(),
                );
                let pat = crate::pattern_live::current(sim, &self.vec_entities, &sel);
                // O Picker (Enio 2026-07-23): a porta EXPLÍCITA, oferecida com UM caminho selecionado
                // que ainda não é um motivo vinculado — a fonte à espera do clique do guia.
                // `pat.is_none()` exclui o motivo já preso (que mostra os controles, não a porta).
                ph2d_panel_vector::set_current_patternpath_can_pick(
                    sel.len() == 1 && pat.is_none(),
                );
                ph2d_panel_vector::set_current_patternpath(
                    pat.is_some(),
                    pat.map_or(0.0, |p| f64::from(p.start_offset)),
                    pat.map_or(1.0, |p| f64::from(p.end_offset)),
                    pat.map_or(1.0, |p| f64::from(p.spacing)),
                    pat.map_or(0.0, |p| f64::from(p.offset)),
                    pat.is_some_and(|p| p.flip),
                    f64::from(crate::pattern_live::current_rotation(
                        sim,
                        &self.vec_entities,
                        &sel,
                    )),
                );
                // Contour (pesquisa `20_*` #9): as MESMAS duas perguntas do Pattern — *"esta
                // seleção permite criar?"* e *"o que está armado, com que valores?"*. `can_add`
                // exige forma selecionada e nenhum contour nela: a seção mostra o botão OU os
                // controles, nunca os dois, e é isso que impede a swatch de existir sem alvo.
                let cont = crate::contour_live::current(sim, &self.vec_entities, &sel);
                ph2d_panel_vector::set_current_contour_can_add(!sel.is_empty() && cont.is_none());
                // O `d` do componente é MUNDO; o painel fala FRAÇÃO. A conversão usa a MESMA
                // `offset_scale` do arm e do drain — três leituras da mesma régua, uma função.
                // ⚠️ **Só com contour armado**, e é medida de custo, não de estilo: o
                // `vec_transform::build` percorre TODO caminho da cena e sobe a cadeia de pais
                // de cada um, alocando um mapa. Calculá-lo aqui incondicionalmente poria essa
                // varredura em todo frame com o painel aberto, para publicar um número que só
                // é lido quando há efeito — e sem contour o `d_frac` publicado é `0.0` de
                // qualquer maneira, sem passar pela escala.
                let cont_scale = cont.map_or(0.0, |_| {
                    let xf = crate::vec_transform::build(sim, &self.vec_entities);
                    crate::vec_expand::offset_scale(vec_scene, &self.vec_pen, &xf)
                });
                ph2d_panel_vector::set_current_contour(
                    cont.is_some(),
                    cont.map_or(4.0, |(_, c)| f64::from(c.steps)),
                    cont.map_or(0.0, |(_, c)| {
                        if cont_scale > 0.0 {
                            c.d / cont_scale
                        } else {
                            0.0
                        }
                    }),
                    cont.map_or(1.0, |(_, c)| f64::from(c.accel)),
                    cont.map_or(1, |(_, c)| c.join),
                    cont.map_or(0, |(_, c)| c.side),
                    cont.map_or([255, 255, 255, 255], |(_, c)| c.to),
                );
                // A BORDA: quando a forma espelhada muda (trocou a seleção, ou o Add acabou de
                // armar), os controles são reescritos no store a partir do componente. Sem isto o
                // `paint` — que lê o store primeiro, para não brigar com o arrasto — mostraria os
                // números da forma anterior sobre a forma nova.
                let cont_mirror = cont.map(|(id, _)| id);
                if cont_mirror != self.vec_contour_mirrored {
                    self.vec_contour_mirrored = cont_mirror;
                    if let Some((_, c)) = cont {
                        let frac = if cont_scale > 0.0 {
                            c.d / cont_scale
                        } else {
                            0.0
                        };
                        let steps = f64::from(c.steps);
                        let accel = f64::from(c.accel);
                        for (slider, chip, track, value) in [
                            (
                                ph2d_editor::ids::VECTOR_CONTOUR_STEPS,
                                ph2d_editor::ids::VECTOR_CONTOUR_STEPS_NUM,
                                ph2d_panel_vector::contour_steps_to_track(steps),
                                steps,
                            ),
                            (
                                ph2d_editor::ids::VECTOR_CONTOUR_OFFSET,
                                ph2d_editor::ids::VECTOR_CONTOUR_OFFSET_NUM,
                                ph2d_panel_vector::contour_d_to_track(frac),
                                frac * 100.0, // LITERAL-PX-OK: fração -> percentual do readout
                            ),
                            (
                                ph2d_editor::ids::VECTOR_CONTOUR_ACCEL,
                                ph2d_editor::ids::VECTOR_CONTOUR_ACCEL_NUM,
                                ph2d_panel_vector::contour_accel_to_track(accel),
                                accel,
                            ),
                        ] {
                            hero.store.set_slider_value(slider, track);
                            hero.store.set_number_value(chip, value);
                        }
                    }
                }
                // Filters (a pilha de FX raster, plano 24): as MESMAS duas perguntas do Contour —
                // *"esta seleção pode receber um filtro?"* (há forma) e *"o que está armado?"*. O
                // painel lê a pilha do PRIMEIRO caminho selecionado que tenha uma; a seção some sem
                // seleção e sem pilha viva. `radius`/`offset`/`opacity` são MUNDO/normalizados —
                // sem conversão de escala (o raio de mundo é o número que o slider mostra).
                let filt = sel
                    .iter()
                    .find_map(|id| crate::fx_live::spec_of(sim, &self.vec_entities, *id));
                ph2d_panel_vector::set_current_filter_can_add(!sel.is_empty());
                // A TABELA dos tipos vem do MOTOR (o painel não alcança o `ph2d-ecs`) — uma
                // segunda tabela discordaria do `kind` na primeira adição, e com sete tipos o
                // modo de falha é um knob morto (ou um que falta) que nenhum gate vê.
                ph2d_panel_vector::set_filter_kinds(
                    ph2d_ecs::FxOp::SPECS
                        .iter()
                        .map(|s| ph2d_panel_vector::FilterKindView {
                            name: s.name,
                            radius_label: s.radius_label,
                            offset_labels: s.offset_labels,
                            color_label: s.color_label,
                            color_b_label: s.color_b_label,
                            modes: s.modes,
                            takes_blend: s.takes_blend,
                            takes_ramp: s.takes_ramp,
                            noise_labels: s.noise_labels,
                            grow_label: s.grow_label,
                            adjust_labels: s.adjust_labels,
                        })
                        .collect(),
                );
                // …e os NOMES das leis de mistura, pela mesma porta e pelo mesmo motivo: quem
                // conhece o `BlendMode` é a shell, não o painel. As leis de COBERTURA (`Behind` /
                // `Clear`) ficam de fora — um degrau aplica a lei dele onde a cobertura já está
                // decidida, e oferecê-las seria a opção que despacha e mente.
                ph2d_panel_vector::set_filter_blend_names(
                    (0..ph2d_ecs::FxOp::BLEND_KINDS)
                        .map(|m| ph2d_painter_effects::BlendMode::from_u8(m).name())
                        .collect(),
                );
                ph2d_panel_vector::set_current_filters(
                    filt.map(|f| {
                        f.ops
                            .iter()
                            .map(|op| ph2d_panel_vector::FilterRowView {
                                kind: op.kind,
                                mode: op.mode,
                                enabled: op.enabled,
                                radius: f64::from(op.radius),
                                offx: f64::from(op.offset[0]),
                                offy: f64::from(op.offset[1]),
                                color: crate::fx_live::colour_bytes(op.color),
                                color_b: crate::fx_live::colour_bytes(op.color_b),
                                opacity: f64::from(op.opacity),
                                blend: op.blend_code(),
                                scale: f64::from(op.scale),
                                // ⚠️ **`detail_clamped`, não `detail`** — a mesma metade de
                                // HONRAR que o produtor da GPU usa. O painel mostra o número que
                                // o dispositivo de facto soma.
                                detail: op.detail_clamped(),
                                seed: op.seed,
                                grow: f64::from(op.grow),
                                // ⚠️ **VOLTAS -> GRAUS na fronteira.** O modelo fala voltas (a
                                // unidade do `HsbParams` do Painter, que é a MESMA lei); o painel
                                // fala graus, porque é a unidade em que um artista pensa uma cor.
                                // A volta é feita aqui e no `apply`, em linhas que se leem juntas
                                // — o idioma que a §12 da física já usa com radianos.
                                hue: f64::from(op.hue) * 360.0,
                                sat: f64::from(op.sat),
                                bright: f64::from(op.bright),
                                stop_pos: op.stop_pos,
                                stop_colors: op.stops.map(crate::fx_live::colour_bytes),
                                stop_count: op.stop_count,
                                // ⚠️ **A rampa é amostrada AQUI, pela função que é o ORÁCULO dos
                                // gates de paridade** (`gradient_map_lut` do
                                // `ph2d-painter-effects`) — o bar é o que o artista lê para prever
                                // o render, então ele TEM de sair da mesma lei que o device honra.
                                // Um lerp de conveniência no painel divergiria em gama justo nos
                                // meios-tons, e o único lugar onde isso apareceria é uma
                                // screenshot. Medido: device vs esta função, **1 nível de byte**.
                                ramp_preview: crate::fx_live::ramp_preview(op),
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
                );
                // ADR-0132: o Trim do caminho selecionado. A MESMA `sole_path` do dispatch --
                // o painel nao pode oferecer controles para um caminho que o clique nao alcanca.
                let fx_target = crate::fx_bridge::sole_path(self.vec_pen.selected_paths());
                ph2d_panel_vector::set_current_effects(
                    fx_target.is_some(),
                    ph2d_vec_scene::effect::PathEffect::KINDS,
                    fx_target
                        .map_or_else(Vec::new, |pid| crate::fx_bridge::stack_view(vec_scene, pid)),
                );
                // Qual chip de gesto acende. O painel pergunta ao MESMO container que o
                // dispatch vai escrever, senao a tela mostraria um gesto e o clique mudaria outro.
                ph2d_panel_vector::set_current_envelope_mode(env_container.map_or(0, |b| {
                    match crate::envelope_gesture::kind_of(sim, b) {
                        ph2d_ecs::EnvelopeKind::Perspective => 0,
                        ph2d_ecs::EnvelopeKind::Mesh => 1,
                        ph2d_ecs::EnvelopeKind::Pins => 2,
                    }
                }));
                // Os presets de gaiola: o painel se auto-popula desta lista, entao acrescentar um
                // preset e' uma linha em `EnvelopeWarp::ALL` e ZERO mudanca de painel.
                let labels: Vec<&'static str> = ph2d_ecs::EnvelopeWarp::ALL
                    .iter()
                    .map(|w| w.label())
                    .collect();
                let (active, bend) = env_container
                    .and_then(|b| crate::envelope_gesture::warp_of(sim, b))
                    .map_or((None, 0.0), |(w, bend)| {
                        (
                            w.and_then(|w| {
                                ph2d_ecs::EnvelopeWarp::ALL.iter().position(|c| *c == w)
                            }),
                            bend,
                        )
                    });
                ph2d_panel_vector::set_current_envelope_presets(&labels, active, bend);
            }
            // Live Shapes: o ALVO dos campos de forma do painel é a forma paramétrica
            // SELECIONADA — os campos DELA aparecem (mesmo na ferramenta Select) e a
            // editam. Sem alvo, valem os da forma ativa do catálogo (default do traço).
            #[cfg(feature = "panel-vector")]
            {
                let sel: Vec<ph2d_vec_scene::VecPathId> = self.vec_pen.selected_paths().to_vec();
                let target =
                    crate::vec_shape_params::panel_shape_target(sim, &self.vec_entities, &sel);
                ph2d_panel_vector::set_current_shape_focus(target.as_ref().map(|(_, _, k, _)| *k));
                // Semente ONE-SHOT: só quando o alvo MUDA (senão brigaria com o arrasto).
                // Além dos campos, a TOOL adota os params — assim painel, tool e objeto
                // concordam, e a próxima forma desenhada herda (modelo Figma).
                let target_id = target.as_ref().map(|(id, _, _, _)| *id);
                if target_id != self.vec_shape_last_target {
                    self.vec_shape_last_target = target_id;
                    if let Some((_, _, kind, world)) = target.as_ref() {
                        crate::vec_shape_params::seed_shape_fields(
                            &mut hero.store,
                            *kind,
                            world,
                            vec_px_to_world,
                        );
                        let ui =
                            crate::vec_shape_params::ui_values_of(*kind, world, vec_px_to_world);
                        vector_bridge::adopt_shape_values(tools, *kind, ui);
                    }
                }
            }
            // **Conectores, 1ª metade:** pendura o `VecConnector` na entidade (que nasceu no
            // `sync` acima) do conector EM GESTO e do recém-fechado.
            //
            // **Antes do `settle`, e isso não é arrumação:** o `settle` pula os conectores
            // (a geometria deles é MUNDO, reescrita a cada frame) — mas só pode pular o que
            // ENXERGA. Sem o componente já pendurado, a linha recém-empurrada seria assentada
            // como um path comum: origem no centro dela, geometria recuada, e a rota do frame
            // seguinte sairia deslocada exatamente por esse delta.
            crate::connector_live::upkeep(
                sim,
                vec_scene,
                &self.vec_entities,
                self.vec_connect.as_ref().map(|d| (d.path, &d.conn)),
                &mut self.vec_connect_pending,
            );
            // **Blend Objects, 1ª metade:** pendura o `VecBlend` na entidade (nascida no `sync`)
            // do blend recém-criado. ANTES do `settle`, pela mesma razão do conector: o `settle`
            // pula o blend, mas só o que ENXERGA — sem o componente já pendurado, o spine
            // recém-empurrado seria assentado como um path comum e o recook do frame seguinte
            // sairia deslocado (ADR-0128).
            crate::blend_live::upkeep(
                sim,
                vec_scene,
                &self.vec_entities,
                &mut self.vec_blend_pending,
            );
            // **Morph Objects, 1ª metade:** idem, e pela MESMA razão — sem o componente pendurado
            // antes do `settle`, o path recém-empurrado seria assentado como um path comum e o
            // recook do frame seguinte sairia deslocado.
            crate::morph_live::upkeep(
                sim,
                vec_scene,
                &self.vec_entities,
                &mut self.vec_morph_pending,
            );
            // **Envelope Objects (ADR-0129 Fatia 3):** SEM `upkeep` — o envelope não cria path
            // nenhum (o container não tem path), então não há entidade nova esperando o `sync`. Tudo
            // (assar + reparentar + pendurar) já aconteceu síncrono no `create`. O `settle` pula os
            // filhos (têm `ChildOf`) e o container (sem path, fora do mapa).
            // ADR-0112: a origem (o pivô) de um path nasce no centro do MUNDO. Assim
            // que a forma pára de crescer, ela vai para o centro dela. Quem está EM GESTO é
            // pulado, e a lista sai de UMA porta (`vec_gesture_paths`) — o porquê está lá.
            let drawing = crate::vec_transform::gesture_paths(
                &self.vec_pen,
                &self.vec_shape,
                &self.vec_pencil,
            );
            crate::vec_transform::settle_origins(sim, vec_scene, &self.vec_entities, &drawing);
            // ADR-0114/ADR-0111: idem para os objetos Flip — o pivô nasce no centro do
            // MUNDO; assim que a arte pára de crescer, ele vai para o centro dela (e a
            // geometria vira LOCAL). O objeto EM GESTO (desenho/borracha ativos) NÃO é
            // assentado — a mão escreve MUNDO a cada frame e somar geometria+Transform
            // deslocaria a arte do cursor.
            let flip_gesturing = (self.flip_draw.is_active() || self.flip_erasing)
                .then(|| flip.objects().first().map(|o| o.id))
                .flatten();
            crate::flip_transform::settle_origins(sim, flip, &self.flip_entities, flip_gesturing);
            // **A ordem de z é a projeção da árvore — e a árvore é lida AQUI, depois do
            // `sync`.** Não é arrumação (BUGS #15): a lista do painel foi publicada no
            // prólogo do frame, quando a forma recém-criada ainda não tinha entidade.
            // Projetar por ela punha a forma nova no FUNDO por um frame, e a captura do
            // undo — tirada no fim deste frame — deixava de ser ponto fixo dos sistemas.
            // Toda raiz ganha um `RootOrder` explícito ANTES de a árvore ser lida — e antes
            // da captura do fim do frame. Sem isto, a raiz sem ordem colate em `u32::MAX` e
            // a árvore a desempata por `Entity::to_bits()` (id de ALOCAÇÃO): o respawn do
            // undo troca os bits, a pilha de z se reordena sozinha a cada Ctrl+Z, e o passo
            // espúrio volta vestido de outra coisa. Não ter empate > escolher desempate.
            ph2d_ecs::assign_missing_root_order(sim.world_mut());
            // O Blend pediu uma sequência de z; agora as entidades existem (o `sync` rodou) e ela
            // pode ser escrita na ÁRVORE — que é quem manda no z (ADR-0110). Escrever na ordem do
            // vetor da cena seria a porta errada: a projeção abaixo a reescreve todo frame.
            for order in std::mem::take(&mut self.vec_restack) {
                crate::vec_entities::restack(sim, &self.vec_entities, &order);
            }
            if let Some(live) = hero_live.as_mut() {
                crate::build_hierarchy_snapshot(
                    sim.world(),
                    &mut live.z_walk_state,
                    &mut live.z_walk_scratch,
                    &mut live.z_snapshot,
                );
                let order = crate::vec_entities::z_order(&live.z_snapshot);
                vec_scene.reorder_to(&order);
            }
            let vec_view = crate::vec_entities::view_state(sim, &self.vec_entities);
            // ADR-0111 — cada path tem `Transform`. A geometria dele é LOCAL; este é
            // o afim que a leva ao mundo (a cadeia de pais inclusa).
            let mut vec_xf = crate::vec_transform::build(sim, &self.vec_entities);
            // **Conectores, 2ª metade:** a geometria é uma função pura da RELAÇÃO — re-cozida
            // aqui, todo frame, sobre os afins DESTE frame. É o que faz a linha SEGUIR a
            // forma que o gizmo acabou de mover.
            crate::connector_live::recook(
                sim,
                vec_scene,
                &self.vec_entities,
                &vec_xf,
                &mut self.vec_connect_sides,
            );
            // **Morph Objects, 2ª metade:** a forma é função pura das duas fontes e do `t` —
            // re-cozida aqui, todo frame, sobre os afins DESTE frame. É o que a faz SEGUIR a
            // forma que o gizmo acabou de mover, e o que faz o `t` da timeline virar movimento.
            // (O `t` já foi escrito: o apply da timeline roda antes desta metade do frame.)
            crate::morph_live::recook(
                sim,
                vec_scene,
                &self.vec_entities,
                &vec_xf,
                &mut self.vec_morph_plans,
            );
            // **Envelope Objects (ADR-0129):** a forma de cada filho é a fonte autorada deformada
            // pela gaiola comum — re-cozida aqui, todo frame. Sem xforms nem mapa: a fonte é LOCAL do
            // container e é o `Transform` do container (via `vec_transform::build`) que leva os filhos
            // ao mundo; o recook varre os containers por QUERY (eles não têm path).
            crate::envelope_live::recook(sim, vec_scene);
            // **Select: arrastar o objeto blend move as fontes** — o gizmo mira as FONTES (não o
            // spine), então ele as move NATIVAMENTE como grupo (`vec_selection::sync_selection`
            // redireciona a seleção do gizmo). O spine as segue no `recook`. Nada a fazer aqui: um
            // gizmo sobre o spine dobraria (Transform + bbox que já andou); sobre as fontes não, a
            // geometria delas é fixa e só o `Transform` se move.
            // **Modo Node: arrastar uma ÂNCORA do spine move a forma-fonte dela** (ADR-0128 C2b) —
            // o inverso da pinagem. Roda ANTES do recook: move a fonte para a âncora arrastada e o
            // recook então re-encosta a âncora no centro (agora coincidentes, sem salto). Como a
            // fonte se moveu, o `vec_xf` é refeito para os passos deste frame já saírem do lugar
            // novo. Só no Node — no Select a fonte se move pelo gizmo (acima) e a âncora a segue.
            if vector_active && self.vec_draw_config.mode == ph2d_tool_vector::DrawMode::Node {
                crate::blend_live::drag_spine_anchors_move_sources(
                    sim,
                    vec_scene,
                    &self.vec_entities,
                    &vec_xf,
                    &mut self.vec_blend_spines,
                );
                vec_xf = crate::vec_transform::build(sim, &self.vec_entities);
            }
            // **Blend Objects, 2ª metade:** os passos são função pura das fontes — re-cozidos
            // aqui, todo frame, sobre os afins DESTE frame. É o que faz a transição SEGUIR a
            // forma que o gizmo acabou de mover (ADR-0128). O buffer é zerado e repopulado.
            crate::blend_live::recook(
                sim,
                vec_scene,
                &self.vec_entities,
                &vec_xf,
                &mut self.vec_blend_spines,
                &mut self.vec_blend_overlay,
            );
            // **Modo Node: o spine sobe para o topo** (ADR-0128) — acima de TODAS as formas e
            // passos, para ser visto e editado. Retira o traço da cena (some do `dispatch`, logo
            // abaixo) e o acrescenta ao fim do overlay do blend (desenhado por último). Em Select
            // o spine fica no seu z (traço sutil), como o Illustrator — o `recook` restaura o
            // traço-base todo frame, então voltar de Node não o deixa invisível.
            if vector_active && self.vec_draw_config.mode == ph2d_tool_vector::DrawMode::Node {
                crate::blend_live::elevate_spines(
                    sim,
                    vec_scene,
                    &self.vec_entities,
                    &mut self.vec_blend_overlay,
                );
            }
            // **Rótulos:** o texto que pertence a uma forma (ou a um conector) e a segue. A pose
            // é uma função pura do hospedeiro — como a rota do conector é da relação dele.
            //
            // **DEPOIS do `recook`, e isso não é arrumação:** a âncora do rótulo de um conector é
            // o meio da rota, e a rota deste frame acabou de ser escrita ali em cima. Antes do
            // `recook` o rótulo penderia da polilinha do frame ANTERIOR — e arrastaria a forma
            // sempre um quadro atrás da linha. (E depois do `build`, pela mesma razão: os afins
            // das formas-alvo já são os deste frame.)
            //
            // O `upkeep_pending` vem primeiro: um rótulo nasce VAZIO, e é a 1ª letra que cria o
            // objeto — o vínculo tem de estar pendurado antes do passe procurar por ele.
            let text_id = self.vec_text_edit.as_ref().and_then(|e| e.id);
            crate::label_live::upkeep_pending(
                sim,
                &self.vec_entities,
                &mut self.vec_label_pending,
                text_id,
                self.vec_text_edit.is_some(),
            );
            crate::label_live::upkeep(
                sim,
                vec_scene,
                &self.vec_entities,
                &mut vec_xf,
                text_id,
                &mut self.vec_label_poses,
            );
            self.vec_pen.set_view(vec_view.clone());
            self.vec_pen.set_xforms(vec_xf.clone());
            // Seleção casada nos dois sentidos: clique na Hierarquia chega no canvas,
            // clique no canvas acende a linha (e a do grupo, se cheio). A seleção do
            // gizmo é COMPARTILHADA com os sprites — só o subconjunto vetorial é nosso.
            crate::vec_selection::sync_selection(
                &mut hero.gizmo,
                sim,
                vec_scene,
                &self.vec_entities,
                &mut self.vec_pen,
                &mut self.vec_sel,
                vector_active,
            );

            let cam_affine = camera.world_to_screen_affine(window_size);
            // A geometria DERIVADA deste frame — hoje, os offsets vivos. Cozida aqui (depois
            // do `sync`, senão uma forma recém-criada ainda não tem entidade e o componente
            // dela não seria encontrado) e desenhada pelo `dispatch` no z de cada forma.
            self.offset_live
                .recook(vec_scene, sim, &self.vec_entities, &vec_xf);
            // O Pattern Along Path vivo (plano 23): as cópias de um motivo ao longo de um guia,
            // cozidas aqui e desenhadas no z do motivo — a fonte nunca é tocada.
            self.pattern_live.recook(vec_scene, sim, &self.vec_entities);
            // O Contour vivo (pesquisa 20 #9): os anéis concêntricos + a rampa de cor, cozidos
            // aqui e desenhados no z da fonte — que entra na lista junto com eles.
            self.contour_live
                .recook(vec_scene, sim, &self.vec_entities, &vec_xf);
            // **O LÁPIS pendura o perfil que o GESTO pede** (W1d) — ao vivo, a cada frame em que
            // o traço está aberto. É aqui e não no `input_dispatch` porque o armamento precisa do
            // mundo ECS, e porque é o único lugar que corre entre o `sync` (que dá entidade ao
            // path recém-nascido) e o cozimento logo abaixo: o artista vê a espessura enquanto
            // desenha, que é a promessa do lápis desde o W1a (*"o ajuste é AO VIVO"*).
            if let Some(id) = self.vec_pencil.active_path() {
                let stops = self
                    .vec_pencil
                    .width_stops(self.vec_draw_config.pencil_width_source);
                crate::profile_live::arm(sim, &self.vec_entities, &[id], &stops);
            }
            // A largura VIVA (ADR-0148): a fita de largura variável, cozida aqui e desenhada no
            // z da fonte — que continua sendo a curva autorada que o modo Node edita.
            self.profile_live
                .recook(vec_scene, sim, &self.vec_entities, &vec_xf);
            // O `dispatch` recebe UMA `LiveGeometry`. Uma forma é offset OU pattern OU contour
            // (nunca dois — cada um é um componente próprio e o painel oferece um de cada vez),
            // então fundir é seguro; começa do offset (em cena típica dos outros, vazio ⇒ clone
            // trivial) e junta os demais por cima.
            let mut vec_live = self.offset_live.live().clone();
            vec_live.extend(
                self.pattern_live
                    .live()
                    .iter()
                    .map(|(id, v)| (*id, v.clone())),
            );
            vec_live.extend(
                self.contour_live
                    .live()
                    .iter()
                    .map(|(id, v)| (*id, v.clone())),
            );
            vec_live.extend(
                self.profile_live
                    .live()
                    .iter()
                    .map(|(id, v)| (*id, v.clone())),
            );
            // A SILHUETA resolvida das formas TRAÇADAS: `preenchimento ∪ contorno-do-traço`,
            // pela booleana, memoizada na geometria de MUNDO. Sem ela o campo de distância de uma
            // forma com traço cai no caminho do raster, cuja semente discreta desenha o pente que
            // o Enio fotografou no bevel. Roda DEPOIS de `vec_live` (a união é do que se DESENHA).
            self.fx_silhouette
                .recook(vec_scene, sim, &self.vec_entities, &vec_xf, &vec_live);
            // O FX raster por-forma (plano 24 — Blur/Glow/Drop Shadow). O produtor
            // (`fx_live`) rasteriza a forma isolada num scratch de GPU, lê de volta e borra na CPU,
            // injetando as imagens no z da forma. Roda DEPOIS de `vec_live` porque honra a geometria
            // derivada. Sem `VecFilter` na cena o mapa fica vazio = BYTE-IDÊNTICO ao mundo pré-FX.
            self.fx_live.recook(
                vec_scene,
                sim,
                &self.vec_entities,
                &vec_xf,
                &vec_live,
                self.fx_silhouette.live(),
                cam_affine,
                surface.gpu(),
                surface.format(),
                vello_pass,
            );
            let vec_fx = self.fx_live.images();
            ph2d_vec_render::dispatch(
                vec_scene,
                &vec_view,
                &vec_xf,
                &vec_live,
                vec_fx,
                cam_affine,
                vector_scene,
            );
            // O **overlay** do Blend Object (ADR-0128): os passos virtuais + as fontes de cima
            // reempilhadas, na ordem de z (a última fonte por cima do último passo). Desenha depois
            // do `dispatch` (que já pôs as fontes no z da cena, embaixo); o overlay reestabelece a
            // pilha do blend por cima. O interleaving fino contra o resto da cena é da Fase C.
            ph2d_vec_render::draw_blend_overlay(&self.vec_blend_overlay, cam_affine, vector_scene);
            // **Pick Shapes** (ADR-0128 C2b): realça as formas escolhidas e costura a ORDEM de
            // clique numa polilinha (a prévia do spine). Fora do modo Pick, a lista não vale —
            // limpa, para não vazar escolhas velhas para o próximo blend.
            if vector_active && self.vec_draw_config.mode == ph2d_tool_vector::DrawMode::PickBlend {
                let preview =
                    crate::blend_live::pick_preview(vec_scene, &vec_xf, &self.vec_blend_picks);
                ph2d_vec_render::draw_blend_overlay(&preview, cam_affine, vector_scene);
            } else if !self.vec_blend_picks.is_empty() {
                self.vec_blend_picks.clear();
            }
            // **O Picker de caminho-guia** (Enio 2026-07-23): armado, o caminho sob o cursor é o que
            // o clique vai prender — a silhueta dele acende, o idioma do conta-gotas. `path_at` é o
            // MESMO resolvedor do clique, então o realce nunca mente sobre o que será escolhido. Fora
            // do modo Select (ou sem a tool) o pick não faz sentido: limpa, para não ficar armado e
            // invisível — a saída sem compromisso, como o clique no vazio.
            if let Some(pick) = self.vec_path_pick {
                if vector_active && self.vec_draw_config.mode == ph2d_tool_vector::DrawMode::Select
                {
                    let w = camera.screen_to_world(self.last_pointer, window_size);
                    let a = camera.screen_to_world((0.0, 0.0), window_size);
                    let b = camera.screen_to_world((1.0, 0.0), window_size);
                    // LITERAL-PX-OK: raio de acerto em px, o MESMO do picking de canvas (`path_at`).
                    let hit_r =
                        10.0 * f64::from(((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt());
                    if let Some(gid) =
                        self.vec_pen
                            .path_at(vec_scene, [f64::from(w[0]), f64::from(w[1])], hit_r)
                        && gid != pick.source()
                        && let Some(outline) =
                            crate::vec_pick::hover_outline(vec_scene, &vec_xf, gid)
                    {
                        ph2d_vec_render::draw_blend_overlay(&[outline], cam_affine, vector_scene);
                    }
                } else {
                    self.vec_path_pick = None;
                }
            }
            // Âncoras/handles/gradiente/marquee só interessam a quem edita nós; no
            // modo Select quem fala é o gizmo (ADR-0112). As guias de snap são caso à
            // parte (valem em TODOS os modos) — `vec_overlay` separa as duas políticas
            // num ponto testável (P1).
            let overlay =
                crate::vec_overlay::vec_overlay_plan(vector_active, self.vec_draw_config.mode);
            if overlay.edit {
                // ⚠️ A gaiola do Envelope SUBSTITUI a edição de nós. Quando a seleção é um envelope,
                // a forma sob a gaiola é DERIVADA (os nós dela são a SAÍDA do warp, não estado
                // editável), então NÃO se desenham as alças de nó dela — elas confundem (não se
                // pode arrastá-las) e expõem o handle longo que o refit deixa numa quina CÔNCAVA
                // (o "traço à deriva" que o Enio reportou numa estrela sob envelope, 2026-07-24).
                // Você edita a GAIOLA (desenhada abaixo). É o idioma do Illustrator/Affinity: com um
                // envelope ativo, os nós do objeto somem.
                let envelope_selected = hero
                    .gizmo
                    .selection
                    .is_some_and(|bits| crate::envelope_gesture::is_envelope(sim, bits));
                if !envelope_selected {
                    ph2d_vec_render::draw_overlays(
                        vec_scene,
                        &vec_view,
                        self.vec_pen.selected(),
                        self.vec_pen.selected_paths(),
                        self.vec_pen.selected_verts(),
                        &vec_xf,
                        cam_affine,
                        vector_scene,
                    );
                }
                // NOTA: as alças de raio de quina (Live Corners) não são mais desenhadas — o
                // arredondar/chanfrar quina virou o par de ferramentas Fillet / Chamfer (o gesto de
                // clicar-e-arrastar). A exclusão de forma VIVA (`crate::corner_handles::has_derived_verts`)
                // segue viva: é o que o press dessas ferramentas consulta no `input_dispatch`.
                // A gaiola do **Envelope** (ADR-0129): os 4 cantos que o Node arrasta para
                // deformar as formas que a gaiola contém. Alça PRÓPRIA (não o gizmo de sprite,
                // §3.3), desenhada só quando a seleção do gizmo é de fato um envelope — o flag de
                // modo (`envelope_cage`) diz o MODO, o `view` devolve `None` quando a entidade
                // selecionada não carrega um `VecEnvelope`. O alvo é o CONTAINER (Fatia 3): a
                // regra seleciona-só-o-container põe os bits dele em `hero.gizmo.selection`.
                if overlay.envelope_cage
                    && let Some(cage) = crate::envelope_gesture::view(
                        sim,
                        hero.gizmo.selection,
                        self.vec_envelope_drag,
                    )
                {
                    ph2d_vec_render::draw_envelope_cage(
                        &cage,
                        cam_affine,
                        hero.theme,
                        vector_scene,
                    );
                }
                crate::vec_overlay_diag::dump(vec_scene, sim, hero.gizmo.selection);
                // Os PINOS (Fatia E) — o mesmo gate de modo, outro overlay: no gesto Pins o `view`
                // devolve `None` (nao ha gaiola) e sao os pinos que se desenham. Nunca os dois.
                if overlay.envelope_cage
                    && let Some(bits) = hero.gizmo.selection
                {
                    let pins = crate::envelope_gesture::pins_world(sim, bits);
                    ph2d_vec_render::draw_envelope_pins(
                        &pins,
                        self.vec_envelope_drag
                            .filter(|(d, _)| *d == bits)
                            .map(|(_, i)| i),
                        cam_affine,
                        hero.theme,
                        vector_scene,
                    );
                }
                // Gradient handles (multi-point dots, or linear/radial endpoints)
                // when the selected path has a gradient fill. A geometria do gradiente
                // é LOCAL como a do path, então sobe pelo afim dele.
                if let Some(sel) = self.vec_pen.selected() {
                    ph2d_vec_render::draw_gradient_handles(
                        vec_scene,
                        Some(sel),
                        self.vec_grad_selected,
                        ph2d_vec_render::path_to_screen(&vec_xf, sel, cam_affine),
                        vector_scene,
                    );
                }
                // Box-select marquee (Shift+drag), in screen-space.
                if let Some((start, cur)) = self.vec_marquee {
                    ph2d_vec_render::draw_marquee(
                        [f64::from(start.0), f64::from(start.1)],
                        [f64::from(cur.0), f64::from(cur.1)],
                        vector_scene,
                    );
                }
            }
            // Smart guides do snap: FORA do guard de modo — explicam o encaixe vivo em
            // qualquer modo, inclusive o gizmo-move do Select (P1, ADR-0112).
            if overlay.snap_guides {
                ph2d_vec_render::draw_snap_guides(&self.vec_snap_guides, cam_affine, vector_scene);
            }
            // **O realce do Shape Builder** — as faces sob o cursor e as já pintadas. Fora do
            // `overlay.edit` porque o Build não é um modo de edição de nó: o que ele
            // manipula é a REGIÃO, não a âncora.
            if let Some(b) = self.vec_build.as_mut() {
                let marked: Vec<ph2d_vec_scene::VecPath> = b
                    .marked
                    .clone()
                    .into_iter()
                    .filter_map(|f| b.arr.face_path(f).cloned())
                    .collect();
                let hover = b.hover.and_then(|f| b.arr.face_path(f).cloned());
                // As faces E as silhuetas já estão em MUNDO (a sessão as assou), então só a
                // câmera. As silhuetas são redesenhadas por cima do véu: uma forma coberta
                // por outra não aparece na tela, e sem elas o realce paira sobre nada.
                ph2d_vec_render::draw_build_faces(
                    b.arr.sources(),
                    hover.as_ref(),
                    &marked,
                    b.subtract,
                    cam_affine,
                    hero.theme,
                    vector_scene,
                );
            }
            // **As alças de ponta do conector** — FORA do `overlay.edit`, e isso é a coisa toda:
            // elas vivem no modo **Select**, que é exatamente onde `overlay.edit` é FALSO (lá
            // quem fala é o gizmo, ADR-0112). Pô-las dentro do guard as tornaria invisíveis no
            // único modo em que existem.
            //
            // O conector não publica gizmo (`vec_gizmo_view::view` o pula), então não há
            // disputa: a caixa de transformação não cobre estas bolinhas.
            if vector_active && self.vec_draw_config.mode == ph2d_tool_vector::DrawMode::Select {
                let handles = crate::connector_handles::view(
                    sim,
                    vec_scene,
                    &self.vec_entities,
                    self.vec_pen.selected_paths(),
                );
                ph2d_vec_render::draw_connector_handles(&handles, cam_affine, vector_scene);
                // Os pontos de passagem — QUADRADOS, por cima das bolinhas: quando um waypoint
                // é arrastado até uma ponta, é ele que está sob o dedo.
                let ways = crate::connector_handles::waypoint_view(
                    sim,
                    &self.vec_entities,
                    self.vec_pen.selected_paths(),
                );
                ph2d_vec_render::draw_connector_waypoints(&ways, cam_affine, vector_scene);
            }
            // **A alça do TEXTO EM CAMINHO** (W5) — FORA do `overlay.edit`, pela MESMA razão das
            // alças do conector logo acima: ela é do modo **Select**, onde `overlay.edit` é FALSO
            // (dentro daquele guard ela nunca desenharia — foi o bug do 1º smoke). O
            // `handle::world` devolve `None` sem um texto vinculado na seleção, e o
            // `overlay.textpath_handle` já confina ao Select. Geometria em MUNDO (o guia traz a
            // pose) ⇒ sobe pelo afim da CÂMERA. Desenhada DEPOIS do gizmo para ficar por cima
            // dele — ela é a ficha que o artista agarra, não um ponto atrás da caixa.
            if overlay.textpath_handle
                && let Some(at) = crate::vec_text_ride::handle::world(
                    sim,
                    vec_scene,
                    &self.vec_entities,
                    self.vec_pen.selected_paths(),
                )
            {
                ph2d_vec_render::draw_text_handle(
                    at,
                    self.vec_textpath_handle_drag,
                    cam_affine,
                    hero.theme,
                    vector_scene,
                );
            }
            // **As DUAS alças do PATTERN ON PATH** (W4) — Start e End do trecho, no MESMO lugar da
            // alça do texto (Select, depois do gizmo). Reusa a ficha (`draw_text_handle`): as duas
            // fichas são iguais, a POSIÇÃO diz qual é Start e qual é End. `handle::world` devolve
            // `None` sem um pattern vinculado no primário.
            if overlay.patternpath_handles
                && let Some((start_pt, end_pt)) = crate::pattern_live::handle::world(
                    sim,
                    vec_scene,
                    &self.vec_entities,
                    self.vec_pen.selected_paths(),
                )
            {
                use crate::pattern_live::PatternHandle;
                let dragging = self.vec_patternpath_handle;
                ph2d_vec_render::draw_text_handle(
                    start_pt,
                    dragging == Some(PatternHandle::Start),
                    cam_affine,
                    hero.theme,
                    vector_scene,
                );
                ph2d_vec_render::draw_text_handle(
                    end_pt,
                    dragging == Some(PatternHandle::End),
                    cam_affine,
                    hero.theme,
                    vector_scene,
                );
            }
            // **As alças de LARGURA** (plano 25 §5) — uma por parada do perfil da forma primária.
            // A ficha fica SOBRE a curva e uma haste sai dela até a borda da fita, que é o que a
            // parada mede: a ficha na borda atravessava a linha vizinha em multiplicador alto, e
            // era isso que fazia uma alça nascer na linha de ao lado (report do Enio 2026-07-30 —
            // ver o doc de `width_handles::handles`). Reusa a MESMA ficha do texto e do pattern.
            // `handles` devolve vazio sem traço (nada a editar).
            if overlay.width_handles
                && let Some(pid) = self.vec_pen.selected()
            {
                let grabbed = self.vec_width_grab.map(|g| g.stop);
                for (k, h) in crate::width_handles::handles(sim, vec_scene, &self.vec_entities, pid)
                    .into_iter()
                    .enumerate()
                {
                    ph2d_vec_render::draw_width_handle(
                        h.at,
                        h.tip,
                        grabbed == Some(k),
                        cam_affine,
                        hero.theme,
                        vector_scene,
                    );
                }
            }
            // Cursor de texto (modo Text): na ponta da última linha em edição. Lê só o
            // campo `vec_text_edit` (fn livre), pra não colidir com o borrow de gfx.
            if let Some((a, b)) = crate::vec_text::caret_of(self.vec_text_edit.as_ref()) {
                ph2d_vec_render::draw_text_caret(a, b, cam_affine, vector_scene);
            }
            // Drain the Painter Falloff right-click handle menu choice (chrome
            // parked the HandleType wire u8 in `pending_falloff_point_handle`) →
            // apply it to the selected control point.
            if let Some(handle) = hero.pending_falloff_point_handle.take()
                && let Some(id) = ph2d_panel_painter_layers::selected_falloff_point()
                && let Some(painter) = tools.active_mut().and_then(|t| {
                    t.as_any_mut()
                        .downcast_mut::<ph2d_tool_painter::PainterTool>()
                })
            {
                painter.set_brush_falloff_point_handle(id, handle);
            }
            // Drain the on-canvas Curve / Free Hand right-click handle-kind choice (chrome parked the wire
            // u8 in `pending_curve_point_handle`) → apply it to the selected control point.
            if let Some(kind) = hero.pending_curve_point_handle.take()
                && let Some(painter) = tools.active_mut().and_then(|t| {
                    t.as_any_mut()
                        .downcast_mut::<ph2d_tool_painter::PainterTool>()
                })
            {
                // Either curve owner: the stroke Shape curve, else the selection Convert-to-Curve editor.
                if !painter.set_curve_handle_kind(kind) {
                    painter.set_selection_curve_handle_kind(kind);
                }
            }
            // Onda 2C: clear the gizmo hit_map BEFORE paint_hero_screen
            // runs. `paint_hero_screen` now paints BOTH the primary gizmo
            // AND the multi-selection extras + global gizmo (the latter
            // via `paint_sprite_gizmo_keyed`, which populates `hit_map` —
            // those entries drive the dispatcher's group-transform routing
            // in `on_mouse_input`). Painting them inside `paint_hero_screen`
            // (before the floating panels) keeps gizmos BELOW the panels
            // both visually and in hit-test (z-order fix 2026-05-31).
            // ADR-0076: the vertex-edit / authoring vector tools (Direct / Pen /
            // Pencil / Shape) must NOT show the object-transform gizmo. Its painted
            // box + handles overlay the shape, and those tools' `vector_*_world`
            // reject any click over a `hit_index` widget — so the gizmo's hit-rects
            // would block EVERY vertex/handle grab + canvas click (Enio: "Direct
            // não move pontos/handles"). The gizmo belongs to object selection
            // (Select) + the arrow/Move tools. Suppress the painted view here; the
            // selection stays armed (hierarchy highlight) — just no box/handles.
            // The Deform Transform temperament shows its OWN whole-region gizmo (drawn in the painter
            // overlays). The object-transform gizmo would sit ON TOP (paint_hero_screen draws after those
            // overlays) and fight it, so suppress it while Deform Transform is active — the deform box IS the
            // transform gizmo there (Enio 2026-07-04).
            let painter_deform_transform = tools
                .active_mut()
                .and_then(|t| {
                    t.as_any_mut()
                        .downcast_mut::<ph2d_tool_painter::PainterTool>()
                })
                .is_some_and(|p| p.deform_gizmo().is_some());
            // A correção de pares do Flip é o MESMO caso das tools de vetor acima: o overlay de
            // Pairs quer o clique do canvas para re-parear, e a caixa+alças do gizmo do objeto
            // registram hits no `hit_index` que fazem `on_canvas` virar falso — roubando TODO
            // clique de re-par. Enquanto Pairs está aberto, o gizmo do objeto some (a seleção
            // fica armada; só a caixa/alças somem).
            let flip_pairs_active = self.flip_active && self.flip_strip.tween_correct.is_some();
            let suppress_gizmo = painter_deform_transform
                || flip_pairs_active
                || tools
                    .active()
                    .map(|t| {
                        let id = t.id();
                        id == ph2d_editor::ToolId::new("vector_direct")
                            || id == ph2d_editor::ToolId::new("vector_pen")
                            || id == ph2d_editor::ToolId::new("vector_pencil")
                            || id == ph2d_editor::ToolId::new("vector_shape")
                            // Motion Nodes: a tool Motion é dona do canvas (o único gizmo é o
                            // do field, slot próprio `field_view`). Um sprite selecionado por
                            // acaso ao entrar mostraria seu gizmo de sprite projetado na
                            // janela CHEIA (deslocado da cena que renderiza na banda do split)
                            // — some junto com o resto do chrome de sprite.
                            || id == ph2d_editor::ToolId::new("motion")
                    })
                    .unwrap_or(false);
            if suppress_gizmo {
                hero.gizmo.view = None;
                hero.gizmo.extra_views.clear();
                hero.gizmo.global_view = None;
            }
            hero.gizmo.gizmo_hit_map.clear();
            // Its sibling for the anchor dots — same reason (no stale entry
            // from a joint that left the scene), same frame.
            hero.gizmo.point_hit_map.clear();
            // Frame profiler: panel/chrome Vello encode (includes the painter panel's Paper preview).
            let hero_t0 = frame_prof_on().then(Instant::now);
            paint_hero_screen(hero, viewport, vector_scene, paint_ctx.text);
            if let Some(t0) = hero_t0 {
                FRAME_PROF_HERO_US.with(|c| c.set(t0.elapsed().as_micros() as u64));
            }
            // Audio Editor floating waveform overlay (docs/Audio/, W1) — painted
            // after the hero chrome, in the Hierarchy↔Inspector gap. Reads the
            // loaded clip from the audio system; no-op when the panel is closed
            // or no clip is loaded.
            #[cfg(feature = "panel-audio-editor")]
            if let Some(audio) = self.audio.as_mut() {
                audio_overlay::draw_audio_overlay(
                    hero,
                    audio,
                    ph2d_editor::zones::Rect::new(viewport.x, viewport.y, viewport.w, viewport.h),
                    vector_scene,
                    paint_ctx.text,
                );
            }
            // Fase 0f: overlay the active rubber-band rect on top of
            // everything (panels, gizmo, hero chrome). Pure shell
            // concern — coords stay in screen space so the rect
            // doesn't shift if the camera pans mid-drag. Semi-
            // transparent fill + 4 thin border rects (no stroke API
            // on VectorScene yet; the 4-fills idiom matches the rest
            // of the shell's overlay painters).
            if let Some(rb) = self.rubber_band {
                let (ax, ay) = rb.anchor_screen;
                let (cx, cy) = rb.current_screen;
                let x0 = ax.min(cx) as f64;
                let y0 = ay.min(cy) as f64;
                let x1 = ax.max(cx) as f64;
                let y1 = ay.max(cy) as f64;
                use ph2d_vector::{Color, Rect as VRect};
                // Selection accent — design tokens use OKLCH; the
                // sRGB approximation here is the canonical Selection
                // color from `ColorToken::Selection` (~#3a8ee6 @ 25%
                // fill, 100% border) baked at boot. Keeping it inline
                // avoids threading the theme into render_loop just
                // for one overlay; a follow-up can swap to a token
                // lookup if the rubber-band needs theme parity.
                let fill = Color::new([0.23, 0.56, 0.90, 0.18]);
                let border = Color::new([0.23, 0.56, 0.90, 1.0]);
                vector_scene.fill_rect(VRect::new(x0, y0, x1, y1), fill);
                vector_scene.fill_rect(VRect::new(x0, y0, x1, y0 + 1.0), border);
                vector_scene.fill_rect(VRect::new(x0, y1 - 1.0, x1, y1), border);
                vector_scene.fill_rect(VRect::new(x0, y0, x0 + 1.0, y1), border);
                vector_scene.fill_rect(VRect::new(x1 - 1.0, y0, x1, y1), border);
            }
            // Hierarchy intent dispatch phase — camera reset +
            // view-focus + 9 hierarchy intents (visibility_toggle /
            // reparent / duplicate / add_child / reset_transform /
            // delete / row_click / rename_seed / rename_commit).
            // Extracted to sibling `hierarchy.rs` as a free fn (Wave
            // 3.2 stage A).
            if hierarchy::dispatch(
                view_focus_kind,
                visibility_toggle_row,
                lock_toggle_row,
                group_toggle_row,
                reparent_intent,
                duplicate_row,
                add_child_row,
                reset_transform_row,
                delete_row,
                hierarchy_row_click,
                hierarchy_select_intent,
                rename_seed_row,
                rename_commit,
                hero,
                hero_live,
                sim,
                present,
                camera,
                toasts,
                window_size,
                &mut duplicate_made,
            ) {
                self.title_dirty = true;
            }
            // A duplicated sprite copies the source's `Sprite` component verbatim, so it SHARES the
            // source pixels — and if the source is being painted, the unbaked paint+mask never reaches
            // either entity (the working state is dropped on the next rebind, losing the paint from
            // both). When the source has live paint: bake it so the original persists, then give the
            // copy its OWN texture (a deep copy of the now-painted result) so it is a fully independent
            // object (Enio 2026-06-24). A non-painted duplicate keeps the shared source — Atlas/Individual
            // both fork on the next edit, so they stay independent in practice and keep atlas batching.
            if let Some((src_bits, new_bits)) = duplicate_made
                && self.last_painter_pushed_entity == Some(src_bits)
                && let Some(painter) = tools.active_mut().and_then(|t| {
                    t.as_any_mut()
                        .downcast_mut::<ph2d_tool_painter::PainterTool>()
                })
                && painter.has_unbaked_edits()
            {
                crate::hero_intents::auto_commit_painter(
                    src_bits,
                    sim,
                    renderer,
                    asset_db,
                    atlas_asset_map,
                    painter,
                );
                self.last_painter_pushed_entity = None; // bridge re-pushes the freshly-baked source
                let src = ph2d_ecs::Entity::from_bits(src_bits);
                let copy = ph2d_ecs::Entity::from_bits(new_bits);
                if let Some(read) = crate::hero_intents::texture_edit::read_sprite_source(
                    src,
                    sim,
                    renderer,
                    asset_db,
                    atlas_asset_map,
                ) {
                    let _ = crate::hero_intents::texture_edit::commit_edited_texture(
                        copy,
                        sim,
                        renderer,
                        &read.image,
                        read.old_size_world,
                    );
                }
            }
            // Inspector commits phase — Transform / Visibility / Name
            // / Sprite source-strategy + Reimport. Extracted to sibling
            // `inspector_commits.rs` as a free fn (Wave 3.2 stage A).
            //
            // W-J2: a physics joint's Position IS its A anchor, so the committed
            // pivot is re-seated through the bridge's anchor door just below.
            // Captured here because `transform_edit` is consumed by the call.
            let joint_pivot_commit =
                transform_edit.map(|info| (info.entity_bits, info.translation));
            if inspector_commits::dispatch(
                reimport_entity,
                transform_edit,
                visibility_edit,
                name_edit,
                sprite_source_change,
                &sprite_edits,
                &ordering_edits,
                &sampling_edits,
                &blend_edits,
                &physics_edits,
                &visibility_section_edits,
                hero,
                sim,
                renderer,
                asset_db,
                atlas_asset_map,
                toasts,
                editor_queue,
                component_registry,
                *transform_type_id,
                *visibility_type_id,
                *name_type_id,
                *sprite_type_id,
            ) {
                self.title_dirty = true;
            }
            // W-J2: re-seat a joint's A anchor after a Position commit — the
            // SAME door the canvas handles write through, so typing a pivot and
            // dragging it mean the same thing. ⚠️ Not the `anchored` sentinel the
            // old tail cleared: that re-derives BOTH locals, so editing X for the
            // A end would silently reset the B end the artist just placed.
            // A joint is a root entity, so its local translation IS world.
            if let Some((bits, world)) = joint_pivot_commit {
                let e = ph2d_ecs::Entity::from_bits(bits);
                if sim
                    .world()
                    .get::<ph2d_physics_ecs::PhysicsJoint>(e)
                    .is_some()
                {
                    physics.set_joint_anchor_world(sim, e, ph2d_physics_ecs::JointSide::A, world);
                }
            }
            // §12 Physics Joint (W3) — the joint edits and the one gesture that
            // creates a joint. Deletion is NOT here: a joint is an object, so
            // "Delete Joint" despawns it through the same path any other object
            // takes, and gets the same undo step for free.
            for &(bits, edit) in &joint_edits {
                // `PickBodyA/B` never reach here — they arm a canvas pick in the
                // action loop above (shell state, not a component edit). `Remove`
                // despawns the joint object; everything else is a field edit.
                if matches!(edit, ph2d_editor::JointFieldEdit::Remove) {
                    let e = ph2d_ecs::Entity::from_bits(bits);
                    let _ = sim.world_mut().despawn(e);
                } else {
                    inspector_joint::apply_joint_edit(
                        sim,
                        bits,
                        edit,
                        editor_queue,
                        component_registry,
                    );
                    // ⚠️ FLUSH per edit, exactly as `inspector_commits::dispatch`
                    // does for every OTHER Inspector edit type (§11 physics,
                    // ordering, blend, name…). `apply_joint_edit` only QUEUES a
                    // `SetComponent`; this block was moved OUT of that dispatch
                    // and shipped without the flush, so a joint parameter edit
                    // sat in the queue until some other edit happened to drain it
                    // — "sometimes it works". And it must be PER edit, not once
                    // after the loop: `apply_joint_edit` read-modify-writes the
                    // WHOLE component, so a second edit in the same frame that
                    // read a not-yet-applied first one would silently drop it
                    // (the same reason the ordering loop flushes per iteration).
                    if let Err(e) = ph2d_ecs::scene::apply_editor_commands(
                        sim.world_mut(),
                        editor_queue,
                        component_registry,
                    ) {
                        toasts.push(ph2d_editor::Toast::error(format!(
                            "Joint commit failed: {e}"
                        )));
                    }
                }
            }
            if join_draw_arm {
                // TOGGLE, nao "arma" (W-J4b): o gesto e modal e come o press no
                // canvas, entao sem uma saida o unico jeito de sair era completar
                // um joint que o artista nao queria. Pela porta unica
                // `toggle_joint_draw`, a MESMA que o Esc usa.
                crate::joint_draw::toggle(&mut self.joint_draw_armed, &mut self.joint_draw);
            }
            if join_chain {
                let (made, last) = crate::joint_draw::join_chain(
                    sim,
                    &inspector_selection,
                    inspector_joint::kind_of(self.join_kind),
                );
                // Select the LAST joint so §12 (Physics Joint) appears
                // immediately — the Kind selector and tuning are right there.
                // Otherwise the bodies stay selected (§11) and the joint is only
                // reachable by hunting for it in the Hierarchy by hand, which is
                // why creating anything but a Pin felt impossible. On a chain it
                // is the link that closes; showing one of the N is more honest
                // than showing none.
                if let Some(j) = last {
                    hero.gizmo.selection = Some(j.to_bits());
                    hero.gizmo.extra_selection.clear();
                }
                if made > 1 {
                    toasts.push(ph2d_editor::Toast::info(format!(
                        "Chained {} bodies with {made} joints",
                        made + 1
                    )));
                }
            }
            // W4 - bake the selection's simulated motion into curves. After the
            // joint work above because a baked body stops being simulated, and
            // the frame's other physics edits should land on the body as the
            // artist authored it.
            if let Some(bits) = bake_request {
                let entities: Vec<ph2d_ecs::Entity> = bits
                    .iter()
                    .map(|&b| ph2d_ecs::Entity::from_bits(b))
                    .collect();
                let (start, end) = physics_bake::bake_range(&self.timeline.doc, &self.playhead);
                let outcome = physics_bake::bake_selection(
                    &mut self.timeline,
                    physics,
                    sim,
                    &entities,
                    start,
                    end,
                    self.fixed_step.fixed_dt(),
                    self.bake_channels,
                    editor_queue,
                    component_registry,
                );
                if outcome.unmappable {
                    toasts.push(ph2d_editor::Toast::info(
                        "Cannot bake here: this clip does not play exactly once",
                    ));
                } else if outcome.refused {
                    toasts.push(ph2d_editor::Toast::info(
                        "Finish the current edit before baking",
                    ));
                } else if outcome.already_baked {
                    toasts.push(ph2d_editor::Toast::info(
                        "Already baked - the timeline drives these bodies now",
                    ));
                } else if outcome.is_empty() {
                    toasts.push(ph2d_editor::Toast::info("Nothing to bake: nothing moved"));
                } else {
                    // Back to the top, because that is where the animation the
                    // artist just made begins - and because the kind change only
                    // reaches rapier at tick 0 (`reconcile_structure`
                    // re-describes a body at rest), so this is also what makes
                    // the hand-over take effect.
                    //
                    // ⚠️ PAUSE as well, and the rewind alone is not enough:
                    // `Playhead::rewind` preserves the play state by design, and
                    // `advance_ticks` runs EARLIER in the frame than
                    // `physics_bridge::dispatch`. Still playing, the clock is
                    // already past 0 by the time the bridge looks, `at_rest` is
                    // never true again, and the flip never reaches rapier: the
                    // body keeps falling as Dynamic and the curve is discarded -
                    // exactly the "clicks Bake, nothing changes" failure the
                    // hand-over exists to prevent.
                    self.playhead.rewind();
                    self.playhead.pause();
                    // The window mirrors the button: a partial range (W-BakeRange)
                    // reads "2.0-5.0s", the common full-range bake just "5.0s".
                    let window = if start > 0.0 {
                        format!("{start:.1}-{end:.1}s")
                    } else {
                        format!("{end:.1}s")
                    };
                    toasts.push(ph2d_editor::Toast::info(format!(
                        "Baked {window} - {} bodies, {} tracks - now Kinematic",
                        outcome.bodies, outcome.tracks
                    )));
                }
            }
            // AutoKey (W4.T1/T2) — the single choke point. Runs HERE, after every
            // UI Transform/opacity write for the frame (gizmo early, Inspector +
            // Hierarchy reset just above) so it reads the settled pose of each
            // selected sprite and keys only what left its curve. Placed after the
            // apply pass too, so an undo/paste/scrub — which the apply writes back
            // to the world — reads world == curve and keys nothing.
            // The pass authors on the clock the APPLY drove this frame: the clip
            // playhead in Keys, the CONTAINER playhead inside one, the timeline's on
            // Arrange — the same three-way pick `timeline_bridge::run` made above, from
            // the same stamped facts (`keys_mode` / `container_open`, which the pass
            // reads to root its scratch). Handing it the wrong clock while a solo drives
            // the pose is how one strip in a lane killed auto-key (2026-07-22).
            let autokey_clock = if self.timeline.keys_mode {
                &self.clip_playhead
            } else if self.timeline.container_open.is_some() {
                &self.container_playhead
            } else {
                &self.playhead
            };
            autokey_pass::run(
                &mut self.timeline,
                autokey_clock,
                &mut self.autokey,
                toasts,
                hero,
                sim.world(),
            );
            // Merge Sprites (Enio 2026-05-27, Hierarchy right-click).
            // Drains BEFORE `image_edit::dispatch` so a same-frame
            // image-edit on one of the originals (extremely unlikely
            // path but documented) doesn't race with the despawn. The
            // multi-selection comes from `hero.gizmo` — primary first,
            // extras after. Right-clicked row resolves to the "primary
            // anchor" the merged sprite parents under.
            if let Some(row) = merge_sprites_row
                && let Some(live) = hero_live.as_ref()
                && let Some(primary_bits) = live.bridge.entity_for(row)
            {
                let in_selection = hero.gizmo.is_selected(primary_bits);
                let selected_count = hero.gizmo.iter_selected().count();
                // Audit B-M3: if the user right-clicked OUTSIDE the
                // multi-selection (and they already had 2+ sprites
                // selected), the previous behaviour silently fell back
                // to "single-entity merge → <2 warning" which read as
                // "select 2+ first" — misleading. Steer them to the
                // actual fix.
                if !in_selection && selected_count >= 2 {
                    toasts.push(ph2d_editor::Toast::warning(
                        "Merge Sprites: right-click on one of the selected sprites",
                    ));
                    self.title_dirty = true;
                } else {
                    let to_merge: Vec<u64> = if in_selection {
                        hero.gizmo.iter_selected().collect()
                    } else {
                        vec![primary_bits]
                    };
                    if hero_intents::drain_merge_sprites(
                        to_merge.clone(),
                        primary_bits,
                        hero.project.pixels_per_meter,
                        sim,
                        renderer,
                        asset_db,
                        atlas_asset_map,
                        toasts,
                    ) {
                        self.title_dirty = true;
                    }
                    // Clear merged entity_bits from gizmo so the global
                    // gizmo doesn't paint over vanished entities
                    // (mirror of Delete).
                    for bits in &to_merge {
                        if hero.gizmo.selection == Some(*bits) {
                            hero.gizmo.selection = None;
                        }
                        hero.gizmo.extra_selection.retain(|b| b != bits);
                    }
                    // Audit B-H2: promote the freshly-spawned merged
                    // entity to the selection so the user's next
                    // action (Move, Apply tool, etc.) operates on the
                    // merged result — matches Photoshop / Figma "after
                    // merge, the merged layer IS the selection".
                    if let Some(result) = hero_intents::take_last_merge_result() {
                        hero.gizmo.replace_selection(Some(result.new_entity_bits));
                    } else if hero.gizmo.selection.is_none()
                        && !hero.gizmo.extra_selection.is_empty()
                    {
                        // Merge bailed before spawning — promote oldest
                        // surviving extra (mirror of Delete's
                        // headless-cleanup path).
                        hero.gizmo.selection = Some(hero.gizmo.extra_selection.remove(0));
                    }
                }
            }
            // Hierarchy "Use as Brush Shape / Grain" → read the right-clicked sprite's pixels, install
            // them as the brush Shape (silhouette) or Grain (texture) image (Rec.601 luminance, mirror of
            // the file-load path), and activate the brush tool so the user can paint immediately. A
            // non-image row toasts + no-ops. Shape wins if both fired in one frame.
            let use_as_brush_intent = use_as_brush_shape_row
                .map(|r| (r, true))
                .or(use_as_brush_texture_row.map(|r| (r, false)));
            if let Some((row, as_shape)) = use_as_brush_intent
                && let Some(live) = hero_live.as_ref()
                && let Some(bits) = live.bridge.entity_for(row)
            {
                // Active painter document: read the LIVE layers NON-DESTRUCTIVELY — Shape captures the
                // layer stack (so the per-layer-colour feature works), Grain composites to luminance.
                // Crucially this does NOT bake/re-push the sprite: a re-push runs `set_source`, which
                // resets the LayerStack and would DESTROY the user's layers — the flatten bug Enio hit
                // (replaces the old auto-commit path; Enio 2026-06-26).
                let on_active_doc = self.last_painter_pushed_entity == Some(bits);
                let mut handled = false;
                if on_active_doc {
                    tools.set_active(&ph2d_editor::ToolId::new("painter"));
                    if let Some(painter) = tools.active_mut().and_then(|t| {
                        t.as_any_mut()
                            .downcast_mut::<ph2d_tool_painter::PainterTool>()
                    }) {
                        if as_shape {
                            painter.capture_layers_as_brush_shape();
                            toasts.push(ph2d_editor::Toast::success("Brush shape set from layers"));
                            handled = true;
                        } else if let Some((lum, w, h)) = painter.composite_to_lum() {
                            painter.set_brush_texture_image(lum, w, h);
                            toasts.push(ph2d_editor::Toast::success("Brush grain set from sprite"));
                            handled = true;
                        }
                    }
                }
                if !handled {
                    // A different (flat) sprite in the hierarchy — read its baked texture (no layers to
                    // lose), mirror of the file-load path.
                    let entity = ph2d_ecs::Entity::from_bits(bits);
                    match crate::hero_intents::texture_edit::read_sprite_source(
                        entity,
                        sim,
                        renderer,
                        asset_db,
                        atlas_asset_map,
                    ) {
                        Some(src) => {
                            let (w, h) = (src.image.width, src.image.height);
                            // Rec.601 luminance: weights 77/150/29 sum to 256, `>> 8` keeps `[0,255]`.
                            let lum: Vec<u8> = src
                                .image
                                .pixels
                                .chunks_exact(4)
                                .map(|p| {
                                    ((u32::from(p[0]) * 77
                                        + u32::from(p[1]) * 150
                                        + u32::from(p[2]) * 29)
                                        >> 8) as u8
                                })
                                .collect();
                            // Reach the painter only via the active tool → activate it first.
                            tools.set_active(&ph2d_editor::ToolId::new("painter"));
                            if let Some(painter) = tools.active_mut().and_then(|t| {
                                t.as_any_mut()
                                    .downcast_mut::<ph2d_tool_painter::PainterTool>()
                            }) {
                                if as_shape {
                                    painter.set_brush_shape_image(lum, w, h);
                                    toasts.push(ph2d_editor::Toast::success(
                                        "Brush shape set from sprite",
                                    ));
                                } else {
                                    painter.set_brush_texture_image(lum, w, h);
                                    toasts.push(ph2d_editor::Toast::success(
                                        "Brush grain set from sprite",
                                    ));
                                }
                            }
                        }
                        None => {
                            let what = if as_shape {
                                "Brush Shape"
                            } else {
                                "Brush Grain"
                            };
                            toasts.push(ph2d_editor::Toast::warning(format!(
                                "Use as {what}: select an image sprite"
                            )));
                        }
                    }
                }
                self.title_dirty = true;
            }
            // Hierarchy "Use as Watercolor Paper / Granulation" → read the row's pixels as luminance and
            // install them as the watercolor paper (Grain slot, canvas-anchored), turning the render-path
            // on so the wash granulates against the layer. Granulation wins if both fired in one frame.
            // Mirror of the "Use as Brush Grain" path above (`docs/Painter/10…` §5).
            let use_as_paper_intent = use_as_granulation_row
                .map(|r| (r, true))
                .or(use_as_paper_row.map(|r| (r, false)));
            if let Some((row, as_granulation)) = use_as_paper_intent
                && let Some(live) = hero_live.as_ref()
                && let Some(bits) = live.bridge.entity_for(row)
            {
                let on_active_doc = self.last_painter_pushed_entity == Some(bits);
                // Luminance: the active painter doc composites its layers (a Group of textures folds in);
                // a different flat sprite reads its baked texture (Rec.601, mirror of the file-load path).
                let lum_wh: Option<(Vec<u8>, u32, u32)> = if on_active_doc {
                    tools.set_active(&ph2d_editor::ToolId::new("painter"));
                    tools
                        .active_mut()
                        .and_then(|t| {
                            t.as_any_mut()
                                .downcast_mut::<ph2d_tool_painter::PainterTool>()
                        })
                        .and_then(|p| p.composite_to_lum())
                } else {
                    let entity = ph2d_ecs::Entity::from_bits(bits);
                    crate::hero_intents::texture_edit::read_sprite_source(
                        entity,
                        sim,
                        renderer,
                        asset_db,
                        atlas_asset_map,
                    )
                    .map(|src| {
                        let (w, h) = (src.image.width, src.image.height);
                        let lum: Vec<u8> = src
                            .image
                            .pixels
                            .chunks_exact(4)
                            .map(|p| {
                                ((u32::from(p[0]) * 77
                                    + u32::from(p[1]) * 150
                                    + u32::from(p[2]) * 29)
                                    >> 8) as u8
                            })
                            .collect();
                        (lum, w, h)
                    })
                };
                match lum_wh {
                    Some((lum, w, h)) => {
                        tools.set_active(&ph2d_editor::ToolId::new("painter"));
                        if let Some(painter) = tools.active_mut().and_then(|t| {
                            t.as_any_mut()
                                .downcast_mut::<ph2d_tool_painter::PainterTool>()
                        }) {
                            if as_granulation {
                                painter.use_layers_as_granulation(lum, w, h);
                                toasts.push(ph2d_editor::Toast::success(
                                    "Watercolor granulation set from layer",
                                ));
                            } else {
                                painter.use_layers_as_watercolor_paper(lum, w, h);
                                toasts.push(ph2d_editor::Toast::success(
                                    "Watercolor paper set from layer",
                                ));
                            }
                        }
                    }
                    None => {
                        let what = if as_granulation {
                            "Granulation"
                        } else {
                            "Watercolor Paper"
                        };
                        toasts.push(ph2d_editor::Toast::warning(format!(
                            "Use as {what}: select an image sprite"
                        )));
                    }
                }
                self.title_dirty = true;
            }
            // Image-edit drain phase + file-picker import — extracted
            // to sibling `image_edit.rs` as a free fn (Wave 3.2 stage A).
            // Returns whether any drain pushed a toast.
            // `padding_apply` carries a `Vec<u64>` (not `Copy`) — capture
            // the Apply-fired flag here so the teardown below can run
            // after the dispatch consumes the value.
            let padding_apply_fired = padding_apply.is_some();
            // Did a texture-RESIZING edit (rasterize / trim / make-square / real-size) hit the SELECTED
            // sprite? If so the Painter's working canvas is now the wrong resolution — reset the
            // push-tracker (below, after the lists are consumed) so `drive_source_push` re-reads the
            // sprite at its new size next frame, re-locking the brush / eyedropper / repeat-image.
            let painter_src_resized = hero.gizmo.selection.is_some_and(|sel| {
                rasterize_entities.contains(&sel)
                    || trim_entities.contains(&sel)
                    || make_square_entities.contains(&sel)
                    || real_size_entities.contains(&sel)
            });
            if image_edit::dispatch(
                trim_entities,
                make_square_entities,
                real_size_entities,
                rasterize_entities,
                padding_apply,
                color_equalization_apply.clone(),
                equalize_sizes_apply.clone(),
                upscale_apply.clone(),
                undo_image_edit,
                hero,
                sim,
                renderer,
                asset_db,
                atlas_asset_map,
                toasts,
                image_edit_undo,
                tools,
                camera,
                next_import_cell,
                &mut self.last_bgremoval_pushed_entity,
                &mut self.last_painter_pushed_entity,
            ) {
                self.title_dirty = true;
            }
            // A resize hit the selected sprite → force the Painter to re-read it at the new resolution
            // (see `painter_src_resized` above). The re-push replaces the now-invalid working canvas.
            if painter_src_resized {
                self.last_painter_pushed_entity = None;
            }
            // Apply teardown — runs AFTER the bake above (which needs
            // the BgRemovalTool still active to read the result). Now
            // that the committed alpha lives in the sprite texture,
            // deactivate the tool exactly like Cancel: the panel hides,
            // the sprite un-suppresses, the Inspector returns, and the
            // on-canvas preview overlay stops re-rendering on top of the
            // freshly baked sprite (that double-draw was the ghost edge
            // outline that appeared only while the image stayed selected).
            if bgremoval_apply_committed
                && let Some(default_id) = tools.default_tool_id()
                && tools.set_active(&default_id)
            {
                self.last_bgremoval_pushed_entity = None;
                self.bgremoval_preview = None;
                self.title_dirty = true;
            }
            // Padding Apply teardown — deactivate the tool so the panel
            // hides + the Inspector returns, exactly like Bg Removal.
            if padding_apply_fired
                && let Some(default_id) = tools.default_tool_id()
                && tools.set_active(&default_id)
            {
                self.title_dirty = true;
            }
            // Color Equalization Apply teardown — deactivate the tool
            // (panel hides, sprite returns to its un-edited live state
            // visually, multi-selection preserved). Mirror of Padding.
            if color_equalization_apply.is_some()
                && let Some(default_id) = tools.default_tool_id()
                && tools.set_active(&default_id)
            {
                self.last_color_equalization_pushed_entity = None;
                self.title_dirty = true;
            }
            // Equalize Sizes Apply teardown — same shape as Padding /
            // Color EQ: bake just ran, so switch back to the default
            // tool. The panel auto-hides because its `panel_visible`
            // gate keys off `tools.active().id() == "equalize_sizes"`
            // and the bridge clears the published snapshot on the
            // next frame.
            if equalize_sizes_apply.is_some()
                && let Some(default_id) = tools.default_tool_id()
                && tools.set_active(&default_id)
            {
                self.title_dirty = true;
            }
            // Upscale Apply teardown — mirror of Color EQ. Clear the
            // preview cache + push-tracker so re-activating starts
            // fresh against the new (post-bake) source.
            if upscale_apply.is_some()
                && let Some(default_id) = tools.default_tool_id()
                && tools.set_active(&default_id)
            {
                self.last_upscale_pushed_entity = None;
                self.upscale_preview = None;
                self.title_dirty = true;
            }
            // Painter Apply teardown (W1 T1.5) — same shape as BgR /
            // Upscale: deactivate the tool so the chrome returns to its
            // pre-painting state, and clear the preview/push-tracker so
            // re-activating starts fresh against the freshly-baked sprite.
            if painter_apply_committed
                && let Some(default_id) = tools.default_tool_id()
                && tools.set_active(&default_id)
            {
                self.last_painter_pushed_entity = None;
                self.painter_preview = None;
                self.title_dirty = true;
            }
            // Legacy `FloatingPanel` Procreate-style paint was retired
            // here (2026-05-17). The pink/magenta tab-strip + Accent
            // toggle decoration was inconsistent with the canonical
            // dark-glass surface used by Inspector / Hierarchy /
            // Widget Gallery. `Tool::build_panel()` still exists for
            // event dispatch but the visual is dropped; per-tool
            // chrome rewires through the new panel style in a
            // follow-up wave (BgRemoval especially needs its preview
            // panel re-painted; Move/Brush were stubs anyway).
            let _ = tools;
            toasts.paint(vector_scene, &mut paint_ctx);
            // The job bars share the toasts' column and stack UNDER them, so they are handed
            // the number of rows already spoken for. The count, not the geometry: the column's
            // ruler lives in `progress::column_row` and neither the shell nor the toast painter
            // gets to have an opinion about where row N is.
            jobs.paint_below(toasts.len(), vector_scene, &mut paint_ctx);
            // Drain frame-local arena AFTER the dispatch + paint pass
            // so any events emitted earlier this frame are still alive
            // for downstream consumers — wired in Phase A+ (currently
            // events are logged, not acted on).
            hero_arena.reset();
        } else {
            layout.paint(vector_scene, &mut paint_ctx);

            // Tool palette in the CREATE zone (top-right). Hidden in Zen
            // mode by virtue of `tool_palette_rects` returning empty.
            // This branch is the legacy no-hero (demo) path, so there is
            // no Image Tools mode → `mode_on = false`. Map slots through
            // the SAME `palette_visible_tool_indices` the click hit-test
            // uses so the two never drift (image tools filtered out when
            // off — no icon, no hit zone).
            let visible = crate::palette_visible_tool_indices(tools, false);
            let palette_rects = layout.tool_palette_rects(visible.len());
            let active_id = tools.active().map(|t| t.id());
            let palette_icons: Vec<(EditorRect, &str, bool)> = palette_rects
                .iter()
                .zip(visible.iter())
                .map(|(r, &i)| {
                    let tool = &tools.tools()[i];
                    let is_active = active_id.as_ref() == Some(&tool.id());
                    (*r, tool.label(), is_active)
                })
                .collect();
            ph2d_editor::paint_tool_palette_icons(
                paint_ctx.text,
                vector_scene,
                &palette_icons,
                paint_ctx.theme,
            );

            // Legacy `FloatingPanel` paint retired (2026-05-17). Same
            // rationale as the live-mode branch above. Tool palette
            // chrome above remains because it's the click entrypoint
            // to switch tools; the per-tool panel itself is gone.
            toasts.paint(vector_scene, &mut paint_ctx);
            jobs.paint_below(toasts.len(), vector_scene, &mut paint_ctx);
        }

        // Paint + present + title — extracted to `present.rs` sibling
        // method (Wave 3.2 stage A). Re-acquires self.gfx + self.host
        // refs inside; values needed are passed explicitly.
        self.run_present_phase(cpu_start, r, g, b);

        // Frame-phase profiler (PH2D_FLUID_PROFILE): the `[fluid]` line proves the
        // fluid drive is ~2 ms, so a 6-fps stall lives elsewhere. This splits the
        // frame: total vs CPU-encode (raw) → the gap is the present/GPU acquire
        // stall; plus the painter dispatch (CPU preview produce + upload).
        if frame_prof_on() {
            let n = FRAME_PROF_N.with(|c| {
                let n = c.get().wrapping_add(1);
                c.set(n);
                n
            });
            if n.is_multiple_of(120) {
                let total = self.frame_ms_ewma;
                let encode = self.frame_cpu_ms_ewma;
                let dispatch_ms = FRAME_PROF_DISPATCH_US.with(|c| c.get()) as f64 / 1000.0;
                // Perf-audit extension (2026-07-07): the phases where a paint slowdown can hide —
                // `tick` = active tool's on_tick (watercolor heartbeat recomposite), `stamp` = the
                // pointer-driven stamps since last frame (Move → apply_watercolor), `hero` = the
                // panel/chrome Vello encode (includes the Paper preview). `stamp` + `tick` happen
                // BEFORE cpu_start, so they add to `total` but NOT to `cpu-encode(raw)`.
                let hero_ms = FRAME_PROF_HERO_US.with(|c| c.get()) as f64 / 1000.0;
                // ⚠️ A JANELA, não a amostra: `média sobre os frames que trabalharam / pico / n`.
                // Ler um frame sorteado fazia um traço inteiro caber entre duas impressões e as
                // duas fases intermitentes lerem 0,00 enquanto o artista pintava.
                let take = |sum: &'static std::thread::LocalKey<std::cell::Cell<u64>>,
                            mx: &'static std::thread::LocalKey<std::cell::Cell<u64>>,
                            n: &'static std::thread::LocalKey<std::cell::Cell<u32>>|
                 -> (f64, f64, u32) {
                    let (s, m, k) = (
                        sum.with(std::cell::Cell::take),
                        mx.with(std::cell::Cell::take),
                        n.with(std::cell::Cell::take),
                    );
                    let avg = if k > 0 {
                        s as f64 / f64::from(k) / 1000.0
                    } else {
                        0.0
                    };
                    (avg, m as f64 / 1000.0, k)
                };
                let (tick_avg, tick_max, tick_n) = take(
                    &FRAME_PROF_TICK_SUM_US,
                    &FRAME_PROF_TICK_MAX_US,
                    &FRAME_PROF_TICK_N,
                );
                let (stamp_avg, stamp_max, stamp_n) = take(
                    &FRAME_PROF_STAMP_SUM_US,
                    &FRAME_PROF_STAMP_MAX_US,
                    &FRAME_PROF_STAMP_N,
                );
                // E o SPLIT do tick da água: um pico no `tool-tick` não diz se foi um passo de
                // sim caro ou um composite sobre um casco grande, e as curas são outras.
                let (
                    (sim_sum, sim_max, sim_n),
                    (comp_sum, comp_max, comp_n),
                    (wait_sum, wait_max, wait_n),
                ) = ph2d_tool_painter::wet_diag::take_window();
                // A partição do WORKER (`wet_diag`): com a sim fora da thread do frame, a linha
                // `sim` acima imprimia `0.00ms x0` — ninguém media o passo, porque quem o dá é o
                // worker. Estes três baldes dizem se a água lenta é TRABALHO (busy alto: só a GPU
                // move) ou AGENDAMENTO (away/sleep alto), que têm curas opostas.
                let (busy, away, sleep) = ph2d_tool_painter::wet_diag::take_worker();
                let span = f64::from(total) * 120.0;
                let pct = |x: f64| if span > 0.0 { 100.0 * x / span } else { 0.0 };
                let per = |sum: f64, n: u64| if n > 0 { sum / n as f64 } else { 0.0 };
                eprintln!(
                    "[frame] total={total:.2}ms (~{:.0} fps) | cpu-encode(raw)={encode:.2}ms \
                     | present/acquire-stall={:.2}ms | painter-dispatch(cpu)={dispatch_ms:.2}ms \
                     | hero-paint={hero_ms:.2}ms\n\
                     [frame]   tool-tick: media {tick_avg:.2}ms pico {tick_max:.2}ms em {tick_n}/120 frames \
                     | stamps: media {stamp_avg:.2}ms pico {stamp_max:.2}ms em {stamp_n}/120\n\
                     [frame]   agua: sim media {:.2}ms pico {sim_max:.2}ms x{sim_n} \
                     | composite media {:.2}ms pico {comp_max:.2}ms x{comp_n} \
                     | ESPERA media {:.2}ms pico {wait_max:.2}ms x{wait_n} (total {:.0}ms)\n\
                     [frame]   worker: busy {:.0}% away {:.0}% sleep {:.0}% \
                     | TAXA DA AGUA {:.1} Hz ({sim_n} passos em {:.1}s)",
                    1000.0 / f64::from(total).max(0.001),
                    (f64::from(total) - f64::from(encode)).max(0.0),
                    per(sim_sum, sim_n),
                    per(comp_sum, comp_n),
                    per(wait_sum, wait_n),
                    wait_sum,
                    pct(busy),
                    pct(away),
                    pct(sleep),
                    if span > 0.0 {
                        1000.0 * sim_n as f64 / span
                    } else {
                        0.0
                    },
                    span / 1000.0,
                );
            }
        }
    }
}
