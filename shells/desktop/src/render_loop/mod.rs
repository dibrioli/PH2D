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
/// Fase do quadro: os controlos autorados e os estados de UI.
mod fase_authored_controls_and_ui_states;
/// Fase do quadro: o AutoKey.
mod fase_autokey;
/// Fase do quadro: o IK e os limites do osso.
mod fase_bone_ik_and_limits;
/// Fase do quadro: os smart bones e os numeros do osso.
mod fase_bone_smart_and_knobs;
/// Fase do quadro: as sobreposicoes do canvas.
mod fase_canvas_overlays;
/// Fase do quadro: o relógio do chrome (`wall_dt`, `ui_dt` e os tiques que andam nele).
mod fase_chrome_clock;
/// Fase do quadro: a paleta de componentes.
mod fase_component_palette;
/// Fase do quadro: os verbos de componente.
mod fase_component_verbs;
/// Fase do quadro: o composto, os encaixes e as reguas.
mod fase_compound_snap_rulers;
/// Fase do quadro: o conector e os parametros de forma.
mod fase_connector_and_shape_params;
/// Fase do quadro: o converter em curvas.
mod fase_convert_to_curves;
/// Fase do quadro: a sincronizacao das entidades e as formas vivas.
mod fase_entity_sync;
/// Fase do quadro: o envelope.
mod fase_envelope;
/// Fase do quadro: os insumos do extract (passo, pré-visualizações, folha aberta, px/m, filtro).
mod fase_extract_inputs;
/// Fase do quadro: os pedidos do modelador 3D.
mod fase_field3d_requests;
/// Fase do quadro: o desenho do modelador 3D.
mod fase_field3d_smoke_draw;
/// Fase do quadro: os comandos da pilha de filtros.
mod fase_filter_commands;
/// Fase do quadro: o valor do filtro e a cor do picker.
mod fase_filter_values_and_colour;
/// Fase do quadro: os relógios do passo fixo (sim, cabeças de leitura, §11 Animation, timers).
mod fase_fixed_step_clocks;
/// Fase do quadro: a tira e o cursor do Flip.
mod fase_flip_strip_and_cursor;
/// Fase do quadro: as fontes.
mod fase_fonts;
/// Fase do quadro: o perfilador (conta os quadros e chama o relatório a cada 120).
mod fase_frame_profile;
/// Fase do quadro: o relatório do perfilador (a partição do quadro a cada 120 quadros).
mod fase_frame_profile_report;
/// Fase do quadro: a câmera de jogo (o herói da cena de smoke e o passe da câmera).
mod fase_game_camera;
/// Fase do quadro: a supressao do gizmo e a moldura do modelador 3D.
mod fase_gizmo_suppression_and_field3d_frame;
/// Fase do quadro: o fim do ramo hero (toasts, barras de trabalho, a arena do quadro).
mod fase_hero_chrome_tail;
/// Fase do quadro: a pintura do ecra hero.
mod fase_hero_paint;
/// Fase do quadro: o despacho da Hierarquia.
mod fase_hierarchy_dispatch;
/// Fase do quadro: agrupar, recolher e fundir sprites.
mod fase_hierarchy_group_merge;
/// Fase do quadro: a trava do Painter na seleccao da Hierarquia.
mod fase_hierarchy_select_lock;
/// Fase do quadro: o dreno de edicao de imagem e os desmontes do Apply.
mod fase_image_edit_apply;
/// Fase do quadro: a entrada (carimbo coalescido, diagnóstico, gamepad, script, soltos).
mod fase_input_and_drops;
/// Fase do quadro: os commits do Inspector.
mod fase_inspector_commits;
/// Fase do quadro: o chrome legado (o ramo sem `HeroScreen`).
mod fase_legacy_chrome;
/// Fase do quadro: o offset e a largura vivos.
mod fase_live_offset_and_width;
/// Fase do quadro: o Motion: publicar, despachar e os sinais.
mod fase_motion_bridge;
/// Fase do quadro: o modal de imagem nova (Cmd/Ctrl+N) cria a tela escolhida.
mod fase_new_image_modal;
/// Fase do quadro: os verbos de no e de arranjo.
mod fase_node_and_arrange_verbs;
/// Fase do quadro: a receita aberta (a marca, o pedido de palco, a trava e o pedido do Cancel).
mod fase_open_recipe;
/// Fase do quadro: as cenas do pincel do Painter (taper, tinta molhada).
mod fase_painter_brush_smokes;
/// Fase do quadro: os efeitos, o spine, os passos e a booleana.
mod fase_path_effects_spine_bool;
/// Fase do quadro: a forma do caminho e as tintas.
mod fase_path_shape_and_paint;
/// Fase do quadro: o padrao no caminho, o pincel e os pickers.
mod fase_pattern_path_and_pickers;
/// Fase do quadro: as edicoes de joint, player e roldana.
mod fase_physics_edits;
/// Fase do quadro: ligar, rigar e assar a fisica.
mod fase_physics_join_rig_bake;
/// Fase do quadro: a sobreposicao da fisica.
mod fase_physics_overlay;
/// Fase do quadro: o passo da física (dispatch, flash, readout do player, juntas que cederam).
mod fase_physics_step;
/// Fase do quadro: o que está sob o cursor (a 1.ª do `run_render_frame`).
mod fase_pointer_subjects;
/// Fase do quadro: os verbos de receita e de assets.
mod fase_recipe_and_asset_verbs;
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
/// Fase do quadro: o realce da seleccao.
mod fase_selection_highlight;
/// Fase do quadro: o osso em foco no painel do esqueleto.
#[cfg(feature = "panel-vector")]
mod fase_selection_mirror_bone_focus;
/// Fase do quadro: o converter e o envelope no painel.
#[cfg(feature = "panel-vector")]
mod fase_selection_mirror_convert_envelope;
/// Fase do quadro: os efeitos e os presets de envelope no painel.
#[cfg(feature = "panel-vector")]
mod fase_selection_mirror_effects_envelope;
/// Fase do quadro: a pilha de filtros no painel.
#[cfg(feature = "panel-vector")]
mod fase_selection_mirror_filters;
/// Fase do quadro: o texto, o padrao e o contorno no painel.
#[cfg(feature = "panel-vector")]
mod fase_selection_mirror_path_links;
/// Fase do quadro: a pele e a ferramenta do osso no painel.
#[cfg(feature = "panel-vector")]
mod fase_selection_mirror_skin;
/// Fase do quadro: a manutenção de sessão (Shape Builder, tween, Colorize, Gap Closure).
mod fase_session_upkeep;
/// Fase do quadro: o latch da forma armada e os campos de forma.
mod fase_shape_fields;
/// Fase do quadro: os verbos da folha de sprites.
mod fase_sheet_verbs;
/// Fase do quadro: o outbox de sinais (os produtores que faltavam e o dreno).
mod fase_signal_outbox;
/// Fase do quadro: o extract (propagação, emissão das sprites e a ordem total do quadro).
mod fase_sim_extract;
/// Fase do quadro: os verbos do esqueleto.
mod fase_skeleton_verbs;
/// Fase do quadro: a estrategia de origem e o re-assento do pivo.
mod fase_source_strategy_and_joint_pivot;
/// Fase do quadro: as cenas do Sprite Inspector (9-slice, âncoras, montagem, Animation).
mod fase_sprite_inspector_smokes;
/// Fase do quadro: as cenas dos pixels da sprite (`.ase`, dither, emissiva).
mod fase_sprite_pixel_smokes;
/// Fase do quadro: a precisao, a emissao e o Remove from Sheet.
mod fase_sprite_precision_emissive;
/// Fase do quadro: o resize coalescido e o modo de apresentação do arrasto.
mod fase_surface_resize;
/// Fase do quadro: o padrao de textura, os gradientes, o alinhamento e o pivo.
mod fase_texpat_gradient_align;
/// Fase do quadro: os campos de texto.
mod fase_text_fields;
/// Fase do quadro: o estilo do texto e o painel de texto.
mod fase_text_panel;
/// Fase do quadro: o relógio dos contêineres da timeline.
mod fase_timeline_containers;
/// Fase do quadro: o dreno da timeline (intents, aplicação, reset, valores das faixas).
mod fase_timeline_drain;
/// Fase do quadro: a vista da timeline (amostragem, intents estacionadas, espelhos do painel).
mod fase_timeline_view;
/// Fase do quadro: os espelhos da ferramenta vetorial, do Flip e da fisica.
mod fase_tool_mirrors;
/// Fase do quadro: as operacoes de transformacao.
mod fase_transform_ops;
/// Fase do quadro: a poeira de impacto (as faíscas por cima do chrome).
mod fase_ui_burst_paint;
/// Fase do quadro: a transicao do hospedeiro.
mod fase_ui_host_transition;
/// Fase do quadro: a previa dos estados e mover com todos os estados.
mod fase_ui_state_preview;
/// Fase do quadro: o Use as Brush Shape / Grain da Hierarquia.
mod fase_use_as_brush;
/// Fase do quadro: o Use as Paper / Granulation da Hierarquia.
mod fase_use_as_paper;
/// Fase do quadro: o Apply Offset e o Power Stroke.
mod fase_vec_expand;
/// Fase do quadro: as faixas do documento.
mod fase_vector_bands;
/// Fase do quadro: o overlay dos ossos.
mod fase_vector_bone_overlay;
/// Fase do quadro: o Apply booleano e a reconciliacao dos conjuntos de Morph.
mod fase_vector_bool_apply_and_morph_reconcile;
/// Fase do quadro: o grupo booleano e o verbo da forma.
mod fase_vector_bool_shape_row;
/// Fase do quadro: o overlay de edicao vectorial e as imagens com pele.
mod fase_vector_edit_overlay;
/// Fase do quadro: a recozedura de FX e de padroes.
mod fase_vector_fx_recook;
/// Fase do quadro: as linhas de corte, as guias e a construcao de forma.
mod fase_vector_guides_and_build;
/// Fase do quadro: o layout vivo, o alinhamento e a silhueta.
mod fase_vector_layout_recook;
/// Fase do quadro: a simetria, o lapis e a geometria viva fundida.
mod fase_vector_live_geometry;
/// Fase do quadro: as etiquetas, a vista do pen e as recozeduras vivas.
mod fase_vector_live_recooks;
/// Fase do quadro: os verbos do Morph.
mod fase_vector_morph_verbs;
/// Fase do quadro: os overlays vectoriais do quadro.
mod fase_vector_overlays;
/// Fase do quadro: o despacho do painel vectorial e a tinta do traco.
mod fase_vector_panel_dispatch;
/// Fase do quadro: a moldura, o layout, o z e as ancoras da seleccao.
mod fase_vector_selection_frame_panel;
/// Fase do quadro: a pele, os estados, o z-index e o layout publicados.
mod fase_vector_selection_states_panel;
/// Fase do quadro: os tokens e as etiquetas das molduras.
mod fase_vector_tokens_and_labels;
/// Fase do quadro: as alcas da ferramenta vectorial.
mod fase_vector_tool_handles;
/// Fase do quadro: o assentamento das origens e da arvore.
mod fase_vector_tree_settle;
/// Fase do quadro: as manutencoes vivas do vector.
mod fase_vector_upkeeps;
/// Fase do quadro: a vista vectorial, os estilos conduzidos e as recozeduras de forma.
mod fase_vector_view_and_drives;
/// Fase do quadro: as pontes do modelador 3D, dos tokens e da escultura.
mod fase_world_panel_bridges;
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

/// Qual dos três números da ÂNCORA o campo escreveu (no MÓDULO: viaja nas intenções de uma fase de outro ficheiro).
#[derive(Clone, Copy, PartialEq)]
enum IkKnob {
    Mix,
    Softness,
    Chain,
}
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
            #[cfg(feature = "sculpt3d")]
            surface,
            renderer,
            sim,
            present,
            camera,
            asset_db,
            theme,
            toasts,
            tools,
            vector_scene,
            vec_scene,
            flip,
            text_system,
            hero_screen,
            hero_live,
            sheets,
            atlas_asset_map,
            asset_catalogs,
            component_registry,
            motion,
            physics,
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
            let duplicate_made: Option<(u64, u64)> = None;
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
            let inspector_queue_dirty = false;
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
                // PRECISION-READONLY: o fecho só entrega os pixels ao canvas de TRABALHO do Painter; a
                // escrita de volta na sprite é o Apply (`hero_intents::image_edit::painter`), por
                // `commit_edited_texture`, que avisa por dentro.
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
                let Some(sel) =
                    self.fase_filter_commands(fase_filter_commands::FilterCommandsIntents {
                        pending_filter_cmd,
                        pending_filter_stop,
                    })
                else {
                    return;
                };
                self.fase_filter_values_and_colour(
                    fase_filter_values_and_colour::FilterValuesAndColourIntents {
                        pending_filter_val,
                    },
                    sel,
                );
            }
            self.fase_pattern_path_and_pickers(
                fase_pattern_path_and_pickers::PatternPathAndPickersIntents {
                    pending_patternpath,
                    pending_pp_spacing,
                    pending_pp_start,
                    pending_pp_end,
                    pending_pp_slide,
                    pending_pp_offset,
                    pending_pp_rotation,
                    pending_pp_pick,
                    pending_texpat_pick,
                    pending_brush_pick,
                    pending_brush,
                },
            );
            let Some(mudos_antes) =
                self.fase_skeleton_verbs(fase_skeleton_verbs::SkeletonVerbsIntents {
                    pending_bone_bind,
                    pending_bone_release,
                    pending_bone_knob,
                    osso_selecionado,
                    selecao_bits,
                })
            else {
                return;
            };
            if let Some(bits) = osso_selecionado {
                let osso = ph2d_ecs::Entity::from_bits(bits);
                let Some(osso) = self.fase_bone_ik_and_limits(
                    fase_bone_ik_and_limits::BoneIkAndLimitsIntents {
                        pending_ik_add,
                        pending_ik_remove,
                        pending_ik_bend,
                        pending_limit_add,
                        pending_limit_remove,
                    },
                    osso,
                ) else {
                    return;
                };
                self.fase_bone_smart_and_knobs(
                    fase_bone_smart_and_knobs::BoneSmartAndKnobsIntents {
                        pending_ik_knob,
                        pending_limit_knob,
                        pending_smart_add,
                        pending_smart_remove,
                        pending_smart_knob,
                        pending_smart_clip,
                        pending_smart_pick,
                    },
                    mudos_antes,
                    osso,
                );
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
            self.fase_envelope(fase_envelope::EnvelopeIntents {
                pending_create_envelope,
                pending_expand_envelope,
                pending_release_envelope,
                pending_envelope_kind,
                pending_clear_pins,
                pending_envelope_preset,
                pending_envelope_bend,
            });
            self.fase_path_effects_spine_bool(
                fase_path_effects_spine_bool::PathEffectsSpineBoolIntents {
                    pending_vec_bool,
                    pending_reset_spine,
                    pending_blend_steps,
                    pending_fx_add,
                    pending_fx_button,
                    pending_fx_param,
                    pending_fx_apply,
                },
            );
            self.fase_live_offset_and_width(
                fase_live_offset_and_width::LiveOffsetAndWidthIntents {
                    pending_width_preset,
                },
            );
            self.fase_authored_controls_and_ui_states(
                fase_authored_controls_and_ui_states::AuthoredControlsAndUiStatesIntents {
                    pending_widget_edit,
                    pending_ui_state,
                },
            );
            self.fase_ui_host_transition(fase_ui_host_transition::UiHostTransitionIntents {
                pending_ui_state_duration,
                pending_ui_spring_toggle,
                pending_ui_spring_knob,
                pending_ui_easing,
                pending_ui_signal_edit,
                pending_ui_signal_name,
            });
            self.fase_ui_state_preview(
                fase_ui_state_preview::UiStatePreviewIntents {
                    pending_ui_preview_toggle,
                    pending_ui_move_all_toggle,
                    pending_morph_preview_toggle,
                },
                report,
            );
            self.fase_component_verbs(
                fase_component_verbs::ComponentVerbsIntents { pending_component },
                window_size,
            );
            if self
                .fase_vec_expand(fase_vec_expand::VecExpandIntents { pending_vec_expand })
                .is_none()
            {
                return;
            }
            self.fase_compound_snap_rulers(fase_compound_snap_rulers::CompoundSnapRulersIntents {
                pending_vec_compound,
                pending_vec_fill_rule,
                pending_vec_snap_on,
                pending_vec_snap_path,
                pending_vec_snap_cross,
                pending_vec_snap_guides,
                pending_rulers,
            });
            self.fase_node_and_arrange_verbs(
                fase_node_and_arrange_verbs::NodeAndArrangeVerbsIntents {
                    pending_vec_select_subpath,
                    pending_vec_select_same,
                    pending_vec_join,
                    pending_vec_weld,
                    pending_vec_cut,
                    pending_vec_symmetry_apply,
                    pending_vec_cut_discard,
                    pending_vec_reverse,
                    pending_vec_average,
                    pending_vec_vertex_kind,
                    pending_vec_delete_vertex,
                    pending_vec_reorder,
                    pending_vec_duplicate,
                    pending_vec_flip,
                    pending_vec_rotate,
                },
                vec_px_to_world,
            );
            let Some(vec_xf_ops) =
                self.fase_transform_ops(fase_transform_ops::TransformOpsIntents {
                    pending_frame_preset,
                    pending_vec_opacity,
                    pending_vec_blend,
                    pending_paint_verb,
                    pending_paint_width,
                    pending_paint_dx,
                    pending_paint_dy,
                    pending_paint_dilate,
                    pending_paint_join,
                    pending_paint_opacity,
                    pending_paint_blend,
                    pending_vec_transform,
                    pending_vec_vert,
                    pending_vec_rotate_by,
                })
            else {
                return;
            };
            let Some(vec_text_sel) = self.fase_connector_and_shape_params(
                fase_connector_and_shape_params::ConnectorAndShapeParamsIntents {
                    pending_vec_shape_param,
                    pending_vec_connector,
                },
                vec_px_to_world,
            ) else {
                return;
            };
            let Some(fase_text_fields::TextFieldsOut {
                editing_session,
                pending_vec_text_axis,
                vec_text_sel,
            }) = self.fase_text_fields(
                fase_text_fields::TextFieldsIntents {
                    pending_vec_text_size,
                    pending_vec_text_weight,
                    pending_vec_text_line_height,
                    pending_vec_text_tracking,
                    pending_vec_text_wrap,
                    pending_vec_text_align,
                    pending_vec_text_axis,
                },
                vec_text_sel,
            )
            else {
                return;
            };
            self.fase_fonts(
                fase_fonts::FontsIntents {
                    pending_vec_text_axis,
                    pending_vec_font_cycle,
                    pending_vec_font_pick,
                    pending_vec_font_import,
                },
                vec_text_sel,
                editing_session,
            );
            self.fase_path_shape_and_paint(fase_path_shape_and_paint::PathShapeAndPaintIntents {
                pending_vec_path_shape,
                pending_vec_toggle_closed,
                pending_vec_fill_kind,
                pending_vec_stroke_kind,
            });
            let Some(vec_xf_ops) = self.fase_texpat_gradient_align(
                fase_texpat_gradient_align::TexpatGradientAlignIntents {
                    pending_vec_pivot_edit,
                    pending_texpat,
                    pending_texpat_source,
                    pending_vec_grad_angle,
                    pending_vec_grad_add,
                    pending_vec_grad_remove,
                    pending_vec_grad_influence,
                    pending_vec_grad_jitter,
                    pending_vec_grad_add_stop,
                    pending_vec_grad_remove_stop,
                    pending_vec_align,
                    pending_vec_distribute,
                },
                vec_xf_ops,
            ) else {
                return;
            };
            let Some((vec_cfg, vec_xf_ops)) = self.fase_vector_panel_dispatch(
                fase_vector_panel_dispatch::VectorPanelDispatchIntents {
                    pending_stroke_present,
                },
                vec_px_to_world,
                vec_xf_ops,
            ) else {
                return;
            };
            self.fase_motion_bridge(vec_xf_ops);
            let Some(vec_cfg) = self.fase_tool_mirrors(vec_cfg) else {
                return;
            };
            self.fase_world_panel_bridges();
            let Some((flip_active, flip_style)) = self.fase_flip_strip_and_cursor(window_size)
            else {
                return;
            };
            self.fase_physics_overlay(window_size, viewport);
            self.fase_canvas_overlays(window_size);
            self.fase_selection_highlight(flip_active, flip_style, viewport);

            self.fase_text_panel(vec_px_to_world);

            self.fase_entity_sync(vec_cfg);
            self.fase_convert_to_curves(fase_convert_to_curves::ConvertToCurvesIntents {
                pending_vec_convert,
            });
            // Habilita "Convert to Curves" pela porta ÚNICA (`vec_convert::is_convertible`) — a
            // MESMA que o conversor honra. Enumerar as fontes aqui foi o que apodreceu duas
            // vezes: o botão ficava desligado num caminho só-efeitos e depois num só-quinas,
            // sempre sem erro nenhum. [[feedback_a_condition_that_enumerates_its_readers_rots]]
            #[cfg(feature = "panel-vector")]
            {
                let Some(env_container) = self.fase_selection_mirror_convert_envelope() else {
                    return;
                };
                self.fase_selection_mirror_skin();
                self.fase_selection_mirror_bone_focus();
                let Some(sel) = self.fase_selection_mirror_path_links() else {
                    return;
                };
                self.fase_selection_mirror_filters(sel);
                self.fase_selection_mirror_effects_envelope(env_container);
            }
            self.fase_shape_fields(vec_px_to_world);
            self.fase_vector_upkeeps();
            let Some((drawing, reparent_intent)) =
                self.fase_vector_tree_settle(fase_vector_tree_settle::TreeSettleIntents {
                    reparent_intent,
                })
            else {
                return;
            };
            let Some((vec_view, vec_xf)) = self.fase_vector_view_and_drives(vector_active) else {
                return;
            };
            let Some((cam_affine, vec_xf, vec_view)) =
                self.fase_vector_live_recooks(window_size, vector_active, vec_view, vec_xf)
            else {
                return;
            };
            let Some((vec_live, vec_view, mut vec_xf)) =
                self.fase_vector_live_geometry(window_size, drawing, vec_view, vec_xf)
            else {
                return;
            };
            // **O Apply corre AQUI, e não no dreno**, porque ele materializa o `plan` que o
            // `recook` acabou de computar — *o que está na tela*. Chamar o motor de novo lá em
            // cima seria a segunda porta, e ela faria a forma SALTAR no clique.
            //
            // ⚠️ A shell publica também *"há grupo booleano selecionado?"* para o painel decidir
            // se oferece o botão: o painel não alcança o mundo ECS, e uma segunda resposta a essa
            // pergunta seria um Apply pintado sobre uma seleção que não tem o que consolidar.
            {
                let Some(sel) = self.fase_vector_selection_frame_panel(
                    fase_vector_selection_frame_panel::FrameLayoutIntents {
                        pending_frame_clip,
                        pending_layout_edit,
                        pending_anchor_edit,
                        pending_layout_field,
                        pending_vec_z,
                    },
                ) else {
                    return;
                };
                let Some(sel) = self.fase_vector_selection_states_panel(
                    fase_vector_selection_states_panel::ResizeBoxIntents { pending_resize_box },
                    sel,
                ) else {
                    return;
                };
                let Some((vec_xf_back, sel)) = self.fase_vector_tokens_and_labels(
                    fase_vector_tokens_and_labels::TokenBindIntents { pending_token_bind },
                    vec_xf,
                    sel,
                ) else {
                    return;
                };
                vec_xf = vec_xf_back;
                let Some((group, sel)) = self.fase_vector_bool_shape_row(
                    fase_vector_bool_shape_row::BoolShapeIntents {
                        pending_bool_shape_op,
                    },
                    sel,
                ) else {
                    return;
                };
                self.fase_vector_morph_verbs(
                    fase_vector_morph_verbs::MorphVerbIntents {
                        pending_morph_arrow,
                    },
                    sel,
                );
                self.fase_vector_bool_apply_and_morph_reconcile(
                    fase_vector_bool_apply_and_morph_reconcile::BoolApplyIntents {
                        pending_bool_apply,
                    },
                    group,
                );
            }
            let Some((vec_view, vec_live, vec_xf)) =
                self.fase_vector_layout_recook(vec_view, vec_xf, vec_live)
            else {
                return;
            };
            let Some((vec_view, vec_xf, cam_affine, vec_live)) =
                self.fase_vector_fx_recook(vec_view, vec_xf, cam_affine, vec_live)
            else {
                return;
            };
            let Some((vec_view, vec_xf, cam_affine)) =
                self.fase_vector_bands(vec_view, vec_xf, cam_affine, vec_live, viewport)
            else {
                return;
            };
            let Some((overlay, vec_xf, cam_affine)) = self.fase_vector_overlays(
                window_size,
                motion_tool_active,
                vector_active,
                vec_xf,
                cam_affine,
            ) else {
                return;
            };
            let Some((vec_xf, cam_affine)) =
                self.fase_vector_edit_overlay(vec_view, vec_xf, cam_affine, overlay)
            else {
                return;
            };
            let Some(cam_affine) =
                self.fase_vector_bone_overlay(vec_px_to_world, cam_affine, overlay)
            else {
                return;
            };
            let Some(cam_affine) = self.fase_vector_guides_and_build(
                tool_preview_bits,
                window_size,
                vec_xf,
                cam_affine,
                overlay,
                viewport,
            ) else {
                return;
            };
            self.fase_vector_tool_handles(vector_active, cam_affine, overlay);
            self.fase_gizmo_suppression_and_field3d_frame(viewport);
            self.fase_field3d_smoke_draw(viewport);
            self.fase_field3d_requests();
            self.fase_hero_paint(viewport);
            let Some(hierarchy_select_intent) =
                self.fase_hierarchy_select_lock(hierarchy_select_intent)
            else {
                return;
            };
            self.fase_recipe_and_asset_verbs(fase_recipe_and_asset_verbs::RecipeVerbIntents {
                catalog_verbs,
                swap_variant,
                apply_added,
                apply_to_level,
                open_asset_browser,
            });
            self.fase_hierarchy_dispatch(
                fase_hierarchy_dispatch::HierarchyIntents {
                    visibility_toggle_row,
                    lock_toggle_row,
                    group_toggle_row,
                    reparent_intent,
                    duplicate_row,
                    duplicate_made,
                    add_child_row,
                    add_root,
                    reset_transform_row,
                    revert_to_master_row,
                    instance_verb_row,
                    instance_verb_stable_id,
                    asset_card_verb,
                    delete_row,
                    hierarchy_row_click,
                    hierarchy_select_intent,
                    rename_seed_row,
                    rename_commit,
                    view_focus_kind,
                },
                window_size,
            );
            let Some(joint_pivot_commit) =
                self.fase_inspector_commits(fase_inspector_commits::InspectorIntents {
                    reimport_entity,
                    transform_edit,
                    visibility_edits,
                    sprite_edits,
                    ordering_edits,
                    sampling_edits,
                    blend_edits,
                    slice_edits,
                    anchor_edits,
                    anim_edits,
                    timer_edits,
                    audio_edits,
                    camera_edits,
                    inspector_queue_dirty,
                    action_edits,
                    physics_edits,
                    visibility_section_edits,
                    name_edit,
                    signal_edit,
                    signal_leave_edit,
                })
            else {
                return;
            };
            self.fase_source_strategy_and_joint_pivot(
                fase_source_strategy_and_joint_pivot::SourceStrategyIntents {
                    sprite_source_change,
                },
                joint_pivot_commit,
            );
            self.fase_component_palette(add_component_for);
            self.fase_sprite_precision_emissive(fase_sprite_precision_emissive::SpriteRowIntents {
                remove_from_sheet_row,
                precision_request,
                emissive_edits,
            });
            self.fase_sheet_verbs(fase_sheet_verbs::SheetIntents {
                pack_sheet_row,
                arrange_sheet_row,
                bake_sheet_row,
                export_sheet_row,
                export_image_row,
            });
            self.fase_physics_edits(fase_physics_edits::PhysicsEditIntents {
                joint_edits,
                wheel_edits,
                player_edits,
                join_draw_arm,
            });
            self.fase_physics_join_rig_bake(fase_physics_join_rig_bake::PhysicsCreateIntents {
                bake_request,
                join_chain,
                rig_now,
                inspector_selection,
            });
            self.fase_autokey();
            self.fase_hierarchy_group_merge(fase_hierarchy_group_merge::HierarchyMergeIntents {
                group_row,
                merge_sprites_row,
                merge_to_layers_row,
            });
            self.fase_use_as_brush(use_as_brush_texture_row, use_as_brush_shape_row);
            self.fase_use_as_paper(use_as_paper_row, use_as_granulation_row);
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
