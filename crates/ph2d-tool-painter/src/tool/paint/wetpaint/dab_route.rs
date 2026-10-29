//! **A ROTA DO DAB** — como um dab do Painter entra na água (filho de
//! [`super`], LOC cap).
//!
//! O corte é por responsabilidade: o pai guarda *o que o tool FAZ com uma
//! sessão* (o ciclo do traço, o tick, o guard de identidade), o `session.rs`
//! guarda *o que uma sessão É*, e aqui fica **a tradução de um dab do sistema
//! de pincel para o motor de fluido** — a única porta onde as duas linguagens
//! se encontram, e por isso a única que precisa saber que [`super::grid_map`]
//! existe.
//!
//! Três conversões vivem nesta porta, e as três TÊM de concordar: a POSIÇÃO e o
//! RAIO viram células (`px_to_cell` / `px_len_to_cell`), a SILHUETA é avaliada
//! de volta em pixels de canvas (`cell_subsample_px`, com o AA por cobertura) e
//! o PAPEL é semeado na lei de canvas. Se qualquer uma discordar, o carimbo sai
//! deslocado de meia célula e **nenhum número o denuncia** — só a tinta no lugar
//! errado.

use super::*;

impl PainterTool {
    /// The Wet Paint dab route ([`Self::stamp_dabs_inner`] arm). The dab list
    /// is already mirrored (Symmetry) and replicated (Tiling) — the engine
    /// sees exactly what the colour routes would have seen.
    pub(in crate::tool::paint) fn stamp_dabs_wetpaint(&mut self, dabs: &[Dab], brush: &BrushSpec) {
        // A ponta de N imagens (L19): também o lote ADIADO (pousado no tique) carimba cada corrida com a sua imagem.
        if self.mangueira_viva() {
            // O lote VIRTUAL (as pontas rectas) leva a lane de cada dab em `fecho`, paralela ao lote: cada corrida lê a
            // sua fatia (recomeçar o índice em 0 dava à 2.ª corrida as lanes da 1.ª).
            let fecho = self
                .paint
                .wetpaint
                .session
                .as_ref()
                .map(|s| s.fecho.clone())
                .unwrap_or_default();
            return self.stamp_dabs_mangueira_com(dabs, |t, i, corrida| {
                if !fecho.is_empty()
                    && let Some(s) = t.paint.wetpaint.session.as_mut()
                {
                    s.fecho = fecho[i..i + corrida.len()].to_vec();
                }
                t.stamp_dabs_wetpaint(corrida, brush);
                if !fecho.is_empty()
                    && let Some(s) = t.paint.wetpaint.session.as_mut()
                {
                    s.fecho.clone_from(&fecho);
                }
            });
        }
        let (w, h) = self.source_size;
        // Two doors in, one belt (doc 21): a LIVE incremental gesture (the
        // W1/W2 path — dabs are the artist's hand, once) or the commit
        // door's one-shot `deposit_pass` replay. Everything else is refused
        // — authoring batches are un-owned and can no longer even reach
        // here (routing sends them flat); this gate is the belt against a
        // routing regression, and it must stay a WALL, not a debug_assert.
        let live = self.paint.wetpaint.live_gesture && brush.stroke_method.is_incremental();
        if dabs.is_empty() || w == 0 || h == 0 || !(live || self.paint.wetpaint.deposit_pass) {
            return;
        }
        // A marca do artista (W9): a tinta destes dabs aparece pela água — marcam já. Na água do artista
        // (W12) a cobertura sai do composite.
        if !self.wet_lote_do_artista() {
            self.marca_os_dabs(dabs, brush.tip_reach(), true);
        }
        self.wetpaint_guard();
        // A água corre com o traço aberto (W1e, F6): o lote que chega com o motor na simulação NÃO
        // espera o estágio em voo — fica na fila e pousa no tick; com o motor em casa, a fila pousa
        // primeiro (a ordem dos dabs é a da mão).
        if live && let Some(sess) = self.paint.wetpaint.session.as_mut() {
            if sess.stroke_open && !sess.try_bring_home_now() {
                sess.adiados.push(session::LoteAdiado {
                    dabs: dabs.to_vec(),
                    brush: *brush,
                    groups: self.paint.dab_groups.clone(),
                    tex_rng: self.paint.tex_rng,
                });
                return;
            }
            if !sess.adiados.is_empty() {
                self.wetpaint_pousa_adiados();
            }
        }
        // The wet ERASER (W2.6) erases the FLUID and only the fluid: with no
        // live session there is nothing wet, and the routing predicate
        // (`wet_owns_the_dabs`) already sent those dabs to the normal
        // eraser — this early-out only covers the guard killing the session
        // between the route decision and here (the batch then does nothing).
        let erasing = self.paint.eraser;
        // ⭐ A pressão → OPACIDADE do pincel (prompt 45) pousa como opacidade: o `b` do motor fica o do
        // toque inteiro e a fracção da pressão multiplica o que pousa (`Dab::opacity`). Só no PINCEL que
        // pinta; o resto lê o `b` de sempre. Doc 45 §2.
        let opacidade_da_pressao = (self.dinamica_do_traco().strength_pressure && !erasing)
            .then(|| brush.method_overlap_factor().clamp(0.0, 1.0));
        let nasceu_agora = self.paint.wetpaint.session.is_none();
        if self.paint.wetpaint.session.is_none() {
            if erasing {
                return;
            }
            self.ensure_wet_session();
        }
        // O dual brush (L6): as marcas da 2.ª ponta na máscara do traço, que recorta o pigmento que pousa.
        let dual =
            self.recorte_do_dual_no_wet(!erasing && self.paint.wetpaint.tool == WetTool::Paint);
        // Take the session out so the prep below can borrow `self.paint`'s
        // Shape state alongside it (disjoint in fact, not in the borrow
        // checker's eyes); restored before the composite.
        let mut taken = self.paint.wetpaint.session.take().expect("ensured above");
        let sess = &mut taken;
        // A PORTA: o motor pode estar com o worker desde o fim do último tick.
        sess.bring_home();
        // A razão da grade é a da SESSÃO, nunca a autorada: é ela que diz de
        // que tamanho o motor VIVO é. As duas concordam sempre (trocar a
        // razão encerra a sessão), e ler a que existe é o que mantém a
        // conversão honesta mesmo se essa porta ganhar um caminho novo.
        let ratio = sess.ratio;
        // ── The artist's PAPER drives the engine's tooth (W2.7) — `lote.rs`. ──
        self.reconcilia_o_papel(sess, brush, ratio);
        // ── The authored facts — same reconcile law as the paper. ──────────
        sess.reconcile_facts(self.wet_facts());
        // ⚠️ **Uma sessão nascida NESTE lote chega aqui com a folha inteira suja, e ela não tem o que
        // mostrar.** O construtor do motor marca `Full` (o `rebake_paper`: um papel novo muda a
        // granulação de tinta que JÁ exista), e o `reconcile_facts` acima re-coze o papel outra vez
        // quando os knobs autorados diferem dos de fábrica. Numa grade sem tinta nem água o composite
        // dessa folha escreve em cada pixel o que ele já tem — premissa gateada (`birth_tests`) — e
        // custava `~20 ms` a 4096² no 1.º traço da sessão; pior, desde que o composite DECLARA onde
        // escreve (`crate::undo::window`), a janela declarada do 1.º traço era a tela inteira e o commit
        // dele guardava o plano `Whole` (dois planos de 67 MB) em vez do traço. ⚠️ É AQUI e não no
        // nascimento porque o re-cozido do `reconcile_facts` vem DEPOIS dele (e há gate para os dois).
        if nasceu_agora {
            let _ = sess.engine.take_dirty();
        }
        // The engine-side TOOL (doc 22): gives the lane doors their
        // `TrailMode` (begin picks Blend by it) and the sim its pause law
        // (`sim_should_run` keeps running under a Blow stroke, the model's
        // one exception). The ERASER always pauses — the engine erases with
        // its own tool untouched, exactly as before.
        let wet_tool = self.paint.wetpaint.tool;
        sess.engine.tool = if erasing {
            ph2d_wet_paint::painter::Tool::Paint
        } else {
            wet_tool.engine()
        };
        // ── The dab's SILHOUETTE — the painter's, not the engine's ─────────
        // `silhouette_at` is the single source of the dab's shape (falloff ×
        // Shape image/procedural × flatten/rotate footprint); the engine's
        // internal falloff/footprint step aside per dab via the shaped door,
        // and the bristle texture stays as the fluid's default grain (W2.3b).
        // ⭐ **A Shape Color** (prompt 41): as camadas dão a cor de cada CÉLULA do depósito, pela porta
        // da aguada (Texture Color pelo valor do Shape, Falloff Color pela silhueta); o padrão que vira cor
        // deixa de recortar (a lei do digital, P38). Assa também o tom do modo B&W.
        let rampa = if erasing {
            None
        } else {
            self.rampa_da_aguada()
        };
        let padrao_vira_cor = rampa.is_some()
            && self.padrao_do_shape_vira_cor(
                super::ramp_lut::RampLutOwner::Shape,
                self.paint.shape_image.is_some(),
            );
        // As Grain Colors (o grão dono da cor): o valor do grão na célula indexa a rampa e deixa de morder
        // a cobertura — a lei do digital (`ph2d_painter_brush::RampPaint::cor`).
        let grao_e_cor = rampa
            .as_ref()
            .is_some_and(super::watercolor_accum_cor::RampaDaAguada::do_grao);
        let shape_image = self.paint.shape_image.as_ref().map(|i| i.as_mask());
        let shape_ramp_lut = (self.paint.shape_color_ramp_enabled
            && self.paint.shape_color_ramp_bw)
            .then_some(self.paint.shape_ramp_lut.as_slice());
        // ⭐ **A ponta de CERDAS não é silhueta** (plano 27 W2): o Shape Bristles é a TEXTURA do motor (o
        // `texv`, que hoje vinha do tapete do próprio motor) — a silhueta fica o contorno da ponta. Um Shape
        // que não é Bristles não tem cerdas: a ponta é LISA (`texv = 1`, D4).
        let cerdas = brush.shape.kind == ph2d_painter_brush::TextureKind::Bristles;
        // ⭐ A marca MENOR do spray é um respingo liso, não um pedaço do tapete de cerdas (W3 K2): o tapete é ~94 % feltro
        // perto de zero e, cortado ao tamanho da marca, o portão do papel comia-a inteira — `spray_count` 10 com `particle`
        // 0,1 pousava 54 de tinta contra 4 719 do pincel sem spray (`docs/Painter/Pinceis/W3_K2_textura.md`).
        let marca_do_spray = brush
            .dab_jitter
            .particula_no(brush.spray_count, brush.clamped_radius())
            < 1.0;
        let filme_liso = sess
            .engine
            .tuning
            .get(ph2d_wet_paint::tuning::Knob::StrokeFilm)
            > 0.0;
        let fios = self
            .fios_do_pincel()
            .escalado(super::lote::escala_dos_fios(&sess.engine, brush));
        let relevo = self.relevo_do_pincel();
        // O rebordo da lâmina (W2 E4, `ph2d_wet_paint::lamina`): `0` = nem se chama a porta.
        let rebordo = sess
            .engine
            .tuning
            .get(ph2d_wet_paint::tuning::Knob::KnifeRidge);
        // … e a faca que NIVELA (W3 K3a, `knifeLevel`) também passa pela lâmina, com ou sem rebordo.
        let lamina = rebordo > 0.0
            || sess
                .engine
                .tuning
                .get(ph2d_wet_paint::tuning::Knob::KnifeLevel)
                > 0.0;
        // ⭐ O pigmento por COMPRIMENTO (prompt 51, doc 51 §2): cada dab pousa o do caminho desde o dab anterior da
        // lane ÷ o passo a que o pigmento foi afinado. Sem cápsula, nunca mais que um dab (o Spacing largo pontilha).
        let passo_do_pigmento =
            sess.engine
                .tuning
                .get(ph2d_wet_paint::tuning::Knob::PigmentSpacing) as f32;
        let capsulas = brush.lays_capsules();
        let shape_active = brush.shape_silhouette_active(shape_image.is_some()) && !cerdas;
        // The artist's GRAIN replaces the engine's bristle texture (W2.4).
        // The bristle IS the fluid's default grain, so an armed Grain slot
        // takes its place outright — two textures multiplying would
        // double-darken, and everywhere else in the app the Grain is THE
        // texture. The per-pixel law is `dab::grain_at`, the same single
        // door the colour route and the impasto height kernel call.
        // A imagem do slot que faz de Grain: a do PAPEL quando ele o é (`papel_como_grain`, como o carimbo).
        let grain_image = if self.papel_como_grain_do(brush) {
            &self.paint.paper_image
        } else {
            &self.paint.texture_image
        }
        .as_ref()
        .map(|i| i.as_mask());
        // Só o Grain que AMOSTRA alguma coisa toma o lugar das cerdas (um Image sem imagem não).
        let grain_active = brush.grain_samples(grain_image.is_some());
        let groups = self.paint.dab_groups.clone();
        let mut dab_rng = super::tiling::DabRng::new(self.paint.tex_rng);
        let canvas_wh = [w as f32, h as f32];
        if !sess.stroke_open {
            sess.stroke_open = true;
            sess.lanes.clear();
            sess.po = super::po::PoDoTraco::default();
            // The eraser and the DIRECT wet tools (Wet/Dry/Blow/Smear) are
            // LANE-LESS (per-dab grid ops, no trail) — open one direct
            // stroke so the sim gets its stroke gate as usual (paused under
            // the gesture; the Blow exception rides `engine.tool`).
            if (erasing || !wet_tool.uses_lanes())
                && let Some(d0) = dabs.first()
            {
                sess.engine.begin_direct_stroke(
                    0,
                    grid_map::px_to_cell(f64::from(d0.center[0]), ratio),
                    grid_map::px_to_cell(f64::from(d0.center[1]), ratio),
                );
            }
        }
        let strength = brush.strength.clamp(1e-3, 1.0);
        let t_dep = std::time::Instant::now();
        // `k` conta os sub-dabs de trás para a frente (`0` = o dab); `passo` = o sub-passo, com cápsula (`lote`).
        let sub = super::lote::sub_passo_do_lote(&sess.engine.tuning, brush, ratio);
        let fino = super::lote::peso_do_passo_fino(brush, ratio);
        super::lote::acerta_o_lote(&mut sess.engine, brush, fino, ratio);
        let esmaece_fino = super::rota_da_lane::esmaece_fino(&sess.engine, brush);
        let escala_tex = super::lote::escala_da_textura(&sess.engine, brush);
        let grao_e_o_papel = self.grao_e_o_papel_do(brush);
        let pigmento_da_marca = if marca_do_spray {
            super::lote::pigmento_da_marca(brush)
        } else {
            1.0
        };
        let escala_cerdas = [
            escala_tex,
            super::lote::escala_atraves(&sess.engine, brush, escala_tex),
        ];
        let sub_dabs: Vec<(usize, u64, Option<f32>, Dab)> = dabs
            .iter()
            .enumerate()
            .flat_map(|(i, d)| {
                let s = ph2d_painter_brush::capsula::sub_dabs(d, sub);
                let n = s.len() as u64;
                let passo = (d.step != [0.0, 0.0]).then_some(sub);
                s.into_iter()
                    .enumerate()
                    .map(move |(j, s)| (i, n - 1 - j as u64, passo, s))
            })
            .collect();
        let copia = super::copias::copia_de_cada_dab(dabs, brush, &groups);
        // As PONTAS RECTAS (J6, [`super::tampa`]): só o pincel que pinta; desligadas, nada abaixo muda.
        let pontas = brush.pontas_rectas && !erasing && wet_tool == WetTool::Paint;
        let alem = self.alem_no_motor(brush);
        // As pontas que ESBATEM (J6t, [`super::esbate`]): só o pincel que pinta; a 0, nada abaixo muda.
        let esbate_e = if !erasing && wet_tool == WetTool::Paint {
            brush.pontas_esbate
        } else {
            0.0
        };
        let virtual_ = !sess.fecho.is_empty();
        let lote = match (virtual_, sess.abrindo) {
            (false, _) => super::tampa::Lote::Mao,
            (true, true) => super::tampa::Lote::Abertura,
            (true, false) => super::tampa::Lote::Fecho,
        };
        if pontas && !virtual_ {
            sess.pincel_do_fecho = Some(*brush);
        }
        // Cada marca do spray é uma GOTA: um traço de um dab que pousa e solta a lane ([`super::rota_da_lane`]).
        let gota = marca_do_spray && !erasing && !virtual_ && wet_tool.uses_lanes();
        let lote_do_dab = pousa_o_dab::LoteDoDab {
            brush,
            erasing,
            opacidade_da_pressao,
            dual,
            ratio,
            wet_tool,
            rampa,
            padrao_vira_cor,
            grao_e_cor,
            shape_image,
            shape_ramp_lut,
            cerdas,
            marca_do_spray,
            filme_liso,
            fios,
            relevo,
            rebordo,
            lamina,
            passo_do_pigmento,
            capsulas,
            shape_active,
            grain_image,
            grain_active,
            groups: &groups,
            canvas_wh,
            strength,
            fino,
            esmaece_fino,
            escala_tex,
            grao_e_o_papel,
            pigmento_da_marca,
            escala_cerdas,
            copia: &copia,
            pontas,
            alem,
            esbate_e,
            virtual_,
            lote,
            gota,
        };
        for (didx, sub_k, passo, d) in sub_dabs.iter().map(|(i, k, p, d)| (*i, *k, *p, d)) {
            pousa_o_dab::pousa_o_dab(&lote_do_dab, sess, &mut dab_rng, didx, sub_k, passo, d);
        }
        // O PÓ SOLTO (J4 `pastel_carvao`): os grãos do lote, À PARTE do rasto ([`super::po`]); `po = 0`, nada.
        // Os dabs virtuais das pontas rectas não deixam pó: são o traço que NÃO houve.
        if !erasing && !virtual_ {
            self.pousa_o_po_do_lote(sess, dabs, &copia, brush, opacidade_da_pressao, strength);
        }
        let deposit_ms = t_dep.elapsed().as_secs_f32() * 1e3;
        self.paint.wetpaint.session = Some(taken);
        let t_comp = std::time::Instant::now();
        self.wetpaint_composite();
        crate::wet_diag::note_stamp(dabs.len(), deposit_ms, t_comp.elapsed().as_secs_f32() * 1e3);
        // A ABERTURA das pontas rectas: as lanes que conheceram o 1.º rumo neste lote ([`super::tampa`]).
        if pontas && !virtual_ {
            self.abre_as_pontas_rectas(brush);
        }
    }
}
