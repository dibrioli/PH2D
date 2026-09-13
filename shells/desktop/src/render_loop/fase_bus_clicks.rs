//! **O dreno do barramento — as três terças da cadeia do clique.** Braços do `match` da [`fase_bus_drain`](super), movidos pela
//! ordem de sempre; a única troca no corpo deles é `pd.<pedido>` onde escreviam `<pedido>`, e o `gfx` de cada
//! sub-dreno é re-derivado (o dreno só corre com ele). Ver o cabeçalho de lá.

use super::*;

impl crate::App {
    /// A 1.ª terça da cadeia do CLIQUE: a pilha de aparência, o blend e o morph, o osso e as suas restrições, o
    /// envelope, o texto e o padrão no caminho, o contorno e os filtros. Devolve se tomou o clique.
    pub(super) fn fase_bus_click_live(&mut self, id: &NodeId, pd: &mut DrainOut) -> bool {
        if let Some(j) = crate::vec_paint_stack::join_code_for_id(*id) {
            // ⭐ A QUINA do offset de CAD (v22) — um clique, não um valor.
            pd.pending_paint_join = Some(j);
        } else if let Some(v) = crate::vec_paint_stack::stack_verb_for_id(*id) {
            // ⭐ A PILHA DE APARÊNCIA: o resolvedor é PURO e vive ao lado dos
            // verbos, como o `vec_rotate_for_id` — aqui só se captura.
            pd.pending_paint_verb = Some(v);
        } else if *id == ph2d_panel_vector::ids::VECTOR_BLEND_RUN {
            // ADR-0128: cria o Blend Object VIVO da seleção (não o destrutivo).
            pd.pending_create_blend = true;
        } else if *id == ph2d_panel_vector::ids::VECTOR_BLEND_RESET_SPINE {
            // ADR-0128 C2b: volta o spine editado ao automático.
            pd.pending_reset_spine = true;
        } else if *id == ph2d_panel_vector::ids::VECTOR_BLEND_EXPAND {
            // ADR-0128 D: materializa os passos e descarta o objeto vivo.
            pd.pending_expand_blend = true;
        } else if *id == ph2d_panel_vector::ids::VECTOR_BLEND_RELEASE {
            // ADR-0128 D: desfaz o blend; as fontes ficam.
            pd.pending_release_blend = true;
        } else if *id == ph2d_panel_vector::ids::VECTOR_MORPH_RUN {
            // O irmão animável do blend: UMA forma, com o `t` keyável.
            pd.pending_create_morph = true;
        } else if *id == ph2d_editor_core::ids::VECTOR_BONE_BIND {
            // ⭐⭐⭐ O ESQUELETO (estudo 42 item 5): prende a seleção aos ossos.
            pd.pending_bone_bind = true;
        } else if *id == ph2d_editor_core::ids::VECTOR_BONE_IK_ADD {
            // ⭐⭐⭐ A ÂNCORA: dá ao osso em foco um alvo que a corrente persegue.
            pd.pending_ik_add = true;
        } else if *id == ph2d_editor_core::ids::VECTOR_BONE_IK_REMOVE {
            pd.pending_ik_remove = true;
        } else if *id == ph2d_editor_core::ids::VECTOR_BONE_LIMIT_ADD {
            // ⭐⭐⭐ O LIMITE DE ÂNGULO: até onde esta junta dobra.
            pd.pending_limit_add = true;
        } else if *id == ph2d_editor_core::ids::VECTOR_BONE_LIMIT_REMOVE {
            pd.pending_limit_remove = true;
        } else if *id == ph2d_editor_core::ids::VECTOR_BONE_SMART_ADD {
            // ⭐⭐⭐ O OSSO INTELIGENTE: anexa o controlo VAZIO — quem lhe dá acção é o painel.
            pd.pending_smart_add = true;
        } else if *id == ph2d_editor_core::ids::VECTOR_BONE_SMART_REMOVE {
            pd.pending_smart_remove = true;
        } else if *id == ph2d_editor_core::ids::VECTOR_BONE_SMART_PICK {
            // ⭐⭐⭐ Arma o gesto de duas mãos do ALVO: o clique seguinte, no
            // canvas OU na hierarquia, diz de que objecto este controlo trata.
            pd.pending_smart_pick = true;
        } else if let Some(i) = ph2d_editor_core::ids::VECTOR_BONE_SMART_CLIP_IDS
            .iter()
            .position(|x| x == id)
        {
            // ⭐⭐⭐ **QUAL acção** — a posição na tabela É o índice do clip, e é
            // ela que impede a lista pintada e a lista honrada de divergirem.
            pd.pending_smart_clip = Some(i);
        } else if let Some(i) = ph2d_editor_core::ids::VECTOR_BONE_BEND_IDS
            .iter()
            .position(|x| x == id)
        {
            // ⭐⭐⭐ **O LADO DA DOBRA** — a posição na tabela É a variante, e
            // é ela que impede a fileira e o vocabulário de divergirem. ⚠️ Um
            // `match` de três braços escritos à mão aqui seria a quinta lista
            // escrita à mão desta seção.
            pd.pending_ik_bend = ph2d_skeleton::BendSide::ALL.get(i).copied();
        } else if *id == ph2d_editor_core::ids::VECTOR_BONE_EXPAND {
            // Solta e fica com a pose de AGORA (o Expand do envelope).
            pd.pending_bone_release = Some(crate::skeleton_live::Keep::Deformed);
        } else if *id == ph2d_editor_core::ids::VECTOR_BONE_RELEASE {
            // Solta e devolve o que o artista DESENHOU.
            pd.pending_bone_release = Some(crate::skeleton_live::Keep::Source);
        } else if *id == ph2d_panel_vector::ids::VECTOR_ENVELOPE_RUN {
            // ADR-0129: envolve a seleção (1..N) num container com gaiola.
            pd.pending_create_envelope = true;
        } else if *id == ph2d_panel_vector::ids::VECTOR_ENVELOPE_EXPAND {
            // ADR-0129: a deformada vira o desenho; a gaiola morre.
            pd.pending_expand_envelope = true;
        } else if *id == ph2d_panel_vector::ids::VECTOR_ENVELOPE_RELEASE {
            // ADR-0129: a fonte autorada volta; a gaiola morre.
            pd.pending_release_envelope = true;
        } else if *id == ph2d_panel_vector::ids::VECTOR_ENVELOPE_PERSPECTIVE {
            // ADR-0129 Fatia D: a homografia -- lados RETOS.
            pd.pending_envelope_kind = Some(ph2d_ecs::EnvelopeKind::Perspective);
        } else if *id == ph2d_panel_vector::ids::VECTOR_ENVELOPE_MESH {
            // ADR-0129 Fatia D: o patch de Coons -- os lados DOBRAM.
            pd.pending_envelope_kind = Some(ph2d_ecs::EnvelopeKind::Mesh);
        } else if *id == ph2d_panel_vector::ids::VECTOR_ENVELOPE_PINS {
            // ADR-0129 Fatia E: o puppet warp (MLS-rigid).
            pd.pending_envelope_kind = Some(ph2d_ecs::EnvelopeKind::Pins);
        } else if *id == ph2d_panel_vector::ids::VECTOR_TEXTPATH_LINK {
            // Plano 22: prende o texto da seleção à outra forma dela.
            pd.pending_textpath = Some(crate::vec_text_ride::TextPathCmd::Link);
        } else if *id == ph2d_panel_vector::ids::VECTOR_TEXTPATH_PICK {
            // Picker: arma; a fonte (o texto em foco) é capturada no drain.
            pd.pending_text_pick = true;
        } else if *id == ph2d_panel_vector::ids::VECTOR_TEXTPATH_DETACH {
            pd.pending_textpath = Some(crate::vec_text_ride::TextPathCmd::Detach);
        } else if *id == ph2d_panel_vector::ids::VECTOR_TEXTPATH_FLIP {
            pd.pending_textpath = Some(crate::vec_text_ride::TextPathCmd::Flip(true));
        } else if *id == ph2d_panel_vector::ids::VECTOR_TEXTPATH_FLIP_OFF {
            pd.pending_textpath = Some(crate::vec_text_ride::TextPathCmd::Flip(false));
        } else if *id == ph2d_panel_vector::ids::VECTOR_PATTERNPATH_LINK {
            pd.pending_patternpath = Some(crate::pattern_live::PatternPathCmd::Link);
        } else if *id == ph2d_panel_vector::ids::VECTOR_PATTERNPATH_PICK {
            // Picker: arma; a fonte (o motivo selecionado) é capturada no drain.
            pd.pending_pp_pick = true;
        } else if *id == ph2d_panel_vector::ids::VECTOR_PATTERNPATH_DETACH {
            pd.pending_patternpath = Some(crate::pattern_live::PatternPathCmd::Detach);
        } else if *id == ph2d_panel_vector::ids::VECTOR_PATTERNPATH_FLIP {
            pd.pending_patternpath = Some(crate::pattern_live::PatternPathCmd::Flip(true));
        } else if *id == ph2d_panel_vector::ids::VECTOR_PATTERNPATH_FLIP_OFF {
            pd.pending_patternpath = Some(crate::pattern_live::PatternPathCmd::Flip(false));
        } else if *id == ph2d_panel_vector::ids::VECTOR_CONTOUR_ADD {
            pd.pending_contour = Some(crate::contour_live::ContourCmd::Add);
        } else if *id == ph2d_panel_vector::ids::VECTOR_CONTOUR_REMOVE {
            pd.pending_contour = Some(crate::contour_live::ContourCmd::Remove);
        } else if *id == ph2d_panel_vector::ids::VECTOR_CONTOUR_EXPAND {
            pd.pending_contour = Some(crate::contour_live::ContourCmd::Expand);
        } else if let Some(code) = crate::contour_live::join_code_of_id(*id) {
            pd.pending_contour_join = Some(code);
        } else if let Some(code) = crate::contour_live::side_code_of_id(*id) {
            pd.pending_contour_side = Some(code);
        } else if let Some(hit) = crate::fx_live::hit_of(*id) {
            pd.pending_filter_cmd = Some(hit);
        } else if let Some(hit) = crate::fx_bridge_dispatch::classify_click(*id) {
            match hit {
                crate::fx_bridge_dispatch::FxClick::Add(k) => {
                    pd.pending_fx_add = Some(k);
                }
                crate::fx_bridge_dispatch::FxClick::Row(r, a) => {
                    pd.pending_fx_button = Some((r, a));
                }
                crate::fx_bridge_dispatch::FxClick::Apply => {
                    pd.pending_fx_apply = true;
                }
            }
        } else {
            return false;
        }
        true
    }

    /// A 2.ª terça: os presets, a booleana viva, a moldura e o painel autorado, o auto layout, o Resize Box e o
    /// traço, os componentes, os estados de UI, as âncoras, os tokens e o morph. Devolve se tomou o clique.
    pub(super) fn fase_bus_click_layout(&mut self, id: &NodeId, pd: &mut DrainOut) -> Option<bool> {
        let gfx = self.gfx.as_mut()?;
        let FrameGfx { hero_screen, .. } = FrameGfx::of(gfx);
        let hero = hero_screen.as_mut()?;
        if *id == ph2d_panel_vector::ids::VECTOR_ENVELOPE_CLEAR_PINS {
            pd.pending_clear_pins = true;
        } else if let Some(i) = (0..ph2d_panel_vector::ids::MAX_ENVELOPE_PRESETS)
            .find(|&i| *id == ph2d_panel_vector::ids::vector_envelope_preset_id(i))
        {
            // ADR-0129 Fatia C: carimba o preset `i` na gaiola.
            pd.pending_envelope_preset = Some(i);
        } else if let Some(i) = (0..ph2d_panel_vector::ids::MAX_WIDTH_PRESETS)
            .find(|&i| *id == ph2d_panel_vector::ids::vector_width_preset_id(i))
        {
            // W2b: escolhe a FORMA da largura (o catálogo de perfis).
            pd.pending_width_preset = Some(i);
        } else if *id == ph2d_panel_vector::ids::VECTOR_BOOL_LIVE_OFF
            || *id == ph2d_panel_vector::ids::VECTOR_BOOL_LIVE_ON
        {
            // O MODO dos oito botões. Panel-local no valor, mas quem o lê no
            // clique de uma das oito é a shell — por isso ele passa por aqui.
            ph2d_panel_vector::state::set_bool_live_on(
                *id == ph2d_panel_vector::ids::VECTOR_BOOL_LIVE_ON,
            );
        } else if let Some(code) = crate::vec_bool_shape::shape_op_for_id(*id) {
            // **O VERBO DESTA FORMA.** ⚠️ O mapeamento saiu daqui para uma
            // porta testavel (`vec_bool_shape::shape_op_for_id`): um `match`
            // de id enterrado neste arquivo nao e' alcancavel por teste
            // nenhum, e foi essa a causa-raiz de os quatro chips shiparem
            // sem um unico gate no caminho `id -> componente escrito`.
            pd.pending_bool_shape_op = Some(code);
        } else if *id == ph2d_panel_vector::ids::VECTOR_FRAME_PANEL_OFF
            || *id == ph2d_panel_vector::ids::VECTOR_FRAME_PANEL_ON
        {
            // **O painel AUTORADO** (plano UI/UX W8b.2). ⚠️ Aplicado AQUI, e
            // nao por um `pending_*` como os vizinhos: os vizinhos escrevem no
            // COMPONENTE (mundo), e este escreve a visibilidade do painel, que
            // e' um fato do `HeroScreen` — que esta' em maos exactamente aqui.
            // Um pending o adiaria para um escopo que teria de re-emprestar o
            // hero para dizer a mesma coisa.
            hero.panel_visibility.insert(
                ph2d_panel_authored::visibility_key(),
                *id == ph2d_panel_vector::ids::VECTOR_FRAME_PANEL_ON,
            );
        } else if *id == ph2d_panel_vector::ids::VECTOR_FRAME_CLIP_OFF
            || *id == ph2d_panel_vector::ids::VECTOR_FRAME_CLIP_ON
        {
            // A MOLDURA recorta ou não. O valor mora no COMPONENTE (mundo),
            // então o clique é da shell — o painel só mostra.
            pd.pending_frame_clip = Some(*id == ph2d_panel_vector::ids::VECTOR_FRAME_CLIP_ON);
        } else if let Some(e) = crate::vec_layout_edit::layout_edit_for_id(*id) {
            // O AUTO LAYOUT (plano UI/UX W2): direção, alinhamento e
            // distribuição moram no COMPONENTE, então o clique e' da shell —
            // o painel so' mostra qual chip esta' aceso.
            pd.pending_layout_edit = Some(e);
        } else if *id == ph2d_panel_vector::ids::VECTOR_TRANSFORM_RESIZE_BOX {
            // **Resize Box** (plano UI/UX W3b): o override mora no COMPONENTE,
            // entao o clique e' da shell — o painel so' mostra o estado.
            pd.pending_resize_box = true;
        } else if *id == ph2d_panel_vector::ids::VECTOR_STROKE_PRESENT {
            // **Stroke** (plano 34): dar ou tirar o traço mexe no DOCUMENTO,
            // então o clique e' da shell — o painel so' mostra o estado.
            pd.pending_stroke_present = true;
        } else if let Some(e) = crate::vec_component_edit::component_edit_for_id(*id) {
            // OS COMPONENTES (plano UI/UX W5): mestre e instância moram no
            // ECS, entao o clique e' da shell — o painel so' mostra que
            // verbos fazem sentido.
            pd.pending_component = Some(e);
        } else if let Some(e) = crate::vec_ui_state_edit::ui_state_edit_for_id(*id) {
            // OS ESTADOS de UI (W7): gravar, mostrar e esquecer uma pose.
            pd.pending_ui_state = Some(e);
        } else if let Some(p) = crate::vec_ui_state_edit::easing_pick_for_id(*id) {
            // **O SELETOR DE CURVA** (W7): a forma e a direcao da transicao.
            pd.pending_ui_easing = Some(p);
        } else if let Some(e) = crate::vec_ui_state_edit::signal_edit_for_id(*id) {
            // ⭐ **A TABELA SINAL → PAPEL**: a ligação mora no DOCUMENTO
            // (`HostStates.on_signal`), então os três gestos atravessam o
            // barramento como os verbos ao lado.
            pd.pending_ui_signal_edit = Some(e);
        } else if *id == ph2d_panel_vector::ids::VECTOR_STATE_SPRING {
            // **A MOLA** (W7m): ela troca o MOTOR da transição, e o motor mora
            // na tabela do documento — então o checkbox atravessa o barramento
            // como os verbos ao lado.
            pd.pending_ui_spring_toggle = true;
        } else if *id == ph2d_panel_vector::ids::VECTOR_STATE_MOVE_ALL {
            // **Mover o widget com TODOS os estados** (W7r): quem desloca é a
            // shell — só ela vê o `Transform` andar —, então o toggle
            // atravessa o barramento como o interruptor de preview ao lado.
            pd.pending_ui_move_all_toggle = true;
        } else if *id == ph2d_panel_vector::ids::VECTOR_STATE_PREVIEW {
            // **O MODO DE PREVIEW** (W7r): ele NÃO é um verbo de estado — não
            // toca a tabela —, então tem rota própria em vez de um variant no
            // `UiStateEdit`, cujo assunto é *o que muda no documento*.
            pd.pending_ui_preview_toggle = true;
        } else if let Some(e) = crate::vec_widget_edit::widget_edit_for_id(*id) {
            // A PELE por-widget (plano UI/UX W6.2): o componente mora no ECS,
            // entao o clique e' da shell — o painel so' mostra que tipo esta'
            // aceso e que verbo faz sentido.
            pd.pending_widget_edit = Some(e);
        } else if let Some(e) = crate::vec_anchor_edit::anchor_edit_for_id(*id) {
            // AS ÂNCORAS (plano UI/UX W3): o par de âncoras mora no
            // COMPONENTE, e a RÉGUA e' capturada do lado da shell — que e'
            // quem mede a moldura. O painel so' mostra qual chip esta' aceso.
            pd.pending_anchor_edit = Some(e);
        } else if let Some(choice) = crate::vec_bindings::token_choice(*id) {
            // Uma escolha do picker de token. O valor mora no COMPONENTE
            // (mundo), então o clique é da shell — o painel só mostra.
            pd.pending_token_bind = Some(choice);
        } else if let Some(p) = ph2d_tool_vector::frames::device_preset(*id) {
            // Um preset é uma 2ª forma de PEDIR a edição de W/H — ele cai na
            // MESMA porta que os campos numéricos do Transform.
            pd.pending_frame_preset = Some(p);
        } else if *id == ph2d_panel_vector::ids::VECTOR_MORPH_PREVIEW {
            // ⭐⭐ **O MODO em que o teclado é da máquina** (plano 32 W9). Ele
            // NÃO é um verbo de seta — não toca o grafo —, então tem rota
            // própria em vez de um variant no `MorphCmd`, cujo assunto é *o que
            // muda no documento*. É a mesma separação do irmão das poses.
            pd.pending_morph_preview_toggle = true;
        } else if let Some(cmd) = crate::vec_morph_edit::morph_cmd_for_id(*id) {
            // ⭐ A seção MORPH STATES (plano 32 W4/W8): fazer o conjunto, ou
            // escolher a acção que dispara uma transição. As duas mexem no
            // MUNDO, então o clique é da shell — o painel só mostra.
            pd.pending_morph_arrow = Some(cmd);
        } else {
            return Some(false);
        }
        Some(true)
    }

    /// A última terça: a booleana, os nós e os caminhos, o arranjo, a tinta e o padrão, o gradiente, o composto, o
    /// snap, as réguas e o texto.
    pub(super) fn fase_bus_click_document(&mut self, id: &NodeId, pd: &mut DrainOut) {
        if *id == ph2d_panel_vector::ids::VECTOR_BOOL_APPLY {
            pd.pending_bool_apply = true;
        } else if let Some(op) = crate::input_dispatch::vec_bool_op_for_id(*id) {
            pd.pending_vec_bool = Some(op);
        } else if let Some(cmd) = crate::vec_expand::expand_for_id(*id) {
            pd.pending_vec_expand = Some(cmd);
        } else if let Some(kind) = crate::input_dispatch::vec_vertex_kind_for_id(*id) {
            pd.pending_vec_vertex_kind = Some(kind);
        } else if *id == ph2d_panel_vector::ids::VECTOR_VERT_DELETE {
            pd.pending_vec_delete_vertex = true;
        } else if *id == ph2d_panel_vector::ids::VECTOR_VERT_SEL_SUBPATH {
            pd.pending_vec_select_subpath = true;
        } else if *id == ph2d_panel_vector::ids::VECTOR_VERT_SEL_SAME {
            pd.pending_vec_select_same = true;
        } else if *id == ph2d_panel_vector::ids::VECTOR_PATH_JOIN {
            pd.pending_vec_join = true;
        } else if *id == ph2d_panel_vector::ids::VECTOR_PATH_WELD {
            pd.pending_vec_weld = true;
        } else if *id == ph2d_panel_vector::ids::VECTOR_PATH_REVERSE {
            pd.pending_vec_reverse = true;
        } else if *id == ph2d_panel_vector::ids::VECTOR_VERT_AVERAGE {
            pd.pending_vec_average = true;
        } else if *id == ph2d_panel_vector::ids::VECTOR_CUT_APPLY {
            pd.pending_vec_cut = true;
        } else if *id == ph2d_tool_vector::ids::VECTOR_SYM_APPLY {
            pd.pending_vec_symmetry_apply = true;
        } else if *id == ph2d_panel_vector::ids::VECTOR_CUT_DISCARD {
            pd.pending_vec_cut_discard = true;
        } else if let Some(order) = crate::input_dispatch::vec_reorder_for_id(*id) {
            pd.pending_vec_reorder = Some(order);
        } else if *id == ph2d_panel_vector::ids::VECTOR_ARRANGE_DUPLICATE {
            pd.pending_vec_duplicate = true;
        } else if let Some(axis) = crate::input_dispatch::vec_flip_for_id(*id) {
            pd.pending_vec_flip = Some(axis);
        } else if let Some(dir) = crate::input_dispatch::vec_rotate_for_id(*id) {
            pd.pending_vec_rotate = Some(dir);
        } else if let Some(op) = crate::input_dispatch::vec_path_shape_for_id(*id) {
            pd.pending_vec_path_shape = Some(op);
        } else if *id == ph2d_panel_vector::ids::VECTOR_PIVOT_EDIT {
            pd.pending_vec_pivot_edit = true;
        } else if *id == ph2d_panel_vector::ids::VECTOR_PATH_CLOSE {
            pd.pending_vec_toggle_closed = true;
        } else if let Some(k) = crate::input_dispatch::vec_fill_kind_for_id(*id) {
            pd.pending_vec_fill_kind = Some(k);
        } else if *id == ph2d_panel_vector::ids::VECTOR_BRUSH_PICK_SHAPE {
            // ⭐ Arma; a FONTE (a forma com o pincel) é capturada no drain,
            // porque o clique seguinte muda a seleção.
            pd.pending_brush_pick = true;
        } else if let Some(c) = crate::vec_stroke_paint::cmd_for_id(*id) {
            pd.pending_brush = Some(c);
        } else if let Some(k) = crate::vec_stroke_paint::kind_for_id(*id) {
            // ⭐ A TINTA do traço (plano 35, wave D). Ela mexe no DOCUMENTO,
            // entao o clique e' da shell — o painel so' mostra qual chip acende.
            pd.pending_vec_stroke_kind = Some(k);
        } else if let Some((slot, knob)) = ph2d_panel_vector::texture_pattern::texpat_knob_of(*id) {
            // ⭐⭐ **O SUJEITO VEM NO PRÓPRIO ID** (plano 35, wave F). Cada
            // secção tem os seus controlos, então o clique já **diz** em qual
            // das duas tintas escrever — e a preferência de sessão que a wave D
            // precisava (`texpat_target`) deixou de existir, com a classe
            // inteira de *"mexi num knob e mudou o outro sujeito"*.
            use ph2d_panel_vector::ids::TexPatKnob as K;
            let slot = if slot == 1 {
                ph2d_vec_render::PatternSlot::Stroke
            } else {
                ph2d_vec_render::PatternSlot::Fill
            };
            match knob {
                K::Tile(i) => {
                    pd.pending_texpat =
                        Some((slot, crate::texture_pattern_edit::TexPatCmd::Tile(i)));
                }
                K::Mode(i) => {
                    pd.pending_texpat =
                        Some((slot, crate::texture_pattern_edit::TexPatCmd::Mode(i)));
                }
                K::Source => pd.pending_texpat_source = Some(slot),
                // Picker (W7): arma; a FONTE (a forma com o padrão) é capturada
                // no drain, porque o clique seguinte muda a seleção.
                K::PickShape => pd.pending_texpat_pick = Some(slot),
                // ⭐ O CADEADO é estado de SESSÃO (o gesto, não o padrão): o
                // clique inverte-o aqui e nada toca no documento.
                // ⚠️ Indexado pelo SLOT: as duas tintas têm cadeados
                // independentes, e partilhá-los era o defeito.
                K::Lock => {
                    let i = usize::from(slot == ph2d_vec_render::PatternSlot::Stroke);
                    self.texpat_lock_aspect[i] = !self.texpat_lock_aspect[i];
                }
                // ⭐ O elo dos VÃOS — mesmo desenho, mesmo índice por slot.
                K::GapLink => {
                    let i = usize::from(slot == ph2d_vec_render::PatternSlot::Stroke);
                    self.texpat_gap_link[i] = !self.texpat_gap_link[i];
                }
                _ => {}
            }
        } else if *id == ph2d_panel_vector::ids::VECTOR_GRAD_ADD_POINT {
            pd.pending_vec_grad_add = true;
        } else if *id == ph2d_panel_vector::ids::VECTOR_GRAD_REMOVE_POINT {
            pd.pending_vec_grad_remove = true;
        } else if *id == ph2d_panel_vector::ids::VECTOR_GRAD_ADD_STOP {
            pd.pending_vec_grad_add_stop = true;
        } else if *id == ph2d_panel_vector::ids::VECTOR_GRAD_REMOVE_STOP {
            pd.pending_vec_grad_remove_stop = true;
        } else if let Some(a) = crate::input_dispatch::vec_align_for_id(*id) {
            pd.pending_vec_align = Some(a);
        } else if let Some(d) = crate::input_dispatch::vec_distribute_for_id(*id) {
            pd.pending_vec_distribute = Some(d);
        } else if *id == ph2d_editor_core::ids::VECTOR_COMPOUND_MAKE {
            pd.pending_vec_compound = Some(true);
        } else if *id == ph2d_panel_vector::ids::VECTOR_COMPOUND_RELEASE {
            pd.pending_vec_compound = Some(false);
        } else if *id == ph2d_panel_vector::ids::VECTOR_FILL_RULE_NONZERO {
            pd.pending_vec_fill_rule = Some(false);
        } else if *id == ph2d_panel_vector::ids::VECTOR_FILL_RULE_EVENODD {
            pd.pending_vec_fill_rule = Some(true);
        } else if *id == ph2d_panel_vector::ids::VECTOR_SNAP_OFF {
            pd.pending_vec_snap_on = Some(false);
        } else if *id == ph2d_editor_core::ids::VECTOR_SNAP_ON {
            pd.pending_vec_snap_on = Some(true);
        } else if *id == ph2d_panel_vector::ids::VECTOR_SNAP_PATH_OFF {
            pd.pending_vec_snap_path = Some(false);
        } else if *id == ph2d_panel_vector::ids::VECTOR_SNAP_PATH_ON {
            pd.pending_vec_snap_path = Some(true);
        } else if *id == ph2d_panel_vector::ids::VECTOR_SNAP_CROSS_OFF {
            pd.pending_vec_snap_cross = Some(false);
        } else if *id == ph2d_panel_vector::ids::VECTOR_SNAP_CROSS_ON {
            pd.pending_vec_snap_cross = Some(true);
        } else if *id == ph2d_panel_vector::ids::VECTOR_SNAP_GUIDES_OFF {
            pd.pending_vec_snap_guides = Some(false);
        } else if *id == ph2d_panel_vector::ids::VECTOR_SNAP_GUIDES_ON {
            pd.pending_vec_snap_guides = Some(true);
        } else if *id == ph2d_panel_vector::ids::VECTOR_RULERS_OFF {
            pd.pending_rulers = Some(false);
        } else if *id == ph2d_panel_vector::ids::VECTOR_RULERS_ON {
            pd.pending_rulers = Some(true);
        } else if *id == ph2d_panel_vector::ids::VECTOR_TEXT_FONT_PREV {
            pd.pending_vec_font_cycle = Some(-1);
        } else if *id == ph2d_panel_vector::ids::VECTOR_TEXT_FONT_NEXT {
            pd.pending_vec_font_cycle = Some(1);
        } else if *id == ph2d_panel_vector::ids::VECTOR_TEXT_FONT_IMPORT {
            pd.pending_vec_font_import = true;
        } else if *id == ph2d_panel_vector::ids::VECTOR_TEXT_ALIGN_LEFT {
            pd.pending_vec_text_align = Some(ph2d_vec_text::TextAlign::Left);
        } else if *id == ph2d_panel_vector::ids::VECTOR_TEXT_ALIGN_CENTER {
            pd.pending_vec_text_align = Some(ph2d_vec_text::TextAlign::Center);
        } else if *id == ph2d_panel_vector::ids::VECTOR_TEXT_ALIGN_RIGHT {
            pd.pending_vec_text_align = Some(ph2d_vec_text::TextAlign::Right);
        } else if *id == ph2d_panel_vector::ids::VECTOR_TEXT_WRAP_AUTO {
            pd.pending_vec_text_wrap = Some(None);
        } else if *id == ph2d_panel_vector::ids::VECTOR_TEXT_WRAP_FIXED {
            // ⚠️ **Fixed semeia com a largura que o texto JÁ mede**, e não com
            // um número de fábrica: clicar Fixed não pode mover um glifo — ele
            // só torna o número editável. Sem sessão viva não há texto a medir,
            // e aí cai no default do slider.
            pd.pending_vec_text_wrap = Some(Some(
                crate::vec_text::seed_wrap_width(self.vec.text_edit.as_ref())
                    .unwrap_or(ph2d_tool_vector::params::DEFAULT_TEXT_WRAP),
            ));
        } else if *id == ph2d_panel_vector::ids::VECTOR_CONVERT_TO_CURVES {
            pd.pending_vec_convert = true;
        }
    }
}
