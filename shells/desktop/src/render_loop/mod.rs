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
/// Fase do quadro: o blend e o morph.
mod fase_blend_and_morph;
/// Fase do quadro: o IK e os limites do osso.
mod fase_bone_ik_and_limits;
/// Fase do quadro: os smart bones e os numeros do osso.
mod fase_bone_smart_and_knobs;
/// Fase do quadro: o dreno do barramento e os pedidos do quadro.
mod fase_bus_drain;
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
/// Fase do quadro: os comandos e os knobs do contorno.
mod fase_contour_verbs;
/// Fase do quadro: o converter em curvas.
mod fase_convert_to_curves;
/// Fase do quadro: os restos do dreno.
mod fase_drain_leftovers;
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
/// Fase do quadro: a receita aberta e as vistas do gizmo.
mod fase_gizmo_views_and_prefab;
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
/// Fase do quadro: a activacao da ferramenta de imagem.
mod fase_image_tool_activation;
/// Fase do quadro: as pontes das ferramentas de imagem.
mod fase_image_tool_bridges;
/// Fase do quadro: o modo Image Tools e as pills.
mod fase_image_tools_mode_and_pills;
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
/// Fase do quadro: o Painter: persistir, despachar e medir.
mod fase_painter_dispatch;
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
/// Fase do quadro: a publicacao dos instantaneos.
mod fase_snapshots_publish;
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
/// Fase do quadro: o texto no caminho.
mod fase_text_on_path;
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
/// Fase do quadro: a escala do desenho vectorial.
mod fase_vector_scale;
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
            theme,
            vector_scene,
            hero_screen,
            hero_live,
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

        // Default editor mode: AppGfx owns a HeroScreen with a
        // retained WidgetStore (ADR-0024). Paint reads + writes its
        // hit_index each frame; pointer/key events are forwarded to
        // it from window_event handlers via `hero_screen.handle_*`.
        // `hero_screen` is `None` only under `PH2D_M5_DEMO=1`.
        // ⚠️ A pergunta é só «há `HeroScreen`?»: o último leitor do `hero` foi para a `fase_snapshots_publish` (P7c), e
        // cada fase deste bloco re-deriva o seu.
        if hero_screen.is_some() {
            let Some(tool_preview_bits) = self.fase_snapshots_publish(
                diag_input_events,
                diag_paint_stamps,
                tool_preview_bits,
                window_size,
            ) else {
                return;
            };
            let Some(motion_tool_active) = self.fase_gizmo_views_and_prefab(window_size) else {
                return;
            };
            let Some(fase_bus_drain::DrainOut {
                pending_image_tool_activation,
                visibility_toggle_row,
                lock_toggle_row,
                group_toggle_row,
                reparent_intent,
                duplicate_row,
                duplicate_made,
                add_child_row,
                group_row,
                add_root,
                reset_transform_row,
                revert_to_master_row,
                instance_verb_row,
                instance_verb_stable_id,
                catalog_verbs,
                asset_card_verb,
                delete_row,
                merge_sprites_row,
                pack_sheet_row,
                arrange_sheet_row,
                remove_from_sheet_row,
                bake_sheet_row,
                export_sheet_row,
                export_image_row,
                merge_to_layers_row,
                use_as_brush_texture_row,
                use_as_brush_shape_row,
                use_as_paper_row,
                use_as_granulation_row,
                hierarchy_row_click,
                hierarchy_select_intent,
                rename_seed_row,
                rename_commit,
                view_focus_kind,
                reimport_entity,
                precision_request,
                emissive_edits,
                trim_entities,
                make_square_entities,
                real_size_entities,
                rasterize_entities,
                undo_image_edit,
                pending_vec_bool,
                pending_vec_expand,
                pending_component,
                pending_widget_edit,
                pending_ui_state,
                pending_ui_state_duration,
                pending_ui_spring_toggle,
                pending_ui_spring_knob,
                pending_ui_easing,
                pending_ui_signal_edit,
                pending_ui_signal_name,
                pending_ui_preview_toggle,
                pending_ui_move_all_toggle,
                pending_morph_arrow,
                pending_morph_preview_toggle,
                pending_bool_apply,
                pending_frame_clip,
                pending_bool_shape_op,
                pending_layout_edit,
                pending_anchor_edit,
                pending_resize_box,
                pending_stroke_present,
                pending_layout_field,
                pending_vec_z,
                pending_token_bind,
                pending_frame_preset,
                pending_vec_select_subpath,
                pending_vec_select_same,
                pending_vec_join,
                pending_vec_weld,
                pending_vec_cut,
                pending_vec_symmetry_apply,
                pending_vec_cut_discard,
                pending_vec_reverse,
                pending_vec_average,
                pending_width_preset,
                pending_create_blend,
                pending_reset_spine,
                pending_expand_blend,
                pending_release_blend,
                pending_blend_steps,
                pending_create_morph,
                pending_morph_t,
                pending_create_envelope,
                pending_bone_bind,
                pending_bone_release,
                pending_bone_knob,
                pending_ik_add,
                pending_ik_remove,
                pending_ik_bend,
                pending_ik_knob,
                pending_limit_add,
                pending_limit_remove,
                pending_limit_knob,
                pending_smart_add,
                pending_smart_remove,
                pending_smart_knob,
                pending_smart_clip,
                pending_smart_pick,
                pending_bone_needs_focus,
                osso_selecionado,
                selecao_bits,
                pending_textpath,
                pending_textpath_offset,
                pending_patternpath,
                pending_pp_spacing,
                pending_pp_start,
                pending_pp_end,
                pending_pp_slide,
                pending_pp_offset,
                pending_contour,
                pending_contour_steps,
                pending_contour_d,
                pending_contour_accel,
                pending_contour_join,
                pending_contour_side,
                pending_filter_cmd,
                pending_filter_stop,
                pending_filter_val,
                pending_pp_rotation,
                pending_pp_pick,
                pending_text_pick,
                pending_expand_envelope,
                pending_release_envelope,
                pending_envelope_kind,
                pending_clear_pins,
                pending_envelope_preset,
                pending_envelope_bend,
                pending_fx_add,
                pending_fx_button,
                pending_fx_param,
                pending_fx_apply,
                pending_vec_vertex_kind,
                pending_vec_delete_vertex,
                pending_vec_reorder,
                pending_vec_duplicate,
                pending_vec_flip,
                pending_vec_rotate,
                pending_vec_path_shape,
                pending_vec_toggle_closed,
                pending_vec_pivot_edit,
                pending_vec_fill_kind,
                pending_texpat,
                pending_texpat_source,
                pending_texpat_pick,
                pending_vec_stroke_kind,
                pending_brush_pick,
                pending_brush,
                pending_vec_grad_angle,
                pending_vec_grad_add,
                pending_vec_grad_remove,
                pending_vec_grad_influence,
                pending_vec_grad_jitter,
                pending_vec_grad_add_stop,
                pending_vec_grad_remove_stop,
                pending_vec_align,
                pending_vec_distribute,
                pending_vec_compound,
                pending_vec_fill_rule,
                pending_vec_snap_on,
                pending_vec_snap_path,
                pending_vec_snap_cross,
                pending_vec_snap_guides,
                pending_rulers,
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
                pending_vec_shape_param,
                pending_vec_connector,
                pending_vec_text_size,
                pending_vec_text_weight,
                pending_vec_text_line_height,
                pending_vec_text_tracking,
                pending_vec_text_wrap,
                pending_vec_text_align,
                pending_vec_text_axis,
                pending_vec_font_cycle,
                pending_vec_font_pick,
                pending_vec_font_import,
                pending_vec_convert,
                transform_edit,
                visibility_edits,
                sprite_source_change,
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
                add_component_for,
                swap_variant,
                apply_added,
                apply_to_level,
                open_asset_browser,
                physics_edits,
                joint_edits,
                wheel_edits,
                player_edits,
                bake_request,
                join_chain,
                join_draw_arm,
                rig_now,
                visibility_section_edits,
                name_edit,
                signal_edit,
                signal_leave_edit,
                bgremoval_leftover,
                painter_leftover,
                inspector_selection,
            }) = self.fase_bus_drain()
            else {
                return;
            };
            self.fase_drain_leftovers(fase_drain_leftovers::DrainLeftoversIntents {
                bgremoval_leftover,
                painter_leftover,
            });
            self.fase_image_tool_activation(
                fase_image_tool_activation::ImageToolActivationIntents {
                    pending_image_tool_activation,
                },
            );
            self.fase_image_tools_mode_and_pills();
            let Some(fase_image_tool_bridges::ImageToolBridgesOut {
                padding_apply,
                bgremoval_apply_committed,
                color_equalization_apply,
                equalize_sizes_apply,
                upscale_apply,
            }) = self.fase_image_tool_bridges(window_size)
            else {
                return;
            };
            let Some(painter_apply_committed) = self.fase_painter_dispatch(window_size, viewport)
            else {
                return;
            };
            let Some((vector_active, vec_px_to_world)) = self.fase_vector_scale(window_size) else {
                return;
            };
            self.fase_blend_and_morph(fase_blend_and_morph::BlendAndMorphIntents {
                pending_create_blend,
                pending_expand_blend,
                pending_release_blend,
                pending_create_morph,
                pending_morph_t,
            });
            self.fase_text_on_path(fase_text_on_path::TextOnPathIntents {
                pending_textpath,
                pending_textpath_offset,
                pending_text_pick,
            });
            self.fase_contour_verbs(fase_contour_verbs::ContourVerbsIntents {
                pending_contour,
                pending_contour_steps,
                pending_contour_d,
                pending_contour_accel,
                pending_contour_join,
                pending_contour_side,
            });
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
