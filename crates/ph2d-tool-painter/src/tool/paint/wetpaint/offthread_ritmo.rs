//! **O worker RITMADO** (W7, filho de [`super`] por LOC): o worker da sim de um gate, que anda num
//! ritmo escrito ([`RitmoDaAgua`]) em vez do relógio — a resposta não pode depender de quanto a
//! máquina demorou. O worker ao relógio e o slot vivem em [`super::offthread`].

use super::fita::RitmoDaAgua;
use ph2d_wet_paint::painter::Engine;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender};

const MORREU: &str = "o worker ritmado morreu (pânico na thread da sim) — o pedido falha em vez de \
                      pendurar";

/// O lado da thread do quadro: o ciclo dos pedidos sem espera, e as entregas cuja quota o worker já
/// correu (`feitas`) — ler o worker antes disso seria ler o relógio outra vez.
pub(super) struct Ritmado {
    nao_volta: Vec<bool>,
    pedidos: usize,
    entregues: u64,
    feitas: Arc<AtomicU64>,
    /// Posto pela [`Guarda`] quando a thread do worker sai — também a desenrolar um pânico.
    morto: Arc<AtomicBool>,
}

/// O lado da thread do worker: as quotas, e os dois sinais que a [`Guarda`] escreve.
pub(super) struct LadoDoWorker {
    estagios: Vec<u32>,
    feitas: Arc<AtomicU64>,
    morto: Arc<AtomicBool>,
}

/// Os dois lados de um worker ritmado.
pub(super) fn par(r: &RitmoDaAgua) -> (LadoDoWorker, Ritmado) {
    let feitas = Arc::new(AtomicU64::new(0));
    let morto = Arc::new(AtomicBool::new(false));
    let worker = LadoDoWorker {
        estagios: r.estagios.clone(),
        feitas: Arc::clone(&feitas),
        morto: Arc::clone(&morto),
    };
    let ui = Ritmado {
        nao_volta: r.nao_volta.clone(),
        pedidos: 0,
        entregues: 0,
        feitas,
        morto,
    };
    (worker, ui)
}

impl Ritmado {
    /// O motor foi entregue ao worker: mais uma quota a esperar.
    pub(super) fn entregou(&mut self) {
        self.entregues += 1;
    }

    /// O próximo pedido sem espera encontra o motor?
    fn volta(&mut self) -> bool {
        let n = self.nao_volta.len().max(1);
        let nao = self
            .nao_volta
            .get(self.pedidos % n)
            .copied()
            .unwrap_or(false);
        self.pedidos += 1;
        !nao
    }

    /// Espera o worker correr a quota da última entrega; PANICA nomeado se ele morreu.
    pub(super) fn espera_a_quota(&self) {
        while self.feitas.load(Ordering::Acquire) < self.entregues
            && !self.morto.load(Ordering::Acquire)
        {
            std::thread::yield_now();
        }
        assert!(!self.morto.load(Ordering::Acquire), "{MORREU}");
    }

    /// O pedido sem espera: depois da quota, o ciclo `nao_volta` decide; quando volta, espera-o.
    pub(super) fn pede(
        &mut self,
        want: &AtomicBool,
        back: &Receiver<Box<Engine>>,
    ) -> Option<Box<Engine>> {
        self.espera_a_quota();
        want.store(true, Ordering::Release);
        if !self.volta() {
            return None;
        }
        // O `to_ui` só cai com a thread: um `Err` aqui é a morte dela.
        Some(back.recv().unwrap_or_else(|_| panic!("{MORREU}")))
    }
}

/// Marca a morte quando a thread sai, a desenrolar ou não: sem ela, um estágio que panica deixava
/// `feitas` parado e [`Ritmado::espera_a_quota`] girava para sempre.
struct Guarda<'a>(&'a LadoDoWorker);

impl Drop for Guarda<'_> {
    fn drop(&mut self) {
        self.0.morto.store(true, Ordering::Release);
        self.0.feitas.fetch_add(1, Ordering::Release);
    }
}

/// A quota de uma entrega, com a mesma regra de «nada a fazer» do relógio.
pub(super) fn corre_a_quota(engine: &mut Engine, quota: u32, steps: &AtomicU64) {
    for _ in 0..quota {
        if !engine.sim.step_pending() && !engine.sim_should_run() {
            break;
        }
        if engine.step_stage().is_some() {
            steps.fetch_add(1, Ordering::Release);
        }
    }
}

/// O worker de um gate: cada entrega corre a sua quota (`corre`) e espera o pedido. O `Drop` do
/// `SimWorker` arma o `want` e fecha os canais, como no relógio.
pub(super) fn worker_ritmado(
    rx: &Receiver<Box<Engine>>,
    tx: &Sender<Box<Engine>>,
    want: &AtomicBool,
    steps: &AtomicU64,
    lado: &LadoDoWorker,
    corre: impl Fn(&mut Engine, u32, &AtomicU64),
) {
    let _guarda = Guarda(lado);
    let mut entrega = 0usize;
    while let Ok(mut engine) = rx.recv() {
        let quota = lado
            .estagios
            .get(entrega % lado.estagios.len().max(1))
            .copied()
            .unwrap_or(0);
        entrega += 1;
        corre(&mut engine, quota, steps);
        lado.feitas.fetch_add(1, Ordering::Release);
        while !want.swap(false, Ordering::AcqRel) {
            std::thread::yield_now();
        }
        if tx.send(engine).is_err() {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::sync::mpsc::channel;
    use std::time::Duration;

    /// **Um estágio que panica no worker ritmado FALHA o pedido — não o pendura.** Sem a guarda,
    /// `feitas` nunca subia e o pedido girava para sempre: o gate pendurava em vez de reprovar.
    #[test]
    fn um_worker_ritmado_que_panica_falha_o_pedido_em_vez_de_pendurar() {
        let (lado, mut ui) = par(&RitmoDaAgua {
            estagios: vec![4],
            nao_volta: vec![false],
        });
        let (to_worker, rx) = channel::<Box<Engine>>();
        let (to_ui, back) = channel::<Box<Engine>>();
        let want = Arc::new(AtomicBool::new(false));
        let w = Arc::clone(&want);
        let worker = std::thread::spawn(move || {
            let steps = AtomicU64::new(0);
            worker_ritmado(&rx, &to_ui, &w, &steps, &lado, |_, _, _| {
                panic!("estágio injectado a panicar")
            });
        });
        to_worker
            .send(Box::new(Engine::new(64, 64)))
            .expect("o worker aceita o motor");
        ui.entregou();
        let (tx, veredito) = channel();
        std::thread::spawn(move || {
            let r = catch_unwind(AssertUnwindSafe(|| ui.pede(&want, &back).is_some()));
            let msg = r.err().map(|e| {
                e.downcast_ref::<String>()
                    .cloned()
                    .or_else(|| e.downcast_ref::<&str>().map(|s| (*s).to_owned()))
                    .unwrap_or_default()
            });
            let _ = tx.send(msg);
        });
        let msg = veredito
            .recv_timeout(Duration::from_secs(5))
            .expect("o pedido PENDUROU 5 s sobre um worker morto — a guarda não marcou a morte");
        let msg = msg.expect("o pedido devolveu como se o worker estivesse vivo");
        assert!(
            msg.contains("o worker ritmado morreu"),
            "pânico sem nome: {msg}"
        );
        assert!(worker.join().is_err(), "o worker devia ter panicado");
        drop(to_worker);
    }
}
