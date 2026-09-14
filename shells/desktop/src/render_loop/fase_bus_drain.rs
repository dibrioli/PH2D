//! **Fase do quadro: O DRENO DO BARRAMENTO** — os pedidos do quadro (`pending_*` e os seus irmãos) e o
//! `for action in bus.drain()` que os enche, devolvidos juntos (OBRA 2 da `line/render-loop`, 2026-09-13).
//!
//! ⭐ **Os pedidos são UM valor e os braços são sub-drenos** (`line/render-bodies`, 2026-09-13): cada pedido é um
//! campo do [`DrainOut`] (o `Default` é o valor com que a declaração o criava, e o comentário dela mora no campo),
//! e o `match` partiu-se por ASSUNTO em `fase_bus_*`, chamadas pela ordem dos braços. Cada uma devolve à seguinte
//! o pedido que não é dela, e por isso um pedido cai num braço só, como no `match` (as guardas incluídas).
//! ⚠️ No corpo de um braço a única troca é `pd.<pedido>` onde ele escrevia `<pedido>`, e só em CÓDIGO. A nota que
//! esta fase trazia — *partir por braço obrigaria o corpo a escrever `*pedido = …` por referências* — media a
//! partição por referências soltas; um contexto nomeado não as pede.

use super::*;

/// A timeline docada, os pedidos que esperam outro ponto do quadro e a barra do topo.
#[path = "fase_bus_chrome.rs"]
mod bus_chrome;
/// As três terças da cadeia do CLIQUE de um painel de ferramenta.
#[path = "fase_bus_clicks.rs"]
mod bus_clicks;
/// A Hierarquia: os interruptores, a árvore, os verbos de instância e de linha, a selecção e o renomear.
#[path = "fase_bus_hierarchy.rs"]
mod bus_hierarchy;
/// O Inspector: a sprite, as secções, o cartão da instância, a física, a visibilidade e os nomes.
#[path = "fase_bus_inspector.rs"]
mod bus_inspector;
/// O canal painel → ferramenta: a activação, o `PanelEvent` de uma ferramenta e as cadeias do campo.
#[path = "fase_bus_tool_panel.rs"]
mod bus_tool_panel;

/// Os pedidos que o dreno do barramento recolheu neste quadro, para as fases que os consomem.
///
/// ⚠️ **O `Default` é o valor com que cada pedido nascia** (`None` · `false` · `Vec::new()`). Três nascem de uma
/// LEITURA do quadro — `osso_selecionado`, `selecao_bits` e `inspector_selection` — e moram no `let mut pd` da
/// `fase_bus_drain`, com a razão ao lado.
#[derive(Default)]
pub(super) struct DrainOut {
    pub(super) pending_image_tool_activation: Option<&'static str>,
    pub(super) visibility_toggle_row: Option<NodeId>,
    pub(super) lock_toggle_row: Option<NodeId>,
    pub(super) group_toggle_row: Option<NodeId>,
    pub(super) reparent_intent: Option<ph2d_editor_core::screens::hero::HierReparentIntent>,
    pub(super) duplicate_row: Option<NodeId>,
    // Set by `hierarchy::dispatch` to `(source_bits, new_bits)` when a sprite is duplicated, so
    // we can fork the copy onto its own texture (independent object) post-dispatch.
    pub(super) duplicate_made: Option<(u64, u64)>,
    pub(super) add_child_row: Option<NodeId>,
    // ⭐⭐ **Agrupar / desagrupar** (2026-08-30): `(linha clicada, agrupar?)`. Um slot só para
    // os dois verbos — eles são o mesmo gesto com o sinal trocado, e dois slots deixariam
    // a porta aberta a alguém drenar os dois no mesmo quadro.
    pub(super) group_row: Option<(NodeId, bool)>,
    // ⭐ **O `Add` do cabeçalho da Hierarquia** (ADR-0166 / F3) — um objeto vazio na raiz.
    // Sem payload: ele não sai de uma linha, e por isso não tem pai (ver `HierAddRoot`).
    pub(super) add_root: bool,
    pub(super) reset_transform_row: Option<NodeId>,
    // ⭐ *Revert to Master* (ADR-0164 / F4.4) — a linha cuja instância volta à receita.
    pub(super) revert_to_master_row: Option<NodeId>,
    // ⭐ Os outros verbos de instância (ADR-0164 / F4.5) — UM slot, porque eles são
    // exclusivos por construção: o menu fecha ao primeiro clique.
    pub(super) instance_verb_row: Option<(NodeId, ph2d_app_components::instance_verbs::Verb)>,
    // ⭐ O mesmo verbo, endereçado por `StableId` — o canal do navegador de assets.
    pub(super) instance_verb_stable_id: Option<(
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
    pub(super) catalog_verbs: Vec<ph2d_editor_core::action_bus::CatalogVerb>,
    pub(super) asset_card_verb: Option<(
        ph2d_editor_core::interaction::drag_payload::DragPayload,
        ph2d_editor_core::action_bus::AssetCardAction,
    )>,
    pub(super) delete_row: Option<NodeId>,
    // Enio 2026-05-27: right-click → Merge Sprites in Hierarchy.
    // Carries the clicked row's `NodeId` (the merged sprite
    // adopts that row's parent for Hierarchy placement); the
    // drain reads the full multi-selection at apply time.
    pub(super) merge_sprites_row: Option<NodeId>,
    // "Pack into Sheet" do menu de contexto da hierarquia — a 2ª porta do verbo do pill
    // `[SHEET]`. Guarda a LINHA (não a entidade): quem a resolve é o `bridge`, no dreno.
    pub(super) pack_sheet_row: Option<NodeId>,
    // "Auto-Arrange Pieces" — re-encaixar os filhos de uma folha que já existe.
    pub(super) arrange_sheet_row: Option<NodeId>,
    // "Remove from Sheet" — a saída da folha, pela linha clicada.
    pub(super) remove_from_sheet_row: Option<NodeId>,
    // As duas saídas do BAKE (plano §7.3, W5.2): assar muda a cena, exportar escreve
    // ficheiros. Linhas separadas porque são dois pedidos diferentes.
    pub(super) bake_sheet_row: Option<NodeId>,
    pub(super) export_sheet_row: Option<NodeId>,
    // **EXPORTAR UMA SPRITE** (plano `docs/Sprite_projeto/18` W9) — irmão do de cima, e a
    // diferença está no nome: aquele escreve a FOLHA, este escreve uma sprite no formato
    // que a extensão escolhida nomear.
    pub(super) export_image_row: Option<NodeId>,
    // **FUNDIR EM CAMADAS** (plano `docs/Sprite_projeto/18` W10) — a mesma geometria do
    // Merge, e cada fonte fica também numa camada do documento do Painter.
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
    // O pedido de troca de PRECISAO (plano `docs/Sprite_projeto/18` W5). `Option` e nao
    // `Vec`: o par so' existe com uma sprite selecionada.
    pub(super) precision_request: Option<(u64, ph2d_color::Precision)>,
    // **A SPRITE COMO FONTE DE LUZ** (plano `docs/Sprite_projeto/18` W8). Recolhido aqui e
    // drenado com o irmão `precision_request` — o mesmo padrão, porque o componente só pode
    // ser escrito onde o `sim` está emprestado mutavelmente.
    // ⚠️ **Um Vec, não um `Option`** — a emissão é uma edição de campo como a Opacidade, e
    // a Opacidade espalha-se pela seleção. Enquanto isto foi `Option<(u64, f32)>` o slider
    // parecia um bulk edit e mudava **uma** sprite (auditoria `docs/Sprite_projeto/20` §3).
    pub(super) emissive_edits: Vec<(u64, f32)>,
    // Fase 0e: per-sprite tools collect a Vec<u64> instead of
    // Option<u64> so a multi-select OneShotImageOp broadcast
    // applies the bake to every selected sprite (legacy
    // single-select still works — the Vec just carries one
    // entry). image_edit::dispatch iterates each Vec.
    pub(super) trim_entities: Vec<u64>,
    pub(super) make_square_entities: Vec<u64>,
    pub(super) real_size_entities: Vec<u64>,
    pub(super) rasterize_entities: Vec<u64>,
    // ⚠️ **Este NÃO é por-sprite, e é a exceção da fila.** A chrome emite um
    // `OneShotImageOp` por entidade selecionada; os irmãos aplicam o bake a cada um
    // isoladamente, e este junta a leva inteira para criar **uma** folha. N atos
    // independentes dariam N folhas de uma peça cada — um verbo que fala da RELAÇÃO
    // entre as peças não cabe num evento por peça.
    pub(super) undo_image_edit: bool,
    // ADR-0108 Fase 1: a Boolean button (Union/Subtract/Intersect) in the
    // docked Vector panel forwards a `ToolPanelEvent::Click`; the op acts
    // on the DOCUMENT (shell-owned `vec_scene`), not the tool's Style, so
    // capture it here and apply after the drain (mirror of the U/I/D
    // hotkeys, next to the vector render).
    pub(super) pending_vec_bool: Option<ph2d_vec_boolean::PathfinderOp>,
    pub(super) pending_vec_expand: Option<crate::vec_expand::Expand>,
    // OS COMPONENTES (plano UI/UX W5): o verbo pedido neste frame.
    pub(super) pending_component: Option<crate::vec_component_edit::ComponentEdit>,
    pub(super) pending_widget_edit: Option<crate::vec_widget_edit::WidgetEdit>,
    // OS ESTADOS de UI (plano UI/UX W7): a tabela mora no documento, entao o clique e' da
    // shell — o painel so' mostra que verbos fazem sentido agora.
    pub(super) pending_ui_state: Option<crate::vec_ui_state_edit::UiStateEdit>,
    pub(super) pending_ui_state_duration: Option<f64>,
    // ⚠️ Um TOGGLE não traz valor: o pedido é *"inverta"*, e quem sabe o estado atual é
    // a tabela. Um `Some(bool)` obrigaria a shell a lê-la duas vezes.
    pub(super) pending_ui_spring_toggle: bool,
    // (é a rigidez?, valor) — só o knob que o artista arrastou.
    pub(super) pending_ui_spring_knob: Option<(bool, f64)>,
    pub(super) pending_ui_easing: Option<crate::vec_ui_state_edit::EasingPick>,
    // ⭐ **A TABELA SINAL → PAPEL** (item 4 do estudo dos contêineres): os três gestos de
    // clique e o COMMIT do nome. Duas variáveis porque são dois canais do barramento —
    // o `Click` e o `SelectOption`, que é o único variante do `PanelEvent` (contrato
    // CONGELADO) que carrega uma string.
    pub(super) pending_ui_signal_edit: Option<crate::vec_ui_state_edit::SignalEdit>,
    pub(super) pending_ui_signal_name: Option<(usize, String)>,
    pub(super) pending_ui_preview_toggle: bool,
    pub(super) pending_ui_move_all_toggle: bool,
    // **A BOOLEANA VIVA** (plano UI/UX W1): o Apply consolida o que o produtor cozinhou
    // NESTE frame, então ele não pode correr aqui — corre logo depois do `recook`, onde o
    // plano existe. Aqui só se anota o clique.
    pub(super) pending_morph_arrow: Option<crate::vec_morph_edit::MorphCmd>,
    pub(super) pending_morph_preview_toggle: bool,
    pub(super) pending_bool_apply: bool,
    // A MOLDURA (plano UI/UX W0): o chip de recorte e o preset de dispositivo.
    pub(super) pending_frame_clip: Option<bool>,
    // **O VERBO DA FORMA selecionada** dentro de uma booleana viva (2026-08-22).
    // Irmao exacto do `pending_frame_clip`, e pelo mesmo motivo: o valor mora num
    // COMPONENTE, entao quem escreve e' a shell — o painel so' mostra qual chip
    // esta' aceso.
    pub(super) pending_bool_shape_op: Option<u8>,
    // O AUTO LAYOUT (plano UI/UX W2, ADR-0153): um chip de radio e um campo numerico.
    pub(super) pending_layout_edit: Option<crate::vec_layout_edit::LayoutEdit>,
    pub(super) pending_anchor_edit: Option<crate::vec_anchor_edit::AnchorEdit>,
    // **Resize Box** (plano UI/UX W3b): o clique e' um TOGGLE, entao nao ha' operando —
    // um bool basta para dizer *"houve clique"*.
    pub(super) pending_resize_box: bool,
    // ⭐ **Stroke** (plano 34): a caixa que dá/tira o traço da forma selecionada. Também é
    // um TOGGLE, então um bool basta — o operando é a ficha da ferramenta, e ela não viaja.
    pub(super) pending_stroke_present: bool,
    pub(super) pending_layout_field: Option<(crate::vec_layout_edit::LayoutField, f64)>,
    // **O Z-INDEX global** (Enio, 2026-08-04): o numero que sobrepoe a ordem da
    // hierarquia. Campo numerico, entao a rota e' a mesma do Transform.
    pub(super) pending_vec_z: Option<f64>,
    // **O TOKEN escolhido no picker** (plano UI/UX W4): a propriedade + o token, ou
    // `None` no token = SOLTAR (a propriedade volta ao literal do documento).
    pub(super) pending_token_bind: Option<(ph2d_ecs::BoundProp, Option<&'static str>)>,
    pub(super) pending_frame_preset: Option<ph2d_tool_vector::frames::DevicePreset>,
    // **A ESCALA da seleção de nós** (plano 25 §6, W3b): os dois alcances que o retângulo
    // não dá. Não são edições de documento — só mudam QUEM está selecionado —, então não
    // abrem passo de undo (o `post_frame_undo` compara o ESTADO, e a seleção não é dele).
    pub(super) pending_vec_select_subpath: bool,
    pub(super) pending_vec_select_same: bool,
    // **As três da W4** (plano 25 §7). Ao contrário das duas acima, estas MUDAM o
    // documento — logo abrem passo de undo, e cada uma abre exatamente um.
    pub(super) pending_vec_join: bool,
    // ⭐⭐⭐ **Soldar** (plano 39): os traços seleccionados partem-se nos cruzamentos.
    pub(super) pending_vec_weld: bool,
    pub(super) pending_vec_cut: bool,
    pub(super) pending_vec_symmetry_apply: bool,
    pub(super) pending_vec_cut_discard: bool,
    pub(super) pending_vec_reverse: bool,
    pub(super) pending_vec_average: bool,
    // O índice do perfil nomeado que o clique pediu (W2b), se algum.
    pub(super) pending_width_preset: Option<usize>,
    // ADR-0128: o botão "Blend" cria um Blend Object VIVO da seleção; o slider Steps
    // ajusta o blend selecionado ao vivo. (O destrutivo `vec_blend::apply` sobrevive só
    // para os smokes — o painel não o alcança mais.)
    pub(super) pending_create_blend: bool,
    pub(super) pending_reset_spine: bool,
    pub(super) pending_expand_blend: bool,
    pub(super) pending_release_blend: bool,
    pub(super) pending_blend_steps: Option<u32>,
    pub(super) pending_create_morph: bool,
    pub(super) pending_morph_t: Option<f32>,
    // ADR-0129: o botão "Envelope" envolve a seleção numa gaiola (container); Expand
    // materializa a deformada e Release ressuscita a fonte autorada — os dois dissolvem.
    pub(super) pending_create_envelope: bool,
    // ⭐⭐⭐ O ESQUELETO (estudo 42 item 5): prender a selecção aos ossos, as duas saídas, e
    // os dois números do osso em foco (`true` = a força, `false` = o comprimento).
    pub(super) pending_bone_bind: bool,
    pub(super) pending_bone_release: Option<crate::skeleton_live::Keep>,
    pub(super) pending_bone_knob: Option<(bool, f64)>,
    pub(super) pending_ik_add: bool,
    pub(super) pending_ik_remove: bool,
    // ⭐ O lado da dobra que o artista escolheu neste quadro, se escolheu.
    pub(super) pending_ik_bend: Option<ph2d_skeleton::BendSide>,
    pub(super) pending_ik_knob: Option<(IkKnob, f64)>,
    pub(super) pending_limit_add: bool,
    pub(super) pending_limit_remove: bool,
    // ⚠️ `(é o MAX?, valor em GRAUS)` — a conversão para radianos é feita onde ele é
    // escrito, que é a porta onde as duas unidades se encontram.
    pub(super) pending_limit_knob: Option<(bool, f64)>,
    pub(super) pending_smart_add: bool,
    pub(super) pending_smart_remove: bool,
    pub(super) pending_smart_knob: Option<(bool, f64)>,
    // A acção escolhida no selector do osso inteligente — o ÍNDICE na lista de clips que o
    // painel pinta; o que se guarda no componente é o NOME dela.
    pub(super) pending_smart_clip: Option<usize>,
    // O *Pick Object* foi carregado — arma o gesto de duas mãos do alvo.
    pub(super) pending_smart_pick: bool,
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
    pub(super) pending_bone_needs_focus: bool,
    pub(super) osso_selecionado: Option<u64>,
    pub(super) selecao_bits: Vec<u64>,
    pub(super) pending_textpath: Option<crate::vec_text_ride::TextPathCmd>,
    pub(super) pending_textpath_offset: Option<f64>,
    // Pattern on Path (plano 23): o comando de vínculo + os dois sliders, drenados como os
    // do texto (o motivo é o PRIMÁRIO, o guia é o outro selecionado).
    pub(super) pending_patternpath: Option<crate::pattern_live::PatternPathCmd>,
    pub(super) pending_pp_spacing: Option<f64>,
    pub(super) pending_pp_start: Option<f64>,
    pub(super) pending_pp_end: Option<f64>,
    pub(super) pending_pp_slide: Option<f64>,
    pub(super) pending_pp_offset: Option<f64>,
    // Contour (pesquisa `20_*` #9): os três comandos + os três sliders + os dois trios
    // exclusivos. `Add`/`Remove` são portas do MODELO (armam/tiram o componente),
    // `Expand` materializa; os knobs só editam o que já existe.
    pub(super) pending_contour: Option<crate::contour_live::ContourCmd>,
    pub(super) pending_contour_steps: Option<f64>,
    pub(super) pending_contour_d: Option<f64>,
    pub(super) pending_contour_accel: Option<f64>,
    pub(super) pending_contour_join: Option<u8>,
    pub(super) pending_contour_side: Option<u8>,
    // Filters (FX raster, plano 24). `Some(Some(k))` arma o tipo `k`; `Some(None)` remove.
    // Filters (a PILHA de FX raster, plano 24): um comando (Add/✕/↑/↓/👁) e um valor de
    // slider por frame, decodificados pela porta única `fx_live::hit_of`.
    pub(super) pending_filter_cmd: Option<crate::fx_live::FilterHit>,
    // O arrasto de um punho da rampa: `(linha, índice de AUTORIA do stop, posição 0..1)`.
    // Um `pending`, como o `FilterHit`, e pelo MESMO motivo: a edição do documento mora no
    // bloco que tem o `sim` em mãos, e o drain do barramento não o tem.
    pub(super) pending_filter_stop: Option<(usize, u8, f32)>,
    pub(super) pending_filter_val: Option<(crate::fx_live::FilterHit, f64)>,
    pub(super) pending_pp_rotation: Option<f64>,
    // O Picker de guia (Enio 2026-07-23): o botão só ARMA — a shell captura a fonte e o
    // clique seguinte no canvas escolhe o guia. Um por feature; a fonte é resolvida no drain.
    pub(super) pending_pp_pick: bool,
    pub(super) pending_text_pick: bool,
    pub(super) pending_expand_envelope: bool,
    pub(super) pending_release_envelope: bool,
    // O GESTO do envelope (ADR-0129 Fatias D+E): Perspective (projetivo) · Mesh (Coons) ·
    // Pins (MLS). Um enum e nao um bool desde que o 3o gesto entrou.
    pub(super) pending_envelope_kind: Option<ph2d_ecs::EnvelopeKind>,
    pub(super) pending_clear_pins: bool,
    // O PRESET de gaiola (ADR-0129 Fatia C): indice em `EnvelopeWarp::ALL`, e o Bend.
    pub(super) pending_envelope_preset: Option<usize>,
    pub(super) pending_envelope_bend: Option<f64>,
    // ADR-0132: a pilha de efeitos. Um clique num BOTAO (add/remove/up/down/toggle) e
    // um arrasto num slider -- os dois enderecados por (linha, parametro), sem que este
    // arquivo saiba que efeitos existem.
    pub(super) pending_fx_add: Option<usize>,
    pub(super) pending_fx_button: Option<(usize, crate::fx_bridge_dispatch::FxRowAction)>,
    pub(super) pending_fx_param: Option<(usize, usize, f64)>,
    // ADR-0132: o "Apply" assa a pilha de efeitos no cozido e a esvazia (Expand Appearance).
    pub(super) pending_fx_apply: bool,
    // ADR-0108 Fase 1: a Vertex button (Corner/Smooth/Symmetric) retypes
    // the selected vertex — a document edit, applied after the drain.
    pub(super) pending_vec_vertex_kind: Option<ph2d_vec_scene::VertexKind>,
    // ADR-0108 Fase 1: "Delete Node" button removes the selected vertex.
    pub(super) pending_vec_delete_vertex: bool,
    // ADR-0108: Arrange buttons — z-order restack + Duplicate + Flip H/V —
    // act on the selected path (document ops), applied after the drain.
    pub(super) pending_vec_reorder: Option<ph2d_vec_scene::ZOrder>,
    pub(super) pending_vec_duplicate: bool,
    pub(super) pending_vec_flip: Option<ph2d_vec_scene::FlipAxis>,
    pub(super) pending_vec_rotate: Option<ph2d_vec_scene::Rotate90>,
    pub(super) pending_vec_path_shape: Option<crate::input_dispatch::VecPathShapeOp>,
    pub(super) pending_vec_toggle_closed: bool,
    pub(super) pending_vec_pivot_edit: bool,
    pub(super) pending_vec_fill_kind: Option<crate::input_dispatch::VecFillKind>,
    // A lei do PADRÃO de textura (plano 33 W5). Uma só por quadro: os controles da secção
    // são exclusivos entre si (o artista mexe num de cada vez), e uma fila daria dois passos
    // de undo para um gesto.
    // ⚠️ **Cada um leva o SUJEITO junto** (plano 35, wave F): o slot sai do id do controlo
    // que foi clicado, e não de uma preferência guardada — *o que o gesto endereça não pode
    // ser lido de outro sítio no drain.*
    pub(super) pending_texpat: Option<(
        ph2d_vec_render::PatternSlot,
        crate::texture_pattern_edit::TexPatCmd,
    )>,
    pub(super) pending_texpat_source: Option<ph2d_vec_render::PatternSlot>,
    pub(super) pending_texpat_pick: Option<ph2d_vec_render::PatternSlot>,
    // ⭐ A TINTA do traço (plano 35, wave D) — irmã do `pending_vec_fill_kind`, e drenada no
    // MESMO sítio, porque as duas podem precisar de abrir o diálogo da arte.
    pub(super) pending_vec_stroke_kind: Option<ph2d_panel_vector::StrokePaintKind>,
    // ⭐ O PINCEL (plano 36, W4): o gesto que arma a arte, e a lei dos knobs.
    pub(super) pending_brush_pick: bool,
    pub(super) pending_brush: Option<crate::vec_stroke_paint::BrushCmd>,
    // Linear-gradient angle (degrees) from the Angle slider (track·360).
    pub(super) pending_vec_grad_angle: Option<f64>,
    pub(super) pending_vec_grad_add: bool,
    pub(super) pending_vec_grad_remove: bool,
    // Multi-point Influence slider (track·4).
    pub(super) pending_vec_grad_influence: Option<f64>,
    pub(super) pending_vec_grad_jitter: Option<f64>,
    pub(super) pending_vec_grad_add_stop: bool,
    pub(super) pending_vec_grad_remove_stop: bool,
    pub(super) pending_vec_align: Option<crate::input_dispatch::VecAlign>,
    pub(super) pending_vec_distribute: Option<crate::input_dispatch::VecDistribute>,
    // Make (true) / Release (false) Compound over the selection.
    pub(super) pending_vec_compound: Option<bool>,
    // Fill rule of the selected compound path: even-odd (true) or non-zero.
    pub(super) pending_vec_fill_rule: Option<bool>,
    // Snap section: encaixar em formas (a grade é do painel de Grid).
    pub(super) pending_vec_snap_on: Option<bool>,
    pub(super) pending_vec_snap_path: Option<bool>,
    pub(super) pending_vec_snap_cross: Option<bool>,
    pub(super) pending_vec_snap_guides: Option<bool>,
    pub(super) pending_rulers: Option<bool>,

    // Numeric Transform field edit (X/Y/W/H) — a SetValue document command.
    // ⭐ A APARÊNCIA do objecto (estudo 42 item 2): o track `0..1` do slider e o CÓDIGO do
    // modo de mistura. Capturados aqui e aplicados ao documento no dreno, como o Transform.
    pub(super) pending_vec_opacity: Option<f64>,
    pub(super) pending_vec_blend: Option<u8>,
    // ⭐⭐⭐ A PILHA DE APARÊNCIA (estudo 42 item 4): o verbo pedido, e as três propriedades
    // da camada ABERTA. ⚠️ O índice vem do PAINEL (a camada aberta é vista dele), então a
    // shell não guarda um segundo — dois índices para a mesma pergunta divergem no
    // primeiro gesto que mexe na pilha.
    pub(super) pending_paint_verb: Option<crate::vec_paint_stack::StackVerb>,
    pub(super) pending_paint_width: Option<f64>,
    // ⭐ ONDE a camada aberta desenha (v21). Dois slots e nao um par: as duas caixas
    // comitam INDEPENDENTES, e um par obrigaria a inventar o eixo que nao mudou.
    pub(super) pending_paint_dx: Option<f64>,
    pub(super) pending_paint_dy: Option<f64>,
    // ⭐ O OFFSET DE CAD da camada aberta (v22) e a quina dele.
    pub(super) pending_paint_dilate: Option<f64>,
    pub(super) pending_paint_join: Option<u8>,
    pub(super) pending_paint_opacity: Option<f64>,
    pub(super) pending_paint_blend: Option<u8>,
    pub(super) pending_vec_transform: Option<(crate::input_dispatch::VecTransformField, f64)>,
    // **ONDE o NÓ vai** — `(eixo_y?, alvo)` na unidade do artista. Um por frame: os dois
    // campos são commitados por gestos distintos, e mandar os dois no mesmo quadro
    // significaria dois deslocamentos, que é o que o `nudge` já faz num.
    pub(super) pending_vec_vert: Option<(bool, f64)>,
    // Transform Angle field (R) — a relative rotation delta (degrees).
    pub(super) pending_vec_rotate_by: Option<f64>,
    // Slider de parâmetro de forma (Sides/Points/Inner/Radius/Turns/Degrees):
    // `(id, track 0..1)`. A tool já o consome como default de desenho; aqui ele
    // também edita a forma VIVA selecionada (Live Shape).
    pub(super) pending_vec_shape_param: Option<(ph2d_editor_core::NodeId, f64)>,
    // Campo do CONECTOR (Route / Jetty / Spread): `(id, valor)`. Não é Style da tool
    // — é a RELAÇÃO, que mora no `VecConnector` de cada conector SELECIONADO (todos
    // eles: é assim que se calibra o diagrama inteiro de uma vez).
    pub(super) pending_vec_connector: Option<(ph2d_editor_core::NodeId, f64)>,
    // Text Size slider (world units) — updates the active session + the
    // size a new session starts at.
    pub(super) pending_vec_text_size: Option<f64>,
    // Text Weight slider (`wght` axis) — updates the active session + the
    // weight a new session starts at.
    pub(super) pending_vec_text_weight: Option<f32>,
    // Paragraph: line-height (× size), tracking (em), and alignment (L/C/R).
    pub(super) pending_vec_text_line_height: Option<f64>,
    pub(super) pending_vec_text_tracking: Option<f64>,
    // ⚠️ `Option<Option<f64>>`: o de fora é *houve pedido neste frame?*, o de dentro é
    // *Auto ou esta largura?*. Colapsá-los faria "voltar para Auto" indistinguível de
    // "ninguém tocou", e o modo Auto seria inalcançável.
    pub(super) pending_vec_text_wrap: Option<Option<f64>>,
    pub(super) pending_vec_text_align: Option<ph2d_vec_text::TextAlign>,
    // Variation-axis field edit: (slot index into the font's non-wght axes, value).
    pub(super) pending_vec_text_axis: Option<(usize, f64)>,
    // Text font-family cycle (`<` = -1 / `>` = +1) from the panel picker.
    pub(super) pending_vec_font_cycle: Option<i32>,
    // Font dropdown option pick — index into `vec_font::pickable_families()`.
    pub(super) pending_vec_font_pick: Option<usize>,
    // "Import Font…" button — opens a native picker for a .ttf/.otf.
    pub(super) pending_vec_font_import: bool,
    // "Convert to Curves" — bake the selected live shape(s) into raw paths.
    pub(super) pending_vec_convert: bool,
    pub(super) transform_edit: Option<ph2d_editor_core::InspectorTransformInfo>,
    pub(super) visibility_edits: Vec<(u64, bool)>,
    pub(super) sprite_source_change: Option<(u64, RequestedSpriteStrategy)>,
    // Sprite field edits (flip/region/sheet/tint/…) — a Vec so a
    // bulk edit that touches several fields in one frame all apply.
    pub(super) sprite_edits: Vec<(u64, ph2d_editor_core::SpriteFieldEdit)>,
    // §7 ordering edits (W3) — optional-component edits, fanned out
    // to the selection like sprite edits.
    pub(super) ordering_edits: Vec<(u64, ph2d_editor_core::OrderingFieldEdit)>,
    pub(super) sampling_edits: Vec<(u64, ph2d_editor_core::SamplingFieldEdit)>,
    pub(super) blend_edits: Vec<(u64, ph2d_editor_core::BlendFieldEdit)>,
    pub(super) slice_edits: Vec<(u64, ph2d_editor_core::SliceFieldEdit)>,
    pub(super) anchor_edits: Vec<(u64, ph2d_editor_core::AnchorFieldEdit)>,
    pub(super) anim_edits: Vec<(u64, ph2d_editor_core::AnimFieldEdit)>,
    pub(super) timer_edits: Vec<(u64, ph2d_editor_core::TimerFieldEdit)>,
    pub(super) audio_edits: Vec<(u64, ph2d_editor_core::AudioFieldEdit)>,
    pub(super) camera_edits: Vec<(u64, ph2d_editor_core::CameraFieldEdit)>,
    // ⭐ A secção TAGS (TOP-20 #9) — ver o dreno dela no `fase_inspector_commits`.
    pub(super) tags_edits: Vec<(u64, ph2d_editor_core::TagsFieldEdit)>,
    // ⭐⭐⭐ O painel TAGS (TOP-20 #9, W4) — gestos sobre a ÁRVORE, que não é do mundo. Ver o
    // dreno deles no `fase_inspector_commits`, que é a fase que tem a árvore E o mundo.
    pub(super) tag_tree_edits: Vec<ph2d_editor_core::TagTreeEdit>,
    // ⚠️ **`inspector_queue_dirty` e não `audio_commit`**: desde a secção CAMERA (TOP-20
    // #7) esta bandeira serve DUAS secções, e o nome antigo passou a descrever metade do
    // que ela significa. *Um nome que já não cobre a população dele mente na próxima
    // leitura.*
    pub(super) inspector_queue_dirty: bool,
    pub(super) action_edits: Vec<(u64, ph2d_editor_core::ActionFieldEdit)>,
    // ⭐ O `+` do Inspector (F3): quem pediu a paleta neste quadro.
    pub(super) add_component_for: Option<u64>,
    // ⭐ A troca de variante pedida neste quadro: `(raiz da instância, StableId do mestre)`.
    pub(super) swap_variant: Option<(u64, u64)>,
    // O `StableId` da peça acrescentada que o cartão mandou aplicar.
    pub(super) apply_added: Option<u64>,
    // ⭐⭐⭐ **O DEGRAU escolhido do *Aplicar*** (F5 critério 4) — `(peça clicada, receita)`.
    // ⚠️ **ADIADO pela razão da irmã de cima**: o verbo precisa do **eco** e dos documentos
    // possuídos, e aqui dentro o `self` já está emprestado.
    pub(super) apply_to_level: Option<(u64, u64)>,
    pub(super) open_asset_browser: bool,
    // ⭐ O pedido de renomear o VALOR de uma propriedade — `(receita, chave, valor)`.
    // ⭐ A entidade cujo campo de nome fechou neste quadro.
    pub(super) physics_edits: Vec<(u64, ph2d_editor_core::PhysicsFieldEdit)>,
    // §12 joints (W3). Kept out of `inspector_commits::dispatch`: that
    // signature is already the length its own doc-comment warns about,
    // and these two are applied in one short block below.
    pub(super) joint_edits: Vec<(u64, ph2d_editor_core::JointFieldEdit)>,
    pub(super) wheel_edits: Vec<(u64, ph2d_editor_core::WheelFieldEdit)>,
    // §14 Platform Player (W5). Sem fan-out, e pela razão da §12/§13: a
    // seção descreve UM personagem, o selecionado — espalhar um `Add`
    // pela seleção criaria N players num clique que pediu um.
    pub(super) player_edits: Vec<(u64, ph2d_editor_core::PlayerFieldEdit)>,
    // The pair to join, at most one per frame — it is a click, not a
    // per-entity edit.
    pub(super) bake_request: Option<Vec<u64>>,
    // W-J4: a rota por SELEÇÃO virou "ligue a sequência" (2 corpos = um
    // joint; N = uma corrente de N−1), então o pedido é um booleano — a
    // ordem vem da própria seleção, que o `join_selected_chain` lê.
    pub(super) join_chain: bool,
    pub(super) join_draw_arm: bool,
    // W-Rig: um clique em *Rig* — booleano pelo mesmo motivo do
    // `join_chain`, porque a SELEÇÃO já diz sobre o que ele age.
    pub(super) rig_now: bool,
    pub(super) visibility_section_edits: Vec<(u64, ph2d_editor_core::VisibilityFieldEdit)>,
    pub(super) name_edit: Option<ph2d_editor_core::InspectorNameInfo>,
    pub(super) signal_edit: Option<ph2d_editor_core::InspectorNameInfo>,
    pub(super) signal_leave_edit: Option<ph2d_editor_core::InspectorNameInfo>,
    pub(super) bgremoval_leftover: Vec<ph2d_editor_core::action_bus::EditorAction>,
    // Painter Apply leftover — same shape as bgremoval (drained
    // back into the bus so `image_edit::dispatch`'s
    // `painter_active` gate runs AFTER any same-frame
    // ActivateTool resolution). Day-7 ship.
    pub(super) painter_leftover: Vec<ph2d_editor_core::action_bus::EditorAction>,
    pub(super) inspector_selection: Vec<u64>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_bus_drain(&mut self) -> Option<DrainOut> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            sim, hero_screen, ..
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
        let mut pd = DrainOut {
            // ⚠️ **O osso seleccionado lê-se AQUI, antes de o mundo ser emprestado mutável** — os
            // verbos lá em baixo já seguram `sim`, e uma leitura de `self` no meio deles não
            // compila. O valor é do QUADRO, e é o mesmo que o gesto e o overlay usam.
            osso_selecionado: crate::bone_gesture::selected_bone(sim, hero.gizmo.iter_selected()),
            // ⭐ **A selecção CRUA, guardada aqui pela mesma razão que o `osso_selecionado`**: o
            // `hero` é uma vista do `gfx`, e quem a lê lá em baixo (o *Bind* da 2.ª mídia) já o
            // tem emprestado de outra maneira. *Ler o valor uma vez é o que torna a pergunta
            // alcançável nos dois sítios.*
            selecao_bits: hero.gizmo.iter_selected().collect(),
            // BulkSelect (T2.0): the live selection (primary + extras),
            // captured before the drain so an Inspector sprite edit can
            // fan out to every selected sprite. Only allocated for a
            // MULTI-selection; single-select takes the empty path and the
            // edit's own `entity_bits` (no per-frame alloc — audit D-5).
            inspector_selection: if hero.gizmo.selected_len() > 1 {
                hero.gizmo.iter_selected().collect()
            } else {
                Vec::new()
            },
            ..DrainOut::default()
        };
        // ⚠️ **A fila sai do `HeroScreen` pela duração do dreno**: o `drain` emprestava-a, e os sub-drenos são
        // métodos da `App`. Nenhum braço lhe escreve, e ela volta com a capacidade dela.
        let mut bus = std::mem::take(&mut hero.bus);
        for action in bus.drain() {
            // Pela ORDEM dos braços do `match` de sempre: cada sub-dreno devolve o pedido que não é dele, e o `_` da
            // barra do topo cala o que nenhum braço toma.
            let _ = self
                .fase_bus_tool_panel(action, &mut pd)
                .and_then(|a| self.fase_bus_timeline_panel(a))
                .and_then(|a| self.fase_bus_tool_requests(a, &mut pd))
                .and_then(|a| self.fase_bus_hierarchy_tree(a, &mut pd))
                .and_then(|a| self.fase_bus_hierarchy_rows(a, &mut pd))
                .and_then(|a| self.fase_bus_sprite_ops(a, &mut pd))
                .and_then(|a| self.fase_bus_inspector_sections(a, &mut pd))
                .and_then(|a| self.fase_bus_inspector_instance(a, &mut pd))
                .and_then(|a| self.fase_bus_physics_sections(a, &mut pd))
                .and_then(|a| self.fase_bus_inspector_identity(a, &mut pd))
                .and_then(|a| self.fase_bus_topbar(a));
        }
        let gfx = self.gfx.as_mut()?;
        FrameGfx::of(gfx).hero_screen.as_mut()?.bus = bus;
        Some(pd)
    }
}
