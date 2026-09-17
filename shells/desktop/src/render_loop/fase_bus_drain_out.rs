//! **O VALOR dos pedidos do quadro** — o [`DrainOut`], irmão de [`super::fase_bus_drain`]
//! pelo tecto de 600 LOC (HR-18).
//!
//! ⚠️ **O corte é o que o cabeçalho do irmão já declarava por escrito:** *«os pedidos são UM
//! valor e os braços são sub-drenos»* — aqui o valor (um campo por pedido, com o comentário
//! dele ao lado); lá a fase que o enche. O ficheiro passou o tecto por ACUMULAÇÃO na rodada
//! de 16/09 (`CLAUDE.md` §5.0).

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro, para as fases que os consomem.
///
/// ⚠️ **O `Default` é o valor com que cada pedido nascia** (`None` · `false` · `Vec::new()`). Três nascem de uma
/// LEITURA do quadro — `osso_selecionado`, `selecao_bits` e `inspector_selection` — e moram no `let mut pd` da
/// `fase_bus_drain`, com a razão ao lado.
#[derive(Default)]
pub(in crate::render_loop) struct DrainOut {
    pub(in crate::render_loop) pending_image_tool_activation: Option<&'static str>,
    pub(in crate::render_loop) visibility_toggle_row: Option<NodeId>,
    pub(in crate::render_loop) lock_toggle_row: Option<NodeId>,
    pub(in crate::render_loop) group_toggle_row: Option<NodeId>,
    pub(in crate::render_loop) reparent_intent:
        Option<ph2d_editor_core::screens::hero::HierReparentIntent>,
    pub(in crate::render_loop) duplicate_row: Option<NodeId>,
    // Set by `hierarchy::dispatch` to `(source_bits, new_bits)` when a sprite is duplicated, so
    // we can fork the copy onto its own texture (independent object) post-dispatch.
    pub(in crate::render_loop) duplicate_made: Option<(u64, u64)>,
    pub(in crate::render_loop) add_child_row: Option<NodeId>,
    // ⭐⭐ **Agrupar / desagrupar** (2026-08-30): `(linha clicada, agrupar?)`. Um slot só para
    // os dois verbos — eles são o mesmo gesto com o sinal trocado, e dois slots deixariam
    // a porta aberta a alguém drenar os dois no mesmo quadro.
    pub(in crate::render_loop) group_row: Option<(NodeId, bool)>,
    // ⭐ **O `Add` do cabeçalho da Hierarquia** (ADR-0166 / F3) — um objeto vazio na raiz.
    // Sem payload: ele não sai de uma linha, e por isso não tem pai (ver `HierAddRoot`).
    pub(in crate::render_loop) add_root: bool,
    pub(in crate::render_loop) reset_transform_row: Option<NodeId>,
    // ⭐ *Revert to Master* (ADR-0164 / F4.4) — a linha cuja instância volta à receita.
    pub(in crate::render_loop) revert_to_master_row: Option<NodeId>,
    // ⭐ Os outros verbos de instância (ADR-0164 / F4.5) — UM slot, porque eles são
    // exclusivos por construção: o menu fecha ao primeiro clique.
    pub(in crate::render_loop) instance_verb_row:
        Option<(NodeId, ph2d_app_components::instance_verbs::Verb)>,
    // ⭐ O mesmo verbo, endereçado por `StableId` — o canal do navegador de assets.
    pub(in crate::render_loop) instance_verb_stable_id: Option<(
        u64,
        ph2d_app_components::instance_verbs::Verb,
        Option<[f32; 2]>,
    )>,
    // ⭐⭐ O menu de um CARTÃO da biblioteca (etapa C) — o par `(endereço, verbo)` que o
    // painel transporta. ⚠️ **Slot próprio, e não o `instance_verb_stable_id`:** metade
    // das seis células é uma RECUSA que só o shell sabe redigir (o número de utilizadores
    // de uma imagem), e dobrá-lo no slot dos verbos de instância obrigaria a inventar um
    // `Verb` para *«não faça nada e diga porquê»*.
    // ⭐⭐ Os verbos de CATÁLOGO (wave A3). ⚠️ **Um `Vec`, e não um slot único**: ao
    // contrário dos verbos de instância, dois destes PODEM chegar no mesmo quadro sem
    // conflito (criar e escolher, por exemplo) — e eles não competem por um sujeito.
    pub(in crate::render_loop) catalog_verbs: Vec<ph2d_editor_core::action_bus::CatalogVerb>,
    pub(in crate::render_loop) asset_card_verb: Option<(
        ph2d_editor_core::interaction::drag_payload::DragPayload,
        ph2d_editor_core::action_bus::AssetCardAction,
    )>,
    pub(in crate::render_loop) delete_row: Option<NodeId>,
    // Enio 2026-05-27: right-click → Merge Sprites in Hierarchy.
    // Carries the clicked row's `NodeId` (the merged sprite
    // adopts that row's parent for Hierarchy placement); the
    // drain reads the full multi-selection at apply time.
    pub(in crate::render_loop) merge_sprites_row: Option<NodeId>,
    // "Pack into Sheet" do menu de contexto da hierarquia — a 2ª porta do verbo do pill
    // `[SHEET]`. Guarda a LINHA (não a entidade): quem a resolve é o `bridge`, no dreno.
    pub(in crate::render_loop) pack_sheet_row: Option<NodeId>,
    // "Auto-Arrange Pieces" — re-encaixar os filhos de uma folha que já existe.
    pub(in crate::render_loop) arrange_sheet_row: Option<NodeId>,
    // "Remove from Sheet" — a saída da folha, pela linha clicada.
    pub(in crate::render_loop) remove_from_sheet_row: Option<NodeId>,
    // As duas saídas do BAKE (plano §7.3, W5.2): assar muda a cena, exportar escreve
    // ficheiros. Linhas separadas porque são dois pedidos diferentes.
    pub(in crate::render_loop) bake_sheet_row: Option<NodeId>,
    pub(in crate::render_loop) export_sheet_row: Option<NodeId>,
    // **EXPORTAR UMA SPRITE** (plano `docs/Sprite_projeto/18` W9) — irmão do de cima, e a
    // diferença está no nome: aquele escreve a FOLHA, este escreve uma sprite no formato
    // que a extensão escolhida nomear.
    pub(in crate::render_loop) export_image_row: Option<NodeId>,
    // **FUNDIR EM CAMADAS** (plano `docs/Sprite_projeto/18` W10) — a mesma geometria do
    // Merge, e cada fonte fica também numa camada do documento do Painter.
    pub(in crate::render_loop) merge_to_layers_row: Option<NodeId>,
    pub(in crate::render_loop) use_as_brush_texture_row: Option<NodeId>,
    pub(in crate::render_loop) use_as_brush_shape_row: Option<NodeId>,
    pub(in crate::render_loop) use_as_paper_row: Option<NodeId>,
    pub(in crate::render_loop) use_as_granulation_row: Option<NodeId>,
    pub(in crate::render_loop) hierarchy_row_click: Option<NodeId>,
    pub(in crate::render_loop) hierarchy_select_intent: Option<hierarchy::HierarchySelectIntent>,
    pub(in crate::render_loop) rename_seed_row: Option<NodeId>,
    pub(in crate::render_loop) rename_commit: Option<(NodeId, String)>,
    pub(in crate::render_loop) view_focus_kind: Option<ph2d_editor_core::ViewFocusKind>,
    pub(in crate::render_loop) reimport_entity: Option<u64>,
    // O pedido de troca de PRECISAO (plano `docs/Sprite_projeto/18` W5). `Option` e nao
    // `Vec`: o par so' existe com uma sprite selecionada.
    pub(in crate::render_loop) precision_request: Option<(u64, ph2d_color::Precision)>,
    // **A SPRITE COMO FONTE DE LUZ** (plano `docs/Sprite_projeto/18` W8). Recolhido aqui e
    // drenado com o irmão `precision_request` — o mesmo padrão, porque o componente só pode
    // ser escrito onde o `sim` está emprestado mutavelmente.
    // ⚠️ **Um Vec, não um `Option`** — a emissão é uma edição de campo como a Opacidade, e
    // a Opacidade espalha-se pela seleção. Enquanto isto foi `Option<(u64, f32)>` o slider
    // parecia um bulk edit e mudava **uma** sprite (auditoria `docs/Sprite_projeto/20` §3).
    pub(in crate::render_loop) emissive_edits: Vec<(u64, f32)>,
    // Fase 0e: per-sprite tools collect a Vec<u64> instead of
    // Option<u64> so a multi-select OneShotImageOp broadcast
    // applies the bake to every selected sprite (legacy
    // single-select still works — the Vec just carries one
    // entry). image_edit::dispatch iterates each Vec.
    pub(in crate::render_loop) trim_entities: Vec<u64>,
    pub(in crate::render_loop) make_square_entities: Vec<u64>,
    pub(in crate::render_loop) real_size_entities: Vec<u64>,
    pub(in crate::render_loop) rasterize_entities: Vec<u64>,
    // ⚠️ **Este NÃO é por-sprite, e é a exceção da fila.** A chrome emite um
    // `OneShotImageOp` por entidade selecionada; os irmãos aplicam o bake a cada um
    // isoladamente, e este junta a leva inteira para criar **uma** folha. N atos
    // independentes dariam N folhas de uma peça cada — um verbo que fala da RELAÇÃO
    // entre as peças não cabe num evento por peça.
    pub(in crate::render_loop) undo_image_edit: bool,
    // ADR-0108 Fase 1: a Boolean button (Union/Subtract/Intersect) in the
    // docked Vector panel forwards a `ToolPanelEvent::Click`; the op acts
    // on the DOCUMENT (shell-owned `vec_scene`), not the tool's Style, so
    // capture it here and apply after the drain (mirror of the U/I/D
    // hotkeys, next to the vector render).
    pub(in crate::render_loop) pending_vec_bool: Option<ph2d_vec_boolean::PathfinderOp>,
    pub(in crate::render_loop) pending_vec_expand: Option<crate::vec_expand::Expand>,
    // OS COMPONENTES (plano UI/UX W5): o verbo pedido neste frame.
    pub(in crate::render_loop) pending_component: Option<crate::vec_component_edit::ComponentEdit>,
    pub(in crate::render_loop) pending_widget_edit: Option<crate::vec_widget_edit::WidgetEdit>,
    // OS ESTADOS de UI (plano UI/UX W7): a tabela mora no documento, entao o clique e' da
    // shell — o painel so' mostra que verbos fazem sentido agora.
    pub(in crate::render_loop) pending_ui_state: Option<crate::vec_ui_state_edit::UiStateEdit>,
    pub(in crate::render_loop) pending_ui_state_duration: Option<f64>,
    // ⚠️ Um TOGGLE não traz valor: o pedido é *"inverta"*, e quem sabe o estado atual é
    // a tabela. Um `Some(bool)` obrigaria a shell a lê-la duas vezes.
    pub(in crate::render_loop) pending_ui_spring_toggle: bool,
    // (é a rigidez?, valor) — só o knob que o artista arrastou.
    pub(in crate::render_loop) pending_ui_spring_knob: Option<(bool, f64)>,
    pub(in crate::render_loop) pending_ui_easing: Option<crate::vec_ui_state_edit::EasingPick>,
    // ⭐ **A TABELA SINAL → PAPEL** (item 4 do estudo dos contêineres): os três gestos de
    // clique e o COMMIT do nome. Duas variáveis porque são dois canais do barramento —
    // o `Click` e o `SelectOption`, que é o único variante do `PanelEvent` (contrato
    // CONGELADO) que carrega uma string.
    pub(in crate::render_loop) pending_ui_signal_edit: Option<crate::vec_ui_state_edit::SignalEdit>,
    pub(in crate::render_loop) pending_ui_signal_name: Option<(usize, String)>,
    pub(in crate::render_loop) pending_ui_preview_toggle: bool,
    pub(in crate::render_loop) pending_ui_move_all_toggle: bool,
    // **A BOOLEANA VIVA** (plano UI/UX W1): o Apply consolida o que o produtor cozinhou
    // NESTE frame, então ele não pode correr aqui — corre logo depois do `recook`, onde o
    // plano existe. Aqui só se anota o clique.
    pub(in crate::render_loop) pending_morph_arrow: Option<crate::vec_morph_edit::MorphCmd>,
    pub(in crate::render_loop) pending_morph_preview_toggle: bool,
    pub(in crate::render_loop) pending_bool_apply: bool,
    // A MOLDURA (plano UI/UX W0): o chip de recorte e o preset de dispositivo.
    pub(in crate::render_loop) pending_frame_clip: Option<bool>,
    // **O VERBO DA FORMA selecionada** dentro de uma booleana viva (2026-08-22).
    // Irmao exacto do `pending_frame_clip`, e pelo mesmo motivo: o valor mora num
    // COMPONENTE, entao quem escreve e' a shell — o painel so' mostra qual chip
    // esta' aceso.
    pub(in crate::render_loop) pending_bool_shape_op: Option<u8>,
    // O AUTO LAYOUT (plano UI/UX W2, ADR-0153): um chip de radio e um campo numerico.
    pub(in crate::render_loop) pending_layout_edit: Option<crate::vec_layout_edit::LayoutEdit>,
    pub(in crate::render_loop) pending_anchor_edit: Option<crate::vec_anchor_edit::AnchorEdit>,
    // **Resize Box** (plano UI/UX W3b): o clique e' um TOGGLE, entao nao ha' operando —
    // um bool basta para dizer *"houve clique"*.
    pub(in crate::render_loop) pending_resize_box: bool,
    // ⭐ **Stroke** (plano 34): a caixa que dá/tira o traço da forma selecionada. Também é
    // um TOGGLE, então um bool basta — o operando é a ficha da ferramenta, e ela não viaja.
    pub(in crate::render_loop) pending_stroke_present: bool,
    pub(in crate::render_loop) pending_layout_field:
        Option<(crate::vec_layout_edit::LayoutField, f64)>,
    // **O Z-INDEX global** (Enio, 2026-08-04): o numero que sobrepoe a ordem da
    // hierarquia. Campo numerico, entao a rota e' a mesma do Transform.
    pub(in crate::render_loop) pending_vec_z: Option<f64>,
    // **O TOKEN escolhido no picker** (plano UI/UX W4): a propriedade + o token, ou
    // `None` no token = SOLTAR (a propriedade volta ao literal do documento).
    pub(in crate::render_loop) pending_token_bind:
        Option<(ph2d_ecs::BoundProp, Option<&'static str>)>,
    pub(in crate::render_loop) pending_frame_preset: Option<ph2d_tool_vector::frames::DevicePreset>,
    // **A ESCALA da seleção de nós** (plano 25 §6, W3b): os dois alcances que o retângulo
    // não dá. Não são edições de documento — só mudam QUEM está selecionado —, então não
    // abrem passo de undo (o `post_frame_undo` compara o ESTADO, e a seleção não é dele).
    pub(in crate::render_loop) pending_vec_select_subpath: bool,
    pub(in crate::render_loop) pending_vec_select_same: bool,
    // **As três da W4** (plano 25 §7). Ao contrário das duas acima, estas MUDAM o
    // documento — logo abrem passo de undo, e cada uma abre exatamente um.
    pub(in crate::render_loop) pending_vec_join: bool,
    // ⭐⭐⭐ **Soldar** (plano 39): os traços seleccionados partem-se nos cruzamentos.
    pub(in crate::render_loop) pending_vec_weld: bool,
    pub(in crate::render_loop) pending_vec_cut: bool,
    pub(in crate::render_loop) pending_vec_symmetry_apply: bool,
    pub(in crate::render_loop) pending_vec_cut_discard: bool,
    pub(in crate::render_loop) pending_vec_reverse: bool,
    pub(in crate::render_loop) pending_vec_average: bool,
    // O índice do perfil nomeado que o clique pediu (W2b), se algum.
    pub(in crate::render_loop) pending_width_preset: Option<usize>,
    // ADR-0128: o botão "Blend" cria um Blend Object VIVO da seleção; o slider Steps
    // ajusta o blend selecionado ao vivo. (O destrutivo `vec_blend::apply` sobrevive só
    // para os smokes — o painel não o alcança mais.)
    pub(in crate::render_loop) pending_create_blend: bool,
    pub(in crate::render_loop) pending_reset_spine: bool,
    pub(in crate::render_loop) pending_expand_blend: bool,
    pub(in crate::render_loop) pending_release_blend: bool,
    pub(in crate::render_loop) pending_blend_steps: Option<u32>,
    pub(in crate::render_loop) pending_create_morph: bool,
    pub(in crate::render_loop) pending_morph_t: Option<f32>,
    // ADR-0129: o botão "Envelope" envolve a seleção numa gaiola (container); Expand
    // materializa a deformada e Release ressuscita a fonte autorada — os dois dissolvem.
    pub(in crate::render_loop) pending_create_envelope: bool,
    // ⭐⭐⭐ O ESQUELETO (estudo 42 item 5): prender a selecção aos ossos, as duas saídas, e
    // os dois números do osso em foco (`true` = a força, `false` = o comprimento).
    pub(in crate::render_loop) pending_bone_bind: bool,
    pub(in crate::render_loop) pending_bone_release: Option<crate::skeleton_live::Keep>,
    pub(in crate::render_loop) pending_bone_knob: Option<(ph2d_app_skeleton::knobs::BoneKnob, f64)>,
    pub(in crate::render_loop) pending_ik_add: bool,
    pub(in crate::render_loop) pending_ik_remove: bool,
    // ⭐ O lado da dobra que o artista escolheu neste quadro, se escolheu.
    pub(in crate::render_loop) pending_ik_bend: Option<ph2d_skeleton::BendSide>,
    /// ⭐ De onde vêm as duas alças de CURVATURA do osso em foco (F8, 2026-09-16).
    pub(in crate::render_loop) pending_bone_handles: Option<ph2d_skeleton::bend::Handles>,
    /// A LINHA escolhida no selector de ponta da curva — o índice na lista publicada neste quadro.
    pub(in crate::render_loop) pending_bone_tip: Option<usize>,
    pub(in crate::render_loop) pending_ik_knob: Option<(IkKnob, f64)>,
    pub(in crate::render_loop) pending_limit_add: bool,
    pub(in crate::render_loop) pending_limit_remove: bool,
    // ⚠️ `(é o MAX?, valor em GRAUS)` — a conversão para radianos é feita onde ele é
    // escrito, que é a porta onde as duas unidades se encontram.
    pub(in crate::render_loop) pending_limit_knob: Option<(bool, f64)>,
    pub(in crate::render_loop) pending_smart_add: bool,
    pub(in crate::render_loop) pending_smart_remove: bool,
    pub(in crate::render_loop) pending_smart_knob: Option<(bool, f64)>,
    // A acção escolhida no selector do osso inteligente — o ÍNDICE na lista de clips que o
    // painel pinta; o que se guarda no componente é o NOME dela.
    pub(in crate::render_loop) pending_smart_clip: Option<usize>,
    // O *Pick Object* foi carregado — arma o gesto de duas mãos do alvo.
    pub(in crate::render_loop) pending_smart_pick: bool,
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
    pub(in crate::render_loop) pending_bone_needs_focus: bool,
    pub(in crate::render_loop) osso_selecionado: Option<u64>,
    pub(in crate::render_loop) selecao_bits: Vec<u64>,
    pub(in crate::render_loop) pending_textpath: Option<crate::vec_text_ride::TextPathCmd>,
    pub(in crate::render_loop) pending_textpath_offset: Option<f64>,
    // Pattern on Path (plano 23): o comando de vínculo + os dois sliders, drenados como os
    // do texto (o motivo é o PRIMÁRIO, o guia é o outro selecionado).
    pub(in crate::render_loop) pending_patternpath: Option<crate::pattern_live::PatternPathCmd>,
    pub(in crate::render_loop) pending_pp_spacing: Option<f64>,
    pub(in crate::render_loop) pending_pp_start: Option<f64>,
    pub(in crate::render_loop) pending_pp_end: Option<f64>,
    pub(in crate::render_loop) pending_pp_slide: Option<f64>,
    pub(in crate::render_loop) pending_pp_offset: Option<f64>,
    // Contour (pesquisa `20_*` #9): os três comandos + os três sliders + os dois trios
    // exclusivos. `Add`/`Remove` são portas do MODELO (armam/tiram o componente),
    // `Expand` materializa; os knobs só editam o que já existe.
    pub(in crate::render_loop) pending_contour: Option<crate::contour_live::ContourCmd>,
    pub(in crate::render_loop) pending_contour_steps: Option<f64>,
    pub(in crate::render_loop) pending_contour_d: Option<f64>,
    pub(in crate::render_loop) pending_contour_accel: Option<f64>,
    pub(in crate::render_loop) pending_contour_join: Option<u8>,
    pub(in crate::render_loop) pending_contour_side: Option<u8>,
    // Filters (FX raster, plano 24). `Some(Some(k))` arma o tipo `k`; `Some(None)` remove.
    // Filters (a PILHA de FX raster, plano 24): um comando (Add/✕/↑/↓/👁) e um valor de
    // slider por frame, decodificados pela porta única `fx_live::hit_of`.
    pub(in crate::render_loop) pending_filter_cmd: Option<crate::fx_live::FilterHit>,
    // O arrasto de um punho da rampa: `(linha, índice de AUTORIA do stop, posição 0..1)`.
    // Um `pending`, como o `FilterHit`, e pelo MESMO motivo: a edição do documento mora no
    // bloco que tem o `sim` em mãos, e o drain do barramento não o tem.
    pub(in crate::render_loop) pending_filter_stop: Option<(usize, u8, f32)>,
    pub(in crate::render_loop) pending_filter_val: Option<(crate::fx_live::FilterHit, f64)>,
    pub(in crate::render_loop) pending_pp_rotation: Option<f64>,
    // O Picker de guia (Enio 2026-07-23): o botão só ARMA — a shell captura a fonte e o
    // clique seguinte no canvas escolhe o guia. Um por feature; a fonte é resolvida no drain.
    pub(in crate::render_loop) pending_pp_pick: bool,
    pub(in crate::render_loop) pending_text_pick: bool,
    pub(in crate::render_loop) pending_expand_envelope: bool,
    pub(in crate::render_loop) pending_release_envelope: bool,
    // O GESTO do envelope (ADR-0129 Fatias D+E): Perspective (projetivo) · Mesh (Coons) ·
    // Pins (MLS). Um enum e nao um bool desde que o 3o gesto entrou.
    pub(in crate::render_loop) pending_envelope_kind: Option<ph2d_ecs::EnvelopeKind>,
    pub(in crate::render_loop) pending_clear_pins: bool,
    // O PRESET de gaiola (ADR-0129 Fatia C): indice em `EnvelopeWarp::ALL`, e o Bend.
    pub(in crate::render_loop) pending_envelope_preset: Option<usize>,
    pub(in crate::render_loop) pending_envelope_bend: Option<f64>,
    // ADR-0132: a pilha de efeitos. Um clique num BOTAO (add/remove/up/down/toggle) e
    // um arrasto num slider -- os dois enderecados por (linha, parametro), sem que este
    // arquivo saiba que efeitos existem.
    pub(in crate::render_loop) pending_fx_add: Option<usize>,
    pub(in crate::render_loop) pending_fx_button:
        Option<(usize, crate::fx_bridge_dispatch::FxRowAction)>,
    pub(in crate::render_loop) pending_fx_param: Option<(usize, usize, f64)>,
    // ADR-0132: o "Apply" assa a pilha de efeitos no cozido e a esvazia (Expand Appearance).
    pub(in crate::render_loop) pending_fx_apply: bool,
    // ADR-0108 Fase 1: a Vertex button (Corner/Smooth/Symmetric) retypes
    // the selected vertex — a document edit, applied after the drain.
    pub(in crate::render_loop) pending_vec_vertex_kind: Option<ph2d_vec_scene::VertexKind>,
    // ADR-0108 Fase 1: "Delete Node" button removes the selected vertex.
    pub(in crate::render_loop) pending_vec_delete_vertex: bool,
    // ADR-0108: Arrange buttons — z-order restack + Duplicate + Flip H/V —
    // act on the selected path (document ops), applied after the drain.
    pub(in crate::render_loop) pending_vec_reorder: Option<ph2d_vec_scene::ZOrder>,
    pub(in crate::render_loop) pending_vec_duplicate: bool,
    pub(in crate::render_loop) pending_vec_flip: Option<ph2d_vec_scene::FlipAxis>,
    pub(in crate::render_loop) pending_vec_rotate: Option<ph2d_vec_scene::Rotate90>,
    pub(in crate::render_loop) pending_vec_path_shape:
        Option<crate::input_dispatch::VecPathShapeOp>,
    pub(in crate::render_loop) pending_vec_toggle_closed: bool,
    pub(in crate::render_loop) pending_vec_pivot_edit: bool,
    pub(in crate::render_loop) pending_vec_fill_kind: Option<crate::input_dispatch::VecFillKind>,
    // A lei do PADRÃO de textura (plano 33 W5). Uma só por quadro: os controles da secção
    // são exclusivos entre si (o artista mexe num de cada vez), e uma fila daria dois passos
    // de undo para um gesto.
    // ⚠️ **Cada um leva o SUJEITO junto** (plano 35, wave F): o slot sai do id do controlo
    // que foi clicado, e não de uma preferência guardada — *o que o gesto endereça não pode
    // ser lido de outro sítio no drain.*
    pub(in crate::render_loop) pending_texpat: Option<(
        ph2d_vec_render::PatternSlot,
        crate::texture_pattern_edit::TexPatCmd,
    )>,
    pub(in crate::render_loop) pending_texpat_source: Option<ph2d_vec_render::PatternSlot>,
    pub(in crate::render_loop) pending_texpat_pick: Option<ph2d_vec_render::PatternSlot>,
    // ⭐ A TINTA do traço (plano 35, wave D) — irmã do `pending_vec_fill_kind`, e drenada no
    // MESMO sítio, porque as duas podem precisar de abrir o diálogo da arte.
    pub(in crate::render_loop) pending_vec_stroke_kind: Option<ph2d_panel_vector::StrokePaintKind>,
    // ⭐ O PINCEL (plano 36, W4): o gesto que arma a arte, e a lei dos knobs.
    pub(in crate::render_loop) pending_brush_pick: bool,
    pub(in crate::render_loop) pending_brush: Option<crate::vec_stroke_paint::BrushCmd>,
    // Linear-gradient angle (degrees) from the Angle slider (track·360).
    pub(in crate::render_loop) pending_vec_grad_angle: Option<f64>,
    pub(in crate::render_loop) pending_vec_grad_add: bool,
    pub(in crate::render_loop) pending_vec_grad_remove: bool,
    // Multi-point Influence slider (track·4).
    pub(in crate::render_loop) pending_vec_grad_influence: Option<f64>,
    pub(in crate::render_loop) pending_vec_grad_jitter: Option<f64>,
    pub(in crate::render_loop) pending_vec_grad_add_stop: bool,
    pub(in crate::render_loop) pending_vec_grad_remove_stop: bool,
    pub(in crate::render_loop) pending_vec_align: Option<crate::input_dispatch::VecAlign>,
    pub(in crate::render_loop) pending_vec_distribute: Option<crate::input_dispatch::VecDistribute>,
    // Make (true) / Release (false) Compound over the selection.
    pub(in crate::render_loop) pending_vec_compound: Option<bool>,
    // Fill rule of the selected compound path: even-odd (true) or non-zero.
    pub(in crate::render_loop) pending_vec_fill_rule: Option<bool>,
    // Snap section: encaixar em formas (a grade é do painel de Grid).
    pub(in crate::render_loop) pending_vec_snap_on: Option<bool>,
    pub(in crate::render_loop) pending_vec_snap_path: Option<bool>,
    pub(in crate::render_loop) pending_vec_snap_cross: Option<bool>,
    pub(in crate::render_loop) pending_vec_snap_guides: Option<bool>,
    pub(in crate::render_loop) pending_rulers: Option<bool>,

    // Numeric Transform field edit (X/Y/W/H) — a SetValue document command.
    // ⭐ A APARÊNCIA do objecto (estudo 42 item 2): o track `0..1` do slider e o CÓDIGO do
    // modo de mistura. Capturados aqui e aplicados ao documento no dreno, como o Transform.
    pub(in crate::render_loop) pending_vec_opacity: Option<f64>,
    pub(in crate::render_loop) pending_vec_blend: Option<u8>,
    // ⭐⭐⭐ A PILHA DE APARÊNCIA (estudo 42 item 4): o verbo pedido, e as três propriedades
    // da camada ABERTA. ⚠️ O índice vem do PAINEL (a camada aberta é vista dele), então a
    // shell não guarda um segundo — dois índices para a mesma pergunta divergem no
    // primeiro gesto que mexe na pilha.
    pub(in crate::render_loop) pending_paint_verb: Option<crate::vec_paint_stack::StackVerb>,
    pub(in crate::render_loop) pending_paint_width: Option<f64>,
    // ⭐ ONDE a camada aberta desenha (v21). Dois slots e nao um par: as duas caixas
    // comitam INDEPENDENTES, e um par obrigaria a inventar o eixo que nao mudou.
    pub(in crate::render_loop) pending_paint_dx: Option<f64>,
    pub(in crate::render_loop) pending_paint_dy: Option<f64>,
    // ⭐ O OFFSET DE CAD da camada aberta (v22) e a quina dele.
    pub(in crate::render_loop) pending_paint_dilate: Option<f64>,
    pub(in crate::render_loop) pending_paint_join: Option<u8>,
    pub(in crate::render_loop) pending_paint_opacity: Option<f64>,
    pub(in crate::render_loop) pending_paint_blend: Option<u8>,
    pub(in crate::render_loop) pending_vec_transform:
        Option<(crate::input_dispatch::VecTransformField, f64)>,
    // **ONDE o NÓ vai** — `(eixo_y?, alvo)` na unidade do artista. Um por frame: os dois
    // campos são commitados por gestos distintos, e mandar os dois no mesmo quadro
    // significaria dois deslocamentos, que é o que o `nudge` já faz num.
    pub(in crate::render_loop) pending_vec_vert: Option<(bool, f64)>,
    // Transform Angle field (R) — a relative rotation delta (degrees).
    pub(in crate::render_loop) pending_vec_rotate_by: Option<f64>,
    // Slider de parâmetro de forma (Sides/Points/Inner/Radius/Turns/Degrees):
    // `(id, track 0..1)`. A tool já o consome como default de desenho; aqui ele
    // também edita a forma VIVA selecionada (Live Shape).
    pub(in crate::render_loop) pending_vec_shape_param: Option<(ph2d_editor_core::NodeId, f64)>,
    // Campo do CONECTOR (Route / Jetty / Spread): `(id, valor)`. Não é Style da tool
    // — é a RELAÇÃO, que mora no `VecConnector` de cada conector SELECIONADO (todos
    // eles: é assim que se calibra o diagrama inteiro de uma vez).
    pub(in crate::render_loop) pending_vec_connector: Option<(ph2d_editor_core::NodeId, f64)>,
    // Text Size slider (world units) — updates the active session + the
    // size a new session starts at.
    pub(in crate::render_loop) pending_vec_text_size: Option<f64>,
    // Text Weight slider (`wght` axis) — updates the active session + the
    // weight a new session starts at.
    pub(in crate::render_loop) pending_vec_text_weight: Option<f32>,
    // Paragraph: line-height (× size), tracking (em), and alignment (L/C/R).
    pub(in crate::render_loop) pending_vec_text_line_height: Option<f64>,
    pub(in crate::render_loop) pending_vec_text_tracking: Option<f64>,
    // ⚠️ `Option<Option<f64>>`: o de fora é *houve pedido neste frame?*, o de dentro é
    // *Auto ou esta largura?*. Colapsá-los faria "voltar para Auto" indistinguível de
    // "ninguém tocou", e o modo Auto seria inalcançável.
    pub(in crate::render_loop) pending_vec_text_wrap: Option<Option<f64>>,
    pub(in crate::render_loop) pending_vec_text_align: Option<ph2d_vec_text::TextAlign>,
    // Variation-axis field edit: (slot index into the font's non-wght axes, value).
    pub(in crate::render_loop) pending_vec_text_axis: Option<(usize, f64)>,
    // Text font-family cycle (`<` = -1 / `>` = +1) from the panel picker.
    pub(in crate::render_loop) pending_vec_font_cycle: Option<i32>,
    // Font dropdown option pick — index into `vec_font::pickable_families()`.
    pub(in crate::render_loop) pending_vec_font_pick: Option<usize>,
    // "Import Font…" button — opens a native picker for a .ttf/.otf.
    pub(in crate::render_loop) pending_vec_font_import: bool,
    // "Convert to Curves" — bake the selected live shape(s) into raw paths.
    pub(in crate::render_loop) pending_vec_convert: bool,
    pub(in crate::render_loop) transform_edit: Option<ph2d_editor_core::InspectorTransformInfo>,
    pub(in crate::render_loop) visibility_edits: Vec<(u64, bool)>,
    pub(in crate::render_loop) sprite_source_change: Option<(u64, RequestedSpriteStrategy)>,
    // Sprite field edits (flip/region/sheet/tint/…) — a Vec so a
    // bulk edit that touches several fields in one frame all apply.
    pub(in crate::render_loop) sprite_edits: Vec<(u64, ph2d_editor_core::SpriteFieldEdit)>,
    // §7 ordering edits (W3) — optional-component edits, fanned out
    // to the selection like sprite edits.
    pub(in crate::render_loop) ordering_edits: Vec<(u64, ph2d_editor_core::OrderingFieldEdit)>,
    pub(in crate::render_loop) sampling_edits: Vec<(u64, ph2d_editor_core::SamplingFieldEdit)>,
    pub(in crate::render_loop) blend_edits: Vec<(u64, ph2d_editor_core::BlendFieldEdit)>,
    pub(in crate::render_loop) slice_edits: Vec<(u64, ph2d_editor_core::SliceFieldEdit)>,
    pub(in crate::render_loop) anchor_edits: Vec<(u64, ph2d_editor_core::AnchorFieldEdit)>,
    pub(in crate::render_loop) anim_edits: Vec<(u64, ph2d_editor_core::AnimFieldEdit)>,
    pub(in crate::render_loop) timer_edits: Vec<(u64, ph2d_editor_core::TimerFieldEdit)>,
    pub(in crate::render_loop) audio_edits: Vec<(u64, ph2d_editor_core::AudioFieldEdit)>,
    pub(in crate::render_loop) camera_edits: Vec<(u64, ph2d_editor_core::CameraFieldEdit)>,
    /// ⭐ As edições das secções FACTORY e LIFECYCLE (TOP-20 #11 e #12).
    pub(in crate::render_loop) factory_edits: Vec<(u64, ph2d_editor_core::FactoryFieldEdit)>,
    /// ⭐ As edições do MOVER DE VISTA DE CIMA (TOP-20 #13).
    pub(in crate::render_loop) topdown_edits:
        Vec<(u64, ph2d_editor_core::topdown_edits::TopDownFieldEdit)>,
    /// ⭐ As edições do PROJÉCTIL (TOP-20 #14).
    pub(in crate::render_loop) projectile_edits:
        Vec<(u64, ph2d_editor_core::projectile_edits::ProjectileFieldEdit)>,
    /// ⭐ As edições do CÉREBRO (TOP-20 #15).
    pub(in crate::render_loop) statemachine_edits: Vec<(
        u64,
        ph2d_editor_core::statemachine_edits::StateMachineFieldEdit,
    )>,
    /// ⭐ As edições do SCRIPT (TOP-20 #16).
    pub(in crate::render_loop) script_edits:
        Vec<(u64, ph2d_editor_core::script_edits::ScriptFieldEdit)>,
    /// ⭐ As edições do EMISSOR DE PARTÍCULAS (TOP-20 #18).
    pub(in crate::render_loop) particles_edits:
        Vec<(u64, ph2d_editor_core::particles_edits::ParticlesFieldEdit)>,
    /// ⭐⭐⭐ A secção HUD (TOP-20 #20).
    pub(in crate::render_loop) hud_edits: Vec<(u64, ph2d_editor_core::hud_edits::HudFieldEdit)>,
    // ⭐ A secção TAGS (TOP-20 #9) — ver o dreno dela no `fase_inspector_commits`.
    pub(in crate::render_loop) tags_edits: Vec<(u64, ph2d_editor_core::TagsFieldEdit)>,
    // ⭐⭐⭐ O painel TAGS (TOP-20 #9, W4) — gestos sobre a ÁRVORE, que não é do mundo. Ver o
    // dreno deles no `fase_inspector_commits`, que é a fase que tem a árvore E o mundo.
    pub(in crate::render_loop) tag_tree_edits: Vec<ph2d_editor_core::TagTreeEdit>,
    // ⚠️ **`inspector_queue_dirty` e não `audio_commit`**: desde a secção CAMERA (TOP-20
    // #7) esta bandeira serve DUAS secções, e o nome antigo passou a descrever metade do
    // que ela significa. *Um nome que já não cobre a população dele mente na próxima
    // leitura.*
    pub(in crate::render_loop) inspector_queue_dirty: bool,
    pub(in crate::render_loop) action_edits: Vec<(u64, ph2d_editor_core::ActionFieldEdit)>,
    // ⭐ O `+` do Inspector (F3): quem pediu a paleta neste quadro.
    pub(in crate::render_loop) add_component_for: Option<u64>,
    // ⭐ A troca de variante pedida neste quadro: `(raiz da instância, StableId do mestre)`.
    pub(in crate::render_loop) swap_variant: Option<(u64, u64)>,
    // O `StableId` da peça acrescentada que o cartão mandou aplicar.
    pub(in crate::render_loop) apply_added: Option<u64>,
    // ⭐⭐⭐ **O DEGRAU escolhido do *Aplicar*** (F5 critério 4) — `(peça clicada, receita)`.
    // ⚠️ **ADIADO pela razão da irmã de cima**: o verbo precisa do **eco** e dos documentos
    // possuídos, e aqui dentro o `self` já está emprestado.
    pub(in crate::render_loop) apply_to_level: Option<(u64, u64)>,
    pub(in crate::render_loop) open_asset_browser: bool,
    // ⭐ O pedido de renomear o VALOR de uma propriedade — `(receita, chave, valor)`.
    // ⭐ A entidade cujo campo de nome fechou neste quadro.
    pub(in crate::render_loop) physics_edits: Vec<(u64, ph2d_editor_core::PhysicsFieldEdit)>,
    // §12 joints (W3). Kept out of `inspector_commits::dispatch`: that
    // signature is already the length its own doc-comment warns about,
    // and these two are applied in one short block below.
    pub(in crate::render_loop) joint_edits: Vec<(u64, ph2d_editor_core::JointFieldEdit)>,
    pub(in crate::render_loop) wheel_edits: Vec<(u64, ph2d_editor_core::WheelFieldEdit)>,
    // §14 Platform Player (W5). Sem fan-out, e pela razão da §12/§13: a
    // seção descreve UM personagem, o selecionado — espalhar um `Add`
    // pela seleção criaria N players num clique que pediu um.
    pub(in crate::render_loop) player_edits: Vec<(u64, ph2d_editor_core::PlayerFieldEdit)>,
    // The pair to join, at most one per frame — it is a click, not a
    // per-entity edit.
    pub(in crate::render_loop) bake_request: Option<Vec<u64>>,
    // W-J4: a rota por SELEÇÃO virou "ligue a sequência" (2 corpos = um
    // joint; N = uma corrente de N−1), então o pedido é um booleano — a
    // ordem vem da própria seleção, que o `join_selected_chain` lê.
    pub(in crate::render_loop) join_chain: bool,
    pub(in crate::render_loop) join_draw_arm: bool,
    // W-Rig: um clique em *Rig* — booleano pelo mesmo motivo do
    // `join_chain`, porque a SELEÇÃO já diz sobre o que ele age.
    pub(in crate::render_loop) rig_now: bool,
    pub(in crate::render_loop) visibility_section_edits:
        Vec<(u64, ph2d_editor_core::VisibilityFieldEdit)>,
    pub(in crate::render_loop) name_edit: Option<ph2d_editor_core::InspectorNameInfo>,
    pub(in crate::render_loop) signal_edit: Option<ph2d_editor_core::InspectorNameInfo>,
    pub(in crate::render_loop) signal_leave_edit: Option<ph2d_editor_core::InspectorNameInfo>,
    pub(in crate::render_loop) bgremoval_leftover: Vec<ph2d_editor_core::action_bus::EditorAction>,
    // Painter Apply leftover — same shape as bgremoval (drained
    // back into the bus so `image_edit::dispatch`'s
    // `painter_active` gate runs AFTER any same-frame
    // ActivateTool resolution). Day-7 ship.
    pub(in crate::render_loop) painter_leftover: Vec<ph2d_editor_core::action_bus::EditorAction>,
    pub(in crate::render_loop) inspector_selection: Vec<u64>,
}
