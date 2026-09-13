//! **Fase do quadro: O DRENO DO BARRAMENTO** — os pedidos do quadro (`pending_*` e os seus irmãos) declarados, e o
//! `for action in hero.bus.drain()` que os enche, devolvidos juntos (OBRA 2 da `line/render-loop`, 2026-09-13).
//!
//! ⚠️ **Uma fase, e não várias:** o dreno é UM `match` de ~1 700 linhas dentro de um `for`, e um braço não é um
//! statement — partir por braço obrigaria o corpo a escrever `*pedido = …` através de referências, e isso já não é o
//! corpo verbatim. Por isso esta fase leva entradas NUMERADAS nos tectos por função e por ficheiro.

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro, para as fases que os consomem.
pub(super) struct DrainOut {
    pub(super) pending_image_tool_activation: Option<&'static str>,
    pub(super) visibility_toggle_row: Option<NodeId>,
    pub(super) lock_toggle_row: Option<NodeId>,
    pub(super) group_toggle_row: Option<NodeId>,
    pub(super) reparent_intent: Option<ph2d_editor_core::screens::hero::HierReparentIntent>,
    pub(super) duplicate_row: Option<NodeId>,
    pub(super) duplicate_made: Option<(u64, u64)>,
    pub(super) add_child_row: Option<NodeId>,
    pub(super) group_row: Option<(NodeId, bool)>,
    pub(super) add_root: bool,
    pub(super) reset_transform_row: Option<NodeId>,
    pub(super) revert_to_master_row: Option<NodeId>,
    pub(super) instance_verb_row: Option<(NodeId, ph2d_app_components::instance_verbs::Verb)>,
    pub(super) instance_verb_stable_id: Option<(
        u64,
        ph2d_app_components::instance_verbs::Verb,
        Option<[f32; 2]>,
    )>,
    pub(super) catalog_verbs: Vec<ph2d_editor_core::action_bus::CatalogVerb>,
    pub(super) asset_card_verb: Option<(
        ph2d_editor_core::interaction::drag_payload::DragPayload,
        ph2d_editor_core::action_bus::AssetCardAction,
    )>,
    pub(super) delete_row: Option<NodeId>,
    pub(super) merge_sprites_row: Option<NodeId>,
    pub(super) pack_sheet_row: Option<NodeId>,
    pub(super) arrange_sheet_row: Option<NodeId>,
    pub(super) remove_from_sheet_row: Option<NodeId>,
    pub(super) bake_sheet_row: Option<NodeId>,
    pub(super) export_sheet_row: Option<NodeId>,
    pub(super) export_image_row: Option<NodeId>,
    pub(super) merge_to_layers_row: Option<NodeId>,
    pub(super) use_as_brush_texture_row: Option<NodeId>,
    pub(super) use_as_brush_shape_row: Option<NodeId>,
    pub(super) use_as_paper_row: Option<NodeId>,
    pub(super) use_as_granulation_row: Option<NodeId>,
    pub(super) hierarchy_row_click: Option<NodeId>,
    pub(super) hierarchy_select_intent: Option<hierarchy::HierarchySelectIntent>,
    pub(super) rename_seed_row: Option<NodeId>,
    pub(super) rename_commit: Option<(NodeId, String)>,
    pub(super) view_focus_kind: Option<ph2d_editor_core::ViewFocusKind>,
    pub(super) reimport_entity: Option<u64>,
    pub(super) precision_request: Option<(u64, ph2d_color::Precision)>,
    pub(super) emissive_edits: Vec<(u64, f32)>,
    pub(super) trim_entities: Vec<u64>,
    pub(super) make_square_entities: Vec<u64>,
    pub(super) real_size_entities: Vec<u64>,
    pub(super) rasterize_entities: Vec<u64>,
    pub(super) undo_image_edit: bool,
    pub(super) pending_vec_bool: Option<ph2d_vec_boolean::PathfinderOp>,
    pub(super) pending_vec_expand: Option<crate::vec_expand::Expand>,
    pub(super) pending_component: Option<crate::vec_component_edit::ComponentEdit>,
    pub(super) pending_widget_edit: Option<crate::vec_widget_edit::WidgetEdit>,
    pub(super) pending_ui_state: Option<crate::vec_ui_state_edit::UiStateEdit>,
    pub(super) pending_ui_state_duration: Option<f64>,
    pub(super) pending_ui_spring_toggle: bool,
    pub(super) pending_ui_spring_knob: Option<(bool, f64)>,
    pub(super) pending_ui_easing: Option<crate::vec_ui_state_edit::EasingPick>,
    pub(super) pending_ui_signal_edit: Option<crate::vec_ui_state_edit::SignalEdit>,
    pub(super) pending_ui_signal_name: Option<(usize, String)>,
    pub(super) pending_ui_preview_toggle: bool,
    pub(super) pending_ui_move_all_toggle: bool,
    pub(super) pending_morph_arrow: Option<crate::vec_morph_edit::MorphCmd>,
    pub(super) pending_morph_preview_toggle: bool,
    pub(super) pending_bool_apply: bool,
    pub(super) pending_frame_clip: Option<bool>,
    pub(super) pending_bool_shape_op: Option<u8>,
    pub(super) pending_layout_edit: Option<crate::vec_layout_edit::LayoutEdit>,
    pub(super) pending_anchor_edit: Option<crate::vec_anchor_edit::AnchorEdit>,
    pub(super) pending_resize_box: bool,
    pub(super) pending_stroke_present: bool,
    pub(super) pending_layout_field: Option<(crate::vec_layout_edit::LayoutField, f64)>,
    pub(super) pending_vec_z: Option<f64>,
    pub(super) pending_token_bind: Option<(ph2d_ecs::BoundProp, Option<&'static str>)>,
    pub(super) pending_frame_preset: Option<ph2d_tool_vector::frames::DevicePreset>,
    pub(super) pending_vec_select_subpath: bool,
    pub(super) pending_vec_select_same: bool,
    pub(super) pending_vec_join: bool,
    pub(super) pending_vec_weld: bool,
    pub(super) pending_vec_cut: bool,
    pub(super) pending_vec_symmetry_apply: bool,
    pub(super) pending_vec_cut_discard: bool,
    pub(super) pending_vec_reverse: bool,
    pub(super) pending_vec_average: bool,
    pub(super) pending_width_preset: Option<usize>,
    pub(super) pending_create_blend: bool,
    pub(super) pending_reset_spine: bool,
    pub(super) pending_expand_blend: bool,
    pub(super) pending_release_blend: bool,
    pub(super) pending_blend_steps: Option<u32>,
    pub(super) pending_create_morph: bool,
    pub(super) pending_morph_t: Option<f32>,
    pub(super) pending_create_envelope: bool,
    pub(super) pending_bone_bind: bool,
    pub(super) pending_bone_release: Option<crate::skeleton_live::Keep>,
    pub(super) pending_bone_knob: Option<(bool, f64)>,
    pub(super) pending_ik_add: bool,
    pub(super) pending_ik_remove: bool,
    pub(super) pending_ik_bend: Option<ph2d_skeleton::BendSide>,
    pub(super) pending_ik_knob: Option<(IkKnob, f64)>,
    pub(super) pending_limit_add: bool,
    pub(super) pending_limit_remove: bool,
    pub(super) pending_limit_knob: Option<(bool, f64)>,
    pub(super) pending_smart_add: bool,
    pub(super) pending_smart_remove: bool,
    pub(super) pending_smart_knob: Option<(bool, f64)>,
    pub(super) pending_smart_clip: Option<usize>,
    pub(super) pending_smart_pick: bool,
    pub(super) pending_bone_needs_focus: bool,
    pub(super) osso_selecionado: Option<u64>,
    pub(super) selecao_bits: Vec<u64>,
    pub(super) pending_textpath: Option<crate::vec_text_ride::TextPathCmd>,
    pub(super) pending_textpath_offset: Option<f64>,
    pub(super) pending_patternpath: Option<crate::pattern_live::PatternPathCmd>,
    pub(super) pending_pp_spacing: Option<f64>,
    pub(super) pending_pp_start: Option<f64>,
    pub(super) pending_pp_end: Option<f64>,
    pub(super) pending_pp_slide: Option<f64>,
    pub(super) pending_pp_offset: Option<f64>,
    pub(super) pending_contour: Option<crate::contour_live::ContourCmd>,
    pub(super) pending_contour_steps: Option<f64>,
    pub(super) pending_contour_d: Option<f64>,
    pub(super) pending_contour_accel: Option<f64>,
    pub(super) pending_contour_join: Option<u8>,
    pub(super) pending_contour_side: Option<u8>,
    pub(super) pending_filter_cmd: Option<crate::fx_live::FilterHit>,
    pub(super) pending_filter_stop: Option<(usize, u8, f32)>,
    pub(super) pending_filter_val: Option<(crate::fx_live::FilterHit, f64)>,
    pub(super) pending_pp_rotation: Option<f64>,
    pub(super) pending_pp_pick: bool,
    pub(super) pending_text_pick: bool,
    pub(super) pending_expand_envelope: bool,
    pub(super) pending_release_envelope: bool,
    pub(super) pending_envelope_kind: Option<ph2d_ecs::EnvelopeKind>,
    pub(super) pending_clear_pins: bool,
    pub(super) pending_envelope_preset: Option<usize>,
    pub(super) pending_envelope_bend: Option<f64>,
    pub(super) pending_fx_add: Option<usize>,
    pub(super) pending_fx_button: Option<(usize, crate::fx_bridge_dispatch::FxRowAction)>,
    pub(super) pending_fx_param: Option<(usize, usize, f64)>,
    pub(super) pending_fx_apply: bool,
    pub(super) pending_vec_vertex_kind: Option<ph2d_vec_scene::VertexKind>,
    pub(super) pending_vec_delete_vertex: bool,
    pub(super) pending_vec_reorder: Option<ph2d_vec_scene::ZOrder>,
    pub(super) pending_vec_duplicate: bool,
    pub(super) pending_vec_flip: Option<ph2d_vec_scene::FlipAxis>,
    pub(super) pending_vec_rotate: Option<ph2d_vec_scene::Rotate90>,
    pub(super) pending_vec_path_shape: Option<crate::input_dispatch::VecPathShapeOp>,
    pub(super) pending_vec_toggle_closed: bool,
    pub(super) pending_vec_pivot_edit: bool,
    pub(super) pending_vec_fill_kind: Option<crate::input_dispatch::VecFillKind>,
    pub(super) pending_texpat: Option<(
        ph2d_vec_render::PatternSlot,
        crate::texture_pattern_edit::TexPatCmd,
    )>,
    pub(super) pending_texpat_source: Option<ph2d_vec_render::PatternSlot>,
    pub(super) pending_texpat_pick: Option<ph2d_vec_render::PatternSlot>,
    pub(super) pending_vec_stroke_kind: Option<ph2d_panel_vector::StrokePaintKind>,
    pub(super) pending_brush_pick: bool,
    pub(super) pending_brush: Option<crate::vec_stroke_paint::BrushCmd>,
    pub(super) pending_vec_grad_angle: Option<f64>,
    pub(super) pending_vec_grad_add: bool,
    pub(super) pending_vec_grad_remove: bool,
    pub(super) pending_vec_grad_influence: Option<f64>,
    pub(super) pending_vec_grad_jitter: Option<f64>,
    pub(super) pending_vec_grad_add_stop: bool,
    pub(super) pending_vec_grad_remove_stop: bool,
    pub(super) pending_vec_align: Option<crate::input_dispatch::VecAlign>,
    pub(super) pending_vec_distribute: Option<crate::input_dispatch::VecDistribute>,
    pub(super) pending_vec_compound: Option<bool>,
    pub(super) pending_vec_fill_rule: Option<bool>,
    pub(super) pending_vec_snap_on: Option<bool>,
    pub(super) pending_vec_snap_path: Option<bool>,
    pub(super) pending_vec_snap_cross: Option<bool>,
    pub(super) pending_vec_snap_guides: Option<bool>,
    pub(super) pending_rulers: Option<bool>,
    pub(super) pending_vec_opacity: Option<f64>,
    pub(super) pending_vec_blend: Option<u8>,
    pub(super) pending_paint_verb: Option<crate::vec_paint_stack::StackVerb>,
    pub(super) pending_paint_width: Option<f64>,
    pub(super) pending_paint_dx: Option<f64>,
    pub(super) pending_paint_dy: Option<f64>,
    pub(super) pending_paint_dilate: Option<f64>,
    pub(super) pending_paint_join: Option<u8>,
    pub(super) pending_paint_opacity: Option<f64>,
    pub(super) pending_paint_blend: Option<u8>,
    pub(super) pending_vec_transform: Option<(crate::input_dispatch::VecTransformField, f64)>,
    pub(super) pending_vec_vert: Option<(bool, f64)>,
    pub(super) pending_vec_rotate_by: Option<f64>,
    pub(super) pending_vec_shape_param: Option<(ph2d_editor_core::NodeId, f64)>,
    pub(super) pending_vec_connector: Option<(ph2d_editor_core::NodeId, f64)>,
    pub(super) pending_vec_text_size: Option<f64>,
    pub(super) pending_vec_text_weight: Option<f32>,
    pub(super) pending_vec_text_line_height: Option<f64>,
    pub(super) pending_vec_text_tracking: Option<f64>,
    pub(super) pending_vec_text_wrap: Option<Option<f64>>,
    pub(super) pending_vec_text_align: Option<ph2d_vec_text::TextAlign>,
    pub(super) pending_vec_text_axis: Option<(usize, f64)>,
    pub(super) pending_vec_font_cycle: Option<i32>,
    pub(super) pending_vec_font_pick: Option<usize>,
    pub(super) pending_vec_font_import: bool,
    pub(super) pending_vec_convert: bool,
    pub(super) transform_edit: Option<ph2d_editor_core::InspectorTransformInfo>,
    pub(super) visibility_edits: Vec<(u64, bool)>,
    pub(super) sprite_source_change: Option<(u64, RequestedSpriteStrategy)>,
    pub(super) sprite_edits: Vec<(u64, ph2d_editor_core::SpriteFieldEdit)>,
    pub(super) ordering_edits: Vec<(u64, ph2d_editor_core::OrderingFieldEdit)>,
    pub(super) sampling_edits: Vec<(u64, ph2d_editor_core::SamplingFieldEdit)>,
    pub(super) blend_edits: Vec<(u64, ph2d_editor_core::BlendFieldEdit)>,
    pub(super) slice_edits: Vec<(u64, ph2d_editor_core::SliceFieldEdit)>,
    pub(super) anchor_edits: Vec<(u64, ph2d_editor_core::AnchorFieldEdit)>,
    pub(super) anim_edits: Vec<(u64, ph2d_editor_core::AnimFieldEdit)>,
    pub(super) timer_edits: Vec<(u64, ph2d_editor_core::TimerFieldEdit)>,
    pub(super) audio_edits: Vec<(u64, ph2d_editor_core::AudioFieldEdit)>,
    pub(super) camera_edits: Vec<(u64, ph2d_editor_core::CameraFieldEdit)>,
    pub(super) inspector_queue_dirty: bool,
    pub(super) action_edits: Vec<(u64, ph2d_editor_core::ActionFieldEdit)>,
    pub(super) add_component_for: Option<u64>,
    pub(super) swap_variant: Option<(u64, u64)>,
    pub(super) apply_added: Option<u64>,
    pub(super) apply_to_level: Option<(u64, u64)>,
    pub(super) open_asset_browser: bool,
    pub(super) physics_edits: Vec<(u64, ph2d_editor_core::PhysicsFieldEdit)>,
    pub(super) joint_edits: Vec<(u64, ph2d_editor_core::JointFieldEdit)>,
    pub(super) wheel_edits: Vec<(u64, ph2d_editor_core::WheelFieldEdit)>,
    pub(super) player_edits: Vec<(u64, ph2d_editor_core::PlayerFieldEdit)>,
    pub(super) bake_request: Option<Vec<u64>>,
    pub(super) join_chain: bool,
    pub(super) join_draw_arm: bool,
    pub(super) rig_now: bool,
    pub(super) visibility_section_edits: Vec<(u64, ph2d_editor_core::VisibilityFieldEdit)>,
    pub(super) name_edit: Option<ph2d_editor_core::InspectorNameInfo>,
    pub(super) signal_edit: Option<ph2d_editor_core::InspectorNameInfo>,
    pub(super) signal_leave_edit: Option<ph2d_editor_core::InspectorNameInfo>,
    pub(super) bgremoval_leftover: Vec<ph2d_editor_core::action_bus::EditorAction>,
    pub(super) painter_leftover: Vec<ph2d_editor_core::action_bus::EditorAction>,
    pub(super) inspector_selection: Vec<u64>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_bus_drain(&mut self) -> Option<DrainOut> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            surface,
            renderer,
            sim,
            toasts,
            tools,
            flip,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let hero = hero_screen.as_mut()?;
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
        let mut reparent_intent: Option<ph2d_editor_core::screens::hero::HierReparentIntent> = None;
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
        let osso_selecionado = crate::bone_gesture::selected_bone(sim, hero.gizmo.iter_selected());
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
        let mut pending_fx_button: Option<(usize, crate::fx_bridge_dispatch::FxRowAction)> = None;
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
        let mut bgremoval_leftover: Vec<ph2d_editor_core::action_bus::EditorAction> = Vec::new();
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
                        pending_bone_needs_focus |= ph2d_editor_core::ids::needs_focused_bone(*id);
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
                        } else if let Some(i) = ph2d_editor_core::ids::VECTOR_BONE_SMART_CLIP_IDS
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
                            pending_textpath = Some(crate::vec_text_ride::TextPathCmd::Flip(true));
                        } else if *id == ph2d_editor_core::ids::VECTOR_TEXTPATH_FLIP_OFF {
                            pending_textpath = Some(crate::vec_text_ride::TextPathCmd::Flip(false));
                        } else if *id == ph2d_editor_core::ids::VECTOR_PATTERNPATH_LINK {
                            pending_patternpath = Some(crate::pattern_live::PatternPathCmd::Link);
                        } else if *id == ph2d_editor_core::ids::VECTOR_PATTERNPATH_PICK {
                            // Picker: arma; a fonte (o motivo selecionado) é capturada no drain.
                            pending_pp_pick = true;
                        } else if *id == ph2d_editor_core::ids::VECTOR_PATTERNPATH_DETACH {
                            pending_patternpath = Some(crate::pattern_live::PatternPathCmd::Detach);
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
                        } else if let Some(hit) = crate::fx_bridge_dispatch::classify_click(*id) {
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
                            .find(|&i| *id == ph2d_editor_core::ids::vector_envelope_preset_id(i))
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
                        } else if let Some(e) = crate::vec_layout_edit::layout_edit_for_id(*id) {
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
                        } else if let Some(e) = crate::vec_ui_state_edit::ui_state_edit_for_id(*id)
                        {
                            // OS ESTADOS de UI (W7): gravar, mostrar e esquecer uma pose.
                            pending_ui_state = Some(e);
                        } else if let Some(p) = crate::vec_ui_state_edit::easing_pick_for_id(*id) {
                            // **O SELETOR DE CURVA** (W7): a forma e a direcao da transicao.
                            pending_ui_easing = Some(p);
                        } else if let Some(e) = crate::vec_ui_state_edit::signal_edit_for_id(*id) {
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
                        } else if let Some(e) = crate::vec_widget_edit::widget_edit_for_id(*id) {
                            // A PELE por-widget (plano UI/UX W6.2): o componente mora no ECS,
                            // entao o clique e' da shell — o painel so' mostra que tipo esta'
                            // aceso e que verbo faz sentido.
                            pending_widget_edit = Some(e);
                        } else if let Some(e) = crate::vec_anchor_edit::anchor_edit_for_id(*id) {
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
                        } else if let Some(op) = crate::input_dispatch::vec_bool_op_for_id(*id) {
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
                        } else if let Some(order) = crate::input_dispatch::vec_reorder_for_id(*id) {
                            pending_vec_reorder = Some(order);
                        } else if *id == ph2d_editor_core::ids::VECTOR_ARRANGE_DUPLICATE {
                            pending_vec_duplicate = true;
                        } else if let Some(axis) = crate::input_dispatch::vec_flip_for_id(*id) {
                            pending_vec_flip = Some(axis);
                        } else if let Some(dir) = crate::input_dispatch::vec_rotate_for_id(*id) {
                            pending_vec_rotate = Some(dir);
                        } else if let Some(op) = crate::input_dispatch::vec_path_shape_for_id(*id) {
                            pending_vec_path_shape = Some(op);
                        } else if *id == ph2d_editor_core::ids::VECTOR_PIVOT_EDIT {
                            pending_vec_pivot_edit = true;
                        } else if *id == ph2d_editor_core::ids::VECTOR_PATH_CLOSE {
                            pending_vec_toggle_closed = true;
                        } else if let Some(k) = crate::input_dispatch::vec_fill_kind_for_id(*id) {
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
                                    let i =
                                        usize::from(slot == ph2d_vec_render::PatternSlot::Stroke);
                                    self.texpat_lock_aspect[i] = !self.texpat_lock_aspect[i];
                                }
                                // ⭐ O elo dos VÃOS — mesmo desenho, mesmo índice por slot.
                                K::GapLink => {
                                    let i =
                                        usize::from(slot == ph2d_vec_render::PatternSlot::Stroke);
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
                        } else if let Some(d) = crate::input_dispatch::vec_distribute_for_id(*id) {
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
                        pending_bone_needs_focus |= ph2d_editor_core::ids::needs_focused_bone(*id);
                        if let Some(field) = crate::input_dispatch::vec_transform_field_for_id(*id)
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
                            pending_ui_state_duration = Some(*v * ph2d_ui_state::MAX_DURATION_S);
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
                        } else if let Some(f) = crate::vec_layout_edit::layout_field_for_id(*id) {
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
                            pending_vec_text_weight =
                                Some(ph2d_tool_vector::params::slider_to_text_weight(*v as f32)
                                    as f32);
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
                            pending_vec_text_tracking =
                                Some(ph2d_tool_vector::params::slider_to_text_tracking(*v as f32));
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
                        } else if let Some(c) = crate::vec_stroke_paint::slider_cmd_for_id(*id, *v)
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
                                K::Height => Some(crate::texture_pattern_edit::TexPatCmd::Axis(
                                    1, *v, cadeado,
                                )),
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
                                K::Angle => Some(crate::texture_pattern_edit::TexPatCmd::Angle(*v)),
                                K::Offset => {
                                    Some(crate::texture_pattern_edit::TexPatCmd::OffsetDenom(*v))
                                }
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
                    } else if let Some(intent) =
                        timeline_bridge::intent_for_transport(&ev, &self.timeline, &self.playhead)
                    {
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
                EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::Reparent(
                    intent,
                )) => {
                    reparent_intent.get_or_insert(intent);
                }
                EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::Duplicate {
                    row,
                }) => {
                    duplicate_row.get_or_insert(row);
                }
                EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::AddChild {
                    row,
                }) => {
                    add_child_row.get_or_insert(row);
                }
                EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::Group {
                    row,
                }) => {
                    group_row.get_or_insert((row, true));
                }
                EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::Ungroup {
                    row,
                }) => {
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
                EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::Detach {
                    row,
                }) => {
                    instance_verb_row
                        .get_or_insert((row, ph2d_app_components::instance_verbs::Verb::Detach));
                }
                // ⭐⭐ *Remove from Library* pela linha da Hierarquia — o MESMO verbo do cartão,
                // com o outro sujeito. Ele resolve a receita a partir de uma cópia
                // (`instance_unmake::recipe_root_of`), que é o que torna esta porta útil.
                EditorAction::Hierarchy(
                    ph2d_editor_core::action_bus::HierRequest::RemoveFromLibrary { row },
                ) => {
                    instance_verb_row
                        .get_or_insert((row, ph2d_app_components::instance_verbs::Verb::Unmake));
                }
                EditorAction::Hierarchy(
                    ph2d_editor_core::action_bus::HierRequest::ApplyToMaster { row },
                ) => {
                    instance_verb_row
                        .get_or_insert((row, ph2d_app_components::instance_verbs::Verb::Apply));
                }
                EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::Delete {
                    row,
                }) => {
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
                EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::PackSheet {
                    row,
                }) => {
                    pack_sheet_row.get_or_insert(row);
                }
                EditorAction::Hierarchy(
                    ph2d_editor_core::action_bus::HierRequest::ArrangeSheet { row },
                ) => {
                    arrange_sheet_row.get_or_insert(row);
                }
                EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::BakeSheet {
                    row,
                }) => {
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
                EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::RowClick {
                    row,
                }) => {
                    hierarchy_row_click.get_or_insert(row);
                }
                // Fase 0e: multi-select-aware hierarchy click +
                // shift-range. Collect into a single latest-wins
                // intent — the dispatch resolves row → entity_bits
                // and applies the matching `GizmoStateGroup`
                // mutation. Range overrides Row when both arrive
                // in the same frame (the user can only be in one
                // selection-gesture at a time).
                EditorAction::Hierarchy(ph2d_editor_core::action_bus::HierRequest::SelectRow {
                    row,
                    modifier,
                }) if !matches!(
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
                    hierarchy_select_intent = Some(hierarchy::HierarchySelectIntent::Range { row });
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
                    if ph2d_app_components::instance_structure::restore_piece(sim, root_bits, piece)
                    {
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
                        self.bake_channels = ph2d_app_physics::bake::BakeChannels::from_tag(tag);
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
        Some(DrainOut {
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
        })
    }
}
