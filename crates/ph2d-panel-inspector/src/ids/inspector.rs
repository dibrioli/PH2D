//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/inspector.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 60 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

// ── Inspector Transform editor (M14.A) ──────────────────────────────────────
// Live binding for `ph2d_ecs::Transform` on the selected entity. A seção pinta quando o snapshot
// `current_inspector_transform()` é `Some` e comita pelo barramento de ações
// (`EditorAction::InspectorTransformEdit`).
//
// ⚠️ Este bloco citava `HeroScreen::inspector_transform` e `pending_transform_edit` — **os dois
// campos deixaram de existir** na migração do ADR-0029 Fase C.1, e o texto ficou a descrever uma
// arquitetura morta (auditoria de 2026-08-21, `docs/Sprite_projeto/20` §2.3).
//
// Z is intentionally hidden — `Transform` is 2D by design (SKILL §3,
// ADR-0025); X/Y NumberInputs only, with R/G axis-color labels.
/// Position X NumberInput (meters, R-tinted label).
pub const INSP_TRANSFORM_POS_X: NodeId = hash_node_id("insp_transform_pos_x");

/// Position Y NumberInput (meters, G-tinted label).
pub const INSP_TRANSFORM_POS_Y: NodeId = hash_node_id("insp_transform_pos_y");

/// Rotation NumberInput (displayed in degrees; stored in radians).
pub const INSP_TRANSFORM_ROT: NodeId = hash_node_id("insp_transform_rot");

/// Scale X NumberInput (unitless, R-tinted label).
pub const INSP_TRANSFORM_SCALE_X: NodeId = hash_node_id("insp_transform_scale_x");

/// Scale Y NumberInput (unitless, G-tinted label).
pub const INSP_TRANSFORM_SCALE_Y: NodeId = hash_node_id("insp_transform_scale_y");

/// Skew X NumberInput (degrees in UI, R-tinted label; ADR-0025-amendment-1).
pub const INSP_TRANSFORM_SKEW_X: NodeId = hash_node_id("insp_transform_skew_x");

/// Skew Y NumberInput (degrees in UI, G-tinted label).
pub const INSP_TRANSFORM_SKEW_Y: NodeId = hash_node_id("insp_transform_skew_y");

/// Reset-to-Identity button in the Transform section header.
pub const INSP_TRANSFORM_RESET: NodeId = hash_node_id("insp_transform_reset");

// ── Inspector Visibility checkbox (M14.D) ───────────────────────────────────
// Mirrors the Hierarchy eye toggle (M14.6A). Painted as a single row
// above the Transform section. Click commits via
// `pending_visibility_edit` → `EditorCommand::SetComponent` for the
// `ph2d_ecs::Visibility` component (same pipeline as Transform).
/// Visibility checkbox in the Inspector header strip.
pub const INSP_VISIBILITY_CHECK: NodeId = hash_node_id("insp_visibility_check");

// **O par de PRECISÃO** — plano
// `docs/Sprite_projeto/18` W5.
//
// ⚠️ **Estes dois ids já existiram e foram APAGADOS**, e a história é o motivo de este comentário
// ser longo. Na primeira encarnação eles eram pintados, registados e hit-indexados — e **sem arm de
// dispatch em lado nenhum**: clicar não fazia nada, nem um toast. O aceso era o literal
// `true`/`false`, não estado, então diziam "RGBA8" até para um sprite cozido. O plano 17 §5
// removeu-os e pôs no lugar uma linha de facto.
//
// ⛔ **Ressuscitá-los só é legítimo porque agora existe o MODELO por trás**: o
// `Asset::ImageRgba16`, o `IndividualTextureStore::FORMAT_16`, o `PixelPayload` no ficheiro e a
// conversão nos dois sentidos. *Um controle nasce quando o modelo o entrega, não quando o desenho
// o imagina.* Quem lá voltar a mexer tem três coisas a manter vivas ao mesmo tempo, e o gate
// `every_precision_button_is_registered_and_dispatches` afirma-as: **pintado**, **registado no
// `WidgetStore`** (senão não é focável e o clique morre) e **com braço no `event.rs`**.
pub const INSP_RENDER_FORMAT_RGBA8: NodeId = hash_node_id("insp_render_format_rgba8");

pub const INSP_RENDER_FORMAT_RGBA16: NodeId = hash_node_id("insp_render_format_rgba16");

/// **A SPRITE COMO FONTE DE LUZ** (`docs/Sprite_projeto/18` W8) — o slider `Emissive` + a chip
/// ligada, na mesma secção que o par `Format`.
///
/// ⚠️ **Vizinho do `Format` de propósito.** A emissão é a única coisa que precisa da folga acima de
/// 1.0 que os 16 bits dão, e pô-la aqui é o que faz a ligação aparecer ao artista sem uma palavra de
/// explicação. Ela **funciona** em 8 bits (o multiplicador empurra a cor para cima na hora do
/// desenho); o que 16 bits acrescenta é poder **guardar** o brilho na própria textura.
pub const INSP_SPRITE_EMISSIVE: NodeId = hash_node_id("insp_sprite_emissive");

pub const INSP_SPRITE_EMISSIVE_CHIP: NodeId = hash_node_id("insp_sprite_emissive_chip");

// W3 Sprite Inspector v2 §7 — Ordering / Sorting control ids.
// (INSP_ORDER_Z_OVERRIDE retired 2026-05-31 — the Z Index field IS the
// override: a non-zero value attaches `ZIndexOverride`, 0 detaches it.)
/// Z Index value (NumberInput); 0 = no override (default / DFS order).
pub const INSP_ORDER_Z_INDEX: NodeId = hash_node_id("insp_order_z_index");

/// "Z as Relative" toggle (shown when override on).
pub const INSP_ORDER_Z_RELATIVE: NodeId = hash_node_id("insp_order_z_relative");

/// "Show Behind Parent" marker toggle.
pub const INSP_ORDER_SHOW_BEHIND: NodeId = hash_node_id("insp_order_show_behind");

/// "Order in Layer" value (NumberInput).
pub const INSP_ORDER_ORDER_IN_LAYER: NodeId = hash_node_id("insp_order_order_in_layer");

/// "Y-Sort" enabled toggle.
pub const INSP_ORDER_YSORT_ENABLED: NodeId = hash_node_id("insp_order_ysort_enabled");

/// "Sorting Group" toggle.
pub const INSP_ORDER_SORTING_GROUP: NodeId = hash_node_id("insp_order_sorting_group");

/// "Sort At Root" toggle (shown when Sorting Group on).
pub const INSP_ORDER_SORT_AT_ROOT: NodeId = hash_node_id("insp_order_sort_at_root");

/// "Top Level" marker toggle.
pub const INSP_ORDER_TOP_LEVEL: NodeId = hash_node_id("insp_order_top_level");

/// Sorting Layer dropdown chip.
pub const INSP_ORDER_SORTING_LAYER: NodeId = hash_node_id("insp_order_sorting_layer");

/// Sorting Layer dropdown option ids (one per canonical project layer,
/// index = LayerId). Spec §5.2 default set: Background / Midground /
/// Default / Foreground / UI.
pub const INSP_ORDER_LAYER_OPT: [NodeId; 5] = [
    hash_node_id("insp_order_layer_opt_0"),
    hash_node_id("insp_order_layer_opt_1"),
    hash_node_id("insp_order_layer_opt_2"),
    hash_node_id("insp_order_layer_opt_3"),
    hash_node_id("insp_order_layer_opt_4"),
];

/// Y-Sort Sort Point segmented control items (Center / Pivot / Custom).
pub const INSP_ORDER_SP_CENTER: NodeId = hash_node_id("insp_order_sp_center");

pub const INSP_ORDER_SP_PIVOT: NodeId = hash_node_id("insp_order_sp_pivot");

pub const INSP_ORDER_SP_CUSTOM: NodeId = hash_node_id("insp_order_sp_custom");

/// Y-Sort Custom Axis NumberInputs (shown only when Sort Point = Custom).
pub const INSP_ORDER_AXIS_X: NodeId = hash_node_id("insp_order_axis_x");

pub const INSP_ORDER_AXIS_Y: NodeId = hash_node_id("insp_order_axis_y");

// ─── W3 §8 Visibility-section controls (ClipChildren / Mask / Layer) ───
/// Clip Children segmented: Disabled / ClipOnly / ClipAndDraw (tags 0/1/2).
pub const INSP_VIS_CLIP: [NodeId; 3] = [
    hash_node_id("insp_vis_clip_disabled"),
    hash_node_id("insp_vis_clip_clip_only"),
    hash_node_id("insp_vis_clip_clip_and_draw"),
];

/// Mask Interaction segmented: None / VisibleInside / VisibleOutside (0/1/2).
pub const INSP_VIS_MASK: [NodeId; 3] = [
    hash_node_id("insp_vis_mask_none"),
    hash_node_id("insp_vis_mask_inside"),
    hash_node_id("insp_vis_mask_outside"),
];

/// Mask alpha-cutoff NumberInput (shown when Mask != None).
pub const INSP_VIS_ALPHA_CUTOFF: NodeId = hash_node_id("insp_vis_alpha_cutoff");

/// "Mask Source (Mask2D)" toggle — makes this sprite a mask source.
/// ⭐⭐⭐ **A RANHURA DA TEXTURA** — a linha *Storage* da §3, que recebe uma imagem largada da
/// biblioteca e, ao clique, abre-a (plano `docs/Components/07`, wave B3).
///
/// ⚠️ **O porquê vive com quem o honra**: a affordance em `sections/render_source.rs`, a lei da
/// queda em `crates/ph2d-app-components/src/asset_drop.rs`, e a razão de ela ser um CONTROLO — e não uma zona
/// inerte — no `populate.rs` do painel. *O transporte não é o sítio onde se explica o gesto.*
pub const INSP_RENDER_TEXTURE_SLOT: NodeId = hash_node_id("insp_render_texture_slot");

pub const INSP_VIS_MASK_SOURCE: NodeId = hash_node_id("insp_vis_mask_source");

/// On-Screen Enabler toggle + its Rect2 editor (x/y/w/h, shown when on).
pub const INSP_VIS_ON_SCREEN: NodeId = hash_node_id("insp_vis_on_screen");

pub const INSP_VIS_RECT_X: NodeId = hash_node_id("insp_vis_rect_x");

pub const INSP_VIS_RECT_Y: NodeId = hash_node_id("insp_vis_rect_y");

pub const INSP_VIS_RECT_W: NodeId = hash_node_id("insp_vis_rect_w");

pub const INSP_VIS_RECT_H: NodeId = hash_node_id("insp_vis_rect_h");

/// VisibilityLayer bitmask — 32 checkboxes (4 cols × 8 rows), bit `n`.
pub const INSP_VIS_LAYER_BIT: [NodeId; 32] = [
    hash_node_id("insp_vis_layer_bit_0"),
    hash_node_id("insp_vis_layer_bit_1"),
    hash_node_id("insp_vis_layer_bit_2"),
    hash_node_id("insp_vis_layer_bit_3"),
    hash_node_id("insp_vis_layer_bit_4"),
    hash_node_id("insp_vis_layer_bit_5"),
    hash_node_id("insp_vis_layer_bit_6"),
    hash_node_id("insp_vis_layer_bit_7"),
    hash_node_id("insp_vis_layer_bit_8"),
    hash_node_id("insp_vis_layer_bit_9"),
    hash_node_id("insp_vis_layer_bit_10"),
    hash_node_id("insp_vis_layer_bit_11"),
    hash_node_id("insp_vis_layer_bit_12"),
    hash_node_id("insp_vis_layer_bit_13"),
    hash_node_id("insp_vis_layer_bit_14"),
    hash_node_id("insp_vis_layer_bit_15"),
    hash_node_id("insp_vis_layer_bit_16"),
    hash_node_id("insp_vis_layer_bit_17"),
    hash_node_id("insp_vis_layer_bit_18"),
    hash_node_id("insp_vis_layer_bit_19"),
    hash_node_id("insp_vis_layer_bit_20"),
    hash_node_id("insp_vis_layer_bit_21"),
    hash_node_id("insp_vis_layer_bit_22"),
    hash_node_id("insp_vis_layer_bit_23"),
    hash_node_id("insp_vis_layer_bit_24"),
    hash_node_id("insp_vis_layer_bit_25"),
    hash_node_id("insp_vis_layer_bit_26"),
    hash_node_id("insp_vis_layer_bit_27"),
    hash_node_id("insp_vis_layer_bit_28"),
    hash_node_id("insp_vis_layer_bit_29"),
    hash_node_id("insp_vis_layer_bit_30"),
    hash_node_id("insp_vis_layer_bit_31"),
];

/// W2 Sprite Inspector v2 — Color & Tint section controls.
/// Final opacity Slider (`0..1` storage; renders today via
/// `RenderInstance.opacity`). Paired with [`INSP_SPRITE_OPACITY_CHIP`].
pub const INSP_SPRITE_OPACITY: NodeId = hash_node_id("insp_sprite_opacity");

/// Numeric chip linked to the Opacity slider, displaying `0..100`
/// (percent) via an integer-mapped projection.
pub const INSP_SPRITE_OPACITY_CHIP: NodeId = hash_node_id("insp_sprite_opacity_chip");

/// Tint Fill (silhouette) checkbox; renders today via `flip_uv` bit 2.
pub const INSP_SPRITE_TINT_FILL: NodeId = hash_node_id("insp_sprite_tint_fill");

/// Inherited modulate color swatch (`Sprite::tint`, cascades to
/// children). Clicking opens the shared `INSP_BLENDER_PICKER`
/// targeting this id; the chosen color round-trips through
/// `widget_color(id)` and is dispatched as `SpriteFieldEdit::Tint`.
/// Renders today via `RenderInstance.tint`.
pub const INSP_SPRITE_TINT_SWATCH: NodeId = hash_node_id("insp_sprite_tint_swatch");

/// Local modulate color swatch (`Sprite::self_tint`, does NOT cascade).
/// Same picker mechanism as [`INSP_SPRITE_TINT_SWATCH`]; dispatched as
/// `SpriteFieldEdit::SelfTint`.
pub const INSP_SPRITE_SELF_TINT_SWATCH: NodeId = hash_node_id("insp_sprite_self_tint_swatch");

/// Per-corner tint swatches `[TL, TR, BL, BR]` — a 4-stop bilinear
/// gradient (Phaser-style). Each opens the shared picker; the chosen
/// color replaces one corner of the `[[f32;4];4]` array and dispatches
/// `SpriteFieldEdit::PerCornerTint`. Renders via the shader's
/// `@location(9..12)` per-corner attributes.
pub const INSP_SPRITE_CORNER_TL: NodeId = hash_node_id("insp_sprite_corner_tl");

/// Per-corner tint swatch — top-right. See [`INSP_SPRITE_CORNER_TL`].
pub const INSP_SPRITE_CORNER_TR: NodeId = hash_node_id("insp_sprite_corner_tr");

/// Per-corner tint swatch — bottom-left. See [`INSP_SPRITE_CORNER_TL`].
pub const INSP_SPRITE_CORNER_BL: NodeId = hash_node_id("insp_sprite_corner_bl");

/// Per-corner tint swatch — bottom-right. See [`INSP_SPRITE_CORNER_TL`].
pub const INSP_SPRITE_CORNER_BR: NodeId = hash_node_id("insp_sprite_corner_br");

/// "Equalize corners" button — copies the top-left corner color to the
/// other three (spec §3.6 hotkey); dispatches `SpriteFieldEdit::PerCornerTint`.
pub const INSP_SPRITE_CORNER_EQUALIZE: NodeId = hash_node_id("insp_sprite_corner_equalize");

/// W2 Sprite Inspector v2 — Region sampling controls (Render Source
/// section, spec §3.3). Toggle + 4 px NumberInputs (x/y/w/h) + filter
/// clip. Renders via the extract `region_subrect` sub-UV (W2.T2.4).
pub const INSP_REGION_ENABLED: NodeId = hash_node_id("insp_region_enabled");

/// Region rect X NumberInput (source pixels). See [`INSP_REGION_ENABLED`].
pub const INSP_REGION_X: NodeId = hash_node_id("insp_region_x");

/// Region rect Y NumberInput (source pixels).
pub const INSP_REGION_Y: NodeId = hash_node_id("insp_region_y");

/// Region rect W NumberInput (source pixels, `>= 0`).
pub const INSP_REGION_W: NodeId = hash_node_id("insp_region_w");

/// Region rect H NumberInput (source pixels, `>= 0`).
pub const INSP_REGION_H: NodeId = hash_node_id("insp_region_h");

/// Region filter-clip toggle (anti atlas-bleed). See [`INSP_REGION_ENABLED`].
pub const INSP_REGION_FILTER_CLIP: NodeId = hash_node_id("insp_region_filter_clip");

/// W2 Sprite Inspector v2 — origin controls (Sprite Sheet section, spec
/// §3.4). Centered toggle + Offset X/Y px NumberInputs. Renders via
/// `Sprite::resolve_anchor` (W2.T2.6).
pub const INSP_SPRITE_CENTERED: NodeId = hash_node_id("insp_sprite_centered");

/// Intrinsic offset X NumberInput (px). See [`INSP_SPRITE_CENTERED`].
pub const INSP_SPRITE_OFFSET_X: NodeId = hash_node_id("insp_sprite_offset_x");

/// Intrinsic offset Y NumberInput (px). See [`INSP_SPRITE_CENTERED`].
pub const INSP_SPRITE_OFFSET_Y: NodeId = hash_node_id("insp_sprite_offset_y");

/// W2 Sprite Inspector v2 — Sprite Sheet grid controls (render today via
/// the extract atlas-rect sub-division). HFrames / VFrames / Frame.
pub const INSP_SPRITE_HFRAMES: NodeId = hash_node_id("insp_sprite_hframes");

/// Sprite-sheet rows NumberInput.
pub const INSP_SPRITE_VFRAMES: NodeId = hash_node_id("insp_sprite_vframes");

/// Active sheet frame index NumberInput.
pub const INSP_SPRITE_FRAME: NodeId = hash_node_id("insp_sprite_frame");

/// The creation gesture, and it lives in §11 (Physics Body) rather than here:
/// a joint does not exist yet when you want to make one, so the button has to
/// be somewhere you already are — looking at two bodies you have selected.
pub const INSP_PHYS_JOIN: NodeId = hash_node_id("insp_phys_join");

/// **Arm the canvas drawing gesture** (W-J4) — press a body, drag, release on
/// another, and the joint is born with its anchors AT the two points.
///
/// The sibling route, and the one that puts the anchors where the artist
/// pointed: `INSP_PHYS_JOIN` has no points to offer, so its anchors come from
/// the seed policy (body B's CENTRE for a spring/rope). Both stay — the button
/// is how a CHAIN is made, the gesture is how a placement is made.
pub const INSP_PHYS_JOIN_DRAW: NodeId = hash_node_id("insp_phys_join_draw");

/// **Rig a subárvore selecionada** (W-Rig) — a terceira rota de criação, e a
/// única que não pede que o artista descreva a estrutura de novo.
///
/// As outras duas ligam o que você APONTA: `INSP_PHYS_JOIN` liga uma sequência
/// marcada, `INSP_PHYS_JOIN_DRAW` liga dois corpos por um arrasto. Nenhuma das
/// duas expressa uma ÁRVORE — uma pelve com três filhos não é uma fila —, e a
/// árvore o artista já desenhou na Hierarquia. Este botão a lê: cada aresta
/// pai→filho vira um joint, e cada parte sem corpo ganha um.
pub const INSP_PHYS_RIG: NodeId = hash_node_id("insp_phys_rig");

/// **Add Shape** (W-Compound) — a terceira porta da face vazia: esta forma vira
/// mais um collider do corpo ancestral, em vez de um corpo novo.
pub const INSP_PHYS_ADD_SHAPE: NodeId = hash_node_id("insp_phys_add_shape");

/// **Signal** (W-Signal) — o nome que este objeto GRITA quando algo chega nele.
///
/// Um `TextInput`, porque o valor É uma string: o sinal é um **contrato por
/// nome** (ADR-0143) e quem escuta casa pelo nome, então não há lista de opções
/// a oferecer nem número a arrastar.
///
/// ⚠️ **Mora na §11 ao lado das rows de COLISÃO, não num card próprio:** ele
/// responde *"e daí?"* à pergunta que Trigger/One-Way fazem, e um controle que
/// só faz sentido junto de outros tem de estar onde eles estão (o argumento do
/// `INSP_JOINT_ANCHOR_B_GROUP`, palavra por palavra).
pub const INSP_PHYS_SIGNAL: NodeId = hash_node_id("insp_phys_signal");

/// **Signal on leave** (W-SignalLeave) — o nome que este objeto GRITA quando
/// algo SAI dele.
///
/// ⚠️ **Uma row PRÓPRIA, e não um modo da row acima:** os dois extremos são dois
/// CONTRATOS (`door_open` / `door_close`), e quem escuta casa numa string — um
/// seletor que trocasse o significado do mesmo campo tornaria impossível autorar
/// os dois ao mesmo tempo, que é o caso de uso inteiro.
pub const INSP_PHYS_SIGNAL_LEAVE: NodeId = hash_node_id("insp_phys_signal_leave");
/// ⭐⭐⭐ **O filtro por TAG dos dois sinais acima** — o CHIP do selector (TOP-20 #9, W3c).
pub const INSP_PHYS_SIGNAL_TAG: NodeId = hash_node_id("insp_phys_signal_tag");
/// **Limpa o filtro** — a armadilha volta a valer para todos.
pub const INSP_PHYS_SIGNAL_TAG_CLEAR: NodeId = hash_node_id("insp_phys_signal_tag_clear");

/// §11 join-kind selector, indexed by `JointKind` tag (Pin / Spring / Rope /
/// Weld / Slider). Painted beside *Join Selected Bodies* so the artist creates
/// the joint TYPE they want in one gesture, instead of making a Pin and
/// converting it — and it qualifies the canvas DRAW gesture too.
///
/// ⚠️ **A lista de tipos existe DUAS vezes** — aqui (o tipo que o próximo gesto
/// CRIA) e em [`INSP_JOINT_KIND`] (o tipo que a joint selecionada É) — e as duas
/// têm de conhecer todo `JointKind`. O Slider chegou só na segunda (W-J5), e o
/// resultado foi um tipo que a simulação tinha e o artista **não conseguia
/// escolher** (Enio: *"Slider não aparece no painel de joints"*): o `seg_row` faz
/// `option_ids.zip(labels)`, então o rótulo a mais foi silenciosamente descartado.
/// Há um gate que compara os comprimentos dos DOIS pares.
pub const INSP_PHYS_JOIN_KIND: [NodeId; 9] = [
    hash_node_id("insp_phys_join_kind_pin"),
    hash_node_id("insp_phys_join_kind_spring"),
    hash_node_id("insp_phys_join_kind_rope"),
    hash_node_id("insp_phys_join_kind_weld"),
    hash_node_id("insp_phys_join_kind_slider"),
    hash_node_id("insp_phys_join_kind_rod"),
    hash_node_id("insp_phys_join_kind_wheel"),
    hash_node_id("insp_phys_join_kind_pulley"),
    hash_node_id("insp_phys_join_kind_custom"),
];

// ── Desceu de `ph2d-editor-core/src/ids/inspector.rs` em 2026-09-13 (2.ª passagem: a cerca com a
//    `line/render-loop` prendia-os na fundação até às duas linhas se integrarem).

pub const INSP_RENDER_STRATEGY_INDIVIDUAL: NodeId = hash_node_id("insp_render_strategy_individual");

pub const INSP_RENDER_STRATEGY_HANDPACKED: NodeId = hash_node_id("insp_render_strategy_handpacked");

/// **«Show sheet on canvas»** — a grelha desdobra-se em células fantasma à volta da viva
/// (Enio, 2026-08-23: *«você digita 8 quadros e não vê onde eles começam ou terminam»*).
///
/// ⚠️ **É VISTA, não documento**, e por isso o valor vive só no [`crate::interaction::WidgetStore`]
/// e a shell lê-o direto — sem `EditorAction`, sem commit, sem undo, sem save. Um sprite com grelha
/// desenha UMA célula, então nada no canvas diz onde os cortes caem; a folha aberta é a resposta, e
/// ela é tão transitória quanto o olhar do artista.
///
/// ⛔ Nunca a promova a componente: ela reabriria com o projeto, e o artista veria uma cena que
/// não montou.
pub const INSP_SHEET_PREVIEW: NodeId = hash_node_id("insp_sheet_preview");
