#![forbid(unsafe_code)]
// ph2d-loc-cap: crate-root module hub — the 80+ `mod` declarations are an
// append-only extension point (every drop-in line adds one, and `mod`
// declarations cannot leave the crate root) alongside the winit `App` entry
// impl + `fn main`. Grew past the HR-18 cap by cross-line `mod` accumulation
// during the 2026-07-17 multi-line integration; the growth is structural.

//! Desktop shell — winit 0.30 + wgpu + ECS + sprite render + M6+M7+M12.
//!
//! Run with: `cargo run -p ph2d-host-desktop`
//!
//! Layered subsystems (each gated to keep the demo bootable even if
//! one fails — never crash the shell over an integration-demo issue):
//! - **M5** SpriteRenderer + 1000-sprite Vogel spiral with bouncing motion
//! - **M6** AssetDb loads 16 real PNG files from `assets/sprites/` (auto-
//!   generated on first launch) and composes them into a 256×256
//!   RGBA8 atlas. Falls back to procedural dummy if anything fails.
//! - **M7** ScriptHost with placeholder Luau script; per-frame gc_step
//!   keeps the GC budget warm for future script-driven gameplay.
//! - **M12** editor data layer: `ZenMode` (Tab toggle), `ToastQueue`
//!   (T key adds info toast), theme switch (M key flips Dark↔Light),
//!   and a `FloatingPanel` (Procreate-style selection demo). Visible
//!   via window title since Vello widget paint requires sharing the
//!   wgpu Surface with the sprite pipeline (see `integration.rs`).
//!
//! M8 add-on (gamepad path):
//! - gilrs adapter (`gilrs_adapter` module) pumps gamepad events into
//!   [`ph2d_input::InputState`] each frame, BEFORE sim tick.
//! - Connection / button / axis events log to the terminal at the
//!   `[Nms]` timestamp prefix used by the rest of the shell.
//! - Axis logs filter dead-zone jitter (only `|value| > 0.25`).
//! - Pencil events are wired through the abstraction but not produced
//!   by this shell (iPad shell will, in M9+).
//!
//! Out of scope here:
//! - **M8 → ScriptHost**: `InputState` lives on `App`, but routing into
//!   `ph2d.input` Luau snapshot is a follow-up.
//! - **M11** Vello text/widget overlay — needs surface-sharing pass.

mod adapter_smoke;
mod align_live;
mod align_smoke;
mod anchor_smoke;
mod app_state;
mod atlas_loader;
mod attribute_demo_smoke;
mod audio;
/// **O OBJETO ASSADO** (`docs/3D/02.2`, rota A) — os canais que uma malha doou a um sprite e a luz
/// que os le'. ⚠️ Deliberadamente FORA da feature `sculpt3d`: um objeto assado sobrevive ao modulo.
mod baked_form;
/// Blend Objects vivos (ADR-0128): o objeto único que interpola 2..=5 formas e as segue
/// (re-cook por frame). Espelha `connector_live`.
mod blend_live;
mod blend_smoke;
mod body_fk;
mod body_grab;
mod body_pose;
/// Os GESTOS da booleana viva: armar (criar/re-mirar) e consolidar. O documento mora aqui; o
/// motor, no `bool_live`.
mod bool_gesture;
/// A BOOLEANA VIVA (plano UI/UX W1): um grupo cujos filhos se combinam e continuam editáveis.
/// O 7º produtor de `LiveGeometry`, e o segundo que TRANSFORMA o mapa em vez de o estender.
mod bool_live;
/// A cena de smoke da booleana viva (`PH2D_BUILD_SMOKE=48`) — irmã de `build_smoke`, teto de LOC.
mod bool_smoke;
mod buffer_smoke;
mod build_smoke;
mod build_smoke_corner_tools;
mod build_smoke_drive;
/// A cena de smoke do **Expand** (Outline Stroke + Offset Path) — `PH2D_BUILD_SMOKE=17`.
mod build_smoke_expand;
mod build_smoke_router;
/// A lei do **zoom do canvas** — a roda escreve um destino, o quadro publica o vivo.
mod canvas_zoom;
mod chrome_hit;
/// Teclado do palette de "Add Node" (busca/filtro, Enter/Backspace/Esc).
mod command_palette_input;
/// **Contour** (pesquisa `20_*` #9) — o cozimento vivo do `VecContour`: N anéis concêntricos
/// com rampa de cor, irmão do `offset_live` de que é a generalização.
mod component_pieces_smoke;
mod component_resize_smoke;
mod component_smoke;
/// O gesto que cria um conector (Down numa forma, Up noutra).
mod connector_gesture;
/// Conectores vivos: a linha que gruda em duas formas e as segue (re-cook por frame).
mod connector_handles;
mod connector_live;
mod contour_live;
/// A cena de smoke do **Contour** (`PH2D_BUILD_SMOKE=25`) — irmã de `build_smoke`, teto de LOC.
mod contour_smoke;
mod corner_handles;
mod cursor_pos;
/// **O Width Tool** — as alças de largura na curva (plano 25 W2, ADR-0148).
mod cut_smoke;
/// O canal da **DOAÇÃO de forma** para a tinta do Painter — plano de normais + o tamanho do canvas.
/// Sem `cfg`, de propósito: o que atravessa é `Vec<f32>`, nunca um tipo do módulo 3D.
mod donated_form;
mod driven_row_smoke;
mod echo_family_smoke;
mod emitter_smoke;
mod envelope_gesture;
mod envelope_live;
/// As cenas de smoke do Envelope (ADR-0129) — irmão de `build_smoke`, teto de LOC.
mod envelope_smoke;
mod expr_blend_smoke;
mod extrap_smoke;
mod falloff_smoke;
/// Motion Nodes: o gizmo de canvas de um field espacial (`field.box`, …). Espelho do
/// `flip_selection_gizmo` — `GizmoTarget::MotionField`, apply nos params do NÓ.
mod field_gizmo;
mod flip_airbrush_smoke;
mod flip_autokey;
mod flip_colorize;
mod flip_colorize_smoke;
mod flip_demo;
mod flip_draw;
mod flip_edit_gesture;
mod flip_edit_smoke;
mod flip_entities;
mod flip_erase;
mod flip_fill;
mod flip_fill_dilate;
mod flip_fill_smoke;
mod flip_fill_target;
mod flip_gap_live;
mod flip_gizmo_view;
mod flip_hardness_smoke;
mod flip_layers;
mod flip_multiframe;
mod flip_multiplane_smoke;
mod flip_peek;
mod flip_pose_gizmo;
mod flip_pose_smoke;
mod flip_pressure_smoke;
mod flip_resample_smoke;
mod flip_reshape;
mod flip_segment_smoke;
mod flip_select;
mod flip_select_pick;
mod flip_select_points;
mod flip_select_segment;
mod flip_selection_gizmo;
mod flip_selection_smoke;
mod flip_self_overlap_smoke;
mod flip_smooth;
mod flip_strip;
mod flip_strip_drag;
mod flip_strip_pins;
mod flip_strip_resolve;
mod flip_strip_smoke;
mod flip_tip_smoke;
mod flip_trace;
mod flip_transform;
mod flip_tween_correct;
mod flip_tween_pairs_smoke;
mod flip_tween_phase_smoke;
mod flip_tween_smoke;
mod flip_tween_torsion_smoke;
mod forwarding;
/// A cena de smoke da MOLDURA (`PH2D_BUILD_SMOKE=49`) — irmã de `build_smoke`, teto de LOC.
mod frame_smoke;
mod fx_adjust_smoke;
mod fx_atlas;
mod fx_blend_smoke;
mod fx_bridge;
mod fx_bridge_dispatch;
/// **FX raster VIVO** — o cozimento do `ph2d_ecs::VecFilter` (Blur/Glow/Drop Shadow, plano 24):
/// isola a forma, borra/tinge, e injeta a imagem no z dela via `ph2d_vec_render::FxImages`.
mod fx_dump;
mod fx_duotone_smoke;
mod fx_gradient_map_smoke;
mod fx_live;
mod fx_live_hit;
mod fx_live_memo;
mod fx_live_resolve;
mod fx_morphology_smoke;
mod fx_raster_smoke;
mod fx_silhouette;
mod fx_smoke;
mod fx_turbulence_smoke;
mod fx_undo_smoke;
mod gizmo_anchor_smoke;
mod global_palette_input;
mod gradient_smoke;
mod grid_smoke;
mod guide_gesture;
mod guide_smoke;
/// A cena de smoke das Color Harmonies (abre o picker com Triad) — `PH2D_HARMONY_SMOKE=1`.
mod harmony_smoke;
mod hero_bridge;
mod hero_intents;
mod image_import;
mod impasto_smoke;
mod init;
mod input_dispatch;
mod input_drop;
mod input_handlers;
mod input_log;
mod instance_live;
mod integration;
mod joint_anchor_drag;
mod joint_draw;
mod joint_rig;
mod joint_rig_drag;
mod keymap;
/// A cena de smoke do Knot (o entrelace celta over/under) — irmão de `build_smoke`.
mod knot_smoke;
mod ktx2_smoke;
mod label_live;
mod lasso_smoke;
mod layout_live;
mod layout_reorder;
/// `layout_scroll_gesture`: a roda que rola uma moldura (o motor e' o `layout_live::scroll`).
mod layout_scroll_gesture;
mod layout_smoke;
mod lens_smoke;
mod line_smoke;
/// SONDA (`--ignored`): quanto custa MOVER uma forma que tem geometria viva. A §11 do plano 25
/// afirma que todo memo de geometria e' chaveado no MUNDO — esta sonda pergunta ao produto.
#[cfg(test)]
#[path = "live_memo_probe.rs"]
mod live_memo_probe;
mod mask_smoke;
mod morph_fade_smoke;
mod morph_live;
mod motion_autofix_smoke;
mod motion_autofix_smoke_appropriate;
mod motion_autofix_smoke_dead_branch;
mod motion_delay_smoke;
mod motion_flip_bake;
mod motion_fx_smoke;
mod motion_node_path_smoke;
mod motion_object_bake;
mod motion_object_smoke;
mod motion_path_smoke;
/// O tile de uma forma PARAMÉTRICA (`source.shape`) — irmão do `motion_object_bake`,
/// e a metade que faz o glow alcançar as formas (bug do Enio, 2026-08-20).
mod motion_shape_bake;
mod motion_shape_smoke;
mod motion_shape_smoke_knobs;
mod motion_state;
mod multi_node_smoke;
mod name_unique;
mod nest_smoke;
mod node_reach_smoke;
mod node_xy_smoke;
/// **Expand** — os cliques de Offset Path / Outline Stroke (o motor é
/// `ph2d_vec_boolean::expand`; aqui mora o que é de documento: z, pose e undo).
mod offset_live;
/// Onion settings modal — the shell half (ADR-0142 W3b): store→onion read-back + the title-band drag.
mod onion_modal;
mod osc_ruler_smoke;
mod palette_persist;
/// **Pattern Along Path** — o cozimento vivo do `VecPatternPath` (plano 23), irmão do `offset_live`.
mod pattern_live;
mod pattern_path_smoke;
mod pencil_smoke;
mod physics_smoke;
mod physics_smoke_authoring;
mod physics_smoke_blast;
mod physics_smoke_brake;
mod physics_smoke_brink;
#[cfg(test)]
mod physics_smoke_brink_tests;
mod physics_smoke_collider;
mod physics_smoke_collision;
mod physics_smoke_compound;
mod physics_smoke_contacts;
mod physics_smoke_damping;
mod physics_smoke_events;
mod physics_smoke_fk;
/// W-PartFace: a chave e a fenda -- editar uma PECA muda o resultado.
mod physics_smoke_foot;
/// W-Probes2: a perna e um LEQUE -- a fenda que o corpo atravessa.
mod physics_smoke_foot_fan;
mod physics_smoke_glide;
mod physics_smoke_grab;
mod physics_smoke_ik;
mod physics_smoke_interact;
mod physics_smoke_joint_anim;
mod physics_smoke_joint_bake;
mod physics_smoke_joint_break;
mod physics_smoke_joint_copy;
mod physics_smoke_joint_custom;
mod physics_smoke_joint_draw;
mod physics_smoke_joint_glyphs;
mod physics_smoke_joint_handles;
mod physics_smoke_joint_motor;
mod physics_smoke_joint_pair;
mod physics_smoke_joint_pose;
mod physics_smoke_joint_rig;
mod physics_smoke_joint_slider;
mod physics_smoke_kin_pure;
mod physics_smoke_kin_push;
mod physics_smoke_kin_water;
mod physics_smoke_kinematic;
mod physics_smoke_lead;
mod physics_smoke_leave;
mod physics_smoke_ledge;
mod physics_smoke_multi_jump;
mod physics_smoke_out;
mod physics_smoke_part;
mod physics_smoke_player;
mod physics_smoke_player_bake;
mod physics_smoke_player_carry;
mod physics_smoke_player_crouch;
mod physics_smoke_player_dash;
mod physics_smoke_player_drop;
mod physics_smoke_player_flank;
mod physics_smoke_player_forgive;
mod physics_smoke_player_grab;
mod physics_smoke_player_run;
mod physics_smoke_player_slope;
mod physics_smoke_player_tape;
mod physics_smoke_player_wall;
mod physics_smoke_probes;
mod physics_smoke_props;
mod physics_smoke_pulley;
mod physics_smoke_pulley_break;
mod physics_smoke_pulley_comp;
mod physics_smoke_pulley_diff;
mod physics_smoke_pulley_tackle;
mod physics_smoke_pulley_weston;
mod physics_smoke_raft;
mod physics_smoke_rail_rope;
mod physics_smoke_rig;
mod physics_smoke_rigs;
mod physics_smoke_rod;
mod physics_smoke_signal;
mod physics_smoke_signal_leave;
mod physics_smoke_soft_weld;
mod physics_smoke_stone;
mod physics_smoke_stop;
mod physics_smoke_surface;
mod physics_smoke_swim;
mod physics_smoke_terminal;
mod physics_smoke_water;
mod physics_smoke_wheel;
mod physics_smoke_world_pin;
mod physics_smoke_zone_force;
mod physics_smoke_zones;
mod picker_smoke;
mod player_input;
mod prefs;
/// **A sonda da §4.3 do plano da UI viva** — o cursor pode ser PRESO nesta máquina? Só de teste:
/// ela abre janela e precisa de uma mão mexendo o rato, então nunca entra num build de produto.
#[cfg(test)]
mod probe_cursor_grab;
/// **A largura VIVA** — o cozimento do `VecStrokeProfile` (ADR-0148), irmão do `offset_live`.
mod profile_live;
/// A cena de smoke da **largura viva** (`PH2D_BUILD_SMOKE=41`) — irmã de `build_smoke`, teto de LOC.
mod profile_smoke;
mod project;
/// **Os canais assados dentro do arquivo** (ADR-0150 W8.7) — gemeo do `project_painter`.
mod project_baked_form;
mod project_painter;
mod project_schema;
/// **As settings do PROJETO viajam no arquivo** (doc 88, D3) — a escala do mundo e a
/// unidade que o artista lê; irmão de `project` pelo teto de LOC.
mod project_settings;
/// **Os pixels próprios de um sprite dentro do arquivo** (plano `docs/Sprite_projeto/17` §3) —
/// irmão do `project_painter`, e o chão que faltava debaixo dele: cobre o funil que TODAS as
/// ferramentas de imagem atravessam, não um produtor só.
mod project_sprite_pixels;
/// **A tabela de COR autorada viaja no arquivo** (plano UI/UX W6) — irmão de `project`
/// pelo teto de LOC, cortado por assunto.
mod project_tokens;
mod render_loop;
mod scroll_smoke;
#[cfg(feature = "sculpt3d")]
mod sculpt3d;
mod shape_build;
mod shape_build_gesture;
mod signal_smoke;
/// ⭐ A cena da TABELA SINAL → PAPEL (`PH2D_BUILD_SMOKE=68`) — ⚠️ NÃO é o `signal_smoke`, que é
/// a cena do R0 (`PH2D_SIGNAL_SMOKE`): ali o assunto é a SAÍDA, aqui é o CONSUMIDOR.
mod signal_table_smoke;
mod sim_populate;
mod sizing_smoke;
/// As cenas de smoke do Sketch (=31) e do Hatch (=32) — irmão de `build_smoke`, teto de LOC.
mod sketch_hatch_smoke;
mod smoke_layout;
mod smoke_script;
mod snap_label_smoke;
mod splice_smoke;
mod stack_smoke;
mod stagger_smoke;
mod substrate_smoke;
mod symmetry_live;
/// A cena de smoke da SIMETRIA de desenho (`PH2D_BUILD_SMOKE=46`) — irmã de `build_smoke`.
mod symmetry_smoke;
mod taper_smoke;
mod text_fx_smoke;
mod text_path_gesture_smoke;
mod text_path_smoke;
/// A cena de smoke do **REFLUXO** de texto (`PH2D_BUILD_SMOKE=63`) — irmã de `build_smoke`.
mod text_wrap_smoke;
mod theme;
mod timeline_onion_smoke;
#[cfg(test)]
#[path = "timeline_orphan_tests.rs"]
mod timeline_orphan_tests;
mod timeline_persist;
mod timescale_smoke;
/// **AS MOLDURAS** (plano UI/UX W0): que intervalo da pilha de z cada `VecFrame` recorta. A
/// metade que a shell possui — o renderer sabe desenhar, a shell sabe a ÁRVORE.
mod token_smoke;
/// A cena de smoke dos **TOKENS** (`PH2D_BUILD_SMOKE=59`) — o painel que re-veste o app.
mod tokens_smoke;
mod transform_family_smoke;
mod transport;
/// A cena de smoke do Twist (o remoinho + o Falloff a modulá-lo) — irmão de `build_smoke`.
mod twist_smoke;
/// ⭐ O smoke da UI VIVA (`PH2D_UI_MOTION_SMOKE`) — o carácter e a corda.
mod ui_motion_smoke;
/// A cena de smoke da **HIERARQUIA** de estados (`PH2D_BUILD_SMOKE=64`) — irmã de `build_smoke`.
mod ui_nested_smoke;
/// A cena de smoke do **PAINEL GERADO** (`PH2D_BUILD_SMOKE=62`) — irmã de `build_smoke`.
mod ui_panel_smoke;
/// **Que painel esta árvore descreve** (plano UI/UX W8b) — a porta única que lê a moldura
/// autorada e devolve o `PanelSpec` que o gerador escreve.
mod ui_panel_spec;
/// **O RATO dentro do modo de preview** (plano UI/UX W7r) — a metade da shell do
/// `render_loop::ui_preview`: quem aponta, e o gesto modal que precede tudo.
mod ui_preview_gesture;
/// A cena de smoke da **MOLA** (`PH2D_BUILD_SMOKE=65`) — irmã de `build_smoke`.
mod ui_spring_smoke;
/// A cena de smoke dos **ESTADOS DE UI** (`PH2D_BUILD_SMOKE=61`) — irmã de `build_smoke`.
mod ui_states_smoke;
mod undo;
mod undo_route;
mod units_smoke;
mod value_curve_smoke;
mod value_gain_smoke;
mod value_median_smoke;
mod value_mix_smoke;
mod value_noise_smoke;
mod value_normalize_smoke;
mod value_pattern_smoke;
mod value_percentile_smoke;
mod value_quantize_smoke;
mod value_reduce_smoke;
mod value_slope_smoke;
mod value_smooth_smoke;
mod value_step_smoke;
mod value_time_smoke;
mod value_unary_smoke;
mod value_wave_smoke;
mod value_wrap_smoke;
/// A cena de smoke dos **VARIANTS** (`PH2D_BUILD_SMOKE=58`) — irmã de `component_pieces_smoke`.
mod variant_smoke;
mod vec_anchor_edit;
mod vec_bindings;
mod vec_blend;
mod vec_component_edit;
mod vec_component_pieces;
/// O painel edita o CONECTOR selecionado (Route / Jetty / Spread) — resolve o valor
/// EFETIVO que o painel exibe e aplica a edição a TODOS os conectores selecionados.
mod vec_connector_panel;
/// Diagnóstico do overlay vetorial (`PH2D_VEC_OVERLAY_DIAG=1`) — nomeia o dono de geometria fora
/// do lugar, em vez de a adivinhar.
mod vec_convert;
/// **A LINHA DE CORTE** (plano 25 §7): o caminho que a tesoura usa como lâmina — adotado depois
/// do `sync`, desenhado pelo overlay, e nunca alvo do próprio corte. Espelha `connector_live`.
mod vec_cut_line;
mod vec_entities;
mod vec_expand;
mod vec_font;
#[cfg(feature = "panel-vector")]
mod vec_font_preview;
/// A moldura da SELEÇÃO (plano UI/UX W0): o que o painel mostra, e o que o chip escreve.
mod vec_frame_edit;
mod vec_frame_labels;
mod vec_frame_resize;
mod vec_frame_spans;
mod vec_gizmo_view;
mod vec_glyph;
mod vec_glyph_build;
/// A porta única de "onde está o caminho-guia, e como se percorre por arco?" (texto E pattern).
mod vec_guide;
/// A CÓPIA segue a âncora do mestre — o corolário da âncora viva, do lado do componente.
mod vec_instance_follow;
mod vec_layout_edit;
mod vec_marquee;
mod vec_overlay;
mod vec_overlay_diag;
mod vec_pencil_input;
/// O **Picker de caminho-guia** — o gesto de duas mãos partilhado pelo Pattern e pelo Text on Path.
mod vec_pick;
mod vec_resize_box_edit;
mod vec_selection;
mod vec_shape_live;
mod vec_shape_params;
mod vec_snap;
/// Os alvos de snap vindos do RASTER (irmão de `vec_snap`, teto de LOC).
mod vec_snap_sprites;
mod vec_text;
mod vec_text_object;
mod vec_text_reopen;
mod vec_text_ride;
mod vec_transform;
/// **OS VERBOS DA PELE** (plano UI/UX W6.2) — vestir, trocar de tipo, despir.
mod vec_ui_state_edit;
/// **OS VARIANTS** (plano UI/UX W5c) — que versão do componente uma instância é. Um conjunto de
/// variants é DERIVADO (os mestres irmãos), e os eixos saem dos NOMES: zero componente novo.
mod vec_variants;
mod vec_widget_drive;
mod vec_widget_edit;
mod vec_widget_value;
mod warp_smoke;
mod wetpaint_smoke;
/// **O DESENHO É O GLIFO** — a porta única que normaliza a forma de um `IconButton` na caixa de
/// 24×24. Ela é UMA porque o canvas e o codegen precisam do mesmo glifo por motivos diferentes.
mod widget_icon;
/// **A PELE por-widget** (plano UI/UX W6.2) — uma forma marcada é pintada pelo pintor REAL do
/// catálogo, no z dela. A ponte mora aqui porque só a shell alcança as duas metades.
mod widget_live;
mod widget_skin_smoke;
mod width_handles;
/// A cena de smoke do **Width Tool** (`PH2D_BUILD_SMOKE=42`) — irmã de `build_smoke`, teto de LOC.
mod width_tool_smoke;
mod winit_host;
mod zorder_smoke;

pub(crate) use app_state::{
    App, AppGfx, HeroLive, ImageEditSnapshot, ImageEditTransaction, is_image_edit_tool,
    palette_visible_tool_indices,
};

// forwarding::* moved to input_dispatch.rs (PR 9b).
// cursor_pos::live_cursor_in_window + image_import::import_images_grid
// moved to input_handlers.rs (Wave 3.2 stage B).
use input_log::log_input_event;
// keymap::winit_to_editor_keycode moved to input_dispatch.rs (PR 9b).
// theme::parse_theme_env moved to init.rs (PR 9c).
use winit_host::LoggingHandler;

use ph2d_core::{FixedStep, Playhead, Vec2, install_panic_hook, panic};
use ph2d_ecs::scene::build_hierarchy_snapshot;
use ph2d_ecs::{Component, SimComponent, SimWorld, Transform};
use ph2d_editor::paint::Paint;
// NodeId surfaces in our `dragging` field; re-exported by ph2d-editor.
use ph2d_editor::NodeId;
use ph2d_host::{HostHandler, Lifecycle, Modifiers, PlatformHost};
use ph2d_input::InputState;
use ph2d_render::SpriteRenderer;
use std::time::Instant;

mod gilrs_adapter;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::ModifiersState;
use winit::window::WindowId;

pub(crate) const SPRITE_COUNT: u32 = 1000;
/// Half-extent of the bouncing world in meters. Camera default has
/// `height_world = 10`, so [-5, 5] in Y is exactly the visible region;
/// X depends on aspect (narrower than visible at 4:3+).
pub(crate) const WORLD_HALF: f32 = 5.0;

// ADR-0025: `Position(Vec2)` removed — `Transform` from `ph2d_ecs` is
// the canonical pose component now. `Velocity` stays local to the
// demo because no other crate consumes it yet (M14.2+ may promote it).
#[derive(Component, Copy, Clone, Debug)]
pub(crate) struct Velocity(pub(crate) Vec2);
impl SimComponent for Velocity {}

impl App {
    fn new() -> Self {
        let gilrs = match gilrs::Gilrs::new() {
            Ok(g) => {
                let pads: Vec<String> = g
                    .gamepads()
                    .map(|(id, pad)| format!("[{:?}] {}", id, pad.name()))
                    .collect();
                if pads.is_empty() {
                    println!("gilrs: initialized; no gamepads connected yet");
                } else {
                    println!("gilrs: detected {} gamepad(s):", pads.len());
                    for p in &pads {
                        println!("  {p}");
                    }
                }
                Some(g)
            }
            Err(e) => {
                eprintln!("gilrs init failed (continuing without gamepad): {e}");
                None
            }
        };
        // Phase 2.1/2.2: open the audio device (None = run silent). Env smokes:
        // `PH2D_AUDIO_SMOKE` plays a 440 Hz beep; `PH2D_AUDIO_FILE=<path>`
        // decodes + loop-plays a real audio file.
        let mut audio = crate::audio::AudioSystem::new();
        if let Some(a) = audio.as_mut() {
            if std::env::var_os("PH2D_AUDIO_SMOKE").is_some() {
                a.play_test_tone();
            }
            if let Some(path) = std::env::var_os("PH2D_AUDIO_FILE") {
                a.play_file(std::path::Path::new(&path));
            }
            // Stage a ready-to-audition loop in the editor (open the Audio Editor pill
            // to see it) — the W6 loop-points smoke, no file picking needed.
            if std::env::var_os("PH2D_AUDIO_LOOP_SMOKE").is_some() {
                a.editor_loop_smoke();
            }
            // Stage the Multiband A/B: the clip that exposes it (a kick over steady
            // highs) plus a two-stage rack, Multiband vs Compress at the same Ratio.
            if std::env::var_os("PH2D_AUDIO_MULTIBAND_SMOKE").is_some() {
                a.editor_multiband_smoke();
            }
            // Stage the W4 voice family: synthesised speech + a rack holding the Vocoder at
            // both ends of its Breath knob (robot / whisper) and the Granular.
            if std::env::var_os("PH2D_AUDIO_VOICE_SMOKE").is_some() {
                a.editor_voice_smoke();
            }
            // Stage the W6 shipping targets: a clip whose 15 kHz shimmer a 24 kHz variant
            // physically cannot carry, plus a loop + markers only the lossless target keeps.
            if std::env::var_os("PH2D_AUDIO_DELIVERY_SMOKE").is_some() {
                a.editor_delivery_smoke();
            }
            // Stage the ADR-0120 knob drag: a 3-minute clip (where the whole-clip copy hurts), a
            // selection, and a ONE-stage rack. Drag Ratio; each frame prints its cost. Re-run with
            // PH2D_AUDIO_SLOW_PREVIEW=1 for the old path -- the feature is byte-identical, so the
            // A/B is the only thing a human can actually check.
            if std::env::var_os("PH2D_AUDIO_KNOB_SMOKE").is_some() {
                a.editor_knob_smoke();
            }
            // Stage the W7 AI Denoise smoke: a voiced tone buried under broadband hiss at ~0 dB
            // SNR. Play, click AI Denoise (needs `--features audio-ml`), play again -- the hiss
            // falls away. The hand on the door for the +12 dB the parity gate already proved.
            if std::env::var_os("PH2D_AUDIO_ML_SMOKE").is_some() {
                a.editor_ml_smoke();
            }
        }
        Self {
            window: None,
            host: None,
            gfx: None,
            exiting: false,
            handler: LoggingHandler::new(),
            fixed_step: FixedStep::default(),
            // Start paused: the transport should not run the moment the window
            // opens (the playhead default is "playing" — foundational, motion
            // relies on it — so the app pauses its own playhead explicitly).
            playhead: {
                let mut ph = Playhead::default();
                ph.pause();
                ph
            },
            // The Keys view's clip-time clock — paused for the same reason, and its
            // own so scrubbing/playing a clip's keys never disturbs the timeline.
            clip_playhead: {
                let mut ph = Playhead::default();
                ph.pause();
                ph
            },
            // The Containers view's interior-time clock — paused, its own, for the
            // same reason: playing a container's lanes never disturbs the scene.
            container_playhead: {
                let mut ph = Playhead::default();
                ph.pause();
                ph
            },
            last_timeline_keys_mode: false,
            last_timeline_container: None,
            timeline_last_selected: None,
            // A 4 s composition on open, not an open-ended one pinned at t = 0 (Enio,
            // 2026-07-23): an AUTHORED default duration so the comp end — and the veil
            // past it — is there from the first frame. A loaded project keeps its own
            // saved duration (`apply_project` replaces this whole `TimelineState`).
            timeline: ph2d_timeline::TimelineState::with_default_duration(),
            timeline_intents: Vec::new(),
            timeline_reveal_after_apply: false,
            timeline_view: ph2d_timeline::TimelineViewSnapshot::default(),
            timeline_signals: Default::default(),
            signals: ph2d_runtime::SignalOutbox::new(),
            signal_toast_reader: ph2d_runtime::SignalReader::new(),
            // O 2o consumidor so' existe com a env var -- o idiom de diagnostico da casa
            // (`PH2D_PAINT_PERF`, `PH2D_FLUID_PROFILE`): um leitor que ninguem pediu seria
            // um cursor a envelhecer sozinho, acumulando `missed` que ninguem le.
            signal_log_reader: std::env::var_os("PH2D_SIGNAL_LOG")
                .map(|_| ph2d_runtime::SignalReader::new()),
            ui_signal_reader: ph2d_runtime::SignalReader::new(),
            timeline_insert_key: false,
            autokey: Default::default(),
            last_frame: Instant::now(),
            pending_resize: None,
            resize_saved_present_mode: None,
            resize_settle_frames: 0,
            modifiers: ModifiersState::default(),
            last_pointer: (0.0, 0.0),
            dragging: None,
            title_dirty: true,
            impasto_smoke_done: false,
            substrate_smoke_done: false,
            line_smoke_done: false,
            sculpt3d_canvas_done: false,
            sculpt3d_bake_request: false,
            sculpt3d_alpha_request: false,
            sculpt3d_toggle_request: false,
            sculpt_doc: Vec::new(),
            #[cfg(feature = "sculpt3d")]
            sculpt3d_pending: None,
            mask_smoke_done: false,
            taper_smoke_done: false,
            wetpaint_smoke_done: false,
            stack_smoke_done: false,
            motion_path_smoke_done: false,
            timeline_onion_smoke_done: false,
            harmony_smoke_done: false,
            signal_smoke_done: false,
            ui_motion_smoke_done: false,
            timescale_smoke_done: false,
            stagger_smoke_done: false,
            buffer_smoke_done: false,
            extrap_smoke_done: false,
            expr_blend_smoke_done: false,
            morph_fade_smoke_done: false,
            nest_smoke_done: false,
            player_readout_log: None,
            physics_smoke_done: false,
            show_colliders: true,
            onion_ghosts: Vec::new(),
            interaction: ph2d_physics_ecs::InteractionSettings::default(),
            blast_flash: None,
            bake_channels: crate::render_loop::physics_bake::BakeChannels::default(),
            gilrs,
            audio,
            #[cfg(feature = "panel-audio-editor")]
            audio_sel_drag: None,
            #[cfg(feature = "panel-audio-editor")]
            audio_scrub_drag: false,
            input: InputState::new(),
            pan_anchor: None,
            held_button: None,
            eyedropper_dragging: false,
            last_cursor: (0.0, 0.0),
            hovered_files: Vec::new(),
            pending_drops: Vec::new(),
            frame_cpu_ms_ewma: 1.0, // optimistic baseline; reseeds on
            // the first frame's measurement
            pivot_content_center: None,
            frame_resize_start: None,
            instance_follow: None,
            rubber_band: None,
            pending_single_replace: None,
            group_drag_starts: Vec::new(),
            cycle_pick_world: None,
            cycle_pick_hits: Vec::new(),
            cycle_pick_idx: 0,
            cycle_pick_count: 0,
            last_bgremoval_pushed_entity: None,
            last_color_equalization_pushed_entity: None,
            color_equalization_previews: std::collections::BTreeMap::new(),
            last_upscale_pushed_entity: None,
            upscale_preview: None,
            bgremoval_preview: None,
            bgremoval_preview_gpu: None,
            last_painter_pushed_entity: None,
            pending_painter_move: None,
            input_events_this_frame: 0,
            paint_stamps_this_frame: 0,
            last_dispatch_us: 0,
            paint_stamp_us_this_frame: 0,
            last_paint_stamp_us: 0,
            last_paint_stamps: 0,
            paint_ms_ewma: 0.0,
            painter_preview: None,
            painter_preview_gpu: None,
            painter_shape_source_preview_gpu: None,
            painter_gpu_preview: None,
            painter_commit_requested: false,
            donated_form: crate::donated_form::DonatedForm::default(),
            painter_undo_requested: false,
            painter_redo_requested: false,
            // ADR-0108 cutover: the Vector drawing tool's shell-held Pen +
            // shape tool + undo history over `AppGfx.vec_scene`.
            vec_pen: ph2d_vec_edit::PenTool::new(),
            vec_shape: ph2d_vec_edit::ShapeTool::new(),
            vec_pencil: ph2d_vec_edit::Pencil::default(),
            vec_draw_config: ph2d_tool_vector::VectorDrawConfig::default(),
            vec_pencil_hand: crate::vec_pencil_input::PencilHand::default(),
            // ADR-0114 W2: estado de desenho do Flip (publicado pelo flip_bridge).
            flip_active: false,
            flip_style: None,
            ui_state_live: false,
            ui_preview: crate::render_loop::ui_preview::UiPreview::default(),
            ui_states_move_all: false,
            ui_states_anchor: None,
            ui_preview_leave: false,
            flip_draw: crate::flip_draw::FlipDraw::default(),
            flip_colorize: crate::flip_colorize::FlipColorize::default(),
            flip_gap: crate::flip_gap_live::GapHelpers::default(),
            pending_flip_colorize_apply: false,
            pending_flip_colorize_clear: false,
            flip_active_layer: None,
            flip_erasing: false,
            flip_strip: crate::flip_strip::FlipStrip::default(),
            flip_reshape: None,
            flip_edit_style: None,
            flip_segment_hover: None,
            flip_segment_hover_at: None,
            flip_edit_gesture: None,
            flip_trace_drag: None,
            flip_peek: None,
            player_keys: crate::player_input::PlayerKeys::default(),
            player_tape: ph2d_physics_ecs::InputTape::new(),
            discarded_run: ph2d_physics_ecs::InputTape::new(),
            flip_pose_drag: None,
            flip_selection_drag: None,
            field_gizmo_drag: None,
            flip_edit_domain: None,
            vec_marquee: None,
            vec_connect: None,
            vec_conn_handle: None,
            vec_blend: None,
            vec_restack: Vec::new(),
            vec_connect_pending: None,
            vec_cut_pending: None,
            vec_connect_sides: crate::connector_live::SideCache::new(),
            vec_blend_pending: None,
            vec_morph_pending: None,
            vec_envelope_drag: None,
            vec_textpath_handle_drag: false,
            motion_path_drag: None,
            motion_path_last_click: None,
            vec_patternpath_handle: None,
            vec_path_pick: None,
            vec_widget_applied: Default::default(),
            joint_body_pick: None,
            joint_clipboard: None,
            wheel_body_pick: None,
            wheel_rope_pick: None,
            joint_anchor_drag: None,
            joint_draw_armed: false,
            joint_draw: None,
            join_kind: 0, // Pin — the default joint kind for "Join Selected Bodies"
            vec_morph_plans: crate::morph_live::MorphPlans::new(),
            vec_blend_overlay: Vec::new(),
            vec_blend_spines: crate::blend_live::BlendSpines::new(),
            vec_blend_picks: Vec::new(),
            vec_label_pending: None,
            vec_label_poses: crate::label_live::LabelPoses::new(),
            offset_live: crate::offset_live::OffsetLive::default(),
            vec_live_drawn: ph2d_vec_render::LiveGeometry::new(),
            profile_live: crate::profile_live::ProfileLive::default(),
            contour_live: crate::contour_live::ContourLive::default(),
            instance_live: crate::instance_live::InstanceLive::default(),
            layout_live: crate::layout_live::LayoutLive::default(),
            vec_view_derived: ph2d_vec_scene::VecViewState::default(),
            align_live: crate::align_live::AlignLive::default(),
            bool_live: crate::bool_live::BoolLive::default(),
            symmetry_live: crate::symmetry_live::SymmetryLive::default(),
            vec_symmetry_origin: None,
            pattern_live: crate::pattern_live::PatternLive::default(),
            fx_live: crate::fx_live::FxLive::default(),
            fx_silhouette: crate::fx_silhouette::FxSilhouette::default(),
            vec_expand_knobs: (0, 2),
            vec_offset_mirrored: None,
            vec_profile_mirrored: None,
            vec_width_grab: None,
            vec_width_ref: None,
            vec_contour_mirrored: None,
            vec_history: ph2d_vec_edit::History::new(),
            undo: crate::undo::ProjectUndo::default(),
            undo_baseline: None,
            undo_request: None,
            undo_button: None,
            any_input_this_frame: false,
            vec_build: None,
            vec_grad_drag: None,
            vec_grad_selected: None,
            vec_clipboard: None,
            vec_pivot_edit: false,
            vec_snap: crate::vec_snap::VecSnapSettings::default(),
            vec_snap_targets: ph2d_vec_edit::SnapTargets::default(),
            vec_snap_guides: Vec::new(),
            guide_drag: None,
            vec_text_edit: None,
            vec_text_size: ph2d_tool_vector::params::DEFAULT_TEXT_SIZE,
            vec_text_weight: ph2d_tool_vector::params::DEFAULT_TEXT_WEIGHT as f32,
            vec_text_line_height: ph2d_tool_vector::params::DEFAULT_TEXT_LINE_HEIGHT,
            vec_text_tracking: ph2d_tool_vector::params::DEFAULT_TEXT_TRACKING,
            // ⚠️ **Auto é o default**, e é decisão de produto: um texto criado com a ferramenta
            // cresce com o que se digita (é o que todo editor faz num clique-e-digite). Uma
            // caixa nasce quando o artista a pede — e o gesto de ARRASTAR uma caixa ainda não
            // existe, então pedi-la é escolher `Fixed`.
            vec_text_wrap: None,
            vec_text_align: ph2d_tool_vector::TextAlign::Left,
            vec_text_extra_axes: vec_font::seed_extra_axes(None),
            vec_text_family: None,
            vec_last_canvas_click: None,
            vec_text_last_target: None,
            vec_shape_last_focus: None,
            vec_entities: Default::default(),
            flip_entities: Default::default(),
            vec_sel: Default::default(),
            frame_ms_ewma: 16.7, // ~60 Hz baseline so the first
                                 // frame's status bar doesn't display
                                 // a wild value while the EWMA seeds.
        }
    }

    /// Pump every queued gilrs event into the [`InputState`] and log
    /// the salient ones. Press / release / axis-change all logged at
    /// elapsed-ms timestamps so behavior is auditable from the
    /// terminal without an explicit debug overlay.
    fn pump_gamepad(&mut self) {
        let Some(g) = self.gilrs.as_mut() else {
            return;
        };
        // begin_frame snapshots last-frame held buttons so
        // pressed()/released() return correct edge-trigger values.
        self.input.begin_frame();
        while let Some(gilrs::Event { event, time: _, .. }) = g.next_event() {
            match event {
                gilrs::EventType::Connected => {
                    println!("[{:>6}ms] gamepad connected", self.handler.elapsed_ms());
                }
                gilrs::EventType::Disconnected => {
                    println!("[{:>6}ms] gamepad disconnected", self.handler.elapsed_ms());
                }
                _ => {
                    if let Some(translated) = gilrs_adapter::translate(event) {
                        self.input.apply_event(translated);
                        log_input_event(self.handler.elapsed_ms(), &translated);
                    }
                }
            }
        }
    }

    fn convert_modifiers(state: ModifiersState) -> Modifiers {
        Modifiers {
            shift: state.shift_key(),
            ctrl: state.control_key(),
            alt: state.alt_key(),
            meta: state.super_key(),
        }
    }

    fn timestamp_ns() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    }

    /// Snapshot `InputState` into the ScriptHost's `ph2d.input` table
    /// so Luau can read held buttons / axis values via the canonical
    /// `gamepad.held.<button>` and `gamepad.axis.<axis>` keys.
    /// Cleared and rebuilt every frame — keys absent from this frame
    /// resolve to `nil` on the Luau side (per the M8 ph2d_input
    /// resolves test).
    fn push_input_to_script(&self) {
        let Some(gfx) = self.gfx.as_ref() else {
            return;
        };
        let Some(host) = gfx.script.as_ref() else {
            return;
        };
        host.clear_input();
        for button in self.input.gamepad.iter_held() {
            let key = format!("gamepad.held.{}", button.as_lua_key());
            host.provide_input(&key, 1.0);
        }
        for (axis, value) in self.input.gamepad.iter_axes() {
            let key = format!("gamepad.axis.{}", axis.as_lua_key());
            host.provide_input(&key, value as f64);
        }
    }

    /// Per-frame render orchestration — body lifted to
    /// [`crate::render_loop`] (Wave 3.1 stage C). See its module docs
    /// for the rationale + the split-impl pattern.
    fn render_frame(&mut self) {
        // A janela já está fechando e a GPU já foi derrubada por `on_close_request` — winit pode
        // entregar um `RedrawRequested` atrasado na mesma iteração, e desenhá-lo seria pedir um frame
        // a um dispositivo que não existe mais.
        if self.exiting {
            return;
        }
        // §4.C — o PEDAÇO sob o cursor no modo Segment (hover). ANTES do render: o overlay
        // o lê no mesmo frame. Barato e guardado (só recomputa quando o cursor move).
        self.flip_segment_hover_refresh();
        // A escolha da paleta de comandos GLOBAL, executada ANTES do render do frame — assim o
        // comando (pegar uma ferramenta, mostrar um painel) já está reflectido no que este frame
        // desenha, em vez de aparecer um quadro depois.
        self.global_palette_drain();
        self.run_render_frame();
        // **A SELEÇÃO** (`flip_select`, W6): no modo Edit ela é o alvo dos ajustes do
        // painel. Só a MUDANÇA de estilo age.
        crate::flip_select::flip_edit_style_refresh(self);
        // **O DOMÍNIO da seleção** (W8): a troca Stroke↔Point converte a seleção no
        // documento (broadcast/promoção) — uma vez, quando o toggle muda.
        crate::flip_select::flip_edit_domain_refresh(self);
        // Depois do frame (estado já reconciliado pelo `sync`, `self` livre do borrow
        // do render loop): drena um Ctrl+Z/Y pendente e registra a ação do frame na
        // fila de undo global, por diff de estado (ver `undo::post_frame_undo`).
        self.post_frame_undo();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // PR 9c of the convention-by-discovery migration: subsystem
        // boot pipeline lives in `init::build_initial_state`. This
        // method now only wires the produced state into `self` and
        // fires lifecycle hooks. See
        // `docs/Migracao/2026-05-convention-by-discovery.md`.
        let (window, host, gfx) = init::build_initial_state(&self.handler, event_loop);
        let size = gfx.surface.size();
        let scale = host.scale_factor();
        self.window = Some(window);
        self.host = Some(host);
        self.gfx = Some(gfx);
        self.handler.on_lifecycle(Lifecycle::Foreground);
        self.handler.on_resize(size, scale);
        self.title_dirty = true;
        if let Some(host_ref) = self.host.as_ref() {
            host_ref.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // PR 9b of the convention-by-discovery migration: per-arm
        // handlers live in `input_dispatch::App::on_<arm>`. This
        // method is a pure dispatch table — adding a new arm is one
        // line here + one method there. See
        // `docs/Migracao/2026-05-convention-by-discovery.md`.
        match event {
            WindowEvent::CloseRequested => self.on_close_request(event_loop),
            WindowEvent::Resized(size) => self.on_resized(size),
            WindowEvent::HoveredFile(path) => self.on_hovered_file(path),
            WindowEvent::HoveredFileCancelled => self.on_hovered_file_cancelled(),
            WindowEvent::DroppedFile(path) => self.on_dropped_file(path),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.on_scale_factor_changed(scale_factor)
            }
            // Perder o foco SOLTA as teclas de caminhada: o `Up` de uma tecla
            // presa nunca chega quando a janela vai embora, e sem isto o player
            // anda sozinho até alguém tocá-la de novo (W3).
            WindowEvent::Focused(false) => self.player_keys.release_all(),
            WindowEvent::ModifiersChanged(mods) => self.on_modifiers_changed(mods),
            WindowEvent::Ime(winit::event::Ime::Commit(text)) => self.on_ime_commit(text),
            WindowEvent::CursorMoved { position, .. } => self.on_cursor_moved(position),
            WindowEvent::MouseWheel { delta, .. } => self.on_mouse_wheel(delta),
            WindowEvent::MouseInput { state, button, .. } => self.on_mouse_input(state, button),
            WindowEvent::KeyboardInput { event, .. } => self.on_keyboard_input(event),
            WindowEvent::RedrawRequested => {
                self.render_frame();
                self.exit_after_frames_tick(event_loop);
            }
            _ => {}
        }
    }
}

/// Floor for `pixels_per_meter` used inside the import math; below
/// this a single sprite would span kilometers and break camera math.
/// The UI clamps to a higher floor (`MIN_PIXELS_PER_METER = 1.0` in
/// `ph2d_editor::project`) but defense-in-depth here keeps the shell
/// safe even if a future config path skips that clamp.
pub(crate) const EPS_PIXELS_PER_METER: f32 = 0.01;

/// When the image-edit undo slot is being overwritten by a new edit,
/// release every pre-edit Individual texture across the previous
/// transaction's entries (multi-sprite Apply leaves N entries; the
/// single-sprite case degenerates to N=1). Atlas-backed pre-sources
/// don't need release — they share the texture via the asset_db.
/// No-op when the slot is empty.
pub(crate) fn drop_undo_pre_sources_if_individual(
    renderer: &mut SpriteRenderer,
    slot: &mut Option<ImageEditTransaction>,
) {
    if let Some(prev) = slot.take() {
        for entry in prev.entries {
            if let ph2d_render::SpriteSource::Individual { texture_id } = entry.pre_source {
                renderer.individual_mut().release(texture_id);
            }
        }
    }
}

/// Commit `entries` (one per sprite the multi-sprite Apply touched) as
/// the new undo transaction, releasing the previous transaction's
/// pre-edit individual textures. No-op when `entries.is_empty()` (no
/// sprite actually changed → nothing to undo). The transaction label
/// comes from the first entry; per-drain code pushes the same label on
/// every entry it appends, so all N entries agree by construction.
pub(crate) fn commit_image_edit_transaction(
    renderer: &mut SpriteRenderer,
    slot: &mut Option<ImageEditTransaction>,
    entries: Vec<ImageEditSnapshot>,
) {
    if entries.is_empty() {
        return;
    }
    let label = entries[0].label;
    drop_undo_pre_sources_if_individual(renderer, slot);
    *slot = Some(ImageEditTransaction { entries, label });
}

/// Floor for the world-space side length of an imported sprite.
/// Guarantees the quad is selectable even if the user picks an
/// absurd `pixels_per_meter` value combined with a 1-pixel image.
pub(crate) const MIN_SPRITE_SIZE: f32 = 0.001;

/// Query the live cursor position relative to `window` in physical
/// pixels (top-left origin). Returns `None` if the platform path
/// fails; callers fall back to a cached value.
///
/// Existence rationale: winit 0.30 on macOS does not emit
/// `CursorMoved` during external file drag operations, so by the
/// time `DroppedFile` fires the cached cursor is stale. We bypass
/// the event stream by asking CoreGraphics for the live cursor
/// directly. Other platforms reach here only as a no-op stub.
/// Resolve the editor theme from a name (typically read from the
/// `PH2D_THEME` env var), falling back to [`Theme::Forge`] for
/// missing/invalid values. Recognised names match `Theme::id()`
/// (`forge`, `workshop`, `sunstone`, `blueprint`).
fn main() {
    install_panic_hook();
    // **A math dos tokens de design** (plano UI/UX W4c.3) — uma linha, e é a única.
    //
    // ⚠️ Ela vive AQUI e não dentro da `ph2d-tokens` por uma aresta MEDIDA: o parser arrasta
    // `ph2d-expr` → `ph2d-nodegraph`, e a `ph2d-tokens` é a folha de que 44 widgets dependem — um
    // botão de ícone passaria a compilar o motor de cozimento para saber de que cor é.
    //
    // ⚠️ E o modo de falha de a ESQUECER é o certo: o app compila, corre, e o painel simplesmente
    // não OFERECE o botão de fórmula (`math_available()` é falso) — em vez de o oferecer e não
    // fazer nada. É o padrão do `set_ml_available` do AI Denoise.
    //
    // ⚠️ Antes do `App::new()`, e na thread do laço de eventos: o host mora num `thread_local`, e
    // quem pergunta por ele é o painel a pintar.
    ph2d_token_math::install();
    let event_loop = EventLoop::new().expect("create EventLoop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    println!("PH2D desktop shell starting (close window or Cmd+Q to exit)…");
    event_loop.run_app(&mut app).expect("event loop crashed");
    println!("PH2D desktop shell exited cleanly.");
}

#[cfg(test)]
mod theme_env_tests {
    use crate::theme::resolve_theme;
    use ph2d_tokens::Theme;

    #[test]
    fn unset_defaults_to_forge() {
        assert_eq!(resolve_theme(None), Theme::Forge);
    }

    #[test]
    fn known_names_resolve() {
        assert_eq!(resolve_theme(Some("workshop")), Theme::Workshop);
        assert_eq!(resolve_theme(Some("sunstone")), Theme::Sunstone);
        assert_eq!(resolve_theme(Some("blueprint")), Theme::Blueprint);
        assert_eq!(resolve_theme(Some("forge")), Theme::Forge);
    }

    #[test]
    fn unknown_falls_back_to_default() {
        assert_eq!(resolve_theme(Some("dracula")), Theme::Forge);
    }
}
