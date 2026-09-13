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
// Tecto de LOC NUMERADO em `tests/it/file_loc_caps.rs` — frame orchestrator — heavy phases already extracted to
// siblings (present/image_edit/sim_extract/snapshots/bgremoval_preview/
// hierarchy); residual is the frame skeleton + EditorAction intent drain.
// FOLLOW-UP: extract the intent-drain match to a `intents.rs` sibling to
// drop back under the cap (2026-05-21: +SetPresentMode/RealSize tipped it).

#[cfg(feature = "panel-audio-editor")]
mod audio_overlay;
mod audio_pieces;
mod audio_spectrogram;
/// **O dreno do painel autorado** — um braço por variante do `AuthoredIntent`, e nenhum curinga.
/// Irmão porque o `FOLLOW-UP` do topo deste arquivo já o pedia, e porque o `if let` que ele
/// substitui matava **seis famílias de widget** de uma vez (2026-08-30).
pub(crate) mod authored_intents;
pub(crate) mod autokey_pass;
pub(crate) mod bgremoval_preview;
mod color_equalization_bridge;
mod cooked_texture_bridge;
/// **A decoração da folha** — a faixa hachurada e o nome, em pixels de TELA. Chrome, nunca estilo
/// do documento: ela diz o que a folha É, e o que se assa são os filhos.
mod demo_legend;
mod equalize_sizes_bridge;
/// O anel do cursor do pincel do FLIP (ADR-0114 W5, smoke do Enio): o Size é absoluto
/// em px de tela, então o anel é px de tela — sem conversão de câmera.
mod gizmo_prune;
/// **O número do arrasto de gizmo** — quem o publica (a lei mora no `editor-core`).
mod gizmo_readout;
mod hierarchy;
mod hierarchy_add_root;
/// ⭐⭐ **O menu de um cartão da biblioteca, e a poda de selecção morta** — irmão por assunto do
/// [`hierarchy`], ver o cabeçalho de lá.
mod hierarchy_asset_verbs;
/// ⭐⭐ **O gesto de APAGAR e as três respostas dele** — irmão por assunto do `hierarchy`.
mod hierarchy_delete;
mod hierarchy_rename;
// ⚠️ **A row *Duplicate*, por ASSUNTO** — o `hierarchy.rs` voltou ao tecto de 600 LOC quando a
// cópia ganhou as duas leis que lhe faltavam (auditoria §1.4/§1.2). Lá o dreno das intenções, aqui
// o que duplicar quer dizer.
mod hierarchy_duplicate;
mod image_edit;
mod inspector_commits;
#[cfg(test)]
mod inspector_commits_tests;
/// W-PartFace: o que o §11 responde sobre uma PEÇA (um filho com `Collider` e
/// sem `RigidBody`) — a volta que a W-Compound não deu.
#[cfg(test)]
mod inspector_part_tests;
/// **A conversão entre estratégias de origem** (Render Source → Strategy) — irmão do
/// `inspector_commits`, e o corte que o marcador de exceção de LOC daquele arquivo pedia.
mod inspector_strategy;
mod sheet_overlay;

// ⚠️ `pub(crate)`: a porta `apply_physics_edit` é a ÚNICA regra de "como uma
// entidade vira corpo" (o collider sai da CAIXA DO SPRITE), e o gerador de rig
// (`ph2d_app_physics::joint_rig`, W-Rig) a chama de fora — uma segunda regra lá faria um rig
// cujos colliders discordam dos que o botão *Add Body* produz. Mesmo alcance do
// `inspector_joint` logo acima, pelo mesmo motivo.
/// A lei do relógio, perguntada pelos DOIS emissores de sinal — ver o módulo.
mod clock_forward;
/// ⭐ **A sonda de PRESENÇA** das seções do Inspector — a lei da F3 é uma varredura só, e ela
/// precisa de perguntar aos oito builders pela mesma porta. Ver [`crate::inspector_presence_tests`].
#[cfg(test)]
pub(crate) mod inspector_presence_probe;
// ⛔ **Duas re-exportações de `ph2d_app_physics` viviam aqui** (`seed_attached_player` ·
// `seed_attached_collider`) e o comentário delas dizia-o: *«esta linha é só o ENDEREÇO por onde a
// tabela de seeds lhes chega»*. O único consumidor era a tabela `SEEDS` do `component_seed`, que
// em 2026-09-12 passou a nomear as duas na crate irmã. ⇒ o `render_loop` era, neste ponto, uma
// FACHADA — *um módulo da shell que só re-exporta uma crate é uma CRATE a usar o nome da shell*
// (o achado da Fase C da `line/app-vec`), e a régua do fecho conta-a como shell.
mod inspector_visibility;
/// MEASUREMENT scaffold: onde as fases PANEL e CHROME do `painter_bridge::dispatch` gastam um frame.
#[cfg(test)]
mod measure_bridge_phases;
mod padding_bridge;
/// Render-and-look probe for the Push phase (diagnostic, `#[ignore]`d — writes lit PNGs).
#[cfg(test)]
mod push_look_probe;
pub(crate) mod record_fit;
pub(crate) mod timeline_bridge;
/// **A AUTORIA de uma chave** — irmão do `timeline_bridge` por teto de LOC (HR-18).
mod timeline_bridge_keys;
pub(crate) mod timeline_onion;
mod timeline_presets;
/// **A ponte do painel de TOKENS** (plano UI/UX W6) — o read-back do picker e os intents de
/// Reset. A shell é o único escritor da camada de override de cor.
pub(crate) mod tokens_bridge;
pub(crate) mod tokens_bridge_dtcg;
#[cfg(test)]
mod wet_brush_look_probe; // render-and-look do pincel GRANDE do wet paint
/// Render-and-look da razão da grade do fluido (diagnóstica, `#[ignore]`d).
#[cfg(test)]
mod wet_grid_look_probe;
// `pub(crate)`: `apply_layer_reparent` is called from `input_dispatch` (outside
// render_loop) to route the W3.T3.8 layer drag-reparent through the allowlisted
// bridge-queries module instead of downcasting in central dispatch.
/// **§5 9-Slice** — snapshot e commit da seção. Irmão do `inspector_ordering`.
/// **Os marcadores das âncoras no canvas** (spec Sprite 07 §7.6) — sem eles a §12 é um
/// formulário que não mexe em nada na tela.
pub(crate) mod anchor_gizmo;
mod anchor_overlay;
/// ⭐⭐⭐ A ponte do `SignalActions` (TOP-20 #5) — onde um sinal vira jogo.
/// ⭐⭐⭐ A ponte do SOM DE CENA (TOP-20 #4) — onde um objecto deixa de ser mudo.
mod audio_2d;
/// ⭐⭐⭐ **A CÂMERA DE JOGO** (TOP-20 #7) — a costura entre a lei pura e a vista da shell.
mod camera_2d;
/// O anel de um objeto VAZIO selecionado — ver o módulo.
mod empty_object_overlay;
/// ⭐ A secção TIMERS (TOP-20 #2, W3) — o snapshot e o commit dela.
/// ⭐ A secção SIGNAL ACTIONS (TOP-20 #5, W3) — o snapshot e o commit dela.
mod inspector_action;
/// **§12 Sockets / Named Anchors** (ADR-0072) — snapshot e commit.
mod inspector_anchor;
mod inspector_anim;
/// ⭐⭐⭐ A secção AUDIO do Inspector (TOP-20 #4, W3) — o snapshot e o commit.
mod inspector_audio;
/// ⭐⭐⭐ **A secção CAMERA** (TOP-20 #7) — o snapshot e o commit dos três componentes.
mod inspector_camera;
mod inspector_commits_sprite;
/// ⭐ **A seção COMPONENT do Inspector** (ADR-0164 / F5) — o que esta cópia tem de diferente
/// da receita, e o gesto que limpa as excepções sem alvo.
pub(crate) mod inspector_instance;
mod inspector_properties;
mod inspector_slice;
mod inspector_timer;
// ⭐ A derivação do `MasterPiece` (`master_editing`, F4.6) mudou-se para a
// `ph2d_app_components` em 2026-09-12: é **lei da família das instâncias**, não do laço. O gate do
// anel de objecto vazio (`group_gizmo_view_tests`) continua a acender a receita pela porta de
// VERDADE, hoje escrita `ph2d_app_components::master_editing::mark`.
pub(crate) use audio_2d::AudioSceneReport;
pub(crate) use camera_2d::CameraSceneReport;
mod signal_actions;
/// ⚠️ A MESMA porta do passe, alcançável dos gates de outro módulo (a cadeia de visibilidade do
/// vetor lê a marca, e o gate dela tem de a poder carimbar). *Um segundo carimbo escrito à mão no
/// teste seria a segunda resposta.*
#[cfg(test)]
pub(crate) fn master_editing_mark_for_tests(
    sim: &mut ph2d_ecs::SimWorld,
    selection: Option<u64>,
) -> bool {
    ph2d_app_components::master_editing::mark(sim, selection, &mut None).touched
}
/// The joint-anchor point gizmo's publish rule — extracted from `snapshots` so
/// "which entity gets a point handle" is gated headless.
// ⭐ O `point_gizmo` MUDOU-SE para [`ph2d_app_physics::overlay::point_gizmo`] (W2/L2 Fase C):
// ele tinha nome genérico e era 100% física — os `use` dele eram
// `ph2d_physics_ecs::{JointSide, PhysicsBridge}` e o `joint_glyphs` da própria crate, e as
// seis funções são junta, corda, roldana e âncora. O laço continua a chamá-lo PELO NOME,
// que é o que o HOWTO §4 manda: o que sai são os CORPOS.
use ph2d_app_physics::overlay::point_gizmo;
mod present;
/// ⭐⭐⭐ **As faixas de desenho** (ADR-0154 Fase 2) — irmão por assunto do [`present`].
mod present_bands;
/// ⭐⭐⭐ O VIDRO JATEADO por trás da receita aberta (o *Edit Prefab*).
mod present_frost;
/// ⭐⭐ Os passes de LUZ do quadro (a sprite emissiva e o glow do Motion).
mod present_fx;
/// ⚠️ `pub(crate)`: os gates do `preview_drive` correm o tique de fora do `render_loop` —
/// é ele o motor que declara a §11 como pré-visualização, e um gate que o encenasse à mão mediria
/// a encenação.
pub(crate) mod sprite_anim_tick;
/// ⭐⭐⭐ **A ponte do `Timer`** (TOP-20 #2) — o tique no passo fixo e o sinal que sai dele.
mod timer_tick;
pub(crate) use sprite_anim_tick::start_autoplay_animations;
pub(crate) use timer_tick::start_autostart_timers;
/// Fase do quadro: as cenas de smoke que pedem a `App` inteira (1.ª metade).
mod fase_app_scene_smokes;
/// Fase do quadro: as cenas de smoke que pedem a `App` inteira (2.ª metade).
mod fase_app_scene_smokes_late;
/// Fase do quadro: as cenas de smoke que precisam do atlas, 1.ª metade (impasto, substrato, LINE).
mod fase_atlas_scene_smokes;
/// Fase do quadro: as cenas de smoke que precisam do atlas, 2.ª metade (máscara, folha, Motion, lente).
mod fase_atlas_scene_smokes_late;
/// Fase do quadro: os painéis de áudio (mixer + editor) ouvidos pelo motor.
mod fase_audio_panels;
/// Fase do quadro: o relógio do chrome (`wall_dt`, `ui_dt` e os tiques que andam nele).
mod fase_chrome_clock;
/// Fase do quadro: os insumos do extract (passo, pré-visualizações, folha aberta, px/m, filtro).
mod fase_extract_inputs;
/// Fase do quadro: os relógios do passo fixo (sim, cabeças de leitura, §11 Animation, timers).
mod fase_fixed_step_clocks;
/// Fase do quadro: o perfilador (conta os quadros e chama o relatório a cada 120).
mod fase_frame_profile;
/// Fase do quadro: o relatório do perfilador (a partição do quadro a cada 120 quadros).
mod fase_frame_profile_report;
/// Fase do quadro: a câmera de jogo (o herói da cena de smoke e o passe da câmera).
mod fase_game_camera;
/// Fase do quadro: o fim do ramo hero (toasts, barras de trabalho, a arena do quadro).
mod fase_hero_chrome_tail;
/// Fase do quadro: o dreno de edicao de imagem e os desmontes do Apply.
mod fase_image_edit_apply;
/// Fase do quadro: a entrada (carimbo coalescido, diagnóstico, gamepad, script, soltos).
mod fase_input_and_drops;
/// Fase do quadro: o chrome legado (o ramo sem `HeroScreen`).
mod fase_legacy_chrome;
/// Fase do quadro: o modal de imagem nova (Cmd/Ctrl+N) cria a tela escolhida.
mod fase_new_image_modal;
/// Fase do quadro: a receita aberta (a marca, o pedido de palco, a trava e o pedido do Cancel).
mod fase_open_recipe;
/// Fase do quadro: as cenas do pincel do Painter (taper, tinta molhada).
mod fase_painter_brush_smokes;
/// Fase do quadro: o passo da física (dispatch, flash, readout do player, juntas que cederam).
mod fase_physics_step;
/// Fase do quadro: o que está sob o cursor (a 1.ª do `run_render_frame`).
mod fase_pointer_subjects;
/// Fase do quadro: a re-acendida dos objetos assados — FORA da feature `sculpt3d`, de propósito.
mod fase_relight_baked_forms;
/// Fase do quadro: o som de cena (as vozes dos objectos).
mod fase_scene_audio;
/// Fase do quadro: o passo do GC do Luau (M7).
mod fase_script_gc;
/// Fase do quadro: o objeto misto do sculpt3d (bake, alpha por imagem, luz a re-autorar).
#[cfg(feature = "sculpt3d")]
mod fase_sculpt3d_bake;
/// Fase do quadro: a cena da doação do sculpt3d (`PH2D_SCULPT3D_SMOKE=2`).
#[cfg(feature = "sculpt3d")]
mod fase_sculpt3d_donation_smoke;
/// Fase do quadro: o pré-quadro do sculpt3d (Grab, pendente, pill, doação, Hierarquia).
mod fase_sculpt3d_pre_frame;
/// Fase do quadro: a manutenção de sessão (Shape Builder, tween, Colorize, Gap Closure).
mod fase_session_upkeep;
/// Fase do quadro: o outbox de sinais (os produtores que faltavam e o dreno).
mod fase_signal_outbox;
/// Fase do quadro: o extract (propagação, emissão das sprites e a ordem total do quadro).
mod fase_sim_extract;
/// Fase do quadro: as cenas do Sprite Inspector (9-slice, âncoras, montagem, Animation).
mod fase_sprite_inspector_smokes;
/// Fase do quadro: as cenas dos pixels da sprite (`.ase`, dither, emissiva).
mod fase_sprite_pixel_smokes;
/// Fase do quadro: o resize coalescido e o modo de apresentação do arrasto.
mod fase_surface_resize;
/// Fase do quadro: o relógio dos contêineres da timeline.
mod fase_timeline_containers;
/// Fase do quadro: o dreno da timeline (intents, aplicação, reset, valores das faixas).
mod fase_timeline_drain;
/// Fase do quadro: a vista da timeline (amostragem, intents estacionadas, espelhos do painel).
mod fase_timeline_view;
/// Fase do quadro: a poeira de impacto (as faíscas por cima do chrome).
mod fase_ui_burst_paint;
/// O empréstimo do `gfx` do quadro: o destructure exaustivo do `AppGfx`, re-derivado por fase.
mod frame_gfx;
/// **Os nove quads do 9-slice** — irmão do `sim_extract`, que está no tecto de LOC.
pub(crate) mod sheet_grid_overlay;
pub(crate) mod sim_extract;
mod sim_extract_sheet;
mod sim_extract_slice;
mod snapshots;
use frame_gfx::FrameGfx;
/// A sprite como FONTE DE LUZ (plano `docs/Sprite_projeto/18` W8) — lê o espelho pelo `SimRef` e
/// devolve as instâncias que emitem. ⚠️ Irmão do `sim_extract` de propósito: ele está no tecto de LOC.
pub(crate) mod sprite_emissive;
mod upscale_bridge;

// ADR-0108 cutover: the single Vector-tool bridge (style sync + recolour).
// Rendering of `AppGfx.vec_scene` stays inline below (ph2d_vec_render).
// pub(crate): `set_mode` é chamado do `vec_text` (o `T` troca o modo pela allowlist
// de downcast deste bridge).
/// **A ponte dos ESTADOS de UI** (plano UI/UX W7) — quem faz a cena ANDAR entre duas poses.
pub(crate) mod ui_preview;
// ⭐ **A LEI dos estados de UI mudou-se para [`ph2d_app_vec::ui_state_bridge`]** (W2 Fase D): ela
// é pura sobre o ECS (zero `App`, zero `gfx`) e a única coisa da shell que ela nomeava era o
// `vec_ui_state_edit`, que foi com ela. ⛔ **O LAÇO e os CAMPOS ficam**: o `app_state` guarda as
// `UiMachines` e o `Cooked`, e é este módulo que decide quando `request`/`dispatch` correm.
// *O que sai são os CORPOS; o que decide a ordem do quadro fica* (HOWTO §4).
/// O NÚMERO do smart guide — a ficha de distância; veja os docs do módulo.
// ⭐ W2/L4: foi para `crates/ph2d-app-vec`. Re-exportado para `render_loop::vec_snap_labels`
// continuar a resolver nos chamadores deste módulo.
use ph2d_app_vec::snap_labels as vec_snap_labels;
pub(crate) use ph2d_app_vec::ui_state_bridge;
// ⭐ **A ponte documento ⇄ cena vectorial mudou-se para [`ph2d_app_vec::vector_bridge`]**
// (W2 Fase D): os seis ficheiros dela são PUROS, e o que os prendia aqui era uma FACHADA —
// `crate::vec_snap::VecSnapSettings` é `pub(crate) use ph2d_app_vec::snap::{…}`, um tipo da
// crate a usar o nome da shell. ⛔ O LAÇO fica: é este módulo que decide quando ela corre.
pub(crate) use ph2d_app_vec::vector_bridge;

use crate::*;

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::paint::PaintCtx;
use ph2d_editor_core::zones::Rect as EditorRect;
use ph2d_editor_core::{Layout as EditorLayout, RequestedSpriteStrategy, Toast, paint_hero_screen};
use std::time::Instant;

thread_local! {
    /// Frame-phase profiler (`PH2D_FLUID_PROFILE` **ou** `PH2D_PAINT_PERF`):
    /// `-1` = unread, else cached on/off. Splits the frame into CPU-encode (raw)
    /// vs the present/acquire stall, plus the painter bridge dispatch (CPU preview)
    /// — to pin a slowdown the `[fluid]` profiler proves is OUTSIDE the fluid drive.
    ///
    /// ⚠️ **Por que o `PH2D_PAINT_PERF` liga este bloco também** (2026-08-03): o
    /// carimbo roda no flush coalescido, na linha ~698, **ANTES** do `cpu_start`
    /// — ou seja fora da janela de encode e fora do `painter-dispatch`. O
    /// `[paint-perf]` é, por construção, **CEGO ao carimbo**: ele reporta
    /// `dispatch p50=0.0` tanto num traço de graça quanto num que custa 300 ms.
    /// A linha `stamps:` (e o `deposito:` ao lado dela) vive só aqui, então um
    /// smoke rodado com o flag que NOMEIA performance de pintura media tudo
    /// menos a pintura e voltava tranquilizando. Um instrumento silencioso é
    /// pior que um ausente.
    static FRAME_PROF_ON: std::cell::Cell<i8> = const { std::cell::Cell::new(-1) };
    static FRAME_PROF_N: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    /// ⚠️ **O RELÓGIO da janela de diagnóstico, MEDIDO.** Ele existe porque a
    /// versão anterior a ASSUMIA (`span = frame_medio × 120`), e os contadores
    /// do worker (`wet_diag`) acumulam em tempo REAL enquanto este bloco só
    /// conta os frames em que ele roda — as duas janelas divergem, e a
    /// divergência aparecia como uma partição impossível: o log do Enio de
    /// 2026-07-31 trouxe `busy 69% away 31% sleep 909%` (soma 1009%) e uma
    /// `TAXA DA AGUA 392,8 Hz` contra os **40 Hz nominais da SPEC**, ou seja um
    /// solver dez vezes fora do ritmo — que **não existia**. Três baldes que
    /// dizem partição TÊM de dividir uma janela medida, senão o instrumento
    /// manda a próxima pessoa caçar uma sim desgovernada que não está lá.
    static FRAME_PROF_SINCE: std::cell::RefCell<Option<std::time::Instant>> =
        const { std::cell::RefCell::new(None) };
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
    /// Quantas ENTREGAS de ponteiro compõem a soma acima — o divisor sem o qual
    /// `stamps: media 105,82ms` não distingue *um re-stamp de forma inteira* de
    /// *cinquenta eventos incrementais*, que pedem curas opostas.
    static FRAME_PROF_STAMP_EV: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    /// **O DIVISOR DO `painter-dispatch`** — quantos PIXELS o dreno do preview
    /// publicou na janela, e em quantos quadros.
    ///
    /// ⚠️ A mesma doença que o `FRAME_PROF_STAMP_EV` curou no carimbo, um
    /// sistema adiante: `painter-dispatch=11,80ms` **sem carimbo nenhum** não
    /// distingue *um retângulo grande uma vez* de *um retângulo pequeno sempre*,
    /// e o custo é dominado por gather + premultiply + upload da área
    /// publicada. Medido headless (doc 28 §5.53): com a água correndo o dreno
    /// publica **8,26 M px por quadro** numa tela de 16,8 M — metade dela — para
    /// **2,07 M células** de água viva, ou seja o retângulo pede **3,99×**.
    static FRAME_PROF_PREVIEW_PX: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static FRAME_PROF_PREVIEW_N: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    /// A espera MEDIDA do `acquire_frame` (ver [`note_acquire_wait`]).
    static FRAME_PROF_ACQUIRE_US: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static FRAME_PROF_ACQUIRE_N: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    /// `paint_hero_screen` µs (panel/chrome Vello encode — includes the Paper preview).
    static FRAME_PROF_HERO_US: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// Quanto o `acquire_frame` de fato BLOQUEOU neste quadro.
///
/// ⚠️ Sem isto a linha `[frame]` publicava `present/acquire-stall` como
/// `total - encode`, e o encode só começa em `cpu_start` — **depois** do
/// `tool-tick` e do flush de carimbo. O residuo se lia como *espera de GPU*
/// enquanto continha trabalho de CPU: medido, `tick 3,31` de um "stall" de
/// 7,91. Um numero derivado por subtração absorve tudo que ninguem mediu.
pub(crate) fn note_acquire_wait(d: std::time::Duration) {
    if frame_prof_on() {
        FRAME_PROF_ACQUIRE_US.with(|c| c.set(c.get() + d.as_micros() as u64));
        FRAME_PROF_ACQUIRE_N.with(|c| c.set(c.get() + 1));
    }
}

/// O dreno do preview publicou `px` pixels neste quadro — o divisor do
/// `painter-dispatch`. Chamado de [`painter_bridge`], no sítio onde a bbox é
/// resolvida; no-op sem o `PH2D_FLUID_PROFILE`.
pub(crate) fn note_preview_px(px: u64) {
    if frame_prof_on() {
        FRAME_PROF_PREVIEW_PX.with(|c| c.set(c.get() + px));
        FRAME_PROF_PREVIEW_N.with(|c| c.set(c.get() + 1));
    }
}

/// Quiet frames after the last resize event before the present mode saved by the fluid-drag override is
/// restored (~0.5s at 60Hz) — long enough that a paused-then-resumed drag doesn't thrash reconfigures.
const RESIZE_SETTLE_FRAMES: u32 = 30;

fn frame_prof_on() -> bool {
    FRAME_PROF_ON.with(|c| {
        if c.get() < 0 {
            // Os DOIS flags acendem esta partição — ver o porquê no doc do `FRAME_PROF_ON`:
            // a linha `stamps:`/`deposito:` mora só aqui, e quem mede pintura pede o
            // `PH2D_PAINT_PERF`.
            let on = ["PH2D_FLUID_PROFILE", "PH2D_PAINT_PERF"]
                .iter()
                .any(|k| std::env::var(k).is_ok_and(|v| v != "0"));
            c.set(i8::from(on));
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
            ph2d_app_painter::paint_perf::end_frame(t0.elapsed().as_secs_f64() as f32 * 1e3);
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
        let player_input = self.fase_pointer_subjects();
        // PH2D_PAINT_PERF: whole-frame timer (aggregated on scope exit, paired with the dispatch info).
        let _paint_frame_timer =
            PaintFrameTimer(ph2d_app_painter::paint_perf::on().then(std::time::Instant::now));
        self.fase_audio_panels();
        let (cpu_start, diag_input_events, diag_paint_stamps) = self.fase_input_and_drops();

        self.fase_app_scene_smokes();
        self.fase_sculpt3d_pre_frame();
        self.fase_app_scene_smokes_late();
        self.fase_session_upkeep();

        let Some(wall_dt) = self.fase_chrome_clock() else {
            return;
        };
        self.fase_atlas_scene_smokes();
        #[cfg(feature = "sculpt3d")]
        self.fase_sculpt3d_donation_smoke();
        #[cfg(feature = "sculpt3d")]
        self.fase_sculpt3d_bake();
        self.fase_relight_baked_forms();
        self.fase_atlas_scene_smokes_late();
        self.fase_sprite_inspector_smokes();
        self.fase_sprite_pixel_smokes();
        self.fase_painter_brush_smokes();
        self.fase_new_image_modal();
        self.fase_script_gc();
        self.fase_surface_resize();
        let Some(fase_fixed_step_clocks::FrameClocks {
            report,
            tool_preview_bits,
            anim_signals,
            timer_signals,
        }) = self.fase_fixed_step_clocks(wall_dt)
        else {
            return;
        };
        self.fase_scene_audio();
        self.fase_game_camera(player_input, report);
        let Some(fase_extract_inputs::ExtractInputs {
            dt,
            preview_overrides,
            sheet_preview,
            ppm,
            default_filter,
        }) = self.fase_extract_inputs(wall_dt)
        else {
            return;
        };
        let Some(fase_timeline_view::TimelineView {
            container,
            dragging_entity,
            keys_mode,
            selected_now,
        }) = self.fase_timeline_view()
        else {
            return;
        };
        self.fase_timeline_containers(container, keys_mode);
        self.fase_timeline_drain(container, dragging_entity, keys_mode, selected_now);
        self.fase_physics_step(player_input);
        self.fase_signal_outbox(anim_signals, timer_signals);
        self.fase_open_recipe();
        self.fase_sim_extract(dt, preview_overrides, sheet_preview, ppm, default_filter);
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        // O empréstimo do `gfx` do quadro: o padrão EXAUSTIVO do `AppGfx` mora em [`frame_gfx`]; aqui
        // ficam só os campos que o resto do corpo ainda lê (os das fases já extraídas saem da lista).
        let FrameGfx {
            doc_guides,
            ui_states,
            ui_machines,
            #[cfg(feature = "sculpt3d")]
            sculpt3d,
            surface,
            renderer,
            sim,
            present,
            camera,
            asset_db,
            theme,
            toasts,
            tools,
            vello_pass,
            vector_scene,
            vec_scene,
            flip,
            text_system,
            hero_screen,
            frame_order,
            band_doc_scenes,
            hero_live,
            next_import_cell,
            sheets,
            sheet_textures,
            next_sheet_id,
            atlas_asset_map,
            asset_catalogs,
            component_registry,
            editor_queue,
            transform_type_id,
            visibility_type_id,
            name_type_id,
            sprite_type_id,
            imageio_exporters,
            motion,
            physics,
            component_palette_target,
            frost_doc_scene,
            frost_front_scene,
            frosting,
            ..
        } = FrameGfx::of(gfx);

        // Sprite-layer clear color = backdrop visible in the canvas
        // area through the transparent regions of `vello_rt`. Live
        // editor mode wants a static neutral surface so it doesn't
        // pulse rainbow under the chrome.
        //
        // ⭐⭐ A cor sai da PORTA (`canvas_clear::canvas_clear_rgb`),
        // que a deriva do mesmo token que o painter do canvas e o
        // cartão do navegador de assets lêem — era um literal
        // `(0.047, 0.047, 0.055)`, cópia à mão do `Bg1` do Forge, e
        // enquanto foi cópia mudar a cor do canvas movia o resto do
        // app e deixava o canvas onde estava.
        //
        // ⛔ A conversão sRGB→linear continua deliberadamente por
        // fazer (o byte divide-se por 255): é a regressão dos
        // "pixelated borders" da M14.5 ronda 2, medida e revertida.
        // O mecanismo inteiro está no cabeçalho do `canvas_clear`.
        let (r, g, b) = if hero_live.is_some() {
            crate::canvas_clear::canvas_clear_rgb(*theme)
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
        let paint_ctx = PaintCtx {
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
                // A resposta da porta única, computada no início deste quadro.
                self.hovered_object,
                sim,
                present,
                camera,
                asset_db,
                atlas_asset_map,
                asset_catalogs,
                sheets,
                renderer,
                window_size,
                self.game_camera_preview,
                self.last_pointer,
                self.frame_ms_ewma,
                self.frame_cpu_ms_ewma,
                diag_input_events,
                diag_paint_stamps,
                self.paint_ms_ewma,
                // Deform Transform live ⇒ the sprite gizmo is suppressed for the frame (its corner
                // handles share the deform gizmo's screen corners on a whole-image transform).
                ph2d_app_painter::painter_bridge_queries::deform_transform_gizmo_active(tools),
                // Em que disposição a folha aberta está — a caixa do gizmo envolve-a inteira.
                &tool_preview_bits,
                vec_scene,
                // O gizmo da forma só existe fora da ferramenta vetorial, ou no modo
                // Select dela (ADR-0112).
                //
                // ⚠️ **E nunca durante o modo de PREVIEW** (W7r): a caixa é derivada da pose
                // AUTORADA, então enquanto a máquina move a forma ela fica para trás e passa a
                // descrever um lugar que a forma já não ocupa — é a razão pela qual o ADR-0128
                // recusou cinco vezes um gizmo sobre geometria que se move. E as alças dela
                // registram hit-rects, que é o mesmo motivo pelo qual o ADR-0112 já a suprime
                // nos modos de nó: uma caixa sobre a apresentação é um ladrão de cliques.
                (!tools
                    .active()
                    .is_some_and(|t| t.id() == ph2d_editor_core::ToolId::new("vector"))
                    || self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Select)
                    && !self.ui_preview.is_on(),
                // As poses que o último desenho derivou — sem elas a caixa do gizmo de um filho
                // colocado aparece onde a forma foi AUTORADA.
                &self.vec.view_derived,
                flip,
                // Idem para o objeto Flip: gizmo fora da tool Flip, ou no modo Select
                // dela — em Draw/Erase ele comeria o clique do canvas (ADR-0112 parity).
                !tools
                    .active()
                    .is_some_and(|t| t.id() == ph2d_editor_core::ToolId::new("flip"))
                    || matches!(
                        self.flip_state.style.map(|s| s.mode),
                        Some(ph2d_tool_flip::FlipMode::Select)
                    ),
                // W4: the range the §11 Bake button covers, resolved HERE
                // because the shell owns both the document and the clock, and
                // shown on the button so the artist never has to guess it.
                // Two numbers now — the loop's start is honoured (W-BakeRange),
                // so a `[2s, 5s]` loop bakes `[2s, 5s]` and the button says so.
                {
                    let (bs, be) =
                        ph2d_app_physics::bake::bake_range(&self.timeline.doc, &self.playhead);
                    (bs as f32, be as f32)
                },
                // Which pose channels the Bake selector shows as chosen.
                self.bake_channels.tag(),
                // The pending join KIND the §11 selector shows as chosen.
                self.physics.join_kind,
                // The armed §12 joint-body eyedropper, so the waiting slot's
                // picker paints pressed.
                self.joint_body_pick,
                // W-JointCopy: quantos joints um Paste atingiria. `0` sem nada
                // copiado — e é o zero que tira o botão da tela.
                //
                // ⚠️ Contado sobre a SELEÇÃO, porque o Paste é a única edição da
                // §12 que faz fan-out; e contando só quem de fato carrega um
                // `PhysicsJoint`, senão o rótulo prometeria dez alvos numa
                // seleção de nove sprites e um joint.
                if self.joint_clipboard.is_some() {
                    hero.gizmo
                        .iter_selected()
                        .filter(|&b| {
                            sim.world()
                                .get::<ph2d_physics_ecs::PhysicsJoint>(ph2d_ecs::Entity::from_bits(
                                    b,
                                ))
                                .is_some()
                        })
                        .count()
                } else {
                    0
                },
                // W17: quantos tiques de corrida gravada o documento carrega — e
                // é o zero que tira o *Clear Recorded Run* da tela, pelo mesmo
                // desenho do Paste acima.
                self.player_tape.len(),
                // W24: e quantos esperam por um desfazer. O par decide QUAL
                // botão a §14 pinta, e os dois nunca são não-zero ao mesmo tempo
                // (descartar esvazia a fita viva).
                self.discarded_run.len(),
                self.fixed_step.fixed_dt(),
                // `W-PlayerOut` A3: o readout do player SELECIONADO. Resolvido
                // aqui porque `publish` não recebe a ponte, e pela porta única —
                // `None` fora de um player, e também com a física desarmada, que
                // é o que faz a §14 dizer *"not simulating"* em vez de mostrar
                // números de uma corrida que acabou.
                hero.gizmo
                    .selection
                    .and_then(|b| physics.player_view(ph2d_ecs::Entity::from_bits(b)))
                    .copied(),
                // **O que a LEI de facto lê deste personagem** — resolvido aqui
                // pela mesma razão do readout acima (`publish` não recebe a
                // ponte) e pela MESMA porta que decide quem escreve a pose. A
                // shell re-derivá-lo do `PlayerMode` era a segunda cópia que
                // fazia a §14 pintar doze cards vivos sobre um player ASSADO,
                // que a lei não dirige.
                hero.gizmo
                    .selection
                    .map_or(ph2d_physics_ecs::PlayerLiveness::INERT, |b| {
                        physics.player_liveness(sim.world(), ph2d_ecs::Entity::from_bits(b))
                    }),
                // W-Pulley W3: o eyedropper de montagem da §13, pelo mesmo motivo.
                self.wheel_body_pick,
                self.wheel_rope_pick,
                // W-J4: o gesto de desenhar está armado?
                self.physics.joint_draw_armed,
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
                    // E as alças da RODA selecionada (W-Pulley W1). Terceira
                    // família, e a única que lê a SELEÇÃO: uma corda com seis
                    // roldanas publicaria doze alças sobrepostas.
                    hs.extend(point_gizmo::wheel_handles(
                        sim,
                        hero.gizmo.selection,
                        self.show_colliders,
                        at_rest,
                    ));
                    // E a QUARTA: os limitadores da corda (W-RopeStop). De toda
                    // polia, como as âncoras — a marca É a feature, e escondê-la
                    // atrás de uma seleção faria o artista ter de descobrir que
                    // ela existe antes de poder descobri-la.
                    hs.extend(point_gizmo::rope_stop_handles(
                        sim,
                        physics,
                        self.show_colliders,
                        at_rest,
                    ));
                    hs
                },
                // The candidate a live anchor drag has caught (the crosshair).
                self.physics.joint_anchor_drag.and_then(|d| d.snap),
                // **O SELO do papel booleano de cada linha** (2026-08-22). ⚠️ Ele lê o plano do
                // quadro ANTERIOR: a hierarquia publica aqui, e a booleana cozinha lá em baixo
                // no mesmo `run_render_frame`. O atraso é de um quadro e o `vec_bool_shape` o
                // documenta — mover qualquer das duas metades na ordem do frame é mudança com
                // gates próprios e sem nada a ganhar.
                // ⭐⭐ **E o selo de quem SEGUE UM DESENHO** (W57), fundido no mesmo mapa: o
                // campo é um selo por linha, e as duas famílias nunca caem na mesma entidade (uma
                // é forma vetorial, a outra é nó do modelador). ⚠️ Fundir aqui, e não somar dois
                // mapas lá dentro, é o que mantém *um produtor, um campo* — a lei que o comentário
                // do `hovered` já escreve dez linhas acima.
                &{
                    let mut b =
                        crate::vec_bool_shape::badges(sim, &self.vec.entities, &self.bool_live);
                    b.extend(ph2d_app_field3d::scene::link_badges());
                    b
                },
                // O registo — ver o parâmetro na assinatura do `publish`.
                component_registry,
            );
            // ⭐⭐⭐ **A RECEITA VEM AO ARTISTA** (Enio, 2026-09-07) — servido AQUI porque é a linha
            // acima que publica a caixa dela, e é dessa caixa que o deslocamento sai. ⚠️ O gizmo
            // deste quadro já foi projectado com a pose ANTIGA, então ele desenha um quadro
            // atrasado; o desenho do mundo (que é encodado mais abaixo) já usa a nova. *Um quadro
            // de 16 ms, contra a alternativa de reconstruir a vista inteira só para o esconder.*
            crate::prefab_stage::run(
                &mut self.prefab_stage_pending,
                &mut self.prefab_stage,
                hero,
                ph2d_editor_core::zones::Rect::new(
                    0.0,
                    0.0,
                    window_size.width as f32,
                    window_size.height as f32,
                ),
                window_size,
                camera,
                sim,
                &mut self.preview_drive,
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
                .is_some_and(|t| t.id() == ph2d_editor_core::ToolId::new("flip"))
                && matches!(
                    self.flip_state.style.map(|s| s.mode),
                    Some(ph2d_tool_flip::FlipMode::Edit)
                );
            hero.gizmo.pose_view = flip_edit_mode
                .then(|| {
                    ph2d_app_flip::pose_gizmo::pose_view(
                        sim,
                        flip,
                        &self.flip_state.entities,
                        ph2d_app_flip::pose_gizmo::PoseViewInputs {
                            playhead: &self.playhead,
                            active_layer: self.flip_state.active_layer,
                            last_pointer: self.last_pointer,
                        },
                        camera,
                        window_size,
                    )
                })
                .flatten();
            hero.gizmo.selection_view = flip_edit_mode
                .then(|| {
                    ph2d_app_flip::selection_gizmo::selection_view(
                        sim,
                        flip,
                        &self.flip_state.entities,
                        ph2d_app_flip::selection_gizmo::SelectionViewInputs {
                            playhead: &self.playhead,
                            active_layer: self.flip_state.active_layer,
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
                .is_some_and(|t| t.id() == ph2d_editor_core::ToolId::new("motion"));
            // As dims da CENA (o sub-retângulo do split, `CenterSplit::scene_viewport`) — o
            // gizmo é pintado e arrastado com ELAS, casando com o `set_viewport` do render
            // (present.rs). É o fix do drift crônico: sob o split a cena renderiza na banda
            // e o chrome projetava a janela cheia. Fora do split = janela cheia (no-op).
            let (scene_w, scene_h) =
                ph2d_app_motion::field_gizmo::scene_window_wh(hero.view.center_split, window_size);
            hero.gizmo.field_view = motion_tool_active
                .then(|| {
                    ph2d_app_motion::field_gizmo::field_view(
                        motion,
                        camera,
                        scene_w,
                        scene_h,
                        self.last_pointer,
                    )
                })
                .flatten();
            // **O gizmo dos DEFORMADORES DE QUADRILÁTERO** (Corner Pin + Bezier Warp) —
            // publicado no mesmo sítio e pela mesma modalidade do field: só com a tool
            // Motion activa. ⚠️ Publicar de novo SUBSTITUI, então largar a selecção limpa
            // as alças em vez de as deixar a pairar.
            ph2d_app_motion::warp_gizmo::publish(ph2d_app_motion::warp_gizmo::resolve(
                motion,
                motion_tool_active,
            ));
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
            let mut reparent_intent: Option<ph2d_editor_core::screens::hero::HierReparentIntent> =
                None;
            let mut duplicate_row: Option<NodeId> = None;
            // Set by `hierarchy::dispatch` to `(source_bits, new_bits)` when a sprite is duplicated, so
            // we can fork the copy onto its own texture (independent object) post-dispatch.
            let mut duplicate_made: Option<(u64, u64)> = None;
            let mut add_child_row: Option<NodeId> = None;
            // ⭐⭐ **Agrupar / desagrupar** (2026-08-30): `(linha clicada, agrupar?)`. Um slot só para
            // os dois verbos — eles são o mesmo gesto com o sinal trocado, e dois slots deixariam
            // a porta aberta a alguém drenar os dois no mesmo quadro.
            let mut group_row: Option<(NodeId, bool)> = None;
            // ⭐ **O `Add` do cabeçalho da Hierarquia** (ADR-0166 / F3) — um objeto vazio na raiz.
            // Sem payload: ele não sai de uma linha, e por isso não tem pai (ver `HierAddRoot`).
            let mut add_root = false;
            let mut reset_transform_row: Option<NodeId> = None;
            // ⭐ *Revert to Master* (ADR-0164 / F4.4) — a linha cuja instância volta à receita.
            let mut revert_to_master_row: Option<NodeId> = None;
            // ⭐ Os outros verbos de instância (ADR-0164 / F4.5) — UM slot, porque eles são
            // exclusivos por construção: o menu fecha ao primeiro clique.
            let mut instance_verb_row: Option<(NodeId, ph2d_app_components::instance_verbs::Verb)> =
                None;
            // ⭐ O mesmo verbo, endereçado por `StableId` — o canal do navegador de assets.
            let mut instance_verb_stable_id: Option<(
                u64,
                ph2d_app_components::instance_verbs::Verb,
                Option<[f32; 2]>,
            )> = None;
            // ⭐⭐ O menu de um CARTÃO da biblioteca (etapa C) — o par `(endereço, verbo)` que o
            // painel transporta. ⚠️ **Slot próprio, e não o `instance_verb_stable_id`:** metade
            // das seis células é uma RECUSA que só o shell sabe redigir (o número de utilizadores
            // de uma imagem), e dobrá-lo no slot dos verbos de instância obrigaria a inventar um
            // `Verb` para *«não faça nada e diga porquê»*.
            // ⭐⭐ Os verbos de CATÁLOGO (wave A3). ⚠️ **Um `Vec`, e não um slot único**: ao
            // contrário dos verbos de instância, dois destes PODEM chegar no mesmo quadro sem
            // conflito (criar e escolher, por exemplo) — e eles não competem por um sujeito.
            let mut catalog_verbs: Vec<ph2d_editor_core::action_bus::CatalogVerb> = Vec::new();
            let mut asset_card_verb: Option<(
                ph2d_editor_core::interaction::drag_payload::DragPayload,
                ph2d_editor_core::action_bus::AssetCardAction,
            )> = None;
            let mut delete_row: Option<NodeId> = None;
            // Enio 2026-05-27: right-click → Merge Sprites in Hierarchy.
            // Carries the clicked row's `NodeId` (the merged sprite
            // adopts that row's parent for Hierarchy placement); the
            // drain reads the full multi-selection at apply time.
            let mut merge_sprites_row: Option<NodeId> = None;
            // "Pack into Sheet" do menu de contexto da hierarquia — a 2ª porta do verbo do pill
            // `[SHEET]`. Guarda a LINHA (não a entidade): quem a resolve é o `bridge`, no dreno.
            let mut pack_sheet_row: Option<NodeId> = None;
            // "Auto-Arrange Pieces" — re-encaixar os filhos de uma folha que já existe.
            let mut arrange_sheet_row: Option<NodeId> = None;
            // "Remove from Sheet" — a saída da folha, pela linha clicada.
            let mut remove_from_sheet_row: Option<NodeId> = None;
            // As duas saídas do BAKE (plano §7.3, W5.2): assar muda a cena, exportar escreve
            // ficheiros. Linhas separadas porque são dois pedidos diferentes.
            let mut bake_sheet_row: Option<NodeId> = None;
            let mut export_sheet_row: Option<NodeId> = None;
            // **EXPORTAR UMA SPRITE** (plano `docs/Sprite_projeto/18` W9) — irmão do de cima, e a
            // diferença está no nome: aquele escreve a FOLHA, este escreve uma sprite no formato
            // que a extensão escolhida nomear.
            let mut export_image_row: Option<NodeId> = None;
            // **FUNDIR EM CAMADAS** (plano `docs/Sprite_projeto/18` W10) — a mesma geometria do
            // Merge, e cada fonte fica também numa camada do documento do Painter.
            let mut merge_to_layers_row: Option<NodeId> = None;
            let mut use_as_brush_texture_row: Option<NodeId> = None;
            let mut use_as_brush_shape_row: Option<NodeId> = None;
            let mut use_as_paper_row: Option<NodeId> = None;
            let mut use_as_granulation_row: Option<NodeId> = None;
            let mut hierarchy_row_click: Option<NodeId> = None;
            let mut hierarchy_select_intent: Option<hierarchy::HierarchySelectIntent> = None;
            let mut rename_seed_row: Option<NodeId> = None;
            let mut rename_commit: Option<(NodeId, String)> = None;
            let mut view_focus_kind: Option<ph2d_editor_core::ViewFocusKind> = None;
            let mut reimport_entity: Option<u64> = None;
            // O pedido de troca de PRECISAO (plano `docs/Sprite_projeto/18` W5). `Option` e nao
            // `Vec`: o par so' existe com uma sprite selecionada.
            let mut precision_request: Option<(u64, ph2d_color::Precision)> = None;
            // **A SPRITE COMO FONTE DE LUZ** (plano `docs/Sprite_projeto/18` W8). Recolhido aqui e
            // drenado com o irmão `precision_request` — o mesmo padrão, porque o componente só pode
            // ser escrito onde o `sim` está emprestado mutavelmente.
            // ⚠️ **Um Vec, não um `Option`** — a emissão é uma edição de campo como a Opacidade, e
            // a Opacidade espalha-se pela seleção. Enquanto isto foi `Option<(u64, f32)>` o slider
            // parecia um bulk edit e mudava **uma** sprite (auditoria `docs/Sprite_projeto/20` §3).
            let mut emissive_edits: Vec<(u64, f32)> = Vec::new();
            // Fase 0e: per-sprite tools collect a Vec<u64> instead of
            // Option<u64> so a multi-select OneShotImageOp broadcast
            // applies the bake to every selected sprite (legacy
            // single-select still works — the Vec just carries one
            // entry). image_edit::dispatch iterates each Vec.
            let mut trim_entities: Vec<u64> = Vec::new();
            let mut make_square_entities: Vec<u64> = Vec::new();
            let mut real_size_entities: Vec<u64> = Vec::new();
            let mut rasterize_entities: Vec<u64> = Vec::new();
            // ⚠️ **Este NÃO é por-sprite, e é a exceção da fila.** A chrome emite um
            // `OneShotImageOp` por entidade selecionada; os irmãos aplicam o bake a cada um
            // isoladamente, e este junta a leva inteira para criar **uma** folha. N atos
            // independentes dariam N folhas de uma peça cada — um verbo que fala da RELAÇÃO
            // entre as peças não cabe num evento por peça.
            let mut undo_image_edit = false;
            // ADR-0108 Fase 1: a Boolean button (Union/Subtract/Intersect) in the
            // docked Vector panel forwards a `ToolPanelEvent::Click`; the op acts
            // on the DOCUMENT (shell-owned `vec_scene`), not the tool's Style, so
            // capture it here and apply after the drain (mirror of the U/I/D
            // hotkeys, next to the vector render).
            let mut pending_vec_bool: Option<ph2d_vec_boolean::PathfinderOp> = None;
            let mut pending_vec_expand: Option<crate::vec_expand::Expand> = None;
            // OS COMPONENTES (plano UI/UX W5): o verbo pedido neste frame.
            let mut pending_component: Option<crate::vec_component_edit::ComponentEdit> = None;
            let mut pending_widget_edit: Option<crate::vec_widget_edit::WidgetEdit> = None;
            // OS ESTADOS de UI (plano UI/UX W7): a tabela mora no documento, entao o clique e' da
            // shell — o painel so' mostra que verbos fazem sentido agora.
            let mut pending_ui_state: Option<crate::vec_ui_state_edit::UiStateEdit> = None;
            let mut pending_ui_state_duration: Option<f64> = None;
            // ⚠️ Um TOGGLE não traz valor: o pedido é *"inverta"*, e quem sabe o estado atual é
            // a tabela. Um `Some(bool)` obrigaria a shell a lê-la duas vezes.
            let mut pending_ui_spring_toggle = false;
            // (é a rigidez?, valor) — só o knob que o artista arrastou.
            let mut pending_ui_spring_knob: Option<(bool, f64)> = None;
            let mut pending_ui_easing: Option<crate::vec_ui_state_edit::EasingPick> = None;
            // ⭐ **A TABELA SINAL → PAPEL** (item 4 do estudo dos contêineres): os três gestos de
            // clique e o COMMIT do nome. Duas variáveis porque são dois canais do barramento —
            // o `Click` e o `SelectOption`, que é o único variante do `PanelEvent` (contrato
            // CONGELADO) que carrega uma string.
            let mut pending_ui_signal_edit: Option<crate::vec_ui_state_edit::SignalEdit> = None;
            let mut pending_ui_signal_name: Option<(usize, String)> = None;
            let mut pending_ui_preview_toggle = false;
            let mut pending_ui_move_all_toggle = false;
            // **A BOOLEANA VIVA** (plano UI/UX W1): o Apply consolida o que o produtor cozinhou
            // NESTE frame, então ele não pode correr aqui — corre logo depois do `recook`, onde o
            // plano existe. Aqui só se anota o clique.
            let mut pending_morph_arrow: Option<crate::vec_morph_edit::MorphCmd> = None;
            let mut pending_morph_preview_toggle = false;
            let mut pending_bool_apply = false;
            // A MOLDURA (plano UI/UX W0): o chip de recorte e o preset de dispositivo.
            let mut pending_frame_clip: Option<bool> = None;
            // **O VERBO DA FORMA selecionada** dentro de uma booleana viva (2026-08-22).
            // Irmao exacto do `pending_frame_clip`, e pelo mesmo motivo: o valor mora num
            // COMPONENTE, entao quem escreve e' a shell — o painel so' mostra qual chip
            // esta' aceso.
            let mut pending_bool_shape_op: Option<u8> = None;
            // O AUTO LAYOUT (plano UI/UX W2, ADR-0153): um chip de radio e um campo numerico.
            let mut pending_layout_edit: Option<crate::vec_layout_edit::LayoutEdit> = None;
            let mut pending_anchor_edit: Option<crate::vec_anchor_edit::AnchorEdit> = None;
            // **Resize Box** (plano UI/UX W3b): o clique e' um TOGGLE, entao nao ha' operando —
            // um bool basta para dizer *"houve clique"*.
            let mut pending_resize_box = false;
            // ⭐ **Stroke** (plano 34): a caixa que dá/tira o traço da forma selecionada. Também é
            // um TOGGLE, então um bool basta — o operando é a ficha da ferramenta, e ela não viaja.
            let mut pending_stroke_present = false;
            let mut pending_layout_field: Option<(crate::vec_layout_edit::LayoutField, f64)> = None;
            // **O Z-INDEX global** (Enio, 2026-08-04): o numero que sobrepoe a ordem da
            // hierarquia. Campo numerico, entao a rota e' a mesma do Transform.
            let mut pending_vec_z: Option<f64> = None;
            // **O TOKEN escolhido no picker** (plano UI/UX W4): a propriedade + o token, ou
            // `None` no token = SOLTAR (a propriedade volta ao literal do documento).
            let mut pending_token_bind: Option<(ph2d_ecs::BoundProp, Option<&'static str>)> = None;
            let mut pending_frame_preset: Option<ph2d_tool_vector::frames::DevicePreset> = None;
            // **A ESCALA da seleção de nós** (plano 25 §6, W3b): os dois alcances que o retângulo
            // não dá. Não são edições de documento — só mudam QUEM está selecionado —, então não
            // abrem passo de undo (o `post_frame_undo` compara o ESTADO, e a seleção não é dele).
            let mut pending_vec_select_subpath = false;
            let mut pending_vec_select_same = false;
            // **As três da W4** (plano 25 §7). Ao contrário das duas acima, estas MUDAM o
            // documento — logo abrem passo de undo, e cada uma abre exatamente um.
            let mut pending_vec_join = false;
            // ⭐⭐⭐ **Soldar** (plano 39): os traços seleccionados partem-se nos cruzamentos.
            let mut pending_vec_weld = false;
            let mut pending_vec_cut = false;
            let mut pending_vec_symmetry_apply = false;
            let mut pending_vec_cut_discard = false;
            let mut pending_vec_reverse = false;
            let mut pending_vec_average = false;
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
            // ⭐⭐⭐ O ESQUELETO (estudo 42 item 5): prender a selecção aos ossos, as duas saídas, e
            // os dois números do osso em foco (`true` = a força, `false` = o comprimento).
            let mut pending_bone_bind = false;
            let mut pending_bone_release: Option<crate::skeleton_live::Keep> = None;
            let mut pending_bone_knob: Option<(bool, f64)> = None;
            /// Qual dos três números da ÂNCORA o campo escreveu.
            #[derive(Clone, Copy, PartialEq)]
            enum IkKnob {
                Mix,
                Softness,
                Chain,
            }
            let mut pending_ik_add = false;
            let mut pending_ik_remove = false;
            // ⭐ O lado da dobra que o artista escolheu neste quadro, se escolheu.
            let mut pending_ik_bend: Option<ph2d_skeleton::BendSide> = None;
            let mut pending_ik_knob: Option<(IkKnob, f64)> = None;
            let mut pending_limit_add = false;
            let mut pending_limit_remove = false;
            // ⚠️ `(é o MAX?, valor em GRAUS)` — a conversão para radianos é feita onde ele é
            // escrito, que é a porta onde as duas unidades se encontram.
            let mut pending_limit_knob: Option<(bool, f64)> = None;
            let mut pending_smart_add = false;
            let mut pending_smart_remove = false;
            let mut pending_smart_knob: Option<(bool, f64)> = None;
            // A acção escolhida no selector do osso inteligente — o ÍNDICE na lista de clips que o
            // painel pinta; o que se guarda no componente é o NOME dela.
            let mut pending_smart_clip: Option<usize> = None;
            // O *Pick Object* foi carregado — arma o gesto de duas mãos do alvo.
            let mut pending_smart_pick = false;
            // ⭐ **A ferramenta tem de ser armada em *Transform* no fim do quadro** — ver a aresta
            // do foco lá em baixo. Um flag, e não a escrita directa, porque ali o `gfx` já está
            // emprestado a `sim`/`hero`.

            // ⭐⭐⭐ **UM CONTROLO DESTA SEÇÃO FOI TOCADO E O SUJEITO DELE É UM OSSO EM FOCO.**
            //
            // ⛔⛔ A pergunta é **DERIVADA** das tabelas de ids (`ids::needs_focused_bone`), e a
            // derivação é a cura: o braço que diz *«nenhum osso em foco»* era uma disjunção escrita
            // à mão — nasceu com dois verbos, tinha oito quando a auditoria de 2026-09-08 a apanhou,
            // e os CAMPOS e as duas fileiras de chips nunca lá entraram. *Uma cura escrita para os
            // verbos que existiam não segue os que vêm.*
            let mut pending_bone_needs_focus = false;
            // ⚠️ **O osso seleccionado lê-se AQUI, antes de o mundo ser emprestado mutável** — os
            // verbos lá em baixo já seguram `sim`, e uma leitura de `self` no meio deles não
            // compila. O valor é do QUADRO, e é o mesmo que o gesto e o overlay usam.
            let osso_selecionado =
                crate::bone_gesture::selected_bone(sim, hero.gizmo.iter_selected());
            // ⭐ **A selecção CRUA, guardada aqui pela mesma razão que o `osso_selecionado`**: o
            // `hero` é uma vista do `gfx`, e quem a lê lá em baixo (o *Bind* da 2.ª mídia) já o
            // tem emprestado de outra maneira. *Ler o valor uma vez é o que torna a pergunta
            // alcançável nos dois sítios.*
            let selecao_bits: Vec<u64> = hero.gizmo.iter_selected().collect();
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
            // A lei do PADRÃO de textura (plano 33 W5). Uma só por quadro: os controles da secção
            // são exclusivos entre si (o artista mexe num de cada vez), e uma fila daria dois passos
            // de undo para um gesto.
            // ⚠️ **Cada um leva o SUJEITO junto** (plano 35, wave F): o slot sai do id do controlo
            // que foi clicado, e não de uma preferência guardada — *o que o gesto endereça não pode
            // ser lido de outro sítio no drain.*
            let mut pending_texpat: Option<(
                ph2d_vec_render::PatternSlot,
                crate::texture_pattern_edit::TexPatCmd,
            )> = None;
            let mut pending_texpat_source: Option<ph2d_vec_render::PatternSlot> = None;
            let mut pending_texpat_pick: Option<ph2d_vec_render::PatternSlot> = None;
            // ⭐ A TINTA do traço (plano 35, wave D) — irmã do `pending_vec_fill_kind`, e drenada no
            // MESMO sítio, porque as duas podem precisar de abrir o diálogo da arte.
            let mut pending_vec_stroke_kind: Option<ph2d_panel_vector::StrokePaintKind> = None;
            // ⭐ O PINCEL (plano 36, W4): o gesto que arma a arte, e a lei dos knobs.
            let mut pending_brush_pick = false;
            let mut pending_brush: Option<crate::vec_stroke_paint::BrushCmd> = None;
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
            let mut pending_vec_snap_path: Option<bool> = None;
            let mut pending_vec_snap_cross: Option<bool> = None;
            let mut pending_vec_snap_guides: Option<bool> = None;
            let mut pending_rulers: Option<bool> = None;

            // Numeric Transform field edit (X/Y/W/H) — a SetValue document command.
            // ⭐ A APARÊNCIA do objecto (estudo 42 item 2): o track `0..1` do slider e o CÓDIGO do
            // modo de mistura. Capturados aqui e aplicados ao documento no dreno, como o Transform.
            let mut pending_vec_opacity: Option<f64> = None;
            let mut pending_vec_blend: Option<u8> = None;
            // ⭐⭐⭐ A PILHA DE APARÊNCIA (estudo 42 item 4): o verbo pedido, e as três propriedades
            // da camada ABERTA. ⚠️ O índice vem do PAINEL (a camada aberta é vista dele), então a
            // shell não guarda um segundo — dois índices para a mesma pergunta divergem no
            // primeiro gesto que mexe na pilha.
            let mut pending_paint_verb: Option<crate::vec_paint_stack::StackVerb> = None;
            let mut pending_paint_width: Option<f64> = None;
            // ⭐ ONDE a camada aberta desenha (v21). Dois slots e nao um par: as duas caixas
            // comitam INDEPENDENTES, e um par obrigaria a inventar o eixo que nao mudou.
            let mut pending_paint_dx: Option<f64> = None;
            let mut pending_paint_dy: Option<f64> = None;
            // ⭐ O OFFSET DE CAD da camada aberta (v22) e a quina dele.
            let mut pending_paint_dilate: Option<f64> = None;
            let mut pending_paint_join: Option<u8> = None;
            let mut pending_paint_opacity: Option<f64> = None;
            let mut pending_paint_blend: Option<u8> = None;
            let mut pending_vec_transform: Option<(crate::input_dispatch::VecTransformField, f64)> =
                None;
            // **ONDE o NÓ vai** — `(eixo_y?, alvo)` na unidade do artista. Um por frame: os dois
            // campos são commitados por gestos distintos, e mandar os dois no mesmo quadro
            // significaria dois deslocamentos, que é o que o `nudge` já faz num.
            let mut pending_vec_vert: Option<(bool, f64)> = None;
            // Transform Angle field (R) — a relative rotation delta (degrees).
            let mut pending_vec_rotate_by: Option<f64> = None;
            // Slider de parâmetro de forma (Sides/Points/Inner/Radius/Turns/Degrees):
            // `(id, track 0..1)`. A tool já o consome como default de desenho; aqui ele
            // também edita a forma VIVA selecionada (Live Shape).
            let mut pending_vec_shape_param: Option<(ph2d_editor_core::NodeId, f64)> = None;
            // Campo do CONECTOR (Route / Jetty / Spread): `(id, valor)`. Não é Style da tool
            // — é a RELAÇÃO, que mora no `VecConnector` de cada conector SELECIONADO (todos
            // eles: é assim que se calibra o diagrama inteiro de uma vez).
            let mut pending_vec_connector: Option<(ph2d_editor_core::NodeId, f64)> = None;
            // Text Size slider (world units) — updates the active session + the
            // size a new session starts at.
            let mut pending_vec_text_size: Option<f64> = None;
            // Text Weight slider (`wght` axis) — updates the active session + the
            // weight a new session starts at.
            let mut pending_vec_text_weight: Option<f32> = None;
            // Paragraph: line-height (× size), tracking (em), and alignment (L/C/R).
            let mut pending_vec_text_line_height: Option<f64> = None;
            let mut pending_vec_text_tracking: Option<f64> = None;
            // ⚠️ `Option<Option<f64>>`: o de fora é *houve pedido neste frame?*, o de dentro é
            // *Auto ou esta largura?*. Colapsá-los faria "voltar para Auto" indistinguível de
            // "ninguém tocou", e o modo Auto seria inalcançável.
            let mut pending_vec_text_wrap: Option<Option<f64>> = None;
            let mut pending_vec_text_align: Option<ph2d_vec_text::TextAlign> = None;
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
            let mut transform_edit: Option<ph2d_editor_core::InspectorTransformInfo> = None;
            let mut visibility_edits: Vec<(u64, bool)> = Vec::new();
            let mut sprite_source_change: Option<(u64, RequestedSpriteStrategy)> = None;
            // Sprite field edits (flip/region/sheet/tint/…) — a Vec so a
            // bulk edit that touches several fields in one frame all apply.
            let mut sprite_edits: Vec<(u64, ph2d_editor_core::SpriteFieldEdit)> = Vec::new();
            // §7 ordering edits (W3) — optional-component edits, fanned out
            // to the selection like sprite edits.
            let mut ordering_edits: Vec<(u64, ph2d_editor_core::OrderingFieldEdit)> = Vec::new();
            let mut sampling_edits: Vec<(u64, ph2d_editor_core::SamplingFieldEdit)> = Vec::new();
            let mut blend_edits: Vec<(u64, ph2d_editor_core::BlendFieldEdit)> = Vec::new();
            let mut slice_edits: Vec<(u64, ph2d_editor_core::SliceFieldEdit)> = Vec::new();
            let mut anchor_edits: Vec<(u64, ph2d_editor_core::AnchorFieldEdit)> = Vec::new();
            let mut anim_edits: Vec<(u64, ph2d_editor_core::AnimFieldEdit)> = Vec::new();
            let mut timer_edits: Vec<(u64, ph2d_editor_core::TimerFieldEdit)> = Vec::new();
            let mut audio_edits: Vec<(u64, ph2d_editor_core::AudioFieldEdit)> = Vec::new();
            let mut camera_edits: Vec<(u64, ph2d_editor_core::CameraFieldEdit)> = Vec::new();
            // ⚠️ **`inspector_queue_dirty` e não `audio_commit`**: desde a secção CAMERA (TOP-20
            // #7) esta bandeira serve DUAS secções, e o nome antigo passou a descrever metade do
            // que ela significa. *Um nome que já não cobre a população dele mente na próxima
            // leitura.*
            let mut inspector_queue_dirty = false;
            let mut action_edits: Vec<(u64, ph2d_editor_core::ActionFieldEdit)> = Vec::new();
            // ⭐ O `+` do Inspector (F3): quem pediu a paleta neste quadro.
            let mut add_component_for: Option<u64> = None;
            // ⭐ A troca de variante pedida neste quadro: `(raiz da instância, StableId do mestre)`.
            let mut swap_variant: Option<(u64, u64)> = None;
            // O `StableId` da peça acrescentada que o cartão mandou aplicar.
            let mut apply_added: Option<u64> = None;
            // ⭐⭐⭐ **O DEGRAU escolhido do *Aplicar*** (F5 critério 4) — `(peça clicada, receita)`.
            // ⚠️ **ADIADO pela razão da irmã de cima**: o verbo precisa do **eco** e dos documentos
            // possuídos, e aqui dentro o `self` já está emprestado.
            let mut apply_to_level: Option<(u64, u64)> = None;
            let mut open_asset_browser = false;
            // ⭐ O pedido de renomear o VALOR de uma propriedade — `(receita, chave, valor)`.
            // ⭐ A entidade cujo campo de nome fechou neste quadro.
            let mut physics_edits: Vec<(u64, ph2d_editor_core::PhysicsFieldEdit)> = Vec::new();
            // §12 joints (W3). Kept out of `inspector_commits::dispatch`: that
            // signature is already the length its own doc-comment warns about,
            // and these two are applied in one short block below.
            let mut joint_edits: Vec<(u64, ph2d_editor_core::JointFieldEdit)> = Vec::new();
            let mut wheel_edits: Vec<(u64, ph2d_editor_core::WheelFieldEdit)> = Vec::new();
            // §14 Platform Player (W5). Sem fan-out, e pela razão da §12/§13: a
            // seção descreve UM personagem, o selecionado — espalhar um `Add`
            // pela seleção criaria N players num clique que pediu um.
            let mut player_edits: Vec<(u64, ph2d_editor_core::PlayerFieldEdit)> = Vec::new();
            // The pair to join, at most one per frame — it is a click, not a
            // per-entity edit.
            let mut bake_request: Option<Vec<u64>> = None;
            // W-J4: a rota por SELEÇÃO virou "ligue a sequência" (2 corpos = um
            // joint; N = uma corrente de N−1), então o pedido é um booleano — a
            // ordem vem da própria seleção, que o `join_selected_chain` lê.
            let mut join_chain = false;
            let mut join_draw_arm = false;
            // W-Rig: um clique em *Rig* — booleano pelo mesmo motivo do
            // `join_chain`, porque a SELEÇÃO já diz sobre o que ele age.
            let mut rig_now = false;
            let mut visibility_section_edits: Vec<(u64, ph2d_editor_core::VisibilityFieldEdit)> =
                Vec::new();
            let mut name_edit: Option<ph2d_editor_core::InspectorNameInfo> = None;
            let mut signal_edit: Option<ph2d_editor_core::InspectorNameInfo> = None;
            let mut signal_leave_edit: Option<ph2d_editor_core::InspectorNameInfo> = None;
            let mut bgremoval_leftover: Vec<ph2d_editor_core::action_bus::EditorAction> =
                Vec::new();
            // Painter Apply leftover — same shape as bgremoval (drained
            // back into the bus so `image_edit::dispatch`'s
            // `painter_active` gate runs AFTER any same-frame
            // ActivateTool resolution). Day-7 ship.
            let mut painter_leftover: Vec<ph2d_editor_core::action_bus::EditorAction> = Vec::new();
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
                use ph2d_editor_core::action_bus::EditorAction;
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
                        if let ph2d_editor_core::tool::PanelEvent::Click(id) = &ev {
                            // ⭐ **A pergunta corre ANTES da cadeia** e é derivada das tabelas: um
                            // controlo novo da seção Skeleton entra aqui sem ninguém se lembrar.
                            pending_bone_needs_focus |=
                                ph2d_editor_core::ids::needs_focused_bone(*id);
                            if let Some(j) = crate::vec_paint_stack::join_code_for_id(*id) {
                                // ⭐ A QUINA do offset de CAD (v22) — um clique, não um valor.
                                pending_paint_join = Some(j);
                            } else if let Some(v) = crate::vec_paint_stack::stack_verb_for_id(*id) {
                                // ⭐ A PILHA DE APARÊNCIA: o resolvedor é PURO e vive ao lado dos
                                // verbos, como o `vec_rotate_for_id` — aqui só se captura.
                                pending_paint_verb = Some(v);
                            } else if *id == ph2d_editor_core::ids::VECTOR_BLEND_RUN {
                                // ADR-0128: cria o Blend Object VIVO da seleção (não o destrutivo).
                                pending_create_blend = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_BLEND_RESET_SPINE {
                                // ADR-0128 C2b: volta o spine editado ao automático.
                                pending_reset_spine = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_BLEND_EXPAND {
                                // ADR-0128 D: materializa os passos e descarta o objeto vivo.
                                pending_expand_blend = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_BLEND_RELEASE {
                                // ADR-0128 D: desfaz o blend; as fontes ficam.
                                pending_release_blend = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_MORPH_RUN {
                                // O irmão animável do blend: UMA forma, com o `t` keyável.
                                pending_create_morph = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_BONE_BIND {
                                // ⭐⭐⭐ O ESQUELETO (estudo 42 item 5): prende a seleção aos ossos.
                                pending_bone_bind = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_BONE_IK_ADD {
                                // ⭐⭐⭐ A ÂNCORA: dá ao osso em foco um alvo que a corrente persegue.
                                pending_ik_add = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_BONE_IK_REMOVE {
                                pending_ik_remove = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_BONE_LIMIT_ADD {
                                // ⭐⭐⭐ O LIMITE DE ÂNGULO: até onde esta junta dobra.
                                pending_limit_add = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_BONE_LIMIT_REMOVE {
                                pending_limit_remove = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_BONE_SMART_ADD {
                                // ⭐⭐⭐ O OSSO INTELIGENTE: anexa o controlo VAZIO — quem lhe dá acção é o painel.
                                pending_smart_add = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_BONE_SMART_REMOVE {
                                pending_smart_remove = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_BONE_SMART_PICK {
                                // ⭐⭐⭐ Arma o gesto de duas mãos do ALVO: o clique seguinte, no
                                // canvas OU na hierarquia, diz de que objecto este controlo trata.
                                pending_smart_pick = true;
                            } else if let Some(i) =
                                ph2d_editor_core::ids::VECTOR_BONE_SMART_CLIP_IDS
                                    .iter()
                                    .position(|x| x == id)
                            {
                                // ⭐⭐⭐ **QUAL acção** — a posição na tabela É o índice do clip, e é
                                // ela que impede a lista pintada e a lista honrada de divergirem.
                                pending_smart_clip = Some(i);
                            } else if let Some(i) = ph2d_editor_core::ids::VECTOR_BONE_BEND_IDS
                                .iter()
                                .position(|x| x == id)
                            {
                                // ⭐⭐⭐ **O LADO DA DOBRA** — a posição na tabela É a variante, e
                                // é ela que impede a fileira e o vocabulário de divergirem. ⚠️ Um
                                // `match` de três braços escritos à mão aqui seria a quinta lista
                                // escrita à mão desta seção.
                                pending_ik_bend = ph2d_skeleton::BendSide::ALL.get(i).copied();
                            } else if *id == ph2d_editor_core::ids::VECTOR_BONE_EXPAND {
                                // Solta e fica com a pose de AGORA (o Expand do envelope).
                                pending_bone_release = Some(crate::skeleton_live::Keep::Deformed);
                            } else if *id == ph2d_editor_core::ids::VECTOR_BONE_RELEASE {
                                // Solta e devolve o que o artista DESENHOU.
                                pending_bone_release = Some(crate::skeleton_live::Keep::Source);
                            } else if *id == ph2d_editor_core::ids::VECTOR_ENVELOPE_RUN {
                                // ADR-0129: envolve a seleção (1..N) num container com gaiola.
                                pending_create_envelope = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_ENVELOPE_EXPAND {
                                // ADR-0129: a deformada vira o desenho; a gaiola morre.
                                pending_expand_envelope = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_ENVELOPE_RELEASE {
                                // ADR-0129: a fonte autorada volta; a gaiola morre.
                                pending_release_envelope = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_ENVELOPE_PERSPECTIVE {
                                // ADR-0129 Fatia D: a homografia -- lados RETOS.
                                pending_envelope_kind = Some(ph2d_ecs::EnvelopeKind::Perspective);
                            } else if *id == ph2d_editor_core::ids::VECTOR_ENVELOPE_MESH {
                                // ADR-0129 Fatia D: o patch de Coons -- os lados DOBRAM.
                                pending_envelope_kind = Some(ph2d_ecs::EnvelopeKind::Mesh);
                            } else if *id == ph2d_editor_core::ids::VECTOR_ENVELOPE_PINS {
                                // ADR-0129 Fatia E: o puppet warp (MLS-rigid).
                                pending_envelope_kind = Some(ph2d_ecs::EnvelopeKind::Pins);
                            } else if *id == ph2d_editor_core::ids::VECTOR_TEXTPATH_LINK {
                                // Plano 22: prende o texto da seleção à outra forma dela.
                                pending_textpath = Some(crate::vec_text_ride::TextPathCmd::Link);
                            } else if *id == ph2d_editor_core::ids::VECTOR_TEXTPATH_PICK {
                                // Picker: arma; a fonte (o texto em foco) é capturada no drain.
                                pending_text_pick = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_TEXTPATH_DETACH {
                                pending_textpath = Some(crate::vec_text_ride::TextPathCmd::Detach);
                            } else if *id == ph2d_editor_core::ids::VECTOR_TEXTPATH_FLIP {
                                pending_textpath =
                                    Some(crate::vec_text_ride::TextPathCmd::Flip(true));
                            } else if *id == ph2d_editor_core::ids::VECTOR_TEXTPATH_FLIP_OFF {
                                pending_textpath =
                                    Some(crate::vec_text_ride::TextPathCmd::Flip(false));
                            } else if *id == ph2d_editor_core::ids::VECTOR_PATTERNPATH_LINK {
                                pending_patternpath =
                                    Some(crate::pattern_live::PatternPathCmd::Link);
                            } else if *id == ph2d_editor_core::ids::VECTOR_PATTERNPATH_PICK {
                                // Picker: arma; a fonte (o motivo selecionado) é capturada no drain.
                                pending_pp_pick = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_PATTERNPATH_DETACH {
                                pending_patternpath =
                                    Some(crate::pattern_live::PatternPathCmd::Detach);
                            } else if *id == ph2d_editor_core::ids::VECTOR_PATTERNPATH_FLIP {
                                pending_patternpath =
                                    Some(crate::pattern_live::PatternPathCmd::Flip(true));
                            } else if *id == ph2d_editor_core::ids::VECTOR_PATTERNPATH_FLIP_OFF {
                                pending_patternpath =
                                    Some(crate::pattern_live::PatternPathCmd::Flip(false));
                            } else if *id == ph2d_editor_core::ids::VECTOR_CONTOUR_ADD {
                                pending_contour = Some(crate::contour_live::ContourCmd::Add);
                            } else if *id == ph2d_editor_core::ids::VECTOR_CONTOUR_REMOVE {
                                pending_contour = Some(crate::contour_live::ContourCmd::Remove);
                            } else if *id == ph2d_editor_core::ids::VECTOR_CONTOUR_EXPAND {
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
                            } else if *id == ph2d_editor_core::ids::VECTOR_ENVELOPE_CLEAR_PINS {
                                pending_clear_pins = true;
                            } else if let Some(i) = (0..ph2d_editor_core::ids::MAX_ENVELOPE_PRESETS)
                                .find(|&i| {
                                    *id == ph2d_editor_core::ids::vector_envelope_preset_id(i)
                                })
                            {
                                // ADR-0129 Fatia C: carimba o preset `i` na gaiola.
                                pending_envelope_preset = Some(i);
                            } else if let Some(i) = (0..ph2d_editor_core::ids::MAX_WIDTH_PRESETS)
                                .find(|&i| *id == ph2d_editor_core::ids::vector_width_preset_id(i))
                            {
                                // W2b: escolhe a FORMA da largura (o catálogo de perfis).
                                pending_width_preset = Some(i);
                            } else if *id == ph2d_editor_core::ids::VECTOR_BOOL_LIVE_OFF
                                || *id == ph2d_editor_core::ids::VECTOR_BOOL_LIVE_ON
                            {
                                // O MODO dos oito botões. Panel-local no valor, mas quem o lê no
                                // clique de uma das oito é a shell — por isso ele passa por aqui.
                                ph2d_panel_vector::state::set_bool_live_on(
                                    *id == ph2d_editor_core::ids::VECTOR_BOOL_LIVE_ON,
                                );
                            } else if let Some(code) = crate::vec_bool_shape::shape_op_for_id(*id) {
                                // **O VERBO DESTA FORMA.** ⚠️ O mapeamento saiu daqui para uma
                                // porta testavel (`vec_bool_shape::shape_op_for_id`): um `match`
                                // de id enterrado neste arquivo nao e' alcancavel por teste
                                // nenhum, e foi essa a causa-raiz de os quatro chips shiparem
                                // sem um unico gate no caminho `id -> componente escrito`.
                                pending_bool_shape_op = Some(code);
                            } else if *id == ph2d_editor_core::ids::VECTOR_FRAME_PANEL_OFF
                                || *id == ph2d_editor_core::ids::VECTOR_FRAME_PANEL_ON
                            {
                                // **O painel AUTORADO** (plano UI/UX W8b.2). ⚠️ Aplicado AQUI, e
                                // nao por um `pending_*` como os vizinhos: os vizinhos escrevem no
                                // COMPONENTE (mundo), e este escreve a visibilidade do painel, que
                                // e' um fato do `HeroScreen` — que esta' em maos exactamente aqui.
                                // Um pending o adiaria para um escopo que teria de re-emprestar o
                                // hero para dizer a mesma coisa.
                                hero.panel_visibility.insert(
                                    ph2d_panel_authored::visibility_key(),
                                    *id == ph2d_editor_core::ids::VECTOR_FRAME_PANEL_ON,
                                );
                            } else if *id == ph2d_editor_core::ids::VECTOR_FRAME_CLIP_OFF
                                || *id == ph2d_editor_core::ids::VECTOR_FRAME_CLIP_ON
                            {
                                // A MOLDURA recorta ou não. O valor mora no COMPONENTE (mundo),
                                // então o clique é da shell — o painel só mostra.
                                pending_frame_clip =
                                    Some(*id == ph2d_editor_core::ids::VECTOR_FRAME_CLIP_ON);
                            } else if let Some(e) = crate::vec_layout_edit::layout_edit_for_id(*id)
                            {
                                // O AUTO LAYOUT (plano UI/UX W2): direção, alinhamento e
                                // distribuição moram no COMPONENTE, então o clique e' da shell —
                                // o painel so' mostra qual chip esta' aceso.
                                pending_layout_edit = Some(e);
                            } else if *id == ph2d_editor_core::ids::VECTOR_TRANSFORM_RESIZE_BOX {
                                // **Resize Box** (plano UI/UX W3b): o override mora no COMPONENTE,
                                // entao o clique e' da shell — o painel so' mostra o estado.
                                pending_resize_box = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_STROKE_PRESENT {
                                // **Stroke** (plano 34): dar ou tirar o traço mexe no DOCUMENTO,
                                // então o clique e' da shell — o painel so' mostra o estado.
                                pending_stroke_present = true;
                            } else if let Some(e) =
                                crate::vec_component_edit::component_edit_for_id(*id)
                            {
                                // OS COMPONENTES (plano UI/UX W5): mestre e instância moram no
                                // ECS, entao o clique e' da shell — o painel so' mostra que
                                // verbos fazem sentido.
                                pending_component = Some(e);
                            } else if let Some(e) =
                                crate::vec_ui_state_edit::ui_state_edit_for_id(*id)
                            {
                                // OS ESTADOS de UI (W7): gravar, mostrar e esquecer uma pose.
                                pending_ui_state = Some(e);
                            } else if let Some(p) =
                                crate::vec_ui_state_edit::easing_pick_for_id(*id)
                            {
                                // **O SELETOR DE CURVA** (W7): a forma e a direcao da transicao.
                                pending_ui_easing = Some(p);
                            } else if let Some(e) =
                                crate::vec_ui_state_edit::signal_edit_for_id(*id)
                            {
                                // ⭐ **A TABELA SINAL → PAPEL**: a ligação mora no DOCUMENTO
                                // (`HostStates.on_signal`), então os três gestos atravessam o
                                // barramento como os verbos ao lado.
                                pending_ui_signal_edit = Some(e);
                            } else if *id == ph2d_editor_core::ids::VECTOR_STATE_SPRING {
                                // **A MOLA** (W7m): ela troca o MOTOR da transição, e o motor mora
                                // na tabela do documento — então o checkbox atravessa o barramento
                                // como os verbos ao lado.
                                pending_ui_spring_toggle = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_STATE_MOVE_ALL {
                                // **Mover o widget com TODOS os estados** (W7r): quem desloca é a
                                // shell — só ela vê o `Transform` andar —, então o toggle
                                // atravessa o barramento como o interruptor de preview ao lado.
                                pending_ui_move_all_toggle = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_STATE_PREVIEW {
                                // **O MODO DE PREVIEW** (W7r): ele NÃO é um verbo de estado — não
                                // toca a tabela —, então tem rota própria em vez de um variant no
                                // `UiStateEdit`, cujo assunto é *o que muda no documento*.
                                pending_ui_preview_toggle = true;
                            } else if let Some(e) = crate::vec_widget_edit::widget_edit_for_id(*id)
                            {
                                // A PELE por-widget (plano UI/UX W6.2): o componente mora no ECS,
                                // entao o clique e' da shell — o painel so' mostra que tipo esta'
                                // aceso e que verbo faz sentido.
                                pending_widget_edit = Some(e);
                            } else if let Some(e) = crate::vec_anchor_edit::anchor_edit_for_id(*id)
                            {
                                // AS ÂNCORAS (plano UI/UX W3): o par de âncoras mora no
                                // COMPONENTE, e a RÉGUA e' capturada do lado da shell — que e'
                                // quem mede a moldura. O painel so' mostra qual chip esta' aceso.
                                pending_anchor_edit = Some(e);
                            } else if let Some(choice) = crate::vec_bindings::token_choice(*id) {
                                // Uma escolha do picker de token. O valor mora no COMPONENTE
                                // (mundo), então o clique é da shell — o painel só mostra.
                                pending_token_bind = Some(choice);
                            } else if let Some(p) = ph2d_tool_vector::frames::device_preset(*id) {
                                // Um preset é uma 2ª forma de PEDIR a edição de W/H — ele cai na
                                // MESMA porta que os campos numéricos do Transform.
                                pending_frame_preset = Some(p);
                            } else if *id == ph2d_editor_core::ids::VECTOR_MORPH_PREVIEW {
                                // ⭐⭐ **O MODO em que o teclado é da máquina** (plano 32 W9). Ele
                                // NÃO é um verbo de seta — não toca o grafo —, então tem rota
                                // própria em vez de um variant no `MorphCmd`, cujo assunto é *o que
                                // muda no documento*. É a mesma separação do irmão das poses.
                                pending_morph_preview_toggle = true;
                            } else if let Some(cmd) = crate::vec_morph_edit::morph_cmd_for_id(*id) {
                                // ⭐ A seção MORPH STATES (plano 32 W4/W8): fazer o conjunto, ou
                                // escolher a acção que dispara uma transição. As duas mexem no
                                // MUNDO, então o clique é da shell — o painel só mostra.
                                pending_morph_arrow = Some(cmd);
                            } else if *id == ph2d_editor_core::ids::VECTOR_BOOL_APPLY {
                                pending_bool_apply = true;
                            } else if let Some(op) = crate::input_dispatch::vec_bool_op_for_id(*id)
                            {
                                pending_vec_bool = Some(op);
                            } else if let Some(cmd) = crate::vec_expand::expand_for_id(*id) {
                                pending_vec_expand = Some(cmd);
                            } else if let Some(kind) =
                                crate::input_dispatch::vec_vertex_kind_for_id(*id)
                            {
                                pending_vec_vertex_kind = Some(kind);
                            } else if *id == ph2d_editor_core::ids::VECTOR_VERT_DELETE {
                                pending_vec_delete_vertex = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_VERT_SEL_SUBPATH {
                                pending_vec_select_subpath = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_VERT_SEL_SAME {
                                pending_vec_select_same = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_PATH_JOIN {
                                pending_vec_join = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_PATH_WELD {
                                pending_vec_weld = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_PATH_REVERSE {
                                pending_vec_reverse = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_VERT_AVERAGE {
                                pending_vec_average = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_CUT_APPLY {
                                pending_vec_cut = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_SYM_APPLY {
                                pending_vec_symmetry_apply = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_CUT_DISCARD {
                                pending_vec_cut_discard = true;
                            } else if let Some(order) =
                                crate::input_dispatch::vec_reorder_for_id(*id)
                            {
                                pending_vec_reorder = Some(order);
                            } else if *id == ph2d_editor_core::ids::VECTOR_ARRANGE_DUPLICATE {
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
                            } else if *id == ph2d_editor_core::ids::VECTOR_PIVOT_EDIT {
                                pending_vec_pivot_edit = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_PATH_CLOSE {
                                pending_vec_toggle_closed = true;
                            } else if let Some(k) = crate::input_dispatch::vec_fill_kind_for_id(*id)
                            {
                                pending_vec_fill_kind = Some(k);
                            } else if *id == ph2d_editor_core::ids::VECTOR_BRUSH_PICK_SHAPE {
                                // ⭐ Arma; a FONTE (a forma com o pincel) é capturada no drain,
                                // porque o clique seguinte muda a seleção.
                                pending_brush_pick = true;
                            } else if let Some(c) = crate::vec_stroke_paint::cmd_for_id(*id) {
                                pending_brush = Some(c);
                            } else if let Some(k) = crate::vec_stroke_paint::kind_for_id(*id) {
                                // ⭐ A TINTA do traço (plano 35, wave D). Ela mexe no DOCUMENTO,
                                // entao o clique e' da shell — o painel so' mostra qual chip acende.
                                pending_vec_stroke_kind = Some(k);
                            } else if let Some((slot, knob)) =
                                ph2d_panel_vector::texture_pattern::texpat_knob_of(*id)
                            {
                                // ⭐⭐ **O SUJEITO VEM NO PRÓPRIO ID** (plano 35, wave F). Cada
                                // secção tem os seus controlos, então o clique já **diz** em qual
                                // das duas tintas escrever — e a preferência de sessão que a wave D
                                // precisava (`texpat_target`) deixou de existir, com a classe
                                // inteira de *"mexi num knob e mudou o outro sujeito"*.
                                use ph2d_editor_core::ids::TexPatKnob as K;
                                let slot = if slot == 1 {
                                    ph2d_vec_render::PatternSlot::Stroke
                                } else {
                                    ph2d_vec_render::PatternSlot::Fill
                                };
                                match knob {
                                    K::Tile(i) => {
                                        pending_texpat = Some((
                                            slot,
                                            crate::texture_pattern_edit::TexPatCmd::Tile(i),
                                        ));
                                    }
                                    K::Mode(i) => {
                                        pending_texpat = Some((
                                            slot,
                                            crate::texture_pattern_edit::TexPatCmd::Mode(i),
                                        ));
                                    }
                                    K::Source => pending_texpat_source = Some(slot),
                                    // Picker (W7): arma; a FONTE (a forma com o padrão) é capturada
                                    // no drain, porque o clique seguinte muda a seleção.
                                    K::PickShape => pending_texpat_pick = Some(slot),
                                    // ⭐ O CADEADO é estado de SESSÃO (o gesto, não o padrão): o
                                    // clique inverte-o aqui e nada toca no documento.
                                    // ⚠️ Indexado pelo SLOT: as duas tintas têm cadeados
                                    // independentes, e partilhá-los era o defeito.
                                    K::Lock => {
                                        let i = usize::from(
                                            slot == ph2d_vec_render::PatternSlot::Stroke,
                                        );
                                        self.texpat_lock_aspect[i] = !self.texpat_lock_aspect[i];
                                    }
                                    // ⭐ O elo dos VÃOS — mesmo desenho, mesmo índice por slot.
                                    K::GapLink => {
                                        let i = usize::from(
                                            slot == ph2d_vec_render::PatternSlot::Stroke,
                                        );
                                        self.texpat_gap_link[i] = !self.texpat_gap_link[i];
                                    }
                                    _ => {}
                                }
                            } else if *id == ph2d_editor_core::ids::VECTOR_GRAD_ADD_POINT {
                                pending_vec_grad_add = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_GRAD_REMOVE_POINT {
                                pending_vec_grad_remove = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_GRAD_ADD_STOP {
                                pending_vec_grad_add_stop = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_GRAD_REMOVE_STOP {
                                pending_vec_grad_remove_stop = true;
                            } else if let Some(a) = crate::input_dispatch::vec_align_for_id(*id) {
                                pending_vec_align = Some(a);
                            } else if let Some(d) =
                                crate::input_dispatch::vec_distribute_for_id(*id)
                            {
                                pending_vec_distribute = Some(d);
                            } else if *id == ph2d_editor_core::ids::VECTOR_COMPOUND_MAKE {
                                pending_vec_compound = Some(true);
                            } else if *id == ph2d_editor_core::ids::VECTOR_COMPOUND_RELEASE {
                                pending_vec_compound = Some(false);
                            } else if *id == ph2d_editor_core::ids::VECTOR_FILL_RULE_NONZERO {
                                pending_vec_fill_rule = Some(false);
                            } else if *id == ph2d_editor_core::ids::VECTOR_FILL_RULE_EVENODD {
                                pending_vec_fill_rule = Some(true);
                            } else if *id == ph2d_editor_core::ids::VECTOR_SNAP_OFF {
                                pending_vec_snap_on = Some(false);
                            } else if *id == ph2d_editor_core::ids::VECTOR_SNAP_ON {
                                pending_vec_snap_on = Some(true);
                            } else if *id == ph2d_editor_core::ids::VECTOR_SNAP_PATH_OFF {
                                pending_vec_snap_path = Some(false);
                            } else if *id == ph2d_editor_core::ids::VECTOR_SNAP_PATH_ON {
                                pending_vec_snap_path = Some(true);
                            } else if *id == ph2d_editor_core::ids::VECTOR_SNAP_CROSS_OFF {
                                pending_vec_snap_cross = Some(false);
                            } else if *id == ph2d_editor_core::ids::VECTOR_SNAP_CROSS_ON {
                                pending_vec_snap_cross = Some(true);
                            } else if *id == ph2d_editor_core::ids::VECTOR_SNAP_GUIDES_OFF {
                                pending_vec_snap_guides = Some(false);
                            } else if *id == ph2d_editor_core::ids::VECTOR_SNAP_GUIDES_ON {
                                pending_vec_snap_guides = Some(true);
                            } else if *id == ph2d_editor_core::ids::VECTOR_RULERS_OFF {
                                pending_rulers = Some(false);
                            } else if *id == ph2d_editor_core::ids::VECTOR_RULERS_ON {
                                pending_rulers = Some(true);
                            } else if *id == ph2d_editor_core::ids::VECTOR_TEXT_FONT_PREV {
                                pending_vec_font_cycle = Some(-1);
                            } else if *id == ph2d_editor_core::ids::VECTOR_TEXT_FONT_NEXT {
                                pending_vec_font_cycle = Some(1);
                            } else if *id == ph2d_editor_core::ids::VECTOR_TEXT_FONT_IMPORT {
                                pending_vec_font_import = true;
                            } else if *id == ph2d_editor_core::ids::VECTOR_TEXT_ALIGN_LEFT {
                                pending_vec_text_align = Some(ph2d_vec_text::TextAlign::Left);
                            } else if *id == ph2d_editor_core::ids::VECTOR_TEXT_ALIGN_CENTER {
                                pending_vec_text_align = Some(ph2d_vec_text::TextAlign::Center);
                            } else if *id == ph2d_editor_core::ids::VECTOR_TEXT_ALIGN_RIGHT {
                                pending_vec_text_align = Some(ph2d_vec_text::TextAlign::Right);
                            } else if *id == ph2d_editor_core::ids::VECTOR_TEXT_WRAP_AUTO {
                                pending_vec_text_wrap = Some(None);
                            } else if *id == ph2d_editor_core::ids::VECTOR_TEXT_WRAP_FIXED {
                                // ⚠️ **Fixed semeia com a largura que o texto JÁ mede**, e não com
                                // um número de fábrica: clicar Fixed não pode mover um glifo — ele
                                // só torna o número editável. Sem sessão viva não há texto a medir,
                                // e aí cai no default do slider.
                                pending_vec_text_wrap = Some(Some(
                                    crate::vec_text::seed_wrap_width(self.vec.text_edit.as_ref())
                                        .unwrap_or(ph2d_tool_vector::params::DEFAULT_TEXT_WRAP),
                                ));
                            } else if *id == ph2d_editor_core::ids::VECTOR_CONVERT_TO_CURVES {
                                pending_vec_convert = true;
                            }
                        }
                        // Transform fields (X/Y/W/H) are numeric SetValue document
                        // commands (not tool Style) — capture; the tool ignores them.
                        if let ph2d_editor_core::tool::PanelEvent::SetValue(id, v) = &ev {
                            // ⭐ **Os CAMPOS entram pela mesma porta derivada que os cliques** — sem
                            // isto, digitar num campo desta seção sem osso em foco continuava a ser
                            // um silêncio sem explicação, que é metade da população da secção.
                            pending_bone_needs_focus |=
                                ph2d_editor_core::ids::needs_focused_bone(*id);
                            if let Some(field) =
                                crate::input_dispatch::vec_transform_field_for_id(*id)
                            {
                                pending_vec_transform = Some((field, *v));
                            } else if *id == ph2d_editor_core::ids::VECTOR_OBJ_OPACITY {
                                pending_vec_opacity = Some(*v);
                            } else if *id == ph2d_editor_core::ids::VECTOR_OBJ_BLEND {
                                // ⚠️ O valor é o CÓDIGO do modo (`BlendMode::to_u8`), e não a linha
                                // do popover: a lista é derivada da tradução para o Vello, e
                                // reconstruí-la aqui seria a segunda cópia dela.
                                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                                let code = v.clamp(0.0, f64::from(u8::MAX)) as u8;
                                pending_vec_blend = Some(code);
                            } else if *id == ph2d_editor_core::ids::VECTOR_PAINT_WIDTH {
                                pending_paint_width = Some(*v);
                            } else if *id == ph2d_editor_core::ids::VECTOR_PAINT_DX {
                                pending_paint_dx = Some(*v);
                            } else if *id == ph2d_editor_core::ids::VECTOR_PAINT_DY {
                                pending_paint_dy = Some(*v);
                            } else if *id == ph2d_editor_core::ids::VECTOR_PAINT_DILATE {
                                pending_paint_dilate = Some(*v);
                            } else if *id == ph2d_editor_core::ids::VECTOR_PAINT_OPACITY {
                                pending_paint_opacity = Some(*v);
                            } else if *id == ph2d_editor_core::ids::VECTOR_PAINT_BLEND {
                                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                                let code = v.clamp(0.0, f64::from(u8::MAX)) as u8;
                                pending_paint_blend = Some(code);
                            } else if *id == ph2d_editor_core::ids::VECTOR_BONE_LENGTH
                                || *id == ph2d_editor_core::ids::VECTOR_BONE_STRENGTH
                            {
                                // ⭐ Os dois números do OSSO (estudo 42 item 5). Eles vivem num
                                // componente da entidade, então quem escreve é a shell — a mesma
                                // rota dos campos do Transform e do layout.
                                pending_bone_knob =
                                    Some((*id == ph2d_editor_core::ids::VECTOR_BONE_STRENGTH, *v));
                            } else if *id == ph2d_editor_core::ids::VECTOR_BONE_IK_MIX {
                                pending_ik_knob = Some((IkKnob::Mix, *v));
                            } else if *id == ph2d_editor_core::ids::VECTOR_BONE_IK_SOFTNESS {
                                pending_ik_knob = Some((IkKnob::Softness, *v));
                            } else if *id == ph2d_editor_core::ids::VECTOR_BONE_IK_CHAIN {
                                pending_ik_knob = Some((IkKnob::Chain, *v));
                            } else if *id == ph2d_editor_core::ids::VECTOR_BONE_LIMIT_MIN
                                || *id == ph2d_editor_core::ids::VECTOR_BONE_LIMIT_MAX
                            {
                                pending_limit_knob =
                                    Some((*id == ph2d_editor_core::ids::VECTOR_BONE_LIMIT_MAX, *v));
                            } else if *id == ph2d_editor_core::ids::VECTOR_BONE_SMART_FROM
                                || *id == ph2d_editor_core::ids::VECTOR_BONE_SMART_TO
                            {
                                pending_smart_knob =
                                    Some((*id == ph2d_editor_core::ids::VECTOR_BONE_SMART_TO, *v));
                            } else if *id == ph2d_editor_core::ids::VECTOR_VERT_X {
                                pending_vec_vert = Some((false, *v));
                            } else if *id == ph2d_editor_core::ids::VECTOR_VERT_Y {
                                pending_vec_vert = Some((true, *v));
                            } else if *id == ph2d_editor_core::ids::VECTOR_STATE_DURATION {
                                // W7: o track `0..1` vira SEGUNDOS pela régua do modelo. A
                                // conversão mora aqui e não no painel porque o número autorado é
                                // do documento — o painel só o mostra.
                                pending_ui_state_duration =
                                    Some(*v * ph2d_ui_state::MAX_DURATION_S);
                            } else if *id == ph2d_editor_core::ids::VECTOR_STATE_STIFFNESS
                                || *id == ph2d_editor_core::ids::VECTOR_STATE_DAMPING
                            {
                                // W7m: o track `0..1` vira o número autorado pela régua AFIM do
                                // modelo — as duas não começam em zero, então o offset é parte da
                                // conversão. Ela mora aqui pela mesma razão da duração: o número
                                // é do documento, e o painel só o mostra.
                                let stiff = *id == ph2d_editor_core::ids::VECTOR_STATE_STIFFNESS;
                                let (lo, hi) = if stiff {
                                    (ph2d_ui_state::MIN_STIFFNESS, ph2d_ui_state::MAX_STIFFNESS)
                                } else {
                                    (ph2d_ui_state::MIN_DAMPING, ph2d_ui_state::MAX_DAMPING)
                                };
                                pending_ui_spring_knob = Some((stiff, lo + *v * (hi - lo)));
                            } else if *id == ph2d_editor_core::ids::VECTOR_ARRANGE_Z {
                                pending_vec_z = Some(*v);
                            } else if let Some(f) = crate::vec_layout_edit::layout_field_for_id(*id)
                            {
                                // Vao, recuo, Grow e Shrink — mesma rota dos campos do Transform:
                                // o valor mora no componente, entao quem escreve e' a shell.
                                pending_layout_field = Some((f, *v));
                            } else if *id == ph2d_editor_core::ids::VECTOR_TRANSFORM_R {
                                pending_vec_rotate_by = Some(*v);
                            } else if *id == ph2d_editor_core::ids::VECTOR_GRAD_ANGLE {
                                // Slider carries the track 0..1 → 0..360°.
                                pending_vec_grad_angle = Some(*v * 360.0);
                            } else if *id == ph2d_editor_core::ids::VECTOR_GRAD_INFLUENCE {
                                // Track 0..1 → influence 0..4.
                                pending_vec_grad_influence = Some(*v * 4.0);
                            } else if *id == ph2d_editor_core::ids::VECTOR_GRAD_JITTER {
                                // Track 0..1 → jitter 0..1 (already a fraction).
                                pending_vec_grad_jitter = Some(*v);
                            } else if *id == ph2d_editor_core::ids::VECTOR_TEXT_SIZE {
                                // Track 0..1 → glyph size (world units); shared mapping.
                                pending_vec_text_size =
                                    Some(ph2d_tool_vector::params::slider_to_text_size(*v as f32));
                            } else if *id == ph2d_editor_core::ids::VECTOR_TEXT_WEIGHT {
                                // Track 0..1 → font weight (wght); shared mapping.
                                pending_vec_text_weight = Some(
                                    ph2d_tool_vector::params::slider_to_text_weight(*v as f32)
                                        as f32,
                                );
                            } else if *id == ph2d_editor_core::ids::VECTOR_TEXT_LINE_HEIGHT {
                                // Track 0..1 → line height (× size); shared mapping.
                                pending_vec_text_line_height = Some(
                                    ph2d_tool_vector::params::slider_to_text_line_height(*v as f32),
                                );
                            } else if *id == ph2d_editor_core::ids::VECTOR_TEXT_WRAP_W {
                                // Track 0..1 -> largura de refluxo (mundo); shared mapping.
                                pending_vec_text_wrap = Some(Some(
                                    ph2d_tool_vector::params::slider_to_text_wrap(*v as f32),
                                ));
                            } else if *id == ph2d_editor_core::ids::VECTOR_TEXT_TRACKING {
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
                            } else if *id == ph2d_editor_core::ids::VECTOR_BLEND_STEPS {
                                // ADR-0128: arrastar Steps ajusta o blend selecionado AO VIVO.
                                pending_blend_steps =
                                    Some(ph2d_tool_vector::params::blend_steps_from_track(*v));
                            } else if *id == ph2d_editor_core::ids::VECTOR_TEXTPATH_OFFSET {
                                // Plano 22: FRAÇÃO do comprimento do caminho, ja' no dominio do
                                // documento (o painel nao converte -- track e valor coincidem).
                                pending_textpath_offset = Some(*v);
                            } else if let Some(c) =
                                crate::vec_stroke_paint::slider_cmd_for_id(*id, *v)
                            {
                                // ⭐ Os knobs do PINCEL (plano 36, W4). O `event.rs` do painel já
                                // converteu o track para o domínio do documento — aqui `*v` é valor.
                                pending_brush = Some(c);
                            } else if let Some((slot, knob)) =
                                ph2d_panel_vector::texture_pattern::texpat_knob_of(*id)
                            {
                                // ⭐⭐ **O SUJEITO VEM NO ID** (plano 35, wave F): cada secção tem os
                                // seus sliders, então arrastar um deles já diz em QUAL das duas
                                // tintas escrever. ⚠️ O `event.rs` do painel já converteu o track
                                // para o domínio do documento — aqui `*v` é valor.
                                use ph2d_editor_core::ids::TexPatKnob as K;
                                let alvo = if slot == 1 {
                                    ph2d_vec_render::PatternSlot::Stroke
                                } else {
                                    ph2d_vec_render::PatternSlot::Fill
                                };
                                // ⚠️ O cadeado é da TINTA que este controlo serve, e não do painel.
                                let cadeado = self.texpat_lock_aspect[slot.min(1)];
                                let cmd = match knob {
                                    // UM eixo do tamanho + o CADEADO da sessão, que decide se o
                                    // outro eixo vem junto.
                                    K::Width => Some(crate::texture_pattern_edit::TexPatCmd::Axis(
                                        0, *v, cadeado,
                                    )),
                                    K::Height => {
                                        Some(crate::texture_pattern_edit::TexPatCmd::Axis(
                                            1, *v, cadeado,
                                        ))
                                    }
                                    // ⭐ UM eixo do VÃO + o ELO da sessão, que decide se o outro
                                    // vem junto — o mesmo desenho do cadeado logo acima.
                                    K::Gap => Some(crate::texture_pattern_edit::TexPatCmd::Gap(
                                        0,
                                        *v,
                                        self.texpat_gap_link[slot.min(1)],
                                    )),
                                    K::GapY => Some(crate::texture_pattern_edit::TexPatCmd::Gap(
                                        1,
                                        *v,
                                        self.texpat_gap_link[slot.min(1)],
                                    )),
                                    // A FASE dentro de uma repetição, em %.
                                    K::ShiftX => {
                                        Some(crate::texture_pattern_edit::TexPatCmd::Shift(0, *v))
                                    }
                                    K::ShiftY => {
                                        Some(crate::texture_pattern_edit::TexPatCmd::Shift(1, *v))
                                    }
                                    // GRAUS aqui; o documento guarda radianos, e a conversão vive
                                    // na porta única (`texture_pattern_edit::apply`).
                                    K::Angle => {
                                        Some(crate::texture_pattern_edit::TexPatCmd::Angle(*v))
                                    }
                                    K::Offset => Some(
                                        crate::texture_pattern_edit::TexPatCmd::OffsetDenom(*v),
                                    ),
                                    _ => None,
                                };
                                pending_texpat = cmd.map(|c| (alvo, c));
                            } else if *id == ph2d_editor_core::ids::VECTOR_PATTERNPATH_SPACING {
                                // Plano 23: ja' convertido pelo event.rs do painel para o dominio do
                                // documento (multiplos da largura do motivo) -- aqui e' valor.
                                pending_pp_spacing = Some(*v);
                            } else if *id == ph2d_editor_core::ids::VECTOR_PATTERNPATH_START {
                                // FRAÇÃO do comprimento (track == valor, como o Offset do texto).
                                pending_pp_start = Some(*v);
                            } else if *id == ph2d_editor_core::ids::VECTOR_PATTERNPATH_END {
                                // FRAÇÃO do comprimento -- o fim do trecho `[Start, End]`.
                                pending_pp_end = Some(*v);
                            } else if *id == ph2d_editor_core::ids::VECTOR_PATTERNPATH_SLIDE {
                                // O CENTRO do trecho -- o drain re-centra a janela (move Start+End).
                                pending_pp_slide = Some(*v);
                            } else if *id == ph2d_editor_core::ids::VECTOR_PATTERNPATH_ROTATION {
                                // A ORIENTAÇÃO do motivo sobre a guia, em GRAUS -- o event.rs do painel
                                // ja' converteu o track bipolar (`-180..180`); aqui e' valor.
                                pending_pp_rotation = Some(*v);
                            } else if *id == ph2d_editor_core::ids::VECTOR_PATTERNPATH_OFFSET {
                                // Desvio perpendicular (unidades de mundo), ja' bipolar (`-2..2`)
                                // convertido pelo event.rs do painel -- aqui e' valor.
                                pending_pp_offset = Some(*v);
                            } else if *id == ph2d_editor_core::ids::VECTOR_CONTOUR_STEPS {
                                // Quantos aneis -- o `event.rs` do painel ja arredondou ao inteiro.
                                pending_contour_steps = Some(*v);
                            } else if *id == ph2d_editor_core::ids::VECTOR_CONTOUR_OFFSET {
                                // A distancia POR PASSO, em FRACAO do tamanho da forma: o painel
                                // fala fracao (um rotulo em unidades de mundo mentiria a cada troca
                                // de selecao) e o componente guarda MUNDO. A conversao e' do `arm`
                                // e do drain, com a MESMA `offset_scale` que o Offset usa.
                                pending_contour_d = Some(*v);
                            } else if *id == ph2d_editor_core::ids::VECTOR_CONTOUR_ACCEL {
                                // A aceleracao da progressao -- o painel ja aplicou o mapa
                                // GEOMETRICO do trilho; aqui e' valor.
                                pending_contour_accel = Some(*v);
                            } else if let Some(hit) = crate::fx_live::hit_of(*id) {
                                pending_filter_val = Some((hit, *v));
                            } else if *id == ph2d_editor_core::ids::VECTOR_ENVELOPE_BEND {
                                // ADR-0129 Fatia C: o `event.rs` do painel ja converteu o track
                                // bipolar para o dominio do documento (`-1..1`) -- aqui e' valor.
                                pending_envelope_bend = Some(*v);
                            } else if let Some((r, prm)) =
                                crate::fx_bridge_dispatch::classify_param(*id)
                            {
                                // O painel entrega o TRACK normalizado; a faixa real e' do
                                // efeito e a ponte a aplica.
                                pending_fx_param = Some((r, prm, *v));
                            } else if *id == ph2d_editor_core::ids::VECTOR_MORPH_T {
                                // Arrastar o `t` move a forma pelo caminho AO VIVO — e é assim que
                                // o artista a estaciona onde ela fica bem, antes do K.
                                #[allow(clippy::cast_possible_truncation)]
                                let t = *v as f32;
                                pending_morph_t = Some(t);
                            } else {
                                // Variation-axis field carries the axis VALUE directly
                                // (not a 0..1 track): match the slot to its font axis.
                                for i in 0..ph2d_editor_core::ids::MAX_TEXT_VARIATION_AXES {
                                    if *id == ph2d_editor_core::ids::vector_text_axis_id(i) {
                                        pending_vec_text_axis = Some((i, *v));
                                        break;
                                    }
                                }
                            }
                        }
                        // ⭐ **O NOME de uma ligação sinal → papel**: `SelectOption(campo,
                        // "<texto>")`. O texto é o que o artista digitou, e vem por este canal
                        // porque o `PanelEvent` é contrato CONGELADO — o `SelectOption` já é o
                        // canal string-valued deste app (o Painter carrega nele
                        // `"layer:channel:index:x:y"`, que não é opção de rádio nenhuma).
                        if let ph2d_editor_core::tool::PanelEvent::SelectOption(id, val) = &ev
                            && let Some(row) = crate::vec_ui_state_edit::signal_name_row(*id)
                        {
                            pending_ui_signal_name = Some((row, val.clone()));
                        }
                        // Font dropdown pick: `SelectOption(chip, "<index>")` → the
                        // family index into `vec_font::pickable_families()`.
                        if let ph2d_editor_core::tool::PanelEvent::SelectOption(id, val) = &ev
                            && *id == ph2d_editor_core::ids::VECTOR_TEXT_FONT_DD
                        {
                            pending_vec_font_pick = val.parse::<usize>().ok();
                        }
                        // O punho de um stop da rampa: `SelectOption(trilho, "linha:idx:x")` — o
                        // dispatch de 2D já converteu o ponteiro contra a barra, então o `x` chega
                        // normalizado. O formato espelha o do editor de falloff do Painter.
                        if let ph2d_editor_core::tool::PanelEvent::SelectOption(id, val) = &ev
                            && (0..ph2d_editor_core::ids::MAX_FILTER_ROWS)
                                .any(|r| *id == ph2d_editor_core::ids::filter_ramp_id(r))
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
                        if let ph2d_editor_core::tool::PanelEvent::Click(id) = &ev {
                            if *id == ph2d_editor_core::ids::FLIP_COLORIZE_APPLY {
                                self.flip_state.pending_colorize_apply = true;
                            } else if *id == ph2d_editor_core::ids::FLIP_COLORIZE_CLEAR {
                                self.flip_state.pending_colorize_clear = true;
                            }
                        }
                        // ADR-0114 W2: Flip layer ops (add/delete/select/visibility/
                        // lock/reorder/opacity/blend) are DOCUMENT edits — apply to
                        // `gfx.flip` + the active-layer pointer (mirror of the vector
                        // Boolean/Arrange capture). No-op for non-Flip ids. Still
                        // forward `ev` to the tool below (it ignores layer ids).
                        ph2d_app_flip::layers::apply_panel_event(
                            &ev,
                            flip,
                            &mut self.flip_state.active_layer,
                            &self.playhead,
                            matches!(
                                self.flip_state.style.map(|s| s.edit_domain),
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
                        ph2d_app_flip::strip::apply_panel_event(
                            &ev,
                            flip,
                            self.flip_state.active_layer,
                            &mut self.playhead,
                            &mut self.flip_state.strip,
                            self.modifiers.shift_key()
                                || self.modifiers.super_key()
                                || self.modifiers.control_key(),
                        );
                        if let Some(t) = tools.active_mut() {
                            // ⚠️ **Sob a mão, o GIZMO é o preview — e um arrasto de KNOB é uma mão sobre
                            // a figura tanto quanto um arrasto no canvas.** O edit abaixo é o que
                            // re-carimba, então o gesto é publicado ANTES dele; o `held_button` é a
                            // MESMA porta que o `post_frame_undo` consulta para *"um arrasto é UM
                            // passo"*. O `false` que ASSENTA vem do `painter_bridge::dispatch`, uma vez
                            // por quadro — aqui não há evento nenhum quando o artista solta.
                            if let Some(p) = t
                                .as_any_mut()
                                .downcast_mut::<ph2d_tool_painter::PainterTool>()
                            {
                                p.set_shape_draft_hold(self.held_button.is_some());
                            }
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
                        if let ph2d_editor_core::tool::PanelEvent::Click(id) = &ev
                            && let Some(prop) = timeline_bridge::prop_for_addprop_id(*id)
                        {
                            if let Some(entity) = hero.gizmo.iter_selected().next() {
                                self.timeline_intents
                                    .push(ph2d_timeline::TimelineIntent::Bind { entity, prop });
                            }
                        } else if let ph2d_editor_core::tool::PanelEvent::Toggle(id, on) = &ev
                            && *id == ph2d_editor_core::ids::TIMELINE_MOTION_PATH
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
                        } else if let ph2d_editor_core::tool::PanelEvent::Click(id) = &ev
                            && *id == ph2d_editor_core::ids::TIMELINE_ONION_SETTINGS
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
                        self.vec.pen.finish();
                        if let Some(default_id) = tools.default_tool_id()
                            && tools.set_active(&default_id)
                        {
                            self.last_bgremoval_pushed_entity = None;
                            self.bgremoval_preview = None;
                            self.title_dirty = true;
                        }
                    }
                    // O pill SCULPT (ADR-0150). ⚠️ **Um pedido, drenado no topo do frame
                    // SEGUINTE** — a mesma rota do `Shift+B` e do padrão do sprite, e pelo mesmo
                    // motivo, que aqui é mais forte: entrar pode ter de CRIAR a cena, e o `device`
                    // está emprestado neste ponto do laço.
                    EditorAction::ToggleSculpt3d => self.sculpt3d_req.toggle_request = true,
                    EditorAction::UndoImageEdit => undo_image_edit = true,
                    // Os botões Undo/Redo da barra: MESMO caminho do Ctrl+Z. O despacho
                    // espera o fim do frame (`post_frame_undo`) porque `undo_or_redo`
                    // precisa de `&mut self` e o `gfx` está emprestado aqui.
                    EditorAction::UndoStep { redo } => self.undo_button = Some(redo),
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::ToggleVisibility { row },
                    ) => {
                        visibility_toggle_row.get_or_insert(row);
                    }
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::ToggleLock { row },
                    ) => {
                        lock_toggle_row.get_or_insert(row);
                    }
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::ToggleGroup { row },
                    ) => {
                        group_toggle_row.get_or_insert(row);
                    }
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::Reparent(intent),
                    ) => {
                        reparent_intent.get_or_insert(intent);
                    }
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::Duplicate { row },
                    ) => {
                        duplicate_row.get_or_insert(row);
                    }
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::AddChild { row },
                    ) => {
                        add_child_row.get_or_insert(row);
                    }
                    EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::Group {
                        row,
                    }) => {
                        group_row.get_or_insert((row, true));
                    }
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::Ungroup { row },
                    ) => {
                        group_row.get_or_insert((row, false));
                    }
                    EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::AddRoot) => {
                        add_root = true;
                    }
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::ResetTransform { row },
                    ) => {
                        reset_transform_row.get_or_insert(row);
                    }
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::RevertToMaster { row },
                    ) => {
                        revert_to_master_row.get_or_insert(row);
                    }
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::MakeComponent { row },
                    ) => {
                        instance_verb_row
                            .get_or_insert((row, ph2d_app_components::instance_verbs::Verb::Make));
                    }
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::Instantiate { row },
                    ) => {
                        instance_verb_row
                            .get_or_insert((row, ph2d_app_components::instance_verbs::Verb::Place));
                    }
                    // ⭐⭐⭐ **ABRIR a receita desta cópia** — pelo MESMO dreno dos outros verbos,
                    // que é onde vivem a resolução do sujeito e a voz de cada recusa.
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::EditPrefab { row },
                    ) => {
                        instance_verb_row
                            .get_or_insert((row, ph2d_app_components::instance_verbs::Verb::Edit));
                    }
                    // ⭐ **O verbo de USAR do navegador de assets** (plano `docs/Components/07`,
                    // wave A7). ⚠️ O sujeito é o `StableId`, não uma `row`: o navegador não tem
                    // linhas, e uma receita está **escondida** da Hierarquia por construção — não
                    // há `row` que a endereçe. A resolução `StableId → Entity` acontece na fase
                    // da hierarquia, onde o `sim` está emprestado.
                    EditorAction::AssetInstantiate { stable_id, at } => {
                        instance_verb_stable_id.get_or_insert((
                            stable_id,
                            ph2d_app_components::instance_verbs::Verb::Place,
                            at,
                        ));
                    }
                    // ⭐⭐ **O menu do cartão** (etapa C). ⚠️ `get_or_insert`, como os irmãos: um
                    // quadro tem um gesto, e o menu fecha ao primeiro clique.
                    EditorAction::AssetCardVerb { asset, verb } => {
                        asset_card_verb.get_or_insert((asset, verb));
                    }
                    EditorAction::AssetCatalogVerb(v) => {
                        catalog_verbs.push(v);
                    }
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::InstantiateLinked { row },
                    ) => {
                        instance_verb_row.get_or_insert((
                            row,
                            ph2d_app_components::instance_verbs::Verb::PlaceLinked,
                        ));
                    }
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::Detach { row },
                    ) => {
                        instance_verb_row.get_or_insert((
                            row,
                            ph2d_app_components::instance_verbs::Verb::Detach,
                        ));
                    }
                    // ⭐⭐ *Remove from Library* pela linha da Hierarquia — o MESMO verbo do cartão,
                    // com o outro sujeito. Ele resolve a receita a partir de uma cópia
                    // (`instance_unmake::recipe_root_of`), que é o que torna esta porta útil.
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::RemoveFromLibrary { row },
                    ) => {
                        instance_verb_row.get_or_insert((
                            row,
                            ph2d_app_components::instance_verbs::Verb::Unmake,
                        ));
                    }
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::ApplyToMaster { row },
                    ) => {
                        instance_verb_row
                            .get_or_insert((row, ph2d_app_components::instance_verbs::Verb::Apply));
                    }
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::Delete { row },
                    ) => {
                        delete_row.get_or_insert(row);
                    }
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::MergeSprites { row },
                    ) => {
                        merge_sprites_row.get_or_insert(row);
                    }
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::MergeToLayers { row },
                    ) => {
                        merge_to_layers_row.get_or_insert(row);
                    }
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::PackSheet { row },
                    ) => {
                        pack_sheet_row.get_or_insert(row);
                    }
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::ArrangeSheet { row },
                    ) => {
                        arrange_sheet_row.get_or_insert(row);
                    }
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::BakeSheet { row },
                    ) => {
                        bake_sheet_row.get_or_insert(row);
                    }
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::ExportSheet { row },
                    ) => {
                        export_sheet_row.get_or_insert(row);
                    }
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::ExportImage { row },
                    ) => {
                        export_image_row.get_or_insert(row);
                    }
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::RemoveFromSheet { row },
                    ) => {
                        remove_from_sheet_row.get_or_insert(row);
                    }
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::UseAsBrushTexture { row },
                    ) => {
                        use_as_brush_texture_row.get_or_insert(row);
                    }
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::UseAsBrushShape { row },
                    ) => {
                        use_as_brush_shape_row.get_or_insert(row);
                    }
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::UseAsPaper { row },
                    ) => {
                        use_as_paper_row.get_or_insert(row);
                    }
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::UseAsGranulation { row },
                    ) => {
                        use_as_granulation_row.get_or_insert(row);
                    }
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::RowClick { row },
                    ) => {
                        hierarchy_row_click.get_or_insert(row);
                    }
                    // Fase 0e: multi-select-aware hierarchy click +
                    // shift-range. Collect into a single latest-wins
                    // intent — the dispatch resolves row → entity_bits
                    // and applies the matching `GizmoStateGroup`
                    // mutation. Range overrides Row when both arrive
                    // in the same frame (the user can only be in one
                    // selection-gesture at a time).
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::SelectRow { row, modifier },
                    ) if !matches!(
                        hierarchy_select_intent,
                        Some(hierarchy::HierarchySelectIntent::Range { .. })
                    ) =>
                    {
                        hierarchy_select_intent =
                            Some(hierarchy::HierarchySelectIntent::Row { row, modifier });
                    }
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::RangeSelect { row },
                    ) => {
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
                        ph2d_editor_core::action_bus::SelectModifier::Replace => {
                            hero.gizmo.replace_selection(Some(entity_bits));
                        }
                        ph2d_editor_core::action_bus::SelectModifier::Add => {
                            hero.gizmo.add_to_selection(entity_bits);
                        }
                        ph2d_editor_core::action_bus::SelectModifier::Toggle => {
                            hero.gizmo.toggle_in_selection(entity_bits);
                        }
                    },
                    EditorAction::ClearSelection => {
                        hero.gizmo.clear_all_selection();
                    }
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::RenameSeed { row },
                    ) => {
                        rename_seed_row.get_or_insert(row);
                    }
                    EditorAction::Hierarchy(
                        ph2d_editor_core::action_bus::HierRequest::RenameCommit { row, new_name },
                    ) if rename_commit.is_none() => {
                        rename_commit = Some((row, new_name));
                    }
                    EditorAction::SetViewFocus { kind } => {
                        view_focus_kind.get_or_insert(kind);
                    }
                    EditorAction::Reimport { entity_bits } => {
                        reimport_entity.get_or_insert(entity_bits);
                    }
                    EditorAction::InspectorSpritePrecisionChange {
                        entity_bits,
                        precision,
                    } => {
                        precision_request.get_or_insert((entity_bits, precision));
                    }
                    // **A SPRITE COMO FONTE DE LUZ** (plano `docs/Sprite_projeto/18` W8).
                    //
                    // ⚠️ **Zero REMOVE o componente**, e é o que faz o quadro voltar a ser
                    // byte-idêntico: uma sprite que não emite não tem por que carregar a linha no
                    // ficheiro nem uma entrada na varredura do passe. Mesmo caminho do
                    // `TextureFilter` — quem tem o `ComponentRegistry` é o shell.
                    EditorAction::InspectorSpriteEmissiveChange {
                        entity_bits,
                        intensity,
                    } => {
                        // BulkSelect fan-out, a mesma forma do `InspectorSpriteEdit` acima.
                        if inspector_selection.is_empty() {
                            emissive_edits.push((entity_bits, intensity));
                        } else {
                            for &t in &inspector_selection {
                                emissive_edits.push((t, intensity));
                            }
                        }
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
                        // BulkSelect fan-out, a mesma forma do `InspectorVisibilitySectionEdit`.
                        if inspector_selection.is_empty() {
                            visibility_edits.push((info.entity_bits, info.visible));
                        } else {
                            for &t in &inspector_selection {
                                visibility_edits.push((t, info.visible));
                            }
                        }
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
                    // §5 9-Slice. Espalha sobre a BulkSelect como as irmãs: uma caixa de diálogo
                    // e as suas variantes partilham a mesma moldura, e ter de repetir a borda em
                    // cada uma seria o gesto que esta seção existe para evitar.
                    EditorAction::InspectorSliceEdit { entity_bits, edit } => {
                        if inspector_selection.is_empty() {
                            slice_edits.push((entity_bits, edit));
                        } else {
                            for &t in &inspector_selection {
                                slice_edits.push((t, edit));
                            }
                        }
                    }
                    // §12 Sockets / Anchors. ⚠️ **NÃO espalha sobre a BulkSelect**, e isso é
                    // uma decisão: uma âncora é identificada pelo NOME, e o índice que a edição
                    // carrega só significa alguma coisa na lista da entidade primária. Espalhar
                    // por índice escreveria na âncora errada de todas as outras — pior que não
                    // espalhar. Fan-out por nome é trabalho para quando houver quem o peça.
                    EditorAction::InspectorAnchorEdit { entity_bits, edit } => {
                        anchor_edits.push((entity_bits, edit));
                    }
                    // §11 Animation. ⚠️ **NÃO espalha sobre a BulkSelect**, e pela MESMA razão da
                    // §12 acima: uma animação é identificada pelo NOME, e o índice que a edição
                    // carrega só significa alguma coisa na biblioteca da entidade primária.
                    EditorAction::InspectorAnimEdit { entity_bits, edit } => {
                        anim_edits.push((entity_bits, edit));
                    }
                    // ⭐ **A secção TIMERS.** ⚠️ **NÃO espalha sobre a BulkSelect**, pela MESMA
                    // razão das duas acima: o índice que a edição carrega só significa alguma
                    // coisa na lista da entidade primária, e espalhá-lo escreveria no timer
                    // errado de todas as outras.
                    EditorAction::InspectorTimerEdit { entity_bits, edit } => {
                        timer_edits.push((entity_bits, edit));
                    }
                    // ⭐ **A secção SIGNAL ACTIONS.** ⚠️ **NÃO espalha sobre a BulkSelect**,
                    // pela MESMA razão das irmãs: o índice só significa alguma coisa na lista
                    // da entidade primária.
                    EditorAction::InspectorActionEdit { entity_bits, edit } => {
                        action_edits.push((entity_bits, edit));
                    }
                    // ⭐ **A secção AUDIO** (TOP-20 #4). ⚠️ **NÃO espalha sobre a BulkSelect**,
                    // pela MESMA razão das irmãs — e aqui há uma segunda: duas das variantes
                    // (`Preview`/`StopPreview`) TOCAM, e espalhá-las faria um clique em `Preview`
                    // disparar N sons de uma vez.
                    EditorAction::InspectorAudioEdit { entity_bits, edit } => {
                        audio_edits.push((entity_bits, edit));
                    }
                    // ⭐ **A secção CAMERA** (TOP-20 #7). ⚠️ **NÃO espalha sobre a BulkSelect**,
                    // pela MESMA razão das irmãs — e aqui há uma segunda: o `Preview` é da VISTA,
                    // e espalhá-lo faria N objectos disputarem um interruptor que é um só.
                    EditorAction::InspectorCameraEdit { entity_bits, edit } => {
                        camera_edits.push((entity_bits, edit));
                    }
                    // ⭐ **O `+` do Inspector** (ADR-0166 / F3) — o painel PEDE e a shell abre,
                    // porque só ela sabe o tipo do objeto, o que ele já tem, e o que o registo
                    // sabe construir.
                    EditorAction::InspectorAddComponentRequested { entity_bits } => {
                        add_component_for = Some(entity_bits);
                    }
                    // ⭐ **Limpar as excepções SEM ALVO** (ADR-0164 / F5.3). Aplicado JÁ, e não
                    // adiado para um local: ele não precisa de nada que este ponto não tenha, e o
                    // `post_frame_undo` (que corre no fim) vê a mudança e regista o passo.
                    // ⭐⭐⭐ **ABRIR a receita que o cartão NOMEIA** (2026-09-07) — a quarta e
                    // última superfície da família. ⚠️ Pelo MESMO dreno dos outros três acessos,
                    // que é onde vivem a resolução do sujeito (`master_subject`) e a voz da recusa.
                    EditorAction::InspectorOpenPrefab { root_bits } => {
                        // ⚠️ **Aplicado JÁ, como os irmãos deste bloco** — ele não precisa de nada
                        // que este ponto não tenha: a lei de abrir é SELECCIONAR, e a porta
                        // (`instance_open`) é a mesma que os outros três acessos usam. ⛔ Deferi-lo
                        // para o dreno dos verbos pediria um terceiro canal (bits, a par de `row` e
                        // `stable_id`) para um verbo que não toca no documento.
                        let mut select_out = None;
                        ph2d_app_components::instance_open::open_prefab(
                            sim,
                            ph2d_ecs::Entity::from_bits(root_bits),
                            toasts,
                            &mut select_out,
                        );
                        if let Some(bits) = select_out {
                            hero.gizmo.replace_selection(Some(bits));
                        }
                    }
                    EditorAction::InspectorClearUnusedOverrides { root_bits } => {
                        let n = inspector_instance::clear_orphans(sim, root_bits);
                        if n > 0 {
                            toasts.push(ph2d_editor_core::Toast::success(format!(
                                "Cleared {n} unused override(s)"
                            )));
                        }
                    }
                    // ⭐⭐⭐ **Largar UMA** (F5.3-ter) — o `✕` da linha. ⚠️ Aplicado JÁ, como o irmão
                    // acima e pela mesma razão: ele não precisa de nada que este ponto não tenha, e
                    // o `post_frame_undo` vê a mudança e regista o passo.
                    // ⭐⭐⭐ **Devolver uma peça recusada** (F5.10). ⚠️ Ela só apaga a DECISÃO — quem
                    // materializa a peça, lhe traz os bytes da receita e exuma a excepção que o
                    // artista tinha nela é o passe estrutural, no quadro seguinte.
                    EditorAction::InspectorRestoreRemovedPiece { root_bits, piece } => {
                        if ph2d_app_components::instance_structure::restore_piece(
                            sim, root_bits, piece,
                        ) {
                            toasts.push(ph2d_editor_core::Toast::success(
                                "Put the piece back \u{2014} it returns as the component has it",
                            ));
                        }
                    }
                    EditorAction::InspectorDropUnusedOverride {
                        root_bits,
                        piece,
                        type_id,
                    } => {
                        if inspector_instance::drop_orphan(sim, root_bits, piece, type_id) {
                            toasts.push(ph2d_editor_core::Toast::success(
                                "Dropped 1 unused override",
                            ));
                        }
                    }
                    // ⭐⭐⭐ **Trocar a VARIANTE** (ADR-0164 / F5, critério 2).
                    //
                    // ⚠️ **ADIADO para depois do dreno**, ao contrário do irmão acima, e a razão é
                    // uma só: a troca precisa do **eco** (`self.instance_echo`) para o esquecer, e
                    // aqui dentro o `self` já está emprestado. *Um gesto que precisa de mais do que
                    // o ponto de aplicação tem, adia-se — não se duplica o estado.*
                    // ⭐⭐ **Mostrar a biblioteca** — o clique na ranhura da textura. ⚠️ Ele
                    // **abre**, nunca alterna: o gesto é *«mostra-me o que cabe aqui»*, e
                    // fechar um painel que o artista acabou de pedir seria responder ao
                    // contrário.
                    EditorAction::OpenAssetBrowser => {
                        open_asset_browser = true;
                    }
                    // ⭐⭐⭐ **Aplicar uma peça ACRESCENTADA** (F5.11). ⚠️ **ADIADO como o irmão
                    // abaixo, e pela mesma família de razões:** ela precisa do registo de
                    // componentes e dos documentos possuídos (a peça pode ser uma forma vetorial),
                    // e aqui dentro o `self` já está emprestado.
                    EditorAction::InspectorApplyAddedPiece { piece } => {
                        apply_added = Some(piece);
                    }
                    EditorAction::InspectorSwapVariant { root_bits, master } => {
                        swap_variant = Some((root_bits, master));
                    }
                    EditorAction::InspectorApplyToLevel {
                        entity_bits,
                        master,
                    } => {
                        apply_to_level = Some((entity_bits, master));
                    }
                    // ⭐⭐⭐ **Renomear o VALOR de uma propriedade** (report do Enio, 2026-08-31).
                    // ⚠️ O sujeito é a RECEITA; o gesto nasce sobre a cópia, que é onde o artista
                    // está a olhar. Ver `ph2d-panel-inspector/src/event_value.rs`.
                    // ⭐⭐⭐ **GRAVAR A VARIAÇÃO** (Enio, 2026-09-01) — o botão do cartão.
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
                        if matches!(edit, ph2d_editor_core::PhysicsFieldEdit::Join) {
                            // ⚠️ **2 ou MAIS** (W-J4): três corpos marcados
                            // fazem uma CORRENTE de N−1 joints, na ordem da
                            // seleção. Não é fan-out (isso criaria um joint por
                            // corpo, entre os mesmos dois) — é UMA operação
                            // sobre a sequência, que a `join_selected_chain`
                            // executa depois do laço.
                            if inspector_selection.len() >= 2 {
                                join_chain = true;
                            }
                        } else if matches!(edit, ph2d_editor_core::PhysicsFieldEdit::Rig) {
                            // ⚠️ **Nem o Rig faz fan-out** (W-Rig), e a razão é a
                            // do Bake mais que a do Join: cada corrida do gerador
                            // percorre a MESMA subárvore, então espalhado ele
                            // rodaria N vezes sobre o mesmo trabalho — a 2ª em
                            // diante achariam tudo já ligado e não fariam nada,
                            // mas o toast contaria a 1ª N vezes.
                            rig_now = true;
                        } else if matches!(edit, ph2d_editor_core::PhysicsFieldEdit::JoinDraw) {
                            // ARMA o gesto de canvas (sem operando, como os
                            // eyedroppers do §12): quem nomeia os dois corpos é
                            // o press e o release, não a seleção. Armado aqui e
                            // honrado no `input_dispatch`.
                            join_draw_arm = true;
                        } else if matches!(edit, ph2d_editor_core::PhysicsFieldEdit::Bake) {
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
                        } else if let ph2d_editor_core::PhysicsFieldEdit::BakeChannels(tag) = edit {
                            // A GLOBAL bake option, not a per-body edit (like
                            // Bake itself): it says how the NEXT bake behaves.
                            // No fan-out, no Collider write — just the app state
                            // the Bake button reads.
                            self.bake_channels =
                                ph2d_app_physics::bake::BakeChannels::from_tag(tag);
                        } else if let ph2d_editor_core::PhysicsFieldEdit::JoinKind(tag) = edit {
                            // The pending join KIND, the same class as BakeChannels:
                            // an app-state option the Join gesture reads, not a
                            // per-body edit. No fan-out, no Collider write.
                            self.physics.join_kind = tag;
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
                            ph2d_editor_core::JointFieldEdit::PickBodyA => {
                                self.joint_body_pick = Some((entity_bits, false));
                            }
                            ph2d_editor_core::JointFieldEdit::PickBodyB => {
                                self.joint_body_pick = Some((entity_bits, true));
                            }
                            // ⚠️ **O ÚNICO fan-out da §12** (W-JointCopy). O
                            // resto da seção descreve UM joint e edita UM; um
                            // paste existe para carimbar o rig inteiro, e sem
                            // isto o gesto é *digitar quinze campos, dez vezes*.
                            // Espalhado sobre a seleção crua: quem não for joint
                            // cai no early-return de `paste_joint_properties`,
                            // do mesmo jeito que o fan-out do §11 atravessa
                            // entidades sem `Collider`.
                            ph2d_editor_core::JointFieldEdit::PasteProperties
                                if !inspector_selection.is_empty() =>
                            {
                                for &t in &inspector_selection {
                                    joint_edits.push((t, edit));
                                }
                            }
                            _ => joint_edits.push((entity_bits, edit)),
                        }
                    }
                    // §14 Platform Player. Sem fan-out, e pela razão da §12: a
                    // seção descreve UM personagem, o que está selecionado.
                    EditorAction::InspectorPlayerEdit { entity_bits, edit } => {
                        // ⚠️ **O `ClearRun` é o único verbo da §14 que não é uma
                        // escrita de componente** (W17): a fita de entrada mora na
                        // shell, então ele é honrado AQUI, onde o `self` é
                        // mutável — o lugar e a razão exatos do `Join` da §11 e do
                        // eyedropper da §12.
                        //
                        // ⚠️ E interceptar não é higiene: descartar é idempotente,
                        // então espalhá-lo pela seleção não corromperia nada HOJE.
                        // É precisamente essa forma que apodrece — o Ctrl+V do
                        // editor de nós colava duas vezes porque um dispatch
                        // duplicado "nunca tinha importado enquanto todos os
                        // verbos eram idempotentes".
                        //
                        // ⚠️ **E a troca em si mora numa PORTA** (`run_stash`,
                        // W25), porque o painel de MUNDO é uma segunda VISTA da
                        // mesma corrida: duas cópias do `mem::take` fariam a
                        // mesma coisa hoje e divergiriam no dia em que o
                        // descarte ganhar um caso especial.
                        if matches!(edit, ph2d_editor_core::PlayerFieldEdit::ClearRun) {
                            // ⚠️ **Descartar GUARDA** (W24): a corrida sai do
                            // documento e fica na sessão, porque o clique era
                            // irreversível — a fita não é `ProjectState`, então
                            // sem isto o único caminho de volta era reabrir o
                            // arquivo.
                            ph2d_app_physics::run_stash::apply(
                                ph2d_app_physics::run_stash::RunVerb::Discard,
                                &mut self.player_tape,
                                &mut self.discarded_run,
                            );
                        } else if matches!(edit, ph2d_editor_core::PlayerFieldEdit::RestoreRun) {
                            ph2d_app_physics::run_stash::apply(
                                ph2d_app_physics::run_stash::RunVerb::Restore,
                                &mut self.player_tape,
                                &mut self.discarded_run,
                            );
                        } else {
                            player_edits.push((entity_bits, edit));
                        }
                    }
                    EditorAction::InspectorWheelEdit { entity_bits, edit } => {
                        // W3: o eyedropper ARMA aqui (onde `self` é mutável), como
                        // o do joint e pela mesma razão — o pick é estado da
                        // shell, não uma escrita de componente.
                        if matches!(edit, ph2d_editor_core::WheelFieldEdit::PickMountBody) {
                            self.wheel_body_pick = Some(entity_bits);
                        } else if matches!(edit, ph2d_editor_core::WheelFieldEdit::PickRope) {
                            // W1: o mesmo lugar e a mesma razão — o pick é estado
                            // da shell. O alvo é a ROTA, resolvido no Down.
                            self.wheel_rope_pick = Some(entity_bits);
                        } else {
                            wheel_edits.push((entity_bits, edit));
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
                    EditorAction::InspectorSignalEdit(info) => {
                        // Mesma coalescência: um `TextChanged` por tecla, e só a
                        // última do quadro vira comando (W-Signal).
                        signal_edit = Some(info);
                    }
                    EditorAction::InspectorSignalLeaveEdit(info) => {
                        // O gêmeo (W-SignalLeave), com slot PRÓPRIO: coalescer os
                        // dois no mesmo faria a última tecla de uma row apagar o
                        // que a outra tinha acabado de dizer.
                        signal_leave_edit = Some(info);
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
                        ph2d_transport::apply(cmd, &mut self.playhead);
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
                let activating_cluster: Option<&'static str> =
                    ph2d_editor_core::installed_registry().and_then(|reg| {
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
                let already_active = tools.active().map(ph2d_editor_core::Tool::id)
                    == Some(ph2d_editor_core::ToolId::new(tool_id));
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
                } else if gate_on && tools.set_active(&ph2d_editor_core::ToolId::new(tool_id)) {
                    // **ENTRAR NO PAINTER COLAPSA A SELEÇÃO À ÚLTIMA** (Enio, 2026-08-19: *"se o
                    // usuário estiver com múltiplas imagens selecionadas e entrar no painter,
                    // selecione a última selecionada e desselecione as outras antes de entrar"*).
                    //
                    // ⚠️ **Antes de entrar, e não depois:** o Painter lê a seleção ao ativar-se
                    // para saber que documento abrir. Colapsar depois deixá-lo-ia um quadro com o
                    // estado que a trava existe para impedir — e um quadro chega para ele ligar a
                    // prévia à sprite errada.
                    if tool_id == "painter" {
                        let dropped = ph2d_app_painter::painter_lock::collapse_to_last(hero);
                        if dropped > 0 {
                            toasts.push(Toast::info(format!(
                                "Painter: kept the last selected sprite ({dropped} deselected)"
                            )));
                        }
                    }
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
            // ⛔⛔ **O espelho passou a servir TODA ferramenta, e a lista à mão morreu.** Ele
            // internava contra um `match` de **um** literal (`"painter"`) e filtrava por
            // `mode_on` — o que era verdade enquanto o único leitor era o trilho do Painter.
            // Deixou de ser em 2026-08-30: os toggles de `vector`/`motion`/`flip` precisam de
            // saber se a ferramenta DELES está activa para escolher entre activar e cancelar, e
            // com o espelho cego eles liam sempre *«não está»* — o segundo clique reactivava.
            //
            // ⭐ A internagem vem do **registry**: os `manifest.id` já são `&'static str`, então
            // procurar o manifesto cujo id bate com o id vivo devolve o `&'static` certo sem
            // alocar e **sem lista escrita à mão** — uma ferramenta nova entra sozinha.
            //
            // ⚠️ **E o filtro `mode_on` saiu**: ele pertence a quem pergunta pelo Painter, e
            // `offers::rail_shows_painter_tools` já o exige (`mode_on && == Some("painter")`).
            // Aqui ele apagava a resposta para as ferramentas que não são de imagem.
            let live = tools.active().map(|t| t.id());
            hero.image_edit.active_tool_id =
                crate::active_tool_mirror::intern_active_tool(live.as_ref().map(|i| i.0.as_str()));
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
                if let Some(reg) = ph2d_editor_core::installed_registry() {
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
                            if let Some(ph2d_editor_core::InteractiveState::Button { state }) =
                                hero.store.get_mut(pill_id)
                            {
                                use ph2d_editor_core::widget::ButtonState;
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
                let painter_id = ph2d_editor_core::ToolId::new("painter");
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
                            toasts,
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
                            toasts,
                        );
                        (painter as &mut dyn ph2d_editor_core::tool::RasterEditTool).deactivate();
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
            let painter_apply_committed = ph2d_app_painter::painter_bridge::dispatch(
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
                &mut self.donated_form,
                toasts,
                self.held_button.is_some(),
                crate::input_dispatch::fill_drag::fill_drag_armed(),
                // O funil de leitura de textura desta shell, entregue como fecho: a crate da
                // família não conhece o `texture_edit` nem o `SourceRead` dele.
                |entity, sim, renderer, asset_db, atlas_asset_map| {
                    crate::hero_intents::texture_edit::read_sprite_source(
                        entity,
                        sim,
                        renderer,
                        asset_db,
                        atlas_asset_map,
                    )
                    .map(|src| {
                        let straight = src.image.into_straight();
                        (straight.pixels, straight.width, straight.height)
                    })
                },
                &note_preview_px,
            );
            // Live-preview a non-selected sprite used as the brush Shape (so its opacity/blend remote-
            // control edits show in real time), into a SECOND preview slot/override.
            ph2d_app_painter::painter_bridge_shape_preview::drive_shape_source_preview(
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
                .is_some_and(|t| t.id() == ph2d_editor_core::ToolId::new("vector"));
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
                let xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
                // Os passos vêm do slider do painel — a fonte da verdade é o widget, não uma
                // cópia no shell (uma cópia driftaria do que o artista está VENDO).
                let steps = hero
                    .store
                    .slider(ph2d_editor_core::ids::VECTOR_BLEND_STEPS)
                    .map_or(ph2d_tool_vector::params::BLEND_STEPS_DEFAULT, |(_, v)| {
                        ph2d_tool_vector::params::blend_steps_from_track(f64::from(v))
                    });
                // A ORDEM da cadeia: no modo Pick Shapes, a de CLIQUE (a lista escolhida a dedo);
                // fora dele, a de z da seleção (ADR-0128 C2b). O Pick é o "escolher a ordem" do Enio.
                let picking = self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::PickBlend;
                let sources = if picking && self.vec.blend_picks.len() >= 2 {
                    self.vec.blend_picks.clone()
                } else {
                    crate::blend_live::selected_closed_in_z(vec_scene, &self.vec.pen)
                };
                if let Some((spine, blend)) =
                    crate::blend_live::create(vec_scene, &xf, &sources, steps)
                {
                    self.vec.pen.select_many(&[spine]);
                    self.vec.blend_pending = Some((spine, blend));
                    self.vec.blend_picks.clear();
                    // Feito o blend, volta ao Select — o objeto novo está selecionado e o gizmo
                    // manda (o modo Pick já cumpriu o papel de juntar a lista). Inline do
                    // `vec_set_draw_mode` (que re-borrowaria o `gfx` já destructurado): a tool é a
                    // dona do modo, `vec_draw_config` é o espelho lido no mesmo frame.
                    crate::render_loop::vector_bridge::set_mode(
                        tools,
                        ph2d_tool_vector::DrawMode::Select,
                    );
                    self.vec.draw_config.mode = ph2d_tool_vector::DrawMode::Select;
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
                let picking = self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::PickBlend;
                let sources = if picking && self.vec.blend_picks.len() >= 2 {
                    self.vec.blend_picks.clone()
                } else {
                    crate::blend_live::selected_closed_in_z(vec_scene, &self.vec.pen)
                };
                // DUAS, e exatamente duas: o morph é um `t` sobre UM par. Uma cadeia de 3+ formas
                // é o Blend — e recusar aqui em voz alta é melhor do que morfar as duas primeiras
                // e deixar o artista a descobrir sozinho quais foram escolhidas.
                if let [a, b] = sources[..] {
                    let (id, morph) = crate::morph_live::create(vec_scene, a, b);
                    self.vec.pen.select_many(&[id]);
                    self.vec.morph_pending = Some((id, morph));
                    self.vec.blend_picks.clear();
                    crate::render_loop::vector_bridge::set_mode(
                        tools,
                        ph2d_tool_vector::DrawMode::Select,
                    );
                    self.vec.draw_config.mode = ph2d_tool_vector::DrawMode::Select;
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
                for id in self.vec.pen.selected_paths() {
                    let Some(&bits) = self.vec.entities.get(id) else {
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
                let xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
                let runs = crate::blend_live::expand(
                    sim,
                    vec_scene,
                    &self.vec.entities,
                    &xf,
                    &mut self.vec.pen,
                );
                if runs.is_empty() {
                    eprintln!("[ph2d-vec] blend: selecione um blend (a linha, ou uma forma dele)");
                } else {
                    let n: usize = runs.iter().map(Vec::len).sum();
                    eprintln!("[ph2d-vec] blend: expandido em {n} forma(s)");
                    self.vec.restack.extend(runs);
                }
            }
            // ADR-0128 Fase D: **Release** — desfaz o blend (os passos somem, as fontes ficam).
            if pending_release_blend
                && crate::blend_live::release(sim, vec_scene, &self.vec.entities, &mut self.vec.pen)
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
                let sel = self.vec.pen.selected_paths().to_vec();
                crate::vec_text_ride::edit(sim, vec_scene, &self.vec.entities, &sel, |l| {
                    l.start_offset = v as f32;
                });
            }
            if let Some(cmd) = pending_textpath {
                let sel = self.vec.pen.selected_paths().to_vec();
                let done = match cmd {
                    crate::vec_text_ride::TextPathCmd::Link => {
                        crate::vec_text_ride::link(sim, vec_scene, &self.vec.entities, &sel)
                    }
                    crate::vec_text_ride::TextPathCmd::Detach => {
                        crate::vec_text_ride::detach(sim, vec_scene, &self.vec.entities, &sel)
                    }
                    crate::vec_text_ride::TextPathCmd::Flip(v) => {
                        crate::vec_text_ride::edit(sim, vec_scene, &self.vec.entities, &sel, |l| {
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
                let sel = self.vec.pen.selected_paths().to_vec();
                if let Some((text, _, _)) =
                    crate::vec_text_object::selected_text_object(sim, &self.vec.entities, &sel)
                {
                    self.vec.path_pick = Some(crate::vec_pick::PathPick::TextObject(text));
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
                let sel: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
                match pending_contour {
                    Some(crate::contour_live::ContourCmd::Add) => {
                        let xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
                        let scale = crate::vec_expand::offset_scale(vec_scene, &self.vec.pen, &xf);
                        let n = crate::contour_live::arm(sim, &self.vec.entities, &sel, scale);
                        eprintln!("[ph2d-vec] contour: armado em {n} forma(s)");
                    }
                    Some(crate::contour_live::ContourCmd::Remove) => {
                        let n = crate::contour_live::remove(sim, &self.vec.entities, &sel);
                        eprintln!("[ph2d-vec] contour: removido de {n} forma(s)");
                    }
                    Some(crate::contour_live::ContourCmd::Expand) => {
                        let xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
                        let runs =
                            self.contour_live
                                .expand(sim, vec_scene, &self.vec.entities, &xf, &sel);
                        if runs.is_empty() {
                            eprintln!(
                                "[ph2d-vec] contour: nada a expandir (selecione uma forma com contour)"
                            );
                        } else {
                            let n: usize = runs.iter().map(|r| r.len().saturating_sub(1)).sum();
                            eprintln!("[ph2d-vec] contour: expandido em {n} anel(is)");
                            self.vec.restack.extend(runs);
                        }
                    }
                    None => {}
                }
                if let Some(v) = pending_contour_steps {
                    let steps = v.max(1.0) as u16;
                    crate::contour_live::edit(sim, &self.vec.entities, &sel, |c| c.steps = steps);
                }
                if let Some(frac) = pending_contour_d {
                    // FRAÇÃO → MUNDO na fronteira, com a MESMA escala do `arm` e do Offset.
                    let xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
                    let d = frac * crate::vec_expand::offset_scale(vec_scene, &self.vec.pen, &xf);
                    crate::contour_live::edit(sim, &self.vec.entities, &sel, |c| c.d = d);
                }
                if let Some(v) = pending_contour_accel {
                    #[allow(clippy::cast_possible_truncation)]
                    let accel = v as f32;
                    crate::contour_live::edit(sim, &self.vec.entities, &sel, |c| c.accel = accel);
                }
                if let Some(code) = pending_contour_join {
                    crate::contour_live::edit(sim, &self.vec.entities, &sel, |c| c.join = code);
                }
                if let Some(code) = pending_contour_side {
                    crate::contour_live::edit(sim, &self.vec.entities, &sel, |c| c.side = code);
                }
                // A COR-ALVO vem do picker OKLCH partilhado, lido de volta como o Stroke e o Fill
                // fazem no `vector_bridge` — mas AQUI, porque o alvo da escrita é um componente
                // ECS e este é o bloco que tem `sim` e o mapa em mãos. A swatch é marcada como
                // picker-swatch no `paint.rs` do painel; o Down abre o picker por dispatch
                // genérico, e o que chega cá é só a escolha.
                if hero.store.picker_target() == Some(ph2d_editor_core::ids::VECTOR_CONTOUR_TO)
                    && let Some((value, _, _, _)) = hero
                        .store
                        .blender_picker(ph2d_editor_core::ids::INSP_BLENDER_PICKER)
                {
                    let to = value.rgba;
                    crate::contour_live::edit(sim, &self.vec.entities, &sel, |c| c.to = to);
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
                let sel: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
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
                    crate::fx_live::edit(sim, &self.vec.entities, &sel, |f| {
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
                                match crate::fx_live::spec_of(sim, &self.vec.entities, *id) {
                                    Some(mut f) if f.has_room() => {
                                        f.ops.push(ph2d_ecs::FxOp::new(kind));
                                        crate::fx_live::set_filter(
                                            sim,
                                            &self.vec.entities,
                                            one,
                                            Some(f),
                                        );
                                    }
                                    Some(_) => {}
                                    None => {
                                        crate::fx_live::set_filter(
                                            sim,
                                            &self.vec.entities,
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
                            crate::fx_live::edit(sim, &self.vec.entities, &sel, |f| {
                                if row < f.ops.len() {
                                    f.ops.remove(row);
                                }
                            });
                        }
                        FilterHit::Up(row) => {
                            crate::fx_live::edit(sim, &self.vec.entities, &sel, |f| {
                                f.move_up(row);
                            });
                        }
                        FilterHit::Down(row) => {
                            crate::fx_live::edit(sim, &self.vec.entities, &sel, |f| {
                                f.move_down(row);
                            });
                        }
                        FilterHit::Hide(row) => {
                            crate::fx_live::edit(sim, &self.vec.entities, &sel, |f| {
                                if let Some(op) = f.ops.get_mut(row) {
                                    op.enabled = !op.enabled;
                                }
                            });
                        }
                        // O MODO é a LEI do degrau, não a intensidade dele — e um clique num modo
                        // que o TIPO não oferece é recusado aqui (o painel não o pinta, mas a
                        // recusa mora onde o valor é escrito).
                        FilterHit::Mode(row, mode) => {
                            crate::fx_live::edit(sim, &self.vec.entities, &sel, |f| {
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
                            crate::fx_live::edit(sim, &self.vec.entities, &sel, |f| {
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
                            crate::fx_live::edit(sim, &self.vec.entities, &sel, |f| {
                                if let Some(op) = f.ops.get_mut(row) {
                                    crate::fx_live::add_stop(op);
                                }
                            });
                        }
                        FilterHit::StopRemove(row) => {
                            let sel_stop = usize::from(ph2d_panel_vector::selected_stop(row));
                            crate::fx_live::edit(sim, &self.vec.entities, &sel, |f| {
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
                    crate::fx_live::edit(sim, &self.vec.entities, &sel, |f| {
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
                            // ⚠️ **GRAUS -> VOLTAS**, o inverso exato da linha que publica o
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
                        .blender_picker(ph2d_editor_core::ids::INSP_BLENDER_PICKER)
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
                    crate::fx_live::edit(sim, &self.vec.entities, &sel, |f| {
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
                &self.vec.entities,
                self.vec.pen.selected_paths(),
            );
            if let Some(v) = pending_pp_spacing
                && let Some(motif) = pp_motif
            {
                crate::pattern_live::edit(sim, &self.vec.entities, motif, |l| l.spacing = v as f32);
            }
            if let Some(v) = pending_pp_start
                && let Some(motif) = pp_motif
            {
                crate::pattern_live::edit(sim, &self.vec.entities, motif, |l| {
                    l.start_offset = v as f32;
                });
            }
            if let Some(v) = pending_pp_end
                && let Some(motif) = pp_motif
            {
                crate::pattern_live::edit(sim, &self.vec.entities, motif, |l| {
                    l.end_offset = v as f32;
                });
            }
            if let Some(v) = pending_pp_offset
                && let Some(motif) = pp_motif
            {
                crate::pattern_live::edit(sim, &self.vec.entities, motif, |l| l.offset = v as f32);
            }
            // A rotação tem porta PRÓPRIA (`set_rotation`) e não o `edit`: ela vive num componente
            // separado, para não bumpar o `PROJECT_SCHEMA` -- e essa porta destaca no neutro.
            if let Some(v) = pending_pp_rotation
                && let Some(motif) = pp_motif
            {
                crate::pattern_live::set_rotation(sim, &self.vec.entities, motif, v as f32);
            }
            // Slide re-centra o trecho `[Start, End]` PRESERVANDO o comprimento: move as duas
            // âncoras juntas (o pedido do Enio). O centro é clampado para a janela caber em [0,1].
            if let Some(v) = pending_pp_slide
                && let Some(motif) = pp_motif
            {
                crate::pattern_live::edit(sim, &self.vec.entities, motif, |l| {
                    let half = (f64::from(l.end_offset) - f64::from(l.start_offset)) * 0.5;
                    let c = v.clamp(half, 1.0 - half);
                    l.start_offset = (c - half) as f32;
                    l.end_offset = (c + half) as f32;
                });
            }
            if let Some(cmd) = pending_patternpath {
                let sel = self.vec.pen.selected_paths().to_vec();
                let done = match cmd {
                    // O guia é o caminho de MAIOR extensão dos dois (independe da ordem de clique)
                    // — a correção do "escolhendo a si mesmo" (Enio).
                    crate::pattern_live::PatternPathCmd::Link => {
                        crate::pattern_live::link_candidate(vec_scene, &sel).is_some_and(
                            |(motif, guide)| {
                                crate::pattern_live::link(sim, &self.vec.entities, motif, guide)
                            },
                        )
                    }
                    crate::pattern_live::PatternPathCmd::Detach => pp_motif
                        .is_some_and(|m| crate::pattern_live::detach(sim, &self.vec.entities, m)),
                    crate::pattern_live::PatternPathCmd::Flip(v) => pp_motif.is_some_and(|m| {
                        crate::pattern_live::edit(sim, &self.vec.entities, m, |l| l.flip = v)
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
            // ⭐⭐⭐ O PINCEL (plano 36, W4): a lei primeiro, o arm depois — a mesma ordem do padrão.
            if let Some(cmd) = pending_brush {
                crate::vec_stroke_paint::apply(vec_scene, &self.vec.pen, cmd);
            }
            if pending_brush_pick && let Some(host) = self.vec.pen.selected() {
                self.vec.path_pick = Some(crate::vec_pick::PathPick::BrushArt(host));
                eprintln!(
                    "[ph2d-vec] brush: pick armado -- clique na FORMA ou no GRUPO que vai ser a \
                     arte do contorno (vazio = desiste)"
                );
            }
            if let Some(slot) = pending_texpat_pick
                && let Some(host) = self.vec.pen.selected()
            {
                self.vec.path_pick = Some(crate::vec_pick::PathPick::TexturePatternArt(host, slot));
                eprintln!(
                    "[ph2d-vec] texture pattern: pick armado -- clique na FORMA ou no GRUPO que \
                     vai ser a arte (vazio = desiste)"
                );
            }
            if pending_pp_pick && let Some(motif) = self.vec.pen.selected() {
                self.vec.path_pick = Some(crate::vec_pick::PathPick::PatternMotif(motif));
                eprintln!(
                    "[ph2d-vec] pattern on path: pick armado -- clique no CAMINHO-guia (vazio = \
                     desiste)"
                );
            }
            // ⭐⭐⭐ **O ESQUELETO** (estudo 42 item 5): os três verbos da seção, aplicados aqui como
            // os do envelope — o dreno acima só CAPTURA, e quem mexe no mundo é este bloco.
            if pending_bone_bind {
                let ids: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
                let semente = osso_selecionado.map(ph2d_ecs::Entity::from_bits);
                let n =
                    crate::skeleton_live::bind(sim, vec_scene, &self.vec.entities, &ids, semente);
                // ⭐⭐⭐ **E AS IMAGENS ESCOLHIDAS** — a 2.ª mídia (ordem do dono, 2026-09-09).
                //
                // ⚠️ **O MESMO botão, e é o desenho todo:** o estado da arte diz que o artista não
                // deve trabalhar na malha, e o gesto que ele já aprendeu é *escolher e prender*. A
                // malha é traçada da própria tinta e nunca aparece na tela.
                //
                // ⚠️ **O sujeito de uma imagem é a SELECÇÃO do gizmo**, e não a lista de caminhos
                // do pen — são duas famílias com dois selectores, e ler o do vector daria sempre
                // zero imagens.
                let imagens: Vec<(ph2d_ecs::Entity, ph2d_asset::AssetId)> = selecao_bits
                    .iter()
                    .copied()
                    .filter_map(ph2d_ecs::Entity::try_from_bits)
                    .filter(|&e| sim.world().get::<ph2d_render::Sprite>(e).is_some())
                    .filter_map(|e| {
                        sim.world()
                            .get::<ph2d_ecs::SpritePixels>(e)
                            .map(|p| (e, p.0))
                    })
                    .collect();
                let mut n_img = 0;
                for (e, id) in imagens {
                    let Some(asset) = asset_db.get(&id) else {
                        continue;
                    };
                    let Some((w, h, cow)) = asset.image_rgba8() else {
                        continue;
                    };
                    if crate::skeleton_live::bind_image(
                        sim,
                        e,
                        &cow,
                        [w, h],
                        ph2d_poly2d::GridOptions::default(),
                        semente,
                    ) {
                        n_img += 1;
                    }
                }
                if n_img > 0 {
                    eprintln!(
                        "[ph2d-vec] osso: {n_img} imagem(ns) presa(s) -- a malha saiu do recorte da \
                         propria tinta e nao aparece na tela"
                    );
                }
                let n = n + n_img;
                if n == 0 {
                    eprintln!(
                        "[ph2d-vec] osso: selecione ao menos UMA forma, e desenhe um esqueleto                          antes (ferramenta Bone)"
                    );
                } else {
                    eprintln!(
                        "[ph2d-vec] osso: {n} forma(s) presa(s) -- no modo Bone: CORPO gira, \
                         bolinha desloca, quadradinho da mancha muda a forca, e o ANEL DUPLO na \
                         ponta da corrente dobra a corrente inteira (IK)"
                    );
                }
            }
            if let Some(keep) = pending_bone_release {
                let ids: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
                crate::skeleton_live::release(sim, vec_scene, &self.vec.entities, &ids, keep);
            }
            if let Some((forca, v)) = pending_bone_knob
                && let Some(bits) = osso_selecionado
                && let Some(mut osso) = sim
                    .world_mut()
                    .get_mut::<ph2d_skeleton_ecs::Bone>(ph2d_ecs::Entity::from_bits(bits))
            {
                // ⛔ Os dois são pisos, não tetos: um comprimento negativo viraria o osso do avesso
                // e uma força negativa daria peso negativo. O TETO é o do documento — §0.0: um
                // limite legítimo diz de que recurso é, e não há recurso nenhum a limitar aqui.
                if forca {
                    osso.strength = v.max(0.0);
                } else {
                    osso.length = v.max(0.0);
                }
            }
            // ⭐⭐⭐ **A ÂNCORA DE IK** — os dois verbos e os três números, aplicados aqui como os do
            // esqueleto: o dreno acima só CAPTURA.
            // ⭐⭐⭐ **QUANTOS CONTROLOS NASCERAM MUDOS ANTES DESTES VERBOS** — a metade de trás da
            // pergunta que o app faz depois deles.
            //
            // ⛔⛔ **Achado da auditoria de 2026-09-08:** o aviso *«este osso é conduzido por uma
            // âncora»* vivia DENTRO do *Add Smart Bone*, logo só disparava na ordem **IK → Smart**.
            // Nas outras duas — pôr a âncora **depois** do controlo, e **alargar o `Chain`** até ele
            // — o app ficava calado sobre exactamente o mesmo facto.
            //
            // ⇒ *um aviso pendurado num VERBO responde por uma ordem; pendurado no FACTO, responde
            // por todas — incluindo as que ninguém enumerou.*
            let mudos_antes = crate::skeleton_smart::governed_controls(sim).len();
            if let Some(bits) = osso_selecionado {
                let osso = ph2d_ecs::Entity::from_bits(bits);
                if pending_ik_add {
                    match crate::skeleton_goal::add(sim, osso) {
                        Some(_) => eprintln!(
                            "[ph2d-vec] osso: ancora de IK criada na ponta -- arraste o LOSANGO e a                              corrente segue-o, para sempre (a timeline anima-o como qualquer objecto)"
                        ),
                        None => eprintln!(
                            "[ph2d-vec] osso: este osso ja' tem ancora -- so' pode haver uma por corrente"
                        ),
                    }
                }
                if pending_ik_remove {
                    crate::skeleton_goal::remove(sim, osso, &mut self.preview_drive);
                }
                if let Some(lado) = pending_ik_bend
                    && let Some(mut g) = sim.world_mut().get_mut::<ph2d_skeleton_ecs::IkGoal>(osso)
                {
                    g.bend = lado;
                }
                // ⭐⭐⭐ **O LIMITE DE ÂNGULO** — os dois verbos e os dois extremos.
                if pending_limit_add && !crate::bone_limit::add_limit(sim, osso) {
                    eprintln!(
                        "[ph2d-vec] osso: esta junta ja' tem limite -- so' pode haver um por osso"
                    );
                }
                if pending_limit_remove {
                    crate::bone_limit::remove_limit(sim, osso);
                }
                // ⭐⭐⭐ **O OSSO INTELIGENTE** — o componente entra VAZIO, e o artista escolhe.
                //
                // ⚠️⚠️ **ELE NÃO CRIA NADA** (ordem do dono, 2026-09-08: *«porque criar Bone Action
                // no inspector e na timeline? Melhor não criar nada»*). O desenho anterior fabricava
                // um clip com o nome do osso e abria a timeline nele — duas coisas por um clique,
                // nenhuma pedida. ⛔ E adoptar o clip ABERTO, que foi o desenho antes desse, era
                // pior ainda: um documento novo tem **uma** acção chamada `"Main"`, logo todo
                // controlo casava com a animação principal da cena, calado.
                //
                // ⇒ o gesto **anexa** o controlo e mais nada; quem lhe dá sujeito são as duas
                // linhas do painel — o *Pick Object* e o selector *Action*.
                if pending_smart_add {
                    sim.world_mut()
                        .entity_mut(osso)
                        .insert(ph2d_skeleton_ecs::SmartBone::default());
                }
                // ⭐⭐⭐ **ARMAR O PICK DO ALVO** — o OSSO é capturado aqui, e não lido no clique
                // seguinte: aquele clique MUDA a selecção, então lê-lo então leria o alvo no lugar
                // do sujeito. É a lei do `PathPick`, escrita no doc dele.
                if pending_smart_pick {
                    self.skeleton.smart_pick = Some(osso.to_bits());
                }
                // ⭐⭐⭐ **TROCAR A ACÇÃO** pelo selector — tudo por UMA porta
                // ([`crate::skeleton_smart::choose_action`]), que é onde a lei vive e onde ela é
                // gateada: a POSIÇÃO resolve-se contra a lista que o PAINEL PINTOU (filtrada pelo
                // alvo), nunca contra `doc.clips()`, e o que se guarda é o NOME.
                //
                // ⛔⛔ Este bloco tinha a lei escrita **aqui** e o gate do outro lado da porta: a
                // mutação que repunha `doc.clips().get(i)` deixava a suíte verde e trazia de volta o
                // report do dono (*«não consegue selecionar o clip desejado»*).
                if let Some(i) = pending_smart_clip
                    && let Some((nome, aberta)) =
                        crate::skeleton_smart::choose_action(sim, &self.timeline.doc, osso, i)
                    && aberta
                {
                    // ⛔⛔ **A acção ABERTA é oferecida e o motor recusa-a** — um documento novo tem
                    // **uma** acção (`"Main"`) e ela **está aberta**, logo a única opção da lista era
                    // a única que não corre, e nada na tela o dizia. ⚠️ A lei fica (um controlo não
                    // percorre o que o artista está a gravar — os dois escreveriam o mesmo objecto
                    // no mesmo quadro); o que não pode é ser **calada**.
                    toasts.push(ph2d_editor_core::Toast::warning(format!(
                        "\"{nome}\" is open in the timeline, so you are EDITING it - the bone will \
                         not run it. Switch the timeline to another animation to see it play."
                    )));
                }
                if pending_smart_remove {
                    // ⭐⭐⭐ **E a POSE VOLTA** — a porta faz as duas metades, como a do *Remove IK*.
                    crate::skeleton_smart::remove(
                        sim,
                        &self.timeline.doc,
                        osso,
                        &mut self.preview_drive,
                    );
                }
                if let Some((e_to, graus)) = pending_smart_knob
                    && let Some(mut sb) = sim
                        .world_mut()
                        .get_mut::<ph2d_skeleton_ecs::SmartBone>(osso)
                {
                    // ⚠️ A MESMA conversão graus→radianos do limite, e pela mesma razão.
                    let rad = graus.to_radians();
                    if e_to {
                        sb.to = rad;
                    } else {
                        sb.from = rad;
                    }
                }
                if let Some((e_max, graus)) = pending_limit_knob
                    && let Some(mut l) = sim
                        .world_mut()
                        .get_mut::<ph2d_skeleton_ecs::BoneLimit>(osso)
                {
                    // ⚠️ **A conversão GRAUS→RADIANOS vive aqui**, na porta entre o campo (que fala
                    // a unidade do artista) e o componente (que fala a do `Transform::rotation`).
                    // ⛔ Sem ela um `90` digitado seria noventa RADIANOS — catorze voltas.
                    //
                    // ⛔⛔ **E pela MESMA porta do arrasto** (`set_edge`, auditoria de 2026-09-08):
                    // escrever cru deixava o campo produzir `min > max`, que a lei lê como faixa de
                    // meia-largura **zero** — a junta congela no ponto médio, e o desenho normaliza
                    // os dois extremos, logo o canvas continua a pintar um sector normal enquanto o
                    // osso não roda um grau.
                    crate::bone_limit::set_edge(&mut l, e_max, graus.to_radians());
                }
                if let Some((qual, v)) = pending_ik_knob
                    && let Some(mut g) = sim.world_mut().get_mut::<ph2d_skeleton_ecs::IkGoal>(osso)
                {
                    match qual {
                        // ⛔ `0..1` é a faixa da LEI, não uma escolha: fora dela o `blend_angle`
                        // satura nos extremos, e um campo que aceita `7` mentiria sobre o efeito.
                        IkKnob::Mix => g.mix = v.clamp(0.0, 1.0),
                        // ⛔ Piso em zero e SEM tecto: a suavidade é uma fracção do alcance, e o
                        // `softened_distance` já a apara pelo próprio alcance — §0.0, o limite é do
                        // recurso e não um palpite.
                        IkKnob::Softness => g.softness = v.max(0.0),
                        // ⛔ Idem: quem apara a corrente é a ARVORE, no passe.
                        #[expect(
                            clippy::cast_possible_truncation,
                            clippy::cast_sign_loss,
                            reason = "o campo é f64 e a contagem de ossos é u32; o piso em 0 já corre acima"
                        )]
                        IkKnob::Chain => g.chain = v.max(0.0) as u32,
                    }
                }
                // ⭐⭐⭐ **E O APP DIZ, seja qual for a ordem em que o artista chegou aqui.**
                //
                // ⚠️ **Avisa e FAZ na mesma**, ⛔ não recusa: tirar a âncora depois é um gesto que
                // existe (*Remove IK*), e um verbo que recusa deixaria o artista sem caminho. O que
                // não pode é o app ficar **calado** sobre um controlo que ele sabe que vai nascer
                // mudo.
                if crate::skeleton_smart::governed_controls(sim).len() > mudos_antes {
                    toasts.push(ph2d_editor_core::Toast::warning(
                        "This bone is driven by an IK anchor, so its angle is derived - turning it \
                         will not run the action. Use a free bone, or Remove IK.",
                    ));
                }
            } else if pending_bone_needs_focus {
                // ⚠️ **Um verbo que morre em SILÊNCIO dá o mesmo sintoma que uma rota cortada** —
                // e foi exactamente esse o report de 2026-09-07 (*«Add IK não funciona»*), cuja
                // causa era outra. O painel só pinta estes botões com um osso em foco, então este
                // braço é a janela de UM quadro entre a publicação do painel e a leitura do dreno;
                // dizê-lo em voz alta é o que separa *«o app recusou»* de *«o botão está morto»*.
                //
                // ⛔⛔ **E a cura cobria DOIS dos oito verbos** (auditoria de 2026-09-08): o limite
                // e o osso inteligente foram acrescentados depois e não vieram a esta condição, logo
                // seis verbos voltaram a morrer calados exactamente na janela que este braço existe
                // para nomear. *Uma cura escrita para os verbos que existiam não segue os que vêm.*
                //
                // ⇒ hoje a condição é **DERIVADA das tabelas de ids** (`ids::needs_focused_bone`) e
                // cobre a seção INTEIRA — os dez verbos, os nove campos e as duas fileiras de chips.
                // Acrescentar um controlo põe-no do lado certo sem ninguém se lembrar deste braço;
                // quem age sobre as FORMAS declara-o em `VECTOR_BONE_ON_SELECTION`.
                eprintln!(
                    "[ph2d-vec] osso: nenhum OSSO em foco -- seleccione um osso (na Hierarquia ou \
                     clicando nele com a ferramenta Bone) antes dos verbos da seccao Skeleton"
                );
            }
            if pending_create_envelope {
                let ids: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
                match crate::envelope_live::create(sim, vec_scene, &self.vec.entities, &ids) {
                    Some(_) => {
                        // O artista SELECIONOU a forma e SÓ ENTÃO clicou Envelope: enveloparr
                        // re-parenteia o filho sem tocar o pen, então o `sync_selection` deste
                        // frame não reroda a promoção filho→container (nem o pen mudou, nem o
                        // conjunto vetorial do gizmo) — e o gizmo ficaria no FILHO, sem gaiola para
                        // desenhar (alças de nó em vez da gaiola). Invalidar a memória do sync força
                        // a promoção no `sync_selection` logo abaixo. Gate:
                        // `enveloping_a_selected_shape_promotes_the_gizmo_to_the_container`.
                        self.vec.sel.invalidate();
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
                    &self.vec.entities,
                    &mut self.vec.pen,
                    crate::envelope_live::Keep::Deformed,
                )
            {
                eprintln!("[ph2d-vec] envelope: expandido (a deformacao virou o desenho)");
            }
            if pending_release_envelope
                && crate::envelope_live::dissolve(
                    sim,
                    vec_scene,
                    &self.vec.entities,
                    &mut self.vec.pen,
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
                    .vec
                    .pen
                    .selected_paths()
                    .iter()
                    .filter_map(|id| self.vec.entities.get(id).copied())
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
                    .vec
                    .pen
                    .selected_paths()
                    .iter()
                    .filter_map(|id| self.vec.entities.get(id).copied())
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
                let sel = self.vec.pen.selected_paths().to_vec();
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
                    &self.vec.entities,
                    &self.vec.pen,
                    &mut self.vec.blend_spines,
                )
            {
                eprintln!("[ph2d-vec] blend: spine resetado ao automático");
            }
            // Arrastar o slider Steps retuna o blend SELECIONADO ao vivo (o recook lê
            // `VecBlend.steps`). Sem blend selecionado, é o valor de criação do próximo Blend.
            if let Some(steps) = pending_blend_steps {
                crate::blend_live::set_selected_steps(
                    sim,
                    &self.vec.entities,
                    &self.vec.pen,
                    steps,
                );
            }
            if let Some(op) = pending_vec_bool {
                // **Um clique, três destinos** (`bool_gesture`): re-mirar um grupo booleano que a
                // seleção já habita · criar um, com o modo `Live` ligado · ou o caminho
                // destrutivo de sempre. ⚠️ A ordem é a lei: sem o primeiro, clicar "Intersect"
                // sobre um grupo vivo com o modo desligado CONSUMIRIA os operandos, e o artista
                // perderia a arte no gesto que ele fez para trocar a operação.
                let sel: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
                let live_mode = ph2d_panel_vector::state::bool_live_on();
                let has_group =
                    crate::bool_gesture::group_of_selection(sim, &self.vec.entities, &sel)
                        .is_some();
                if has_group || live_mode {
                    crate::bool_gesture::arm(
                        sim,
                        vec_scene,
                        &self.vec.entities,
                        &sel,
                        crate::bool_live::code_of_op(op),
                    );
                } else {
                    let xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
                    crate::input_dispatch::apply_vec_boolean(vec_scene, &mut self.vec.pen, &xf, op);
                }
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
                let sel: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
                // **O painel espelha o que está SELECIONADO.** Sem isto, escolher uma forma com
                // offset vivo mostraria os knobs globais do painel e o chip mentiria sobre a
                // forma que está na tela. A borda é a SELEÇÃO (não o clique), e ela corre ANTES
                // da borda dos chips — publicar depois faria o espelho parecer um clique novo e
                // reescreveria a forma com os valores que acabaram de sair dela.
                let mirror = (sel.len() == 1).then(|| sel[0]).filter(|id| {
                    crate::offset_live::spec_of(sim, &self.vec.entities, *id).is_some()
                });
                let knobs = if mirror != self.vec.offset_mirrored {
                    self.vec.offset_mirrored = mirror;
                    match mirror
                        .and_then(|id| crate::offset_live::spec_of(sim, &self.vec.entities, id))
                    {
                        Some(spec) => {
                            ph2d_panel_vector::set_expand_join(spec.join);
                            ph2d_panel_vector::set_expand_side(spec.side);
                            let scale =
                                crate::vec_expand::offset_scale(vec_scene, &self.vec.pen, &{
                                    ph2d_vec_entities::transform::build(sim, &self.vec.entities)
                                });
                            hero.store.set_slider_value(
                                ph2d_editor_core::ids::VECTOR_EXPAND_OFFSET,
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
                    Some(id) if id == ph2d_editor_core::ids::VECTOR_EXPAND_OFFSET
                );
                // O slider fala FRAÇÃO do tamanho da forma (−100%..+100%); o `d` de mundo nasce
                // de `fração × escala` (a porta única `vec_expand::offset_scale`, `ada45fac`).
                // ⚠️ A escala NÃO precisa mais ser congelada no grab: o preview deixou de
                // churnar a cena, então a bbox das FONTES não se move durante o arrasto.
                let frac = hero
                    .store
                    .slider(ph2d_editor_core::ids::VECTOR_EXPAND_OFFSET)
                    .map_or(ph2d_tool_vector::params::OFFSET_DEFAULT_FRAC, |(_, v)| {
                        ph2d_tool_vector::params::slider_to_offset_frac(v)
                    });
                if offset_grabbed && !sel.is_empty() {
                    let xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
                    let d = frac * crate::vec_expand::offset_scale(vec_scene, &self.vec.pen, &xf);
                    crate::offset_live::arm(sim, &self.vec.entities, &sel, d, knobs.0, knobs.1);
                    self.vec.offset_mirrored = (sel.len() == 1).then(|| sel[0]);
                }
                // Um chip de Corner/Side clicado RETUNA os offsets vivos da seleção — e só
                // eles: sem offset armado, o chip arma o próximo arrasto e não inventa
                // geometria de lugar nenhum. Comparar com o quadro anterior é o que distingue
                // "o artista clicou" de "o painel está no valor de sempre".
                if knobs != self.vec.expand_knobs.0 {
                    self.vec.expand_knobs.0 = knobs;
                    crate::offset_live::retune(sim, &self.vec.entities, &sel, knobs);
                }
            }
            // ── A LARGURA VIVA (ADR-0148) ────────────────────────────────────────
            // Os quatro sliders `W Start/Mid/End/Pos` deixaram de ser parâmetros de um comando
            // e passaram a AUTORAR um perfil vivo: o traço engrossa e afina enquanto o slider
            // anda, e o botão *Power Stroke* MATERIALIZA — o mesmo par que o Offset já tinha.
            {
                let sel: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
                // **O painel espelha o que está SELECIONADO** — a mesma lei (e a mesma ordem) do
                // offset acima: a borda é a SELEÇÃO, e ela corre ANTES de o arrasto ser lido.
                let mirror = (sel.len() == 1).then(|| sel[0]).filter(|id| {
                    crate::profile_live::spec_of(sim, &self.vec.entities, *id).is_some()
                });
                if mirror != self.vec.profile_mirrored {
                    self.vec.profile_mirrored = mirror;
                    if let Some(p) = mirror
                        .and_then(|id| crate::profile_live::spec_of(sim, &self.vec.entities, id))
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
                    crate::profile_live::arm(sim, &self.vec.entities, &sel, &p.profile.to_stops());
                    // O espelho da seleção acabou de ser ESCRITO por nós: sem isto o bloco do
                    // frame seguinte veria o `mirror` inalterado e não reescreveria nada — mas
                    // com uma seleção de uma forma só ele passaria a divergir na primeira troca.
                    self.vec.profile_mirrored = (sel.len() == 1).then(|| sel[0]);
                }
                let grabbed = matches!(
                    hero.store.active_id(),
                    Some(id) if id == ph2d_editor_core::ids::VECTOR_EXPAND_W_START
                        || id == ph2d_editor_core::ids::VECTOR_EXPAND_W_MID
                        || id == ph2d_editor_core::ids::VECTOR_EXPAND_W_END
                        || id == ph2d_editor_core::ids::VECTOR_EXPAND_W_POS
                );
                if grabbed && !sel.is_empty() {
                    let stops = crate::profile_live::preset_from_store(&hero.store).to_stops();
                    crate::profile_live::arm(sim, &self.vec.entities, &sel, &stops);
                    self.vec.profile_mirrored = (sel.len() == 1).then(|| sel[0]);
                }
            }
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
                    crate::render_loop::ui_state_bridge::request(
                        ui_machines,
                        ui_states,
                        host,
                        role,
                    );
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
            // `ui_signal_reader`. Sem o `let _`, o `read` devolve um iterador preguiçoso e **nada
            // é consumido**: o cursor não andaria, e o gate que o prova é o da entrada limpa.
            {
                let acting = self.ui_preview.is_on();
                let moves: Vec<(ph2d_vec_scene::VecPathId, ph2d_ui_state::StateRole)> = self
                    .signals
                    .read(&mut self.ui_signal_reader)
                    .filter(|_| acting)
                    .flat_map(|sig| ui_states.targets(&sig.name).collect::<Vec<_>>())
                    .collect();
                for (host, role) in moves {
                    crate::render_loop::ui_state_bridge::request(
                        ui_machines,
                        ui_states,
                        host,
                        role,
                    );
                }
            }
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
                    self.ui_preview.enter(
                        ui_machines,
                        ui_states,
                        sim,
                        vec_scene,
                        &self.vec.entities,
                    );
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
            if let Some(verb) = pending_component {
                let sel: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
                // ⭐⭐⭐ **O CLIQUE da secção *Prefab*, e há UM motor** (F4.6c fechada, 2026-09-07).
                //
                // ⚠️ **Aqui viveu um `if armed() { … } else { … }`** — o modelo geral de um lado e o
                // motor `VecInstance` do outro, com uma variável de ambiente a escolher. O `else`
                // morreu com a fatia: *dois motores para o mesmo estado é pior que um motor lento*,
                // e a régua de que nada se perdeu está em `instance_piece_override_tests.rs`, que a
                // wave anterior escreveu **como pré-condição desta**.
                //
                // ⚠️ **O sujeito resolve-se ANTES dos documentos** — o mapa `path ⟺ entidade`
                // entra no `OwnedDocs` emprestado mutavelmente, e pedi-lo outra vez lá dentro
                // seria o segundo empréstimo.
                //
                // ⚠️ **Pela MESMA função que a secção usa para se MOSTRAR** — duas resoluções
                // dariam um botão oferecido sobre o grupo e um clique a agir sobre um filho.
                let subject = crate::vec_component_general::subject_of(
                    &self.vec.entities,
                    &sel,
                    (hero.gizmo.selected_len() == 1)
                        .then_some(hero.gizmo.selection)
                        .flatten(),
                );
                let step = crate::input_dispatch::screen_offset_world(
                    camera,
                    window_size,
                    crate::input_dispatch::PASTE_OFFSET_PX,
                );
                let mut select_out = None;
                let mut arm_pick = false;
                if let Some(subject) = subject {
                    let mut docs = ph2d_app_components::instance_docs::OwnedDocs {
                        vec_scene,
                        vec_entities: &mut self.vec.entities,
                    };
                    if crate::vec_component_general::dispatch(
                        verb,
                        sim,
                        component_registry,
                        &mut self.instance_echo,
                        subject,
                        toasts,
                        &mut docs,
                        [step.0 as f32, step.1 as f32],
                        &mut select_out,
                        &mut arm_pick,
                    ) {
                        self.title_dirty = true;
                    }
                }
                if let Some(bits) = select_out {
                    hero.gizmo.replace_selection(Some(bits));
                }
                // ⭐⭐ **A shell só ESCREVE o pick — quem decide é o módulo do modo.** O
                // `PathPick` vive no `App`, e por isso o dreno não lhe chega; mas a pergunta
                // *«este verbo abre o gesto de duas mãos?»* é lei do modo, e fica lá.
                if arm_pick && let Some(&at) = sel.first() {
                    self.vec.path_pick = Some(crate::vec_pick::PathPick::InstanceMain(at));
                }
            }
            if let Some(cmd) = pending_vec_expand {
                let xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
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
                    self.vec.pen.selected_paths().to_vec();
                if matches!(cmd, crate::vec_expand::Expand::PowerStroke { .. })
                    && crate::profile_live::materialise(
                        vec_scene,
                        sim,
                        &mut self.vec.pen,
                        &self.vec.entities,
                        &xf,
                        &sel_now,
                    )
                {
                    // A forma nova não tem perfil vivo, e knobs parados num afinamento sobre ela
                    // mentiriam sobre o que está na cena — o mesmo argumento do slider de Offset.
                    self.vec.profile_mirrored = None;
                    crate::profile_live::write_preset_to_store(
                        &mut hero.store,
                        &ph2d_vec_scene::WidthProfile::UNIFORM,
                    );
                    return;
                }
                let materialised = matches!(cmd, crate::vec_expand::Expand::Offset { .. }) && {
                    let ids: Vec<ph2d_vec_scene::VecPathId> =
                        self.vec.pen.selected_paths().to_vec();
                    crate::offset_live::materialise(
                        vec_scene,
                        sim,
                        &mut self.vec.pen,
                        &self.vec.entities,
                        &xf,
                        &ids,
                    )
                };
                if materialised {
                    self.vec.offset_mirrored = None;
                    // O slider volta ao zero: a forma nova não tem offset vivo, e um slider
                    // parado em +40% sobre ela mentiria sobre o que está na cena.
                    hero.store.set_slider_value(
                        ph2d_editor_core::ids::VECTOR_EXPAND_OFFSET,
                        ph2d_tool_vector::params::offset_frac_to_slider(0.0),
                    );
                } else {
                    // O caminho NUMÉRICO (sem offset vivo armado): a distância vem do slider —
                    // a MESMA fonte que o chip mostra —, fração × escala da seleção atual.
                    let d = hero
                        .store
                        .slider(ph2d_editor_core::ids::VECTOR_EXPAND_OFFSET)
                        .map_or(ph2d_tool_vector::params::OFFSET_DEFAULT_FRAC, |(_, v)| {
                            ph2d_tool_vector::params::slider_to_offset_frac(v)
                        })
                        * crate::vec_expand::offset_scale(vec_scene, &self.vec.pen, &xf);
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
                        &mut self.vec.pen,
                        &xf,
                        cmd.clone(),
                        d,
                    );
                    // O botão de Offset (caminho numérico: arrastar sem seleção viva e clicar)
                    // recentra o slider — cada aplicação offseta pelo valor mostrado e zera.
                    if matches!(cmd, crate::vec_expand::Expand::Offset { .. }) {
                        hero.store.set_slider_value(
                            ph2d_editor_core::ids::VECTOR_EXPAND_OFFSET,
                            ph2d_tool_vector::params::offset_frac_to_slider(0.0),
                        );
                    }
                }
            }
            if let Some(make) = pending_vec_compound {
                crate::input_dispatch::apply_vec_compound(vec_scene, &mut self.vec.pen, make);
            }
            if let Some(even_odd) = pending_vec_fill_rule {
                crate::input_dispatch::apply_vec_fill_rule(vec_scene, &self.vec.pen, even_odd);
            }
            // Snap settings are TOOL state, not document state — no undo step.
            if let Some(on) = pending_vec_snap_on {
                self.vec.snap.on = on;
            }
            if let Some(on) = pending_vec_snap_path {
                self.vec.snap.path = on;
            }
            if let Some(on) = pending_vec_snap_cross {
                self.vec.snap.crossings = on;
            }
            if let Some(on) = pending_vec_snap_guides {
                self.vec.snap.guides = on;
            }
            // ⚠️ A régua é estado do HERO, não da ferramenta: ela é chrome de canvas, aparece
            // com qualquer ferramenta na mão, e é o mesmo flag que a tecla/menu de vista
            // mexeria. O painel do vetor é só mais um lugar de onde se alcança o interruptor.
            if let Some(on) = pending_rulers {
                hero.view.rulers_visible = on;
            }
            if let Some(kind) = pending_vec_vertex_kind {
                crate::input_dispatch::apply_vec_vertex_kind(vec_scene, &mut self.vec.pen, kind);
            }
            if pending_vec_delete_vertex {
                crate::input_dispatch::apply_vec_delete_vertex(vec_scene, &mut self.vec.pen);
            }
            // ⚠️ Os dois só mudam QUEM está selecionado, e a seleção não é estado de documento — o
            // undo global (por diff) não vê passo nenhum neles.
            if pending_vec_select_subpath {
                self.vec.pen.select_subpath_verts(vec_scene);
            }
            if pending_vec_select_same {
                self.vec.pen.select_verts_of_same_kind(vec_scene);
            }
            // **As três da W4.** O passo de undo é o da fila global (por diff do quadro): ele só
            // nasce se algo de fato mudou.
            for (armed, op) in [
                (pending_vec_join, 0u8),
                (pending_vec_reverse, 1),
                (pending_vec_average, 2),
            ] {
                if !armed {
                    continue;
                }
                match op {
                    0 => self.vec.pen.join_selection(vec_scene),
                    1 => self.vec.pen.reverse_selected_paths(vec_scene),
                    _ => self.vec.pen.average_selected_verts(vec_scene),
                };
            }
            // ⭐⭐⭐ **SOLDAR** (plano 39) — ao lado das três acima, e **fora** do laço delas porque
            // ela precisa das POSES: dois traços só se cruzam depois de o `Transform` os pôr no
            // lugar, e medir na geometria local diria que eles não se encontram.
            // ⚠️ Um comando que não cortou nada não muda a cena, e por isso não gasta um Ctrl+Z (o
            // undo global regista por diff).
            if pending_vec_weld {
                let xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
                crate::vec_weld::apply_vec_weld(
                    vec_scene,
                    &mut self.vec.pen,
                    &xf,
                    crate::vec_snap::vec_weld_tolerance(vec_px_to_world),
                );
            }
            // **O CORTE e o DESCARTE da lâmina.** Como as três acima: o passo é o do undo global, e
            // só nasce se algo de fato mudou (uma lâmina que não atravessa nada não deixa linha na
            // fila do Ctrl+Z).
            // **Apply Symmetry** — o único gesto desta seção que toca o documento. Até aqui as
            // cópias eram DESENHO; a partir daqui são geometria, e a simetria sai com a
            // forma-fonte (o `sync` do frame seguinte despawna a entidade dela).
            if pending_vec_symmetry_apply {
                let xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
                // ⚠️ TODA forma armada, não a seleção: a simetria é um MODO, e *"consolidar a
                // forma e desativar a simetria"* vale para o que o modo produziu. O porquê de
                // consolidar só o selecionado ser destrutivo está na `armed_paths`.
                let ids = crate::symmetry_live::armed_paths(sim, &self.vec.entities, vec_scene);
                crate::symmetry_live::materialise(
                    vec_scene,
                    sim,
                    &mut self.vec.pen,
                    &self.vec.entities,
                    &xf,
                    &ids,
                );
            }
            if pending_vec_cut {
                // A seleção que o corte exige é a da LÂMINA, e ela pode chegar por qualquer das
                // duas listas do pen (a de objeto e a de caminho) — perguntar só a uma delas faria
                // o botão recusar um gesto legítimo.
                let mut selected = self.vec.pen.selected_paths().to_vec();
                if let Some(one) = self.vec.pen.selected()
                    && !selected.contains(&one)
                {
                    selected.push(one);
                }
                let armed = self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Cut;
                if crate::vec_cut_line::apply_cut(
                    sim,
                    vec_scene,
                    &self.vec.entities,
                    &selected,
                    armed,
                ) > 0
                {
                    // A seleção descreve formas que já não existem — as peças as substituíram.
                    self.vec.pen.select(None);
                }
            }
            if pending_vec_cut_discard {
                crate::vec_cut_line::discard(sim, vec_scene, &self.vec.entities);
            }
            // **Os botões Arrange escrevem na ÁRVORE** (Enio, 2026-08-04). Eles chamavam o
            // `VecScene::reorder_path`, que mexe na ordem do VETOR da cena — e essa é reescrita a
            // cada frame pela projeção da árvore (ADR-0110), então os quatro estavam MORTOS:
            // acendiam, mexiam, e o frame seguinte desfazia. O undo é o GLOBAL por diff (a árvore
            // é `ProjectState`). (Até 2026-09-12 esta nota contrastava-o com a `vec_history`, a
            // pilha do vetor que guardava a CENA — ela morreu sem nunca ter tido leitor.)
            if let Some(order) = pending_vec_reorder
                && let Some(sel) = self.vec.pen.selected()
            {
                ph2d_vec_entities::entities::zorder::reorder(sim, &self.vec.entities, sel, order);
            }
            if pending_vec_duplicate {
                // Offset the clone by a fixed SCREEN distance (px → world) so it's
                // visibly separated at any zoom.
                //
                // ⚠️ A const é a PARTILHADA. Ela vivia aqui como cópia local de 12.0 ao lado do
                // `PASTE_OFFSET_PX`, que é o mesmo número pela mesma razão — exatamente a
                // divergência contra a qual o doc de `screen_offset_world` avisa.
                let off = crate::input_dispatch::PASTE_OFFSET_PX * vec_px_to_world;
                crate::input_dispatch::apply_vec_duplicate(vec_scene, &mut self.vec.pen, off, off);
            }
            if let Some(axis) = pending_vec_flip {
                crate::input_dispatch::apply_vec_flip(vec_scene, &self.vec.pen, axis);
            }
            if let Some(dir) = pending_vec_rotate {
                crate::input_dispatch::apply_vec_rotate(vec_scene, &self.vec.pen, dir);
            }
            // O afim de cada path, para as operações que falam MUNDO (align, distribute,
            // campos X/Y/W/H). O mapa é o do frame passado — os paths envolvidos já
            // existem, então basta.
            let vec_xf_ops = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
            // ⚠️ **A VOLTA da fronteira de display, e ela mora AQUI e não dentro do
            // `apply_vec_transform`.** O `target` é o número que o artista DIGITOU, logo está na
            // unidade dele; a operação fala mundo. Converter dentro dela quebraria o outro
            // chamador logo abaixo — o preset de dispositivo devolve **unidades de DOCUMENTO**
            // (`DevicePreset::size`, o aspecto do aparelho normalizado ao `LONG_SIDE`), que é dado
            // AUTORADO e não um número da face do artista: ele tem de atravessar intocado.
            // ⭐⭐⭐ **A APARÊNCIA do objecto entra no DOCUMENTO** (estudo 42 item 2).
            //
            // ⚠️ **Escreve na selecção INTEIRA e lê do primário** — a lei desta janela, e a porta é
            // uma só (`vec_appearance`). ⚠️ E o passo de undo sai de graça: o registo é por DIFF, e
            // um arrasto de slider não regista por quadro (um gesto em curso suprime a captura),
            // então a corrida inteira colapsa em UM passo ao soltar.
            {
                let sel = self.vec.pen.selected_paths().to_vec();
                if let Some(track) = pending_vec_opacity {
                    #[allow(clippy::cast_possible_truncation)]
                    crate::vec_appearance::set_opacity(vec_scene, &sel, track as f32);
                }
                if let Some(code) = pending_vec_blend {
                    crate::vec_appearance::set_blend(vec_scene, &sel, code);
                }
                // ⭐⭐⭐ **E A PILHA** (item 4). ⚠️ O verbo primeiro, as propriedades depois: um
                // clique em «apagar» e um arrasto de slider não chegam no mesmo frame, mas se
                // chegassem, escrever numa camada que o verbo acabou de remover seria um `None`
                // silencioso — e a ordem torna isso impossível de acontecer ao contrário.
                if let Some(v) = pending_paint_verb {
                    crate::vec_paint_stack::apply(vec_scene, &sel, v);
                }
                // ⚠️ **O índice é o da camada ABERTA no painel** — a única que mostra estes três
                // controlos. Sem camada aberta não há sujeito, e escrever seria adivinhar.
                // ⭐⭐⭐ **A COR de uma camada** — o picker partilhado escreve nela.
                //
                // ⛔ Sem este braço a swatch de uma camada abre o picker, o artista escolhe uma
                // cor e **nada acontece** — o `clippy` apanhou-o como `set_color` never used, que
                // é a assinatura do controlo morto que o §5.0 nomeia.
                if let Some(i) = crate::vec_paint_stack::layer_of_picker_target(&hero.store)
                    && let Some((value, _, _, _)) = hero
                        .store
                        .blender_picker(ph2d_editor_core::ids::INSP_BLENDER_PICKER)
                {
                    // ⚠️ O picker já entrega bytes (`rgba`), como as swatches de base — converter
                    // aqui seria a segunda régua de *"que cor é esta?"*.
                    let c = value.rgba;
                    crate::vec_paint_stack::set_color(
                        vec_scene,
                        &sel,
                        i,
                        ph2d_vec_scene::Rgba8::new(c[0], c[1], c[2], c[3]),
                    );
                }
                if let Some(i) = ph2d_panel_vector::state::open_layer_index() {
                    if let Some(w) = pending_paint_width {
                        crate::vec_paint_stack::set_width(vec_scene, &sel, i, w);
                    }
                    // ⭐ ONDE ela desenha (v21). ⚠️ O eixo que NAO comitou le-se do documento, e
                    // nao de um default: escrever `0` no gemeo apagaria o valor que o artista
                    // acabou de por na outra caixa.
                    if pending_paint_dx.is_some() || pending_paint_dy.is_some() {
                        // ⛔ `sel.first()`, nunca `sel[0]`: a camada aberta é estado de VISTA e
                        // sobrevive a um quadro em que a selecção esvaziou.
                        let atual = sel
                            .first()
                            .and_then(|id| vec_scene.path(*id))
                            .and_then(|p| p.paints.get(i).map(|e| e.offset))
                            .unwrap_or([0.0, 0.0]);
                        let novo = [
                            pending_paint_dx.unwrap_or(atual[0]),
                            pending_paint_dy.unwrap_or(atual[1]),
                        ];
                        crate::vec_paint_stack::set_offset(vec_scene, &sel, i, novo);
                    }
                    if let Some(t) = pending_paint_opacity {
                        #[allow(clippy::cast_possible_truncation)]
                        crate::vec_paint_stack::set_opacity(vec_scene, &sel, i, t as f32);
                    }
                    if let Some(code) = pending_paint_blend {
                        crate::vec_paint_stack::set_blend(vec_scene, &sel, i, code);
                    }
                    if let Some(d) = pending_paint_dilate {
                        crate::vec_paint_stack::set_dilate(vec_scene, &sel, i, d);
                    }
                    if let Some(j) = pending_paint_join {
                        crate::vec_paint_stack::set_dilate_join(vec_scene, &sel, i, j);
                    }
                }
            }
            if let Some((field, target)) = pending_vec_transform {
                let target = ph2d_editor_core::LengthDisplay::of(&hero.project).to_world(target);
                crate::input_dispatch::apply_vec_transform(
                    sim,
                    &self.vec.entities,
                    vec_scene,
                    &self.vec.pen,
                    &vec_xf_ops,
                    field,
                    target,
                );
            }
            // **O NÓ ANDA PELA PORTA DAS SETAS.**
            //
            // ⚠️ O que o dreno aplica é um **DESLOCAMENTO**, nunca uma posição: `PenTool::nudge`
            // é a porta que o teclado já usa, e ela move âncora **e handles** e converte
            // mundo→local **por FORMA** (`delta_to_local`) — duas formas de escalas diferentes
            // andariam distâncias diferentes sob uma conversão só. Um `set_vertex_position` seria
            // a segunda resposta a *"como um nó se move?"*, e as duas divergiriam no dia em que
            // uma delas ganhasse um caso especial (o `nudge` já tem um).
            //
            // ⚠️ E o número digitado atravessa a MESMA fronteira de display do Transform: ele sai
            // da face do artista e volta pela mesma porta.
            if let Some((is_y, target)) = pending_vec_vert {
                let target = ph2d_editor_core::LengthDisplay::of(&hero.project).to_world(target);
                if let Some(now) = self.vec.pen.selected_anchor_world(vec_scene) {
                    let (dx, dy) = if is_y {
                        (0.0, target - now[1])
                    } else {
                        (target - now[0], 0.0)
                    };
                    self.vec.pen.nudge(vec_scene, dx, dy);
                }
            }
            // **O preset de dispositivo da MOLDURA** (plano UI/UX W0) — dois números pela porta
            // que os campos W/H já usam. Um preset não é um caminho novo: é o mesmo pedido feito
            // de outra forma, e é por isso que ele herda o undo e o clamp de dimensão degenerada.
            if let Some(p) = pending_frame_preset {
                let (pw, ph) = p.size();
                for (field, target) in [
                    (crate::input_dispatch::VecTransformField::W, pw),
                    (crate::input_dispatch::VecTransformField::H, ph),
                ] {
                    crate::input_dispatch::apply_vec_transform(
                        sim,
                        &self.vec.entities,
                        vec_scene,
                        &self.vec.pen,
                        &vec_xf_ops,
                        field,
                        target,
                    );
                }
            }
            if let Some(deg) = pending_vec_rotate_by {
                crate::input_dispatch::apply_vec_rotate_by(vec_scene, &self.vec.pen, deg);
            }
            // Configs de texto: aplicam na SESSÃO viva; sem sessão, no objeto de TEXTO
            // SELECIONADO (o texto segue editável no Select até virar curva). O
            // `vec_text_sel` é a seleção corrente para o caminho do objeto.
            let vec_text_sel: Vec<ph2d_vec_scene::VecPathId> =
                self.vec.pen.selected_paths().to_vec();
            // **O conector, pelo painel.** Editar um campo FIXA o valor (`None` → `Some`) em
            // TODOS os conectores selecionados — é o que permite calibrar o diagrama inteiro
            // de uma vez, em vez de linha por linha. A geometria não é escrita aqui: ela é
            // função pura da relação, e o `connector_live::recook` deste mesmo frame (mais
            // abaixo) a refaz. O undo global pega a mudança pelo diff do mundo ECS.
            if let Some((id, v)) = pending_vec_connector {
                crate::vec_connector_panel::edit_selected_connectors(
                    sim,
                    &self.vec.entities,
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
                    &self.vec.entities,
                    &vec_text_sel,
                    // ⚠️ **O MESMO modo que a pintura leu**: armado para desenhar, a caixa move o
                    // default do próximo traço e NÃO alcança a forma selecionada — senão digitar
                    // *"Pontas"* na Estrela armada poria lados no Polígono que está na tela
                    // (os slots são por índice). O espelho é do frame anterior, e isso basta:
                    // trocar de modo e digitar não são o mesmo gesto.
                    self.vec.draw_config.mode,
                    self.vec.shape_armed,
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
            let editing_session = self.vec.text_edit.is_some();
            if let Some(size) = pending_vec_text_size {
                crate::vec_text::apply_text_size(
                    &mut self.vec.text_edit,
                    &mut self.vec.text.size,
                    vec_scene,
                    size,
                );
                if !editing_session {
                    crate::vec_text::edit_selected_text(
                        sim,
                        vec_scene,
                        &self.vec.entities,
                        &vec_text_sel,
                        |p| p.size = size,
                    );
                }
            }
            if let Some(weight) = pending_vec_text_weight {
                crate::vec_text::apply_text_weight(
                    &mut self.vec.text_edit,
                    &mut self.vec.text.weight,
                    vec_scene,
                    weight,
                );
                if !editing_session {
                    crate::vec_text::edit_selected_text(
                        sim,
                        vec_scene,
                        &self.vec.entities,
                        &vec_text_sel,
                        |p| p.weight = weight,
                    );
                }
            }
            if let Some(lh) = pending_vec_text_line_height {
                crate::vec_text::apply_text_line_height(
                    &mut self.vec.text_edit,
                    &mut self.vec.text.line_height,
                    vec_scene,
                    lh,
                );
                if !editing_session {
                    crate::vec_text::edit_selected_text(
                        sim,
                        vec_scene,
                        &self.vec.entities,
                        &vec_text_sel,
                        |p| p.line_height = lh,
                    );
                }
            }
            if let Some(wrap) = pending_vec_text_wrap {
                crate::vec_text::apply_text_wrap(
                    &mut self.vec.text_edit,
                    &mut self.vec.text.wrap,
                    vec_scene,
                    wrap,
                );
                if !editing_session {
                    crate::vec_text::edit_selected_text(
                        sim,
                        vec_scene,
                        &self.vec.entities,
                        &vec_text_sel,
                        |p| p.wrap_width = wrap,
                    );
                }
            }
            if let Some(tr) = pending_vec_text_tracking {
                crate::vec_text::apply_text_tracking(
                    &mut self.vec.text_edit,
                    &mut self.vec.text.tracking,
                    vec_scene,
                    tr,
                );
                if !editing_session {
                    crate::vec_text::edit_selected_text(
                        sim,
                        vec_scene,
                        &self.vec.entities,
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
                    &self.vec.entities,
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
                        &self.vec.entities,
                        &vec_text_sel,
                        |p| p.align = crate::vec_text::align_to_u8(align),
                    );
                }
                crate::vec_text::apply_text_align(
                    &mut self.vec.text_edit,
                    &mut self.vec.text.align,
                    vec_scene,
                    align,
                );
            }
            // A família "corrente" para o ciclo `<`/`>` é a do ALVO: o objeto de texto
            // selecionado (sem sessão) ou o default da shell.
            let cur_family = if editing_session {
                self.vec.text.family.clone()
            } else {
                crate::vec_text::selected_text_object(sim, &self.vec.entities, &vec_text_sel)
                    .map_or_else(|| self.vec.text.family.clone(), |(_, _, p)| p.family)
            };
            if let Some(dir) = pending_vec_font_cycle {
                let next = crate::vec_font::cycle_family(cur_family.as_deref(), dir);
                if !editing_session {
                    crate::vec_text::set_selected_text_font(
                        sim,
                        vec_scene,
                        &self.vec.entities,
                        &vec_text_sel,
                        next.clone(),
                    );
                }
                crate::vec_text::set_text_font(
                    &mut self.vec.text_edit,
                    &mut self.vec.text.family,
                    &mut self.vec.text.extra_axes,
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
                        &self.vec.entities,
                        &vec_text_sel,
                        family.clone(),
                    );
                }
                crate::vec_text::set_text_font(
                    &mut self.vec.text_edit,
                    &mut self.vec.text.family,
                    &mut self.vec.text.extra_axes,
                    vec_scene,
                    family,
                );
            }
            if let Some((index, value)) = pending_vec_text_axis {
                crate::vec_text::apply_text_axis(
                    &mut self.vec.text_edit,
                    &mut self.vec.text.extra_axes,
                    vec_scene,
                    index,
                    value,
                );
            }
            if pending_vec_font_import {
                let imported = crate::vec_text::import_text_font(
                    &mut self.vec.text_edit,
                    &mut self.vec.text.family,
                    &mut self.vec.text.extra_axes,
                    vec_scene,
                );
                // Sem sessão, a fonte importada vai para o objeto de texto SELECIONADO.
                if imported && !editing_session {
                    let fam = self.vec.text.family.clone();
                    crate::vec_text::set_selected_text_font(
                        sim,
                        vec_scene,
                        &self.vec.entities,
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
                crate::input_dispatch::apply_vec_path_shape(vec_scene, &self.vec.pen, op);
            }
            if pending_vec_toggle_closed {
                crate::input_dispatch::apply_vec_toggle_closed(vec_scene, &mut self.vec.pen);
            }
            if let Some(kind) = pending_vec_fill_kind {
                crate::texture_pattern_edit::log_shape(
                    &format!("ANTES de mudar para {kind:?}"),
                    vec_scene,
                    &self.vec.pen,
                );
                // ⭐ **A 4ª condição da costura: o chip tem de LEVAR A ALGUM LUGAR.** Escolher
                // *Tile* numa forma sem padrão abre o diálogo da arte — um chip que muda o tipo de
                // preenchimento para algo invisível é o defeito que esta linha já recebeu três
                // vezes. ⚠️ Desistir devolve `None`, e o `apply` **não muda nada**.
                // ⚠️ Sem closure: `self.vec.pen.selected()` e `self.texture_pattern_source_for`
                // (que é `&mut self`) não cabem no mesmo `and_then`.
                let mut pattern = None;
                if kind == crate::input_dispatch::VecFillKind::Pattern
                    && let Some(sel) = self.vec.pen.selected()
                    && let Some(source) = crate::texture_pattern_pick::source_for(
                        vec_scene,
                        sel,
                        ph2d_vec_render::PatternSlot::Fill,
                    )
                {
                    // ⭐ **A ARTE mede-se pela porta que ASSA** — e o mapa de afins e a geometria
                    // viva são os do quadro anterior, que é o que os seis sítios de pick desta
                    // shell já consomem: a forma que o artista acabou de apontar já lá está.
                    //
                    // ⏳ **MEDIDO e não curado** (auditoria de 2026-08-30): quando a forma JÁ tem
                    // padrão, o `apply_vec_set_fill_kind` preserva-o e descarta este `size` inteiro
                    // — é a forma do *«consumidor que projecta o valor fora»*. ⛔ Um guarda aqui
                    // codificaria um facto sobre **outra** função (*"o `source_for` só devolve
                    // não-`None` quando o slot já era padrão"*), e é essa acoplagem que diverge em
                    // silêncio. O custo é **por clique**, não por quadro.
                    let xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
                    let arte = crate::texture_pattern_pick::art_dims(
                        asset_db,
                        vec_scene,
                        &xf,
                        &self.vec.live_drawn,
                        sel,
                        &source,
                        &|id| {
                            ph2d_vec_entities::entities::object_selection_for(
                                sim,
                                vec_scene,
                                &self.vec.entities,
                                id,
                            )
                        },
                    );
                    let (size, origin) =
                        crate::texture_pattern_pick::default_placement(vec_scene, sel, arte);
                    pattern = Some((source, size, origin));
                }
                crate::input_dispatch::apply_vec_set_fill_kind(
                    vec_scene,
                    &self.vec.pen,
                    kind,
                    pattern,
                );
                crate::texture_pattern_edit::log_shape("DEPOIS", vec_scene, &self.vec.pen);
                // The old handle no longer addresses the new fill kind — reset the
                // gradient selection so the overlay highlight + panel don't cling to it.
                self.vec.grad_selected = None;
                self.vec.grad_drag = None;
            }
            // ⭐⭐ **A TINTA DO TRAÇO** (plano 35, wave D) — a 4ª condição da costura, outra vez:
            // escolher *Pattern* num traço que ainda não tem padrão **abre o diálogo da arte**.
            // ⚠️ Desistir devolve `None`, e o `set_kind` **não muda nada** — apagar-lhe a cor do
            // traço por ter fechado um diálogo seria o pior dos dois mundos.
            if let Some(kind) = pending_vec_stroke_kind {
                let mut pattern = None;
                // ⭐⭐⭐ **E O TRAÇO PELA MESMA PORTA** (report do Enio, 2026-08-30: *"e para
                // Stroke?"*). Ele ia direto ao `pick_source`, que abre SEMPRE o diálogo de imagem —
                // o mesmo defeito do preenchimento, e ainda mais cru, porque nem passava pela porta
                // que decide.
                //
                // ⚠️⚠️ **A lei já estava escrita neste ficheiro, um braço acima, para o PINCEL:**
                // *"`art: None` é legítimo por TIPO — um pincel sem arte escolhida desenha a
                // `fallback`, e a arte entra depois pelo gesto de duas mãos. Exigi-la aqui faria o
                // chip abrir um diálogo de ficheiro, que é precisamente o que o plano 36 recusa."*
                // O braço `Pattern` não a herdou porque o `PatternSource` não tinha variante vazia —
                // e desde 30/08 tem. *A regra certa estava no mesmo `match`, para o vizinho.*
                if kind == ph2d_panel_vector::StrokePaintKind::Pattern
                    && let Some(sel) = self.vec.pen.selected()
                    && let Some(source) = crate::texture_pattern_pick::source_for(
                        vec_scene,
                        sel,
                        ph2d_vec_render::PatternSlot::Stroke,
                    )
                {
                    // ⚠️ **A colocação sai da MESMA porta do preenchimento** — o tamanho preserva o
                    // aspecto da arte e o canto é o da FORMA, nunca a origem do mundo. Uma segunda
                    // lei de nascimento aqui reabriria o report do `Clamp` em branco.
                    // ⭐ **A ARTE mede-se pela porta que ASSA** — e o mapa de afins e a geometria
                    // viva são os do quadro anterior, que é o que os seis sítios de pick desta
                    // shell já consomem: a forma que o artista acabou de apontar já lá está.
                    //
                    // ⏳ **MEDIDO e não curado** (auditoria de 2026-08-30): quando a forma JÁ tem
                    // padrão, o `apply_vec_set_fill_kind` preserva-o e descarta este `size` inteiro
                    // — é a forma do *«consumidor que projecta o valor fora»*. ⛔ Um guarda aqui
                    // codificaria um facto sobre **outra** função (*"o `source_for` só devolve
                    // não-`None` quando o slot já era padrão"*), e é essa acoplagem que diverge em
                    // silêncio. O custo é **por clique**, não por quadro.
                    let xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
                    let arte = crate::texture_pattern_pick::art_dims(
                        asset_db,
                        vec_scene,
                        &xf,
                        &self.vec.live_drawn,
                        sel,
                        &source,
                        &|id| {
                            ph2d_vec_entities::entities::object_selection_for(
                                sim,
                                vec_scene,
                                &self.vec.entities,
                                id,
                            )
                        },
                    );
                    let (size, origin) =
                        crate::texture_pattern_pick::default_placement(vec_scene, sel, arte);
                    pattern = Some((source, size, origin));
                }
                crate::vec_stroke_paint::set_kind(vec_scene, &self.vec.pen, kind, pattern);
            }
            // ⭐ **A secção PATTERN** (plano 33 W5) — a arte primeiro (ela abre um diálogo, que
            // congela o laço), depois a lei. As duas desaguam na MESMA porta.
            if let Some(slot) = pending_texpat_source
                && let Some(sel) = self.vec.pen.selected()
            {
                // ⚠️ O `source_for` devolve a fonte que a forma JÁ tem quando ela tem uma — e aqui o
                // artista pediu para TROCAR. O diálogo abre sempre, então o caminho é o directo.
                if let Some(source) = crate::texture_pattern_pick::pick_source(asset_db) {
                    // ⭐ **O tamanho a adoptar SE a forma ainda não tinha arte** — quem decide é o
                    // `apply`; aqui só se MEDE, porque é aqui que o `AssetDb` está. Ver a variante.
                    // ⭐ **A ARTE mede-se pela porta que ASSA** — e o mapa de afins e a geometria
                    // viva são os do quadro anterior, que é o que os seis sítios de pick desta
                    // shell já consomem: a forma que o artista acabou de apontar já lá está.
                    //
                    // ⏳ **MEDIDO e não curado** (auditoria de 2026-08-30): quando a forma JÁ tem
                    // padrão, o `apply_vec_set_fill_kind` preserva-o e descarta este `size` inteiro
                    // — é a forma do *«consumidor que projecta o valor fora»*. ⛔ Um guarda aqui
                    // codificaria um facto sobre **outra** função (*"o `source_for` só devolve
                    // não-`None` quando o slot já era padrão"*), e é essa acoplagem que diverge em
                    // silêncio. O custo é **por clique**, não por quadro.
                    let xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
                    let arte = crate::texture_pattern_pick::art_dims(
                        asset_db,
                        vec_scene,
                        &xf,
                        &self.vec.live_drawn,
                        sel,
                        &source,
                        &|id| {
                            ph2d_vec_entities::entities::object_selection_for(
                                sim,
                                vec_scene,
                                &self.vec.entities,
                                id,
                            )
                        },
                    );
                    let (size, _) =
                        crate::texture_pattern_pick::default_placement(vec_scene, sel, arte);
                    pending_texpat = Some((
                        slot,
                        crate::texture_pattern_edit::TexPatCmd::Source(source, size),
                    ));
                }
            }
            // ⭐⭐ **O SUJEITO VEIO NO ID DO CONTROLO** (plano 35, wave F): cada secção tem os seus,
            // então não há preferência a coagir nem alvo a resolver — *o gesto diz em quem escreve*.
            if let Some((slot, cmd)) = pending_texpat {
                crate::texture_pattern_edit::apply(vec_scene, &self.vec.pen, slot, cmd);
            }
            if let Some(deg) = pending_vec_grad_angle {
                crate::input_dispatch::apply_vec_set_grad_angle(vec_scene, &self.vec.pen, deg);
            }
            if pending_vec_grad_add {
                crate::input_dispatch::apply_vec_grad_add_point(vec_scene, &self.vec.pen);
            }
            if pending_vec_grad_remove {
                self.vec.grad_selected = crate::input_dispatch::apply_vec_grad_remove_point(
                    vec_scene,
                    &self.vec.pen,
                    self.vec
                        .grad_selected
                        .and_then(ph2d_vec_render::GradHandle::point),
                )
                .map(ph2d_vec_render::GradHandle::Point);
            }
            if let Some(v) = pending_vec_grad_influence {
                crate::input_dispatch::apply_vec_grad_influence(
                    vec_scene,
                    &self.vec.pen,
                    self.vec
                        .grad_selected
                        .and_then(ph2d_vec_render::GradHandle::point),
                    v,
                );
            }
            if let Some(v) = pending_vec_grad_jitter {
                crate::input_dispatch::apply_vec_grad_jitter(
                    vec_scene,
                    &self.vec.pen,
                    self.vec
                        .grad_selected
                        .and_then(ph2d_vec_render::GradHandle::point),
                    v,
                );
            }
            if pending_vec_grad_add_stop {
                self.vec.grad_selected =
                    crate::input_dispatch::apply_vec_grad_add_stop(vec_scene, &self.vec.pen)
                        .map(ph2d_vec_render::GradHandle::Stop)
                        .or(self.vec.grad_selected);
            }
            if let Some(a) = pending_vec_align {
                crate::input_dispatch::apply_vec_align(vec_scene, &self.vec.pen, &vec_xf_ops, a);
            }
            if let Some(d) = pending_vec_distribute {
                crate::input_dispatch::apply_vec_distribute(
                    vec_scene,
                    &self.vec.pen,
                    &vec_xf_ops,
                    d,
                );
            }
            if pending_vec_grad_remove_stop
                && let Some(si) = self
                    .vec
                    .grad_selected
                    .and_then(ph2d_vec_render::GradHandle::stop)
            {
                // Only an interior stop can be removed; a no-op otherwise keeps the
                // current selection (endpoint handles aren't removable stops).
                self.vec.grad_selected = crate::input_dispatch::apply_vec_grad_remove_stop(
                    vec_scene,
                    &self.vec.pen,
                    Some(si),
                )
                .map(ph2d_vec_render::GradHandle::Stop);
            }
            if pending_vec_pivot_edit {
                // Arma "Set Center": a próxima pressão no canvas põe a ORIGEM ali.
                self.vec.pivot_edit = true;
            }
            // ⭐⭐⭐ **A FERRAMENTA ARMA-SE AQUI** — antes de ela republicar o espelho (`vec_cfg`
            // abaixo), senão a escrita da aresta do foco seria revertida no mesmo quadro.
            //
            // ⛔⛔ Ordem do dono (2026-09-09): *«ao seleccionar o osso … o botão Transform é
            // seleccionado»*. Quem o pede é a aresta lá em baixo, que não pode tocar em `gfx.tools`
            // (ele está emprestado a `sim`/`hero`).
            if let Some(acao) = self.skeleton.bone_arm_pending.take() {
                vector_bridge::arm_bone(tools, acao);
            }
            let vec_cfg = vector_bridge::dispatch(
                hero,
                tools,
                vec_scene,
                &mut self.vec.pen,
                &mut self.vec.shape,
                &mut self.vec.pencil,
                vec_px_to_world,
                self.vec.grad_selected,
                &vec_xf_ops,
                sim,
                &self.vec.entities,
                self.vec.pivot_edit,
                self.vec.snap,
                self.texpat_lock_aspect,
                self.texpat_gap_link,
                self.texture_pattern_live.tiles(),
            );
            // ⭐ **Stroke** (plano 34): dar/tirar o traço da forma selecionada. **Honrar e só depois
            // publicar**, a mesma ordem do `resize_box` — publicar antes deixaria a caixa a mostrar
            // o estado ANTERIOR por um quadro, e o artista veria o clique *"não pegar"*.
            //
            // ⚠️ **Aqui, e não no dreno de baixo:** a ficha do traço novo sai do `vec_pen`, que o
            // `dispatch` acabou de sincronizar com a ferramenta (`pen.set_style`). Um sítio mais
            // tarde leria a ficha do quadro anterior.
            if pending_stroke_present {
                crate::vec_stroke_present::toggle(vec_scene, &self.vec.pen, vec_px_to_world);
            }
            ph2d_panel_vector::state::set_stroke_present(
                crate::vec_stroke_present::selected_stroke_present(vec_scene, &self.vec.pen),
            );
            // ⭐ **A TINTA do traço** (plano 35, wave D) — publicada AQUI, ao lado da irmã, e não no
            // `vector_bridge`: o dreno acima pode ter acabado de dar ou tirar o traço, e uma
            // publicação anterior a ele mostraria a fileira de um traço que já não existe.
            // ⭐ **O DIAGNÓSTICO da selecção** (`PH2D_PATTERN_LOG=1`) — aqui, depois dos drenos,
            // porque é aqui que a cena é o que o artista vê. Por EVENTO: só quando a selecção muda.
            crate::texture_pattern_edit::log_selection(vec_scene, &self.vec.pen);
            // ⭐ A lei do PINCEL da selecção (plano 36, W4) — `None` esconde a secção *Brush*.
            ph2d_panel_vector::set_current_brush(
                // ⚠️ O `sel` fica ATADO até ao fim: a pergunta *"tem arte?"* precisa do
                // ANFITRIÃO (a recusa é sobre pertença), e a redacção anterior consumia-o no
                // primeiro `and_then`.
                self.vec.pen.selected().and_then(|sel| {
                    vec_scene
                        .path(sel)
                        .and_then(|p| p.stroke.as_ref())
                        .and_then(ph2d_vec_scene::StrokeSpec::brush)
                        .map(|b| ph2d_panel_vector::BrushRow {
                            // ⚠️ *"Tem arte?"* é uma pergunta sobre a CENA, não sobre o campo: um id que
                            // aponta para uma forma apagada é um pincel sem arte, e o rótulo do botão
                            // tem de o dizer.
                            //
                            // ⛔⛔ **E ela vai pela porta que RESOLVE** (auditoria de 2026-08-30). Um
                            // `scene.path(a).is_some()` escrito aqui é uma SEGUNDA resposta: desde que
                            // a arte pode ser um grupo, a recusa é sobre **pertença** — o caminho pode
                            // existir e a arte ser recusada na mesma, e o botão dizia *"Change
                            // Shape…"* sobre um traço que pinta a cor de recurso, sem mensagem.
                            has_art: b.art.is_some_and(|a| {
                                !ph2d_vec_art_live::pattern::art_members(sel, a, &|id| {
                                    ph2d_vec_entities::entities::object_selection_for(
                                        sim,
                                        vec_scene,
                                        &self.vec.entities,
                                        id,
                                    )
                                })
                                .is_empty()
                            }),
                            spacing: b.spacing,
                            scale: b.scale,
                            offset: b.offset,
                            rotation_deg: b.rotation_deg,
                            flip: b.flip,
                        })
                }),
            );
            ph2d_panel_vector::state::set_stroke_paint_kind(
                crate::vec_stroke_paint::selected_stroke_paint_kind(vec_scene, &self.vec.pen),
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
            ph2d_app_motion::motion_bridge::publish_shapes(
                motion,
                sim,
                vec_scene,
                &self.vec.entities,
                &vec_xf_ops,
                hero.gizmo.selection,
            );
            // ADR-0154: `source.shape` geometry is NOT published here — it is published
            // by the bridge POST-drain, pre-cook (a param edit is drained inside the
            // bridge, so publishing here would set the pre-edit key while the cook reads
            // the post-edit key ⇒ a 1-frame vanish = flicker). See `motion_bridge`.
            // doc 86 §2: and the engine OBJECTS (named sprites) into the same
            // external table — AFTER shapes (which clears it), so the cook sees
            // both curves and objects. The atlas resolves each sprite's tile.
            // ⚠️ **As DUAS lojas de aparência**, não só o atlas — ver `Appearance`: um sprite
            // KTX2 assado era fonte INVISÍVEL só porque quem o resolvia não estava em mão
            // aqui dentro, e ele está: é o mesmo `renderer` de onde sai o atlas.
            let cooked = |id| renderer.cooked_texture_id(id);
            ph2d_app_motion::motion_bridge::publish_objects(
                motion,
                sim,
                ph2d_app_motion::motion_bridge::Appearance {
                    atlas: renderer.atlas(),
                    cooked: &cooked,
                },
                // ⚠️ O relógio, porque o canal DESLOCADO resolve um param que pode vir de um
                // fio — e um param conduzido só tem valor num INSTANTE (doc 58).
                self.playhead.time(),
            );
            // ...and the CURSOR, last, into the same table (`ph2d_nodegraph::external`).
            // It is not a document value — it is an editor input that changes every
            // frame — so publishing it is what lets `motion.look_at` aim at the mouse
            // without the node learning what a window or a camera is. Last, because
            // `publish_shapes` CLEARS and the objects append; and in the reserved `$`
            // namespace, which the artist-name publishes above refuse.
            ph2d_app_motion::motion_bridge::publish_cursor(
                motion,
                camera,
                self.last_cursor,
                hero.view.center_split,
                surface.size(),
            );
            ph2d_app_motion::motion_bridge::dispatch(
                hero,
                tools,
                motion,
                &mut self.playhead,
                self.fixed_step.fixed_dt(),
                self.last_pointer,
                toasts,
                surface.gpu(),
            );
            // O grafo gritou — o shell é quem publica (ADR-0075: o produtor não chama
            // ninguém). ⚠️ **Este produtor pousa UM QUADRO atrás dos outros dois**, e o
            // fato fica NOMEADO em vez de escondido: o dispatch de Motion roda depois de
            // os consumidores lerem, então um `pulse.signal` chega ao toast no quadro
            // seguinte. O duplo-buffer da outbox torna isso *atrasado, nunca perdido* — a
            // rede, não a licença. Fechar o vão é MOVER a leitura dos consumidores para
            // baixo deste dispatch, o que reordena uma sequência gateada e é decisão
            // própria (o `toasts` tem de continuar vivo lá).
            // ⚠️ **A LEITURA acontece AQUI, e não dentro do dispatch, porque este é o ponto
            // onde TODA rota de cook converge.** Ela morava no laço de tiques da bomba de
            // CPU — correto, e mudo: a rota da GPU **híbrida** marcha por outra porta e
            // devolve `Handled`, então aquele laço nem roda. Medido na cena `=26`, que planeja
            // híbrida (boundaries `[5, 4]`, 4 estágios): o grafo cozinhava, desenhava e não
            // gritava nada — com a suíte verde, porque todo gate dirigia a porta de sinks.
            //
            // ⚠️ **A LEI mora aqui junto com a leitura**, pelo mesmo motivo: perguntá-la
            // dentro do dispatch obrigaria cada rota a lembrar-se dela. Um scrub re-cozinha o
            // grafo e não pode gritar — um sinal é travessia de play para a frente.
            if clock_forward::clock_is_playing_forward(&self.playhead, self.timeline_signals.jumped)
            {
                ph2d_app_motion::motion_bridge::signals::collect_signals(motion);
            }
            for sig in motion.signals_out.drain(..) {
                self.signals.publish(ph2d_runtime::Signal::from_motion(
                    &sig.name, sig.tick, sig.rows,
                ));
            }
            // Mirror the tool's mode + shape params for the input dispatch's
            // pen-vs-shape routing (the downcast lives in the bridge).
            self.vec.draw_config = vec_cfg;

            // ADR-0114 W2 T2.17 (ready-to-smoke): ativar a tool Flip num documento
            // VAZIO cria um objeto inicial (1 camada) pra desenhar na hora — sem
            // ele o `bake_stroke` sai (não há objeto). Só na borda de ativação;
            // um doc já povoado (ex. PH2D_FLIP_DEMO) não é tocado.
            {
                let now_active = tools
                    .active()
                    .is_some_and(|t| t.id() == ph2d_editor_core::ToolId::new("flip"));
                if now_active && !self.flip_state.active && flip.is_empty() {
                    let oid = flip.push_object("Flip");
                    if let Some(obj) = flip.object_mut(oid) {
                        self.flip_state.active_layer = Some(obj.add_layer("Layer 1"));
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
            // ⚠️ Distinct from `ph2d_app_physics::bridge::dispatch::dispatch` far above, which steps
            // the SIMULATION at the Playhead tick. Two bridges, two phases.
            self.show_colliders = ph2d_app_physics::panel_bridge::dispatch(
                hero,
                physics,
                self.show_colliders,
                &mut self.physics.interaction,
                // W25: a corrida gravada é um fato do DOCUMENTO, e este é o
                // painel do documento. A §14 mostra o mesmo par de números; as
                // duas vistas caem na mesma porta (`run_stash`).
                ph2d_app_physics::panel_bridge::RunTapes {
                    live: &mut self.player_tape,
                    stash: &mut self.discarded_run,
                    fixed_dt: self.fixed_step.fixed_dt(),
                },
            );
            // ADR-0161 W4: a peça de modelagem 3D é uma ENTIDADE, e esta é a ponte
            // que a mantém assim — nasce no mundo, aparece na Hierarquia, e as
            // edições do painel escrevem no COMPONENTE. Inerte sem o smoke armado.
            // ⭐ **O pill é a porta de armar.** A visibilidade do painel É o interruptor do
            // módulo: enquanto a única entrada era `PH2D_FIELD_SMOKE`, ele não existia
            // para quem abre o app.
            // ⭐ **QUEM TOMA O CANVAS LIBERTA QUEM O TINHA** (W40). Enio, 2026-08-22: *"o modo
            // Modelagem nunca é desativado e não consigo usar nenhum outro modo do app… Não consigo
            // esculpir nada pois o modo de modelagem permanece interferindo."*
            //
            // ⚠️ Fecha-se o **painel**, e não se desarma em silêncio: o pill *é* o interruptor do
            // módulo (a linha abaixo), então um desarme invisível deixaria o botão aceso a mentir.
            // A lei (borda, não estado contínuo) e o porquê estão em `ph2d_app_field3d::mode`.
            {
                let clay_on = {
                    #[cfg(feature = "sculpt3d")]
                    {
                        sculpt3d
                            .as_ref()
                            .is_some_and(ph2d_app_sculpt3d::Sculpt3dScene::clay_on_screen)
                    }
                    #[cfg(not(feature = "sculpt3d"))]
                    {
                        false
                    }
                };
                let owner = ph2d_app_field3d::mode::Owner {
                    tool: tools.active().map(ph2d_editor_core::Tool::id),
                    clay: clay_on,
                };
                if ph2d_app_field3d::mode::note_owner(owner.clone())
                    && hero.is_panel_visible(ph2d_panel_model3d::PANEL_ID)
                {
                    hero.panel_visibility
                        .insert(ph2d_panel_model3d::PANEL_ID, false);
                    toasts.push(ph2d_editor_core::Toast::info(
                        "Modelling stepped aside for the other tool",
                    ));
                }
                // ⭐⭐ **E a metade SIMÉTRICA**: abrir o MODEL tira o barro da tela — e, desde
                // 2026-08-31, também **larga a ferramenta em mãos**.
                //
                // ⛔⛔ **A lei dizia-se «duas metades simétricas» e só uma soltava uma FERRAMENTA.**
                // Report do Enio: *«se abro Nodes e depois Model, o grafo de Nodes persiste»*. Ele
                // chegou lá pela aba nova, mas o defeito é do módulo e é anterior a ela: abrir o
                // MODEL pelo menu *Window* com o Motion em mãos deixava a `motion_bridge` a
                // reabrir `motion_graph`/`motion_params` **a cada quadro**, porque nada largava a
                // ferramenta. *A metade que faltava não era um caso — era o outro lado da lei.*
                //
                // ⚠️ **O edge é lido UMA vez e fora do `cfg`**: o `model_just_opened` CONSOME a
                // transição (ele troca o valor guardado), então uma segunda chamada no mesmo
                // quadro leria `false` — e, enquanto ele vivia dentro do `#[cfg(sculpt3d)]`, uma
                // build sem aquela feature nunca o avançava.
                let model_opened = ph2d_app_field3d::mode::model_just_opened(
                    hero.is_panel_visible(ph2d_panel_model3d::PANEL_ID),
                );
                //
                // ⚠️ A decisão E o re-baseline vivem os dois no `model_takes_the_canvas` — são um
                // acto só, e separá-los deixava uma mutação sobreviver com o produto em ciclo.
                if model_opened
                    && let Some(neutral) = tools.default_tool_id()
                    && ph2d_app_field3d::mode::model_takes_the_canvas(&owner, &neutral)
                {
                    tools.set_active(&neutral);
                    self.title_dirty = true;
                    toasts.push(ph2d_editor_core::Toast::info("Modelling took the canvas"));
                }
                // ⚠️ A saída do BARRO é a **porta do próprio módulo de escultura**
                // (`toggle_clay`), nunca uma escrita aqui: ela conhece a ordem do ciclo (sair do
                // barro vai para a LUZ, não para o desligado), e essa ordem é uma decisão de
                // produto com um dono.
                #[cfg(feature = "sculpt3d")]
                if model_opened
                    && let Some(scene) = sculpt3d.as_mut()
                    && scene.clay_on_screen()
                {
                    let label = scene.toggle_clay();
                    eprintln!("[field3d] o MODEL abriu; a escultura cedeu -> {label}");
                }
            }
            // ⭐⭐ **UM PROJETO QUE TRAZ UMA PEÇA ABRE O PAINEL** (W45) — a resposta à pergunta que o
            // load deixou. ⚠️ É aqui e não no load porque **o mundo vive no `gfx`**, e o load corre
            // sem janela: perguntar lá daria *"não há peça"* sempre. Mesma forma (e mesma razão
            // escrita) do `sculpt3d_install_pending` do módulo irmão.
            if ph2d_app_field3d::smoke::take_open_if_part_request()
                && ph2d_app_field3d::scene::world_has_a_part(sim.world_mut())
            {
                ph2d_app_field3d::smoke::ask_open_panel();
            }
            ph2d_app_field3d::smoke::set_armed_by_panel(
                hero.is_panel_visible(ph2d_panel_model3d::PANEL_ID),
            );
            // ⭐⭐⭐ **A FILA É O CABEÇALHO DA ÁREA** — as vistas e a câmera saíram do painel e
            // pintam-se na fila de ferramentas, que já é uma região da área e já subtrai a altura
            // que subtrai (`ph2d_panel_model3d::area_bar`).
            //
            // ⚠️ **Escrito em TODO quadro, desarmado incluído** — é a mesma lei do transbordo do
            // `⋯`: quem fecha o módulo deixa de contribuir e a fila volta ao que era **no mesmo
            // quadro**. Sem o ramo vazio, nove chips ficavam na fila a despachar para um painel
            // que já não está lá.
            let model_armed = hero.is_panel_visible(ph2d_panel_model3d::PANEL_ID);
            ph2d_panel_model3d::publish_area_bar(&mut hero.store, model_armed);
            // ⭐ A seleção do app é a do gizmo 3D: clicar numa linha da Hierarquia é o que faz as
            // setas aparecerem no objeto. Uma seleção própria deste módulo seria uma segunda ideia
            // de "o que está selecionado" no mesmo app.
            // ⭐ Um clique na peça (ou a peça a nascer) pede uma seleção. É a MESMA porta que a
            // Hierarquia usa — uma seleção própria deste módulo seria uma segunda ideia de "o que
            // está selecionado" dentro do mesmo app.
            if let Some(req) = ph2d_app_field3d::scene::ecs_bridge(
                sim,
                hero.gizmo.selection,
                &hero.gizmo.extra_selection,
                vec_scene,
            ) {
                // ⭐ **A lei mora numa porta só** (`field3d_scene::apply`) — o gate chama a MESMA.
                ph2d_app_field3d::scene::apply(&mut hero.gizmo, req);
            }
            // O painel de TOKENS (plano UI/UX W6), na MESMA fase e pela mesma razão: um painel de
            // MUNDO, cuja visibilidade é do artista. ⚠️ Ele tem de correr DEPOIS do dispatch de
            // eventos (o intent de Reset é enfileirado ali) e ANTES do paint (senão o frame
            // pintaria a cor de antes do clique e o picker piscaria de volta).
            if tokens_bridge::dispatch(hero, toasts) {
                self.title_dirty = true;
            }
            // O painel da cena 3D (ADR-0150 W12), na MESMA fase e pela mesma
            // razão dos dois acima: depois do dispatch de eventos (os intents
            // são enfileirados ali) e ANTES do paint (senão o frame pintaria o
            // estado de antes do clique e o chip piscaria de volta).
            // ⚠️ O retorno é o pedido de BAKE, e ele arma o MESMO campo que o
            // `Shift+B` — uma porta, dois pedintes. O gesto é consumido no
            // dispatch do frame SEGUINTE (o `bake::drain` roda mais cedo neste),
            // exatamente como o do teclado.
            #[cfg(feature = "sculpt3d")]
            for req in ph2d_app_sculpt3d::panel_bridge::dispatch(hero, sculpt3d.as_mut()) {
                match req {
                    ph2d_app_sculpt3d::Sculpt3dFrameRequest::Bake => {
                        self.sculpt3d_req.bake_request = true;
                    }
                    ph2d_app_sculpt3d::Sculpt3dFrameRequest::AlphaFromSprite => {
                        self.sculpt3d_req.alpha_request = true;
                    }
                }
            }
            // O ARRASTO da tira (mover a chave / esticar o hold): o painel enfileirou o
            // pedido no pen-up do frame anterior; aqui ele vira documento — ANTES do
            // publish, senão o snapshot deste frame descreveria a tira de antes do gesto e
            // a célula piscaria de volta por um frame.
            // (o retorno diz se o documento mudou; ninguém precisa dele aqui — o undo é
            // GLOBAL e por DIFF: `post_frame_undo` compara o `ProjectState`, do qual o
            // `FlipDoc` faz parte. É o mesmo motivo pelo qual o drain do `PanelEvent`
            // logo acima também ignora o seu.)
            let _ = ph2d_app_flip::strip_drag::apply_strip_intents(
                flip,
                self.flip_state.active_layer,
                &mut self.flip_state.strip,
            );
            let (flip_active, flip_style) = ph2d_app_flip::bridge::publish(
                hero,
                tools,
                flip,
                self.flip_state.active_layer,
                &self.playhead,
                &self.flip_state.strip,
            );
            self.flip_state.active = flip_active;
            self.flip_state.style = flip_style;
            // O anel do pincel (W5): mostra no canvas o tamanho do que vai acontecer.
            // Depois do publish (o estilo do frame já está no cache) e na cena de
            // overlay, como o anel do Painter.
            ph2d_app_flip::cursor::draw_flip_cursor(
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
            // entity's Transform (see `ph2d_app_physics::overlay::outline::joint_marks`).
            //
            // W-J1: a VIEW, not just the anchor pair — the drawing reads the
            // very `JointDesc` the solver was handed (kind, limits, motor,
            // length) plus the live poses, so the glyph cannot describe a joint
            // the solver is not enforcing.
            let joint_views: Vec<ph2d_physics_ecs::JointView> = physics.joint_views().collect();
            // A arena que as faixas das views indexam (W-Pulley W1) — a MESMA
            // fatia que o solver está usando neste frame.
            let joint_wheels = physics.pulley_wheel_arena().to_vec();
            // E o ângulo de cada uma — o giro que faz uma roda parecer uma roda.
            let joint_spins = physics.pulley_wheel_spins().to_vec();
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
            // W-Probes: o que os sensores do player olharam no ULTIMO tique, do
            // UNICO dono do fato (a ponte). Ate isto, nada na tela dizia onde a
            // perna, o flanco, a quina ou o teto do agachar procuram.
            let probes = physics.player_probe_marks().to_vec();
            ph2d_app_physics::overlay::outline::draw(
                self.show_colliders,
                velocity_at_rest,
                sim,
                &joint_views,
                &joint_wheels,
                &joint_spins,
                joint_gravity,
                // W-J3: o limite que o arrasto está posando AGORA, para o
                // fantasma de B. Lido do componente (o arrasto já escreveu nele
                // neste frame), então a silhueta e o arco mostram o mesmo número.
                self.physics
                    .joint_anchor_drag
                    .and_then(|d| d.posed_limit(sim)),
                // W-J4: a banda elástica, se um gesto de criar está em voo (e o
                // corpo A ainda existe — apagá-lo sob o gesto o invalida).
                ph2d_app_physics::joint_draw::body_alive(sim, self.physics.joint_draw)
                    .then(|| ph2d_app_physics::joint_draw::band(self.physics.joint_draw))
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
                    .then(|| self.physics.interaction.aim_radius())
                    .flatten()
                    .map(|r| (pointer_world, r)),
                // O campo VIVO, do ÚNICO dono do fato (a ponte).
                physics.attract_marks(),
                // E o último estouro, enquanto o flash dura.
                self.blast_flash.map(|(c, r, _)| (c, r)),
                &contacts,
                &flashes,
                &waterlines,
                &probes,
                &triggered,
                // W20: a descida em curso, do ÚNICO dono do fato (a ponte). Sem
                // isto uma prancha fantasma é indistinguível de uma sólida, que
                // é a forma como toda esta classe de defeito ficou silenciosa.
                physics.any_player_is_dropping(),
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
            // **O ANEL DO PINCEL 3D** (ADR-0150 W12). Ele é desenhado no PONTO DE
            // ACERTO reprojetado, então ele é ao mesmo tempo a mira e o
            // instrumento: se ele não estiver debaixo do mouse sobre o barro, a
            // fiação do pick está errada e dá para VER — que é a única coisa que
            // as sondas headless não alcançam.
            //
            // ⚠️ Sob painel ele não é desenhado: o ponteiro ali não é da cena (o
            // `pointer_down` já recusa pela MESMA porta), e uma mira sobre o
            // chrome prometeria um gesto que o clique não faz.
            #[cfg(feature = "sculpt3d")]
            if let Some(scene) = sculpt3d.as_ref() {
                let (px, py) = self.last_pointer;
                let over_panel = hero
                    .store
                    .panel_rect(ph2d_editor_core::screens::hero::ids::SCULPT3D_PANEL)
                    .is_some_and(|r| r.contains(px, py));
                if !over_panel && let Some(mark) = scene.cursor_mark(px, py) {
                    use ph2d_vector::{Affine, Brush, Color, Stroke};
                    let rgba = if mark.on_surface {
                        ph2d_app_sculpt3d::ON_SURFACE_RGBA
                    } else {
                        ph2d_app_sculpt3d::OFF_SURFACE_RGBA
                    };
                    vector_scene.inner_mut().stroke(
                        // ⚠️ `Affine::IDENTITY`: no Vello o transform do `stroke`
                        // MULTIPLICA a largura — o caminho já está em pixels.
                        &Stroke::new(1.5), // LITERAL-PX-OK: chrome de overlay, espessura de tela
                        Affine::IDENTITY,
                        &Brush::Solid(Color::new(rgba)),
                        None,
                        &mark.path,
                    );
                }
            }
            // A TRAJETÓRIA do objeto selecionado (ADR-0141): um binding Position guarda
            // uma curva, e sem desenhá-la o artista vê o objeto aparecer noutro lugar a
            // cada frame sem ter onde pegar o caminho. Os PONTOS são um por quadro, e o
            // espaçamento entre eles é a velocidade. No-op sem seleção ou sem Position.
            // ⚠️ **Só na aba Keys** (Enio, 2026-07-31) — a trajetória é do CLIP ATIVO, e
            // fora dali quem dirige o objeto é a PILHA; o MESMO booleano que sola o clip
            // (`keys_mode`) decide se há alça a oferecer. Um `true` literal aqui deixaria
            // todo gate do overlay verde com a alça fantasma de volta na tela — daí o
            // arch-gate `the_motion_path_is_offered_only_on_the_keys_tab`.
            ph2d_app_motion::motion_path_overlay::draw(
                self.timeline.keys_mode,
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
                    .and_then(|oid| self.flip_state.entities.get(&oid).copied())
                    .map(ph2d_ecs::Entity::from_bits)
                    .filter(|e| sim.world().get_entity(*e).is_ok())
                    .map_or(ph2d_vec_scene::Xform::IDENTITY, |e| {
                        ph2d_flip_entities::transform::object_xform(sim, e)
                    });
                // W8/§4.C: o realce fala a linguagem do DOMÍNIO — halo de traço (Stroke),
                // dots (Point), ou halo do PEDAÇO + preview de hover (Segment).
                let overlay_domain = match flip_style.map(|s| s.edit_domain) {
                    Some(ph2d_tool_flip::EditDomain::Point) => {
                        ph2d_app_flip::selection_overlay::OverlayDomain::Point
                    }
                    Some(ph2d_tool_flip::EditDomain::Segment) => {
                        ph2d_app_flip::selection_overlay::OverlayDomain::Segment
                    }
                    _ => ph2d_app_flip::selection_overlay::OverlayDomain::Stroke,
                };
                let hover = self
                    .flip_state
                    .segment_hover
                    .as_ref()
                    .map(|(si, pts)| (*si, pts.as_slice()));
                ph2d_app_flip::selection_overlay::draw_flip_selection(
                    flip_active,
                    matches!(
                        flip_style.map(|s| s.mode),
                        Some(ph2d_tool_flip::FlipMode::Edit)
                    ),
                    overlay_domain,
                    hover,
                    flip,
                    &self.playhead,
                    self.flip_state.active_layer,
                    &l2w,
                    camera,
                    surface.size(),
                    vector_scene,
                );
                // A caixa do marquee (W6.1) — em px de tela, como o realce.
                ph2d_app_flip::selection_overlay::draw_flip_marquee(
                    self.flip_state.edit_gesture,
                    vector_scene,
                );

                // **§12 Sockets / Named Anchors** (spec Sprite 07 §7.6) — os marcadores só
                // aparecem com a seção EXPANDIDA, senão todo sprite com âncoras ficaria coberto
                // de cruzes. Sem eles a §12 é um formulário que não mexe em nada na tela.
                // **O gizmo dos deformadores de quadrilátero** — o contorno, os braços e
                // as alças. Lê o retrato publicado no prólogo (a tool e a selecção já
                // foram decididas lá), então aqui não há regra nenhuma, só tinta.
                //
                // ⛔⛔⛔ **ELE JÁ ESTEVE ~2 000 LINHAS ABAIXO, E O GIZMO SUMIU DO ECRÃ.**
                // (report do Enio, 2026-09-08: *«nessa última rodada vc sumiu com o gizmo do
                // Bezier Warp»*.) A mudança tinha sido feita para o pôr por cima do documento
                // vectorial e das formas vivas do Motion, que codificam na MESMA cena depois
                // daqui — mas essa ordem foi **inferida de números de linha e nunca medida**, e
                // a que se pagou foi real.
                //
                // ⚠️ **O mecanismo do desaparecimento NÃO está nomeado**, e as quatro hipóteses
                // óbvias foram descartadas com medição: o sítio novo corre (contagem de chavetas
                // que ignora strings e comentários: só `impl` → `fn` → `if let Some(hero)`), a
                // cena não é reposta nem trocada entre os dois pontos, o `camera` e o `surface`
                // não são sombreados, e o retrato é publicado **uma vez só** (não é um `take`).
                // ⇒ *uma mudança de sítio sem uma medição do que o sítio garante é um palpite*,
                // e quem a repetir começa por instrumentar o quadro, não por mover a linha.
                if let Some(v) = ph2d_app_motion::warp_gizmo::view() {
                    let port = ph2d_app_motion::warp_gizmo::param_port(motion, v.node);
                    ph2d_app_motion::warp_overlay::draw_warp_gizmo(
                        true,
                        &v,
                        &port,
                        camera,
                        hero.view.center_split,
                        surface.size(),
                        vector_scene,
                    );
                } else {
                    // ⚠️ **O outro lado da sonda, e ele é o que distingue os dois casos.** Sem
                    // esta linha, um `PH2D_WARP_DIAG=1` que não imprime nada lê-se como *«a sonda
                    // não está a correr»* — que é exactamente a ambiguidade que fez duas curas
                    // seguidas serem palpites.
                    ph2d_app_motion::warp_overlay::diag(
                        "nao ha' retrato publicado (`view()` = None)",
                    );
                }
                anchor_overlay::draw_anchor_marks(
                    !hero
                        .store
                        .is_collapsed(ph2d_editor_core::ids::INSP_LIVE_ANCHOR_SECTION),
                    sim.world(),
                    hero.gizmo.selection,
                    // ⚠️ A linha ABERTA vem do PAINEL — é o canal que o gizmo estreou. Ela é o
                    // que decide quem ganha alças; sem ela o canvas não sabe a quem obedecer.
                    ph2d_panel_inspector::open_anchor_row(),
                    hero.project.pixels_per_meter,
                    camera,
                    surface.size(),
                    vector_scene,
                    // Reborrow por `paint_ctx`, como o rótulo do overlay de física — o
                    // `text_system` já está emprestado desde o começo do frame.
                    paint_ctx.text,
                );

                // ⭐ **O ANEL do objeto vazio** (Enio, 2026-08-26) — um objeto sem geometria não
                // emite pixel nenhum, e sem marca o artista não sabe onde ele está. A pergunta
                // *«está vazio?»* é a MESMA que dimensiona a caixa do gizmo (`group_gizmo_view`).
                empty_object_overlay::draw_empty_object_marks(
                    sim,
                    hero.project.pixels_per_meter,
                    hero.theme,
                    camera,
                    surface.size(),
                    vector_scene,
                );

                // Tween v2 — a correção de pares: os dois desenhos-chave sobrepostos + as
                // linhas de par (pela confiança) + órfãos, no MESMO `l2w` do objeto. Só
                // desenha com a sessão Pairs aberta.
                ph2d_app_flip::tween_overlay::draw(
                    flip_active && self.flip_state.strip.tween_correct.is_some(),
                    self.flip_state.strip.tween_correct.as_ref(),
                    &l2w,
                    camera,
                    surface.size(),
                    vector_scene,
                );

                // Gap Closure (doc 06 §8): os helpers ao vivo — cada vão que o alcance
                // atual fecha, desenhado onde o clique vai fechá-lo. Os segmentos vêm do
                // worker (`flip_gap_live`, coords de ARTE); a pergunta do modo é a MESMA
                // porta do tick, e a projeção é a MESMA cadeia do render (l2w ∘ pose).
                ph2d_app_flip::gap_overlay::draw(
                    ph2d_app_flip::gap_live::wants_gap_helpers(flip_active, flip_style),
                    &self.flip_state.gap.segments,
                    &l2w,
                    // A MESMA pose que a autoria dobra (`flip_transform::active_pose`) —
                    // função livre porque aqui `self.gfx` está destruturado.
                    ph2d_flip_entities::transform::active_pose(
                        flip,
                        self.flip_state.active_layer,
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
                &mut self.vec.text_edit,
                self.vec.draw_config.mode,
                &self.vec.pen,
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
                let in_text_mode = self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Text;
                let sel: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
                let target = crate::vec_text::panel_text_target(
                    sim,
                    &self.vec.entities,
                    &sel,
                    self.vec.text_edit.as_ref(),
                );
                let visible = in_text_mode || target.is_some();
                ph2d_panel_vector::set_current_text_visible(visible);
                ph2d_panel_vector::set_current_text(target.as_ref().map(|t| t.text.clone()));
                // Família / alinhamento: do alvo; sem alvo (modo Text sem sessão), os
                // defaults correntes da shell (o que a próxima sessão vai usar).
                let family = target
                    .as_ref()
                    .map_or_else(|| self.vec.text.family.clone(), |t| t.family.clone());
                ph2d_panel_vector::set_current_text_font(
                    visible.then(|| crate::vec_font::display_name(family.as_deref())),
                );
                ph2d_panel_vector::set_current_text_align(
                    visible.then(|| target.as_ref().map_or(self.vec.text.align, |t| t.align)),
                );
                // ⚠️ A fileira Width lê o ALVO quando há um selecionado, e o default da shell
                // quando não há — a mesma regra do Align logo acima. Sem isto o painel mostraria
                // "Auto" sobre um texto que reflui, e o 1º clique em Fixed não mudaria nada.
                ph2d_panel_vector::set_current_text_wrap(
                    target.as_ref().map_or(self.vec.text.wrap, |t| t.wrap_width),
                );
                // Semente dos sliders: só quando o ALVO muda (senão brigaria com o drag).
                let target_id = target.as_ref().map(|t| t.id);
                if target_id != self.vec.text_last_target {
                    self.vec.text_last_target = target_id;
                    ph2d_panel_vector::set_current_text_seed(target.as_ref().map(|t| t.sliders));
                }
                // ⛔ **E o FACTO que a fileira Weight precisava, e que ninguém publicava:** *esta
                // fonte expõe `wght`?* Sem `fvar` o `skrifa` ignora a localização de eixo, então
                // numa fonte ESTÁTICA aquele slider era pintado e **inerte**.
                //
                // ⚠️ **Ele NÃO é derivável dos `slots` abaixo**, e a tentação é exactamente o que
                // estava refutado: aquela lista é *"os eixos ALÉM do peso"*, então uma fonte
                // variável **só de peso** (a `Cantarell-VF` desta máquina, `fvar = ['wght']`)
                // publica-a vazia e ainda assim tem um Weight vivo. Duas perguntas, duas
                // publicações.
                //
                // ⚠️ Calculado **dentro do `visible`**, e junto dos eixos, porque
                // `has_weight_axis` resolve a família — e resolver uma família do sistema constrói
                // o catálogo do fontique (50–200 ms). Fora do modo Text isso seria pago por quadro
                // para responder a uma pergunta que a secção escondida não faz.
                ph2d_panel_vector::set_current_text_has_weight(
                    visible && crate::vec_font::has_weight_axis(family.as_deref()),
                );
                // Eixos de variação da fonte do alvo (nome + range + valor).
                let slots = if visible {
                    let descs = crate::vec_font::variation_axes(family.as_deref());
                    let values: Vec<f32> = target.as_ref().map_or_else(
                        || self.vec.text.extra_axes.iter().map(|(_, v)| *v).collect(),
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
            if self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Text
                && ph2d_panel_vector::take_want_font_previews()
            {
                ph2d_panel_vector::set_current_text_font_previews(
                    crate::vec_font_preview::build_previews(),
                );
            }

            // ADR-0110 — a árvore do editor é a Hierarquia. Reconcilia documento e
            // entidades (path novo ⇒ entidade; entidade apagada ⇒ path), projeta a
            // ordem de z da árvore na pilha, e lê visibilidade/trava herdadas.
            ph2d_vec_entities::entities::sync(sim, vec_scene, &mut self.vec.entities);
            // ⭐⭐⭐ **O BALDE** (plano 40): a entidade do preenchimento acabou de nascer — é agora
            // que a RECEITA (a semente) lhe é presa e que ele vai para o FUNDO. ⚠️ O
            // `insert_path(0, …)` NÃO é o fundo: quem manda no desenho é o `RootOrder` da entidade,
            // e o `sync` dá a toda entidade nova **o maior**.
            if !self.vec.bucket_new.is_empty() {
                crate::vec_bucket::arm_new_fills(sim, &self.vec.entities, &mut self.vec.bucket_new);
            }
            // ADR-0114: idem para os objetos Flip (objeto novo ⇒ entidade; entidade
            // apagada ⇒ objeto). No W0 é no-op (nenhuma tool cria objetos ainda); a
            // tool do W2 passa a populá-lo.
            ph2d_flip_entities::entities::sync(sim, flip, &mut self.flip_state.entities);
            // Live Shapes: mantém o `VecShape::Text` na entidade do texto ativo (a
            // entidade já existe pós-sync) para o objeto lembrar que é texto — re-cook,
            // painel, Convert e save/undo. Idempotente; só com sessão viva.
            if let Some(edit) = self.vec.text_edit.as_ref() {
                crate::vec_text::upsert_text_shape(sim, &self.vec.entities, edit);
            }
            // Live Shapes: a forma recém-desenhada NASCE VIVA — geometria re-cozida
            // centrada (pivô no centro), pose no `Transform`, `VecShape` na entidade.
            // Antes do `settle` (que pula formas vivas). Idempotente.
            crate::vec_shape_live::make_committed_shape_live(
                sim,
                vec_scene,
                &self.vec.entities,
                &mut self.vec.shape,
                // O gesto foi o da ferramenta MOLDURA? A forma nasce igual e ganha o `VecFrame`.
                vec_cfg.mode == ph2d_tool_vector::DrawMode::Frame,
            );
            // "Convert to Curves": assa a(s) forma(s) viva(s) selecionada(s) em paths
            // crus — o TEXTO explode num grupo por-letra; as PARAMÉTRICAS descartam o
            // `VecShape` (a geometria já é a forma); e a pilha de EFEITOS é assada no cozido
            // (ADR-0132). A porta única (`vec_convert::to_curves`) usa o MESMO bake do botão
            // "Apply" da seção Effects. Re-seleciona o resultado.
            if pending_vec_convert {
                let sel: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
                let xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
                let new_sel = crate::vec_convert::to_curves(
                    sim,
                    vec_scene,
                    &mut self.vec.entities,
                    &mut self.vec.pen,
                    &xf,
                    &sel,
                );
                self.vec.pen.select_many(&new_sel);
            }
            // Habilita "Convert to Curves" pela porta ÚNICA (`vec_convert::is_convertible`) — a
            // MESMA que o conversor honra. Enumerar as fontes aqui foi o que apodreceu duas
            // vezes: o botão ficava desligado num caminho só-efeitos e depois num só-quinas,
            // sempre sem erro nenhum. [[feedback_a_condition_that_enumerates_its_readers_rots]]
            #[cfg(feature = "panel-vector")]
            {
                let convertible = self.vec.pen.selected_paths().iter().any(|id| {
                    crate::vec_convert::is_convertible(sim, &self.vec.entities, vec_scene, *id)
                });
                ph2d_panel_vector::set_current_convertible(convertible);
                // ADR-0129: Expand/Release só são OFERECIDOS quando a seleção é de fato um
                // envelope. A pergunta é a MESMA porta que decide a seleção (selecionar-só-o-
                // container) e executa o dissolve — três consumidores, uma resposta.
                let sel_bits: Vec<u64> = self
                    .vec
                    .pen
                    .selected_paths()
                    .iter()
                    .filter_map(|id| self.vec.entities.get(id).copied())
                    .collect();
                let env_container = crate::envelope_live::sole_container(sim, &sel_bits);
                ph2d_panel_vector::set_current_has_envelope(env_container.is_some());
                // ⭐⭐⭐ **O ESQUELETO** (estudo 42 item 5): as duas perguntas que só a shell
                // responde — *"a selecção tem forma PRESA?"* (decide se as saídas são oferecidas) e
                // *"o que está aceso é um osso, e com que números?"* (decide os dois campos).
                //
                // ⚠️ A 2ª passa pela MESMA porta que o gesto e o overlay usam
                // (`bone_gesture::selected_bone`): QUATRO consumidores, uma resposta — o dedo,
                // o dreno dos verbos, este painel e o desenho do overlay (o quarto entrou na wave
                // do gizmo de limite, e esta conta ficou em três até 2026-09-08).
                let presa = self.vec.pen.selected_paths().iter().any(|id| {
                    self.vec.entities.get(id).is_some_and(|&b| {
                        sim.world()
                            .get::<ph2d_skeleton_ecs::SkinBind>(ph2d_ecs::Entity::from_bits(b))
                            .is_some()
                    })
                });
                ph2d_panel_skeleton::set_current_skinned(presa);
                // ⭐ E a pergunta da fileira *Deform*, que é OUTRA: ela é sobre a CENA, porque a
                // escolha é global. ⛔ Varrer `selected_paths` aqui não a responderia — uma imagem
                // presa é uma sprite, e nunca aparece naquela lista.
                ph2d_panel_skeleton::set_current_skinned_image(sim.world().iter_entities().any(
                    |er| crate::render_loop::sim_extract::skinned_image(sim.world(), er.id()),
                ));
                // E se a CENA tem esqueleto — é isso que faz a seção aparecer (ou não) fora do modo
                // Osso. ⛔ Sem esta metade ela seria um cabeçalho permanente num app que nunca viu
                // um osso, que é exactamente o report que a tabela de escopo curou em 31/08.
                // ⭐⭐⭐ **QUEM ABRE O PAINEL DE BONES** (ordem do dono, 2026-09-09).
                //
                // ⛔⛔ **A visibilidade deixou de ser DERIVADA da cena.** Enquanto ela era
                // `tem_esqueleto || ferramenta_osso`, escrita em TODO quadro, o menu *Window →
                // Bones* seria um interruptor morto — o quadro seguinte repunha a decisão da shell
                // por cima da do artista. *Duas fontes de verdade para o mesmo bool, e a que o
                // artista toca é a que perde.*
                //
                // ⇒ ficam **duas portas, as duas de ARESTA**: a linha do menu (o `skeleton_toggle`)
                // e a selecção de um osso (mais abaixo). Nenhuma das duas escreve em todo quadro.
                let ferramenta_osso = self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Bone;
                // ⭐ E o VERBO do arrasto, como ÍNDICE — é o que mantém aquele painel sem depender
                // da crate da ferramenta de vector.
                ph2d_panel_skeleton::set_current_bone_tool(ferramenta_osso.then(|| {
                    usize::from(
                        self.vec.draw_config.bone_action == ph2d_tool_vector::BoneAction::Transform,
                    )
                }));
                // ⭐ E COMO a pele é desenhada (report das arestas retas, 2026-09-10) — também como
                // ÍNDICE, e ⛔ sem `Option`: esta pergunta tem sempre resposta.
                ph2d_panel_skeleton::set_current_skin_deform(usize::from(
                    self.vec.draw_config.skin_deform == ph2d_tool_vector::SkinDeform::Smooth,
                ));
                // ⭐⭐⭐ **O PICK DO ALVO RESOLVE-SE AQUI**, antes de se perguntar qual osso está em
                // foco — e a ordem é o desenho: quem resolve é *«a selecção passou a ser outra
                // coisa»*, e o clique que a mudou pode ter vindo do CANVAS **ou** da HIERARQUIA. As
                // duas escrevem a mesma selecção, então as duas superfícies saem de graça; um
                // segundo caminho de acerto seria a segunda resposta à mesma pergunta.
                //
                // ⚠️ **E a selecção VOLTA ao osso**, ao contrário dos irmãos `PathPick`: aqui o
                // artista escolheu um objecto *para o osso*, e o contexto dele é o painel do osso.
                // Sem isto a secção Skeleton desaparecia debaixo dele no instante do acerto.
                //
                // ⛔ Clique no vazio **não** desarma (a lista de objectos anima-se por engano com
                // facilidade); quem desiste é o `Escape`.
                // ⛔⛔ **UM PICK NÃO SOBREVIVE AO SUJEITO DELE** (auditoria de 2026-09-08). Ele
                // consome o `Down` primário em **toda** ferramenta, então um pick esquecido é o
                // canvas morto ao botão esquerdo, **sem nada na tela que o diga** — o botão
                // *Picking…* deixa de ser pintado no instante em que o osso deixa de estar em foco.
                //
                // ⚠️ **Pergunta-se o FACTO, não os eventos:** abrir outro projecto, o `Ctrl+Z` e
                // apagar o osso deixam todos os mesmos bits mortos, e uma lista de sítios a limpar
                // esqueceria o quarto. O irmão `vec_path_pick` tem cinco limpezas escritas à mão, e
                // o comentário de uma delas já escrevia a lei: *«não faz sentido: limpa, para não
                // ficar armado e invisível»*.
                if let Some(bits) = self.skeleton.smart_pick
                    && ph2d_ecs::Entity::try_from_bits(bits).is_none_or(|e| {
                        sim.world().get::<ph2d_skeleton_ecs::SmartBone>(e).is_none()
                    })
                {
                    self.skeleton.smart_pick = None;
                }
                // ⚠️ **EXACTAMENTE UM seleccionado**, e não *«o primeiro que não é o osso»*: o
                // estado normal do *Bind* é **forma + osso** escolhidos (é a razão de existir do
                // `bone_gesture::selected_bone`), e ali a leitura antiga resolvia o pick **no mesmo
                // quadro em que ele era armado**, sem o artista clicar em nada.
                let alvo_do_pick = self.skeleton.smart_pick.and_then(|bits_osso| {
                    let sel: Vec<u64> = hero.gizmo.iter_selected().collect();
                    (sel.len() == 1 && sel[0] != bits_osso).then(|| sel[0])
                });
                if let Some(bits_osso) = self.skeleton.smart_pick
                    && let Some(alvo) = alvo_do_pick
                    && crate::skeleton_smart::set_target(
                        sim,
                        ph2d_ecs::Entity::from_bits(bits_osso),
                        ph2d_ecs::Entity::from_bits(alvo),
                    )
                {
                    self.skeleton.smart_pick = None;
                    hero.gizmo.replace_selection(Some(bits_osso));
                }
                let osso_em_foco =
                    crate::bone_gesture::selected_bone(sim, hero.gizmo.iter_selected());
                // ⭐⭐⭐ **UM OSSO NOVO EM FOCO REVELA A SECÇÃO** (report do dono, 2026-09-08:
                // *«selecionar o bone nem sempre abre a secção de skeleton no painel»*).
                //
                // ⚠️ **A ARESTA é o que se publica, nunca o estado:** com um osso escolhido o
                // painel rolaria a cada quadro e o artista não conseguiria ler mais nada. É a mesma
                // lei que a timeline já segue — *«seleccionar um objecto NOVO leva a timeline à aba
                // Keys»* (Enio, 2026-07-22).
                //
                // ⚠️ **Quem decide se ROLA é o painel**, que é o único sítio onde a faixa visível e
                // o `y` do cabeçalho existem: daqui sai o *pedido*, e um cabeçalho já à vista fica
                // onde está.
                // ⭐⭐⭐ **UM OSSO NOVO TRAZ A ABA DO PAINEL PARA A FRENTE.**
                //
                // ⛔⛔ **É o sucessor do «revelar-ao-focar»** (report do dono, 2026-09-08), e o
                // painel próprio mudou-lhe o EFEITO sem mudar a lei: a rolagem existia porque a
                // secção caía `1394 px` abaixo de 785 px de outro assunto; aqui o cabeçalho é a
                // primeira linha e não há dobra onde se esconder. O que sobra é o encaixe
                // partilhado — se o Inspector estiver por cima, revelar é **trazer a aba**.
                //
                // ⚠️ A ARESTA continua a ser a lei (`skeleton_reveal::on_focus`): pedi-lo em todo
                // quadro prenderia a aba e o artista não conseguiria olhar para outra.
                if crate::skeleton_reveal::on_focus(&mut self.skeleton.osso_revelado, osso_em_foco)
                {
                    // ⭐⭐⭐ **ORDEM DO DONO (2026-09-09):** *«se já existe um osso no mundo, ao
                    // seleccionar o osso o painel de Bones é aberto e o botão Transform é
                    // seleccionado»*. As três metades saem da MESMA aresta, e é isso que as mantém
                    // de acordo: abrir sem armar deixaria a fileira apagada sobre um osso escolhido.
                    <_ as ph2d_editor_core::panel::PanelHostInternal>::set_panel_visible(
                        hero,
                        <ph2d_panel_skeleton::SkeletonPanel as ph2d_editor_core::panel::Panel>::ID,
                        true,
                    );
                    hero.store
                        .bump_panel_z(ph2d_editor_core::ids::SKELETON_PANEL);
                    // ⚠️ **A ferramenta arma-se no QUADRO SEGUINTE** (`bone_arm_pending`): aqui o
                    // `gfx` já está emprestado a `sim`/`hero`, e um segundo empréstimo dele não
                    // compila. O espelho da shell escreve-se **já**, para este quadro rotear certo
                    // e a fileira acender no mesmo instante em que o osso é escolhido.
                    self.skeleton.bone_arm_pending = Some(ph2d_tool_vector::BoneAction::Transform);
                    self.vec.draw_config.mode = ph2d_tool_vector::DrawMode::Bone;
                    self.vec.draw_config.bone_action = ph2d_tool_vector::BoneAction::Transform;
                }
                // ⭐ **PORQUE a secção não tem sujeito** (report do dono, 2026-09-08: *«seleccionar o
                // bone nem sempre abre a secção de skeleton»*). ⚠️ A pergunta tem três respostas que
                // se leem iguais na tela — *nada seleccionado* · *seleccionado e não é osso* · *é
                // osso e a secção está fechada/fora da dobra* —, e esta linha separa-as: ela diz o
                // que ESTÁ seleccionado e o que cada um É.
                if std::env::var_os("PH2D_BONE_LOG").is_some() && osso_em_foco.is_none() {
                    let quem: Vec<String> = hero
                        .gizmo
                        .iter_selected()
                        .map(|b| {
                            let e = ph2d_ecs::Entity::from_bits(b);
                            let nome = sim.world().get::<ph2d_ecs::Name>(e).map_or_else(
                                || "<sem nome>".to_string(),
                                |n| n.as_str().to_string(),
                            );
                            let osso = sim.world().get::<ph2d_skeleton_ecs::Bone>(e).is_some();
                            format!("{nome}(osso={osso})")
                        })
                        .collect();
                    if !quem.is_empty() {
                        eprintln!(
                            "[bone] a seccao SKELETON esta' sem sujeito, e ha' {} seleccionado(s): \
                             {quem:?} -- nenhum deles e' um osso",
                            quem.len()
                        );
                    }
                }
                ph2d_panel_skeleton::set_current_bone(osso_em_foco.and_then(|b| {
                    sim.world()
                        .get::<ph2d_skeleton_ecs::Bone>(ph2d_ecs::Entity::from_bits(b))
                        .map(|v| (v.length, v.strength))
                }));
                // ⭐⭐⭐ **A ÂNCORA do osso em foco** — é isto que decide entre *Add IK* e *Remove IK*
                // no painel, e se os três números dela têm sujeito. ⚠️ Pela MESMA porta que publica
                // os números do osso (`selected_bone`): duas perguntas *"qual osso está aceso?"*
                // divergiriam no primeiro clique.
                // ⭐ O limite da junta em foco, em GRAUS — a mesma porta e o mesmo guarda de foco.
                ph2d_panel_skeleton::set_current_bone_limit(osso_em_foco.and_then(|b| {
                    sim.world()
                        .get::<ph2d_skeleton_ecs::BoneLimit>(ph2d_ecs::Entity::from_bits(b))
                        .map(|l| (l.min.to_degrees(), l.max.to_degrees()))
                }));
                // ⭐ O osso inteligente em foco — a faixa (em GRAUS), a acção e o alvo, pela MESMA
                // porta: publicá-los por portas separadas deixaria um quadro em que a faixa é de um
                // osso e o nome é do anterior.
                let smart = osso_em_foco.and_then(|b| {
                    sim.world()
                        .get::<ph2d_skeleton_ecs::SmartBone>(ph2d_ecs::Entity::from_bits(b))
                        .cloned()
                });
                ph2d_panel_skeleton::set_current_bone_smart(smart.as_ref().map(|s| {
                    ph2d_panel_skeleton::SmartBoneView {
                        from: s.from.to_degrees(),
                        to: s.to.to_degrees(),
                        clip: s.clip.clone(),
                        target: s.target.clone(),
                        picking: self.skeleton.smart_pick == osso_em_foco,
                    }
                }));
                // ⭐⭐⭐ **A lista de ACÇÕES, filtrada pelo ALVO** — é ela que responde *«qual
                // animação?»* na tela. ⚠️ Publicada **só** quando há um osso inteligente em foco:
                // sem sujeito ela seria um selector sem nada para escolher.
                ph2d_panel_skeleton::set_current_bone_actions(
                    smart.as_ref().map_or_else(Vec::new, |s| {
                        crate::skeleton_smart::actions_for(sim.world(), &self.timeline.doc, s)
                    }),
                );
                ph2d_panel_skeleton::set_current_bone_ik(osso_em_foco.and_then(|b| {
                    sim.world()
                        .get::<ph2d_skeleton_ecs::IkGoal>(ph2d_ecs::Entity::from_bits(b))
                        .map(|g| (g.mix, g.softness, f64::from(g.chain), g.bend))
                }));
                // Text on Path (plano 22): as duas perguntas que só a shell sabe responder —
                // *"esta seleção permite prender?"* (um texto + um caminho) e *"o texto em foco
                // já cavalga alguma coisa, e com que valores?"*. A primeira usa a MESMA porta
                // que o clique honra (`link_candidate`), senão o botão apareceria e recusaria.
                let sel = self.vec.pen.selected_paths().to_vec();
                ph2d_panel_vector::set_current_textpath_can_link(
                    crate::vec_text_ride::link_candidate(sim, &self.vec.entities, &sel).is_some(),
                );
                let ride = crate::vec_text_ride::current(sim, &self.vec.entities, &sel);
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
                let pat = crate::pattern_live::current(sim, &self.vec.entities, &sel);
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
                        &self.vec.entities,
                        &sel,
                    )),
                );
                // Contour (pesquisa `20_*` #9): as MESMAS duas perguntas do Pattern — *"esta
                // seleção permite criar?"* e *"o que está armado, com que valores?"*. `can_add`
                // exige forma selecionada e nenhum contour nela: a seção mostra o botão OU os
                // controles, nunca os dois, e é isso que impede a swatch de existir sem alvo.
                let cont = crate::contour_live::current(sim, &self.vec.entities, &sel);
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
                    let xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
                    crate::vec_expand::offset_scale(vec_scene, &self.vec.pen, &xf)
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
                if cont_mirror != self.vec.contour_mirrored {
                    self.vec.contour_mirrored = cont_mirror;
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
                                ph2d_editor_core::ids::VECTOR_CONTOUR_STEPS,
                                ph2d_editor_core::ids::VECTOR_CONTOUR_STEPS_NUM,
                                ph2d_panel_vector::contour_steps_to_track(steps),
                                steps,
                            ),
                            (
                                ph2d_editor_core::ids::VECTOR_CONTOUR_OFFSET,
                                ph2d_editor_core::ids::VECTOR_CONTOUR_OFFSET_NUM,
                                ph2d_panel_vector::contour_d_to_track(frac),
                                frac * 100.0, // LITERAL-PX-OK: fração -> percentual do readout
                            ),
                            (
                                ph2d_editor_core::ids::VECTOR_CONTOUR_ACCEL,
                                ph2d_editor_core::ids::VECTOR_CONTOUR_ACCEL_NUM,
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
                    .find_map(|id| crate::fx_live::spec_of(sim, &self.vec.entities, *id));
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
                                // o dispositivo de fato soma.
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
                let fx_target = crate::fx_bridge::sole_path(self.vec.pen.selected_paths());
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
                let sel: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
                // ⭐⭐ **O LATCH de «armado para desenhar».** A tool publica o clique no catálogo;
                // a selecção que MUDA o apaga — e desenhar selecciona a forma nova, então o ciclo
                // Live Shape volta sozinho no gesto seguinte. Sem isto, *"armei o Polígono"* e
                // *"acabei de desenhar uma estrela"* leem-se iguais (os dois são `DrawMode::Shape`
                // com uma forma viva na selecção) e um dos dois fica errado, seja qual for a regra.
                // ⚠️ **O alvo CRU** (`panel_shape_target`, e não a porta): o latch tem de ver a
                // selecção real para saber quando se apagar. Ler a porta aqui seria um laço — ela
                // devolve `None` justamente porque o latch está aceso, e ele nunca mais cairia.
                let alvo_vivo =
                    crate::vec_shape_params::panel_shape_target(sim, &self.vec.entities, &sel)
                        .map(|(id, ..)| id);
                // ⚠️ **O DESARME primeiro, o ARME depois.** Um clique é um EVENTO drenado; a
                // mudança de alvo é um NÍVEL comparado com o frame anterior. Se algum dia os dois
                // caírem no mesmo frame, quem tem de ganhar é o gesto que se sabe ter acontecido.
                if alvo_vivo != self.vec.shape_armed_target {
                    self.vec.shape_armed_target = alvo_vivo;
                    self.vec.shape_armed = false;
                }
                if vector_bridge::take_shape_armed(tools) {
                    self.vec.shape_armed = true;
                }
                let target = crate::vec_shape_params::shape_field_target(
                    sim,
                    &self.vec.entities,
                    &sel,
                    self.vec.draw_config.mode,
                    self.vec.shape_armed,
                );
                ph2d_panel_vector::set_current_shape_focus(target.as_ref().map(|(_, _, k, _)| *k));
                // Semente ONE-SHOT: só quando o alvo MUDA (senão brigaria com o arrasto).
                // Além dos campos, a TOOL adota os params — assim painel, tool e objeto
                // concordam, e a próxima forma desenhada herda (modelo Figma).
                // ⚠️ O gatilho é o PAR `(alvo, tipo)`. Só o alvo deixava *"nada selecionado,
                // catálogo em Star"* e *"…em Polygon"* comparando iguais (`None == None`), e
                // como os slots do store são por ÍNDICE — compartilhados por TODAS as formas —
                // os campos ficavam com os números da forma anterior.
                let catalog = vector_bridge::shape_catalog(tools);
                let focus = crate::vec_shape_params::shape_seed_focus(
                    target.as_ref().map(|(id, _, k, _)| (*id, *k)),
                    catalog.map(|(k, _)| k).unwrap_or_default(),
                );
                if Some(focus) != self.vec.shape_last_focus {
                    self.vec.shape_last_focus = Some(focus);
                    // A conversão para UI é UMA, aqui: os dois consumidores (o store que o
                    // painel pinta e a tool que adota) leem o MESMO array. Fazê-la dentro do
                    // `seed_shape_fields` a duplicaria.
                    let ui = match target.as_ref() {
                        // Alvo vivo: os parâmetros DELE, que estão em mundo.
                        Some((_, _, kind, world)) => {
                            crate::vec_shape_params::ui_values_of(*kind, world, vec_px_to_world)
                        }
                        // Sem alvo: o que a tool guarda para aquele tipo — já em UI, e é o
                        // default do próximo desenho.
                        None => catalog.map(|(_, v)| v).unwrap_or_default(),
                    };
                    crate::vec_shape_params::seed_shape_fields(&mut hero.store, focus.1, &ui);
                    vector_bridge::adopt_shape_values(tools, focus.1, ui);
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
                &self.vec.entities,
                self.vec.connect.as_ref().map(|d| (d.path, &d.conn)),
                &mut self.vec.connect_pending,
            );
            // **Blend Objects, 1ª metade:** pendura o `VecBlend` na entidade (nascida no `sync`)
            // do blend recém-criado. ANTES do `settle`, pela mesma razão do conector: o `settle`
            // pula o blend, mas só o que ENXERGA — sem o componente já pendurado, o spine
            // recém-empurrado seria assentado como um path comum e o recook do frame seguinte
            // sairia deslocado (ADR-0128).
            crate::blend_live::upkeep(
                sim,
                vec_scene,
                &self.vec.entities,
                &mut self.vec.blend_pending,
            );
            // **A LINHA DE CORTE, 1ª metade:** pendura o `VecCutPath` na entidade (nascida no
            // `sync`) da lâmina recém-desenhada. Mesma posição e mesma razão dos dois de cima.
            crate::vec_cut_line::upkeep(
                sim,
                vec_scene,
                &self.vec.entities,
                &mut self.vec.cut_pending,
            );
            // **Morph Objects, 1ª metade:** idem, e pela MESMA razão — sem o componente pendurado
            // antes do `settle`, o path recém-empurrado seria assentado como um path comum e o
            // recook do frame seguinte sairia deslocado.
            crate::morph_live::upkeep(
                sim,
                vec_scene,
                &self.vec.entities,
                &mut self.vec.morph_pending,
            );
            // ⭐⭐ **O CONJUNTO de estados** (plano 32 W8) — mesma posição e mesma razão do irmão
            // acima, e mais uma: é aqui que os membros são reparentados e escondidos, e as quatro
            // escritas têm de cair no MESMO quadro para o Ctrl+Z desfazer o conjunto inteiro.
            ph2d_vec_entities::morph_set::upkeep(
                sim,
                vec_scene,
                &self.vec.entities,
                &mut self.vec.morph_set_pending,
            );
            // **Envelope Objects (ADR-0129 Fatia 3):** SEM `upkeep` — o envelope não cria path
            // nenhum (o container não tem path), então não há entidade nova esperando o `sync`. Tudo
            // (assar + reparentar + pendurar) já aconteceu síncrono no `create`. O `settle` pula os
            // filhos (têm `ChildOf`) e o container (sem path, fora do mapa).
            // ADR-0112: a origem (o pivô) de um path nasce no centro do MUNDO. Assim
            // que a forma pára de crescer, ela vai para o centro dela. Quem está EM GESTO é
            // pulado, e a lista sai de UMA porta (`vec_gesture_paths`) — o porquê está lá.
            let drawing = ph2d_vec_entities::transform::gesture_paths(
                &self.vec.pen,
                &self.vec.shape,
                &self.vec.pencil,
            );
            ph2d_vec_entities::transform::settle_origins(
                sim,
                vec_scene,
                &self.vec.entities,
                &drawing,
            );
            // ADR-0114/ADR-0111: idem para os objetos Flip — o pivô nasce no centro do
            // MUNDO; assim que a arte pára de crescer, ele vai para o centro dela (e a
            // geometria vira LOCAL). O objeto EM GESTO (desenho/borracha ativos) NÃO é
            // assentado — a mão escreve MUNDO a cada frame e somar geometria+Transform
            // deslocaria a arte do cursor.
            let flip_gesturing = (self.flip_state.draw.is_active() || self.flip_state.erasing)
                .then(|| flip.objects().first().map(|o| o.id))
                .flatten();
            ph2d_flip_entities::transform::settle_origins(
                sim,
                flip,
                &self.flip_state.entities,
                flip_gesturing,
            );
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
            // **As duas gémeas do `RootOrder`** (ADR-0164 F1), aqui pela MESMA razão e no
            // MESMO sítio: elas têm de correr depois do `sync` (as entidades novas do quadro
            // já existem) e **antes da captura do fim do quadro**, senão o objeto criado neste
            // quadro entra no snapshot sem identidade e sem ordem — e o primeiro Ctrl+Z não
            // teria o que repor.
            //
            // ⚠️ `StableId` é a identidade DURÁVEL: ela sobrevive ao respawn do undo, que é a
            // propriedade inteira pela qual a wave existe. `SiblingOrder` faz a ordem entre
            // irmãos ser DADO — antes dela, reordenar não era desfazível nem sobrevivia a um
            // restore (classe BUGS #15), porque a ordem vivia na lista `Children` do bevy, que
            // é memória de runtime.
            //
            // ⚠️ **As duas são idempotentes**, e isso não é higiene: se reescrevessem por
            // quadro, o diff do undo veria o arquétipo de toda entidade mudar e **cada quadro
            // com input viraria um passo espúrio** — a doença que o `RootOrder` curou.
            ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
            ph2d_ecs::assign_missing_sibling_order(sim.world_mut());
            // O Blend pediu uma sequência de z; agora as entidades existem (o `sync` rodou) e ela
            // pode ser escrita na ÁRVORE — que é quem manda no z (ADR-0110). Escrever na ordem do
            // vetor da cena seria a porta errada: a projeção abaixo a reescreve todo frame.
            for order in std::mem::take(&mut self.vec.restack) {
                ph2d_vec_entities::entities::restack(sim, &self.vec.entities, &order);
            }
            // ⭐⭐⭐ **O ARRASTO DA HIERARQUIA ESCREVE A ÁRVORE, LOGO ELE MORA AQUI** — ao lado do
            // `restack` e dos três `assign_missing_*`, e **antes** de a árvore ser lida.
            //
            // ⛔⛔ **Report do Enio, 2026-09-07: *«reordenei objectos na hierarquia e não funcionou
            // o undo»*.** Ele estava certo, e o defeito NÃO era o undo: era este dreno correr
            // ~2 340 linhas **depois** da projecção, dentro do `hierarchy::dispatch`. A sequência
            // medida com `PH2D_UNDO_LOG=1`:
            //
            //   * quadro N — a árvore muda (`RootOrder` de `[(1,0),(2,1),(3,2)]` para
            //     `[(1,0),(2,2),(3,1)]`), mas a projecção já correu sobre a árvore VELHA ⇒ a
            //     captura do fim do quadro guarda `world` novo com `vec` **velho**;
            //   * quadro N+1 — a projecção lê a árvore nova e reescreve a pilha; sem entrada, o
            //     passo é SUPRIMIDO e funde-se no seguinte;
            //   * mais tarde nasce um passo cujo conteúdo inteiro é `partes: ["vec"]`
            //     (`base=[0,1,2] atual=[0,2,1] · só a ORDEM=true`) — um **fantasma**;
            //   * `Ctrl+Z` repõe a pilha e **não** a árvore, a projecção do quadro seguinte
            //     re-deriva a pilha da árvore que ninguém desfez, e o fantasma **renasce**. O log
            //     do dono mostra o ciclo a repetir-se com a fila parada em `5`: cada `Ctrl+Z` gasta
            //     um passo que o próprio quadro volta a criar, e o passo REAL (o `["world"]`)
            //     nunca chega a ser alcançado.
            //
            // ⚠️ **É a doença que o [`ph2d_vec_entities::entities::z_order`] já documenta** — *«a captura
            // deixava de ser ponto fixo dos sistemas»* — a voltar por outra porta: ali era a forma
            // recém-nascida contra a lista do painel, aqui é a árvore reordenada contra a projecção
            // do mesmo quadro. ⇒ a lei não é sobre QUEM escreve, é sobre QUANDO: **todo escritor da
            // árvore corre antes de ela ser lida, e a leitura antes da captura.**
            //
            // ⚠️ **O `take` é load-bearing:** o `hierarchy::dispatch` lá em baixo continua a receber
            // o parâmetro (a assinatura é dele), e vê `None` — aplicar duas vezes reordenaria duas.
            if let Some(intent) = reparent_intent.take()
                && let Some(live) = hero_live.as_ref()
            {
                hero_intents::drain_reparent(intent, live, sim, toasts);
                self.title_dirty = true;
            }
            if let Some(live) = hero_live.as_mut() {
                crate::build_hierarchy_snapshot(
                    sim.world(),
                    &mut live.z_walk_state,
                    &mut live.z_walk_scratch,
                    &mut live.z_snapshot,
                );
                let order = ph2d_vec_entities::entities::z_order(sim.world(), &live.z_snapshot);
                vec_scene.reorder_to(&order);
            }
            let mut vec_view = ph2d_vec_entities::entities::view_state(sim, &self.vec.entities);
            // **As MOLDURAS** (plano UI/UX W0): que intervalo da pilha cada uma recorta. Sai do
            // MESMO snapshot que acabou de ditar a pilha de z — derivá-lo de outra fonte seria uma
            // segunda resposta a *"em que ordem estas formas estão?"* — e da pilha FINAL, porque o
            // Z global pode ter tirado um descendente de dentro do intervalo.
            if let Some(live) = hero_live.as_ref() {
                let order: Vec<ph2d_vec_scene::VecPathId> =
                    vec_scene.paths().iter().map(|p| p.id).collect();
                vec_view.clips = crate::vec_frame_spans::clip_spans(sim, &live.z_snapshot, &order);
            }
            // **OS TOKENS** (plano UI/UX W4): a tinta que cada binding produz no modo VIGENTE.
            // Resolvido aqui, no passe de DESENHO, e não dentro do `view_state` — aquela porta é
            // chamada por todo hit-test e gesto, e nenhum deles pergunta de que cor a forma é.
            vec_view.bound = crate::vec_bindings::resolve(
                sim,
                &self.vec.entities,
                crate::vec_bindings::TokenCtx {
                    theme: hero.theme,
                    pixels_per_meter: hero.project.pixels_per_meter,
                },
            );
            // ⭐⭐⭐ **A APARÊNCIA QUE UM MOTOR CONDUZ** — hoje a opacidade que a linha do tempo
            // escreve num caminho vetorial (`ph2d_ecs::VecDrivenStyle`). Depois dos tokens (ela
            // desvanece a tinta que eles resolveram) e **antes** das rows autoradas: se as duas
            // falarem da mesma forma, quem manda é o controlo que o artista está a segurar, e o
            // motor é o estado de fundo — o precedente é o passe de estados de UI, mais abaixo.
            let driven = crate::vec_driven_style::resolve(sim, &self.vec.entities);
            crate::vec_driven_style::apply(&driven, &mut vec_view);
            // ⭐ E o componente volta ao AUTORADO — depois de ser lido, nunca antes. Sem isto,
            // apagar uma track de opacidade deixava a forma congelada no último valor da curva
            // para sempre (ver o doc da função).
            crate::vec_driven_style::settle_to_authored(sim, &self.vec.entities, vec_scene);
            // **AS ROWS AUTORADAS** (plano UI/UX W8b.3): o valor VIVO de cada controle que dirige
            // uma forma. Depois dos tokens, porque a opacidade desvanece o que de fato vai ser
            // desenhado; e aqui, no passe de desenho, pela MESMA razão que os tokens — nenhum
            // hit-test pergunta em que ponto um slider está.
            let drives = crate::vec_widget_drive::resolve(sim, &self.vec.entities, &hero.store);
            crate::vec_widget_drive::apply(&drives, &mut vec_view);
            // ADR-0111 — cada path tem `Transform`. A geometria dele é LOCAL; este é
            // o afim que a leva ao mundo (a cadeia de pais inclusa).
            let mut vec_xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
            // **Conectores, 2ª metade:** a geometria é uma função pura da RELAÇÃO — re-cozida
            // aqui, todo frame, sobre os afins DESTE frame. É o que faz a linha SEGUIR a
            // forma que o gizmo acabou de mover.
            crate::connector_live::recook(
                sim,
                vec_scene,
                &self.vec.entities,
                &vec_xf,
                &mut self.vec.connect_sides,
            );
            // **Morph Objects, 2ª metade:** a forma é função pura das duas fontes e do `t` —
            // re-cozida aqui, todo frame, sobre os afins DESTE frame. É o que a faz SEGUIR a
            // forma que o gizmo acabou de mover, e o que faz o `t` da timeline virar movimento.
            // (O `t` já foi escrito: o apply da timeline roda antes desta metade do frame.)
            // ⭐ **A MÁQUINA DE MORPH, um quadro** (plano 32 W5) — ela escreve o PAR e o `t`, e o
            // `recook` logo abaixo transforma-os em forma. ⚠️ **Antes do recook, de propósito**: é
            // a mesma ordem pela qual o `t` da timeline vira movimento.
            //
            // ⚠️ **Só no MODO DE PRÉ-VISUALIZAÇÃO** (plano 32 W9). A condição de uma seta é uma
            // tecla; a escutar durante a edição, carregar em `Z` morfa a forma **e** faz o que o
            // `Z` faz no editor — os dois, sem nada na tela a explicar.
            //
            // ⛔ **O playhead deixou de ser a porta**, e a troca é a cura de um report do Enio
            // (2026-08-25): o Play **não tranca o teclado do editor**, então com ele a andar as
            // setas do teclado morfavam a forma *e* moviam as formas. Este modo tranca.
            crate::morph_machine_drive::tick(
                &mut self.morph_machines,
                sim,
                &self.vec.entities,
                &ph2d_input::Input::new(&hero.input_map, &self.input_actions),
                // ⛔⛔ **O sistema de States tem PRECEDÊNCIA** (W11e, 2.º report do Enio): ordenar
                // os dois motores dentro do quadro não bastava, porque a transição só fala no
                // MEIO — no repouso e na chegada quem escrevia era a máquina de teclas, parada
                // onde o ▶ a deixou. Ver `morph_machine_drive::drives`.
                crate::morph_machine_drive::drives(self.morph_preview, self.ui_state_live),
                self.fixed_step.fixed_dt(),
                &mut self.preview_drive,
            );
            // ⭐⭐⭐ **O conjunto de estados ANIMADO POR UMA TRANSIÇÃO DE UI** (plano 32 W11c) —
            // corre depois do `tick` e ANTES do `recook`, que é a única janela em que faz sentido:
            // ele escreve o par e o `t`, e o `recook` é quem os transforma em geometria.
            //
            // ⚠️ **Depois do `tick` de propósito:** se as duas coisas escrevem o mesmo objecto,
            // quem manda é a transição de UI — ela é o gesto que o artista acabou de fazer, e a
            // máquina de teclas é o estado de fundo.
            crate::morph_machine_drive::apply_ui_steps(
                sim,
                &self.vec.entities,
                &self.ui_cooked.morph_steps,
                &mut self.preview_drive,
            );
            crate::morph_live::recook(
                sim,
                vec_scene,
                &self.vec.entities,
                &vec_xf,
                &mut self.vec.morph_plans,
            );
            // **Envelope Objects (ADR-0129):** a forma de cada filho é a fonte autorada deformada
            // pela gaiola comum — re-cozida aqui, todo frame. Sem xforms nem mapa: a fonte é LOCAL do
            // container e é o `Transform` do container (via `vec_transform::build`) que leva os filhos
            // ao mundo; o recook varre os containers por QUERY (eles não têm path).
            crate::envelope_live::recook(sim, vec_scene);
            // ⭐⭐⭐ **O ESQUELETO** (estudo 42 item 5): a forma presa aos ossos é re-cozida da fonte
            // autorada e da pose de AGORA. Ao lado do envelope de propósito — os dois deformam a
            // partir de uma fonte guardada em bytes — e DEPOIS dele, porque a pele fala de formas
            // que já existem na cena e o envelope pode acabar de reescrever uma.
            //
            // ⭐⭐⭐ **AS ÂNCORAS** — a cinemática INVERSA que persiste. Corre **imediatamente antes**
            // do recook da pele, e a ordem é load-bearing: ela escreve a pose dos ossos, e o recook
            // é quem transforma a pose em geometria. Ao contrário, a pele mostraria a pose do quadro
            // anterior — um atraso de um quadro, invisível parado e visível a arrastar.
            //
            // ⚠️ O `preview_drive` entra na assinatura porque o que este passe escreve é
            // **pré-visualização**: o documento é a pose da ÂNCORA, e a rotação dos ossos governados
            // é derivada dela.
            // ⭐⭐⭐ **OS OSSOS INTELIGENTES correm ANTES da âncora**, e a ordem é load-bearing:
            // um controlo escreve a pose de BASE (ele é a correcção autorada) e a IK é a restrição
            // que persegue um alvo — ela tem de ver a pose já corrigida. ⛔ Ao contrário, a IK
            // resolveria sobre uma pose que a acção ainda vai mudar, e o alvo deixaria de ser
            // alcançado no mesmo quadro.
            crate::skeleton_smart::drive(sim, &self.timeline.doc, &mut self.preview_drive);
            crate::skeleton_goal::solve(sim, &mut self.preview_drive);
            // ⚠️ Sem `xforms`: a pele resolve a pose de cada osso e da forma pela hierarquia (a
            // propagação de `Transform` que a casa já corre), que é a mesma razão de a cinemática
            // directa não precisar de código.
            crate::skeleton_live::recook(sim, vec_scene);
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
            if vector_active && self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Node {
                crate::blend_live::drag_spine_anchors_move_sources(
                    sim,
                    vec_scene,
                    &self.vec.entities,
                    &vec_xf,
                    &mut self.vec.blend_spines,
                );
                vec_xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
            }
            // **Blend Objects, 2ª metade:** os passos são função pura das fontes — re-cozidos
            // aqui, todo frame, sobre os afins DESTE frame. É o que faz a transição SEGUIR a
            // forma que o gizmo acabou de mover (ADR-0128). O buffer é zerado e repopulado.
            crate::blend_live::recook(
                sim,
                vec_scene,
                &self.vec.entities,
                &vec_xf,
                &mut self.vec.blend_spines,
                &mut self.vec.blend_overlay,
            );
            // **Modo Node: o spine sobe para o topo** (ADR-0128) — acima de TODAS as formas e
            // passos, para ser visto e editado. Retira o traço da cena (some do `dispatch`, logo
            // abaixo) e o acrescenta ao fim do overlay do blend (desenhado por último). Em Select
            // o spine fica no seu z (traço sutil), como o Illustrator — o `recook` restaura o
            // traço-base todo frame, então voltar de Node não o deixa invisível.
            if vector_active && self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Node {
                crate::blend_live::elevate_spines(
                    sim,
                    vec_scene,
                    &self.vec.entities,
                    &mut self.vec.blend_overlay,
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
            let text_id = self.vec.text_edit.as_ref().and_then(|e| e.id);
            crate::label_live::upkeep_pending(
                sim,
                &self.vec.entities,
                &mut self.vec.label_pending,
                text_id,
                self.vec.text_edit.is_some(),
            );
            crate::label_live::upkeep(
                sim,
                vec_scene,
                &self.vec.entities,
                &mut vec_xf,
                text_id,
                &mut self.vec.label_poses,
            );
            self.vec.pen.set_view(vec_view.clone());
            self.vec.pen.set_xforms(vec_xf.clone());
            // Seleção casada nos dois sentidos: clique na Hierarquia chega no canvas,
            // clique no canvas acende a linha (e a do grupo, se cheio). A seleção do
            // gizmo é COMPARTILHADA com os sprites — só o subconjunto vetorial é nosso.
            crate::vec_selection::sync_selection(
                &mut hero.gizmo,
                sim,
                vec_scene,
                &self.vec.entities,
                &mut self.vec.pen,
                &mut self.vec.sel,
                vector_active,
            );

            // Motion drift fix (2026-07-25 continuação): sob o split da tool Motion a CENA
            // renderiza num sub-retângulo (present.rs, `CenterSplit::scene_viewport`) e as
            // instâncias/grade projetam com as dims DA CENA (a porta única). As formas VETORIAIS
            // ficaram de fora daquele fix e projetavam a JANELA CHEIA — então um `motion.path`
            // andava numa cópia da curva deslocada+encolhida (o report do Enio: "objetos afastados
            // do path, com drift em relação ao canvas"). Projetá-las com as MESMAS dims casa a
            // curva desenhada com os walkers. Fora do split = janela cheia, byte-idêntico.
            let cam_affine =
                camera.world_to_screen_affine(ph2d_app_motion::field_gizmo::scene_camera_window(
                    hero.view.center_split,
                    window_size,
                ));
            // A SONDA (`PH2D_PAN_DIAG=1`): a cena do Vello é construída AQUI, com o
            // mundo→tela já aplicado na CPU; as sprites recebem a câmera noutro ponto do
            // quadro. Guardar o centro daqui é o que permite comparar os dois instantes.
            ph2d_pan_diag::note_vello_camera(camera.center);
            // A geometria DERIVADA deste frame — hoje, os offsets vivos. Cozida aqui (depois
            // do `sync`, senão uma forma recém-criada ainda não tem entidade e o componente
            // dela não seria encontrado) e desenhada pelo `dispatch` no z de cada forma.
            self.offset_live
                .recook(vec_scene, sim, &self.vec.entities, &vec_xf);
            // O Pattern Along Path vivo (plano 23): as cópias de um motivo ao longo de um guia,
            // cozidas aqui e desenhadas no z do motivo — a fonte nunca é tocada.
            self.pattern_live.recook(vec_scene, sim, &self.vec.entities);
            // ⭐ O offset de CAD de cada camada (v22). ⚠️ Só precisa da CENA: a distância é LOCAL,
            // então a pose não entra na chave — é isso que faz o memo sobreviver ao arrasto.
            self.paint_dilate_live.recook(vec_scene);
            // O Contour vivo (pesquisa 20 #9): os anéis concêntricos + a rampa de cor, cozidos
            // aqui e desenhados no z da fonte — que entra na lista junto com eles.
            self.contour_live
                .recook(vec_scene, sim, &self.vec.entities, &vec_xf);
            // ── A SIMETRIA de DESENHO (plano 25 W6.3) ────────────────────────────
            // *"A linha deve aparecer logo que se aperta o botão e não quando se inicia o desenho.
            // A simetria funciona apenas para formas que serão desenhadas com a tool ligada … com
            // o botão checado pode-se fazer quantos desenhos desejar que a linha permanece no
            // lugar"* (Enio, 2026-08-01).
            //
            // ⚠️ **Aqui e não na malha de ações**, e por duas razões que se somam: o `sync` já
            // correu (uma forma recém-desenhada já tem entidade, senão o componente não teria
            // onde pousar) e o `settle_origins` já assentou o pivô do gesto que acabou — que é o
            // frame em que a captura do eixo sela. É o mesmo sítio, e pela mesma razão, em que o
            // LÁPIS pendura o perfil dele logo abaixo.
            //
            // ⚠️ A adopção olha para `drawing` — quem está EM GESTO —, **nunca** para a seleção.
            // É essa ausência que cumpre *"não deve fazer simetria de formas que já existem
            // previamente"*.
            {
                let style = self.vec.draw_config.symmetry;
                let live = if style.on {
                    // A semeadura acontece UMA vez, na aresta desligado→ligado: *"a tela é a
                    // referência para a posição inicial da linha"*. Re-semear por frame faria a
                    // linha seguir a câmera, e panhar o canvas arrastaria o eixo junto.
                    let origin = *self.vec.symmetry_origin.get_or_insert_with(|| {
                        let (w, h) = (window_size.width as f32, window_size.height as f32);
                        let c = camera.screen_to_world((w * 0.5, h * 0.5), window_size);
                        [f64::from(c[0]), f64::from(c[1])]
                    });
                    self.symmetry_live.adopt(
                        sim,
                        &self.vec.entities,
                        vec_scene,
                        &vec_xf,
                        style,
                        origin,
                        &drawing,
                    )
                } else {
                    // Desligado, o eixo de sessão morre: a próxima ligação re-semeia no centro do
                    // ecrã, que é o que o artista pede ao ligar. Os COMPONENTES ficam — desarmar
                    // esconde as cópias, não as destrói.
                    self.vec.symmetry_origin = None;
                    0
                };
                // O painel só oferece o **Apply** quando há o que consolidar — e "o que se vê" é
                // vazio com o modo desligado, então um Apply ali consolidaria coisa invisível.
                ph2d_panel_vector::state_symmetry::set_symmetry_live_count(live);
            }
            // A SIMETRIA VIVA (plano 25 W6.3): as cópias do modo simétrico, cozidas aqui e
            // desenhadas no z da fonte — que entra na lista junto com elas.
            self.symmetry_live.recook(
                vec_scene,
                sim,
                &self.vec.entities,
                &vec_xf,
                self.vec.draw_config.symmetry.on,
            );
            // **O LÁPIS pendura o perfil que o GESTO pede** (W1d) — ao vivo, a cada frame em que
            // o traço está aberto. É aqui e não no `input_dispatch` porque o armamento precisa do
            // mundo ECS, e porque é o único lugar que corre entre o `sync` (que dá entidade ao
            // path recém-nascido) e o cozimento logo abaixo: o artista vê a espessura enquanto
            // desenha, que é a promessa do lápis desde o W1a (*"o ajuste é AO VIVO"*).
            if let Some(id) = self.vec.pencil.active_path() {
                let stops = self
                    .vec
                    .pencil
                    .width_stops(self.vec.draw_config.pencil_width_source);
                crate::profile_live::arm(sim, &self.vec.entities, &[id], &stops);
            }
            // A largura VIVA (ADR-0148): a fita de largura variável, cozida aqui e desenhada no
            // z da fonte — que continua sendo a curva autorada que o modo Node edita.
            self.profile_live
                .recook(vec_scene, sim, &self.vec.entities, &vec_xf);
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
                self.symmetry_live
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
            // **A BOOLEANA VIVA roda DEPOIS dos cinco e ANTES do alinhamento**, e a ordem é a lei
            // da wave — trocar dois destes termos dá arte diferente sem nenhum gate vermelho:
            //
            // - depois dos cinco, porque ela consome *o que os filhos de fato desenham* (um
            //   operando com offset vivo tem de entrar deslocado);
            // - antes do alinhamento, porque o alinhamento é um campo do `StrokeSpec` do
            //   RESULTADO — alinhar os operandos e só então os combinar responderia outra
            //   pergunta;
            // - e ela TRANSFORMA o mapa (não o estende) pela mesma razão do alinhamento: é um
            //   componente do PAI, então convive com o offset de cada filho.
            self.bool_live.recook(
                vec_scene,
                sim,
                &self.vec.entities,
                &vec_xf,
                &self.ui_cooked.bool_morphs,
                &mut vec_live,
            );
            // **QUEM FOI ABSORVIDO**, publicado no mesmo fôlego em que a absorção acontece. Sem
            // isto o operando consumido — que recebe uma lista VAZIA logo acima — fica
            // indistinguível de uma forma ANIQUILADA por um offset, e a lei *nada desenhado, nada
            // pego* torna-o inalcançável pelo canvas (Enio, 2026-08-22).
            vec_view.absorbed = self.bool_live.absorbed();
            // **O Apply corre AQUI, e não no dreno**, porque ele materializa o `plan` que o
            // `recook` acabou de computar — *o que está na tela*. Chamar o motor de novo lá em
            // cima seria a segunda porta, e ela faria a forma SALTAR no clique.
            //
            // ⚠️ A shell publica também *"há grupo booleano selecionado?"* para o painel decidir
            // se oferece o botão: o painel não alcança o mundo ECS, e uma segunda resposta a essa
            // pergunta seria um Apply pintado sobre uma seleção que não tem o que consolidar.
            {
                let sel: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
                // **A MOLDURA da seleção** (plano UI/UX W0): honra o chip e publica o estado. O
                // clique é honrado ANTES da publicação para o painel mostrar, no mesmo frame, o
                // valor que o artista acabou de escolher — publicar primeiro deixaria o chip a
                // piscar de volta ao valor antigo por um quadro.
                //
                // ⚠️ **O RECORTE deixou de ser a moldura** (2026-08-21): o chip vale para
                // qualquer forma FECHADA, então o sujeito sai do `vec_clip_edit` — que precisa da
                // cena para ler o `closed`, coisa que a pergunta da moldura nunca precisou.
                if let Some(clip) = pending_frame_clip {
                    crate::vec_clip_edit::set_selected_clip(
                        sim,
                        vec_scene,
                        &self.vec.entities,
                        &sel,
                        clip,
                    );
                }
                ph2d_panel_vector::state::set_frame_clip(crate::vec_clip_edit::selected_clip(
                    sim,
                    vec_scene,
                    &self.vec.entities,
                    &sel,
                ));
                // ⚠️ **A outra metade, e ela tem outro sujeito.** A seção Frame (Show as Panel +
                // presets de dispositivo) pergunta pela MOLDURA, que continua a sair do
                // `frame_of_selection` — publicar o recorte para as duas ofereceria os presets de
                // telefone sobre uma elipse que o artista mandou recortar.
                ph2d_panel_vector::state::set_frame_present(
                    crate::vec_frame_edit::frame_of_selection(sim, &self.vec.entities, &sel)
                        .is_some(),
                );
                // ⚠️ O chip *Show as Panel* le' a visibilidade REAL do painel autorado, e nao uma
                // copia: o X do painel escreve o MESMO mapa, entao fechar por la' apaga o chip
                // sozinho. Um bool proprio aqui seria a segunda resposta que fica acesa.
                ph2d_panel_vector::state::set_frame_panel_open(
                    hero.is_panel_visible(ph2d_panel_authored::visibility_key()),
                );
                // **O AUTO LAYOUT** (plano UI/UX W2) — honra o clique ANTES de publicar, pela
                // mesma razao do recorte acima: publicar primeiro deixaria o chip a piscar de
                // volta ao valor antigo por um quadro.
                if let Some(e) = pending_layout_edit {
                    crate::vec_layout_edit::apply_layout_edit(sim, &self.vec.entities, &sel, e);
                }
                if let Some((f, v)) = pending_layout_field {
                    // ⚠️ **A VOLTA da fronteira de display, e ela pergunta ao TIPO.** No mesmo
                    // painel viajam três naturezas de número: o vão / o recuo / os limites são
                    // comprimentos, `Columns` é uma CONTAGEM e `Grow`/`Shrink` são razões do
                    // flexbox. Converter os três dividiria "três colunas" por cem, em silêncio —
                    // todos são `f64`. Quem responde é `LayoutField::is_length`, cujo `match` o
                    // compilador cobra quando uma variante nova entra.
                    let v = if f.is_length() {
                        ph2d_editor_core::LengthDisplay::of(&hero.project).to_world(v)
                    } else {
                        v
                    };
                    crate::vec_layout_edit::apply_layout_field(sim, &self.vec.entities, &sel, f, v);
                }
                // **O Z-INDEX**, honrado ANTES de publicar (a ordem dos vizinhos): publicar
                // primeiro deixaria o campo a mostrar o valor ANTERIOR por um quadro.
                if let Some(v) = pending_vec_z
                    && let Some(id) = sel.first()
                {
                    ph2d_vec_entities::entities::zorder::set_authored_z(
                        sim,
                        &self.vec.entities,
                        *id,
                        // O clamp e' o do COMPONENTE, e mora na porta que escreve — nao no widget.
                        v.round().clamp(f64::from(i32::MIN), f64::from(i32::MAX)) as i32,
                    );
                }
                // AS ÂNCORAS (plano UI/UX W3): aplicar, e só depois publicar — publicar antes
                // deixaria o chip a mostrar a regra ANTERIOR por um frame, e o artista veria a
                // escolha "não pegar" (a mesma ordem que os tokens abaixo).
                if let Some(e) = pending_anchor_edit {
                    crate::vec_anchor_edit::apply_anchor_edit(
                        sim,
                        vec_scene,
                        &self.vec.entities,
                        &sel,
                        e,
                    );
                }
                ph2d_panel_vector::state::set_anchor_state(
                    crate::vec_anchor_edit::selected_anchors(sim, &self.vec.entities, &sel),
                );
                // OS COMPONENTES (plano UI/UX W5): publicar DEPOIS de o produtor ter cozido —
                // é dele que vem a resposta *"esta instância está órfã?"*, e perguntá-la aqui
                // outra vez seria a segunda porta.
                //
                // ⭐⭐⭐ **UM motor, e por isso UMA leitura** (F4.6c fechada, 2026-09-07). Aqui
                // viveu o outro lado do interruptor — sem a env var a secção descrevia o
                // `VecInstance` — e com ele morreram as DUAS listas que só aquele motor enchia:
                // as PEÇAS e os VARIANTS da instância vetorial.
                //
                // ⚠️ **Elas já saíam vazias no caminho de omissão desde 2026-09-06, por
                // DECLARAÇÃO** — o `VecInstance` era componente registado e viajava na cópia
                // profunda do *Make*, então uma leitura ingénua enchia-as e o painel pintava
                // controlos que o dreno geral recusava **em silêncio**. Apagar o produtor apaga a
                // pergunta: *o que não existe não precisa de ser publicado vazio.*
                //
                // ⚠️ **As duas capacidades NÃO se perderam, e a régua é anterior a este corte:**
                // esconder uma peça de UMA cópia e pintá-la são hoje o `Visibility`/`Sprite` da
                // própria peça (`ph2d_app_components::instance_structure::instance_piece_override_tests`, escrito como
                // pré-condição desta fatia), e os variants vivem no cartão do Inspector (F5).
                ph2d_panel_vector::state::set_component_state(
                    crate::vec_component_general::state_of(
                        sim,
                        &self.vec.entities,
                        &sel,
                        // ⭐ **O objecto único na mão** — é o que faz a secção aparecer sobre um
                        // GRUPO, que não é path nenhum. Ver [`vec_component_general::subject_of`].
                        (hero.gizmo.selected_len() == 1)
                            .then_some(hero.gizmo.selection)
                            .flatten(),
                        // ⚠️ O rótulo do botão troca enquanto o gesto de duas mãos está aberto, e
                        // é ele que diz ao artista que o app está à espera do segundo clique.
                        matches!(
                            self.vec.path_pick,
                            Some(crate::vec_pick::PathPick::InstanceMain(_))
                        ),
                    ),
                );
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
                            ph2d_vec_entities::entities::zorder::authored_z(
                                sim,
                                &self.vec.entities,
                                *id,
                            )
                        })
                        .map(|z| z as f32),
                );
                // **Resize Box** (plano UI/UX W3b): honrar e so' depois publicar, a mesma ordem
                // dos irmaos acima — publicar antes deixaria a caixa a mostrar o estado ANTERIOR
                // por um frame, e o artista veria o clique "nao pegar".
                if pending_resize_box {
                    crate::vec_resize_box_edit::toggle_resize_box(sim, &self.vec.entities, &sel);
                }
                ph2d_panel_vector::state::set_resize_box(
                    crate::vec_resize_box_edit::selected_resize_box(sim, &self.vec.entities, &sel),
                );
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
                // **OS TOKENS** (plano UI/UX W4): aplicar a escolha, e depois publicar o que a
                // seleção tem preso. Nesta ordem — publicar antes deixaria o chip a mostrar o
                // token ANTERIOR por um frame, e o artista veria a escolha "não pegar".
                // ⚠️ O detach vem ANTES da escolha do picker: no mesmo frame em que o artista
                // escolhe um TOKEN o `colour_authored` está limpo, então nenhum dos dois pisa no
                // outro — mas na ordem inversa uma cor autorada soltaria o token recém-escolhido.
                crate::vec_bindings::detach_on_authored(sim, &self.vec.entities, &sel);
                if let Some((prop, token)) = pending_token_bind {
                    crate::vec_bindings::set_selected_binding(
                        sim,
                        &self.vec.entities,
                        &sel,
                        prop,
                        token,
                    );
                }
                ph2d_panel_vector::state::set_token_bindings(
                    crate::vec_bindings::selected_bindings(sim, &self.vec.entities, &sel),
                );
                // **As ETIQUETAS das molduras** (Enio 2026-08-01) — publicadas em TODO frame, com
                // qualquer ferramenta em mãos. ⚠️ Aqui não vale a cerca da RÉGUA (que só vive com
                // o Vector porque OCUPA a borda do canvas e comeria o pen-down do Painter): uma
                // etiqueta é desenho puro, sem região de hit, e uma moldura é mobília de cena que
                // se precisa reconhecer mesmo enquanto se pinta dentro dela.
                hero.gizmo.frame_labels = crate::vec_frame_labels::frame_labels(
                    sim,
                    vec_scene,
                    &self.vec.entities,
                    &vec_xf,
                    &sel,
                );
                let group = crate::bool_gesture::group_of_selection(sim, &self.vec.entities, &sel);
                ph2d_panel_vector::state::set_bool_group_selected(group.is_some());
                // **O VERBO DA FORMA: honrar o clique ANTES de publicar** — a ordem é a mesma do
                // chip do recorte, e pela mesma razão: publicar primeiro deixaria o chip a piscar
                // de volta ao valor antigo por um quadro.
                //
                // ⚠️ O escritor **reconfere** a triagem em vez de confiar no que o painel pintou:
                // entre pintar a fileira e o clique chegar passa um frame, e nele a seleção pode
                // ter mudado.
                // ⚠️ O sujeito é o **PRIMÁRIO**, e não «a seleção». Tocar um filho seleciona o
                // GRUPO inteiro (`input_dispatch`), então uma regra de contagem tornava esta
                // fileira inalcançável por clique — foi o defeito de 22/08. O primário sobrevive
                // à expansão (`set_object_selection` preserva-o) e é a forma que o dedo apontou.
                let primary = self.vec.pen.selected();
                if let Some(code) = pending_bool_shape_op {
                    crate::vec_bool_shape::set_selected_shape_op(
                        sim,
                        &self.vec.entities,
                        &self.bool_live,
                        &sel,
                        primary,
                        code,
                    );
                }
                ph2d_panel_vector::state::set_bool_shape_row(
                    crate::vec_bool_shape::shape_row_of_selection(
                        sim,
                        &self.vec.entities,
                        &self.bool_live,
                        &sel,
                        primary,
                    ),
                );
                // ⭐⭐ **FAZER O CONJUNTO** (plano 32 W8) — as formas escolhidas viram um objecto
                // com todas as transições ligadas. ⚠️ **Porta própria e não o `apply`**: ele age
                // sobre o componente de um Morph que aqui ainda **não existe**, e é o `sync` do
                // quadro seguinte que faz nascer a entidade — daí o pendente.
                if pending_morph_arrow == Some(crate::vec_morph_edit::MorphCmd::MakeSet) {
                    if let Some(p) = ph2d_vec_entities::morph_set::create(
                        sim,
                        vec_scene,
                        &self.vec.entities,
                        &sel,
                        ph2d_editor_core::ids::MAX_MORPH_STATES,
                    ) {
                        eprintln!(
                            "[ph2d-vec] morph states: {} formas, {} transicoes",
                            p.members.len(),
                            p.members.len() * (p.members.len() - 1)
                        );
                        // ⭐⭐ **O OBJECTO NOVO FICA SELECCIONADO** — a mesma escolha do botão
                        // Morph ao lado, e aqui ela é load-bearing por uma razão a mais: sem isto
                        // a selecção continuaria a ser as formas-membro, que acabaram de ficar
                        // **ocultas e filhas do conjunto** — e a seção voltaria a oferecer
                        // *"Make Morph States"* sobre elas, prometendo um segundo conjunto por
                        // cima do primeiro.
                        self.vec.pen.select_many(&[p.path]);
                        self.vec.morph_set_pending = Some(p);
                        // ⭐ **COMMIT** — criar o conjunto muda o documento de vez, e é exactamente
                        // o gesto que uma confirmação pelo ouvido serve (a lei do D1).
                        self.pending_ui_sound = Some(crate::ui_sound::UiSound::Commit);
                    }
                }
                // ⭐⭐ **OS TRÊS VERBOS DE MUNDO** (plano 32 W11b) — Play · Desconectar · Desfazer
                // tudo. Eles reparentam, mostram e apagam entidades, então **não** cabem no `apply`
                // (que só tem o componente).
                else if let Some(cmd) = pending_morph_arrow
                    && matches!(
                        cmd,
                        crate::vec_morph_edit::MorphCmd::Play { .. }
                            | crate::vec_morph_edit::MorphCmd::Disconnect { .. }
                            | crate::vec_morph_edit::MorphCmd::Dissolve
                    )
                    && let Some(host) =
                        crate::vec_morph_edit::morph_of_selection(sim, &self.vec.entities, &sel)
                {
                    // ⚠️ **Cada braço deriva o que precisa DENTRO da porta dele** — a lista de
                    // formas era derivada aqui e passada aos três, e era ela que convidava a
                    // escrever a lógica de cada verbo neste `match` (onde nenhum gate a alcança).
                    // ⚠️ **Os dois verbos que APAGAM o conjunto saem pela MESMA porta** — o
                    // `Dissolve` sempre, e o `Disconnect` quando tira a penúltima forma (abaixo de
                    // `MIN_STATES` um conjunto deixa de ser uma relação). Cada um a remover o path
                    // por si seriam duas respostas a *"o que é apagar um conjunto"*.
                    let removed = match cmd {
                        // ⭐ **PLAY: liga a pré-visualização se estiver desligada.** A máquina só
                        // anda dentro do modo (é ele que tem o relógio), e um Play que não tocasse
                        // nada seria um botão morto com nome de verbo.
                        crate::vec_morph_edit::MorphCmd::Play { row } => {
                            self.morph_preview = true;
                            // ⚠️ **Pela porta**, e não por um `get_mut` aqui: este braço corre
                            // DEPOIS do `tick`, que esvazia o mapa fora do modo — o `get_mut`
                            // encontrava-o vazio e o botão só ligava a pré-visualização (report do
                            // Enio, 2026-08-26). A porta abre a máquina semeada pelo mundo.
                            crate::morph_machine_drive::play(
                                &mut self.morph_machines,
                                sim,
                                &self.vec.entities,
                                host,
                                row,
                            );
                            None
                        }
                        // ⚠️ **Pela porta**, e não pelas metades aqui: a segunda — a forma solta
                        // LEVAR as poses dela — não é alcançável de um teste escrita neste braço, e
                        // foi assim que ela ficou por escrever uma wave inteira.
                        crate::vec_morph_edit::MorphCmd::Disconnect { row } => {
                            // ⚠️ **UMA linha por CLIQUE** (`PH2D_MORPH_LOG=1`) — e ela imprime as
                            // duas coisas que decidem se a arrumação alcança a tabela: o
                            // `VecPathId` do conjunto, e as CHAVES que a tabela de States tem.
                            // Se as duas não baterem, a arrumação sai cedo e nada acontece.
                            if crate::morph_machine_drive::log_on() {
                                eprintln!(
                                    "[morph] CLIQUE ⊘ row={row} conjunto={:?} \
                                     chaves-da-tabela={:?} formas={:?}",
                                    ph2d_vec_entities::morph_set::path_of(&self.vec.entities, host),
                                    ui_states.hosts().collect::<Vec<_>>(),
                                    ph2d_vec_entities::morph_set::graph_of(
                                        sim,
                                        &self.vec.entities,
                                        host
                                    )
                                    .shapes(),
                                );
                                // ⚠️ **O INVENTÁRIO da cena**, porque a linha acima disse que a
                                // tabela está sob um id que não é o do conjunto — e a pergunta
                                // seguinte é *o que é aquele id*. Sem isto, a resposta era mais
                                // uma corrida do Enio.
                                for p in vec_scene.paths() {
                                    let e = ph2d_vec_entities::morph_set::path_of(
                                        &self.vec.entities,
                                        host,
                                    )
                                    .filter(|h| *h == p.id);
                                    let ent = self
                                        .vec
                                        .entities
                                        .get(&p.id)
                                        .map(|&b| ph2d_ecs::Entity::from_bits(b));
                                    let (morph, machine, name, pai) =
                                        ent.map_or((false, false, String::new(), None), |en| {
                                            let w = sim.world();
                                            (
                                                w.get::<ph2d_ecs::VecMorph>(en).is_some(),
                                                w.get::<ph2d_ecs::VecMorphMachine>(en).is_some(),
                                                w.get::<ph2d_ecs::Name>(en)
                                                    .map(|n| n.0.clone())
                                                    .unwrap_or_default(),
                                                w.get::<ph2d_ecs::ChildOf>(en).and_then(|c| {
                                                    ph2d_vec_entities::morph_set::path_of(
                                                        &self.vec.entities,
                                                        c.parent(),
                                                    )
                                                }),
                                            )
                                        });
                                    eprintln!(
                                        "[morph]   path {} nome={name:?} morph={morph} \
                                         maquina={machine} pai={pai:?} verts={} e-o-conjunto={}",
                                        p.id,
                                        p.verts.len(),
                                        e.is_some(),
                                    );
                                }
                            }
                            ph2d_vec_entities::morph_set::disconnect_row(
                                sim,
                                &self.vec.entities,
                                host,
                                row,
                            )
                        }
                        crate::vec_morph_edit::MorphCmd::Dissolve => {
                            ph2d_vec_entities::morph_set::dissolve(sim, &self.vec.entities, host)
                        }
                        _ => None,
                    };
                    if let Some(path) = removed {
                        vec_scene.remove_path(path);
                        self.vec.pen.clear();
                    }
                    self.pending_ui_sound = Some(crate::ui_sound::UiSound::Commit);
                }
                // ⭐ **A TECLA de uma forma** (plano 32 W4). As acções são as MESMAS que o menu
                // mostrou (as do Input Map do projecto): resolver o índice contra uma segunda
                // leitura poria o nome escolhido a apontar para outro.
                else if let Some(cmd) = pending_morph_arrow
                    && let Some(e) =
                        crate::vec_morph_edit::morph_of_selection(sim, &self.vec.entities, &sel)
                {
                    let actions: Vec<String> = hero
                        .input_map
                        .actions()
                        .iter()
                        .map(|a| a.name.clone())
                        .collect();
                    crate::vec_morph_edit::apply(sim, &self.vec.entities, e, cmd, &actions);
                }
                if pending_bool_apply
                    && let Some(g) = group
                    && let Some(plan) = self.bool_live.plan(g)
                {
                    let n = crate::bool_gesture::bake(sim, vec_scene, &mut self.vec.pen, plan, g);
                    eprintln!("[ph2d-vec] boolean live: consolidada ({n} path[s])");
                    // ⭐ **COMMIT** (D1): consolidar é o gesto que muda o documento de vez, e é
                    // exactamente o que uma confirmação pelo ouvido serve.
                    self.pending_ui_sound = Some(crate::ui_sound::UiSound::Commit);
                }
                // ⭐⭐⭐ **A RECONCILIAÇÃO dos conjuntos de Morph States** (W11g + W11i) — o par que
                // a cena desenha, a máquina viva, e a tabela de States, todos contra a lista de
                // membros do quadro.
                //
                // ⚠️ **TARDE no quadro, depois do despacho dos painéis, e é uma decisão:** as três
                // rotas que tiram uma forma do conjunto — o ⊘, **apagar** e **arrastar para fora**
                // na Hierarquia — acontecem aqui em cima. A arrumação entra assim na **mesma**
                // fotografia do gesto que a causou, e o artista desfaz tudo num Ctrl+Z; a correr
                // antes, ela chegaria um quadro atrasada e custaria um **segundo** passo.
                crate::morph_machine_drive::reconcile(
                    &mut self.morph_machines,
                    sim,
                    vec_scene,
                    &self.vec.entities,
                    ui_states,
                );
            }
            // **O AUTO LAYOUT roda entre a booleana e o alinhamento** (ADR-0153), e as duas
            // metades da ordem são a lei:
            //
            // - DEPOIS da booleana, porque ele coloca *o que os filhos de fato desenham* — um
            //   grupo booleano é UMA forma para o fluxo, e ela tem de estar cozida antes de ser
            //   medida;
            // - ANTES do alinhamento, porque o alinhamento recorta a faixa do traço na largura
            //   AUTORADA: escalar uma forma depois de a faixa estar recortada esticaria a
            //   espessura dela junto, e o traço do artista mudaria de peso ao redimensionar a
            //   moldura.
            //
            // E ele TRANSFORMA o mapa (não o estende) pelo motivo do `bool_live`: é um componente
            // do PAI, então convive com o offset vivo de cada filho.
            self.layout_live.recook(
                vec_scene,
                sim,
                &self.vec.entities,
                &vec_xf,
                &mut vec_live,
                crate::vec_bindings::TokenCtx {
                    theme: hero.theme,
                    pixels_per_meter: hero.project.pixels_per_meter,
                },
            );
            // **A POSE que cada filho colocado recebeu**, publicada para quem NÃO desenha
            // geometria: as âncoras do modo Node, a caixa do gizmo e o hit-test leem a pose
            // AUTORADA, e ela não se mexeu com o layout.
            vec_view.poses = self.layout_live.poses();
            // **E os TRÊS fatos derivados são PUBLICADOS** para quem vier depois do desenho. O
            // hit-test monta o `VecViewState` dele do zero a cada evento de ponteiro, e aquela
            // porta só sabe o que a ÁRVORE diz (escondido, travado) — sem isto ele decide como se
            // nenhuma moldura existisse, nenhuma forma tivesse sido colocada e nenhum operando
            // tivesse sido absorvido.
            self.vec.view_derived.clips.clone_from(&vec_view.clips);
            self.vec.view_derived.poses.clone_from(&vec_view.poses);
            self.vec
                .view_derived
                .absorbed
                .clone_from(&vec_view.absorbed);
            // **O ALINHAMENTO roda por ÚLTIMO, e TRANSFORMA o mapa em vez de o estender.**
            // Os cinco acima são mutuamente exclusivos (um componente cada, um por vez no
            // painel), e é isso que torna o `extend` seguro. O alinhamento não é membro dessa
            // família — é um campo do `StrokeSpec`, então convive com um offset vivo; fundido
            // por `extend` ele apagaria o offset (ou seria apagado), em silêncio.
            self.align_live.recook(vec_scene, &vec_xf, &mut vec_live);
            // A SILHUETA resolvida das formas TRAÇADAS: `preenchimento ∪ contorno-do-traço`,
            // pela booleana, memoizada na geometria de MUNDO. Sem ela o campo de distância de uma
            // forma com traço cai no caminho do raster, cuja semente discreta desenha o pente que
            // o Enio fotografou no bevel. Roda DEPOIS de `vec_live` (a união é do que se DESENHA).
            self.fx_silhouette
                .recook(vec_scene, sim, &self.vec.entities, &vec_xf, &vec_live);
            // O FX raster por-forma (plano 24 — Blur/Glow/Drop Shadow). O produtor
            // (`fx_live`) rasteriza a forma isolada num scratch de GPU, lê de volta e borra na CPU,
            // injetando as imagens no z da forma. Roda DEPOIS de `vec_live` porque honra a geometria
            // derivada. Sem `VecFilter` na cena o mapa fica vazio = BYTE-IDÊNTICO ao mundo pré-FX.
            // ⭐ **OS LADRILHOS DE PADRÃO** (plano 33, W4) — assados AQUI, e a posição é a lei:
            // ⛔ **ANTES do `fx_live`**, que os consome. O report do Enio (*"filters anula
            // pattern"*) foi exactamente isto: com o assado depois, a rasterização isolada do FX
            // desenhava a forma com a cor de recurso, e a imagem de FX **toma o lugar** do desenho.
            //
            // Memoizados, porque assar custa (`1,047 ms` para um ladrilho de `536x1072` em colmeia)
            // e desenhar não custa nada (uma `fill()`).
            //
            // ⚠️ O filtro sai da porta ÚNICA da casa (`image_quality_for`), a mesma que o upscale e
            // a pré-visualização do BgRemoval usam — um padrão de pixel art tem de amostrar como uma
            // sprite de pixel art, e adivinhá-lo aqui daria duas respostas à mesma pergunta.
            //
            // ⚠️ Usa o `asset_db` que JÁ está desestruturado neste escopo: o empréstimo MUTÁVEL do
            // `gfx` abre muito acima e vive até ao fim do quadro.
            // ⭐ O assador de FORMA (W7) entra INJECTADO: ele é render + readback, e cablá-lo no
            // memo poria uma `GpuContext` na assinatura e tornaria todo gate dele dependente de uma
            // placa. Aqui ele é a porta única `motion_object_bake::bake_rgba`.
            // ⭐⭐⭐ **A arte de uma estampa e' um OBJECTO, e um objecto pode ser um GRUPO**
            // (Enio, 2026-08-30). O `object_selection_for` e' a MESMA porta que o clique no canvas
            // usa para decidir o que uma seleccao apanha — *"um grupo entra e sai da seleccao
            // INTEIRO"* —, e ela devolve os caminhos pela ordem do documento, que e' a de z.
            let object_of = |id| {
                ph2d_vec_entities::entities::object_selection_for(
                    sim,
                    vec_scene,
                    &self.vec.entities,
                    id,
                )
            };
            let mut bake_shape = |id| {
                ph2d_app_motion::motion_object_bake::bake_rgba_many(
                    &mut self.texture_pattern_scratch,
                    vec_scene,
                    &vec_xf,
                    &vec_live,
                    &object_of(id),
                    surface.gpu(),
                    surface.format(),
                    &object_of,
                )
                .map(|(rgba, w, h, _)| (w, h, rgba))
            };
            // ⭐⭐⭐ **A POSE dos membros entra pela MESMA fonte que o assado lê** (report do Enio,
            // 2026-08-30: *"ao mover os objetos do grupo que serve como shape, a pattern não
            // atualiza em tempo real"*). O `bake_rgba_many` acima recebe o `vec_xf`; a chave do memo
            // tem de o ler também, senão ela é cega ao gesto — a geometria de um `VecPath` é LOCAL
            // (ADR-0110) e mover um membro não lhe toca um byte.
            let pose_of = |id| vec_xf.get(&id).copied().unwrap_or_default();
            self.texture_pattern_live.recook(
                vec_scene,
                asset_db,
                ph2d_editor_core::image_quality_for(hero.project.image_filter),
                &mut bake_shape,
                &object_of,
                &pose_of,
            );
            self.fx_live.recook(
                vec_scene,
                sim,
                &self.vec.entities,
                &vec_xf,
                &vec_live,
                self.fx_silhouette.live(),
                &vec_view,
                self.texture_pattern_live.tiles(),
                cam_affine,
                surface.gpu(),
                surface.format(),
                vello_pass,
            );
            // doc 86 §2 (A2): bake the named vector shapes to tiles, right where
            // the FX stack bakes — the same handles (renderer, gpu, scene,
            // transforms, live geometry) are in hand. Cached by content ⇒ a
            // static scene bakes once; the membrane publishes the tiles next frame.
            ph2d_app_motion::motion_bridge::bake_objects(
                motion,
                vec_scene,
                &self.vec.entities,
                &vec_xf,
                &vec_live,
                surface.gpu(),
                renderer,
                surface.format(),
                sim,
            );
            // ⚠️ **E os tiles das formas PARAMÉTRICAS** (bug do Enio, 2026-08-20:
            // *"tudo deve brilhar"*). Irmão do bake acima e no mesmo sítio pela mesma
            // razão — `renderer` + `gpu` em mão. Só assa o que FALTA, e só o que o
            // `object_bake` não cobre: as duas rotas partilham o store, então uma
            // geometria de objeto já tem tile e não pode pagar um segundo.
            {
                // ⚠️ **E só assa se HOUVER quem consuma o tile.** Ele existe para o
                // bright-pass do glow e para mais nada; o `present` já pergunta pelo
                // `fx.glow` com esta mesma função, e a pergunta é feita AQUI pela MESMA
                // porta para as duas não divergirem. Sem a guarda, uma forma com um param
                // animado paga um **readback de GPU por quadro** por um tile que ninguém
                // lê — e o readback é a metade lenta deste assador, como o doc dele diz.
                let glows = ph2d_node_fx_glow::from_graph(&motion.doc.graph)
                    .is_some_and(|g| g.intensity > 0.0);
                let ph2d_app_motion::motion_state::MotionState {
                    shape_bake,
                    shape_store,
                    object_bake,
                    pump,
                    ..
                } = &mut *motion;
                // ⚠️ **O conjunto VIVO é o de todas as instâncias deste quadro** — é ele
                // que decide o DESPEJO. O pedido ao assador é um subconjunto (tira o que
                // o `object_bake` já cobre); despejar por ele largaria um tile que ainda
                // está em cena. Ver `ShapeBake::evict_outside`, e o OOM que o motivou.
                let live: std::collections::BTreeSet<u32> = pump
                    .vector_instances
                    .iter()
                    .map(|vi| vi.geometry_id)
                    .collect();
                // Sem glow ninguém lê tile nenhum, então o conjunto pedido é VAZIO — e o
                // despejo abaixo corre na mesma, largando o que a sessão já assou.
                let wanted: Vec<u32> = live
                    .iter()
                    .copied()
                    .filter(|_| glows)
                    .filter(|gid| object_bake.tile_texture_for_gid(*gid).is_none())
                    .collect();
                // ⚠️ **`PH2D_GLOW_DIAG=1`** diz se este assador correu e o que ele
                // conseguiu — sem isto, «não assou» e «não foi chamado» leem igual.
                let asked = wanted.len();
                shape_bake.bake_missing(
                    shape_store,
                    wanted,
                    surface.gpu(),
                    renderer,
                    surface.format(),
                );
                // ⚠️ **E LARGA o que saiu de cena, libertando a textura.** Sem isto um
                // param de forma animado assa um tile por QUADRO e a placa acaba
                // (medido: OOM no quadro 19706 da `=76`).
                let freed = shape_bake.evict_outside(&live, renderer);
                if freed > 0 && std::env::var_os("PH2D_GLOW_DIAG").is_some() {
                    eprintln!("[glow-diag] assador de formas: tiles largados={freed}");
                }
                if asked > 0 && std::env::var_os("PH2D_GLOW_DIAG").is_some() {
                    let done = pump
                        .vector_instances
                        .iter()
                        .filter(|vi| shape_bake.tile_for_gid(vi.geometry_id).is_some())
                        .count();
                    eprintln!(
                        "[glow-diag] assador de formas: pedidas={asked} com_tile_agora={done}"
                    );
                }
            }
            // doc 86 §2 (A3): bake the named FLIP objects to tiles, alongside the
            // vector bake. The Flip doc is destructured above (`flip`); the entity
            // map + playhead are disjoint `self` fields. Composes each object's
            // layers at the current frame through a scratch Flip raster + compositor.
            ph2d_app_motion::motion_bridge::bake_flip_objects(
                motion,
                flip,
                &self.flip_state.entities,
                &self.playhead,
                surface.gpu(),
                renderer,
                sim,
            );
            let vec_fx = self.fx_live.images();
            let vec_patterns = self.texture_pattern_live.tiles();
            // As PELES de widget deste frame (plano UI/UX W6.2). Cozidas AQUI, depois do `sync`
            // (senão uma forma recém-marcada ainda não tem entidade) e com a câmera na mão —
            // quem sabe onde a forma está na tela é quem tem a projeção.
            // **O PAINEL AUTORADO SEGUE O DOCUMENTO** — publicado a cada quadro em que ele
            // está na tela. Sem isto o painel desenha a tabela COMPILADA, e o artista precisa de
            // colar o código gerado e recompilar para ver o que acabou de desenhar: um ciclo de
            // compilação dentro do laço de autoria (report do Enio, 2026-08-09).
            //
            // ⚠️ **Só com o painel VISÍVEL**, e é a lei do ADR-0125: o custo de descrever um
            // painel é trabalho de autoria, não de quadro. Fechado, ele não paga nada — e a
            // publicação de `None` devolve o painel à tabela compilada, que é o que um build sem
            // documento autorado tem de mostrar.
            let authored_frame = hero
                .is_panel_visible(
                    <ph2d_panel_authored::AuthoredPanel as ph2d_editor_core::panel::Panel>::ID,
                )
                .then(|| crate::ui_panel_spec::authored_frame(sim, vec_scene))
                .flatten();
            // **O RETORNO DO PICKER** — a cor escolhida pinta a forma que veste a swatch.
            //
            // ⚠️ **ANTES de derivar as rows, e a ordem é o assunto:** a row publica o
            // preenchimento da forma, então escrever a cor depois de a ler daria uma swatch a
            // mostrar a cor ANTIGA por um quadro — o piscar que faz o artista clicar duas vezes.
            //
            // ⚠️ E o alvo do picker é PARTILHADO (Painter, Vector, timeline usam o mesmo canal);
            // o `picker_shape` devolve `None` quando ele não é uma row desta moldura, que é o caso
            // comum. Escrever sem essa pergunta pintaria a forma errada a cada vez que outro
            // painel abrisse o picker.
            if let Some(frame) = authored_frame
                && let Some(target) = hero.store.picker_target()
                && let Some(path) =
                    crate::ui_panel_spec::picker_shape(sim, vec_scene, frame, target)
                && let Some((value, _, _, _)) = hero
                    .store
                    .blender_picker(ph2d_editor_core::ids::INSP_BLENDER_PICKER)
            {
                // ⚠️ A porta RECUSA a cor igual, e é ela que impede a escrita ao ABRIR: o
                // `pointer_down` semeia o picker no clique da swatch, então sem a recusa o gesto
                // de *olhar* a cor escreveria o documento — achatando um gradiente e gravando um
                // passo de undo por quadro. A lei é a do `set_piece_colour`, ali em cima.
                crate::ui_panel_spec::paint_swatch_colour(vec_scene, path, value.rgba);
            }
            let live_rows =
                authored_frame.map(|f| crate::ui_panel_spec::live_rows(sim, vec_scene, f));
            ph2d_panel_authored::rows::set_live_rows(live_rows);
            let vec_skins = crate::widget_live::build(
                vec_scene,
                sim,
                &self.vec.entities,
                &vec_xf,
                &vec_live,
                cam_affine,
                paint_ctx.text,
                hero.theme,
            );
            // ⭐⭐⭐ **AS FAIXAS DO DOCUMENTO** (ADR-0154 Fase 2). Quando a cena INTERCALA vetor e
            // sprite, o documento deixa de ir para a cena do chrome e passa a ser codificado uma
            // vez por faixa, em cenas próprias — o presente desenha-as intercaladas com as faixas
            // de sprite, e o chrome fica por cima de tudo, como sempre.
            //
            // ⭐ **A faixa exprime-se ESCONDENDO o resto**, e não filtrando o laço do `dispatch`.
            // A razão está escrita lá dentro: o `push`/`pop` da camada de recorte vive **fora** do
            // filtro de escondido, de propósito, para as molduras se emparelharem mesmo quando não
            // desenham. ⇒ uma forma fora da faixa não desenha **e** a moldura dela continua a
            // recortar quem cai lá dentro. Filtrar o laço desemparelharia a pilha.
            //
            // ⚠️ Sem intercalação isto fica vazio e o documento vai para a cena do chrome, byte a
            // byte como sempre.
            // ⭐⭐⭐ **A arte dos PINCÉIS deste quadro, MEMOIZADA** (`line/Vector`, 2026-08-30) —
            // resolvida aqui pela mesma razão que o ladrilho do padrão o é: a crate de desenho não
            // alcança a cena, e o guarda de ciclo tem de viver onde se pode medir.
            //
            // ⚠️ **Resolvida UMA vez, FORA do laço das faixas** (integração de 2026-09-04): a
            // `line/Vector` memoizou-a porque sem memo `50` pincéis com grupos de `16` custam
            // **14,28 ms — 85,5% de um quadro**; a `line/components` pôs o `dispatch` dentro de um
            // laço por faixa. As duas juntas pagariam a montagem da chave uma vez POR FAIXA, e o
            // mapa é o mesmo para todas — ele é função da cena, não da faixa.
            let brush_arts = self.brush_live.resolve(
                vec_scene,
                &|id| {
                    ph2d_vec_entities::entities::object_selection_for(
                        sim,
                        vec_scene,
                        &self.vec.entities,
                        id,
                    )
                },
                &vec_xf,
            );
            band_doc_scenes.clear();
            // ⭐⭐⭐ **HÁ RECEITA ABERTA NESTE QUADRO?** — o interruptor do vidro jateado, escrito
            // UMA vez e lido pelo presente (re-derivá-lo lá seria a segunda resposta, e um quadro
            // em que as duas discordassem desenharia a receita duas vezes ou nenhuma).
            //
            // ⚠️⚠️ **A pergunta é ao MUNDO e não à vista do vetor:** o `isolated` dela é enchido a
            // partir das FORMAS marcadas, e uma receita feita só de imagens deixa-o vazio — o vidro
            // nunca subiria justamente para os prefabs de sprite.
            *frosting = ph2d_app_components::master_editing::any_open(sim);
            frost_doc_scene.reset();
            frost_front_scene.reset();
            let doc_bands = crate::draw_bands::doc_bands_of(frame_order);
            for band in &doc_bands {
                let keep = frame_order.vector_ids_in(*band);
                let mut band_view = vec_view.clone();
                for path in vec_scene.paths() {
                    if !keep.contains(&path.id) {
                        band_view.hidden.push(path.id);
                    }
                }
                let mut target = ph2d_vector::VectorScene::new();
                ph2d_vec_render::dispatch(
                    vec_scene,
                    &band_view,
                    &vec_xf,
                    &vec_live,
                    vec_fx,
                    &vec_skins,
                    vec_patterns,
                    brush_arts,
                    self.paint_dilate_live.out(),
                    cam_affine,
                    &mut target,
                );
                band_doc_scenes.push(target);
            }
            if !doc_bands.is_empty() {
                // O documento já foi codificado nas faixas — a cena do chrome fica só com o chrome.
            } else {
                // ⭐⭐⭐ **COM O VIDRO, o documento sai da cena do CHROME.** Ele tem de aterrar no
                // acumulador do mundo **antes** do borrão, e os painéis entram depois dele; na
                // mesma cena, os dois seriam borrados ou nítidos juntos. ⛔ Sem receita aberta é a
                // cena de sempre, byte a byte.
                let target: &mut ph2d_vector::VectorScene = if *frosting {
                    frost_doc_scene
                } else {
                    vector_scene
                };
                ph2d_vec_render::dispatch(
                    vec_scene,
                    &vec_view,
                    &vec_xf,
                    &vec_live,
                    vec_fx,
                    &vec_skins,
                    vec_patterns,
                    brush_arts,
                    self.paint_dilate_live.out(),
                    cam_affine,
                    target,
                );
            }
            // ⭐⭐⭐ **E A RECEITA, sozinha, na cena que fica ACIMA do vidro.** ⚠️ Ela é codificada
            // aqui **em todos os casos** — com faixas ou sem elas —, porque o `dispatch` e as
            // faixas saltam-na sempre: sem esta chamada a receita aberta simplesmente não desenha.
            // ⛔ Sem receita aberta a porta devolve sem escrever nada.
            ph2d_vec_render::dispatch_isolated(
                vec_scene,
                &vec_view,
                &vec_xf,
                &vec_live,
                vec_fx,
                &vec_skins,
                vec_patterns,
                brush_arts,
                self.paint_dilate_live.out(),
                cam_affine,
                frost_front_scene,
            );
            // ⚠️ **O mapa que foi DESENHADO fica guardado, e é ele que o PICK lê.**
            //
            // O `vec_gizmo_pick` declara no próprio doc que a pergunta *"o que está desenhado
            // aqui?"* é feita ao MESMO mapa que este `dispatch` consome — e a fiação contradizia-o:
            // os seis sítios de pick da `input_dispatch` passavam só o `offset_live`, então tudo o
            // que os outros oito produtores desenham era **visível e não-clicável** (medido: numa
            // simetria armada, 3 de 3 pontos da metade espelhada estão na tela e o clique
            // atravessa).
            //
            // ⚠️ É um **MOVE**, não um clone: `vec_live` morre aqui, e a fusão é remontada do zero
            // no frame seguinte. Guardar custa zero; re-derivar no input custaria a segunda porta.
            //
            // ⚠️ E é isto que faz um produtor NOVO nascer coberto: quem acrescenta uma linha à
            // fusão acima ganha o pick de graça, sem saber que este parágrafo existe.
            self.vec.live_drawn = vec_live;
            // ADR-0154: the live GPU shapes of the Motion scene. Gated on the
            // Motion tool like the Motion sprites (present.rs) — a `source.shape`'s
            // `geometry_id` instances are drawn into the SAME scene the vector
            // document rides in, so they composite behind the chrome and over the
            // sprites (Fase 1: vector over sprite), aligned with the Motion sprites
            // by the same `cam_affine`.
            if motion_tool_active {
                // ⭐⭐⭐ **A ARTE dos quads do passe vectorial** (a terceira média): resolvida
                // aqui porque é aqui que o `renderer` e a GPU estão em mão, e memoizada em
                // [`ph2d_app_motion::motion_leaf_images`] porque cada leitura PARA a GPU.
                let (gpu, atlas, individual) =
                    (surface.gpu(), renderer.atlas(), renderer.individual());
                // ⚠️ **O `synced` é a única porta**, e ele recebe o relógio de mudança do
                // atlas — esquecer a sincronização é erro de compilação (§2.5).
                let mut cache = self.motion_shell.leaf_images.synced(atlas.epoch());
                let mut art = |tex: u32, uv: [f32; 4]| cache.art(gpu, atlas, individual, tex, uv);
                ph2d_app_motion::motion_shape_gen::encode(
                    &motion.pump.vector_instances,
                    &motion.shape_store,
                    &mut art,
                    cam_affine,
                    vector_scene,
                );
                // ⛔⛔ **LARGAR O ATLAS** (auditoria §2.5): a cópia em CPU dele é `268 MB` e
                // ficava retida pela vida do processo. Os RECORTES ficam — eles são a resposta
                // memoizada e são pequenos.
                self.motion_shell.leaf_images.end_frame();
            }
            // O **overlay** do Blend Object (ADR-0128): os passos virtuais + as fontes de cima
            // reempilhadas, na ordem de z (a última fonte por cima do último passo). Desenha depois
            // do `dispatch` (que já pôs as fontes no z da cena, embaixo); o overlay reestabelece a
            // pilha do blend por cima. O interleaving fino contra o resto da cena é da Fase C.
            ph2d_vec_render::draw_blend_overlay(&self.vec.blend_overlay, cam_affine, vector_scene);
            // ⛔ **AS SETAS DO MORPH NÃO SE DESENHAM** (Enio, 2026-08-25: *"as setas são virtuais e
            // ninguém jamais vê"*). A W3a pintava-as aqui, em âmbar, entre as formas que ligavam.
            // Elas deixaram de existir como desenho porque deixaram de ser AUTORADAS: o conjunto é
            // o grafo COMPLETO, gerado por um botão — desenhar `n(n-1)` setas entre formas que já
            // estão escondidas é ruído sobre uma resposta que ninguém precisa de ler no canvas.
            // A lista da seção *Morph States* é a superfície, e é lá que a condição se escolhe.
            // **Pick Shapes** (ADR-0128 C2b): realça as formas escolhidas e costura a ORDEM de
            // clique numa polilinha (a prévia do spine). Fora do modo Pick, a lista não vale —
            // limpa, para não vazar escolhas velhas para o próximo blend.
            if vector_active && self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::PickBlend {
                let preview =
                    crate::blend_live::pick_preview(vec_scene, &vec_xf, &self.vec.blend_picks);
                ph2d_vec_render::draw_blend_overlay(&preview, cam_affine, vector_scene);
            } else if !self.vec.blend_picks.is_empty() {
                self.vec.blend_picks.clear();
            }
            // **O Picker de caminho-guia** (Enio 2026-07-23): armado, o caminho sob o cursor é o que
            // o clique vai prender — a silhueta dele acende, o idioma do conta-gotas. `path_at` é o
            // MESMO resolvedor do clique, então o realce nunca mente sobre o que será escolhido. Fora
            // do modo Select (ou sem a tool) o pick não faz sentido: limpa, para não ficar armado e
            // invisível — a saída sem compromisso, como o clique no vazio.
            if let Some(pick) = self.vec.path_pick {
                if vector_active && self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Select
                {
                    let w = camera.screen_to_world(self.last_pointer, window_size);
                    let a = camera.screen_to_world((0.0, 0.0), window_size);
                    let b = camera.screen_to_world((1.0, 0.0), window_size);
                    // LITERAL-PX-OK: raio de acerto em px, o MESMO do picking de canvas (`path_at`).
                    let hit_r =
                        10.0 * f64::from(((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt());
                    if let Some(gid) =
                        self.vec
                            .pen
                            .path_at(vec_scene, [f64::from(w[0]), f64::from(w[1])], hit_r)
                        && gid != pick.source()
                        && let Some(outline) =
                            crate::vec_pick::hover_outline(vec_scene, &vec_xf, gid)
                    {
                        ph2d_vec_render::draw_blend_overlay(&[outline], cam_affine, vector_scene);
                    }
                } else {
                    self.vec.path_pick = None;
                }
            }
            // Âncoras/handles/gradiente/marquee só interessam a quem edita nós; no
            // modo Select quem fala é o gizmo (ADR-0112). As guias de snap são caso à
            // parte (valem em TODOS os modos) — `vec_overlay` separa as duas políticas
            // num ponto testável (P1).
            let overlay =
                crate::vec_overlay::vec_overlay_plan(vector_active, self.vec.draw_config.mode);
            // ⭐ **O CONTORNO DE PROVENIÊNCIA** (estudo de UI viva, C2) — a forma que o ponteiro
            // aponta, venha ele do canvas ou de uma linha da Hierarquia.
            //
            // ⚠️ **FORA do `overlay.edit`, e é a metade do desenho:** aquele portão só abre no
            // modo Node, e é justamente no modo **Select** — onde o artista escolhe qual das cinco
            // formas de uma booleana quer — que a pergunta *"qual delas é esta?"* se faz.
            //
            // ⚠️ E **não** quando ela já está selecionada: o gizmo já a nomeia, e um segundo
            // realce por cima do primeiro diz duas vezes a mesma coisa com tintas diferentes.
            //
            // ⚠️ **SEM portão de modo** (Enio, 2026-08-23: *"pode estender isso para todos os
            // objetos mesmo fora do modo vector?"*): o realce segue o CLIQUE, e o clique pega
            // objecto em qualquer modo. Um portão de modo aqui faria a pergunta *"qual destes é
            // este?"* só ter resposta onde ela já era mais fácil.
            if let Some(bits) = self.hovered_object
                && !self.hover_outline.is_empty()
                && !hero.gizmo.iter_selected().any(|s| s == bits)
            {
                ph2d_vec_render::draw_hover_outline(&self.hover_outline, cam_affine, vector_scene);
            }
            // ⭐⭐⭐ **O REALCE DO TRIM** (plano 38): o pedaço que o clique vai apagar, a vermelho,
            // como no Fusion. ⚠️ **A geometria vem da MESMA porta que o corte** — ela é calculada
            // no dreno do ponteiro e guardada, então o que acende neste quadro é literalmente o que
            // o `vec_trim::apply` vai comer. Uma segunda conta aqui seria a divergência mais cara
            // que uma ferramenta destrutiva pode ter.
            if !self.vec.trim_piece.is_empty() {
                ph2d_vec_render::draw_trim_piece(&self.vec.trim_piece, cam_affine, vector_scene);
            }
            // ⭐⭐⭐ **A FACE que o Balde vai preencher** (plano 40), na TINTA que ele vai depositar.
            // ⚠️ A geometria vem da MESMA porta que o preenchimento usa; e a tinta é a corrente,
            // não uma cor neutra — um realce noutra cor prometeria uma coisa e entregaria outra.
            // ⚠️ A tinta é lida do CAMPO (`vec_pen`), e não pelo `bucket_paint()`: um método em
            // `&self` pede o objecto INTEIRO emprestado, e aqui há um empréstimo mútuo vivo. Os
            // campos são disjuntos; a lei do `alpha == 0` é a mesma dos dois lados.
            let tinta_balde = self.vec.pen.style().fill;
            if let Some(face) = self.vec.bucket_face.as_ref()
                && tinta_balde.a != 0
            {
                let t = tinta_balde;
                ph2d_vec_render::draw_bucket_face(
                    &face.face,
                    [t.r, t.g, t.b, t.a],
                    cam_affine,
                    vector_scene,
                );
            }
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
                        self.vec.pen.selected(),
                        self.vec.pen.selected_paths(),
                        self.vec.pen.selected_verts(),
                        &vec_xf,
                        cam_affine,
                        vector_scene,
                    );
                    // ⭐⭐⭐ **A MARCA DO NÓ SOLDADO** (plano 39). Report do Enio (2026-09-01):
                    // *"as linhas não compartilham o mesmo nó"* — e ele **não tinha como ver**:
                    // duas pontas coincidentes e duas pontas a um pixel pintam o mesmo quadrado.
                    //
                    // ⚠️ **A LEI é a do arrasto** (`welded_nodes`, mesma `WELD_TOL` e mesmo
                    // predicado do `welded_with`); o que se decide AQUI é só o que se vê: um
                    // caminho escondido não mostra marca, como não mostra âncora.
                    //
                    // ⚠️ **E a projecção é a MESMA do overlay** (`overlay_transform`) — um segundo
                    // caminho poria o anel ao lado da âncora que ele afirma abraçar, e justamente
                    // sob auto layout, onde conferir a olho é mais difícil.
                    let marcas: Vec<[f64; 2]> = self
                        .vec
                        .pen
                        .welded_nodes(vec_scene)
                        .into_iter()
                        .filter(|(id, _)| !vec_view.is_hidden(*id))
                        .filter_map(|(id, i)| {
                            let v = vec_scene.path(id)?.vert(i)?;
                            let t = ph2d_vec_render::overlay_transform(
                                &vec_view, &vec_xf, id, cam_affine,
                            );
                            let p = t * ph2d_vector::Point::new(v.anchor[0], v.anchor[1]);
                            Some([p.x, p.y])
                        })
                        .collect();
                    ph2d_vec_render::draw_weld_marks(&marcas, hero.theme, vector_scene);
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
                        self.vec.envelope_drag,
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
                        self.vec
                            .envelope_drag
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
                if let Some(sel) = self.vec.pen.selected() {
                    ph2d_vec_render::draw_gradient_handles(
                        vec_scene,
                        Some(sel),
                        self.vec.grad_selected,
                        ph2d_vec_render::path_to_screen(&vec_xf, sel, cam_affine),
                        vector_scene,
                    );
                }
                // O gesto de REGIÃO em curso, em px de tela — retângulo ou LAÇO, conforme a
                // forma que o press congelou.
                if let Some(m) = self.vec.marquee.as_ref() {
                    match m.shape {
                        ph2d_tool_vector::params::MarqueeShape::Box => {
                            ph2d_vec_render::draw_marquee(
                                [f64::from(m.start.0), f64::from(m.start.1)],
                                [f64::from(m.cur.0), f64::from(m.cur.1)],
                                vector_scene,
                            );
                        }
                        ph2d_tool_vector::params::MarqueeShape::Lasso => {
                            let pts: Vec<(f64, f64)> = m
                                .closed_path()
                                .into_iter()
                                .map(|(x, y)| (f64::from(x), f64::from(y)))
                                .collect();
                            ph2d_vec_render::draw_lasso(&pts, vector_scene);
                        }
                    }
                }
            }
            let pele_suave = match self.vec.draw_config.skin_deform {
                ph2d_tool_vector::SkinDeform::Fast => None,
                ph2d_tool_vector::SkinDeform::Smooth => {
                    Some(crate::skeleton_skin_image::refine_options())
                }
            };
            // ⭐⭐⭐ **AS IMAGENS PRESAS AO ESQUELETO** — a 2.ª mídia (ordem do dono, 2026-09-09).
            //
            // ⚠️ **ANTES dos ossos, e a ordem é a leitura:** a imagem é a ARTE e o rig é o chrome
            // que se desenha por cima dela. Invertê-la esconderia o esqueleto debaixo do desenho
            // exactamente quando o artista o está a posar.
            //
            // ⚠️ **A sprite original é escondida pelo passe de sprites** (`vec_overlay::skinned`),
            // senão ela ficaria por baixo, por deformar — e o artista veria a arte DUAS vezes.
            //
            // ⚠️ **A escolha é lida ANTES do `&mut self.skin_image_cache`**: os dois vivem no
            // `self`, e o compilador não deixa emprestar um deles imutavelmente no meio da chamada
            // que já empresta o outro mutavelmente.
            crate::skeleton_skin_image::draw_skinned_images(
                sim,
                asset_db,
                &mut self.skeleton.skin_image_cache,
                cam_affine,
                vector_scene,
                pele_suave,
            );
            // ⭐⭐⭐ **OS OSSOS** (estudo 42 item 5): desenhados enquanto a ferramenta de VETOR está
            // na mão, e só então.
            //
            // ⚠️ **FORA do `overlay.edit`, e é a mesma razão da linha de corte abaixo:** aquele
            // portão fecha em Select/Build/Bucket porque *âncoras* ali são ruído — mas um osso não
            // é uma âncora, é o corpo do rig. Escondê-lo no Select tiraria da tela a única coisa
            // que diz onde o esqueleto está enquanto se mexe nas formas dele.
            //
            // ⛔ Não é uma forma da cena: nada disto entra no documento, no SVG ou no z-order.
            if overlay.bones {
                let ossos = crate::skeleton_live::bone_segments(sim);
                if !ossos.is_empty() {
                    // ⭐⭐⭐ **A REGIÃO DE INFLUÊNCIA do osso em foco** — o *Bone Strength* do Moho.
                    // Ela entra ANTES dos ossos: é um fundo, e o rig desenha-se por cima dela.
                    //
                    // ⚠️ **O foco é a SELECÇÃO, e é a mesma pergunta que o dedo faz** — a alça só é
                    // agarrável onde ela é pintada (`bone_pick::hover` recebe o mesmo `foco`).
                    //
                    // ⚠️ A selecção CRUA basta e filtra-se sozinha: `influence_region` devolve
                    // `None` para o que não é osso, então não há aqui uma segunda pergunta
                    // *"isto é um osso?"* a divergir da que o `hover` faz.
                    // ⭐⭐⭐ **O OSSO EM FOCO SAI DA MESMA PORTA QUE O DEDO USA**
                    // ([`crate::bone_gesture::selected_bone`]), e não do primário do gizmo.
                    //
                    // ⛔⛔ **Eram DUAS respostas para «qual osso está em foco», e o doc de uma delas
                    // afirmava serem a mesma.** O dedo lê a selecção INTEIRA (o `selected_bone`
                    // explica porquê: prender uma forma a um esqueleto entre vários faz-se
                    // escolhendo os dois, e aí **o primário é a forma**); o desenho lia só o
                    // primário. ⇒ com uma forma seleccionada ao lado do osso, o dedo oferecia as
                    // alças de um osso e o canvas pintava-as noutro sítio — ou em sítio nenhum.
                    // Report do dono (2026-09-08): *«gizmo não mantém ângulo fixo em relação ao
                    // osso»*.
                    //
                    // ⚠️ O doc do `selected_bone_bits` já prescrevia isto: *«no laço de desenho o
                    // `gfx` está emprestado mutável de ponta a ponta, e ali chama-se a função livre
                    // acima — a lei é a mesma, e é por isso que ela vive numa função só»*.
                    let osso_focado =
                        crate::bone_gesture::selected_bone(sim, hero.gizmo.iter_selected());
                    ph2d_skeleton_render::draw_influence(
                        osso_focado.and_then(|b| crate::skeleton_live::influence_region(sim, b)),
                        matches!(
                            self.skeleton.bone_hover,
                            Some(h) if h.part == ph2d_skeleton_render::BonePart::Influence
                        ),
                        cam_affine,
                        hero.theme,
                        vector_scene,
                    );
                    // ⭐⭐⭐ **O ARCO DE LIMITE do osso em foco** — o setor por onde a ponta dele
                    // pode passar, mais as duas paredes agarráveis.
                    //
                    // ⚠️ **Depois da influência e ANTES dos ossos**: os dois são fundo, e o arco
                    // vive por cima da mancha (é por isso que o véu dele é mais fraco). O rig
                    // desenha-se por cima dos dois.
                    //
                    // ⚠️ **A mesma pergunta que o dedo faz** — `bone_pick::hover` só oferece as
                    // paredes do osso em FOCO, e é esta linha que decide de quem elas são.
                    ph2d_skeleton_render::draw_limit(
                        osso_focado
                            .and_then(|b| {
                                // ⚠️ **O MESMO zoom que o dedo usa** (`bone_pick::hover` recebe
                                // este `vec_px_to_world`): a folga das alças é uma grandeza de TELA
                                // sobre geometria de MUNDO, e dois zooms diferentes poriam a alça
                                // pintada num sítio e a agarrável noutro.
                                crate::bone_limit::arc(
                                    sim,
                                    ph2d_ecs::Entity::from_bits(b),
                                    vec_px_to_world,
                                )
                            })
                            .as_ref(),
                        self.skeleton.bone_hover.map(|h| h.part),
                        cam_affine,
                        hero.theme,
                        vector_scene,
                    );
                    // ⭐⭐⭐ **EM *CRIAR*, TODA PONTA É UMA PORTA** (ordem do dono, 2026-09-09:
                    // *«para criar um osso como filho de outro o clique deve acontecer na ponta do
                    // osso pai»*).
                    //
                    // ⛔⛔ **Sem isto o alvo do parentesco seria INVISÍVEL no meio de uma corrente:**
                    // ali a ponta de um osso é a raiz do seguinte, e o que está desenhado no ponto é
                    // a bolinha da junta do FILHO — o artista veria o alvo de outro osso onde tem de
                    // carregar para ramificar deste. *Um alvo que não está onde a coisa parece estar
                    // é um alvo ausente* (a lei que as paredes do limite já pagaram).
                    //
                    // ⚠️ **O anel só muda de VERBO com o modo, nunca de sítio**: em *Transformar* ele
                    // é o *end effector* (cinemática inversa) e por isso só existe em quem fecha a
                    // corrente e não tem âncora; em *Criar* ele é *«daqui nasce um filho»*, que vale
                    // para todo osso. É a mesma alça a dizer o que o clique faz AGORA.
                    let criar = self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Bone
                        && self.vec.draw_config.bone_action == ph2d_tool_vector::BoneAction::Create;
                    let pontas: Vec<u64> = if criar {
                        ossos.iter().map(|&(b, _, _)| b).collect()
                    } else {
                        // ⭐⭐ **As pontas SEM âncora** — num osso ancorado o anel é substituído pelo
                        // losango do alvo, e desenhar os dois prometeria dois verbos onde há um.
                        crate::skeleton_goal::unanchored_ends(sim)
                    };
                    ph2d_skeleton_render::draw_bones(
                        &ossos,
                        hero.gizmo.selection,
                        self.skeleton.bone_hover,
                        &pontas,
                        cam_affine,
                        hero.theme,
                        vector_scene,
                    );
                    // ⭐⭐⭐ **AS ÂNCORAS DE IK** — o losango do alvo e o tracejado até à ponta. Elas
                    // vêm DEPOIS dos ossos porque o alvo é o que a mão agarra: ele fica por cima.
                    ph2d_skeleton_render::draw_goals(
                        &crate::skeleton_goal::anchors(sim),
                        hero.gizmo.selection,
                        // ⛔ **Em *Criar* o `Tip` quer dizer «daqui nasce um filho», não «arrasta a
                        // âncora»** — e o losango do alvo pode estar LONGE da ponta. Passar o realce
                        // acenderia, a metros do dedo, uma alça que aquele modo não executa.
                        if criar {
                            None
                        } else {
                            self.skeleton.bone_hover
                        },
                        cam_affine,
                        hero.theme,
                        vector_scene,
                    );
                }
                // ⭐⭐⭐ **O OSSO QUE ESTÁ A NASCER** (Enio, 2026-09-07: *«deve aparecer logo no
                // mouse down e crescer conforme o usuário arrasta»*). ⛔ Ele fica FORA do `if
                // !ossos.is_empty()` de propósito: o PRIMEIRO osso de uma cena nasce quando não há
                // osso nenhum, e era exactamente esse que o artista desenhava às cegas.
                if let Some((origem, ponta, arma)) = self.skeleton.bone_preview {
                    ph2d_skeleton_render::draw_bone_preview(
                        origem,
                        ponta,
                        arma,
                        cam_affine,
                        hero.theme,
                        vector_scene,
                    );
                }
            }
            // **A LINHA DE CORTE** (W4) — hachurada, com a tesoura na ponta. Ela é a única
            // geometria da cena que o render de ARTE não desenha (perde fill e stroke ao ser
            // adotada como lâmina), e é aqui que ela reaparece.
            //
            // ⚠️ **FORA do `overlay.edit`, e a razão é um defeito reportado** (Enio, 2026-07-31:
            // *"quando movido com Select a aparência rachurada deve permanecer"*): aquele guard é
            // FALSO no Select, então a lâmina — que o render de arte não desenha — **desaparecia
            // por completo** justamente no modo em que se a move. A hachura não é feedback de um
            // MODO: ela é a aparência do objeto, e um objeto não muda de aparência porque o
            // artista pegou outra ferramenta.
            for id in crate::vec_cut_line::cut_lines(sim, &self.vec.entities) {
                if let Some(p) = vec_scene.paths().iter().find(|p| p.id == id) {
                    ph2d_vec_render::draw_cut_line(
                        &ph2d_vec_render::build_bezpath(p),
                        ph2d_vec_render::path_to_screen(&vec_xf, id, cam_affine),
                        vector_scene,
                    );
                }
            }
            // Smart guides do snap: FORA do guard de modo — explicam o encaixe vivo em
            // qualquer modo, inclusive o gizmo-move do Select (P1, ADR-0112).
            if overlay.snap_guides {
                // As guias do DOCUMENTO primeiro: elas são o fundo permanente contra o qual a
                // guia de snap (viva, tracejada, de um frame) se lê. Desenhá-las por cima
                // esconderia a marca do encaixe atrás da linha que ele acabou de encontrar.
                //
                // ⚠️ O recorte é a JANELA DA CENA, não o retângulo do canvas: o `cam_affine`
                // projeta nas dims da cena, e o chrome dos painéis é pintado DEPOIS, por cima.
                // Pedir o canvas aqui obrigaria a shell a espelhar a aritmética de layout que
                // o `paint_hero_screen` já faz — a segunda porta exata que a régua evita.
                let (sw, sh) = ph2d_app_motion::field_gizmo::scene_window_wh(
                    hero.view.center_split,
                    window_size,
                );
                ph2d_vec_render::draw_document_guides(
                    &doc_guides.iter().copied().collect::<Vec<_>>(),
                    [0.0, 0.0, f64::from(sw), f64::from(sh)],
                    cam_affine,
                    vector_scene,
                );
                ph2d_vec_render::draw_snap_guides(&self.vec.snap_guides, cam_affine, vector_scene);
                // **O NÚMERO da guia** — depois do traço dela, porque a cena tem de estar
                // livre para o renderizador de texto (a mesma ordem do readout de joint).
                // O zoom sai do MESMO afim que acabou de desenhar o segmento: a precisão
                // que a ficha mostra é a que aquele desenho de fato resolve.
                let c = cam_affine.as_coeffs();
                let px_per_world = (c[0] * c[0] + c[1] * c[1]).sqrt();
                super::render_loop::vec_snap_labels::draw(
                    &self.vec.snap_guides,
                    cam_affine,
                    px_per_world,
                    ph2d_editor_core::LengthDisplay::of(&hero.project),
                    hero.theme,
                    paint_ctx.text,
                    vector_scene,
                );
            }
            // **A DECORAÇÃO DA FOLHA** (Enio 2026-08-19) — a faixa hachurada e o nome. Fica FORA
            // do bloco das guias de propósito: aquele é gateado pelo toggle de snap, e a folha
            // tem de se ler como folha esteja o encaixe ligado ou não.
            {
                let c = cam_affine.as_coeffs();
                let px_per_world = (c[0] * c[0] + c[1] * c[1]).sqrt();
                super::render_loop::sheet_overlay::draw(
                    sim,
                    cam_affine,
                    px_per_world,
                    hero.theme,
                    paint_ctx.text,
                    vector_scene,
                );
                // **AS LINHAS DA GRELHA** sobre a folha aberta de uma sprite (Enio, 2026-08-23).
                // ⚠️ Vizinha da decoração da folha-OBJETO de propósito: as duas dizem *«isto está
                // cortado assim»*, uma para a folha empacotada e outra para a grelha de um sprite,
                // e ler as duas seguidas é o que impede a próxima de nascer num terceiro sítio.
                if let Some(e) = super::render_loop::sim_extract_sheet::previewed(hero) {
                    super::render_loop::sheet_grid_overlay::draw(
                        sim,
                        e,
                        // ⚠️ **As linhas seguem o quad que foi de facto DESENHADO.** Sob pintura o
                        // quad desdobra-se e centra-se no pivô; fora dela ele é uma célula e a
                        // folha dispõe-se à volta dela. Desenhar sempre a segunda disposição
                        // deslocava as linhas **meia célula** sobre a arte pintada (report do
                        // Enio, 2026-08-23, com foto).
                        super::render_loop::sim_extract_sheet::is_tool_previewed(
                            &tool_preview_bits,
                            e,
                        ),
                        hero.project.pixels_per_meter,
                        cam_affine,
                        px_per_world,
                        hero.theme,
                        vector_scene,
                    );
                }
                // **A LEGENDA DA CENA DE SMOKE** (Enio 2026-08-23: *"melhore as explicações do
                // smoke"*) — o rótulo pousa em cima do caso que ele explica. No-op quando
                // nenhuma cena publicou, que é todo arranque normal do editor.
                super::render_loop::demo_legend::draw(
                    &ph2d_app_motion::motion_demo_legend::captions(),
                    cam_affine,
                    hero.theme,
                    paint_ctx.text,
                    vector_scene,
                );
            }
            // **As linhas da SIMETRIA** — *"quando ligada linhas aparecem no canvas"* (Enio).
            //
            // ⚠️ FORA do `overlay.snap_guides` de propósito: aquele toggle é do encaixe, e a
            // simetria não é um encaixe — é o eixo do que está a ser desenhado. Escondê-la com o
            // snap deixaria o artista com as cópias na tela e sem o eixo que as produz.
            {
                // Duas linhas, dois FATOS. A de SESSÃO diz *onde o próximo desenho vai espelhar* e
                // fica onde foi semeada; a de cada forma diz *onde AQUELA forma espelha* e viaja
                // com ela. Coincidem até o artista mover o desenho — e é exatamente aí que ver as
                // duas passa a valer, porque a promessa de que a linha acompanha o objeto só é
                // legível contra a que não acompanha.
                let mut axes = if self.vec.draw_config.symmetry.on {
                    crate::symmetry_live::live_axes(vec_scene, sim, &self.vec.entities, &vec_xf)
                } else {
                    Vec::new()
                };
                if let Some(origin) = self.vec.symmetry_origin {
                    axes.push(crate::symmetry_live::session_axis(
                        self.vec.draw_config.symmetry,
                        origin,
                    ));
                }
                if !axes.is_empty() {
                    let (sw, sh) = ph2d_app_motion::field_gizmo::scene_window_wh(
                        hero.view.center_split,
                        window_size,
                    );
                    ph2d_vec_render::draw_symmetry_axes(
                        &axes,
                        [0.0, 0.0, f64::from(sw), f64::from(sh)],
                        cam_affine,
                        vector_scene,
                    );
                }
            }
            // **O realce do Shape Builder** — as faces sob o cursor e as já pintadas. Fora do
            // `overlay.edit` porque o Build não é um modo de edição de nó: o que ele
            // manipula é a REGIÃO, não a âncora.
            if let Some(b) = self.vec.build.as_mut() {
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
            if vector_active && self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Select {
                let handles = crate::connector_handles::view(
                    sim,
                    vec_scene,
                    &self.vec.entities,
                    self.vec.pen.selected_paths(),
                );
                ph2d_vec_render::draw_connector_handles(&handles, cam_affine, vector_scene);
                // Os pontos de passagem — QUADRADOS, por cima das bolinhas: quando um waypoint
                // é arrastado até uma ponta, é ele que está sob o dedo.
                let ways = crate::connector_handles::waypoint_view(
                    sim,
                    &self.vec.entities,
                    self.vec.pen.selected_paths(),
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
                    &self.vec.entities,
                    self.vec.pen.selected_paths(),
                )
            {
                ph2d_vec_render::draw_text_handle(
                    at,
                    self.vec.textpath_handle_drag,
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
                    &self.vec.entities,
                    self.vec.pen.selected_paths(),
                )
            {
                use crate::pattern_live::PatternHandle;
                let dragging = self.vec.patternpath_handle;
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
                && let Some(pid) = self.vec.pen.selected()
            {
                let grabbed = self.vec.width_grab.map(|g| g.stop);
                for (k, h) in crate::width_handles::handles(sim, vec_scene, &self.vec.entities, pid)
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
            if let Some((a, b)) = crate::vec_text::caret_of(self.vec.text_edit.as_ref()) {
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
            let flip_pairs_active =
                self.flip_state.active && self.flip_state.strip.tween_correct.is_some();
            let suppress_gizmo = painter_deform_transform
                || flip_pairs_active
                || tools
                    .active()
                    .map(|t| {
                        let id = t.id();
                        id == ph2d_editor_core::ToolId::new("vector_direct")
                            || id == ph2d_editor_core::ToolId::new("vector_pen")
                            || id == ph2d_editor_core::ToolId::new("vector_pencil")
                            || id == ph2d_editor_core::ToolId::new("vector_shape")
                            // Motion Nodes: a tool Motion é dona do canvas (o único gizmo é o
                            // do field, slot próprio `field_view`). Um sprite selecionado por
                            // acaso ao entrar mostraria seu gizmo de sprite projetado na
                            // janela CHEIA (deslocado da cena que renderiza na banda do split)
                            // — some junto com o resto do chrome de sprite.
                            || id == ph2d_editor_core::ToolId::new("motion")
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
            // ADR-0161 — o smoke do módulo de modelagem 3D (`PH2D_FIELD_SMOKE=1..3`).
            //
            // ⚠️ **ANTES do `paint_hero_screen`, e é a correção de um smoke do Enio (19/08):**
            // *"o fundo está cinza escuro e ACIMA do canvas"*. Depois da chrome, o traçado é uma
            // sobreposição que tapa o app; antes dela, é **conteúdo de canvas**, com painéis e
            // chrome por cima — que é o que um visualizador 3D tem de ser.
            //
            // O equivalente estrutural é o `sculpt3d`, que entra como passe de GPU no `present.rs`
            // com `LoadOp::Load`. Aqui o traçado é de CPU, então o análogo honesto é a ordem de
            // pintura. Um canvas 3D de primeira classe (modo próprio, não `env`) é item de wave.
            //
            // No-op silencioso sem a variável; todo o estado vive no próprio módulo, de propósito
            // (`field3d_smoke`, §"Estado contido").
            // ⭐⭐ **A PARTE LIVRE DA ÁREA** (W50) — Enio, no smoke da W49: *"fica escondido entre
            // botões […] quando houver painel à direita melhor deslocar o gizmo para esquerda e
            // abaixar um pouco"*.
            //
            // ⚠️ A área que o módulo recebe é o **viewport inteiro**, e a moldura do app é pintada
            // por cima dele. Os retângulos vêm de quem os conhece — o `panel_rect` do store (só
            // publicado enquanto o painel está aberto) e o índice de acerto da faixa do topo (só
            // escrito no quadro em que ela de facto pintou). A **lei** de como eles empurram o
            // gizmo é pura e vive no módulo (`ph2d_viewport3d::navball::safe_corner`).
            {
                let mut obstacles: Vec<ph2d_editor_core::zones::Rect> = Vec::new();
                for id in crate::forwarding::CHROME_BACKDROPS {
                    if let Some(r) = hero.hit_index.rect_for(id) {
                        obstacles.push(r);
                    }
                }
                // ⚠️ **Todos** os painéis publicados neste quadro, sem lista de ids: uma segunda
                // cópia da lista que o `cursor_over_hero_panel` já carrega seria uma lista a mais
                // para alguém esquecer. Um painel flutuante no meio do canvas não move o gizmo — a
                // lei só conta quem toca a **aresta** da área.
                obstacles.extend(hero.store.panel_rects());
                // ⭐⭐ **A ÁREA é a de DESENHO, não a janela** (2026-08-30). Ela era o viewport
                // inteiro, e por isso as colunas docadas tocavam-lhe a aresta e **empurravam** o
                // gizmo — o remédio do sintoma que a D1 manda retirar quando os painéis passam a
                // ser regiões irmãs. Com a área certa, uma coluna docada deixa de a alcançar e a
                // fuga fica **inerte por construção**, sem uma linha de lei mudar.
                //
                // ⚠️ **A fuga FICA**, e não por preguiça: o que ainda a alcança são as janelas que
                // declaram flutuar (Grid Snap, galeria) — e a lei dela já diz que só conta quem
                // toca a **aresta**. Apagá-la deixaria o gizmo por baixo de uma dessas.
                //
                // ⚠️ `last_canvas` **é** a `HeroLayout::draw_area` publicada pelo quadro anterior
                // (ver `screens/hero/paint.rs`); no primeiro quadro ela é degenerada, e aí vale a
                // janela — que é o comportamento de sempre.
                let area = ph2d_viewport3d::layout::area(
                    hero,
                    ph2d_editor_core::zones::Rect::new(
                        viewport.x, viewport.y, viewport.w, viewport.h,
                    ),
                );
                let safe = ph2d_viewport3d::navball::safe_corner(area, &obstacles);
                ph2d_app_field3d::smoke::note_safe(safe);
                // ⭐⭐⭐ **E A ESCULTURA LÊ O MESMO PAR** (2026-09-08, ordem do Enio: *«traga esses
                // features para esse módulo»*). ⚠️ **Calculado UMA vez e publicado nos dois**, e não
                // duas vezes com a mesma receita: a lista de obstáculos deste quadro é a coisa cara
                // e é a que envelhece — dois censos dela divergiriam no quadro em que um painel
                // abre. *A área do canvas 3D é uma pergunta só; ter dois donos é ter duas
                // respostas.*
                #[cfg(feature = "sculpt3d")]
                if let Some(scene) = sculpt3d.as_mut() {
                    // ⚠️ **A ÁREA primeiro, os gizmos depois** — eles moram no
                    // quadrante ACTIVO, e quem sabe onde ele está é a divisão,
                    // que acaba de ser publicada.
                    scene.note_canvas(area);
                    scene.note_nav(safe, self.last_pointer);
                    scene.note_gizmo_hot(self.last_pointer);
                }
            }
            // ⭐⭐ **A VIAGEM ENTRE VISTAS** (W51) — Enio: *"falta um Lerp() rápido para mudança
            // suave das views como no blender"*.
            //
            // ⚠️ **A curva e a duração são as da CASA**, não minhas: `Role::Surface` é o papel cujo
            // doc descreve este caso à letra — *"viaja (o reduced motion mata-a) e **nunca
            // ultrapassa**… uma roda nomeia um DESTINO, e passar dele e voltar lê como a régua a
            // mentir"*. Uma vista nomeada é um destino, e com `Role::Travel` a peça passaria da
            // frente e voltava, com a janela inteira a balançar.
            //
            // ⚠️ **E ela NÃO morre no *reduced motion*** (W52, decisão do Enio): o papel é o
            // `Viewpoint`, e o critério dele é que *o que substitui esta animação é um CORTE que
            // desorienta mais do que ela*. Ver `ph2d_app_field3d::flight::ROLE`.
            if let Some((generation, fresh)) = ph2d_app_field3d::smoke::flight_track() {
                let id = ph2d_editor_core::screens::hero::ids::model3d_view_travel(generation);
                if fresh {
                    // Semear em 0: a primeira vez que um id é visto, o `animate` **chega** ao alvo
                    // (um widget que acaba de aparecer não tem de onde vir). Sem esta linha a
                    // viagem estaria terminada antes de começar.
                    hero.motion.animate(id, 0.0, ph2d_app_field3d::flight::ROLE);
                }
                let t = hero.motion.animate(id, 1.0, ph2d_app_field3d::flight::ROLE);
                ph2d_app_field3d::smoke::note_flight_progress(t);
            }
            // ⭐⭐⭐ **A ÁREA, e não a JANELA** (report do Enio, 2026-08-31: *«a viewport ainda não
            // se encaixa na área correta para ela — veja que atravessa as réguas»*).
            //
            // ⛔⛔ Isto era `viewport` cru, então os quadrantes ladrilhavam o ecrã inteiro: por
            // baixo da barra de menus, da fila de ferramentas, da coluna da esquerda e das duas
            // réguas — e com a divisão aberta as costuras caíam onde não há área nenhuma.
            //
            // ⚠️ **A MESMA porta que alimenta o gizmo de navegação** (`ph2d_viewport3d::layout::area`), e é
            // por isso que os dois se encaixam: um segundo rect aqui seria a fonte por onde a
            // imagem e a moldura voltavam a discordar.
            ph2d_app_field3d::smoke::draw(
                ph2d_viewport3d::layout::area(
                    hero,
                    ph2d_editor_core::zones::Rect::new(
                        viewport.x, viewport.y, viewport.w, viewport.h,
                    ),
                ),
                hero.theme,
                paint_ctx.text,
                vector_scene,
            );
            // ⭐⭐⭐ **O GIZMO DA VIEWPORT DA ESCULTURA** (2026-09-08) — as seis bolas de eixo, a
            // MESMA lei e o MESMO pintor do módulo vizinho (`ph2d_viewport3d::navball`), com a base desta
            // câmera. Ver `sculpt3d_navball`.
            //
            // ⚠️ **Pintado aqui e não no bloco do anel do pincel**, que corre ~3 000 linhas acima:
            // é aqui que o par `área`/`safe` deste quadro já foi publicado, e desenhá-lo antes
            // usaria o do quadro anterior — meio widget deslocado sempre que uma coluna abrisse.
            //
            // ⚠️ **Sem barro na tela ele não é pintado**: o módulo está desarmado, e um gizmo de
            // navegação sobre uma cena 2D prometeria um gesto que não existe ali.
            #[cfg(feature = "sculpt3d")]
            if let Some(scene) = sculpt3d.as_mut()
                && scene.clay_on_screen()
            {
                // ⭐⭐⭐ **O RÓTULO DE CADA VISTA** — só com a divisão aberta. Com uma vista só a
                // pergunta *«qual é qual?»* não existe, e um rótulo permanente seria ruído sobre a
                // peça. ⚠️ A chave sai da CÂMERA daquele quadrante, nunca do sítio dele: orbitar a
                // vista de cima faz dela *User*, que é o que ela passou a ser.
                let quadros = scene.vp_rects();
                // ⚠️ **O chip de cada rótulo é PUBLICADO por quem o pinta** — a largura dele é a do
                // TEXTO, e só o pintor a mede. É ele o alvo do clique que abre o menu daquela
                // vista; com **uma** vista não há rótulo e a lista fica vazia, o que faz o chip ser
                // inalcançável em vez de invisível-mas-clicável.
                let chips: Vec<_> = if quadros.len() > 1 {
                    quadros
                        .iter()
                        .enumerate()
                        .map(|(i, r)| {
                            ph2d_app_field3d::gizmo_paint::paint_view_label(
                                vector_scene,
                                paint_ctx.text,
                                *r,
                                scene.vp_label_key(i),
                                hero.theme,
                            )
                        })
                        .collect()
                } else {
                    Vec::new()
                };
                scene.note_view_labels(chips);
                // ⭐⭐ **AS COSTURAS E A MOLDURA DO ACTIVO** — o MESMO pintor do módulo vizinho.
                // Ele já é no-op com uma vista só.
                ph2d_app_field3d::gizmo_paint::paint_split(
                    vector_scene,
                    &quadros,
                    scene.vp_active(),
                    hero.theme,
                );
                // ⭐⭐⭐ **O GIZMO DE TRANSFORMAÇÃO** (2026-09-08) — as alças por cima da peça e no
                // referencial da JANELA (a `SculptCam` já lhes soma a quina do quadrante activo).
                //
                // ⚠️ **Sem teste de profundidade, e é o que todo modelador faz**: uma alça
                // escondida atrás da superfície que ela move seria inalcançável exactamente quando
                // o artista precisa dela.
                //
                // ⚠️ **Vazio sem transform armado** — a lista sai vazia da porta, e o pintor é
                // no-op sobre ela. *Um `if` aqui seria a segunda resposta a «há gizmo agora?».*
                {
                    let handles = scene.gizmo_handles();
                    if !handles.is_empty() {
                        ph2d_app_field3d::gizmo_paint::paint(
                            vector_scene,
                            &handles,
                            scene.gizmo_hot(),
                            hero.theme,
                            // ⚠️ **Origem ZERO**: ao contrário do módulo vizinho, estas alças já
                            // chegam em coordenadas de janela. Somar a quina aqui seria somá-la
                            // duas vezes, e o gizmo sairia ao dobro da distância da peça.
                            [0.0, 0.0],
                        );
                    }
                }
                // ⭐⭐ **O GIZMO DA VIEWPORT**, por cima da moldura — que é onde um gizmo de janela
                // vive. Ver `sculpt3d_navball`.
                if let Some((area, safe)) = scene.nav_rects() {
                    let balls = scene.navball(area, safe);
                    ph2d_viewport3d::navball_paint::paint(
                        vector_scene,
                        &balls,
                        scene.nav_hot(),
                        hero.theme,
                        [area.x, area.y],
                        ph2d_viewport3d::navball::centre_in(area, safe),
                    );
                }
                // ⭐⭐⭐ **O MENU DA VISTA, POR CIMA DE TUDO** (report do Enio, 2026-09-08: *«ao
                // clicar nos nomes das views não aparece a lista de view como no módulo de
                // modelagem 3d»*) — ele é modal, e o clique seguinte é dele caia onde cair.
                //
                // ⚠️ **O rectângulo é PUBLICADO por quem o pinta**, como o chip: a largura dele é a
                // da linha mais comprida, e só o pintor a mede. É o mesmo pintor do módulo vizinho.
                if let Some(i) = scene.view_menu_open()
                    && let (Some(chip), Some(canvas)) = (scene.chip_of(i), scene.canvas())
                {
                    let r = ph2d_app_field3d::gizmo_paint::paint_view_menu(
                        vector_scene,
                        paint_ctx.text,
                        chip,
                        canvas,
                        hero.theme,
                    );
                    scene.note_view_menu_rect(r);
                }
            }
            // ADR-0161 W4: o painel de modelagem abre sozinho na primeira vez que o
            // smoke desenha (auto-play), e só nessa — reabri-lo todo quadro faria o
            // botao de fechar dele nao funcionar.
            // ⭐ **O pedido de exportar**, tirado ao lado do de abrir o painel — os dois
            // atravessam da ponte com a cena para o app pela mesma porta, e por isso são
            // consumidos no mesmo sítio.
            if let Some(level) = ph2d_app_field3d::smoke::take_export_request() {
                ph2d_app_field3d::export::field3d_export(level, toasts);
            }
            // ⭐⭐⭐ **E a RESPOSTA da bancada** (`ph2d_app_field3d::export_job`): desde 2026-08-25 a
            // exportação corre fora da thread que desenha, e o que volta é a mensagem pronta.
            // ⚠️ Ela é drenada **aqui**, ao lado do pedido, pela lei das caixas de correio deste
            // módulo — *uma porta, vários pedintes*.
            if let Some(done) = ph2d_app_field3d::export_job::take_finished() {
                toasts.push(ph2d_editor_core::Toast::info(done));
            }
            // ⭐ **E o de IMPORTAR**, pela mesma porta e pelo mesmo motivo (ADR-0161 W22).
            if ph2d_app_field3d::smoke::take_import_request() {
                ph2d_app_field3d::import::field3d_import(toasts);
            }
            // ⭐⭐⭐ **E o de RELIGAR** (W76), pela mesma porta e pelo mesmo motivo: escolher o
            // arquivo é um diálogo, e um diálogo não corre com o mundo emprestado.
            if let Some(e) = ph2d_app_field3d::smoke::take_relink_request() {
                ph2d_app_field3d::import::field3d_relink(e, toasts);
            }
            // ⭐⭐ **O PERFIL DESENHADO VIRA PEÇA** (W53) — o fluxo do MoI, que o motor tem medido e
            // gateado desde a W3 e que **nenhum botão alcançava**.
            //
            // ⚠️ Servido **aqui** porque quem tem a cena vetorial é o `AppGfx`; a ponte com a cena
            // recebe o mundo. É a mesma divisão dos pedidos acima.
            //
            // ⚠️ **E o shell publica se HÁ contorno**, todo quadro: é isso que faz os dois botões
            // aparecerem só quando há o que extrudar (a lei da W34).
            {
                let closed = crate::blend_live::selected_closed_in_z(vec_scene, &self.vec.pen);
                ph2d_app_field3d::smoke::note_profile(closed.first().copied());
                if let Some(which) = ph2d_app_field3d::smoke::take_profile_request() {
                    let msg = ph2d_app_field3d::profile::from_selection(vec_scene, &closed, which);
                    toasts.push(ph2d_editor_core::Toast::info(msg));
                }
            }
            // ⭐ **E a escultura da CENA** (W39) — o vínculo que não passa pelo disco.
            //
            // ⚠️ Ela é servida **aqui** e não na ponte com a cena porque quem tem a escultura viva é
            // o `AppGfx`; a ponte recebe o mundo. É a mesma divisão dos dois pedidos acima.
            //
            // ⚠️ **E o shell publica se ela EXISTE**, todo quadro: é isso que faz o botão aparecer
            // só quando há o que trazer (a lei da W34). Sem a feature, fica sempre falso.
            #[cfg(feature = "sculpt3d")]
            {
                let live = sculpt3d
                    .as_ref()
                    .map(ph2d_app_sculpt3d::Sculpt3dScene::mesh);
                ph2d_app_field3d::smoke::note_live_sculpt(live.is_some());
                if ph2d_app_field3d::smoke::take_scene_sculpt_request() {
                    let msg = live.map_or_else(
                        || "There is no sculpture in the scene to bring in".to_string(),
                        |m| ph2d_app_field3d::import::field3d_scene_sculpt(m.clone()),
                    );
                    toasts.push(ph2d_editor_core::Toast::info(msg));
                }
            }
            // ⭐⭐⭐ **A PALETA DE FORMAS, as DUAS pontas** (W100) — abrir para quem pediu, e mandar
            // ao mundo o que ela escolheu. Irmã por assunto do `component_attach`, que faz o mesmo
            // com o `+` do Inspector.
            //
            // ⚠️ **Aberta DEPOIS dos dois `note_*` acima**, e a ordem é load-bearing: o modelo dela
            // carrega a disponibilidade de *Extrude*/*Revolve*/*Sculpt from scene*, e construí-lo
            // antes das notas deste quadro mostraria a resposta do quadro anterior — visível
            // exatamente no gesto que importa (escolher o contorno e abrir a paleta a seguir).
            if ph2d_app_field3d::smoke::take_shape_palette_request() {
                let (live_sculpt, profile) = ph2d_app_field3d::smoke::palette_conditions();
                hero.store
                    .open_command_palette(ph2d_app_field3d::shape_palette::build(
                        live_sculpt,
                        profile,
                    ));
            }
            // ⚠️ O pick chega **noutro quadro** (a paleta fica aberta), e o dreno é **CONDICIONAL**:
            // este canal já tinha TRÊS consumidores (a biblioteca do Motion, o `Ctrl+K` e o `+` do
            // Inspector), e um `take` incondicional engoliria o pick de outro — com o sintoma a ser
            // *«às vezes não faz nada»*.
            if let Some(id) = hero.store.take_command_pick_if(|id| {
                ph2d_app_field3d::shape_palette::slot_of_pick(id).is_some()
            }) && let Some(slot) = ph2d_app_field3d::shape_palette::slot_of_pick(id)
            {
                ph2d_app_field3d::smoke::ask_shape(slot);
            }
            // ⭐ **E o que o módulo tem a DIZER** (W23 + W25): a escultura que não voltou do
            // arquivo, e a peça que não cozinha. A ponte com a cena descobre as duas ao cozer o
            // documento, e a fila de avisos é daqui. Sem esta linha as duas falham em silêncio — a
            // peça some da tela e nada explica porquê.
            for msg in ph2d_app_field3d::notice::drain() {
                toasts.push(ph2d_editor_core::Toast::info(msg));
            }
            if ph2d_app_field3d::smoke::take_open_panel_request() {
                // O ID vem do PAINEL, nunca de um literal: uma segunda cópia da chave de
                // visibilidade é como se abre um painel que ninguém pinta.
                hero.panel_visibility
                    .insert(ph2d_panel_model3d::PANEL_ID, true);
            }
            // Frame profiler: panel/chrome Vello encode (includes the painter panel's Paper preview).
            let hero_t0 = frame_prof_on().then(Instant::now);
            paint_hero_screen(hero, viewport, vector_scene, paint_ctx.text);
            // ⭐⭐ **A ARRUMAÇÃO é detectada no QUADRO, não no hook de ponteiro** (decisão D4).
            //
            // ⛔⛔ Ela viveu no `forward_to_hero` durante uma entrega, com os outros dois
            // inquilinos da persistência — e **não funcionava para a largura da coluna**: o
            // arrasto da borda faz `return` no Move E no Up (`input_dispatch`), então nunca
            // alcançava o detector. O mesmo valia para a largada de uma aba, que é resolvida
            // DENTRO do `paint`. *Um detector no caminho de um gesto só vê os gestos que passam
            // por ele; o quadro vê todos, porque é onde o estado assenta.*
            crate::layout_persist::save_if_changed(hero);
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
                    ph2d_editor_core::zones::Rect::new(
                        viewport.x, viewport.y, viewport.w, viewport.h,
                    ),
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
            // **A TRAVA DO PAINTER, na porta da HIERARQUIA** (Enio, 2026-08-19). Enquanto o
            // Painter tem um documento aberto, clicar noutra linha não troca a sprite debaixo do
            // pincel: recusa, e o aviso diz por onde sair.
            //
            // ⚠️ A intenção é **consumida** (posta a `None`), não saltada: deixá-la viva faria a
            // mesma recusa repetir-se no quadro seguinte, e o artista veria o aviso a piscar.
            if let Some(intent) = hierarchy_select_intent {
                let locked = ph2d_app_painter::painter_lock::locked_entity(tools, hero);
                let (target, additive) = match intent {
                    hierarchy::HierarchySelectIntent::Row { row, modifier } => (
                        hero_live.as_ref().and_then(|l| l.bridge.entity_for(row)),
                        !matches!(
                            modifier,
                            ph2d_editor_core::action_bus::SelectModifier::Replace
                        ),
                    ),
                    // Um intervalo é aditivo por definição.
                    hierarchy::HierarchySelectIntent::Range { .. } => (None, true),
                };
                if ph2d_app_painter::painter_lock::decide(locked, target, additive)
                    == ph2d_app_painter::painter_lock::Decision::Refuse
                {
                    toasts.push(Toast::warning(ph2d_app_painter::painter_lock::REFUSAL));
                    hierarchy_select_intent = None;
                    self.title_dirty = true;
                }
            }
            // ⭐⭐⭐ **A troca de VARIANTE, aplicada** (ADR-0164 / F5, critério 2).
            //
            // ⚠️ **Aqui, e não no dreno do Inspector**, porque é aqui que o `sim` e o **eco** estão
            // os dois à mão — a troca tem de o esquecer, senão o passe seguinte lê a diferença
            // contra o mestre NOVO como *«a instância mexeu-se»* e congela a cópia com o valor do
            // mestre VELHO ([`ph2d_app_components::instance_variant::swap`]).
            // ⭐⭐⭐ **RENOMEAR O VALOR de uma propriedade** (report do Enio, 2026-08-31).
            //
            // ⚠️ **O sujeito é a RECEITA, e o gesto nasceu sobre a CÓPIA.** É a razão de existir:
            // autorar o valor obrigava a seleccionar outro objecto do que aquele que se está a
            // olhar — e ele tentou pelo nome da cópia quatro vezes, com o modelo a ignorá-lo
            // correctamente as quatro.
            //
            if open_asset_browser {
                <_ as ph2d_editor_core::panel::PanelHostInternal>::set_panel_visible(
                    hero,
                    ph2d_panel_asset_browser::PANEL_ID,
                    true,
                );
            }
            // ⭐⭐⭐ **APLICAR num degrau da escada** (ADR-0164 / F5, critério 4).
            //
            // ⚠️ **O toast diz a QUE receita** — os degraus ficam um debaixo do outro no cartão e a
            // diferença entre eles só se vê no gesto SEGUINTE (mexer noutra cópia daquele mestre).
            // Uma confirmação igual para os dois deixaria o artista sem saber qual carregou.
            if let Some((entity_bits, master)) = apply_to_level {
                let name = inspector_instance::master_named(sim, master)
                    .unwrap_or_else(|| "prefab".to_string());
                match ph2d_app_components::instance_apply_deep::apply_to_level(
                    sim,
                    component_registry,
                    &mut self.instance_echo,
                    ph2d_ecs::Entity::from_bits(entity_bits),
                    master,
                    &mut ph2d_app_components::instance_docs::OwnedDocs {
                        vec_scene,
                        vec_entities: &mut self.vec.entities,
                    },
                ) {
                    Ok(done) if done.changed == 0 && done.left == 0 => {
                        toasts.push(Toast::info("Nothing overridden here"));
                    }
                    Ok(done) => {
                        // ⚠️ **O que ficou por aplicar é DITO** — uma excepção cuja escada não
                        // alcança aquela receita fica onde está, e um número que desaparece em
                        // silêncio lê-se como trabalho perdido.
                        toasts.push(Toast::success(if done.left > 0 {
                            format!(
                                "Applied {} change(s) to \u{201c}{name}\u{201d} \u{2014} {} left (not part of it)",
                                done.changed, done.left
                            )
                        } else {
                            format!("Applied {} change(s) to \u{201c}{name}\u{201d}", done.changed)
                        }));
                        self.title_dirty = true;
                    }
                    Err(_) => {
                        toasts.push(Toast::warning("Not part of an instance"));
                    }
                }
            }
            // ⭐⭐⭐ **APLICAR uma peça acrescentada** (F5.11) — ela entra na receita e o passe
            // estrutural leva-a às irmãs no quadro seguinte.
            //
            // ⚠️ **O sujeito resolve-se por `StableId`**, e não pelos bits que o cartão viu: entre
            // o clique e este ponto pode ter corrido um Ctrl+Z, que respawna tudo com bits novos.
            if let Some(piece) = apply_added {
                let mut docs = ph2d_app_components::instance_docs::OwnedDocs {
                    vec_scene,
                    vec_entities: &mut self.vec.entities,
                };
                let subject = ph2d_app_components::instance_verbs::entity_for_stable_id(sim, piece)
                    .map(ph2d_ecs::Entity::from_bits);
                match subject
                    .ok_or(ph2d_app_components::instance_added::AddRefusal::NotAdded)
                    .and_then(|e| {
                        ph2d_app_components::instance_added::promote(
                            sim,
                            component_registry,
                            &mut docs,
                            e,
                        )
                    }) {
                    Ok(p) => {
                        let name =
                            crate::render_loop::inspector_instance::master_named(sim, p.master)
                                .unwrap_or_else(|| "the prefab".to_string());
                        toasts.push(Toast::success(format!(
                            "Added {} piece(s) to \u{201c}{name}\u{201d} \u{2014} every copy gets them",
                            p.pieces
                        )));
                        self.title_dirty = true;
                    }
                    // ⚠️ **Todo caminho negativo fala** — a lei do menu dos verbos. Um botão que
                    // come o clique em silêncio é pior que um ausente.
                    Err(ph2d_app_components::instance_added::AddRefusal::NotAdded) => {
                        toasts.push(Toast::warning(
                            "That piece came from the component \u{2014} it is already in it",
                        ));
                    }
                    Err(_) => {
                        toasts.push(Toast::warning("That is not a piece of a copy"));
                    }
                }
            }
            if let Some((root_bits, master)) = swap_variant {
                match ph2d_app_components::instance_variant::swap(
                    sim,
                    &mut self.instance_echo,
                    ph2d_ecs::Entity::from_bits(root_bits),
                    master,
                    // ⚠️ **A fileira de versões nunca adivinha.** Ali os mestres são aparentados
                    // por construção; uma queda para heurística seria a operação automática que o
                    // plano F5 proíbe. Quem pede um dos três modos é o menu da biblioteca.
                    ph2d_app_components::instance_swap_match::WhenUnrelated::Refuse,
                ) {
                    Ok(r) => {
                        toasts.push(Toast::success(if r.dropped > 0 {
                            format!(
                                "Switched variant \u{2014} {} override(s) kept, {} piece(s) unused",
                                r.overrides_kept, r.dropped
                            )
                        } else {
                            format!(
                                "Switched variant \u{2014} {} override(s) kept",
                                r.overrides_kept
                            )
                        }));
                        self.title_dirty = true;
                    }
                    // ⚠️ **Todo caminho negativo fala.** Um chip que come o clique em silêncio é
                    // pior que um ausente — a mesma lei que o menu dos verbos paga.
                    Err(ph2d_app_components::instance_variant::SwapRefusal::Already) => {}
                    Err(ph2d_app_components::instance_variant::SwapRefusal::Unrelated) => {
                        toasts.push(Toast::warning(
                            "These components are not related \u{2014} switching would lose every override",
                        ));
                    }
                    Err(_) => {
                        toasts.push(Toast::warning("That is not a copy of a prefab"));
                    }
                }
            }
            // ⭐⭐ **Os verbos de catálogo** (wave A3). ⚠️ Eles correm ANTES do `hierarchy::dispatch`
            // de propósito: a taxonomia não é o mundo, e misturá-los no mesmo bloco daria a
            // impressão de que um catálogo é um objecto da cena.
            for v in &catalog_verbs {
                if crate::asset_catalog_verbs::drain(v, asset_catalogs, toasts) {
                    self.title_dirty = true;
                }
            }
            if hierarchy::dispatch(
                view_focus_kind,
                visibility_toggle_row,
                lock_toggle_row,
                group_toggle_row,
                reparent_intent,
                duplicate_row,
                add_child_row,
                add_root,
                reset_transform_row,
                revert_to_master_row,
                instance_verb_row,
                instance_verb_stable_id,
                asset_card_verb,
                atlas_asset_map,
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
                vec_scene,
                &mut self.vec.entities,
                &mut self.vec.pen,
                &mut duplicate_made,
                component_registry,
                &mut self.instance_echo,
            ) {
                self.title_dirty = true;
            }
            // ⭐⭐⭐ **O *Duplicate* de uma PEÇA da escultura** (ADR-0150, 2026-09-04): a cópia
            // profunda **deixa cair** o `Sculpt3dPieceRef` — copiar o id daria duas entidades
            // sobre a mesma peça (`instance_docs::DROPPED`) —, então a linha nova nasceria vazia.
            // ⇒ regista-se um PEDIDO, e o `sculpt3d::entities::sync` do quadro seguinte duplica a
            // peça e põe o `ref` novo na cópia. *A cena está emprestada neste ponto do laço.*
            #[cfg(feature = "sculpt3d")]
            if let Some((src_bits, new_bits)) = duplicate_made
                && let Some(piece) = sim
                    .world()
                    .get::<ph2d_ecs::Sculpt3dPieceRef>(ph2d_ecs::Entity::from_bits(src_bits))
            {
                self.sculpt3d.dup = Some((piece.0, new_bits));
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
                    toasts,
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
                        asset_db,
                        &read.image,
                        read.old_size_world,
                        toasts,
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
                &visibility_edits,
                name_edit,
                signal_edit,
                signal_leave_edit,
                &sprite_edits,
                &ordering_edits,
                &sampling_edits,
                &blend_edits,
                &slice_edits,
                &anchor_edits,
                &anim_edits,
                &timer_edits,
                &action_edits,
                &physics_edits,
                &visibility_section_edits,
                hero,
                sim,
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
            // ⭐⭐⭐ **A secção AUDIO** (TOP-20 #4, W3) — e ela corre AQUI, e não no
            // `inspector_commits`, porque as edições dela são de DUAS naturezas: a maioria escreve
            // um campo do documento, e três (`Preview`, `Stop`, `Browse`) tocam no DISPOSITIVO ou
            // abrem um diálogo. ⚠️ O `inspector_commits` não tem — nem devia ter — a placa de som
            // nem a janela. *Duas naturezas, dois sítios; a fronteira é o que cada edição TOCA.*
            for (bits, edit) in &audio_edits {
                match edit {
                    ph2d_editor_core::AudioFieldEdit::Preview => {
                        let e = ph2d_ecs::Entity::from_bits(*bits);
                        audio_2d::play_target(sim, self.audio.as_mut(), e);
                    }
                    ph2d_editor_core::AudioFieldEdit::StopPreview => {
                        let e = ph2d_ecs::Entity::from_bits(*bits);
                        audio_2d::stop_target(sim, self.audio.as_mut(), e);
                    }
                    ph2d_editor_core::AudioFieldEdit::Browse => {
                        // ⚠️ **A lista de extensões é a MESMA do resto do app** (`decode_any`), e
                        // não uma escrita à mão: uma segunda lista ao lado de um predicado é o
                        // defeito que o diálogo de importação já pagou — o `.ase` esteve invisível
                        // lá durante meses.
                        if let Some(p) = rfd::FileDialog::new()
                            .add_filter("audio", ph2d_audio_decode::decode_any::AUDIO_IMPORT_EXTS)
                            .pick_file()
                        {
                            let edit = ph2d_editor_core::AudioFieldEdit::Sound(
                                p.to_string_lossy().into_owned(),
                            );
                            if inspector_audio::apply_audio_edit(
                                sim,
                                *bits,
                                &edit,
                                editor_queue,
                                component_registry,
                            )
                            .is_none()
                            {
                                inspector_queue_dirty = true;
                            }
                        }
                    }
                    _ => {
                        if let Some(t) = inspector_audio::apply_audio_edit(
                            sim,
                            *bits,
                            edit,
                            editor_queue,
                            component_registry,
                        ) {
                            toasts.push(t);
                        } else {
                            inspector_queue_dirty = true;
                        }
                    }
                }
            }
            // ⭐⭐⭐ **A secção CAMERA** (TOP-20 #7, W3) — aqui pela MESMA razão da irmã de cima:
            // as edições são de DUAS naturezas. O `Preview` liga a VISTA (que só a `App` tem) e as
            // restantes escrevem um campo do documento. *Duas naturezas, um sítio que tem as duas.*
            for (bits, edit) in &camera_edits {
                if let ph2d_editor_core::CameraFieldEdit::Preview(on) = edit {
                    self.game_camera_preview = *on;
                    continue;
                }
                inspector_camera::apply_camera_edit(
                    sim,
                    *bits,
                    edit,
                    editor_queue,
                    component_registry,
                );
                inspector_queue_dirty = true;
            }
            if inspector_queue_dirty
                && let Err(e) = ph2d_ecs::scene::apply_editor_commands(
                    sim.world_mut(),
                    editor_queue,
                    component_registry,
                )
            {
                toasts.push(ph2d_editor_core::Toast::error(format!(
                    "Audio commit failed: {e}"
                )));
                self.title_dirty = true;
            }
            // A troca de ESTRATÉGIA de origem sai por uma porta própria (irmã, pelo teto de LOC):
            // ela precisa do `atlas_asset_map` e do `next_import_cell` em modo MUTÁVEL — a volta
            // ao atlas ocupa uma célula nova —, e o `dispatch` acima recebe o mapa por leitura.
            if inspector_strategy::dispatch(
                sprite_source_change,
                hero,
                sim,
                renderer,
                asset_db,
                atlas_asset_map,
                next_import_cell,
                toasts,
                editor_queue,
                component_registry,
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
                // W6: e o mesmo commit numa RODLANA montada precisa do sentinela
                // desarmado — o centro dela também é derivado, e o
                // `sync_mounted_wheels` o reescreveria. Pela MESMA porta do dot de
                // canvas; ⚠️ aqui o sentinela É a resposta certa (uma roldana tem
                // UM eixo, então não há segunda metade a perder como no joint).
                ph2d_physics_ecs::reseat_mounted_axle(sim.world_mut(), e);
            }
            // ⚠️ **E fica DEPOIS do dreno do pivot do joint, pela razão que os dois blocos abaixo
            // já têm escrita:** ele esteve NO MEIO do par `let joint_pivot_commit = …` →
            // `if let Some(…) = joint_pivot_commit`, e o gate
            // `the_position_commit_reseats_the_anchor_through_the_door` reprovou — ele lê os 3000
            // bytes a seguir à captura à procura da porta, e 27 linhas alheias empurraram-na para
            // fora da janela. *A janela é a forma de o gate exigir que a captura e o dreno de uma
            // intenção fiquem à vista um do outro; a cura é tirar o intruso, nunca alargá-la.*
            // ⭐ **O `+` do Inspector, as DUAS pontas** (ADR-0166 / F3) — abrir a paleta para quem
            // pediu, e anexar o que ela escolheu. Irmã por assunto (`component_attach`), como a
            // biblioteca do Motion é irmã do `motion_bridge`.
            ph2d_app_components::component_attach::open_palette_if_asked(
                hero,
                sim,
                component_registry,
                add_component_for,
                component_palette_target,
            );
            // ⭐ **A caixa *Show all*** — o widget vira o estado dele e avisa; quem reconstrói o
            // modelo é quem abriu a paleta (só ele sabe o que «mostrar tudo» quer dizer).
            ph2d_app_components::component_attach::refresh_palette_on_toggle(
                hero,
                sim,
                component_registry,
                *component_palette_target,
            );
            // ⚠️ O pick chega **noutro quadro** (a paleta fica aberta), e por isso o alvo vive no
            // `AppGfx` em vez de num local deste laço.
            let picked =
                ph2d_app_components::component_attach::route_pick(hero, component_palette_target);
            ph2d_app_components::component_attach::attach_picked(
                picked.as_ref(),
                sim,
                component_registry,
                // ⭐ A COMPOSIÇÃO entrega as sementes da família dona (auditoria A1): a família de
                // componentes não conhece a física.
                ph2d_app_physics::physics_seed::COMPONENT_SEEDS,
                toasts,
            );
            // A troca de PRECISÃO sai por uma porta própria (plano `docs/Sprite_projeto/18` W5).
            //
            // ⚠️ **E fica DEPOIS do dreno do pivot do joint pela MESMA razão que o bloco abaixo**,
            // que já a tem escrita: o gate `the_position_commit_reseats_the_anchor_through_the_door`
            // lê os 3000 bytes a seguir à captura do pivot, e um bloco alheio no meio empurra a
            // porta para fora da janela. *A cura é tirar o intruso do meio, não alargar a janela* —
            // e este bloco já esteve lá, e já a reprovou.
            // ⚠️ Ela corre **depois** da troca de estratégia, e a ordem é load-bearing: converter
            // para 16 bits FORÇA `Individual`, então deixá-la correr antes faria um clique em
            // `Atlas` no mesmo quadro desfazer a conversão em silêncio.
            if crate::precision_convert::apply(
                precision_request,
                sim,
                renderer,
                asset_db,
                atlas_asset_map,
                toasts,
            ) {
                self.title_dirty = true;
            }
            // **A emissao autorada** (plano `docs/Sprite_projeto/18` W8). Zero REMOVE o componente:
            // uma sprite que nao emite nao carrega a linha no ficheiro nem uma entrada na varredura
            // do passe, e o quadro volta a ser byte-identico.
            //
            // ⚠️ Escreve o componente DIRETO, e nao pela `EditorCommandQueue`: o undo deste projeto
            // e' por DIFF de snapshot (`App::post_frame_undo`), e o `SpriteEmissive` esta' registado,
            // logo ele entra na captura como qualquer outro componente. A fila serve os caminhos que
            // precisam de aplicar por NOME vindo do painel; aqui o tipo e' conhecido.
            for (bits, intensity) in emissive_edits.drain(..) {
                let entity = ph2d_ecs::Entity::from_bits(bits);
                let em = ph2d_ecs::SpriteEmissive(intensity);
                if let Ok(mut e) = sim.world_mut().get_entity_mut(entity) {
                    // ⚠️ Só escreve numa entidade que TEM `Sprite`: a seleção crua pode conter um
                    // path vetorial ou um joint, e emitir luz é um facto sobre uma sprite. Mesmo
                    // filtro que o fan-out do §11 aplica ao atravessar entidades sem `Collider`.
                    if e.get::<ph2d_render::Sprite>().is_none() {
                        continue;
                    }
                    if em.emits() {
                        e.insert(ph2d_ecs::SpriteEmissive(em.clamped()));
                    } else {
                        e.remove::<ph2d_ecs::SpriteEmissive>();
                    }
                    self.title_dirty = true;
                }
            }
            // ⚠️ **Este bloco fica DEPOIS do dreno do pivot do joint, de propósito.** Ele esteve
            // no meio do par `let joint_pivot_commit = …` → `if let Some(…) = joint_pivot_commit`,
            // e o gate `the_position_commit_reseats_the_anchor_through_the_door` reprovou: ele lê os
            // 3000 bytes a seguir à captura à procura da porta `set_joint_anchor_world`, e um bloco
            // alheio no meio empurra-a para fora da janela. *A cura é tirar o intruso do meio, não
            // alargar a janela* — a janela é a forma de o gate exigir que a captura e o dreno de uma
            // intenção fiquem à vista um do outro.
            // **EMPACOTAR A SELEÇÃO NUMA FOLHA** (plano `docs/Sprite_projeto/17` §7) — o pill
            // `[SHEET]` da fila de Image Tools, que substitui o `PH2D_SHEET_SMOKE`.
            //
            // ⚠️ **UMA chamada para a leva INTEIRA**, e não uma por entidade: empacotar N sprites
            // é um ato só. A folha nasce, eles viram filhos dela e o arranjo automático coloca-os.
            // **EMPACOTAR NUMA FOLHA** — uma porta só: o "Pack into Sheet" do menu de contexto
            // da hierarquia. ⚠️ O pill `[SHEET]` da fila de Image Tools EXISTIU e foi **retirado
            // por decisão do Enio** (2026-08-19), com a crate inteira: aquela fila é por-sprite,
            // e este verbo é da SELEÇÃO — o pill difundia uma ação por entidade e o dreno tinha
            // de as voltar a juntar. O menu resolve a linha clicada e aplica a lei do "Merge
            // Sprites" vizinho: a seleção inteira quando a linha faz parte dela, só ela quando
            // não faz. *Não reconstrua o pill sem ler isto.*
            // **RETIRAR DA FOLHA** (Enio 2026-08-19) — pelo MESMO caminho do arrasto-para-a-raiz
            // da hierarquia, e é isso que o torna barato: o `drain_reparent` já preserva a pose de
            // MUNDO (a peça fica onde está, não salta) e já reatribui o `RootOrder` de todas as
            // raízes. Uma segunda saída escrita à mão seria a que se esquecia do `RootOrder` —
            // e o sintoma disso é a hierarquia a reordenar-se sozinha no save seguinte.
            //
            // ⚠️ O toast de recusa não é decoração: sem ele, clicar "Remove from Sheet" numa
            // sprite que não está em folha nenhuma não faria **nada**, e é assim que um item de
            // menu se lê como partido.
            if let Some(row) = remove_from_sheet_row
                && let Some(live) = hero_live.as_ref()
            {
                let in_sheet = live
                    .bridge
                    .entity_for(row)
                    .map(ph2d_ecs::Entity::from_bits)
                    .and_then(|e| crate::sheet_bounds::sheet_parent(sim, e));
                if in_sheet.is_some() {
                    hero_intents::drain_reparent(
                        ph2d_editor_core::screens::hero::HierReparentIntent {
                            dragged: row,
                            new_parent: None,
                            before: None,
                            after: None,
                        },
                        live,
                        sim,
                        toasts,
                    );
                    toasts.push(Toast::success("Removed from sheet"));
                } else {
                    toasts.push(Toast::warning(
                        "Remove from Sheet: this object is not in a sheet",
                    ));
                }
                self.title_dirty = true;
            }
            let mut sheet_targets: Vec<u64> = Vec::new();
            if let Some(row) = pack_sheet_row
                && let Some(live) = hero_live.as_ref()
                && let Some(anchor) = live.bridge.entity_for(row)
            {
                if hero.gizmo.is_selected(anchor) {
                    sheet_targets.extend(hero.gizmo.iter_selected());
                } else {
                    sheet_targets.push(anchor);
                }
            }
            if !sheet_targets.is_empty() {
                let ppm = hero.project.pixels_per_meter;
                // ⚠️ **Um item, um verbo — e este item CRIA.** Ele fazia as duas coisas conforme o
                // alvo (com sprites criava, com uma folha re-arranjava), e a economia era falsa:
                // um verbo que só se descobre por ter selecionado a coisa certa não está no menu,
                // está escondido nele. O Enio pediu o segundo **pelo nome** (2026-08-19), que é a
                // prova de que ele não o encontrava. Agora recusar aponta para onde ele mora.
                if !crate::sheet_frame::sheets_among(sim, &sheet_targets).is_empty() {
                    toasts.push(Toast::warning(
                        "Pack into Sheet: that is already a sheet - use Auto-Arrange Pieces",
                    ));
                } else {
                    // **CRIAR pergunta primeiro** (Enio 2026-08-19: *"Ao criar uma sheet um modal
                    // com a resolução deve aparecer antes da criação"*). Os alvos ficam
                    // RESERVADOS até o Create — o modal é modal, mas o mundo continua a andar, e
                    // recalcular a seleção no Create leria o que ela for ENTÃO, não o que era
                    // quando ele pediu. *A pergunta e a resposta têm de falar do mesmo conjunto.*
                    match crate::sheet_frame::suggested_size(sim, &sheet_targets, ppm) {
                        Some(px) => {
                            self.pending_sheet_targets = sheet_targets.clone();
                            hero.store.open_sheet_size_dialog(px);
                        }
                        // Sem peça nenhuma não há o que perguntar: um modal a pedir a resolução de
                        // uma folha vazia é a caixa de diálogo que não devia ter aberto.
                        None => {
                            toasts.push(Toast::warning("Sheet: select at least one sprite first"));
                        }
                    }
                }
                self.title_dirty = true;
            }
            // **ARRUMAR AS PEÇAS AUTOMATICAMENTE** — o item próprio (Enio 2026-08-19: *"uma opção
            // no menu do botão direito da sheet: arrumar as sprites filhas automaticamente"*).
            //
            // ⚠️ Ele não pergunta resolução nenhuma: ela foi escolhida quando a folha nasceu, e o
            // gesto aqui é *arrume*, não *redimensione*. O que não couber acende a moldura.
            //
            // ⚠️ E aceita a SELEÇÃO inteira, como os irmãos: com três folhas selecionadas, arruma
            // as três. A lei de alvo é a mesma do "Merge Sprites" — a seleção quando a linha
            // clicada faz parte dela, só ela quando não faz — porque duas leis de alvo no mesmo
            // menu seriam adivinhação.
            if let Some(row) = arrange_sheet_row
                && let Some(live) = hero_live.as_ref()
                && let Some(anchor) = live.bridge.entity_for(row)
            {
                let mut targets: Vec<u64> = Vec::new();
                if hero.gizmo.is_selected(anchor) {
                    targets.extend(hero.gizmo.iter_selected());
                } else {
                    targets.push(anchor);
                }
                let sheets = crate::sheet_frame::sheets_among(sim, &targets);
                if sheets.is_empty() {
                    toasts.push(Toast::warning(
                        "Auto-Arrange: select a sheet - to make one, use Pack into Sheet",
                    ));
                } else {
                    crate::sheet_frame::repack_all(sim, vec_scene, &sheets, toasts);
                }
                self.title_dirty = true;
            }
            // **ASSAR** — a folha vira UMA textura, e cada peça uma janela nela (plano §7.3).
            //
            // ⚠️ Age sobre a linha clicada e só sobre ela, ao contrário dos irmãos: assar é caro
            // (lê N texturas da GPU) e produz um ficheiro por folha, então difundi-lo pela seleção
            // faria um clique distraído reamostrar meia cena. *O custo do gesto decide o alcance
            // dele.*
            if let Some(row) = bake_sheet_row
                && let Some(live) = hero_live.as_ref()
                && let Some(bits) = live.bridge.entity_for(row)
            {
                crate::sheet_bake::bake(
                    sim,
                    renderer,
                    asset_db,
                    atlas_asset_map,
                    sheets,
                    sheet_textures,
                    next_sheet_id,
                    bits,
                    toasts,
                );
                self.title_dirty = true;
            }
            // **EXPORTAR** — os mesmos pixels, mas para disco, e **sem tocar na cena**.
            if let Some(row) = export_sheet_row
                && let Some(live) = hero_live.as_ref()
                && let Some(bits) = live.bridge.entity_for(row)
                && let Some((authored, _)) = crate::sheet_bake::compose_sheet(
                    sim,
                    renderer,
                    asset_db,
                    atlas_asset_map,
                    next_sheet_id,
                    bits,
                    toasts,
                )
            {
                crate::sheet_export::export(&authored, toasts);
            }
            // **EXPORTAR UMA SPRITE** (plano `docs/Sprite_projeto/18` W9, Enio 2026-08-21). Os 16
            // exportadores da engine já estavam registados e nenhum gesto os alcançava; esta é a
            // porta. ⚠️ Uma sprite de 16 bits é oferecida em ALTA PRECISÃO primeiro, e só cai para
            // 8 bits quando o formato escolhido a recusa (e aí diz-se).
            if let Some(row) = export_image_row
                && let Some(live) = hero_live.as_ref()
                && let Some(bits) = live.bridge.entity_for(row)
            {
                crate::image_export::export_with_dialog(
                    ph2d_ecs::Entity::from_bits(bits),
                    sim,
                    renderer,
                    asset_db,
                    atlas_asset_map,
                    imageio_exporters,
                    toasts,
                );
                self.title_dirty = true;
            }
            // O Create do modal — a criação de facto, com os alvos que ficaram reservados.
            if let Some(size_px) = hero.store.take_sheet_size_request() {
                let targets = std::mem::take(&mut self.pending_sheet_targets);
                let ppm = hero.project.pixels_per_meter;
                if let Some(sheet) = crate::sheet_frame::create_at(
                    sim,
                    vec_scene,
                    &mut self.vec.entities,
                    &targets,
                    ppm,
                    size_px,
                    toasts,
                ) {
                    // A folha fica selecionada: é ela que o artista vai querer mover,
                    // redimensionar ou nomear a seguir — e é o convite a verificar que tudo
                    // isso funciona sem uma linha de código próprio.
                    hero.gizmo.replace_selection(Some(sheet));
                }
                self.title_dirty = true;
            }
            // §12 Physics Joint (W3) — the joint edits and the one gesture that
            // creates a joint. Deletion is NOT here: a joint is an object, so
            // "Delete Joint" despawns it through the same path any other object
            // takes, and gets the same undo step for free.
            for &(bits, edit) in &joint_edits {
                // `PickBodyA/B` never reach here — they arm a canvas pick in the
                // action loop above (shell state, not a component edit). `Remove`
                // despawns the joint object; everything else is a field edit.
                if matches!(edit, ph2d_editor_core::JointFieldEdit::Remove) {
                    let e = ph2d_ecs::Entity::from_bits(bits);
                    let _ = sim.world_mut().despawn(e);
                } else if let ph2d_editor_core::JointFieldEdit::AnchorToWorld(on) = edit {
                    // Estrutural como os dois irmãos: acrescenta/remove o
                    // MARCADOR `JointWorldAnchor` (W-JointWorld). Não há campo de
                    // `PhysicsJoint` a escrever, então ele não passa pelo funil.
                    let e = ph2d_ecs::Entity::from_bits(bits);
                    if on {
                        sim.world_mut()
                            .entity_mut(e)
                            .insert(ph2d_physics_ecs::JointWorldAnchor);
                    } else {
                        sim.world_mut()
                            .entity_mut(e)
                            .remove::<ph2d_physics_ecs::JointWorldAnchor>();
                    }
                } else if matches!(edit, ph2d_editor_core::JointFieldEdit::CopyProperties) {
                    // ARMA a área de transferência — estado da shell, nenhum
                    // componente muda (por isso não passa pelo funil nem pela
                    // fila). Guarda o componente INTEIRO: quem decide o que é
                    // uma *propriedade* é `with_properties_of`, no paste, e uma
                    // segunda triagem aqui seria a segunda resposta que diverge.
                    let e = ph2d_ecs::Entity::from_bits(bits);
                    self.joint_clipboard = sim
                        .world()
                        .get::<ph2d_physics_ecs::PhysicsJoint>(e)
                        .copied();
                } else if matches!(edit, ph2d_editor_core::JointFieldEdit::PasteProperties) {
                    // O fan-out já aconteceu no laço de ações: aqui é sempre UM
                    // joint. A fonte é a área de transferência; sem ela o botão
                    // nem foi pintado, e este ramo é um no-op honesto.
                    if let Some(src) = self.joint_clipboard {
                        ph2d_app_physics::joint::paste_joint_properties(
                            sim,
                            bits,
                            &src,
                            editor_queue,
                            component_registry,
                        );
                        // FLUSH por edição, pela razão que o ramo abaixo
                        // documenta: `paste_joint_properties` só ENFILEIRA, e
                        // ele read-modify-writes o componente inteiro — um
                        // segundo paste no mesmo frame que lesse um primeiro
                        // ainda não aplicado o descartaria em silêncio, que é
                        // exatamente o caso do fan-out sobre dez joints.
                        if let Err(e) = ph2d_ecs::scene::apply_editor_commands(
                            sim.world_mut(),
                            editor_queue,
                            component_registry,
                        ) {
                            toasts.push(ph2d_editor_core::Toast::error(format!(
                                "Joint commit failed: {e}"
                            )));
                        }
                    }
                } else if matches!(edit, ph2d_editor_core::JointFieldEdit::AddWheel) {
                    // Estrutural como o `Remove`, do outro lado: SPAWNA um
                    // objeto. O undo global por-diff o captura como captura
                    // qualquer outro spawn, sem um passo próprio a inventar.
                    ph2d_app_physics::joint_wheel::add_pulley_wheel(sim, physics, bits);
                } else {
                    ph2d_app_physics::joint::apply_joint_edit(
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
                        toasts.push(ph2d_editor_core::Toast::error(format!(
                            "Joint commit failed: {e}"
                        )));
                    }
                }
            }
            // §14 Platform Player (W5). Toda edição escreve o componente no
            // lugar (ou o anexa/remove), então não há fila de comandos de
            // entidade a drenar: o undo global por-diff captura o passo.
            for &(bits, edit) in &player_edits {
                ph2d_app_physics::inspector::player::apply_player_edit(sim, bits, edit);
            }
            // §13 Pulley Wheel (W-Pulley W1). Toda edição é de COMPONENTE — não
            // há aqui o par estrutural da §12 (criar/apagar uma roldana é criar
            // ou apagar um OBJETO, e a Hierarquia já sabe fazer os dois).
            for &(bits, edit) in &wheel_edits {
                let route_changed = ph2d_app_physics::joint_wheel::apply_wheel_edit(
                    sim,
                    bits,
                    edit,
                    editor_queue,
                    component_registry,
                );
                // FLUSH por edição, pela razão que o laço do joint documenta ao
                // lado: o apply lê-modifica-escreve o componente INTEIRO, então
                // uma segunda edição no mesmo frame que lesse a primeira ainda
                // não aplicada a descartaria em silêncio.
                if let Err(e) = ph2d_ecs::scene::apply_editor_commands(
                    sim.world_mut(),
                    editor_queue,
                    component_registry,
                ) {
                    toasts.push(ph2d_editor_core::Toast::error(format!(
                        "Wheel commit failed: {e}"
                    )));
                }
                // ⚠️ **DEPOIS do flush, e só quando a ROTA mudou.** O `L0` da corda
                // é derivado da rota e era semeado UMA vez; digitar um raio maior
                // deixava a restrição `L(rota) <= L0` violada e o solver comia a
                // diferença num salto (medido: 14,12 m a raio 0,90). É a mesma
                // porta que o arrasto da alça usa — uma resposta, dois gestos.
                if route_changed {
                    ph2d_physics_ecs::reseat_wheel_geometry(
                        sim.world_mut(),
                        ph2d_ecs::Entity::from_bits(bits),
                    );
                }
            }
            if join_draw_arm {
                // TOGGLE, nao "arma" (W-J4b): o gesto e modal e come o press no
                // canvas, entao sem uma saida o unico jeito de sair era completar
                // um joint que o artista nao queria. Pela porta unica
                // `toggle_joint_draw`, a MESMA que o Esc usa.
                ph2d_app_physics::joint_draw::toggle(
                    &mut self.physics.joint_draw_armed,
                    &mut self.physics.joint_draw,
                );
            }
            if join_chain {
                let (made, last) = ph2d_app_physics::joint_draw::join_chain(
                    sim,
                    &inspector_selection,
                    ph2d_app_physics::joint::kind_of(self.physics.join_kind),
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
                    toasts.push(ph2d_editor_core::Toast::info(format!(
                        "Chained {} bodies with {made} joints",
                        made + 1
                    )));
                }
            }
            // W-Rig: a TERCEIRA rota de criação — a única que não pede ao artista
            // que redescreva uma estrutura que ele já desenhou. Depois do
            // `join_chain` porque as duas escrevem joints, e a ordem entre elas
            // num mesmo frame só importaria se o artista tivesse clicado nas duas
            // (ele não pode: são dois botões).
            if rig_now {
                let roots: Vec<u64> = hero.gizmo.iter_selected().collect();
                let plan = ph2d_app_physics::joint_rig::plan(sim, &roots);
                let out = ph2d_app_physics::joint_rig::apply(
                    sim,
                    &plan,
                    ph2d_app_physics::joint::kind_of(self.physics.join_kind),
                    editor_queue,
                    component_registry,
                );
                // ⚠️ O flush mora DENTRO do gerador, entre dar corpo e ligar: a
                // emenda mede o `Collider` que a primeira metade acabou de
                // enfileirar. Aqui fica só o deck de toasts, que é deste laço.
                if let Some(e) = out.error {
                    toasts.push(ph2d_editor_core::Toast::error(format!(
                        "Rig commit failed: {e}"
                    )));
                }
                let (bodies, joints, last) = (out.bodies, out.joints, out.last);
                // Seleciona o ÚLTIMO joint, pelo motivo que o `join_chain`
                // documenta: a §12 abre na hora, com o Kind e a afinação à mão —
                // e afinar UM e carimbar o resto é o gesto que a W-JointCopy
                // acabou de tornar barato.
                if let Some(j) = last {
                    hero.gizmo.selection = Some(j.to_bits());
                    hero.gizmo.extra_selection.clear();
                }
                if joints > 0 {
                    toasts.push(ph2d_editor_core::Toast::info(format!(
                        "Rigged {bodies} new bodies with {joints} joints"
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
                let (start, end) =
                    ph2d_app_physics::bake::bake_range(&self.timeline.doc, &self.playhead);
                let outcome = ph2d_app_physics::bake::bake_selection(
                    &mut self.timeline,
                    physics,
                    sim,
                    &entities,
                    start,
                    end,
                    self.fixed_step.fixed_dt(),
                    self.bake_channels,
                    &mut self.player_tape,
                    editor_queue,
                    component_registry,
                );
                if outcome.unmappable {
                    toasts.push(ph2d_editor_core::Toast::info(
                        "Cannot bake here: this clip does not play exactly once",
                    ));
                } else if outcome.refused {
                    toasts.push(ph2d_editor_core::Toast::info(
                        "Finish the current edit before baking",
                    ));
                } else if outcome.already_baked {
                    toasts.push(ph2d_editor_core::Toast::info(
                        "Already baked - the timeline drives these bodies now",
                    ));
                } else if outcome.is_empty() {
                    toasts.push(ph2d_editor_core::Toast::info(
                        "Nothing to bake: nothing moved",
                    ));
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
                    // `ph2d_app_physics::bridge::dispatch::dispatch`. Still playing, the clock is
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
                    toasts.push(ph2d_editor_core::Toast::info(format!(
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
            // The pass authors on the clock the APPLY drove this parent: the clip
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
                &self.preview_drive,
            );
            // Merge Sprites (Enio 2026-05-27, Hierarchy right-click).
            // Drains BEFORE `image_edit::dispatch` so a same-frame
            // image-edit on one of the originals (extremely unlikely
            // path but documented) doesn't race with the despawn. The
            // multi-selection comes from `hero.gizmo` — primary first,
            // extras after. Right-clicked row resolves to the "primary
            // anchor" the merged sprite parents under.
            // ⚠️ As DUAS entradas do menu caem aqui: «Merge Sprites» e «Merge to Layers». A
            // geometria é a mesma e o modo é a única diferença — ver a chamada abaixo.
            // ⭐⭐⭐ **AGRUPAR / DESAGRUPAR** (Enio, 2026-08-30) — o alcance de um verbo que já
            // existia em `Ctrl+G` e que nenhum menu do app nomeava. A lei do sujeito e as frases
            // vivem em [`crate::hier_group`], puras e gateadas; aqui só se resolve a linha em bits,
            // se aplica e se diz.
            if let Some((row, agrupar)) = group_row
                && let Some(live) = hero_live.as_ref()
                && let Some(row_bits) = live.bridge.entity_for(row)
            {
                let escolhidos: Vec<u64> = hero.gizmo.iter_selected().collect();
                let sujeito = crate::hier_group::subject(row_bits, &escolhidos);
                let desfecho = crate::hier_group::apply(sim, &sujeito, agrupar);
                if let crate::hier_group::Outcome::Grouped { group, .. } = desfecho {
                    // O grupo novo passa a ser a selecção — o gesto seguinte do artista é sobre
                    // ELE, e não sobre as peças que acabaram de deixar de ser objectos de topo.
                    hero.gizmo.selection = Some(group);
                    hero.gizmo.extra_selection.clear();
                    // ⚠️ Recolher fica para quando a Hierarquia conhecer a linha — ver o campo.
                    self.pending_group_collapse = Some(group);
                }
                toasts.push(desfecho.toast());
                self.title_dirty = true;
            }
            // ⭐ A segunda metade do recolher: a linha do grupo já existe? Então recolhe-a **uma
            // vez** e esquece. ⚠️ Sem o `take`, o artista abria o grupo e o quadro seguinte
            // fechava-o outra vez — um controlo que se desfaz sozinho lê-se como avaria.
            if let Some(bits) = self.pending_group_collapse
                && let Some(live) = hero_live.as_ref()
                && let Some(node) = live.bridge.node_for(bits)
            {
                if !hero.store.is_hierarchy_collapsed(node) {
                    hero.store.toggle_hierarchy_collapsed(node);
                }
                self.pending_group_collapse = None;
            }
            if let Some(row) = merge_sprites_row.or(merge_to_layers_row)
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
                    toasts.push(ph2d_editor_core::Toast::warning(
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
                        // ⚠️ O MODO vem de qual das duas linhas do menu foi clicada, e as duas
                        // caem neste mesmo dreno: a geometria é idêntica e duplicá-la seria pedir
                        // que duas cópias concordassem para sempre.
                        merge_to_layers_row.is_some(),
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
                    // **O DOCUMENTO EM CAMADAS** (plano `docs/Sprite_projeto/18` W10). A sprite
                    // fundida já tem a textura achatada — ela desenha certo desde já, e grava. O
                    // que isto acrescenta é o documento do Painter: abrir o Painter nela mostra
                    // uma camada por sprite de origem, na ordem em que foram compostas.
                    //
                    // ⚠️ **Instala-se AQUI e não dentro da fusão**: quem tem a `ToolRegistry` é o
                    // shell. O `sprite_merge` sabe geometria; não sabe onde vive a ferramenta.
                    if let Some(doc) = hero_intents::take_last_merged_layers() {
                        crate::merge_layers::install(tools, &doc, toasts);
                    }
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
                    tools.set_active(&ph2d_editor_core::ToolId::new("painter"));
                    if let Some(painter) = tools.active_mut().and_then(|t| {
                        t.as_any_mut()
                            .downcast_mut::<ph2d_tool_painter::PainterTool>()
                    }) {
                        if as_shape {
                            painter.capture_layers_as_brush_shape();
                            toasts.push(ph2d_editor_core::Toast::success(
                                "Brush shape set from layers",
                            ));
                            handled = true;
                        } else if let Some((lum, w, h)) = painter.composite_to_lum() {
                            painter.set_brush_texture_image(lum, w, h);
                            toasts.push(ph2d_editor_core::Toast::success(
                                "Brush grain set from sprite",
                            ));
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
                                .as_chunks::<4>()
                                .0
                                .iter()
                                .map(|p| {
                                    ((u32::from(p[0]) * 77
                                        + u32::from(p[1]) * 150
                                        + u32::from(p[2]) * 29)
                                        >> 8) as u8
                                })
                                .collect();
                            // Reach the painter only via the active tool → activate it first.
                            tools.set_active(&ph2d_editor_core::ToolId::new("painter"));
                            if let Some(painter) = tools.active_mut().and_then(|t| {
                                t.as_any_mut()
                                    .downcast_mut::<ph2d_tool_painter::PainterTool>()
                            }) {
                                if as_shape {
                                    // ⚠️ A COR do sprite viaja junto (Enio, 2026-08-09): a silhueta
                                    // continua sendo a mesma luminância, byte a byte, mas o slot passa a
                                    // guardar uma camada com o RGB, e é isso que dá ao checkbox "Use
                                    // Texture Colors" o que ligar. Antes daqui saía só a máscara — a cor
                                    // morria na conversão para cinza, e pintar com as cores da textura
                                    // era possível apenas para o documento ABERTO no Painter.
                                    painter.set_brush_shape_image_rgba(
                                        &src.image.pixels,
                                        w,
                                        h,
                                        Some(bits),
                                    );
                                    toasts.push(ph2d_editor_core::Toast::success(
                                        "Brush shape set from sprite",
                                    ));
                                } else {
                                    painter.set_brush_texture_image(lum, w, h);
                                    toasts.push(ph2d_editor_core::Toast::success(
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
                            toasts.push(ph2d_editor_core::Toast::warning(format!(
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
                    tools.set_active(&ph2d_editor_core::ToolId::new("painter"));
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
                            .as_chunks::<4>()
                            .0
                            .iter()
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
                        tools.set_active(&ph2d_editor_core::ToolId::new("painter"));
                        if let Some(painter) = tools.active_mut().and_then(|t| {
                            t.as_any_mut()
                                .downcast_mut::<ph2d_tool_painter::PainterTool>()
                        }) {
                            if as_granulation {
                                painter.use_layers_as_granulation(lum, w, h);
                                toasts.push(ph2d_editor_core::Toast::success(
                                    "Watercolor granulation set from layer",
                                ));
                            } else {
                                painter.use_layers_as_watercolor_paper(lum, w, h);
                                toasts.push(ph2d_editor_core::Toast::success(
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
                        toasts.push(ph2d_editor_core::Toast::warning(format!(
                            "Use as {what}: select an image sprite"
                        )));
                    }
                }
                self.title_dirty = true;
            }
            self.fase_image_edit_apply(
                fase_image_edit_apply::ImageEditIntents {
                    trim_entities,
                    make_square_entities,
                    real_size_entities,
                    rasterize_entities,
                    undo_image_edit,
                },
                padding_apply,
                bgremoval_apply_committed,
                color_equalization_apply,
                equalize_sizes_apply,
                upscale_apply,
                painter_apply_committed,
            );
            self.fase_hero_chrome_tail(viewport);
        } else {
            self.fase_legacy_chrome(viewport);
        }

        self.fase_ui_burst_paint();

        // Paint + present + title — extracted to `present.rs` sibling
        // method (Wave 3.2 stage A). Re-acquires self.gfx + self.host
        // refs inside; values needed are passed explicitly.
        self.run_present_phase(cpu_start, r, g, b);

        self.fase_frame_profile();
    }
}
