//! O reparse do tree-sitter FORA da tecla (`docs/CodeEditor/03_onda2.md` I1): uma thread com o seu
//! [`Parser`](ph2d_code_syntax::luau_parser) corre os [`ReparseJob`] e devolve os [`Reparsed`] por
//! canal, cada um com a chave do documento que o pediu. O quadro só envia e drena.
//!
//! Coalescência: um trabalho novo de um documento substitui o anterior
//! ([`Syntax::job`](ph2d_code_syntax::Syntax::job)); a thread, ao acordar, corre só o ÚLTIMO de cada
//! documento na fila e devolve os outros por correr (`None`), para a contagem do quadro bater.
//!
//! As árvores que a [`Syntax`](ph2d_code_syntax::Syntax) larga ao adoptar também se libertam AQUI
//! ([`GrammarWorker::retire`]): a 7 500 linhas largar uma custa 0,5 ms de p50 na thread que a larga.

use ph2d_code_syntax::{ReparseJob, Reparsed, Retired};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

enum Msg<K> {
    Job(K, ReparseJob),
    Retire(Vec<Retired>),
}

pub struct GrammarWorker<K> {
    jobs: Option<Sender<Msg<K>>>,
    done: Receiver<(K, Option<Reparsed>)>,
    /// Prontos e já tirados do canal por [`Self::wait_all`], à espera do próximo [`Self::drain`].
    ready: Vec<(K, Option<Reparsed>)>,
    /// Enviados e ainda não devolvidos (corridos ou saltados) por um [`Self::drain`].
    in_flight: usize,
    worker: Option<thread::JoinHandle<()>>,
}

impl<K: PartialEq + Send + 'static> GrammarWorker<K> {
    pub fn spawn() -> Self {
        let (jobs, rx) = mpsc::channel::<Msg<K>>();
        let (tx, done) = mpsc::channel();
        let worker = thread::Builder::new()
            .name("ph2d-code-intel-grammar".into())
            .spawn(move || {
                let mut parser = ph2d_code_syntax::luau_parser();
                while let Ok(first) = rx.recv() {
                    let mut batch = Vec::new();
                    for m in std::iter::once(first).chain(rx.try_iter()) {
                        match m {
                            Msg::Job(k, j) => batch.push((k, j)),
                            Msg::Retire(trees) => drop(trees),
                        }
                    }
                    let superseded: Vec<bool> = (0..batch.len())
                        .map(|i| batch[i + 1..].iter().any(|(k, _)| *k == batch[i].0))
                        .collect();
                    for ((key, job), skip) in batch.into_iter().zip(superseded) {
                        let out = (!skip).then(|| job.run(&mut parser));
                        if tx.send((key, out)).is_err() {
                            return;
                        }
                    }
                }
            })
            .expect("thread do worker da gramática");
        Self {
            jobs: Some(jobs),
            done,
            ready: Vec::new(),
            in_flight: 0,
            worker: Some(worker),
        }
    }

    pub fn submit(&mut self, key: K, job: ReparseJob) {
        if let Some(j) = &self.jobs
            && j.send(Msg::Job(key, job)).is_ok()
        {
            self.in_flight += 1;
        }
    }

    /// Liberta `trees` na thread do worker (nunca na do quadro).
    pub fn retire(&self, trees: Vec<Retired>) {
        if trees.is_empty() {
            return;
        }
        if let Some(j) = &self.jobs {
            let _ = j.send(Msg::Retire(trees));
        }
    }

    /// O que já ficou pronto — nunca bloqueia.
    pub fn drain(&mut self) -> Vec<(K, Reparsed)> {
        let mut back = std::mem::take(&mut self.ready);
        back.extend(self.done.try_iter());
        self.in_flight -= back.len();
        back.into_iter()
            .filter_map(|(k, r)| r.map(|r| (k, r)))
            .collect()
    }

    /// Espera que TODO o trabalho enviado fique pronto, sem o entregar: o próximo [`Self::drain`]
    /// leva-o (testes e sondas — o quadro nunca espera).
    pub fn wait_all(&mut self) {
        while self.in_flight > self.ready.len() {
            match self.done.recv() {
                Ok(r) => self.ready.push(r),
                Err(_) => break,
            }
        }
    }

    #[must_use]
    pub fn in_flight(&self) -> usize {
        self.in_flight
    }
}

impl<K> Drop for GrammarWorker<K> {
    fn drop(&mut self) {
        // Fechar o canal de entrada termina o laço da thread.
        self.jobs = None;
        if let Some(w) = self.worker.take() {
            let _ = w.join();
        }
    }
}
