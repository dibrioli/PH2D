//! **Fase do quadro: O RELATÓRIO DO PERFILADOR** (`PH2D_FLUID_PROFILE`) — a partição do quadro que sai a
//! cada 120 quadros: total contra encode, o acquire MEDIDO, o dispatch do Painter, as janelas do tique e
//! dos carimbos, as duas rotas do depósito, a água e o worker (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! ⚠️ Corre DENTRO do perfilador (`fase_frame_profile`), que é quem conta os quadros e arma o relógio da
//! janela; esta fase é o corpo do `if n.is_multiple_of(120)`. Os baldes que ela lê são ZERADOS pela
//! leitura (`take`), então ela é o único leitor — a lei do `wash_diag`.

use super::*;

/// **A JANELA de um balde do perfilador: `(média sobre os quadros que trabalharam, pico, n)`** — e
/// a leitura ZERA-O, logo há exactamente um leitor (a lei do `wash_diag`).
///
/// ⚠️ **A JANELA, não a amostra.** A linha `[frame]` sai a cada 120 quadros e lia o valor do quadro
/// SORTEADO — e um traço inteiro cabe entre duas impressões, então `tool-tick` e `stamps` liam
/// **`0,00` por construção** enquanto o artista pintava.
///
/// ⚠️ Ela vive **fora** do corpo do relatório porque não captura nada — e foi o tecto de LOC da
/// função que o tornou visível (a partição nova do `cpu-encode` levou-a a `213` de `200`).
fn janela(
    sum: &'static std::thread::LocalKey<std::cell::Cell<u64>>,
    mx: &'static std::thread::LocalKey<std::cell::Cell<u64>>,
    n: &'static std::thread::LocalKey<std::cell::Cell<u32>>,
) -> (f64, f64, u32) {
    let (s, m, k) = (
        sum.with(std::cell::Cell::take),
        mx.with(std::cell::Cell::take),
        n.with(std::cell::Cell::take),
    );
    let avg = if k > 0 {
        s as f64 / f64::from(k) / 1000.0
    } else {
        0.0
    };
    (avg, m as f64 / 1000.0, k)
}

/// **O que a separação de contactos fez no último quadro** — `(peças, varreduras, vizinhos por
/// peça, separações neste quadro, peças GRANDES)`.
///
/// ⚠️ Os cinco juntos porque **o relógio sozinho não distingue as causas**: `49 ms` é compatível
/// com muitas peças, com muitas varreduras e com uma pilha apertada — três cenas com três curas
/// diferentes (doc 115 §24).
///
/// ⚠️⚠️ **E o quinto entrou porque eu tive de INFERIR a quarta causa** (doc 115 §28): o report de
/// `132`–`156` vizinhos por peça era compatível com uma pilha densa **e** com uma peça grande a
/// inflar a célula de todas, e só a segunda leitura explicava o número. *Uma contagem de vizinhos
/// diz que a grelha está cara; só esta diz PORQUÊ.*
fn numeros_da_separacao() -> (u64, u64, u64, u64, u64, u64) {
    (
        FRAME_PROF_PECAS.with(std::cell::Cell::get),
        FRAME_PROF_VARREDURAS.with(std::cell::Cell::get),
        FRAME_PROF_VIZINHOS.with(std::cell::Cell::get),
        FRAME_PROF_SEPARACOES.with(std::cell::Cell::get),
        FRAME_PROF_GRANDES.with(std::cell::Cell::get),
        FRAME_PROF_BISSECCAO.with(std::cell::Cell::get),
    )
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_frame_profile_report(&mut self) {
        let total = self.frame_ms_ewma;
        let encode = self.frame_cpu_ms_ewma;
        let dispatch_ms = FRAME_PROF_DISPATCH_US.with(|c| c.get()) as f64 / 1000.0;
        // Perf-audit extension (2026-07-07): the phases where a paint slowdown can hide —
        // `tick` = active tool's on_tick (watercolor heartbeat recomposite), `stamp` = the
        // pointer-driven stamps since last frame (Move → apply_watercolor), `hero` = the
        // panel/chrome Vello encode (includes the Paper preview).
        //
        // ⚠️ **Só o STAMP fica fora da janela de encode, e este comentário
        // dizia "stamp + tick".** O flush coalescido roda na linha ~698,
        // ANTES do `cpu_start` (712); o `on_tick` roda na ~1198, DEPOIS —
        // ou seja DENTRO do encode. O log da partição de três provou
        // isto no primeiro smoke: nas janelas sem carimbo o
        // `fora-do-encode` mede **0,00-1,05 ms** enquanto o `tool-tick`
        // mede **3,01-3,93**, o que seria impossível se o tick estivesse
        // fora. Um comentário que contradiz o código shipado é pior que
        // comentário nenhum.
        let hero_ms = FRAME_PROF_HERO_US.with(|c| c.get()) as f64 / 1000.0;
        // ⚠️ A JANELA, não a amostra: `média sobre os frames que trabalharam / pico / n`.
        // Ler um frame sorteado fazia um traço inteiro caber entre duas impressões e as
        // duas fases intermitentes lerem 0,00 enquanto o artista pintava.
        // ⭐⭐⭐ **A PARTIÇÃO DO `cpu-encode`** (report do dono, 2026-09-18): ele lia `22,85 ms` com a
        // placa em `1,58` e o `acquire` em `0,03` — *o tecto é a CPU e a linha não dizia de quê*.
        // Estas duas fases são as que escalam com a POPULAÇÃO da cena, e as curas delas moram em
        // módulos diferentes.
        let (motion_avg, motion_max, motion_n) = janela(
            &FRAME_PROF_MOTION_SUM_US,
            &FRAME_PROF_MOTION_MAX_US,
            &FRAME_PROF_MOTION_N,
        );
        // ⚠️⚠️ **`tiques_*` e não `sim_*`: o bloco da ÁGUA já usa `sim_avg`/`sim_max`/`sim_n`**, e a
        // 1.ª redacção destes três ficou SOMBREADA por ele — a linha nova imprimiria os números da
        // água com a etiqueta da simulação. *Um instrumento que mente é pior que instrumento
        // nenhum*, e quem o apanhou foi o aviso de variável não usada.
        let (pecas, varreduras, vizinhos, separacoes, grandes, bisseccao) = numeros_da_separacao();
        let (tiques_avg, tiques_max, tiques_n) = janela(
            &FRAME_PROF_SIM_SUM_US,
            &FRAME_PROF_SIM_MAX_US,
            &FRAME_PROF_SIM_N,
        );
        let (tick_avg, tick_max, tick_n) = janela(
            &FRAME_PROF_TICK_SUM_US,
            &FRAME_PROF_TICK_MAX_US,
            &FRAME_PROF_TICK_N,
        );
        let (stamp_avg, stamp_max, stamp_n) = janela(
            &FRAME_PROF_STAMP_SUM_US,
            &FRAME_PROF_STAMP_MAX_US,
            &FRAME_PROF_STAMP_N,
        );
        // ⚠️ **Qual rota o depósito tomou.** O report de 2026-08-03 (*"não houve melhora
        // real"*) admitia duas leituras opostas — o ramo em banda não dispara neste pincel, ou
        // dispara e o tempo está noutro lugar — e nenhum número no log as separava. `take` ZERA,
        // então este é o ÚNICO leitor (a lei do `wash_diag`).
        let dep = ph2d_tool_painter::band_diag::take();
        let (band_par, band_ser, band_dabs) = (dep.banded, dep.serial, dep.dabs);
        // ⚠️ Sem este balde, *"não melhorou"* admite duas leituras opostas — a rota do device não é
        // tomada, ou é tomada e o tempo está noutro lugar — e as curas são opostas (doc 33 §S3).
        let band_dev = dep.device;
        // ⚠️ **VISITAS, não o raio.** O custo de um lote é quadrático no raio do pincel,
        // então um log com `dabs` e sem o raio convida a uma aritmética infundada — foi o
        // que eu quase publiquei lendo o smoke de 2026-08-03. A soma das pegadas É o
        // trabalho e não assume nada.
        #[allow(clippy::cast_precision_loss)]
        let band_mvis = dep.visits as f64 / 1.0e6;
        #[allow(clippy::cast_precision_loss)]
        let dev_mvis = dep.dev_visits as f64 / 1.0e6;
        // ⚠️ **A razão tem de ter as DUAS metades do MESMO evento.** Até 2026-08-04 o
        // numerador vinha das quatro fases do RE-STAMP e o denominador das visitas do
        // DEPÓSITO — populações diferentes —, então numa sessão de mão livre ele imprimia
        // `0.0 ns/visita` ao lado de 99 M visitas, e o zero lia-se como *"o carimbo é de
        // graça"*. Cada rota agora divide o próprio tempo pelo próprio trabalho, e é essa
        // razão que responde se o dispositivo vale a pena NESTE pincel.
        #[allow(clippy::cast_precision_loss)]
        let ns_per = |us: u64, visits: u64| {
            if visits > 0 {
                us as f64 * 1000.0 / visits as f64
            } else {
                0.0
            }
        };
        let dev_ns = ns_per(dep.dev_us, dep.dev_visits);
        let band_ns = ns_per(dep.cpu_us, dep.visits);
        let dev_dabs = dep.dev_dabs;
        // As quatro fases, POR ENTREGA — cada uma tem cura diferente.
        #[allow(clippy::cast_precision_loss)]
        let per_delivery = |us: u64| {
            if dep.deliveries > 0 {
                us as f64 / f64::from(dep.deliveries) / 1000.0
            } else {
                0.0
            }
        };
        let (rs_restore, rs_relief) = (per_delivery(dep.restore_us), per_delivery(dep.relief_us));
        let (rs_save, rs_stamp) = (per_delivery(dep.save_us), per_delivery(dep.stamp_us));
        let stamp_ev = FRAME_PROF_STAMP_EV.with(std::cell::Cell::take);
        // O custo POR ENTREGA — a média acima dividida pelo que ela soma.
        let stamp_per = if stamp_ev > 0 {
            stamp_avg * f64::from(stamp_n) / f64::from(stamp_ev)
        } else {
            0.0
        };
        // E o SPLIT do tick da água: um pico no `tool-tick` não diz se foi um passo de
        // sim caro ou um composite sobre um casco grande, e as curas são outras.
        let ((sim_sum, sim_max, sim_n), (comp_sum, comp_max, comp_n), (wait_sum, wait_max, wait_n)) =
            ph2d_tool_painter::wet_diag::take_window();
        // A partição do WORKER (`wet_diag`): com a sim fora da thread do frame, a linha
        // `sim` acima imprimia `0.00ms x0` — ninguém media o passo, porque quem o dá é o
        // worker. Estes três baldes dizem se a água lenta é TRABALHO (busy alto: só a GPU
        // move) ou AGENDAMENTO (away/sleep alto), que têm curas opostas.
        let (busy, away, sleep) = ph2d_tool_painter::wet_diag::take_worker();
        // ⚠️ **DRENADO SEMPRE, junto com os irmãos** — um balde que só é
        // lido sob condição acumula entre janelas e passa a reportar a
        // MÉDIA de uma janela que ninguém escolheu.
        let cells = ph2d_tool_painter::wet_diag::take_cells();
        // A janela REAL desde o último dreno — nunca `frame_medio × 120`.
        let span = FRAME_PROF_SINCE.with(|c| {
            let now = std::time::Instant::now();
            c.borrow_mut()
                .replace(now)
                .map_or(0.0, |t0| (now - t0).as_secs_f64() * 1000.0)
        });
        let pct = |x: f64| if span > 0.0 { 100.0 * x / span } else { 0.0 };
        // O divisor do dispatch: a AREA que o dreno publicou, por quadro
        // que de fato drenou (a agua corre a ~38 Hz contra 60 de display,
        // entao dividir pelos 120 do laco diluiria o retangulo).
        let prev_n = FRAME_PROF_PREVIEW_N.with(std::cell::Cell::take);
        let prev_px = FRAME_PROF_PREVIEW_PX.with(std::cell::Cell::take);
        let prev_mpx = if prev_n > 0 {
            prev_px as f64 / f64::from(prev_n) / 1.0e6
        } else {
            0.0
        };
        let per = |sum: f64, n: u64| if n > 0 { sum / n as f64 } else { 0.0 };
        // ⚠️ **A PARTIÇÃO DO QUADRO, com o acquire MEDIDO.** O residuo
        // `total - encode` era publicado como "present/acquire-stall" e
        // se lia como espera de GPU, mas o encode só começa em
        // `cpu_start`: tudo antes dele (o `tool-tick`, o flush de
        // carimbo, o pump de eventos) caía no residuo com o nome
        // errado. Agora `acquire` é medido no sítio e `fora-do-encode`
        // é o que sobra — CPU que o quadro paga e a linha escondia.
        let acq_n = FRAME_PROF_ACQUIRE_N.with(std::cell::Cell::take);
        let acq_us = FRAME_PROF_ACQUIRE_US.with(std::cell::Cell::take);
        let acq_ms = if acq_n > 0 {
            acq_us as f64 / f64::from(acq_n) / 1000.0
        } else {
            0.0
        };
        let outside_ms = (f64::from(total) - f64::from(encode) - acq_ms).max(0.0);
        eprintln!(
            "[frame] total={total:.2}ms (~{:.0} fps) | cpu-encode(raw)={encode:.2}ms \
                     | acquire(medido)={acq_ms:.2}ms | fora-do-encode={outside_ms:.2}ms \
                     | painter-dispatch(cpu)={dispatch_ms:.2}ms \
                     ({prev_mpx:.2} M px publicados em {prev_n} quadros) | hero-paint={hero_ms:.2}ms\n\
                     [frame]   MOTION (cozer + separar): media {motion_avg:.2}ms pico {motion_max:.2}ms em {motion_n}/120 · {pecas} pecas x {varreduras} varreduras x {vizinhos} vizinhos, {separacoes} separacao(oes)/quadro, {grandes} grande(s), uma-camada-por-ordem={bisseccao} \
                     | SIMULACAO (os tiques): media {tiques_avg:.2}ms pico {tiques_max:.2}ms em {tiques_n}/120\n\
                     [frame]   tool-tick: media {tick_avg:.2}ms pico {tick_max:.2}ms em {tick_n}/120 frames \
                     | stamps: media {stamp_avg:.2}ms pico {stamp_max:.2}ms em {stamp_n}/120 \
                     ({stamp_ev} entregas, {stamp_per:.2}ms cada)\n\
                     [frame]   deposito DEVICE: {band_dev} lotes, {dev_dabs} dabs, \
                     {dev_mvis:.2} M visitas ({dev_ns:.1} ns/visita)\n\
                     [frame]   deposito CPU: {band_par} em BANDA + {band_ser} serial(is), \
                     {band_dabs} dabs, {band_mvis:.2} M visitas ({band_ns:.1} ns/visita)\n\
                     [frame]   re-stamp por entrega: restore {rs_restore:.2}ms | relevo \
                     {rs_relief:.2}ms | save {rs_save:.2}ms | CARIMBO {rs_stamp:.2}ms \
                     (x{} entregas)\n\
                     [frame]   agua: sim media {:.2}ms pico {sim_max:.2}ms x{sim_n} \
                     | composite media {:.2}ms pico {comp_max:.2}ms x{comp_n} \
                     | ESPERA media {:.2}ms pico {wait_max:.2}ms x{wait_n} (total {:.0}ms)\n\
                     [frame]   worker: busy {:.0}% away {:.0}% sleep {:.0}% \
                     | TAXA DA AGUA {:.1} Hz ({sim_n} passos em {:.1}s)\n\
                     [frame]   poca: {:.2} M celulas | {:.1} ns/celula \
                     (o custo por celula e CONSTANTE quando a agua fica lenta por TRABALHO, \
                     e sobe quando fica por CONTENCAO)",
            1000.0 / f64::from(total).max(0.001),
            dep.deliveries,
            per(sim_sum, sim_n),
            per(comp_sum, comp_n),
            per(wait_sum, wait_n),
            wait_sum,
            pct(busy),
            pct(away),
            pct(sleep),
            if span > 0.0 {
                1000.0 * sim_n as f64 / span
            } else {
                0.0
            },
            span / 1000.0,
            cells as f64 / 1e6,
            if cells > 0 {
                1e6 * per(sim_sum, sim_n) / cells as f64
            } else {
                0.0
            },
        );
    }
}
