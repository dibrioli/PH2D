//! **Quem rende as variações.** Uma thread por pedido, com o ticket do lugar: o pedido novo cancela o velho por dentro da porta
//! do motor, e o velho que chegou a sair nunca passa do [`Ninhada::recebe`]. O `RenderJob` não serve: ele drena até o pedido
//! mais novo, e sete de oito morreriam ali. O pior corpo de 120 s custa 9,4 s por render (W2): é ele que pede o paralelo.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::JoinHandle;

use ph2d_sfx_dsp::{RenderCache, RenderError};
use ph2d_sfx_lab::job::Onda;
use ph2d_sfx_lab::medida::escuta_nivelada;
use ph2d_sfx_recipe::{Recipe, Takes};

/// As variações em volta do som atual.
pub const FILHOS: usize = 8;
/// O lugar do som ouvido na lista Heard.
pub const AVULSO: usize = FILHOS;
const LUGARES: usize = FILHOS + 1;
const NINGUEM: u64 = u64::MAX;
const ESCALA_CHEIA: f32 = 1.0;

#[derive(Clone, Debug)]
pub struct Filho {
    pub receita: Recipe,
    /// O som do arquivo, sem nada por cima.
    pub arquivo: Vec<f32>,
    /// O que se ouve.
    pub nivelada: Vec<f32>,
    pub onda: Onda,
    /// O maior valor absoluto do arquivo.
    pub pico: f32,
}

impl Filho {
    /// O arquivo desta variação passa da escala cheia.
    #[must_use]
    pub fn estoura(&self) -> bool {
        self.pico > ESCALA_CHEIA
    }
}

type Chegada = (usize, u64, Result<Filho, RenderError>);

pub struct Ninhada {
    novos: Arc<[AtomicU64; LUGARES]>,
    tx: Sender<Chegada>,
    rx: Receiver<Chegada>,
    ticket: u64,
    threads: Vec<JoinHandle<()>>,
}

impl Default for Ninhada {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self {
            novos: Arc::new(std::array::from_fn(|_| AtomicU64::new(NINGUEM))),
            tx,
            rx,
            ticket: 0,
            threads: Vec::new(),
        }
    }
}

impl Ninhada {
    /// Volta na hora; o que estava a render neste lugar desiste.
    pub fn pede(&mut self, lugar: usize, receita: Recipe, takes: Arc<Takes>) {
        self.ticket += 1;
        let ticket = self.ticket;
        self.novos[lugar].store(ticket, Ordering::SeqCst);
        self.threads.retain(|t| !t.is_finished());
        let (novos, tx) = (self.novos.clone(), self.tx.clone());
        let feita = std::thread::Builder::new()
            .name("ph2d-sfx-variacao".into())
            .spawn(move || {
                let cancelado = || novos[lugar].load(Ordering::SeqCst) != ticket;
                match rende(receita, &takes, &cancelado) {
                    Err(RenderError::Cancelled) => {}
                    r => {
                        let _ = tx.send((lugar, ticket, r));
                    }
                }
            });
        match feita {
            Ok(t) => self.threads.push(t),
            Err(e) => {
                let _ = self.tx.send((lugar, ticket, Err(RenderError::SemThread)));
                tracing::warn!("sfx lab: variation thread did not start: {e}");
            }
        }
    }

    /// Desiste de tudo o que está a render.
    pub fn esquece(&self) {
        for n in self.novos.iter() {
            n.store(NINGUEM, Ordering::SeqCst);
        }
    }

    /// O que chegou agora. Um render velho morre aqui.
    pub fn recebe(&self) -> Vec<(usize, Result<Filho, RenderError>)> {
        let mut out = Vec::new();
        while let Ok((lugar, ticket, r)) = self.rx.try_recv() {
            if self.novos[lugar].load(Ordering::SeqCst) == ticket {
                out.push((lugar, r));
            }
        }
        out
    }

    #[must_use]
    pub fn em_voo(&self) -> bool {
        self.threads.iter().any(|t| !t.is_finished())
    }
}

impl Drop for Ninhada {
    fn drop(&mut self) {
        self.esquece();
        for t in self.threads.drain(..) {
            let _ = t.join();
        }
    }
}

fn rende(
    receita: Recipe,
    takes: &Takes,
    cancelado: &dyn Fn() -> bool,
) -> Result<Filho, RenderError> {
    // Sem guardar as camadas: oito caches na memória não comprariam nada, e cada filho é outro som.
    let pcm = RenderCache::default().render_cancelavel(
        &receita,
        receita.seed,
        receita.engine,
        takes,
        cancelado,
    )?;
    if cancelado() {
        return Err(RenderError::Cancelled);
    }
    let nivelada = escuta_nivelada(&pcm.samples, pcm.rate);
    if cancelado() {
        return Err(RenderError::Cancelled);
    }
    let pico = pcm.samples.iter().fold(0.0_f32, |a, s| a.max(s.abs()));
    Ok(Filho {
        receita,
        onda: Onda::de(&pcm.samples),
        pico,
        arquivo: pcm.samples,
        nivelada,
    })
}
