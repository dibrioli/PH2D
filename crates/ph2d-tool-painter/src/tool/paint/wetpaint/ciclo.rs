//! O CICLO de uma sessão molhada no tool (filho de [`super`], LOC cap): quem é dono dos dabs, a marca do
//! artista, o fim do traço, os lotes adiados, o tique do quadro e o guarda da identidade da tela.

use super::*;

impl PainterTool {
    /// Whether the WET module owns this batch of dabs — the ONE routing
    /// question, asked by BOTH `stamp_dabs` (to bypass the snapshot/restore
    /// wrapper, which would kill the session — see W2.5) and
    /// `stamp_dabs_inner` (to enter the wet arm). Two copies of this
    /// condition would diverge, and the diverged half is a canvas gate that
    /// either leaks or kills the water.
    ///
    /// Wet Paint owns (doc 21): the INCREMENTAL methods' live batches and the
    /// commit door's `deposit_pass` replay — nothing else. A non-incremental
    /// AUTHORING batch (DragDot / Anchored / the shape editors' re-stamps) is
    /// deliberately UN-owned: it falls through the completely normal flat
    /// pipeline (snapshot/restore wrapper + colour routes), which is what
    /// makes the flat preview — and hands it Selection/protection/alpha-lock
    /// byte-identically to Paint mode. The eraser with no live session (W2.6)
    /// also falls through (it erases the BAKED canvas, which is what is
    /// visibly there). Non-wet modes: the mode conjunct short-circuits first —
    /// not one instruction changes (gate G0a).
    pub(in crate::tool::paint) fn wet_owns_the_dabs(&self) -> bool {
        matches!(self.paint.paint_mode, PaintMode::WetPaint)
            && (self.paint.brush.stroke_method.is_incremental() || self.paint.wetpaint.deposit_pass)
            && (!self.paint.eraser || self.paint.wetpaint.session.is_some())
    }

    /// **A tinta molhada do artista pinta POR CIMA da imagem** (W12 das macros, decisão do dono 20/10): com
    /// a marca a escrever, a sessão que não nasceu com ela (a da gravação, ainda molhada) assa antes do 1.º
    /// dab do artista e a dele nasce aqui — antes de qualquer marca, para o nascimento a ver limpa. A
    /// borracha molhada apaga o fluido que houver: fica com a lei da W9.
    pub(in crate::tool::paint) fn wet_da_marca(&mut self) {
        if !self.undo.marca.ativa()
            || self.undo.marca.suspensa
            || self.paint.eraser
            || !self.wet_owns_the_dabs()
        {
            return;
        }
        self.wetpaint_guard();
        let dela = self.paint.wetpaint.session.as_ref().map(|s| s.marca);
        if dela.is_some_and(|id| !self.undo.marca.molhada_viva(id)) {
            self.wetpaint_end_session();
        }
        self.ensure_wet_session();
    }

    /// Os dabs de agora vão para a sessão molhada do artista: a cobertura deles sai do composite.
    pub(in crate::tool::paint) fn wet_lote_do_artista(&self) -> bool {
        self.wet_owns_the_dabs()
            && self
                .paint
                .wetpaint
                .session
                .as_ref()
                .is_some_and(|s| self.undo.marca.molhada_viva(s.marca))
    }

    /// Pen-up: close the engine's direct stroke (the sim resumes). Called from
    /// `paint_end`; the session itself stays alive — the water is still wet.
    pub(in crate::tool::paint) fn wetpaint_stroke_end(&mut self) {
        // A fila pousa ANTES de o gesto deixar de ser vivo (a rota recusa um lote de um gesto morto).
        self.wetpaint_pousa_adiados();
        // As pontas rectas fecham o fim com o gesto ainda vivo ([`tampa`]).
        self.fecha_as_pontas_rectas();
        self.paint.wetpaint.live_gesture = false;
        if self.paint.tracos_fora > 0 {
            return; // a água é dos traços abertos: o último fecha-a (W16)
        }
        if let Some(sess) = self.paint.wetpaint.session.as_mut()
            && sess.stroke_open
        {
            sess.bring_home();
            sess.engine.end_direct_stroke();
            sess.stroke_open = false;
            sess.lanes.clear();
        }
    }

    /// **Pousa os lotes adiados** ([`session::LoteAdiado`]) pela rota de sempre, com o estado por lote
    /// de cada um — trazendo o motor (bloqueia ≤ um estágio: quem chama já decidiu que é a hora).
    pub(in crate::tool::paint) fn wetpaint_pousa_adiados(&mut self) {
        let Some(sess) = self.paint.wetpaint.session.as_mut() else {
            return;
        };
        if sess.adiados.is_empty() {
            return;
        }
        sess.bring_home();
        let lotes = std::mem::take(&mut sess.adiados);
        let (grupos, semente) = (
            std::mem::take(&mut self.paint.dab_groups),
            self.paint.tex_rng,
        );
        for l in lotes {
            self.paint.dab_groups = l.groups;
            self.paint.tex_rng = l.tex_rng;
            self.stamp_dabs_wetpaint(&l.dabs, &l.brush);
        }
        self.paint.dab_groups = grupos;
        self.paint.tex_rng = semente;
    }

    /// Per-frame heartbeat: **MOSTRA o que o worker já simulou** e devolve o
    /// motor a ele. No session = a true no-op (the OFF contract: not one byte is
    /// looked at).
    ///
    /// ⚠️ **O frame não dá passo nenhum** — a sim vive em [`offthread`], no
    /// relógio DELA. O tick faz três coisas: pergunta ao contador se há passo
    /// novo (atômico, sem trazer o motor), composita se houver, e entrega o
    /// motor de volta. O `dt` do shell deixou de ser entrada da água: era ele
    /// que fechava o laço de realimentação da era do agendamento (doc 28 §5.31).
    pub(in crate::tool::paint) fn wetpaint_tick(&mut self, dt_s: f32) {
        // Doc 21 §F layer 2: the deposit flag must never survive into a tick.
        debug_assert!(!self.paint.wetpaint.deposit_pass);
        if self.paint.wetpaint.session.is_none() {
            return;
        }
        self.wetpaint_guard();
        // Doc 21 law D: the water FREEZES while the artist authors a flat
        // preview (flow AND drying) — zero steps, zero composites, zero knob
        // reconciles; a composite would copy `sess.base` verbatim where the
        // pigment is empty and ERASE the preview inside its dirty rect (a
        // torn state). The guard above still ran: foreign swaps (undo, layer
        // switch) kill the session even mid-authoring. The hold is DERIVED
        // (`drag_preview` is the flat re-stamp record in this mode) — no
        // state, no release door; commit/cancel drop the record and the
        // next tick simply resumes.
        if self.wet_authoring_hold() {
            // ⚠️ **TRAZER O MOTOR PARA CASA é o que torna a pausa REAL** — sem
            // isto o `return` congela só o DISPLAY (Enio 2026-08-03: *"uma pausa
            // falsa na simulação — pausa no visual mas está ocorrendo por
            // trás"*; medido: **12 passos em 300 ms**, o ritmo nominal cheio,
            // com a tela parada).
            //
            // ⚠️ **A lei estava certa e foi a MUDANÇA DE THREAD que a quebrou.**
            // Enquanto a sim rodava dentro do tick, este `return` de fato
            // significava *zero passos*; com a sim em thread própria (doc 29) o
            // motor já está com o worker desde o tick anterior, e o mesmo
            // `return` passou a significar apenas *não OLHE*. O worker seguia no
            // relógio dele e a evolução acumulada SALTAVA na tela quando o
            // artista soltava — e o comentário aqui continuou afirmando "zero
            // passos" por uma wave inteira. *Um `return` que congelava o mundo
            // congela só a vista no dia em que o mundo muda de thread.*
            //
            // Pede e não espera (a porta do tick): se o motor está no canal, o
            // `want` fica armado e o tick seguinte o encontra — no máximo um
            // punhado de passos escapa antes de a pausa fechar.
            if let Some(sess) = self.paint.wetpaint.session.as_mut() {
                sess.try_bring_home();
            }
            return;
        }
        self.wet_varre_oleo();
        if let Some(divida) = self.paint.wetpaint.relogio_fixo.as_mut() {
            *divida += f64::from(dt_s) / offthread::STEP_S;
            let passos = divida.floor();
            *divida -= passos;
            self.wet_step_sync(passos as usize);
            return;
        }
        let facts = self.wet_facts();
        let Some(sess) = self.paint.wetpaint.session.as_mut() else {
            return;
        };
        // ── Há algo novo a mostrar? ─────────────────────────────────────────
        //
        // Perguntado ao CONTADOR de passos do worker (um atômico), NUNCA
        // trazendo o motor para casa: sem isto o tick buscaria o engine a cada
        // frame só para descobrir que nada mudou, e o worker perderia ~30% do
        // núcleo esperando na fronteira de estágio.
        let done = sess.sim_steps();
        let fresh = done != sess.seen_steps;
        // Facts land while the water SITS too (Dry Speed / Gravity / the
        // tilt are sim forces): the tick reconciles with the stamp's door.
        let facts_moved = sess.applied != facts;
        let adiados = !sess.adiados.is_empty();
        if fresh || facts_moved || adiados {
            // ⚠️ **PEDE e não espera.** O tick é a única porta que pode voltar
            // de mãos vazias (a água corre a ~33 Hz, o display a 60 ⇒ mostrar
            // um passo no frame seguinte é invisível), e bloquear aqui custava
            // o estágio inteiro DENTRO do frame — medido, 60,6 ms de pior tick
            // na poça de três traços. O `seen_steps` só anda quando de fato
            // compositamos, então nada é perdido: o frame seguinte tenta.
            //
            // ⚠️ **Sem `return` no ramo de falha, e a razão é o `hand_off_sim`
            // abaixo:** quando o pedido falha o motor está **no canal** — nem
            // aqui, nem com o worker —, e cair no hand-off (que é no-op ali)
            // mantém a estrutura honesta. ⚠️ **Isto NÃO é o que custou 60× de
            // taxa**, embora eu tenha escrito isso antes de reler a medição: o
            // colapso 33,4 → 0,5 Hz foi o `want` sobrevivendo à entrega
            // (`hand_off_sim`), e consertar o `return` não mudou um Hz. As duas
            // mutações estão documentadas em `offthread_tests.rs`.
            let t_wait = std::time::Instant::now();
            let got = sess.try_bring_home();
            crate::wet_diag::note_wait(t_wait.elapsed().as_secs_f32() * 1e3);
            if got {
                sess.seen_steps = done;
                sess.reconcile_facts(facts);
                if adiados {
                    self.wetpaint_pousa_adiados();
                }
                if fresh {
                    let t_comp = std::time::Instant::now();
                    self.wetpaint_composite();
                    // Diagnóstico (`PH2D_FLUID_PROFILE`): a metade COMPOSITE do
                    // tick. Com a sim fora da thread, ela é a ÚNICA metade que o
                    // frame ainda paga.
                    crate::wet_diag::note_composite(t_comp.elapsed().as_secs_f32() * 1e3);
                }
            }
        }
        // ⚠️ **O hand-off é INCONDICIONAL** (no-op se o motor já está lá), e não
        // um `else` do bloco acima: gateá-lo em "houve trabalho" deixaria uma
        // sessão cujos facts nascem iguais ao boot com o motor em casa para
        // sempre — água que nunca simula, com todos os gates verdes.
        if let Some(sess) = self.paint.wetpaint.session.as_mut() {
            sess.hand_off_sim();
        }
    }

    /// The canvas-identity guard (module doc): a foreign `canvas_rgba` swap
    /// ends the session before anything composites over restored pixels.
    ///
    /// The comparison is `Weak::as_ptr` against `Arc::as_ptr` — see
    /// [`WetSession::canvas`] for why the token is weak, and why comparing its
    /// address is sound *because* it is weak (the handle pins the allocation,
    /// so the address it names can never be re-issued to a different canvas).
    pub(super) fn wetpaint_guard(&mut self) {
        if let Some(sess) = &self.paint.wetpaint.session
            && !std::ptr::eq(sess.canvas.as_ptr(), Arc::as_ptr(&self.canvas_rgba))
        {
            self.paint.wetpaint.session = None;
            self.undo.marca.acaba_a_molhada();
        }
    }
}
