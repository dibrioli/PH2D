//! **O ÍNDICE das fases do quadro.** O `run_render_frame` chama-as por ordem, os corpos moram
//! nelas, e cada `mod` abaixo é uma linha porque ela existe — este ficheiro cresce uma por FASE e
//! **nunca perde uma**, logo o tecto dele não mede autor nenhum: mede o NÚMERO DE FASES.
//!
//! ⚠️ **A ORDEM é a lei, e o oráculo dela é o texto emendado** (`frame_text::render_frame`), que
//! colhe só `fn fase_*` — uma fase com outro nome desaparece dali **em silêncio**.
//!
//! ⭐ Sem entrada NUMERADA nos tectos de LOC desde a `line/render-bodies` (2026-09-13). ⛔ Em
//! 2026-09-19 ele bateu `602` ao ganhar a fase do tween, e o corte foi a narrativa da Wave 3.1/3.2
//! — uma migração FECHADA, cuja história vive nos handoffs dela.

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
/// ⭐ O que *Reset Transform* quer dizer numa linha — irmão do `hierarchy` pelo tecto de função,
/// cortado por assunto. Ver o cabeçalho dele para o defeito medido que ele cura.
mod hierarchy_reset;
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
/// MEASUREMENT scaffold: onde as fases PANEL e CHROME do `painter_bridge::dispatch` gastam um frame.
#[cfg(test)]
mod measure_bridge_phases;
mod padding_bridge;
pub(crate) mod timeline_bridge;
/// **A AUTORIA de uma chave** — irmão do `timeline_bridge` por teto de LOC (HR-18).
mod timeline_bridge_keys;
#[cfg(test)]
#[path = "timeline_presets_menu_tests.rs"]
mod timeline_presets_menu_tests;
/// Os dois gates da resolução de PRESETS da timeline. ⚠️ A LEI mudou-se para
/// [`ph2d_panel_timeline::presets`] na integração de 2026-09-16 (tecto da shell) e eles FICAM:
/// um deles lê o `default_interp` do `timeline_bridge`, que é costura desta shell.
#[cfg(test)]
#[path = "timeline_presets_tests.rs"]
mod timeline_presets_tests;
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
/// ⭐⭐⭐ A ponte do `SignalActions` (#5) — DESCEU para a família em 19/09; o porquê está no
/// cabeçalho dela. ⛔ A do SOM tentou descer junto e o gate das camadas apanhou-a, e por isso FICA.
use ph2d_app_components::signal_actions_bridge as signal_actions;
/// ⭐⭐⭐ **A CÂMERA DE JOGO** (TOP-20 #7) — a costura entre a lei pura e a vista da shell.
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
/// **O commit da §2 Sprite** — irmão do `inspector_commits` por CAP de LOC.
mod inspector_commits_sprite;
mod inspector_factory;
/// ⭐ **A seção COMPONENT do Inspector** (ADR-0164 / F5) — o que esta cópia tem de diferente
/// da receita, e o gesto que limpa as excepções sem alvo.
pub(crate) mod inspector_instance;
mod inspector_properties;
mod inspector_slice;
/// ⭐⭐⭐ O instantâneo e o dreno da secção STATE MACHINE (TOP-20 #15) — ver o cabeçalho.
mod inspector_statemachine;
/// ⭐⭐⭐ **A secção TAGS** (TOP-20 #9) — o snapshot e o commit, que é o único a tocar em DOIS
/// documentos (o mundo e a árvore de tags).
mod inspector_tags;
mod inspector_timer;
mod inspector_topdown;
/// ⭐⭐⭐ O painel TAGS (TOP-20 #9, W4) — o instantâneo da ÁRVORE e os gestos sobre ela.
mod tags_panel;
// ⭐ A derivação do `MasterPiece` (`master_editing`, F4.6) mudou-se para a
// `ph2d_app_components` em 2026-09-12: é **lei da família das instâncias**, não do laço. O gate do
// anel de objecto vazio (`group_gizmo_view_tests`) continua a acender a receita pela porta de
// VERDADE, hoje escrita `ph2d_app_components::master_editing::mark`.
// ⭐⭐ **A ponte do som de cena DESCEU para a família em 25/09** (catraca `the_shell_only_shrinks`);
// o nome `audio_2d` fica aqui para os três chamadores do quadro continuarem a lê-lo igual.
pub(crate) use ph2d_app_audio::audio_2d::{self, AudioSceneReport};
// ⭐⭐ **A fase da câmera SAIU para a crate da família em 19/09** (catraca `the_shell_only_shrinks`)
// — o relatório dela continua a ser lido aqui pela `fase_game_camera`.
pub(crate) use ph2d_app_components::camera_2d::CameraSceneReport;
#[cfg(test)]
#[path = "master_editing_for_tests.rs"]
pub(crate) mod master_editing_for_tests;
/// ⭐⭐⭐ **A ponte do cérebro autorável** (TOP-20 #15) — ver o cabeçalho dela.
mod state_machine_tick;
/// The joint-anchor point gizmo's publish rule — extracted from `snapshots` so
/// "which entity gets a point handle" is gated headless.
// ⭐ O `point_gizmo` MUDOU-SE para a crate da física (W2/L2 Fase C, handoff daquela fase): o laço
// continua a chamá-lo PELO NOME, que é o que o HOWTO §4 manda — o que sai são os CORPOS.
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

/// ⭐⭐⭐ **A corrente inteira da VIGIA DO CONTADOR**, sem janela — ver o cabeçalho do irmão.
/// ⚠️ Ele mora aqui porque o `signal_actions` e o `timer_tick` são privados a este módulo.
#[cfg(test)]
#[path = "counter_watch_chain_tests.rs"]
mod counter_watch_chain_tests;
/// ⭐⭐⭐ **A ORDEM DO QUADRO do suplente #24** — a metade que FICOU quando a ponte desceu.
#[cfg(test)]
#[path = "fase_signal_outbox_tests.rs"]
mod fase_signal_outbox_tests;
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
mod fase_criar_accao_do_mapa;
/// Fase do quadro: os restos do dreno.
mod fase_drain_leftovers;
/// Fase do quadro: a sincronizacao das entidades e as formas vivas.
mod fase_entity_sync;
/// Fase do quadro: o envelope.
mod fase_envelope;
/// Fase do quadro: os insumos do extract (passo, pré-visualizações, folha aberta, px/m, filtro).
mod fase_extract_inputs;
/// Fase do quadro: o outbox de sinais (os produtores que faltavam e o dreno).
mod fase_fabrica_e_morte;
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
/// Fase do quadro: o meio do prelúdio (os relógios, a timeline, a física, o extract) e o chão do quadro.
mod fase_frame_open;
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
/// Fase do quadro: o ramo do `HeroScreen` — a publicação, o dreno e os quatro blocos que consomem os pedidos.
mod fase_hero_frame;
/// Fase do quadro: a pintura do ecra hero.
mod fase_hero_paint;
/// Fase do quadro: o despacho da Hierarquia.
mod fase_hierarchy_dispatch;
/// Fase do quadro: agrupar, recolher e fundir sprites.
mod fase_hierarchy_group_merge;
/// Fase do quadro: a trava do Painter na seleccao da Hierarquia.
mod fase_hierarchy_select_lock;
mod fase_hud;
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
/// Fase do quadro: os gizmos do Motion que leem o cozido (o colisor e o warp).
mod fase_motion_gizmos;
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
mod fase_paralaxe;
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
mod fase_sequences;
/// Fase do quadro: a manutenção de sessão (Shape Builder, tween, Colorize, Gap Closure).
mod fase_session_upkeep;
/// Fase do quadro: o latch da forma armada e os campos de forma.
mod fase_shape_fields;
/// Fase do quadro: os verbos da folha de sprites.
mod fase_sheet_verbs;
mod fase_signal_log;
mod fase_signal_outbox;
mod fase_sim_extract;
/// Fase do quadro: o extract (propagação, emissão das sprites e a ordem total do quadro).
mod fase_skeleton_drives;
/// Fase do quadro: os verbos do esqueleto.
mod fase_skeleton_verbs;
/// Fase do quadro: a publicação dos instantâneos.
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
/// ⭐⭐⭐ **A TABELA nome → acção** (#5) — fase-filha da `fase_signal_outbox`, por tecto de FUNÇÃO.
mod fase_tabela_de_accoes;
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
mod fase_timeline_key_insert;
/// Fase do quadro: a vista da timeline (amostragem, intents estacionadas, espelhos do painel).
mod fase_timeline_view;
/// Fase do quadro: os espelhos da ferramenta vetorial, do Flip e da fisica.
mod fase_tool_mirrors;
/// Fase do quadro: as operacoes de transformacao.
mod fase_transform_ops;
/// ⭐⭐⭐ **OS TWEENS** (suplente #22) — fase-filha da `fase_signal_outbox`, como a tabela e a
/// fábrica, e pela mesma razão: um `Start Timer` publicado neste quadro tem de mexer NESTE quadro.
mod fase_tweens;
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
mod fase_vector_click_previews;
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
/// O perfilador de fases do quadro (`PH2D_FLUID_PROFILE` / `PH2D_PAINT_PERF`): os contadores e as notas.
mod frame_prof;
/// ⭐⭐⭐ Os dois motores da janela dos cérebros — ver o cabeçalho do módulo.
mod motores_do_quadro;
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
use frame_prof::*;

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::paint::PaintCtx;
use ph2d_editor_core::zones::Rect as EditorRect;
use ph2d_editor_core::{Layout as EditorLayout, RequestedSpriteStrategy, Toast, paint_hero_screen};
use std::time::Instant;

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
        // ⚠️ **O relógio da SIMULAÇÃO** (`PH2D_FLUID_PROFILE`) — ver `frame_prof`. Ela corre
        // `report.ticks` tiques, logo um quadro atrasado paga-a várias vezes.
        let sim_t0 = std::time::Instant::now();
        let Some((report, tool_preview_bits)) = self.fase_frame_simulation(wall_dt, player_input)
        else {
            return;
        };
        {
            let us = sim_t0.elapsed().as_micros() as u64;
            FRAME_PROF_SIM_SUM_US.with(|c| c.set(c.get() + us));
            FRAME_PROF_SIM_MAX_US.with(|c| c.set(c.get().max(us)));
            FRAME_PROF_SIM_N.with(|c| c.set(c.get() + 1));
        }
        let Some(fase_frame_open::FrameCanvas {
            r,
            g,
            b,
            window_size,
            viewport,
            hero,
        }) = self.fase_frame_canvas()
        else {
            return;
        };
        if hero {
            if self
                .fase_hero_frame(
                    diag_input_events,
                    diag_paint_stamps,
                    tool_preview_bits,
                    report,
                    window_size,
                    viewport,
                )
                .is_none()
            {
                return;
            }
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
