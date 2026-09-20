//! **O dreno do barramento — o canal painel → ferramenta.** Braços do `match` da [`fase_bus_drain`](super), movidos pela
//! ordem de sempre; a única troca no corpo deles é `pd.<pedido>` onde escreviam `<pedido>`, e o `gfx` de cada
//! sub-dreno é re-derivado (o dreno só corre com ele). Ver o cabeçalho de lá.

use super::*;
// ⚠️ O `IkKnob` mora na FASE que o drena, não no índice — ver o cabeçalho dele.
use super::fase_bone_smart_and_knobs::IkKnob;
use ph2d_editor_core::action_bus::EditorAction;

impl crate::App {
    /// ⭐ **O canal painel → ferramenta** (ADR-0040 TG-A/TG-B): a activação genérica de uma ferramenta, e o
    /// `PanelEvent` que um painel de ferramenta levanta — os cliques e os campos que são comandos do DOCUMENTO
    /// viram pedidos, e o evento segue, por valor, para a ferramenta activa.
    pub(super) fn fase_bus_tool_panel(
        &mut self,
        action: EditorAction,
        pd: &mut DrainOut,
    ) -> Option<EditorAction> {
        match action {
            // ADR-0040 TG-A: generic activation. Per-tool flags
            // preserve the existing mode_on gating / activation
            // side effects after the drain.
            // ADR-0040 TG-A: generic activation. Audit F1 (2026-05-26):
            // data-driven via cluster lookup no drain abaixo; sem
            // per-tool flag flooding.
            EditorAction::ActivateTool { tool_id } => {
                pd.pending_image_tool_activation = Some(tool_id);
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
                    pd.pending_bone_needs_focus |= ph2d_editor_core::ids::needs_focused_bone(*id);
                    // A cadeia do clique, em três terças pela ordem de sempre: cada uma devolve se tomou o clique.
                    if !self.fase_bus_click_live(id, pd) && !self.fase_bus_click_layout(id, pd)? {
                        self.fase_bus_click_document(id, pd);
                    }
                }
                // Transform fields (X/Y/W/H) are numeric SetValue document
                // commands (not tool Style) — capture; the tool ignores them.
                if let ph2d_editor_core::tool::PanelEvent::SetValue(id, v) = &ev {
                    // ⭐ **Os CAMPOS entram pela mesma porta derivada que os cliques** — sem
                    // isto, digitar num campo desta seção sem osso em foco continuava a ser
                    // um silêncio sem explicação, que é metade da população da secção.
                    pd.pending_bone_needs_focus |= ph2d_editor_core::ids::needs_focused_bone(*id);
                    // E a do campo, em duas metades da mesma maneira.
                    if !self.fase_bus_value_bones_and_text(id, v, pd) {
                        self.fase_bus_value_live(id, v, pd);
                    }
                }
                self.fase_bus_tool_panel_forward(ev, pd)?;
            }
            other => return Some(other),
        }
        None
    }

    /// O resto do `PanelEvent`: os três canais de texto (`SelectOption`), o Colorize, as camadas e a tira do Flip,
    /// e o evento, por valor, para a ferramenta activa.
    fn fase_bus_tool_panel_forward(
        &mut self,
        ev: ph2d_editor_core::tool::PanelEvent,
        pd: &mut DrainOut,
    ) -> Option<()> {
        let gfx = self.gfx.as_mut()?;
        let FrameGfx { tools, flip, .. } = FrameGfx::of(gfx);
        // ⭐ **O NOME de uma ligação sinal → papel**: `SelectOption(campo,
        // "<texto>")`. O texto é o que o artista digitou, e vem por este canal
        // porque o `PanelEvent` é contrato CONGELADO — o `SelectOption` já é o
        // canal string-valued deste app (o Painter carrega nele
        // `"layer:channel:index:x:y"`, que não é opção de rádio nenhuma).
        if let ph2d_editor_core::tool::PanelEvent::SelectOption(id, val) = &ev
            && let Some(row) = crate::vec_ui_state_edit::signal_name_row(*id)
        {
            pd.pending_ui_signal_name = Some((row, val.clone()));
        }
        // Font dropdown pick: `SelectOption(chip, "<index>")` → the
        // family index into `vec_font::pickable_families()`.
        if let ph2d_editor_core::tool::PanelEvent::SelectOption(id, val) = &ev
            && *id == ph2d_panel_vector::ids::VECTOR_TEXT_FONT_DD
        {
            pd.pending_vec_font_pick = val.parse::<usize>().ok();
        }
        // O punho de um stop da rampa: `SelectOption(trilho, "linha:idx:x")` — o
        // dispatch de 2D já converteu o ponteiro contra a barra, então o `x` chega
        // normalizado. O formato espelha o do editor de falloff do Painter.
        if let ph2d_editor_core::tool::PanelEvent::SelectOption(id, val) = &ev
            && (0..ph2d_panel_vector::ids::MAX_FILTER_ROWS)
                .any(|r| *id == ph2d_panel_vector::ids::filter_ramp_id(r))
        {
            let mut parts = val.split(':');
            if let (Some(Ok(row)), Some(Ok(idx)), Some(Ok(x))) = (
                parts.next().map(str::parse::<usize>),
                parts.next().map(str::parse::<u8>),
                parts.next().map(str::parse::<f32>),
            ) {
                pd.pending_filter_stop = Some((row, idx, x));
                // ⚠️ **Diagnóstico de UM elo, atrás de env.** O report *"não é
                // possível arrastar os pontos de cor"* não reproduz headless — o
                // gate de seam dirige o gesto REAL e chega ao barramento —, então o
                // que falta medir é o que só o app vivo tem: se esta linha imprime,
                // o painel entregou e o defeito está a jusante; se não imprime, o
                // evento nunca chegou (e o `[hero] unhandled event` o dirá).
                if std::env::var_os("PH2D_FX_RAMP_DIAG").is_some() {
                    eprintln!("[ramp] painel entregou: linha {row} stop {idx} -> x {x:.4}");
                }
            }
        }
        // ADR-0114 C2: Colorize Apply/Clear — mexem no buffer de rabiscos do
        // shell + no doc, e o `self.gfx` está preso pelo borrow deste bloco;
        // marca-se um pending no `self` (campo disjunto) e aplica-se no topo
        // do PRÓXIMO frame, com `self` livre (latência de 1 frame, imperceptível
        // num botão).
        if let ph2d_editor_core::tool::PanelEvent::Click(id) = &ev {
            if *id == ph2d_panel_flip::ids::FLIP_COLORIZE_APPLY {
                self.flip_state.pending_colorize_apply = true;
            } else if *id == ph2d_panel_flip::ids::FLIP_COLORIZE_CLEAR {
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
        Some(())
    }

    /// A 1.ª metade da cadeia do CAMPO: o Transform, a aparência e a pilha, o osso e as suas restrições, o nó, a mola
    /// e a duração dos estados, o Z, o layout, o gradiente e o texto. Devolve se tomou o valor.
    fn fase_bus_value_bones_and_text(&mut self, id: &NodeId, v: &f64, pd: &mut DrainOut) -> bool {
        if let Some(field) = crate::input_dispatch::vec_transform_field_for_id(*id) {
            pd.pending_vec_transform = Some((field, *v));
        } else if *id == ph2d_panel_vector::ids::VECTOR_OBJ_OPACITY {
            pd.pending_vec_opacity = Some(*v);
        } else if *id == ph2d_panel_vector::ids::VECTOR_OBJ_BLEND {
            // ⚠️ O valor é o CÓDIGO do modo (`BlendMode::to_u8`), e não a linha
            // do popover: a lista é derivada da tradução para o Vello, e
            // reconstruí-la aqui seria a segunda cópia dela.
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let code = v.clamp(0.0, f64::from(u8::MAX)) as u8;
            pd.pending_vec_blend = Some(code);
        } else if *id == ph2d_panel_vector::ids::VECTOR_PAINT_WIDTH {
            pd.pending_paint_width = Some(*v);
        } else if *id == ph2d_panel_vector::ids::VECTOR_PAINT_DX {
            pd.pending_paint_dx = Some(*v);
        } else if *id == ph2d_panel_vector::ids::VECTOR_PAINT_DY {
            pd.pending_paint_dy = Some(*v);
        } else if *id == ph2d_panel_vector::ids::VECTOR_PAINT_DILATE {
            pd.pending_paint_dilate = Some(*v);
        } else if *id == ph2d_panel_vector::ids::VECTOR_PAINT_OPACITY {
            pd.pending_paint_opacity = Some(*v);
        } else if *id == ph2d_panel_vector::ids::VECTOR_PAINT_BLEND {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let code = v.clamp(0.0, f64::from(u8::MAX)) as u8;
            pd.pending_paint_blend = Some(code);
        } else if let Some(k) = ph2d_app_skeleton::knobs::of_id(*id) {
            // ⭐ Os números do OSSO (estudo 42 item 5, mais os cinco da F8). Eles
            // vivem num componente da entidade, então quem escreve é a shell — a
            // mesma rota dos campos do Transform e do layout.
            pd.pending_bone_knob = Some((k, *v));
        } else if *id == ph2d_editor_core::ids::VECTOR_BONE_IK_MIX {
            pd.pending_ik_knob = Some((IkKnob::Mix, *v));
        } else if *id == ph2d_editor_core::ids::VECTOR_BONE_IK_SOFTNESS {
            pd.pending_ik_knob = Some((IkKnob::Softness, *v));
        } else if *id == ph2d_editor_core::ids::VECTOR_BONE_IK_CHAIN {
            pd.pending_ik_knob = Some((IkKnob::Chain, *v));
        } else if *id == ph2d_editor_core::ids::VECTOR_BONE_IK_OFFSET {
            // ⭐ O desvio do apontar. ⚠️ O valor chega em GRAUS (é o que o campo mostra) e o dreno
            // converte — a mesma lei do limite da junta, duas linhas abaixo.
            pd.pending_ik_knob = Some((IkKnob::Offset, *v));
        } else if *id == ph2d_editor_core::ids::VECTOR_BONE_LIMIT_MIN
            || *id == ph2d_editor_core::ids::VECTOR_BONE_LIMIT_MAX
        {
            pd.pending_limit_knob = Some((*id == ph2d_editor_core::ids::VECTOR_BONE_LIMIT_MAX, *v));
        } else if *id == ph2d_editor_core::ids::VECTOR_BONE_SMART_FROM
            || *id == ph2d_editor_core::ids::VECTOR_BONE_SMART_TO
        {
            pd.pending_smart_knob = Some((*id == ph2d_editor_core::ids::VECTOR_BONE_SMART_TO, *v));
        } else if *id == ph2d_panel_vector::ids::VECTOR_VERT_X {
            pd.pending_vec_vert = Some((false, *v));
        } else if *id == ph2d_panel_vector::ids::VECTOR_VERT_Y {
            pd.pending_vec_vert = Some((true, *v));
        } else if *id == ph2d_panel_vector::ids::VECTOR_STATE_DURATION {
            // W7: o track `0..1` vira SEGUNDOS pela régua do modelo. A
            // conversão mora aqui e não no painel porque o número autorado é
            // do documento — o painel só o mostra.
            pd.pending_ui_state_duration = Some(*v * ph2d_ui_state::MAX_DURATION_S);
        } else if *id == ph2d_panel_vector::ids::VECTOR_STATE_STIFFNESS
            || *id == ph2d_panel_vector::ids::VECTOR_STATE_DAMPING
        {
            // W7m: o track `0..1` vira o número autorado pela régua AFIM do
            // modelo — as duas não começam em zero, então o offset é parte da
            // conversão. Ela mora aqui pela mesma razão da duração: o número
            // é do documento, e o painel só o mostra.
            let stiff = *id == ph2d_panel_vector::ids::VECTOR_STATE_STIFFNESS;
            let (lo, hi) = if stiff {
                (ph2d_ui_state::MIN_STIFFNESS, ph2d_ui_state::MAX_STIFFNESS)
            } else {
                (ph2d_ui_state::MIN_DAMPING, ph2d_ui_state::MAX_DAMPING)
            };
            pd.pending_ui_spring_knob = Some((stiff, lo + *v * (hi - lo)));
        } else if *id == ph2d_panel_vector::ids::VECTOR_ARRANGE_Z {
            pd.pending_vec_z = Some(*v);
        } else if let Some(f) = crate::vec_layout_edit::layout_field_for_id(*id) {
            // Vao, recuo, Grow e Shrink — mesma rota dos campos do Transform:
            // o valor mora no componente, entao quem escreve e' a shell.
            pd.pending_layout_field = Some((f, *v));
        } else if *id == ph2d_panel_vector::ids::VECTOR_TRANSFORM_R {
            pd.pending_vec_rotate_by = Some(*v);
        } else if *id == ph2d_panel_vector::ids::VECTOR_GRAD_ANGLE {
            // Slider carries the track 0..1 → 0..360°.
            pd.pending_vec_grad_angle = Some(*v * 360.0);
        } else if *id == ph2d_panel_vector::ids::VECTOR_GRAD_INFLUENCE {
            // Track 0..1 → influence 0..4.
            pd.pending_vec_grad_influence = Some(*v * 4.0);
        } else if *id == ph2d_panel_vector::ids::VECTOR_GRAD_JITTER {
            // Track 0..1 → jitter 0..1 (already a fraction).
            pd.pending_vec_grad_jitter = Some(*v);
        } else if *id == ph2d_panel_vector::ids::VECTOR_TEXT_SIZE {
            // Track 0..1 → glyph size (world units); shared mapping.
            pd.pending_vec_text_size =
                Some(ph2d_tool_vector::params::slider_to_text_size(*v as f32));
        } else if *id == ph2d_panel_vector::ids::VECTOR_TEXT_WEIGHT {
            // Track 0..1 → font weight (wght); shared mapping.
            pd.pending_vec_text_weight =
                Some(ph2d_tool_vector::params::slider_to_text_weight(*v as f32) as f32);
        } else if *id == ph2d_panel_vector::ids::VECTOR_TEXT_LINE_HEIGHT {
            // Track 0..1 → line height (× size); shared mapping.
            pd.pending_vec_text_line_height = Some(
                ph2d_tool_vector::params::slider_to_text_line_height(*v as f32),
            );
        } else if *id == ph2d_panel_vector::ids::VECTOR_TEXT_WRAP_W {
            // Track 0..1 -> largura de refluxo (mundo); shared mapping.
            pd.pending_vec_text_wrap = Some(Some(ph2d_tool_vector::params::slider_to_text_wrap(
                *v as f32,
            )));
        } else if *id == ph2d_panel_vector::ids::VECTOR_TEXT_TRACKING {
            // Track 0..1 → tracking (em fraction); shared mapping.
            pd.pending_vec_text_tracking =
                Some(ph2d_tool_vector::params::slider_to_text_tracking(*v as f32));
        } else {
            return false;
        }
        true
    }

    /// A 2.ª metade: o conector, a forma viva, o blend, o texto e o padrão no caminho, o pincel, a textura, o
    /// contorno, os filtros, o envelope, os efeitos, o morph e os eixos da fonte.
    fn fase_bus_value_live(&mut self, id: &NodeId, v: &f64, pd: &mut DrainOut) {
        if crate::vec_connector_panel::is_connector_field_id(*id) {
            // Os três campos do conector: a shell os aplica em TODOS os
            // conectores selecionados (a tool os ignora — não são Style).
            pd.pending_vec_connector = Some((*id, *v));
        } else if crate::vec_shape_params::is_shape_field_id(*id) {
            // Sliders de forma: a tool os toma como default de
            // desenho (abaixo, no forward) E eles editam a forma
            // VIVA selecionada — o track cru vai junto, porque a
            // conversão depende da variante da forma.
            pd.pending_vec_shape_param = Some((*id, *v));
        } else if *id == ph2d_panel_vector::ids::VECTOR_BLEND_STEPS {
            // ADR-0128: arrastar Steps ajusta o blend selecionado AO VIVO.
            pd.pending_blend_steps = Some(ph2d_tool_vector::params::blend_steps_from_track(*v));
        } else if *id == ph2d_panel_vector::ids::VECTOR_TEXTPATH_OFFSET {
            // Plano 22: FRAÇÃO do comprimento do caminho, ja' no dominio do
            // documento (o painel nao converte -- track e valor coincidem).
            pd.pending_textpath_offset = Some(*v);
        } else if let Some(c) = crate::vec_stroke_paint::slider_cmd_for_id(*id, *v) {
            // ⭐ Os knobs do PINCEL (plano 36, W4). O `event.rs` do painel já
            // converteu o track para o domínio do documento — aqui `*v` é valor.
            pd.pending_brush = Some(c);
        } else if let Some((slot, knob)) = ph2d_panel_vector::texture_pattern::texpat_knob_of(*id) {
            // ⭐⭐ **O SUJEITO VEM NO ID** (plano 35, wave F): cada secção tem os
            // seus sliders, então arrastar um deles já diz em QUAL das duas
            // tintas escrever. ⚠️ O `event.rs` do painel já converteu o track
            // para o domínio do documento — aqui `*v` é valor.
            use ph2d_panel_vector::ids::TexPatKnob as K;
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
                K::Width => Some(crate::texture_pattern_edit::TexPatCmd::Axis(0, *v, cadeado)),
                K::Height => Some(crate::texture_pattern_edit::TexPatCmd::Axis(1, *v, cadeado)),
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
                K::ShiftX => Some(crate::texture_pattern_edit::TexPatCmd::Shift(0, *v)),
                K::ShiftY => Some(crate::texture_pattern_edit::TexPatCmd::Shift(1, *v)),
                // GRAUS aqui; o documento guarda radianos, e a conversão vive
                // na porta única (`texture_pattern_edit::apply`).
                K::Angle => Some(crate::texture_pattern_edit::TexPatCmd::Angle(*v)),
                K::Offset => Some(crate::texture_pattern_edit::TexPatCmd::OffsetDenom(*v)),
                _ => None,
            };
            pd.pending_texpat = cmd.map(|c| (alvo, c));
        } else if *id == ph2d_panel_vector::ids::VECTOR_PATTERNPATH_SPACING {
            // Plano 23: ja' convertido pelo event.rs do painel para o dominio do
            // documento (multiplos da largura do motivo) -- aqui e' valor.
            pd.pending_pp_spacing = Some(*v);
        } else if *id == ph2d_panel_vector::ids::VECTOR_PATTERNPATH_START {
            // FRAÇÃO do comprimento (track == valor, como o Offset do texto).
            pd.pending_pp_start = Some(*v);
        } else if *id == ph2d_panel_vector::ids::VECTOR_PATTERNPATH_END {
            // FRAÇÃO do comprimento -- o fim do trecho `[Start, End]`.
            pd.pending_pp_end = Some(*v);
        } else if *id == ph2d_panel_vector::ids::VECTOR_PATTERNPATH_SLIDE {
            // O CENTRO do trecho -- o drain re-centra a janela (move Start+End).
            pd.pending_pp_slide = Some(*v);
        } else if *id == ph2d_panel_vector::ids::VECTOR_PATTERNPATH_ROTATION {
            // A ORIENTAÇÃO do motivo sobre a guia, em GRAUS -- o event.rs do painel
            // ja' converteu o track bipolar (`-180..180`); aqui e' valor.
            pd.pending_pp_rotation = Some(*v);
        } else if *id == ph2d_panel_vector::ids::VECTOR_PATTERNPATH_OFFSET {
            // Desvio perpendicular (unidades de mundo), ja' bipolar (`-2..2`)
            // convertido pelo event.rs do painel -- aqui e' valor.
            pd.pending_pp_offset = Some(*v);
        } else if *id == ph2d_panel_vector::ids::VECTOR_CONTOUR_STEPS {
            // Quantos aneis -- o `event.rs` do painel ja arredondou ao inteiro.
            pd.pending_contour_steps = Some(*v);
        } else if *id == ph2d_panel_vector::ids::VECTOR_CONTOUR_OFFSET {
            // A distancia POR PASSO, em FRACAO do tamanho da forma: o painel
            // fala fracao (um rotulo em unidades de mundo mentiria a cada troca
            // de selecao) e o componente guarda MUNDO. A conversao e' do `arm`
            // e do drain, com a MESMA `offset_scale` que o Offset usa.
            pd.pending_contour_d = Some(*v);
        } else if *id == ph2d_panel_vector::ids::VECTOR_CONTOUR_ACCEL {
            // A aceleracao da progressao -- o painel ja aplicou o mapa
            // GEOMETRICO do trilho; aqui e' valor.
            pd.pending_contour_accel = Some(*v);
        } else if let Some(hit) = crate::fx_live::hit_of(*id) {
            pd.pending_filter_val = Some((hit, *v));
        } else if *id == ph2d_panel_vector::ids::VECTOR_ENVELOPE_BEND {
            // ADR-0129 Fatia C: o `event.rs` do painel ja converteu o track
            // bipolar para o dominio do documento (`-1..1`) -- aqui e' valor.
            pd.pending_envelope_bend = Some(*v);
        } else if let Some((r, prm)) = crate::fx_bridge_dispatch::classify_param(*id) {
            // O painel entrega o TRACK normalizado; a faixa real e' do
            // efeito e a ponte a aplica.
            pd.pending_fx_param = Some((r, prm, *v));
        } else if *id == ph2d_panel_vector::ids::VECTOR_MORPH_T {
            // Arrastar o `t` move a forma pelo caminho AO VIVO — e é assim que
            // o artista a estaciona onde ela fica bem, antes do K.
            #[allow(clippy::cast_possible_truncation)]
            let t = *v as f32;
            pd.pending_morph_t = Some(t);
        } else {
            // Variation-axis field carries the axis VALUE directly
            // (not a 0..1 track): match the slot to its font axis.
            for i in 0..ph2d_panel_vector::ids::MAX_TEXT_VARIATION_AXES {
                if *id == ph2d_panel_vector::ids::vector_text_axis_id(i) {
                    pd.pending_vec_text_axis = Some((i, *v));
                    break;
                }
            }
        }
    }
}
