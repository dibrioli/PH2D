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
        // ── The artist's PAPER drives the engine's tooth (W2.7) ────────────
        // RECONCILED per batch, not seeded once: paper is substrate under
        // live water, but the SLOT is authored state and the session spans
        // strokes — a key of everything the seed reads decides. Key moved =
        // re-seed; slot disarmed = the engine re-bakes its own preset
        // (`rebake_paper`); unchanged = one small struct compare. The law is
        // the painter's own canvas-fixed paper sampling (the NEUTRAL door
        // `sample_tiled_rot_wrapped`, the same law the watercolor substrate
        // reads — one paper, never a second system). Known v1 gap: the
        // bitmap Size seam-snap under sprite Tiling (`snap_slot_size`) is
        // not applied — procedurals wrap exactly; a bitmap paper may seam.
        let period = [
            if self.paint.tiling[0] { w as f32 } else { 0.0 },
            if self.paint.tiling[1] { h as f32 } else { 0.0 },
        ];
        // O Tooth (BUGS #32) mora no MOTOR, que o aplica nas duas portas que escrevem o papel: o assado
        // dele re-assa no `reconcile_facts` abaixo (papel sujo); um papel do artista re-semeia pela chave.
        let tooth = brush
            .paper_depth
            .clamp(0.0, ph2d_painter_brush::PAPER_TOOTH_MAX);
        sess.engine.set_paper_tooth(tooth);
        // ⚠️ **UM papel, o do PAINEL (W1e, F2; dono 10/10).** Com um Kind armado, o do artista; SEM Kind,
        // um papel SEM DESENHO (escolha do dono, 10/10 — `prancha_w1g_sem_papel.png`): as alturas do papel
        // de fábrica, ao acaso célula a célula. A água comporta-se como num papel (o limiar do depósito
        // por célula é o que a faz sangrar no molhado); o desenho do papel — as manchas — some. Nunca o
        // papel procedural do MOTOR: a chave existe nos dois casos, então o motor não re-assa o dele.
        let want = Some(PaperKey {
            tex: brush.paper,
            image_version: self.paint.paper_image_version,
            period,
            tooth,
        });
        if sess.paper_key != want && !brush.paper.is_active() {
            let q = super::super::substrate_relief::alturas_do_papel_de_fabrica();
            // O Tooth esculpe um papel; sem papel não age (as alturas entram inteiras, `dente(h, 1) = h`).
            sess.engine.set_paper_tooth(1.0);
            sess.engine.seed_paper_with(&|cx, cy| {
                f64::from(q[super::super::substrate_relief::celula_ao_acaso(cx, cy, q.len())])
            });
            sess.engine.set_paper_tooth(tooth);
            sess.paper_key = want;
        }
        if sess.paper_key != want {
            let paper_tex = brush.paper;
            let paper_img = self.paint.paper_image.as_ref().map(|i| i.as_mask());
            let rot = ph2d_painter_brush::texture::angle_basis(paper_tex.angle_deg);
            // ⚠️ O `seed_paper_with` fala em CÉLULAS (ele passa `cell − 1`),
            // e o papel é uma lei de CANVAS — sob razão > 1 amostrá-lo na
            // coordenada de célula encolheria o dente do papel pelo mesmo
            // fator, e o papel deixaria de casar com o do resto do app.
            sess.engine.seed_paper_with(&|cx, cy| {
                let px = grid_map::cell_center_texel(cx as i32 + 1, ratio);
                let py = grid_map::cell_center_texel(cy as i32 + 1, ratio);
                f64::from(ph2d_painter_brush::texture::sample_tiled_rot_wrapped(
                    &paper_tex,
                    px,
                    py,
                    paper_img.as_ref(),
                    rot,
                    period,
                ))
            });
            sess.paper_key = want;
        }
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
        let shape_image = self.paint.shape_image.as_ref().map(|i| i.as_mask());
        let shape_ramp_lut = (self.paint.shape_color_ramp_enabled
            && self.paint.shape_color_ramp_bw)
            .then_some(self.paint.shape_ramp_lut.as_slice());
        // ⭐ **A ponta de CERDAS não é silhueta** (plano 27 W2): o Shape Bristles é a TEXTURA do motor (o
        // `texv`, que hoje vinha do tapete do próprio motor) — a silhueta fica o contorno da ponta. Um Shape
        // que não é Bristles não tem cerdas: a ponta é LISA (`texv = 1`, D4).
        let cerdas = brush.shape.kind == ph2d_painter_brush::TextureKind::Bristles;
        let fios = self.fios_do_pincel();
        let relevo = self.relevo_do_pincel();
        // ⭐ O pigmento por COMPRIMENTO (prompt 51, doc 51 §2): cada dab pousa o do caminho que representa — o
        // caminho desde o dab anterior da lane ÷ o passo do Spacing a que o pigmento foi afinado. Sem cápsula,
        // nunca mais que um dab inteiro: o Spacing largo de propósito pontilha.
        let passo_do_pigmento =
            sess.engine
                .tuning
                .get(ph2d_wet_paint::tuning::Knob::PigmentSpacing) as f32;
        let capsulas = brush.lays_capsules();
        let sub_da_capsula = sess
            .engine
            .tuning
            .get(ph2d_wet_paint::tuning::Knob::CapsuleSpacing) as f32;
        let tecto_da_capsula = sess
            .engine
            .tuning
            .get(ph2d_wet_paint::tuning::Knob::CapsuleMaxPx) as f32;
        let mais_perto = sess
            .engine
            .tuning
            .get(ph2d_wet_paint::tuning::Knob::CapsuleRoundNearest)
            > 0.5;
        let shape_active = brush.shape_silhouette_active(shape_image.is_some()) && !cerdas;
        // The artist's GRAIN replaces the engine's bristle texture (W2.4).
        // The bristle IS the fluid's default grain, so an armed Grain slot
        // takes its place outright — two textures multiplying would
        // double-darken, and everywhere else in the app the Grain is THE
        // texture. The per-pixel law is `dab::grain_at`, the same single
        // door the colour route and the impasto height kernel call.
        let grain_image = self.paint.texture_image.as_ref().map(|i| i.as_mask());
        // Só o Grain que AMOSTRA alguma coisa toma o lugar das cerdas (um Image sem imagem não).
        let grain_active = brush.grain_samples(grain_image.is_some());
        let groups = self.paint.dab_groups.clone();
        let mut dab_rng = super::tiling::DabRng::new(self.paint.tex_rng);
        let canvas_wh = [w as f32, h as f32];
        // The engine speaks sRGB **0..255** (its boot colour is `[50, 140, 210]` and the render
        // writes plane values straight through `clamp_u8`); the dab stores sRGB 0..1. Passing the
        // normalized value painted BLACK — Enio's W1 smoke, "a cor está preta".
        let ink = |c: [f32; 3]| {
            [
                f64::from(c[0]) * 255.0,
                f64::from(c[1]) * 255.0,
                f64::from(c[2]) * 255.0,
            ]
        };
        if !sess.stroke_open {
            sess.stroke_open = true;
            sess.lanes.clear();
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
        // ⭐ The dab-CAPSULE (Continuous, [`ph2d_painter_brush::capsula`]) reaches the engine as its
        // sub-dabs, oldest first, at least one engine cell apart: the deposit gates each against the
        // paper (`trail/deposit.rs::cell`), so only the sub-dabs themselves lay the dense stroke —
        // a capsule composed host-side either dilutes under the gate or widens past it (measured).
        // `k` conta os sub-dabs de trás para a frente (o `0` é o próprio dab); `passo` é o sub-passo (o arco que cada
        // sub-dab representa), com cápsula. O sub-passo é o do raio do PINCEL — o do dab traz a pressão e a caneta
        // deitada (×3,3), e o traçador sempre espaçou pelo do pincel.
        let sub = {
            let n =
                ph2d_painter_brush::capsula::sub_passo_px_a(brush.clamped_radius(), sub_da_capsula);
            let n = if tecto_da_capsula > 0.0 {
                n.min(tecto_da_capsula)
            } else {
                n
            };
            let cel = f32::from(ratio).max(1.0);
            if mais_perto {
                ph2d_painter_brush::capsula::sub_passo_mais_perto(n, cel)
            } else {
                ph2d_painter_brush::capsula::sub_passo_sem_batimento(n, cel)
            }
        };
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
        for (didx, sub_k, passo, d) in sub_dabs.iter().map(|(i, k, p, d)| (*i, *k, *p, d)) {
            let [x, y] = d.center;
            // LANE matching, geometric: the dab belongs to the lane whose
            // last position is within its own radius (consecutive dabs of one
            // copy are a spacing apart; other copies are far). A dab with no
            // lane in reach BEGINS one — a symmetry copy at stroke start, or
            // a Tiling wrap born mid-stroke at the sprite edge. Near a radial
            // centre the copies converge and may swap lanes; there their
            // positions coincide, so a swap deposits the same paint.
            // The ERASER skips the lanes entirely — every copy just erases
            // where it lands. The direct wet tools keep the geometric lane
            // MATCHING (it is what carries each symmetry/tiling copy's own
            // previous dab centre — the smear/blow displacement source) but
            // never touch the engine's lane trails.
            let mut lane_born = false;
            let li = if erasing {
                0
            } else {
                let thr = d.radius_px.max(4.0);
                let mut best = thr * thr;
                let mut lane = None;
                for (i, l) in sess.lanes.iter().enumerate() {
                    let (ddx, ddy) = (x - l.pos[0], y - l.pos[1]);
                    let d2 = ddx * ddx + ddy * ddy;
                    if d2 <= best {
                        best = d2;
                        lane = Some(i);
                    }
                }
                match lane {
                    Some(i) => {
                        if wet_tool.uses_lanes() {
                            // A janela do rasto anda pela CORDA entre centros: pelo sub-passo antes de arredondar, o
                            // dab do resto pesava um sub-passo inteiro — a escada do Oil Bristle a 103 px (doc 51 §4).
                            let chord = grid_map::px_len_to_cell(f64::from(best.sqrt()), ratio);
                            sess.engine.direct_segment(i, chord);
                        }
                        i
                    }
                    None => {
                        let i = sess.lanes.len();
                        lane_born = true;
                        if wet_tool.uses_lanes() {
                            // The DAB's colour, not the brush's: Randomize
                            // is already resolved per dab by the stroke
                            // engine (W2.2).
                            sess.engine.color = ink(d.color);
                            sess.engine.begin_direct_stroke(
                                i,
                                grid_map::px_to_cell(f64::from(x), ratio),
                                grid_map::px_to_cell(f64::from(y), ratio),
                            );
                        }
                        sess.lanes.push(Lane {
                            pos: d.center,
                            ink: d.color,
                            arco: d.arc_len,
                        });
                        i
                    }
                }
            };
            // The lane's PREVIOUS centre (before this dab updates it) — the
            // direct tools' displacement source; a born lane has none, and
            // the eraser has no lanes at all.
            let lane_prev: Option<[f32; 2]> = (!erasing && !lane_born).then(|| sess.lanes[li].pos);
            // Per-dab fresh ink (Randomize): reload the lane's trail — a
            // brush dipped in new paint (see `Trail::set_base_color`).
            if !erasing && wet_tool == WetTool::Paint && d.color != sess.lanes[li].ink {
                sess.engine.set_stroke_color(li, ink(d.color));
                sess.lanes[li].ink = d.color;
            }
            let (b, opacidade) = match opacidade_da_pressao {
                Some(ov) if wet_tool == WetTool::Paint => (
                    f64::from(ov) * 10.0,
                    (f64::from(d.coverage) / (f64::from(strength) * f64::from(ov)).max(1e-9))
                        .clamp(0.0, 1.0),
                ),
                _ => (((d.coverage / strength).clamp(0.0, 1.0) as f64) * 10.0, 1.0),
            };
            // Per-dab silhouette closure — the impasto walk's exact recipe
            // (spec at the dab's radius → rotor → footprint → Shape basis in
            // the stroke frame), evaluated per engine cell (cell − 1 = px).
            let spec = brush.at_dab(d);
            let peso = lane_prev
                .filter(|_| passo_do_pigmento > 0.0)
                .map_or(1.0, |_| {
                    let c = (d.arc_len - sess.lanes[li].arco).max(0.0);
                    let w = c / brush.passo_ao_longo_px(passo_do_pigmento, d.dir);
                    // O passo do PRÓPRIO Spacing de afinação é o dab inteiro, ao byte (a corda sai dos centros em f32).
                    if (w - 1.0).abs() < 1e-3 {
                        1.0
                    } else if capsulas {
                        w
                    } else {
                        w.min(1.0)
                    }
                });
            // O pente do relevo também é por comprimento: as células de caminho desde o dab anterior da lane (a
            // borracha, sem lanes, pelo sub-passo da cápsula).
            let passos_do_pente = lane_prev.map_or(1.0, |_| {
                let n = f64::from((d.arc_len - sess.lanes[li].arco).max(0.0)) / f64::from(ratio);
                if (n - 1.0).abs() < 1e-3 { 1.0 } else { n }
            });
            let fp = spec.dab_footprint_of(d);
            let dab_index = didx;
            let tex_rng = dab_rng.enter(&groups, dab_index);
            // Cada sub-dab da cápsula tira o SEU sorteio (a posição da textura, o pó) do do dab: repetido, os `n`
            // sub-dabs pousavam a mesma textura e o traço ganhava o período do Spacing (doc 51 §3, `os_fios`). O
            // `k = 0` é o próprio dab, o último: o fluxo segue para o próximo dab como sem a cápsula.
            if sub_k > 0 {
                *tex_rng = sorteio_do_sub_dab(*tex_rng, sub_k);
            }
            let shape_basis = (shape_active || cerdas).then(|| {
                ph2d_painter_brush::texture::shape_basis(
                    &spec.shape,
                    &mut *tex_rng,
                    canvas_wh,
                    fp,
                    ph2d_painter_brush::texture::ShapeFrame::Stroke {
                        arc_len: d.arc_len,
                        unit_px: d.stroke_radius_px,
                    },
                )
            });
            let shape_input = shape_basis.as_ref().filter(|_| shape_active).map(|sb| {
                ph2d_painter_brush::ShapeInput {
                    basis: sb,
                    image: shape_image.as_ref(),
                    ramp_lut: shape_ramp_lut,
                }
            });
            // O alcance da ponta de imagem (Bug #52): o motor percorre `r · reach` células, e a
            // janela do rasto cabe o máximo do traço (com rotação por dab, o disco de qualquer ângulo).
            // A ponta reta (prompt 44): o canto do quadrado rodado sai do disco — a caixa é a da pegada
            // DESTE dab (`fp`, com a rotação dele); a janela só precisa do canto em QUALQUER ângulo quando a
            // pegada roda por dab (o Follow, o Jitter Rotate) — fixa, a do dab basta (medido: 1,21× o custo).
            let mut reach = spec.dab_tip_reach(shape_basis.as_ref());
            if spec.dab_quina > 0.0 {
                let e = fp.extent();
                reach = reach.max(e[0].max(e[1]));
            }
            let quina_roda =
                spec.dab_quina > 0.0 && (spec.follows_stroke() || spec.has_per_dab_rotation());
            let janela = if spec.shape_has_per_dab_rotation(shape_image.is_some())
                || quina_roda
                || d.tilt.is_some()
            {
                spec.tip_reach().max(reach)
            } else {
                reach
            };
            sess.engine.silhouette_reach = [f64::from(reach), f64::from(janela)];
            // Grain basis AFTER the Shape basis, from the same RNG stream —
            // the colour route's resolve order (Shape before Grain), so the
            // random draws land where every other route puts them.
            let grain_basis = grain_active.then(|| {
                ph2d_painter_brush::texture::dab_basis(&spec.texture, &mut *tex_rng, canvas_wh, fp)
            });
            // O PÓ (prompt 43): a semente das partículas deste dab, um passo do MESMO fluxo (o splitmix64 do
            // pincel) — só com o pó ligado, para que nenhum outro pincel veja o fluxo mudar.
            if sess.engine.tuning.get(ph2d_wet_paint::tuning::Knob::Dust) > 0.0 {
                *tex_rng = tex_rng.wrapping_add(0x9E37_79B9_7F4A_7C15);
                let z = (*tex_rng ^ (*tex_rng >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
                sess.engine.sim.semente_do_po = (z ^ (z >> 31)) as u32;
            }
            let inv_r = 1.0 / d.radius_px.max(0.01);
            // ⚠️ A silhueta é avaliada em espaço de CANVAS, sempre — só
            // AMOSTRADA por célula. É por isso que a razão da grade não muda a
            // forma do pincel, apenas quão fino o fluido a resolve: o `t` sai
            // da distância em pixels ao centro do dab, e o Shape/Grain leem o
            // texel de canvas onde o sub-ponto caiu.
            //
            // ⚠️ **E ela é amostrada por COBERTURA, não no centro** — o AA do
            // depósito (`grid_map::cell_subsamples`): com a razão > 1 uma
            // decisão de um ponto por célula quantizava a borda do pincel em
            // degraus de `ratio` px, e nenhuma interpolação da SAÍDA recupera
            // isso. Em razão 1 são `n = 1` sub-pontos e o corpo abaixo é o de
            // sempre, ao bit.
            let (aa_n, aa_step) = grid_map::cell_subsamples(ratio);
            let aa_inv = 1.0 / f32::from(u16::from(aa_n) * u16::from(aa_n));
            // ⚠️ **Um carimbo REDONDO não roda** (W1e): rodar `(u, v)` e medir o comprimento muda o
            // último bit do raio, e com a água a correr no traço esse bit vira píxeis (o censo mediu o
            // Jitter Rotate e o ângulo a «mudar a tinta» de um carimbo sem ângulo). O Shape e o Grain
            // seguem a pegada rodada (`fp`): neles o ângulo é real.
            let deitada = d.tilt.is_some_and(|t| t.stretches());
            let fp_raio = if !spec.pegada_anisotropica() && spec.dab_curve.is_flat() && !deitada {
                ph2d_painter_brush::FootprintDeform::new(0.0, 0)
            } else {
                fp
            };
            // ⭐ **Lei B** (plano 27 W3): o padrão PROCEDURAL do Shape não entra na silhueta que o dente do
            // papel corta — vai ao motor no canal `padrao`, que multiplica DEPOIS do portão. A ponta de
            // imagem fica na silhueta (ela é o contorno), e a borracha apaga pela silhueta inteira.
            let padrao_a_parte = shape_input.is_some()
                && spec.shape.kind != ph2d_painter_brush::TextureKind::Image
                && !erasing;
            let sil_input = shape_input.filter(|_| !padrao_a_parte);
            let sil = |cx: i32, cy: i32| -> f64 {
                let mut acc = 0.0f32;
                for j in 0..aa_n {
                    let sy = grid_map::cell_subsample_px(cy, ratio, j, aa_step);
                    for i in 0..aa_n {
                        let sx = grid_map::cell_subsample_px(cx, ratio, i, aa_step);
                        let t = fp_raio
                            .falloff_t((sx - d.center[0]) * inv_r, (sy - d.center[1]) * inv_r);
                        acc += ph2d_painter_brush::dab::silhouette_at(
                            &spec,
                            sil_input,
                            t,
                            sx.floor() as i64,
                            sy.floor() as i64,
                            d.center,
                            d.radius_px,
                        );
                    }
                }
                f64::from(acc * aa_inv)
            };
            // Per-dab grain closure (armed slot only): replaces the bristle
            // sample inside the shaped stamp, cell − 1 = px like `sil`.
            let spec_ref = &spec;
            // O valor do Shape numa célula, pela cobertura (os sub-pontos do AA) — o padrão e a cor leem-no.
            let valor_da_forma =
                |si: ph2d_painter_brush::ShapeInput<'_>, cx: i32, cy: i32| -> f32 {
                    let mut acc = 0.0f32;
                    for j in 0..aa_n {
                        let sy = grid_map::cell_subsample_px(cy, ratio, j, aa_step);
                        for i in 0..aa_n {
                            let sx = grid_map::cell_subsample_px(cx, ratio, i, aa_step);
                            acc += ph2d_painter_brush::dab::shape_value_at(
                                spec_ref,
                                si,
                                sx.floor() as i64,
                                sy.floor() as i64,
                                d.center,
                                d.radius_px,
                            );
                        }
                    }
                    acc * aa_inv
                };
            let padrao = shape_input
                .filter(|_| padrao_a_parte && !padrao_vira_cor)
                .map(|si| move |cx: i32, cy: i32| -> f64 { f64::from(valor_da_forma(si, cx, cy)) });
            let padrao_arg = padrao
                .as_ref()
                .map(|p| p as ph2d_wet_paint::brush::CellFn<'_>);
            // A cor da célula: a Texture Color lê o valor do Shape (as cerdas incluídas — é a textura que a
            // prévia mostra), a Falloff Color a silhueta; sRGB `0..1` → os `0..255` do motor.
            let forma_da_cor = shape_basis
                .as_ref()
                .map(|sb| ph2d_painter_brush::ShapeInput {
                    basis: sb,
                    image: shape_image.as_ref(),
                    ramp_lut: None,
                });
            let (centro_x, centro_y) = (
                grid_map::px_to_cell(f64::from(x), ratio),
                grid_map::px_to_cell(f64::from(y), ratio),
            );
            // Os FIOS (W2): a pressão de cada fio e a oscilação, no comprimento de caminho do dab, em px.
            let fios_dab = fios.no_dab(
                f64::from(d.arc_len),
                [f64::from(d.dir[0]), f64::from(d.dir[1])],
                f64::from(d.radius_px / d.stroke_radius_px.max(1e-3)),
            );
            // O RELEVO de cada fio (W3): as faixas dos fios, mesmo sem pressão nem cor, e a altura de cada uma.
            let relevo_dab = (!relevo.neutro()).then(|| {
                fios_dab.unwrap_or_else(|| {
                    fios.resolvido(
                        f64::from(d.arc_len),
                        [f64::from(d.dir[0]), f64::from(d.dir[1])],
                        f64::from(d.radius_px / d.stroke_radius_px.max(1e-3)),
                    )
                })
            });
            let cor = rampa.as_ref().map(|r| {
                move |cx: i32, cy: i32, queda: f64| -> [f32; 4] {
                    let v = forma_da_cor.map(|si| valor_da_forma(si, cx, cy));
                    let c = r.cor_do_dab(v, queda as f32, d.color_shift);
                    [c[0] * 255.0, c[1] * 255.0, c[2] * 255.0, c[3]]
                }
            });
            // O que é de cada FIO, uma vez por dab (o traço rápido pagava os Perlin e o HSLuv por célula).
            let px_cel = f64::from(ratio);
            let tabela = fios_dab.map(|f| f.tabela(f64::from(d.radius_px) * 2.0 + 4.0 * px_cel));
            let tabela = tabela.as_ref();
            // A cor de cada FIO (W2, o ruído HSLuv) sobre a da Shape Color ou a do dab.
            let cor_fios = fios_dab.filter(|_| fios.tem_cor()).map(|f| {
                let (cor, base) = (cor.as_ref(), d.color.map(|c| c * 255.0));
                let t = tabela.expect("a tabela dos fios do dab");
                // Sem Shape Color a cor de base é a do dab: a de cada fio sai UMA vez (o HSLuv era por célula).
                let (k0, k1) = t.faixa();
                let do_fio: Vec<[f32; 3]> = if cor.is_none() {
                    (k0..=k1).map(|k| f.cor_do_fio_em(base, k, t)).collect()
                } else {
                    Vec::new()
                };
                move |cx: i32, cy: i32, queda: f64| -> [f32; 4] {
                    let c = cor.map_or([base[0], base[1], base[2], 1.0], |c| c(cx, cy, queda));
                    let dc = [f64::from(cx) - centro_x, f64::from(cy) - centro_y];
                    let fio = f.no_ponto_em(dc.map(|v| v * px_cel), t).fio;
                    let rgb = usize::try_from(fio - k0)
                        .ok()
                        .and_then(|i| do_fio.get(i).copied())
                        .unwrap_or_else(|| f.cor_do_fio_em([c[0], c[1], c[2]], fio, t));
                    [rgb[0], rgb[1], rgb[2], c[3]]
                }
            });
            let cor_arg = match (cor_fios.as_ref(), cor.as_ref()) {
                (Some(c), _) => Some(c as ph2d_wet_paint::brush::ColorFn<'_>),
                (None, Some(c)) => Some(c as ph2d_wet_paint::brush::ColorFn<'_>),
                (None, None) => None,
            };
            let gimg = grain_image.as_ref();
            let grain = grain_basis.as_ref().map(|gb| {
                move |cx: i32, cy: i32| -> f64 {
                    let px = grid_map::cell_center_texel(cx, ratio);
                    let py = grid_map::cell_center_texel(cy, ratio);
                    f64::from(ph2d_painter_brush::dab::grain_at(
                        spec_ref,
                        gb,
                        gimg,
                        px,
                        py,
                        d.center,
                        d.radius_px,
                    ))
                }
            });
            // As cerdas do Shape, pela MESMA porta do Grain e no referencial do motor: o desvio em células ao
            // centro do dab (`x − cx`, como `brush.rs::stamp_row_shaped`) — no neutro, o `sample_bristle` do
            // motor ao bit. Um Grain armado continua a tomar o lugar delas (W2.4; D7 medido no 27 §6.7).
            let cerdas_sb = shape_basis.as_ref().filter(|_| cerdas && grain.is_none());
            let cerdas_fn = cerdas_sb.map(|sb| {
                let forma = &spec_ref.shape;
                let px_por_celula = f64::from(ratio);
                let raio = d.radius_px;
                move |cx: i32, cy: i32| -> f64 {
                    let (fx, fy) = (f64::from(cx), f64::from(cy));
                    let dc = [fx - centro_x, fy - centro_y];
                    let tapete = |q: [f64; 2]| {
                        ph2d_painter_brush::texture::bristles_at(
                            forma,
                            sb,
                            q,
                            [fx, fy],
                            px_por_celula,
                            raio,
                        )
                    };
                    let Some(f) = fios_dab else {
                        return tapete(dc);
                    };
                    let p = match tabela {
                        Some(t) => f.no_ponto_em(dc.map(|v| v * px_por_celula), t),
                        None => f.no_ponto(dc.map(|v| v * px_por_celula)),
                    };
                    p.ganho * tapete(p.q.map(|v| v / px_por_celula))
                }
            });
            let lisa = |_: i32, _: i32| -> f64 { 1.0 };
            let grain_arg: Option<ph2d_wet_paint::brush::CellFn<'_>> =
                Some(match (grain.as_ref(), cerdas_fn.as_ref()) {
                    (Some(g), _) => g,
                    (None, Some(c)) => c,
                    (None, None) => &lisa,
                });
            // O ponto e o raio, em CÉLULAS — as duas coisas que o motor mede
            // na sua própria grade. O `d.radius_px` continua sendo o raio da
            // silhueta (em pixels), e é assim que o pincel não muda de tamanho
            // quando a razão muda: o mesmo disco, resolvido mais grosso.
            let cell_x = grid_map::px_to_cell(f64::from(x), ratio);
            let cell_y = grid_map::px_to_cell(f64::from(y), ratio);
            let cell_r = grid_map::px_len_to_cell(f64::from(d.radius_px), ratio);
            // A caixa do relevo (W3): a da PEGADA da silhueta (a extensão de `fp_raio` × o alcance do Shape).
            let meia = {
                let (e, alc) = (fp_raio.extent(), sess.engine.silhouette_reach[0].max(1.0));
                [0, 1].map(|i| cell_r * alc * f64::from(e[i]))
            };
            // O depósito e o pente leem a MESMA silhueta: guardada no dab ([`super::sil_memo`]), ao byte.
            let memo = relevo_dab.map(|_| super::sil_memo::SilMemo::nova(cell_x, cell_y, meia));
            let sil = |cx: i32, cy: i32| -> f64 {
                memo.as_ref()
                    .map_or_else(|| sil(cx, cy), |m| m.ou(cx, cy, sil))
            };
            if erasing {
                // W2.6: the same §9 mapping, routed to the engine's ERASER —
                // silhouette and grain shape the erase exactly as they shape
                // the deposit.
                sess.engine.dispatch_pressure_dab_erase(
                    cell_x,
                    cell_y,
                    b,
                    cell_r,
                    Some(&sil),
                    grain_arg,
                );
                // A borracha tira também o relevo que houver (sem o campo, nada a tirar).
                if !sess.engine.active_grid().altura.is_empty() {
                    let passos = passo.map_or(1.0, |s| f64::from(s) / px_cel);
                    sess.engine
                        .pentear_relevo(cell_x, cell_y, meia, &sil, &|_, _| 0.0, passos);
                }
                continue;
            }
            match wet_tool {
                WetTool::Paint => {
                    sess.engine.dispatch_pressure_dab_lane_cor(
                        li,
                        cell_x,
                        cell_y,
                        b,
                        cell_r,
                        opacidade * f64::from(peso),
                        Some(&sil),
                        grain_arg,
                        padrao_arg,
                        cor_arg,
                    );
                    if let Some(f) = relevo_dab {
                        let contacto = f64::from(d.radius_px / d.stroke_radius_px.max(1e-3));
                        // O alvo depende só do FIO: tabelado uma vez por dab (o ruído de cada fio custava dois
                        // Perlin por célula — doc 49_medicao §5, o custo).
                        let (k0, k1) = f.faixa((meia[0].hypot(meia[1]) + 2.0) * px_cel);
                        let (k0, k1) = (k0 - 1, k1 + 1);
                        let tabela: Vec<f64> = (k0..=k1)
                            .map(|k| relevo.alvo_do_fio(k, f.ganho_do_fio(k), contacto))
                            .collect();
                        let do_fio = |k: i32| -> f64 {
                            usize::try_from(k - k0)
                                .ok()
                                .and_then(|i| tabela.get(i).copied())
                                .unwrap_or_else(|| {
                                    relevo.alvo_do_fio(k, f.ganho_do_fio(k), contacto)
                                })
                        };
                        // Contínuo através dos fios (W3b): o degrau por fio era a escada do traço inclinado.
                        let alvo = |cx: i32, cy: i32| -> f64 {
                            let dc = [f64::from(cx) - centro_x, f64::from(cy) - centro_y];
                            let ([a, b], t) = f.entre_fios(dc.map(|v| v * px_cel));
                            ph2d_wet_paint::bristles_relevo::entre(do_fio(a), do_fio(b), t) / px_cel
                        };
                        sess.engine.pentear_relevo(
                            cell_x,
                            cell_y,
                            meia,
                            &sil,
                            &alvo,
                            passos_do_pente,
                        );
                    }
                }
                // Blend keeps the engine's own fixed-hardness stamp (the
                // model's tools ignore the brush silhouette on purpose).
                WetTool::Blend => sess
                    .engine
                    .dispatch_pressure_dab_lane_blend(li, cell_x, cell_y, b, cell_r),
                // Direct tools: per-dab grid ops; the displacement source is
                // THIS lane's previous centre (host-tracked per copy).
                WetTool::Wet | WetTool::Dry | WetTool::Blow | WetTool::Smear => {
                    sess.engine.dispatch_pressure_dab_tool(
                        wet_tool.engine(),
                        cell_x,
                        cell_y,
                        b,
                        cell_r,
                        lane_prev.map(|p| {
                            [
                                grid_map::px_to_cell(f64::from(p[0]), ratio),
                                grid_map::px_to_cell(f64::from(p[1]), ratio),
                            ]
                        }),
                    );
                }
            }
            sess.lanes[li].pos = d.center;
            sess.lanes[li].arco = d.arc_len;
        }
        let deposit_ms = t_dep.elapsed().as_secs_f32() * 1e3;
        self.paint.wetpaint.session = Some(taken);
        let t_comp = std::time::Instant::now();
        self.wetpaint_composite();
        crate::wet_diag::note_stamp(dabs.len(), deposit_ms, t_comp.elapsed().as_secs_f32() * 1e3);
    }
}

/// O estado do fluxo do sub-dab `k` da cápsula, tirado do do dab (o splitmix64).
fn sorteio_do_sub_dab(rng: u64, k: u64) -> u64 {
    let mut z = (rng ^ k.wrapping_mul(0xD1B5_4A32_D192_ED03)).wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}
