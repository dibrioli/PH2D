//! **O laço do worker ao relógio** (filho de [`super`] por LOC): o `spawn` da thread e o
//! `worker_loop`. Aqui entra o gancho do compositor da óptica (J13b P5).

use super::offthread_ritmo;
use super::{IDLE_SLEEP, STEP_S, SimWorker, otica, placa};
use ph2d_wet_paint::painter::Engine;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Instant;

/// Quanto atraso o worker aceita antes de DESISTIR do tempo perdido.
///
/// ⚠️ **Dois passos, e o número aperta porque o relógio é CONTÍNUO** (ver
/// `worker_loop`): com o tempo das locações contando como dívida, uma pincelada
/// de três segundos acumularia os 0,25 s antigos = **dez passos de catch-up numa
/// rajada**, que é precisamente o que a política do `max_substeps` da física
/// recusa. A 0,05 s o pior catch-up é de dois passos — o bastante para absorver
/// um round trip, curto o bastante para nunca virar rajada.
///
/// ⚠️ Com o passo mais caro que `STEP_S` (4K: ~45 ms) o `acc` vive no tecto e, ao
/// fechar cada passo, fica em `STEP_S`: folga ZERO. Quem serve a imagem não pode
/// perguntar só pela folga (DF-52: 161–175 passos sem imagem) — ver `otica::agenda`.
pub(super) const MAX_BEHIND_S: f64 = 2.0 * STEP_S;

impl SimWorker {
    /// Sobe a thread — ao relógio, ou no ritmo escrito de um gate.
    pub(super) fn spawn(ritmo: Option<&super::RitmoDaAgua>) -> Self {
        let (to_worker, rx_engine) = channel::<Box<Engine>>();
        let (to_ui, back) = channel::<Box<Engine>>();
        let want = Arc::new(AtomicBool::new(false));
        let steps = Arc::new(AtomicU64::new(0));
        let (w, st) = (Arc::clone(&want), Arc::clone(&steps));
        let (lado, ui) = ritmo.map(offthread_ritmo::par).unzip();
        // O compositor da óptica: só ao relógio (o ritmado é dos gates determinísticos).
        let (otica_w, otica) = if lado.is_none() && otica::ligada() {
            let (a, b) = otica::par();
            (Some(a), Some(b))
        } else {
            (None, None)
        };
        std::thread::Builder::new()
            .name("ph2d-wet-sim".into())
            .spawn(move || match lado {
                Some(l) => offthread_ritmo::worker_ritmado(
                    &rx_engine,
                    &to_ui,
                    &w,
                    &st,
                    &l,
                    offthread_ritmo::corre_a_quota,
                ),
                None => worker_loop(&rx_engine, &to_ui, &w, &st, otica_w),
            })
            .expect("a thread da sim de agua");
        Self {
            to_worker,
            back,
            want,
            steps,
            ritmo: ui,
            otica,
            perdido: false,
            pedido: false,
        }
    }
}

/// O laço do worker: recebe o engine, roda estágios no ritmo de 40 Hz, devolve quando pedido.
fn worker_loop(
    rx: &Receiver<Box<Engine>>,
    tx: &Sender<Box<Engine>>,
    want: &AtomicBool,
    steps: &AtomicU64,
    mut otica: Option<otica::DoWorker>,
) {
    // O relógio é do WORKER: ele deve `STEP_S` de tempo simulado por `STEP_S` de tempo de parede, e
    // ficar para trás é câmera lenta (nunca uma rajada de catch-up — a lição do `max_substeps`).
    let mut acc = 0.0f64;
    // ⚠️ **O relógio NÃO reinicia a cada locação, e esta linha vale 2,4× de taxa.**
    //
    // Ele nascia dentro do `while let`, com o racional de que *"o tempo em que o
    // engine esteve na UI não é dívida do worker"* — e o preço disso é aritmético:
    // `acc` é a DÍVIDA que decide se há passo, então descartar tempo real faz o
    // worker sub-passar na **exata fração descartada**. Medido, um traço a 4096²:
    // **31,9 → 38,4 Hz**, 96% do nominal.
    //
    // ⚠️ **E o ganho é 1,2×, não os 2,4× que a aritmética sugeria** — a estimativa
    // era minha, o número é do produto. O `16,5 Hz` do log do Enio **não** é este
    // defeito: naquela poça o limite é TRABALHO (~60 ms/passo), não relógio, e
    // nenhuma correção de agendamento o move. A fixture de três traços mede
    // **12,5 Hz antes e depois desta linha**, que é a assinatura do regime
    // work-limited — a câmera lenta honesta do `max_substeps`.
    let mut last = Instant::now();
    // A instrumentação (`PH2D_FLUID_PROFILE`): os três baldes que particionam a
    // janela do worker + o COMPUTE acumulado do passo em voo. Ver
    // [`crate::wet_diag`] — sem eles o log do produto imprimia `sim 0.00ms x0`,
    // e a pergunta *"a água lenta é trabalho ou agendamento?"* não tinha número.
    let mut step_us: u64 = 0;
    let mut placa = placa::Placa::default();
    while let Ok(mut engine) = rx.recv() {
        placa.recebe(&mut engine);
        crate::wet_diag::note_away_closed();
        loop {
            if want.swap(false, Ordering::AcqRel) {
                // Devolve em fronteira de ESTÁGIO — o pior caso de espera da UI.
                // ⚠️ O relógio do `away` arma ANTES do `send`, que MOVE o engine.
                // ⚠️ E o intervalo é PUBLICADO em vez de guardado aqui: uma
                // retenção mais longa que a janela de diagnóstico tem de ser
                // creditada às janelas que ela ATRAVESSA, não àquela em que
                // fecha (`wet_diag::note_away_open`).
                crate::wet_diag::note_away_open();
                placa.devolve(&mut engine);
                if tx.send(engine).is_err() {
                    return;
                }
                break;
            }
            let now = Instant::now();
            acc = (acc + (now - last).as_secs_f64()).min(MAX_BEHIND_S);
            last = now;
            let in_flight = engine.sim.step_pending();
            if !in_flight && (acc < STEP_S || !engine.sim_should_run()) {
                // Ocioso: a imagem pedida (J13b P5) cabe aqui sem atrasar passo nenhum.
                if let Some(o) = otica.as_mut() {
                    o.serve(&mut engine, acc, steps.load(Ordering::Acquire));
                }
                // Nada a fazer: dorme um pouco em vez de girar num núcleo.
                let t = Instant::now();
                std::thread::sleep(IDLE_SLEEP);
                crate::wet_diag::note_sleep(t.elapsed().as_micros() as u64);
                continue;
            }
            // N10 W2b: o passo que COMEÇA sem água é um no-op do motor (`sim_step_stage` volta antes do
            // `frame += 1`, `marca_passo` não declara): não conta, e a tela não tem o que recompor.
            let sem_agua = !in_flight && !engine.active_grid().has_fluid;
            let t = Instant::now();
            let closed = placa.passo(&mut engine).is_some() || engine.step_stage().is_some();
            let spent = t.elapsed().as_micros() as u64;
            step_us += spent;
            crate::wet_diag::note_busy(spent);
            if closed && sem_agua {
                acc -= STEP_S;
                step_us = 0;
            } else if closed {
                acc -= STEP_S;
                steps.fetch_add(1, Ordering::Release);
                super::acorda_o_laco();
                crate::wet_diag::note_step(step_us as f32 / 1000.0);
                // ⚠️ **O custo sem o TAMANHO não é atribuível** — a mesma linha
                // de log serve *"a poça é grande"* e *"a máquina está
                // disputada"*, que pedem curas opostas (`note_cells`). É
                // `O(linhas)` e sai de uma faixa que o rebuild já materializou.
                crate::wet_diag::note_cells(engine.active_grid().live_span_cells());
                step_us = 0;
                // O passo fechou: a imagem pedida, se a cópia couber na folga ([`otica::DoWorker::serve`]).
                if let Some(o) = otica.as_mut() {
                    o.serve(&mut engine, acc, steps.load(Ordering::Acquire));
                }
            }
        }
    }
}
