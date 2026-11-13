//! The paint-stroke **lifecycle** — begin / extend / tick / end + close: the undo snapshot, the
//! engine `Stroke` drive, the watercolor recomposite calls and the per-stroke state resets. Split
//! from `paint.rs` for the workspace LOC cap; a child of `paint`, so it keeps access to
//! `PaintState`'s module-private fields.

use super::*;

impl PainterTool {
    /// `true` when the active layer can be painted and the working buffer is sized — a **Raster** layer
    /// OR a **Mask** (its coverage buffer is bound to `canvas_rgba` like a raster's, so painting writes
    /// Rec.601-luma coverage: black conceals, white reveals). Group/adjustment/texture aren't paintable.
    pub(super) fn paint_target_ready(&self) -> bool {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 || self.canvas_rgba.is_empty() {
            return false;
        }
        self.layers
            .active()
            .and_then(|id| self.layers.get(id))
            .is_some_and(|l| matches!(l.kind, LayerKind::Raster(_) | LayerKind::Mask(_)))
    }

    /// Um traço aberto ou a tinta viva do Wet — a troca da activa não prova a figura igual (B18.3).
    pub(crate) fn traco_ou_tinta_viva(&self) -> bool {
        self.paint.stroke_undo.is_some() || self.paint.wetpaint.session.is_some()
    }

    /// Só um traço aberto (sem a tinta viva do Wet, que adormece e marca o próprio sujo: B18.4).
    pub(crate) fn traco_por_fechar(&self) -> bool {
        self.paint.stroke_undo.is_some()
    }

    /// A tinta viva do Wet está no ar (a sessão da activa).
    pub(crate) fn tinta_viva(&self) -> bool {
        self.paint.wetpaint.session.is_some()
    }

    /// Begin a stroke at `ev` and stamp the first dab. Snapshots the model for undo
    /// **before** painting so the whole stroke restores to the pre-stroke pixels.
    pub(super) fn paint_begin(&mut self, ev: CanvasPointer) {
        // Mask tool: ensure the TRANSIENT scratch mask exists for the current layer (tool-side, no layer
        // created) before the stroke paints into it.
        if matches!(self.paint.paint_mode, PaintMode::Mask) {
            self.ensure_mask_scratch();
        }
        // Inpaint heal brush: start a fresh defect mask for this stroke (the previous stroke's mark was
        // consumed + cleared on its pen-up).
        if matches!(self.paint.paint_mode, PaintMode::Inpaint) {
            let (w, h) = self.source_size;
            let n = (w as usize) * (h as usize);
            if self.paint.inpaint_mask.len() == n {
                self.paint.inpaint_mask.iter_mut().for_each(|m| *m = 0);
            } else {
                self.paint.inpaint_mask = vec![0u8; n];
            }
        }
        // Wet Paint: arm the live-gesture gate — the deposit route refuses dabs outside a real
        // pen-down..pen-up gesture (shape re-stamps against a fluid would pile paint; see `wetpaint`).
        if matches!(self.paint.paint_mode, PaintMode::WetPaint) {
            self.paint.wetpaint.live_gesture = true;
        }
        // **Style: Solid** — o gesto é um CAMINHO acumulado e a MANCHA é o polígono dele (plano 38
        // §1.1). ⚠️ **Quem o alimenta é a porta do CARIMBO** (`note_ink_path`), com os centros dos
        // dabs: desde a W8 a mancha é o polígono da TINTA, não o do ponteiro — metade dos tipos de
        // linha move a tinta para longe da mão, e era essa discordância que o Enio fotografou. Aqui
        // só se ZERA, para o gesto novo não herdar o caminho do anterior.
        self.paint.solid_path.clear();
        // …e a mancha devida do gesto anterior morre com ele: um `owed` sobrevivente faria o
        // primeiro `park_stroke` deste traço preencher um caminho que já não existe.
        self.paint.solid_fill_owed = false;
        self.paint.solid_passagem = super::uma_passagem::UmaPassagem::default();
        self.paint.traco_relogio = 0.0; // B22.1: o relógio do traço nasce no pen-down
        self.paint.movido_desde_o_passo = false;
        // **O passo de undo começa AQUI** (doc 28 §5.26): a cadeia é reconciliada e o journal passa a
        // descrever este traço em vez do intervalo desde o último commit.
        // Pinceladas sobrepostas (W16): o grupo inteiro é UM passo, aberto no primeiro pen-down.
        let sozinho = self.paint.tracos_fora == 0;
        if sozinho {
            // B16.3: o meio (aguada ou não) decide-se aqui e vale até ao pen-up do grupo.
            self.paint.aguada_do_traco = Some(self.paint.brush.watercolor);
            crate::sonda_partes::parte("down.antes");
            self.begin_undo_step();
            crate::sonda_partes::parte("down.begin_undo_step");
            if matches!(self.paint.paint_mode, PaintMode::WetPaint) {
                if self.paint.wetpaint.session.is_none() {
                    self.wet_acorda(); // o traço não espera o quadro: a tinta acorda sob o pincel
                }
                crate::sonda_partes::parte("down.wet_acorda");
                self.wetpaint_guard(); // o que outra ferramenta escreveu desde o último quadro é um passo à parte
                crate::sonda_partes::parte("down.wetpaint_guard");
                self.wet_guarda_o_passo(); // o undo deste passo devolve a tinta viva de agora
                crate::sonda_partes::parte("down.wet_guarda_o_passo");
            }
        }
        // ⚠️ **ELIDINDO o relevo** (degrau 4, doc 28 §5.72): este `before` DESCREVE os três planos em
        // vez de os segurar, e é isso que deixa o 1º dab escrever no lugar — a porta de fork copia
        // quando `strong_count > 1`, e um `Weak` não conta. O lado `before` do delta sai do journal.
        if sozinho {
            let before = self.snapshot_model_eliding_relief();
            self.paint.stroke_undo = Some(before);
            crate::sonda_partes::parte("down.snapshot");
        }
        // DEPOIS do `before`: desfazer o 1.º traço Wet de uma camada devolve-a à lei da outra tinta.
        if matches!(self.paint.paint_mode, PaintMode::WetPaint)
            && let Some(id) = self.layers.active()
        {
            // A marca muda a lei da camada onde ela tem tinta (B18.4: a camada nova, nada; não a tela inteira).
            if self.tintas_molhadas.insert(id) {
                self.a_marca_molhada_virou(id);
            }
            crate::sonda_partes::parte("down.marca_molhada");
        }
        self.paint.drag_preview = None;
        self.paint.plano_do_traco = None; // o plano do traço (L8/L5) nasce no 1.º lote deste traço
        self.mancha_na_aguada = None; // a do gesto anterior é tinta (ou foi descascada)
        self.paint.wetpaint.pending_deposit.clear(); // doc 21: a new gesture invalidates the stash
        self.paint.wetpaint.sulco = None; // cada traço da Borracha morta parte da imagem de agora
        self.paint.line_anchor = Some(ev.pos);
        // EDGE-1 wet session: while the paper is still WET and the canvas is untouched since OUR
        // last bake, consecutive watercolor strokes are ONE wash — keep the union buffers + cum
        // rect + session base, so the bake re-renders the union with a single rim (the previous
        // stroke's inner rim melts). Anything else (dried, foreign edit, mode/layer change) starts
        // a fresh session over the current canvas.
        let wet_session = !sozinho || self.wet_session_continues();
        // Reset the Accumulate-OFF cap mask (re-grown by the first dab) + the per-layer-colour
        // accumulation (so the recomposite snapshots THIS stroke's pre-pixels) — both per stroke.
        self.paint.stroke_mask.clear();
        self.paint.tecto_largo.clear();
        self.paint.po_do_papel = None; // o pó é o da tela deste pen-down
        self.paint.pixel_trilhos.zera(); // a ponta de píxel (L18) começa o trilho no traço novo
        // ⚠️ E o cap de CADA camada do composite com ele — eles são a mesma grandeza, só que por
        // camada (ver `composite::stamp_dabs_composite`). Esquecer um deixaria a 2.ª camada Brush
        // com o tecto do traço ANTERIOR, e ela deixaria de pintar a meio de um desenho.
        for m in &mut self.paint.composite_mask {
            m.clear();
        }
        // A subamostragem de uma camada maior conta arco DESTE traço.
        self.paint.composite_arco = [f32::NEG_INFINITY; crate::tool::paint::composite::N_CAMADAS];
        // ⭐ E a PILHA é do traço: o `pre` e a história dos lotes morrem com o gesto anterior, senão
        // a recomposição do primeiro lote deste parte de uma tela que já não existe.
        //
        // ⚠️ **A mutação que apaga ESTA chamada sobrevive, e é declarado:** o `close_stroke` fecha
        // a pilha em todo pen-up, logo esta é a metade DEFENSIVA — ela cobre o gesto que acaba sem
        // passar por lá (uma troca de modo, um cancelamento). *Duas guardas que se cobrem leem-se
        // como uma sobrevivência num relatório de mutação e não são*: a do pen-up tem gate próprio
        // (`um_traco_novo_nao_recompoe_da_tela_do_anterior`, a metade da memória).
        // O que o gesto anterior deixou por compor entra na tela ANTES de ela virar o `pre` deste.
        crate::sonda_partes::parte("down.limpezas");
        self.compoe_o_pendente();
        self.paint.pilha.fecha();
        crate::sonda_partes::parte("down.compoe_o_pendente");
        // ⚠️ Um fluxo de RNG por camada, semeado do fluxo do pincel. Sem a semente eles nasceriam
        // TODOS iguais e as camadas teriam a mesma realização de Grain/Randomize umas das outras.
        self.paint.rng_camada = std::array::from_fn(|i| {
            self.paint
                .tex_rng
                .wrapping_add((i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
        });
        if sozinho {
            self.reset_stroke_height(); // Impasto: this stroke's relief starts empty (see `super::impasto`)
        }
        // Sculpt: belt-and-braces. A committed gesture already ended its own session, so this normally
        // finds nothing — but any path that leaves one open (a shape abandoned without Cancel) would
        // otherwise have THIS stroke re-render from a `pre` frozen for a different one.
        if sozinho {
            self.end_sculpt_session();
        }
        // ⚠️ The protection epoch deliberately SURVIVES a pen-down (§13.13, Enio's call): a protection
        // that restarted per stroke let `1 − (1−keep)^N` through, so eight passes erased it (0.522 → 1.000
        // measured). It ends when the thing it describes changes, not when a gesture does — asked once, in
        // `begin_or_keep_gate_epoch`, instead of enumerated here and at 21 other sites.
        if !wet_session {
            self.paint.stroke_coverage.clear();
            self.paint.stroke_color.clear();
            self.paint.stroke_density.clear();
            self.paint.stroke_deplete.clear();
            self.paint.stroke_deplete_prox.clear();
            self.paint.wet_reserve_cache = None;
            self.paint.wet_rewet_cache = None; // B16.5: idem
            self.paint.wet_styles.clear();
            self.paint.stroke_water = Vec::new();
            self.paint.wet_cum_dirty = None;
        }
        if sozinho {
            self.paint.wet_frame_dirty = None;
            self.paint.wet_frame_linhas.clear();
            self.paint.wet_mistura.novo_traco(); // o `proprio` da mistura molhada é deste traço só
        }
        self.paint.wet_mistura.ha_tinta_da_sessao = wet_session; // o Smudge sobre tinta molhada
        // THIS-stroke footprint restarts every stroke (even continuing a wet session): only what THIS
        // stroke paints re-wets the moisture map, so earlier washes keep their own drying clocks (#4).
        self.paint.wet_stroke_dirty = None;
        self.paint.wet_pour_marca = None; // B16.11: o 1.º pour do traço caminha-o inteiro
        self.paint.wet_level_smear_pos = None;
        self.paint.wet_smear_pos = None; // the Wet Mix true-smear chain restarts with the stroke
        self.reset_wet_mix(); // the mixer reservoir starts fresh (no pickup) each stroke
        self.mistura_opaca_recomeca(); // a mistura opaca do Digital: o pincel começa limpo
        // Watercolor render-path: freeze the pre-stroke canvas as the optical base (shared `Arc`, so O(1);
        // the first composite `make_mut` forks the live buffer, leaving this pristine) PLUS the real
        // ground (the composite of the layers below + document paper colour) the optics read the
        // Beer–Lambert base / rewet reference from. The wash is reconstructed over these every frame
        // instead of over-painting in place. In a CONTINUING session this refrozen base (which now
        // includes the union baked so far) feeds only the mixer pickup + rewet; the composite keeps
        // reading the SESSION base below.
        crate::sonda_partes::parte("down.zera_sessao");
        self.freeze_watercolor_ground(wet_session);
        crate::sonda_partes::parte("down.freeze_ground");
        if !wet_session {
            // Fresh session: composite base = this pen-down's frozen canvas; the guard Arc is set
            // at the bake. A stale wet map from a broken session keeps drying on its own.
            self.paint.wet_session_base = self.paint.watercolor_base.clone();
            self.paint.wet_session_canvas = None;
        }
        if self.watercolor_render_active() {
            // EDGE-1 per-stroke style (doc 13 topo): capture THIS stroke's wash params at pen-down
            // — the union re-bake resolves them per pixel by the owner map, so an already-painted
            // wash keeps ITS Concentration/Edge/water instead of being re-styled by the current
            // brush (Enio 2026-07-09).
            let forced_wet = self.paint.wet_session_wetness;
            let marcas = self.marcas_da_aguada();
            self.paint.wet_styles.push_capture(
                &self.pincel_com_papel_na_escala(),
                forced_wet,
                marcas,
            );
            // Cada poça seca no seu tempo: a que secou longe da tinta molhada assa ANTES do primeiro
            // carimbo, para este traço velar sobre ela (o alcance já conta o estilo deste traço).
            if wet_session {
                self.assa_as_pocas_secas();
            }
        }
        crate::sonda_partes::parte("down.estilos");
        self.paint.per_layer_stroke.reset();
        // Smear chains its source from the previous dab; a fresh stroke has none yet.
        self.paint.last_smear_pos = None;
        // Clone: establish the source→dest offset for this stroke (aligned keeps it across strokes,
        // non-aligned re-anchors to the sampled source each stroke). No-op unless a source is sampled.
        self.clone_begin_offset(ev.pos);
        // Pin the symmetry centre to the current canvas centre for the auto-centre modes before the
        // stroke captures the spec (the engine mirrors/rotates about `brush.symmetry.center`).
        self.resolve_symmetry_geometry();
        // Clone ignores Symmetry (its panel section is hidden): mirrored dabs would clone from mirrored
        // source positions, which is nonsensical — strip it from the captured spec so a leftover-enabled
        // flag can't silently mirror. Other modes keep Symmetry.
        let mut spec = self.stroke_spec();
        if matches!(self.paint.paint_mode, PaintMode::Clone) {
            spec.symmetry.enabled = false;
        }
        // ⭐ E o AUTORADO fica guardado: é dele que cada ponto seguinte recompõe a forma do dab
        // contra a dobra VIVA da arte (ver `Stroke::set_canvas_dab`). ⛔ Sem isto o traço inteiro
        // usa a dobra do sítio onde começou.
        self.paint.stroke_authored = Some(self.authored_spec());
        let mut stroke = Stroke::new(spec, self.dinamica_do_traco(), self.paint.seed);
        // Seed the texture RNG from this stroke's seed, decorrelated from the jitter stream so the
        // two don't lock-step (HR-5: deterministic per stroke).
        self.paint.tex_rng = self.paint.seed ^ 0x7465_7874_7572_6573;
        self.paint.seed = self.paint.seed.wrapping_add(1);
        self.mistura_opaca_pressao(ev.pressure);
        let mut dabs = std::mem::take(&mut self.paint.dabs);
        stroke.set_tilt(ev.tilt);
        stroke.begin(
            StrokePoint {
                pos: ev.pos,
                pressure: ev.pressure,
            },
            &mut dabs,
        );
        self.dual_comeca(
            StrokePoint {
                pos: ev.pos,
                pressure: ev.pressure,
            },
            ev.tilt,
        ); // a 2.ª ponta (L6)
        crate::sonda_partes::parte("down.stroke_begin");
        self.stamp_stroke_dabs(&dabs);
        crate::sonda_partes::parte("down.stamp");
        self.paint.dabs = dabs;
        // ⚠️ O traço volta ANTES do composite: o `park_stroke` fecha o evento (fios e a mancha do
        // Solid na aguada), e o composite é o LEITOR dele — invertidos, o quadro sai sem a mancha
        // (o «pisca» do smoke de 2026-10-04, `a_mancha_nao_some_no_quadro_em_que_o_tique_carimba`).
        self.park_stroke(stroke);
        crate::sonda_partes::parte("down.park");
        // Watercolor render-path: reconstruct the wash optically over the frozen base (live; grows with
        // the stroke). Replaces the per-dab deposit that `stamp_dabs` skipped in watercolor mode.
        if self.watercolor_render_active() {
            self.apply_watercolor(false);
            crate::sonda_partes::parte("down.apply_watercolor");
            self.pour_canvas_wet(); // #2: moisture is laid LIVE (the damp shows during the stroke, not at pen-up)
            crate::sonda_partes::parte("down.pour_canvas_wet");
        }
    }

    /// The cursor moved over the canvas with **no button down** (image px). Advances the hover heading so
    /// the brush-cursor ring already wears the orientation the next dab will use — a calligraphic nib is
    /// aimed on the way in, not after you commit (Enio 2026-07-19).
    ///
    /// ⚠️ Deliberately **not** a `PointerPhase::Move`: a Move with no stroke open is a stray event that the
    /// paint paths, the shape editors and the deform gizmo all have their own opinions about. This touches
    /// exactly one field and paints nothing.
    ///
    /// It runs the SAME filter at the SAME length scale as the engine's own heading
    /// ([`ph2d_painter_brush::heading::advance`] / `smooth_len`), so the ring does not jump when the stroke
    /// takes over. Only maintained when a slot follows the stroke — a plain brush pays a bool test.
    pub fn on_canvas_hover(&mut self, pos: [f32; 2]) {
        crate::macro_track::porta!(
            self,
            |t| {
                let (w, h) = t.macro_referencia();
                crate::macro_track::Passo::Hover(crate::macro_track::passo::rel(pos, w, h))
            },
            { self.on_canvas_hover_corpo(pos) }
        )
    }

    fn on_canvas_hover_corpo(&mut self, pos: [f32; 2]) {
        // A CANETA da seleção também mira antes do clique: a linha-fantasma até o cursor é o que mostra
        // onde a próxima âncora cai. Fora do `follows_stroke`, que é a pergunta do anel do PINCEL — duas
        // perguntas distintas sobre o mesmo evento.
        self.selection_pen_hover(pos);
        if !self.paint.brush.follows_stroke() {
            // Nothing reads it; keep the state cold so a plain brush is untouched.
            self.paint.hover_pos = None;
            self.paint.hover_heading = [0.0, 0.0];
            return;
        }
        let prev = self.paint.hover_pos.replace(pos);
        let Some(prev) = prev else { return };
        let step = [pos[0] - prev[0], pos[1] - prev[1]];
        let len = (step[0] * step[0] + step[1] * step[1]).sqrt();
        if len <= f32::EPSILON {
            return; // no motion: hold the established heading (the engine holds too)
        }
        let smooth =
            ph2d_painter_brush::heading::smooth_len(2.0 * self.paint.brush.clamped_radius());
        self.paint.hover_heading = ph2d_painter_brush::heading::advance(
            self.paint.hover_heading,
            [step[0] / len, step[1] / len],
            len,
            smooth,
        );
    }

    /// Extend the in-progress stroke to `ev`, stamping any dabs the spacing emits.
    /// Returns `false` if no stroke is active (a stray Move).
    pub(super) fn paint_extend(&mut self, ev: CanvasPointer) -> bool {
        let Some(mut stroke) = self.paint.stroke.take() else {
            return false;
        };
        // Line + Alt: snap the cursor to a 45° increment around the press point (Blender
        // `constrain_line`). Tool-side so the engine's Line fill stays a plain anchor→cursor segment.
        let pos = match (self.paint.brush.stroke_method, self.paint.line_anchor) {
            (StrokeMethod::Line, Some(anchor)) if self.paint.line_constrain => {
                brush_settings::snap_to_45(anchor, ev.pos)
            }
            _ => ev.pos,
        };
        // ⭐⭐⭐ **A DOBRA DA ARTE É RELIDA AQUI, e só ela.** A shell reenvia a deformação a cada
        // evento; sem esta linha o traço inteiro pintava com a dobra do primeiro ponto.
        if let Some(autorado) = self.paint.stroke_authored {
            let vivo = self.compose_canvas_warp(autorado);
            stroke.set_canvas_dab(
                vivo.radius_px,
                vivo.dab_flatten,
                vivo.dab_angle_deg,
                vivo.dab_curve,
            );
        }
        self.mistura_opaca_pressao(ev.pressure);
        let mut dabs = std::mem::take(&mut self.paint.dabs);
        stroke.set_tilt(ev.tilt);
        stroke.extend(
            StrokePoint {
                pos,
                pressure: ev.pressure,
            },
            &mut dabs,
        );
        self.dual_estende(
            StrokePoint {
                pos,
                pressure: ev.pressure,
            },
            ev.tilt,
        ); // a 2.ª ponta (L6), o mesmo caminho
        // **Style: Solid** — o caminho E a mancha são escritos pela porta do carimbo (`stamp_dabs` a
        // bracketa, `super::solid_deposit`), e não por uma chamada solta neste sítio. ⚠️ Desde a W7 o
        // dab **é** carimbado (a tinta do gesto é a região cercada **MAIS** o traço que a cerca, como
        // um `FlipStroke` com `fill`) e desde a W8 é ele quem DIZ onde a região passa.
        super::ribbon_diag::note(super::ribbon_diag::Source::Extend, dabs.len());
        self.stamp_stroke_dabs(&dabs);
        // Watercolor render-path: the dabs are ACCUMULATED here (the path is the data, so this is per
        // event and stays per event) and the optical reconstruction is **left to the frame**, not run
        // here. `paint_tick` pays it once — it already knows a Move arrived, from `moved_this_frame`,
        // which it reads as `parked` — and that is the cadence `apply_watercolor`'s own doc names
        // ("each frame recomposites", "renderFrame", "the frame dirty rect").
        //
        // ⚠️ **This used to recomposite per pointer event, and that was the whole defect.** The wash
        // window is padded by the influence radius — footprint-sized — so every event paid a near-full
        // footprint pass whether or not the spacing emitted a single dab: the same 1200 px path cost
        // 87,5 ms delivered in 30 events and **172,5 ms in 240** (1,97×), while Digital measured
        // 34,7 → 36,0 (1,00×) on the identical test. A high-Hz tablet was buying the artist nothing but
        // work. Skipping the intermediates is byte-identical because the reconstruction is a pure
        // function of the frozen base plus the accumulators — measured at 0 differing bytes by
        // `measure_whether_the_wash_depends_on_the_polling_rate`, not assumed.
        //
        // ⚠️ **The doubling was already SEEN and cured on the wrong side.** The soak comment in
        // `paint_tick` records a 2026-07-07 profile where `stamps` and `tool-tick` each carried a full
        // composite, and the fix suppressed the tick's — the ONE per frame — on the premise that "the
        // pointer-Move flush already recomposited this frame's window". That premise holds for the
        // COALESCED methods (one delivery per frame) and is false for the incremental freehand path,
        // which is the default watercolor brush: there the flush is one per raw event.
        self.paint.dabs = dabs;
        self.park_stroke(stroke); // antes do composite — ver o `paint_begin`
        if self.watercolor_render_active() && self.wash.per_event {
            self.apply_watercolor(false); // the frozen legacy route — measurement + mutation only
            self.pour_canvas_wet();
        }
        self.paint.moved_this_frame = true;
        true
    }

    /// Per-frame heartbeat while a stroke is held (`dt_s` = wall time since the last frame); clears the
    /// move flag and drives two time-based behaviours (both no-op when their method doesn't apply):
    /// - **Airbrush timer:** deposit dabs at the brush's Rate, moving OR parked (Blender fires it on a
    ///   timer, not on motion — builds up when held, sparse when swept). No-op for non-Airbrush.
    /// - **Stabilizer catch-up:** when parked, walk the lagged path to the cursor (`settle` is
    ///   Space-only) so a high-stabilizer stroke arrives without waiting for pointer-up.
    pub(crate) fn paint_tick(&mut self, dt_s: f32) {
        // Decay the transient in-gizmo Stencil preview (armed by panel param changes); runs every frame
        // even with no open stroke, so it fades out shortly after the user stops changing the params.
        if self.paint.stencil_preview_s > 0.0 {
            self.paint.stencil_preview_s = (self.paint.stencil_preview_s - dt_s).max(0.0);
        }
        // EDGE-1: the paper dries every frame too (stroke open or not — same class as the decay
        // above): the persistent moisture poured at each bake fades over the drying window.
        self.dry_canvas_wet(dt_s);
        // Wet Paint: the 40 Hz fluid sim heartbeat (paused while a stroke is down — the engine's own
        // gate); a plain no-op with no live session, like the two decays above.
        self.wetpaint_tick(dt_s);
        // Keep the auto-centre symmetry pivot on the canvas centre every frame (also no-op when idle),
        // so the dashed overlay guide stays correct after a resize / fresh-sprite bind without paint.
        self.resolve_symmetry_geometry();
        // Live-editable wash (Enio 2026-07-11): while the paper is still wet and no stroke is open, a moved
        // Grain/Paper texture param re-renders the WHOLE committed wash (central + every Tiling copy) so the
        // wet wash reflects the new texture live — not only the next stroke. Inert unless a param changed.
        if self.paint.stroke.is_none()
            && self.paint.tracos_fora == 0
            && self.watercolor_render_active()
            && self.paint.wet_editable_base.is_some()
            && self.paint.wet_editable_tex != Some(self.wet_editable_sig())
        {
            self.rerender_editable_wash();
        }
        self.paint_tick_do_traco(dt_s);
    }

    /// B23.2: o que o [`Self::paint_tick`] ainda faz SEM evento, ramo a ramo — o laço não dorme
    /// enquanto um for verdade: `[traço aberto, papel a secar, sessão Wet Paint, prévia do Stencil]`.
    /// O papel seca por `carry.min(255)` num passo: dormir mudaria o byte.
    #[must_use]
    pub fn quer_quadro(&self) -> [bool; 4] {
        [
            self.traco_aberto() || self.paint.tracos_fora > 0,
            self.paint.canvas_wet_rect.is_some(),
            self.paint.wetpaint.session.is_some(),
            self.paint.stencil_preview_s > 0.0,
        ]
    }

    /// O quadro do traço aberto no tool — o resto do `paint_tick`, sem o que é do quadro inteiro (W16: o
    /// tocador corre-o também em cada traço guardado, [`super::tracos`]).
    pub(super) fn paint_tick_do_traco(&mut self, dt_s: f32) {
        let parked = !self.paint.moved_this_frame;
        self.paint.moved_this_frame = false;
        let Some(mut stroke) = self.paint.stroke.take() else {
            return;
        };
        let mut dabs = std::mem::take(&mut self.paint.dabs);
        // Watercolor render-path: accumulate the airbrush/settle dabs, then recomposite over the frozen
        // base once — but only when a dab actually landed (a held stroke ticks every frame).
        let wet = self.watercolor_render_active();
        // B22.1: aerógrafo, alcance do estabilizador e encharque andam em passos fixos (`ritmo.rs`).
        self.paint.movido_desde_o_passo |= !parked;
        let stamped = self.passos_do_traco(&mut stroke, &mut dabs, dt_s, wet);
        // THE frame composite. `stamped` is this tick's own airbrush/settle/dwell work; `owed` is the
        // frame's pointer Moves, which now accumulate without recompositing. Either way it runs at most
        // ONCE per frame, and the tick sits after the pointer flush and before the preview upload, so
        // what the artist sees this frame is fully reconstructed — the deferral costs no latency.
        self.paint.dabs = dabs;
        self.park_stroke(stroke); // antes do composite — ver o `paint_begin`
        let compoe = wet && (stamped || (!parked && !self.wash.per_event));
        if compoe || (wet && self.wash.recompoe_tudo) {
            self.apply_watercolor(false);
        }
        if compoe {
            self.pour_canvas_wet(); // #2: live moisture on the held/settling heartbeat too
        }
    }

    /// Finish the stroke at `ev` (stamp the final segment, flush the freehand smoother's tail so
    /// the stroke reaches the release point, then close + record undo).
    pub(super) fn paint_end(&mut self, ev: CanvasPointer) {
        self.paint_extend(ev);
        if let Some(mut stroke) = self.paint.stroke.take() {
            let mut dabs = std::mem::take(&mut self.paint.dabs);
            stroke.finish(&mut dabs);
            self.dual_passo(|m, o| m.finish(o));
            super::ribbon_diag::note(super::ribbon_diag::Source::Finish, dabs.len());
            super::ribbon_diag::report(&self.paint.brush);
            self.stamp_dabs(&dabs);
            self.paint.dabs = dabs;
            self.park_stroke(stroke);
        }
        // A ponta de píxel (L18): o último píxel do pixel-perfect esperava pelo seguinte — o traço acabou.
        self.pixel_fim_do_traco();
        // ⛔ A cauda antiga restaurava a TELA e re-carimbava o traço aqui (saiu a 2026-08-10). A de hoje (L5) só
        // existe no Digital e re-pinta no PLANO do traço (`fecha_o_plano_do_traco`, abaixo): sem restaurar a
        // tela nem re-carimbar o traço inteiro. Ver `ph2d_painter_brush::taper`.
        // **Style: Solid** — a corda final ganha o corpo que os rascunhos dela não depositaram.
        self.assenta_o_corpo_da_corda();
        // …e na água a região enche AGORA, antes do commit e do fecho do traço (um passo de Undo).
        self.enche_a_mancha_na_agua();
        // Drag Dot: the dab at the release point is the commit — keep it (drop the restore record).
        self.commit_drag_preview();
        // O PLANO do traço (L8/L5): a cauda re-pintada ao levantar, ANTES do fecho — um passo de Undo; antes dela, as
        // marcas da 2.ª ponta (L6) que nenhum lote levou.
        self.descarrega_o_dual();
        self.fecha_o_plano_do_traco();
        // Wet Paint: close the engine's direct stroke (the sim resumes; the session — the water —
        // stays alive across strokes). BEFORE close_stroke so the undo entry sees the final state.
        if matches!(self.paint.paint_mode, PaintMode::WetPaint) {
            self.wetpaint_stroke_end();
        }
        // Inpaint heal brush: reconstruct the marked defect BEFORE close_stroke, so the structural-undo
        // entry captures pre-stroke → healed as a single Cmd+Z step.
        if matches!(self.paint.paint_mode, PaintMode::Inpaint) {
            self.heal_inpaint();
        }
        // Watercolor render-path: bake the final optical composite over the frozen base (`commit` drops
        // the base). BEFORE close_stroke so pre-stroke → wash is one undo step (mirror of heal_inpaint).
        if self.watercolor_render_active() && self.paint.tracos_fora == 0 {
            // Keep the pre-wash BASE + frozen GROUND the bake composites over, so the wash stays
            // re-renderable while the paper is wet (live Grain/Paper edits, Enio 2026-07-11): the commit
            // drops the live base and `close_stroke` drops the ground, so capture them first.
            let editable_base = self
                .paint
                .wet_session_base
                .clone()
                .or_else(|| self.paint.watercolor_base.clone());
            let editable_backdrop = self.paint.wet_backdrop.clone();
            let region = self.apply_watercolor(true);
            // EDGE-1: pour the wash into the persistent moisture map AFTER the bake, then arm the
            // session guard — the exact canvas Arc our bake produced. A stroke landing while the
            // paper is still wet AND the guard still matches continues this wash (union re-bake).
            self.pour_canvas_wet();
            self.arma_a_guarda_da_sessao();
            // Arm the live-editable wash over the committed footprint (already full-axis on a tiled axis).
            if let (Some(base), Some(region)) = (editable_base, region) {
                self.paint.wet_editable_base = Some(base);
                self.paint.wet_editable_backdrop = editable_backdrop;
                self.paint.wet_editable_region = Some(region);
                self.paint.wet_editable_tex = Some(self.wet_editable_sig());
            }
        }
        self.close_stroke();
    }

    /// Finalize the current stroke: drop the in-progress state and push one undo
    /// entry (pre-stroke → current) so the whole stroke undoes/redoes as a unit. No-op when no stroke
    /// is open. Reuses the structural-undo stack (a full-canvas snapshot per stroke; tile delta later).
    pub(super) fn close_stroke(&mut self) {
        // ⭐ A composição adiada do último quadro entra na tela ANTES de tudo o que se segue ler a
        // tela (o undo, o relevo) e antes de o esfregão perder a sessão (`composite_por_quadro`).
        self.compoe_o_pendente();
        if self.paint.tracos_fora > 0 {
            // Pinceladas sobrepostas (W16): este traço fecha; o relevo, a aguada, o vidro e o passo do
            // desfazer são do grupo e fecham no último.
            self.paint.stroke = None;
            self.paint.pilha.fecha();
            self.paint.line_anchor = None;
            self.paint.last_smear_pos = None;
            return;
        }
        // Impasto: fold this stroke's relief into the layer BEFORE the undo entry is recorded, so the
        // step captures the height together with the pigment that made it — one Ctrl+Z takes both.
        self.commit_stroke_height();
        // Sculpt: the stroke is finished, so the session is finished — free it whole (Enio's smoke,
        // 2026-07-13). It used to be PARKED here, keeping Radius / Smooth↔Sharpen live on the stroke you
        // had just made; but a sculpt is an operation, not a substance, and picking Sharpen to sharpen
        // somewhere ELSE turned the Smooth behind you into its opposite (`super::sculpt::SculptState`).
        // The relief is already in `heights`, and `heights` is in the `ModelSnapshot` — so the undo entry
        // the line below records takes the carving with the paint that was under it. (No-op after
        // `commit_drag_preview`, which the freehand pen-up already ran: one death, not two.)
        self.end_sculpt_session();
        self.drop_erase_session(); // a mesma morte, um canal adiante (no-op depois do `commit_drag_preview`)
        // Smear: the knife's warp session is per STROKE (unlike Deform's, which spans them for
        // Reconstruct). The result stays on the canvas and becomes the next stroke's baseline.
        self.end_smear_session();
        // ⚠️ NOT the protection epoch: it outlives the stroke on purpose, so the next stroke through the
        // same protection converges on `keep` instead of eroding past it (§13.13).
        self.paint.stroke = None;
        self.paint.aguada_do_traco = None; // B16.3: o interruptor volta a valer
        // ⭐ A pilha morre com o traço — o `pre` é uma tela inteira, e segurá-la depois do pen-up
        // custaria `w·h·4` bytes por gesto sem ninguém a ler.
        self.paint.pilha.fecha();
        self.paint.line_anchor = None;
        self.paint.last_smear_pos = None;
        self.paint.watercolor_base = None; // defensive: the render-path drops it on commit already
        self.paint.plano_do_traco = None; // a tela já guarda a composição; o plano morre com o traço
        self.paint.wet_backdrop = None; // the ground is per-stroke (the stack below may change)
        self.paint.wet_soak_pos = None;
        // O VIDRO de uma aguada por baixo segue o que este traço cobriu (doc 48), antes de o desfazer
        // fotografar o depois; no papel de cor o quadro refaz a caixa.
        if let Some(r) = self.continua_o_vidro_do_traco(None)
            && self.papel_atravessa_vidro()
        {
            self.mark_dirty(r);
        }
        self.paint.wetpaint.sulco = None;
        if let Some(before) = self.paint.stroke_undo.take() {
            self.commit_structural_edit(before);
        }
    }
}

impl PainterTool {
    /// **A tela de antes do traço aberto** (o `before` elidido a descreve, B17.4). `fugaz` = quem pede larga-a antes da
    /// próxima escrita: aí o próprio vivo serve quando nada foi escrito. Senão ela é materializada uma vez e guardada no
    /// `before` (um dono a mais do VIVO forkaria a tela no dab seguinte).
    pub(crate) fn tela_de_antes_do_traco(
        &mut self,
        fugaz: bool,
    ) -> Option<std::sync::Arc<Vec<u8>>> {
        let s = self.paint.stroke_undo.as_ref()?;
        if s.canvas_elided.is_none() {
            return Some(std::sync::Arc::clone(&s.canvas_rgba));
        }
        // Numa troca de plano de pé, a tela está guardada fora do campo: é a da testemunha.
        let live = if self.undo.write_state.on_foreign_plane() {
            s.canvas_elided.as_ref()?.upgrade()?
        } else {
            std::sync::Arc::clone(&self.canvas_rgba)
        };
        let mut c = self.undo.canvas_held(s, &live)?;
        if std::sync::Arc::ptr_eq(&c, &live) {
            if fugaz {
                return Some(c);
            }
            c = std::sync::Arc::new(live.as_ref().clone());
        }
        let s = self.paint.stroke_undo.as_mut()?;
        s.canvas_rgba = std::sync::Arc::clone(&c);
        s.canvas_elided = None;
        Some(c)
    }
}
