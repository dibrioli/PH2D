//! **Fase do quadro: OS RELÓGIOS DO PASSO FIXO** — o EWMA do tempo de quadro e o batimento da ferramenta
//! activa, o acumulador do passo fixo e as três cabeças de leitura (cena, clip, container), quem está sob
//! pré-visualização de ferramenta, e os dois relógios que correm nos mesmos tiques: a §11 Animation e os
//! timers (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! ⚠️ **A primeira fase que DEVOLVE um contexto nomeado** ([`FrameClocks`]): os quatro locais que
//! atravessam a fronteira voltam ao quadro com os MESMOS nomes, e o resto do corpo não muda uma linha.
//! ⚠️ Os sinais NÃO se publicam aqui — o tique corre antes do dreno da timeline (ver o corpo).

use super::*;

/// **O CONTEXTO que os relógios do passo fixo entregam ao resto do quadro** — os locais que ATRAVESSAM a
/// fronteira da fase, com os MESMOS nomes que o corpo do quadro já usava.
pub(super) struct FrameClocks {
    /// Os tiques que o passo fixo correu neste quadro (e o tempo largado pelo tecto de sub-passos).
    pub(super) report: ph2d_core::FixedStepReport,
    /// Quem está sob pré-visualização de ferramenta — lido pelo tique da §11, pelo extract e pela grelha.
    pub(super) tool_preview_bits: [Option<u64>; 3],
    /// Os sinais da §11 Animation, publicados DEPOIS do dreno da timeline.
    pub(super) anim_signals: Vec<sprite_anim_tick::AnimSignal>,
    /// Os sinais dos timers, publicados no mesmo sítio.
    pub(super) timer_signals: Vec<timer_tick::TimerSignal>,
    /// ⭐⭐⭐ **Quem morreu de velho neste quadro** (TOP-20 #12). As mortes são drenadas na
    /// `fase_signal_outbox`, com as do fora-do-ecrã — ⛔ **nunca aqui**: o oráculo mediu que um
    /// moribundo continua visível a toda consulta até ao fim do quadro, e é isso que torna a
    /// travessia do quadro segura sem uma regra escrita em lado nenhum.
    pub(super) deaths: Vec<ph2d_ecs::Death>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo. O `None` é inalcançável (os guardas correram na `fase_chrome_clock`).
    pub(super) fn fase_fixed_step_clocks(&mut self, wall_dt: f64) -> Option<FrameClocks> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx { tools, sim, .. } = FrameGfx::of(gfx);

        // Drive fixed-step accumulator.
        // M14.4g: feed EWMA frame-time using the same `wall_dt` —
        // single source of truth for "how long did the last frame
        // take". α=0.1 smooths over jitter while still tracking
        // sustained changes in ~10 frames.
        let frame_ms_now = (wall_dt * 1000.0) as f32;
        const ALPHA: f32 = 0.1;
        self.frame_ms_ewma = ALPHA * frame_ms_now + (1.0 - ALPHA) * self.frame_ms_ewma;
        // **W15.1 (ADR-0040-amendment-2):** per-frame heartbeat on the ACTIVE tool,
        // with the real frame delta. Drives the watercolor live wet-on-wet diffusion
        // (ADR-0049 / ADR-0077 D11) so the wash keeps blooming + drying after pen-up;
        // a no-op default for every other tool, so this costs nothing elsewhere.
        if let Some(t) = tools.active_mut() {
            // Frame profiler: the heartbeat is where the watercolor live recomposite runs while the
            // brush is held (soak pour → apply_watercolor) — time it so a paint slowdown is attributable
            // (one Instant when profiling; zero cost otherwise).
            let tick_t0 = frame_prof_on().then(Instant::now);
            t.on_tick(frame_ms_now);
            if let Some(t0) = tick_t0 {
                let us = t0.elapsed().as_micros() as u64;
                // A janela: soma, pico e quantos frames de fato trabalharam.
                if us > 0 {
                    FRAME_PROF_TICK_SUM_US.with(|c| c.set(c.get() + us));
                    FRAME_PROF_TICK_MAX_US.with(|c| c.set(c.get().max(us)));
                    FRAME_PROF_TICK_N.with(|c| c.set(c.get() + 1));
                }
                let st = self.last_paint_stamp_us;
                if st > 0 {
                    FRAME_PROF_STAMP_SUM_US.with(|c| c.set(c.get() + st));
                    FRAME_PROF_STAMP_MAX_US.with(|c| c.set(c.get().max(st)));
                    FRAME_PROF_STAMP_N.with(|c| c.set(c.get() + 1));
                    // ⚠️ O DIVISOR, acumulado no MESMO ponto que a soma: contado
                    // noutro lugar, os dois baldes discordariam sobre QUAIS
                    // frames entraram na janela, e a média por evento seria de
                    // uma amostra que ninguém escolheu.
                    FRAME_PROF_STAMP_EV.with(|c| c.set(c.get() + self.last_paint_stamps));
                }
            }
            // Fill dwell gesture: a held-still ColorDrop fires the fill + enters live threshold-adjust
            // (see `input_dispatch::fill_drag`). `last_pointer` is disjoint from `self.gfx.tools`.
            crate::input_dispatch::fill_drag::fill_drag_tick(t, self.last_pointer, frame_ms_now);
        }
        let report = self.fixed_step.advance(wall_dt);
        if report.dropped_secs > 0.0 {
            eprintln!(
                "warn: dropped {:.3}s of sim time (max_substeps cap)",
                report.dropped_secs
            );
        }
        panic::set_frame_id(self.fixed_step.tick_count());
        // Advance the engine-wide timeline cursor by the ticks that ran this
        // frame. Every animatable system samples the Playhead for its current
        // value; while paused it holds. (General timeline, M0 time wire.)
        self.playhead.advance_ticks(report.ticks);
        // The Keys view's clip clock advances on the same ticks — it only moves
        // when ITS transport is playing, so advancing both is harmless and keeps
        // "play a clip's keys" working without a second tick source.
        self.clip_playhead.advance_ticks(report.ticks);
        // The Containers view's clock (Enio, 2026-07-22): editing a container's
        // lanes plays the CONTAINER's own interior, not the scene. A third clock,
        // like the clip's — it only advances while ITS transport plays.
        self.container_playhead.advance_ticks(report.ticks);

        // **QUEM ESTÁ SOB PRÉ-VISUALIZAÇÃO DE FERRAMENTA**, calculado UMA vez e lido por três
        // sítios deste quadro: o tique da §11 (uma folha em pintura toca), o `preview_overrides`
        // do extract, e o overlay da grelha (as linhas seguem o quad desdobrado).
        //
        // ⚠️ **Um `let`, e não três leituras dos mesmos campos.** Os três respondem à mesma
        // pergunta, e com uma cópia em cada um deles a próxima fonte de pré-visualização entraria
        // só em dois — dando linhas sobre um quad que não desdobrou, ou uma folha parada a pintar.
        let tool_preview_bits: [Option<u64>; 3] = [
            self.painter_preview_gpu.map(|g| g.entity_bits),
            self.bgremoval.preview_gpu.map(|g| g.entity_bits),
            self.painter_shape_source_preview_gpu.map(|g| g.entity_bits),
        ];

        // **A §11 ANIMATION anda AQUI**, ao lado dos outros relógios e pela mesma razão: um
        // `SpriteAnimator` é `SimComponent` e o replay tem de reproduzir o frame avançado, o que
        // só um passo fixo e contado dá.
        //
        // ⚠️ **`report.ticks × fixed_dt` numa chamada só** — e este comentário já disse o
        // contrário («nunca um passo grande, um salto atravessaria o fim de um ciclo sem o
        // fechar»). Era falso, e foi uma mutação que o disse: a `ph2d_ecs::advance` tem laço de
        // recuperação próprio e fecha exatamente os mesmos ciclos. Gate:
        // `catching_up_in_one_call_is_the_same_as_catching_up_step_by_step`.
        // **OS SINAIS DA §11** (spec §8.10) saem daqui e são publicados ao lado dos da física, no
        // MESMO outbox — ver o produtor 2, abaixo. ⚠️ Não se publica aqui: o tique corre **antes**
        // do dreno da timeline, e um sinal publicado antes de o quadro virar chegaria ao consumidor
        // um quadro atrasado. Um atraso de um quadro é invisível num toast e deixa de o ser no dia
        // em que o consumidor for SOM.
        let anim_signals = sprite_anim_tick::tick_sprite_animations(
            sim,
            report.ticks,
            self.fixed_step.fixed_dt(),
            &tool_preview_bits,
            // ⚠️ **O relógio e a célula que ele produz são pré-visualização**, e é esta declaração
            // que impede que cada clique dado durante a reprodução vire um Ctrl+Z vazio.
            &mut self.preview_drive,
        );
        // ⭐⭐⭐ **OS TIMERS** (TOP-20 #2) — no MESMO sítio e pela mesma razão que os da §11: o
        // relógio corre no passo fixo (o replay reproduz o instante de disparo) e os sinais são
        // publicados **depois** do dreno, junto dos da física.
        //
        // ⚠️ **Sem o `preview_drive`, e é a separação que o paga:** o `TimerRuntime` não é um
        // componente registado, então o undo não o fotografa e não há passo espúrio a declarar.
        let timer_signals = timer_tick::tick_timers(sim, report.ticks, self.fixed_step.fixed_dt());
        // ⭐⭐⭐ **AS VIDAS** (TOP-20 #12) — no MESMO sítio e pela mesma razão que os timers: o
        // relógio corre no passo fixo, logo o replay reproduz o instante da morte.
        //
        // ⚠️ **Sem o `preview_drive`, e pela mesma separação:** o `LifetimeRuntime` não é um
        // componente registado, e quem morre **nunca foi documento** (ele exige o `Spawned`).
        //
        // ⚠️ `tick_count()` é o tique DEPOIS de este quadro correr, e é o que a lei compara com o
        // `born_tick` para não envelhecer um recém-nascido no tique em que ele nasceu.
        let deaths = ph2d_ecs::tick_lifetimes(
            sim.world_mut(),
            (report.ticks as u64).saturating_mul((self.fixed_step.fixed_dt() * 1e6) as u64),
            self.fixed_step.tick_count(),
        );
        Some(FrameClocks {
            report,
            tool_preview_bits,
            anim_signals,
            timer_signals,
            deaths,
        })
    }
}
