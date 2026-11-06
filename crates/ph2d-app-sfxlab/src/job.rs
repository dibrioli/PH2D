//! **`RenderJob`: quando rende.** Uma thread por aba, dona do `RenderCache`. O pedido novo cancela o velho por dentro da
//! porta do motor, e o velho que chegou a sair nunca passa do [`RenderJob::poll`].

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::JoinHandle;

use ph2d_sfx_dsp::{Pcm, RenderCache, RenderError, limitar_escuta};
use ph2d_sfx_recipe::{LayerId, Recipe, Takes};

/// As colunas guardadas de uma onda; a tela só reamostra. O recurso é a RAM: 16 KB por camada. A pista mede 775 px numa
/// janela de 1930 (foto de 06/11), logo sobra resolução mesmo com a coluna fechada.
pub const COLUNAS_DA_ONDA: usize = 2048;
const NINGUEM: u64 = u64::MAX;

/// Mínimo e máximo por coluna, sem ganho.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Onda {
    pub frames: usize,
    pub colunas: Vec<[f32; 2]>,
}

impl Onda {
    #[must_use]
    pub fn de(amostras: &[f32]) -> Self {
        let n = COLUNAS_DA_ONDA.min(amostras.len());
        let colunas = (0..n)
            .map(|i| {
                let (de, ate) = (i * amostras.len() / n, (i + 1) * amostras.len() / n);
                amostras[de..ate]
                    .iter()
                    .fold([f32::INFINITY, f32::NEG_INFINITY], |[a, b], s| {
                        [a.min(*s), b.max(*s)]
                    })
            })
            .collect();
        Self {
            frames: amostras.len(),
            colunas,
        }
    }
}

pub struct Pedido {
    pub receita: Recipe,
    /// Só a escuta ouve isto: o documento e as ondas não.
    pub solo: Option<LayerId>,
    pub takes: Arc<Takes>,
}

#[derive(Clone, Debug)]
pub struct Feito {
    pub ticket: u64,
    /// O som do arquivo, sem limitador.
    pub pcm: Pcm,
    /// O som como se ouve: com o solo e o limitador.
    pub escuta: Vec<f32>,
    pub mix: Onda,
    pub camadas: BTreeMap<LayerId, Onda>,
}

pub struct RenderJob {
    tx: Option<Sender<(u64, Pedido)>>,
    rx: Receiver<(u64, Result<Feito, RenderError>)>,
    novo: Arc<AtomicU64>,
    thread: Option<JoinHandle<()>>,
}

impl Default for RenderJob {
    fn default() -> Self {
        let (tx, entrada) = channel();
        let (saida, rx) = channel();
        let novo = Arc::new(AtomicU64::new(NINGUEM));
        let dela = novo.clone();
        let thread = std::thread::Builder::new()
            .name("ph2d-sfx-render".into())
            .spawn(move || corre(&entrada, &saida, &dela))
            .ok();
        Self {
            tx: Some(tx),
            rx,
            novo,
            thread,
        }
    }
}

impl RenderJob {
    /// Volta na hora. `ticket` tem de crescer.
    pub fn submit(&self, pedido: Pedido, ticket: u64) {
        self.novo.store(ticket, Ordering::SeqCst);
        if let Some(tx) = &self.tx {
            let _ = tx.send((ticket, pedido));
        }
    }

    /// A resposta do pedido mais novo, se já chegou. Tudo o que for mais velho morre aqui.
    pub fn poll(&self) -> Option<Result<Feito, RenderError>> {
        let novo = self.novo.load(Ordering::SeqCst);
        let mut pronto = None;
        while let Ok((ticket, r)) = self.rx.try_recv() {
            if ticket == novo {
                pronto = Some(r);
            }
        }
        pronto
    }
}

impl Drop for RenderJob {
    fn drop(&mut self) {
        self.novo.store(NINGUEM, Ordering::SeqCst);
        self.tx = None;
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

fn corre(
    entrada: &Receiver<(u64, Pedido)>,
    saida: &Sender<(u64, Result<Feito, RenderError>)>,
    novo: &AtomicU64,
) {
    let mut cache = RenderCache::default();
    while let Ok(mut atual) = entrada.recv() {
        while let Ok(mais) = entrada.try_recv() {
            atual = mais;
        }
        let (ticket, pedido) = atual;
        let cancelado = || novo.load(Ordering::SeqCst) != ticket;
        match rende(&mut cache, ticket, &pedido, &cancelado) {
            Err(RenderError::Cancelled) => {}
            r => {
                if saida.send((ticket, r)).is_err() {
                    return;
                }
            }
        }
    }
}

fn rende(
    cache: &mut RenderCache,
    ticket: u64,
    p: &Pedido,
    cancelado: &dyn Fn() -> bool,
) -> Result<Feito, RenderError> {
    let r = &p.receita;
    let pcm = cache.render_cancelavel(r, r.seed, r.engine, &p.takes, cancelado)?;
    let ouvida = match p.solo {
        Some(id) => cache.render_cancelavel(&r.solo(id), r.seed, r.engine, &p.takes, cancelado)?,
        None => pcm.clone(),
    };
    let escuta = limitar_escuta(&ouvida.samples, ouvida.rate);
    let camadas = r
        .layers
        .iter()
        .filter_map(|l| cache.camada(l.id).map(|b| (l.id, Onda::de(b))))
        .collect();
    Ok(Feito {
        ticket,
        mix: Onda::de(&pcm.samples),
        camadas,
        escuta,
        pcm,
    })
}
