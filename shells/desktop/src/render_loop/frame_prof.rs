//! **O perfilador de fases do quadro** (`PH2D_FLUID_PROFILE` / `PH2D_PAINT_PERF`) — os contadores da janela, as
//! duas notas que outras fases chamam e o relógio do quadro inteiro, movidos verbatim do `render_loop/mod.rs`
//! (`line/render-bodies`, 2026-09-13). A única troca é a visibilidade: o que era privado do `render_loop` é
//! `pub(super)` aqui, e o `mod.rs` importa-o de volta (`use frame_prof::*`), logo as fases leem os mesmos nomes.

thread_local! {
    /// Frame-phase profiler (`PH2D_FLUID_PROFILE` **ou** `PH2D_PAINT_PERF`):
    /// `-1` = unread, else cached on/off. Splits the frame into CPU-encode (raw)
    /// vs the present/acquire stall, plus the painter bridge dispatch (CPU preview)
    /// — to pin a slowdown the `[fluid]` profiler proves is OUTSIDE the fluid drive.
    ///
    /// ⚠️ **Por que o `PH2D_PAINT_PERF` liga este bloco também** (2026-08-03): o
    /// carimbo roda no flush coalescido, na linha ~698, **ANTES** do `cpu_start`
    /// — ou seja fora da janela de encode e fora do `painter-dispatch`. O
    /// `[paint-perf]` é, por construção, **CEGO ao carimbo**: ele reporta
    /// `dispatch p50=0.0` tanto num traço de graça quanto num que custa 300 ms.
    /// A linha `stamps:` (e o `deposito:` ao lado dela) vive só aqui, então um
    /// smoke rodado com o flag que NOMEIA performance de pintura media tudo
    /// menos a pintura e voltava tranquilizando. Um instrumento silencioso é
    /// pior que um ausente.
    pub(super) static FRAME_PROF_ON: std::cell::Cell<i8> = const { std::cell::Cell::new(-1) };
    pub(super) static FRAME_PROF_N: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    /// ⚠️ **O RELÓGIO da janela de diagnóstico, MEDIDO.** Ele existe porque a
    /// versão anterior a ASSUMIA (`span = frame_medio × 120`), e os contadores
    /// do worker (`wet_diag`) acumulam em tempo REAL enquanto este bloco só
    /// conta os frames em que ele roda — as duas janelas divergem, e a
    /// divergência aparecia como uma partição impossível: o log do Enio de
    /// 2026-07-31 trouxe `busy 69% away 31% sleep 909%` (soma 1009%) e uma
    /// `TAXA DA AGUA 392,8 Hz` contra os **40 Hz nominais da SPEC**, ou seja um
    /// solver dez vezes fora do ritmo — que **não existia**. Três baldes que
    /// dizem partição TÊM de dividir uma janela medida, senão o instrumento
    /// manda a próxima pessoa caçar uma sim desgovernada que não está lá.
    pub(super) static FRAME_PROF_SINCE: std::cell::RefCell<Option<std::time::Instant>> =
        const { std::cell::RefCell::new(None) };
    pub(super) static FRAME_PROF_DISPATCH_US: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    /// Active tool's `on_tick` µs (the watercolor heartbeat: soak pour + live recomposite) —
    /// the perf-audit phase the original split missed (2026-07-07, "grave FPS drop" hunt).
    /// Acumulado sobre a JANELA, nunca lido de um frame só (ver abaixo).
    /// ⚠️ **A JANELA, não uma amostra.** A linha `[frame]` sai a cada 120 frames e lia o valor do
    /// frame SORTEADO — e um traço inteiro cabe entre duas impressões, então `tool-tick` e `stamps`
    /// liam **0,00 por construção** enquanto o artista pintava (smoke do Enio, 2026-07-29: quatro
    /// amostras seguidas zeradas num app onde a água estava viva). É a mesma doença que o split de
    /// fases teve com a mediana (§4.8.2): *um custo intermitente é invisível num redutor que só
    /// olha um instante.* Estes três acumulam sobre a janela e zeram a cada impressão.
    pub(super) static FRAME_PROF_TICK_SUM_US: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    pub(super) static FRAME_PROF_TICK_MAX_US: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    pub(super) static FRAME_PROF_TICK_N: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    /// ⭐⭐⭐ **A FASE DO MOTION e a da SIMULAÇÃO** — as duas que escalam com a POPULAÇÃO da cena
    /// (report do dono, 2026-09-18: *«1000 = 40 fps»*, com `cpu-encode = 22,85 ms` e a placa em
    /// `1,58`).
    ///
    /// ⚠️ **Sem elas a linha `[frame]` diz que o tecto é a CPU e não diz DE QUÊ** — e as duas curas
    /// possíveis (o cozimento do grafo, ou os tiques da simulação sobre mil entidades) moram em
    /// módulos diferentes. *Um total sem partição manda procurar no sítio errado.*
    ///
    /// Acumulam sobre a janela, como o `tick` e o `stamp`.
    pub(super) static FRAME_PROF_MOTION_SUM_US: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    pub(super) static FRAME_PROF_MOTION_MAX_US: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    pub(super) static FRAME_PROF_MOTION_N: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    /// **Quantas varreduras a separação de facto correu no último quadro** — o PREÇO do knob, em
    /// número, ao lado do relógio que ele custa.
    pub(super) static FRAME_PROF_VARREDURAS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    /// As PEÇAS e os CANDIDATOS da última separação — o par que distingue *«muitas peças»* de
    /// *«uma pilha apertada»*, que o relógio sozinho não separa.
    pub(super) static FRAME_PROF_PECAS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    pub(super) static FRAME_PROF_VIZINHOS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    /// Quantas peças a grelha pôs na camada dos GRANDES — a causa, e não só o sintoma.
    pub(super) static FRAME_PROF_GRANDES: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    /// Quantas SEPARAÇÕES correram neste quadro — um quadro atrasado recupera vários tiques e só o
    /// último é desenhado, logo isto tem de ler `1`.
    pub(super) static FRAME_PROF_SEPARACOES: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    /// O valor do contador CUMULATIVO no quadro anterior — o divisor da diferença acima.
    ///
    /// ⚠️ Ele mora aqui e **não num campo da `App`**: a catraca de campos daquela struct só desce, e
    /// um número que é do PERFILADOR não é estado do editor.
    pub(super) static FRAME_PROF_SEPARACOES_ANTES: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    pub(super) static FRAME_PROF_SIM_SUM_US: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    pub(super) static FRAME_PROF_SIM_MAX_US: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    pub(super) static FRAME_PROF_SIM_N: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    /// Idem para o carimbo de dabs (`stamps`), que é o outro inquilino intermitente do frame.
    pub(super) static FRAME_PROF_STAMP_SUM_US: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    pub(super) static FRAME_PROF_STAMP_MAX_US: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    pub(super) static FRAME_PROF_STAMP_N: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    /// Quantas ENTREGAS de ponteiro compõem a soma acima — o divisor sem o qual
    /// `stamps: media 105,82ms` não distingue *um re-stamp de forma inteira* de
    /// *cinquenta eventos incrementais*, que pedem curas opostas.
    pub(super) static FRAME_PROF_STAMP_EV: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    /// **O DIVISOR DO `painter-dispatch`** — quantos PIXELS o dreno do preview
    /// publicou na janela, e em quantos quadros.
    ///
    /// ⚠️ A mesma doença que o `FRAME_PROF_STAMP_EV` curou no carimbo, um
    /// sistema adiante: `painter-dispatch=11,80ms` **sem carimbo nenhum** não
    /// distingue *um retângulo grande uma vez* de *um retângulo pequeno sempre*,
    /// e o custo é dominado por gather + premultiply + upload da área
    /// publicada. Medido headless (doc 28 §5.53): com a água correndo o dreno
    /// publica **8,26 M px por quadro** numa tela de 16,8 M — metade dela — para
    /// **2,07 M células** de água viva, ou seja o retângulo pede **3,99×**.
    pub(super) static FRAME_PROF_PREVIEW_PX: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    pub(super) static FRAME_PROF_PREVIEW_N: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    /// A espera MEDIDA do `acquire_frame` (ver [`note_acquire_wait`]).
    pub(super) static FRAME_PROF_ACQUIRE_US: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    pub(super) static FRAME_PROF_ACQUIRE_N: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    /// `paint_hero_screen` µs (panel/chrome Vello encode — includes the Paper preview).
    pub(super) static FRAME_PROF_HERO_US: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// Quanto o `acquire_frame` de fato BLOQUEOU neste quadro.
///
/// ⚠️ Sem isto a linha `[frame]` publicava `present/acquire-stall` como
/// `total - encode`, e o encode só começa em `cpu_start` — **depois** do
/// `tool-tick` e do flush de carimbo. O residuo se lia como *espera de GPU*
/// enquanto continha trabalho de CPU: medido, `tick 3,31` de um "stall" de
/// 7,91. Um numero derivado por subtração absorve tudo que ninguem mediu.
pub(crate) fn note_acquire_wait(d: std::time::Duration) {
    if frame_prof_on() {
        FRAME_PROF_ACQUIRE_US.with(|c| c.set(c.get() + d.as_micros() as u64));
        FRAME_PROF_ACQUIRE_N.with(|c| c.set(c.get() + 1));
    }
}

/// O dreno do preview publicou `px` pixels neste quadro — o divisor do
/// `painter-dispatch`. Chamado de [`painter_bridge`], no sítio onde a bbox é
/// resolvida; no-op sem o `PH2D_FLUID_PROFILE`.
pub(crate) fn note_preview_px(px: u64) {
    if frame_prof_on() {
        FRAME_PROF_PREVIEW_PX.with(|c| c.set(c.get() + px));
        FRAME_PROF_PREVIEW_N.with(|c| c.set(c.get() + 1));
    }
}

/// Quiet frames after the last resize event before the present mode saved by the fluid-drag override is
/// restored (~0.5s at 60Hz) — long enough that a paused-then-resumed drag doesn't thrash reconfigures.
pub(super) const RESIZE_SETTLE_FRAMES: u32 = 30;

pub(super) fn frame_prof_on() -> bool {
    FRAME_PROF_ON.with(|c| {
        if c.get() < 0 {
            // Os DOIS flags acendem esta partição — ver o porquê no doc do `FRAME_PROF_ON`:
            // a linha `stamps:`/`deposito:` mora só aqui, e quem mede pintura pede o
            // `PH2D_PAINT_PERF`.
            let on = ["PH2D_FLUID_PROFILE", "PH2D_PAINT_PERF"]
                .iter()
                .any(|k| std::env::var(k).is_ok_and(|v| v != "0"));
            c.set(i8::from(on));
        }
        c.get() > 0
    })
}

/// `PH2D_PAINT_PERF=1` diagnostic (2026-07-24, the mask-path FPS report): the WHOLE-frame wall clock,
/// handed to [`paint_perf::end_frame`] on drop so it pairs with the per-dispatch info recorded by
/// `painter_bridge`. The aggregator prints ONE summary line per window (not one per frame — that
/// drowned the terminal), and the frame-vs-dispatch split says whether a slow frame's cost is IN the
/// painter preview production or OUTSIDE it (panel, sim_extract, present).
pub(super) struct PaintFrameTimer(pub(super) Option<std::time::Instant>);
impl Drop for PaintFrameTimer {
    fn drop(&mut self) {
        if let Some(t0) = self.0 {
            ph2d_app_painter::paint_perf::end_frame(t0.elapsed().as_secs_f64() as f32 * 1e3);
        }
    }
}
