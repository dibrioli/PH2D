//! **What a wet SESSION is** — the data model behind [`PaintMode::WetPaint`],
//! split from `wetpaint.rs` for the workspace file-LOC cap. The parent keeps
//! what the tool DOES to a session (the dab route, the stroke lifecycle, the
//! identity guard); this file is what a session IS and how it reconciles with
//! the authored facts.

use super::*;

/// Everything the session pushes into the ENGINE (doc 22) — compared as one
/// value per batch/tick; the boot constant equals the engine's own boot (gate-pinned).
#[derive(Clone, Copy, PartialEq)]
pub(in crate::tool::paint) struct WetEngineFacts {
    pub(in crate::tool::paint) knobs: WetKnobs,
    pub(in crate::tool::paint) tilt: (bool, u8, u8),
    pub(in crate::tool::paint) km_mixing: bool,
    /// A **Opacity** do Wet Paint: a quantidade de pigmento (`sliders.pigmento`).
    pub(in crate::tool::paint) opacidade: f32,
    /// O «Glaze layering (K-M)»: a lei do ECRÃ, que assentar e re-molhar preservam ([`ph2d_wet_paint::assenta`]).
    pub(in crate::tool::paint) km_glaze: bool,
    /// O **Tooth** do Paper (BUGS #32) — chega ao motor também no tique, para o papel do PRÓPRIO motor
    /// re-assar quando só ele mexe (o papel do artista re-semeia pela [`PaperKey`]).
    pub(in crate::tool::paint) tooth: f32,
    /// As opções da W1g: a Água ao vivo e a Borda que pesa (`0..1`).
    pub(in crate::tool::paint) agua_viva: bool,
    pub(in crate::tool::paint) borda_que_pesa: f64,
}

impl WetEngineFacts {
    /// The session's `applied` init — the engine's REAL boot (`WetKnobs::ENGINE_BOOT` == `Engine::new`),
    /// NOT the tool's product default (`WetKnobs::DEFAULT`). The first reconcile then sees the authored
    /// knobs differ from this baseline and pushes them; collapsing the two would skip the product values.
    pub(in crate::tool::paint) const BOOT: Self = Self {
        knobs: WetKnobs::ENGINE_BOOT,
        tilt: (true, 4, 3),
        km_mixing: false,
        km_glaze: false,
        opacidade: 1.0,
        tooth: 1.0,
        agua_viva: false,
        borda_que_pesa: 0.0,
    };
}

pub(in crate::tool::paint) struct WetSession {
    /// O motor (grid + sim + tuning), do tamanho da tela da camada — e **onde
    /// ele está**: na thread do frame ou com o worker da sim
    /// ([`super::offthread`]). Pergunte por [`Self::eng`], nunca pelo slot.
    pub(in crate::tool::paint) engine: EngineSlot,
    /// **Quantos pixels de canvas medem uma célula desta grade** — congelado no
    /// nascimento (ver [`super::grid_map`]). A razão AUTORADA vive no
    /// `WetPaintState`; esta é a que o motor vivo de fato tem, e é ela que a
    /// rota do dab e o composite perguntam. Trocar a autorada encerra a sessão,
    /// então as duas concordam sempre — mas quem responde *"de que tamanho é a
    /// grade que estou usando?"* tem de ser a grade, não a preferência.
    pub(in crate::tool::paint) ratio: u8,
    /// As dimensões da grade (sem o pad ring), derivadas de `(source_size,
    /// ratio)` uma vez no nascimento. O `pigment` abaixo tem este tamanho, não
    /// o do canvas.
    pub(in crate::tool::paint) grid: (usize, usize),
    /// A thread da sim, criada no primeiro `hand_off_sim`. `None` = o motor
    /// nunca saiu daqui (sessão recém-nascida, ou um teste que só dirige).
    pub(in crate::tool::paint) worker: Option<SimWorker>,
    /// Quantos passos do worker este tool já COMPOSITOU. É o que responde
    /// *"há algo novo na tela?"* sem trazer o motor para casa — sem isto o tick
    /// buscaria o engine a cada frame e o worker perderia o núcleo esperando.
    pub(in crate::tool::paint) seen_steps: u64,
    /// The canvas frozen at session start — every composite renders over THIS.
    /// `pub(super)` so [`PainterTool::wet_splat_gates`] can serve it as the
    /// alpha-lock's α reference (the frozen base the composite reads).
    pub(in crate::tool::paint) base: Arc<Vec<u8>>,
    /// O modo de **Blend** com que esta tinta molhada se compõe sobre a [`Self::base`] — o da SESSÃO,
    /// congelado quando ela nasce ([`super::modo_da_sessao`]); trocá-lo com tinta na tela fecha a
    /// sessão (`set_brush_blend`).
    pub(in crate::tool::paint) blend: ph2d_painter_brush::BrushBlend,
    /// The exact `canvas_rgba` allocation OUR last composite produced (or the
    /// one the session started from). A mismatch = foreign mutation = session
    /// over. `pub(super)` so the doc-21 commit door can re-arm it after an
    /// OWNED authoring write (`wetpaint_commit::wetpaint_rearm_after_own_write`).
    ///
    /// ⚠️ **`Weak`, never `Arc` — the guard asks about IDENTITY, not ownership,
    /// and a second strong handle is not free.** `wetpaint_composite` ends in
    /// `Arc::make_mut(&mut self.canvas_rgba)`, which hands back the slice only
    /// to a SOLE owner and **copies the whole canvas** otherwise: measured,
    /// this handle turned every pointer move into a full document copy —
    /// **9.86 ms at 4096²**, the one cost that made the Wet Paint move scale
    /// with the canvas while the other three media stayed flat. A `Weak` costs
    /// **0.0000 ms** (`make_mut` moves the 24-byte `Vec` header to a fresh
    /// allocation and leaves the pixels where they are) — measured, not cited.
    ///
    /// And it is the *right* handle for a second reason: a `Weak` keeps the
    /// allocation alive, so no later `Arc` can be born at that address — the
    /// ABA that makes a raw-pointer identity check unsound (the ADR-0124
    /// lesson: **ask the value, never `as_ptr()` of something you don't pin**).
    pub(in crate::tool::paint) canvas: Weak<Vec<u8>>,
    /// Session-persistent pigment scratch — **`grid.0 * grid.1 * 4`, o tamanho
    /// da GRADE e não o do canvas** (o render de pigmento do motor escreve uma
    /// linha de célula por vez; o composite é quem sobe para o canvas). The
    /// region render fully overwrites the rect it is asked for, so stale bytes
    /// outside are inert.
    pub(super) pigment: Vec<u8>,
    /// Per-LANE stroke state (one lane per symmetry copy / tile offset,
    /// matched geometrically): last dab centre (the chord source) + the last
    /// fresh-ink colour sent (Randomize change detector). Cleared per stroke.
    pub(super) lanes: Vec<Lane>,
    /// A direct stroke is open in the engine (pen-down .. pen-up).
    pub(super) stroke_open: bool,
    /// What the engine's PAPER plane was last seeded from: `None` = the
    /// engine's own preset tile (session birth / paper disarmed); `Some` =
    /// the artist's Paper slot, keyed by everything the seed reads. The
    /// session **reconciles** against this every dab batch — paper is
    /// substrate under live water, but the SLOT is authored state, and the
    /// session spans strokes: seeding only at session birth left every later
    /// adjustment (Brightness/Contrast, any knob, the arm itself) silently
    /// ignored until the bake (Enio 2026-07-21, "os ajustes ... não estão
    /// sendo levados em consideração").
    pub(super) paper_key: Option<PaperKey>,
    /// The [`WetEngineFacts`] last pushed into the engine — starts at the
    /// boot constant (the engine boots with exactly it, gate-pinned) and the
    /// reconcile pushes the authored values when they differ. Same law as
    /// `paper_key`: the facts are authored state, the session is not.
    pub(super) applied: WetEngineFacts,
    /// **O rascunho da autoria, na PRÓPRIA água** — o recorte do grid de antes
    /// do depósito que este quadro carimbou, para o quadro seguinte poder
    /// desfazê-lo (o gêmeo exato do [`DragPreview`] do canvas, uma camada
    /// abaixo).
    ///
    /// ⚠️ Ele mora na SESSÃO, e não no `WetPaintState`, porque descreve o grid
    /// DESTA sessão: se o guard de identidade a matar, o recorte descreve uma
    /// água que não existe mais e morre com ela. Guardá-lo fora seria a lista
    /// de sítios de limpeza que o §13.13 já pagou uma vez.
    pub(super) preview: Option<ph2d_wet_paint::grid::GridPatch>,
    /// **Os lotes que chegaram com o motor na simulação** e o traço aberto (W1e, F6): pousam pela
    /// mesma rota, pela ordem, no tick seguinte (ou no lote seguinte, ou no fim do traço). Moram na
    /// sessão porque descrevem a água DELA.
    pub(super) adiados: Vec<LoteAdiado>,
    /// A fita da macro ([`super::fita`]), herdada do tool no nascimento: grava as respostas do worker,
    /// ou responde por ele ao tocar. `None` = nem uma coisa nem outra (custo zero).
    pub(super) fita: Option<super::fita::FitaPartilhada>,
    /// A tocar: o motor já foi ao worker do roteiro (o equivalente de `worker.is_some()`).
    pub(super) roteiro_teve_worker: bool,
    /// O ritmo escrito do worker de um gate ([`super::fita::RitmoDaAgua`]), herdado do tool.
    pub(super) ritmo: Option<super::fita::RitmoDaAgua>,
}

/// Um lote de dabs adiado — tudo o que a rota do dab lê por LOTE.
#[derive(Clone)]
pub(super) struct LoteAdiado {
    pub(super) dabs: Vec<Dab>,
    pub(super) brush: BrushSpec,
    pub(super) groups: Vec<u32>,
    pub(super) tex_rng: u64,
}

/// Everything the paper seed reads — the reconcile re-seeds exactly when one
/// of these moves, and an unchanged key costs one small struct compare per
/// batch. The image rides by its monotonic VERSION, never the `Arc` address
/// (an address survives a content swap and a content survives an address
/// swap — the ADR-0124 cache-identity trap).
#[derive(Clone, Copy, PartialEq)]
pub(super) struct PaperKey {
    pub(super) tex: ph2d_painter_brush::TextureSettings,
    pub(super) image_version: u64,
    pub(super) period: [f32; 2],
    /// O **Tooth** (`paper_depth`) — o seed escala o relevo por ele ([`ph2d_wet_paint::paper::dente`]).
    pub(super) tooth: f32,
}

impl WetSession {
    /// Há tinta molhada na tela? (algum pixel do plano de pigmento com alfa) — o que decide se trocar
    /// o modo de Blend fixa a sessão ou só lhe troca o modo (`set_brush_blend`).
    pub(in crate::tool::paint) fn ha_tinta(&self) -> bool {
        self.pigment.as_chunks::<4>().0.iter().any(|p| p[3] > 0)
    }

    /// Push the authored FACTS into the engine when they moved (W3, grown by
    /// doc 22). ONE door for the stamp AND the tick — Dry Speed and Gravity
    /// act on water already sitting on the canvas, so a knob must land while
    /// the sim runs, not only at the next stroke. Unchanged facts cost one
    /// struct compare. The tuning knobs go through `Engine::set_knob` (the
    /// door that reacts to what a knob invalidates: bristle rebuild lazy,
    /// paper re-bake, repaint) — and the paper re-bake only fires while the
    /// ENGINE's own tile is the tooth source (an armed artist Paper slot
    /// overrides it; its three physical knobs hide in the panel then).
    pub(super) fn reconcile_facts(&mut self, f: WetEngineFacts) {
        use ph2d_wet_paint::tuning::KNOB_DEFS;
        if self.applied == f {
            return;
        }
        let a = self.applied;
        let own_paper = self.paper_key.is_none();
        self.bring_home();
        let e = &mut *self.engine;
        if f.knobs.water != a.knobs.water {
            e.sliders.water = f.knobs.water;
        }
        if f.knobs.erase != a.knobs.erase {
            e.sliders.erase = f.knobs.erase;
        }
        if f.opacidade != a.opacidade {
            e.sliders.pigmento = f64::from(f.opacidade);
        }
        let moved = KNOB_DEFS.iter().zip(&f.knobs.knobs).zip(&a.knobs.knobs);
        for ((def, new), old) in moved {
            if new != old {
                e.set_knob(def.knob, *new);
            }
        }
        if f.tilt != a.tilt {
            let (on, ring, spoke) = f.tilt;
            let d = ph2d_wet_paint::sim::tilt_dir_for_spoke(spoke);
            e.sim.tilt_on = on;
            e.sim.tilt_dir_x = d[0];
            e.sim.tilt_dir_y = d[1];
            e.sim.tilt_scale = f64::from(ring) / 4.0;
        }
        if f.km_mixing != a.km_mixing {
            e.sim.km_mixing = f.km_mixing;
            // A K–M já é subtractiva: a velatura por cima dela contava a absorção duas vezes, e o ciclo
            // assenta ↔ re-molha levava um miolo molhado a PRETO (prompt 15, 05 §4b «Mistura»).
            e.sim.glaze = if f.km_mixing {
                0.0
            } else {
                ph2d_wet_paint::colorops::VELATURA_MEDIDA
            };
        }
        if f.km_glaze != a.km_glaze {
            // «Glaze layering»: camadas transparentes. O ecrã e as transferências verticais lêem a lei da
            // sessão pela mesma porta (`Sim::gather_params`).
            e.sim.camadas_transparentes = f.km_glaze;
        }
        if f.agua_viva != a.agua_viva {
            e.sim.sim_while_painting = f.agua_viva;
            // A velatura do filme depende do modo (medida nos dois, `colorops`).
            e.sim.glaze_filme = if f.agua_viva {
                ph2d_wet_paint::colorops::VELATURA_DO_FILME_VIVA
            } else {
                ph2d_wet_paint::colorops::VELATURA_DO_FILME
            };
        }
        if f.borda_que_pesa != a.borda_que_pesa {
            e.sim.sedimenta = f.borda_que_pesa * super::SEDIMENTA_DA_BORDA;
        }
        if f.tooth != a.tooth {
            e.set_paper_tooth(f.tooth);
        }
        if own_paper && e.paper_dirty() {
            e.rebake_paper();
        }
        self.applied = f;
    }
}

/// **A cópia de uma sessão** (o instantâneo do tocador de uma macro, [`PainterTool::instantaneo`]): o
/// motor tem de estar NESTA thread — um worker vivo não se copia, e quem chama confere antes
/// ([`PainterTool::wet_copiavel`]). A [`Self::canvas`] da cópia aponta a mesma alocação que a tela
/// copiada partilha (`Arc`), então a guarda de identidade passa; a fita é a mesma até quem copia
/// instalar a dele.
impl Clone for WetSession {
    fn clone(&self) -> Self {
        let WetSession {
            engine,
            ratio,
            grid,
            worker,
            seen_steps,
            base,
            blend,
            canvas,
            pigment,
            lanes,
            stroke_open,
            paper_key,
            applied,
            preview,
            adiados,
            fita,
            roteiro_teve_worker,
            ritmo,
        } = self;
        assert!(
            worker.is_none(),
            "WetSession::clone com um worker vivo — `wet_copiavel` recusa antes"
        );
        WetSession {
            engine: engine.clone(),
            ratio: *ratio,
            grid: *grid,
            worker: None,
            seen_steps: *seen_steps,
            base: Arc::clone(base),
            blend: *blend,
            canvas: Weak::clone(canvas),
            pigment: pigment.clone(),
            lanes: lanes.clone(),
            stroke_open: *stroke_open,
            paper_key: *paper_key,
            applied: *applied,
            preview: preview.clone(),
            adiados: adiados.clone(),
            fita: fita.clone(),
            roteiro_teve_worker: *roteiro_teve_worker,
            ritmo: ritmo.clone(),
        }
    }
}

impl PainterTool {
    /// A tinta molhada deixa-se copiar: sem sessão, ou com o motor nesta thread e sem worker (o tocador
    /// de uma macro nunca tem um: o worker dele é o roteiro).
    pub(crate) fn wet_copiavel(&self) -> bool {
        self.paint.wetpaint.session.as_ref().is_none_or(|s| {
            s.worker.is_none() && !matches!(s.engine, super::offthread::EngineSlot::Away)
        })
    }

    /// **O que a água de uma cópia retém**: o motor (sem o rascunho do solver) e o plano de pigmento. A
    /// `base` é partilhada e ninguém a escreve; o rascunho da autoria é um recorte do dab — não contam.
    pub(crate) fn wet_bytes_retidos(&self) -> usize {
        use super::offthread::EngineSlot;
        self.paint.wetpaint.session.as_ref().map_or(0, |s| {
            let motor = match &s.engine {
                EngineSlot::Here(e) | EngineSlot::Roteiro(e) => e.bytes_retidos(),
                EngineSlot::Away => 0,
            };
            motor + s.pigment.len()
        })
    }

    /// **O plano de pigmento da sessão molhada** (o que decide «há tinta?» ao trocar o Blend) — para as
    /// sondas e os gates; `None` sem sessão.
    #[must_use]
    pub fn wet_pigmento(&self) -> Option<&[u8]> {
        self.paint
            .wetpaint
            .session
            .as_ref()
            .map(|s| s.pigment.as_slice())
    }

    /// A fita que a água deste tool usa (a do tocador de uma macro).
    pub(crate) fn wet_fita(&self) -> Option<super::fita::FitaPartilhada> {
        self.paint.wetpaint.fita.clone()
    }

    /// A fita deixa de ser partilhada com quem este tool foi copiado: uma cópia PROFUNDA dela (as
    /// respostas que faltam, a divergência) instalada no tool e na sessão. Devolve-a.
    pub(crate) fn wet_fita_propria(&mut self) -> Option<super::fita::FitaPartilhada> {
        let f = self.paint.wetpaint.fita.as_ref()?;
        let propria = Arc::new(std::sync::Mutex::new(super::fita::trava(f).clone()));
        self.wet_instala_fita(Some(Arc::clone(&propria)));
        Some(propria)
    }
}

/// One replication lane of the open stroke — see [`WetSession::lanes`].
#[derive(Clone)]
pub(super) struct Lane {
    pub(super) pos: [f32; 2],
    pub(super) ink: [f32; 3],
}
