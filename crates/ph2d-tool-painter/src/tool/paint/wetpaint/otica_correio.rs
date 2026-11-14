//! A CAIXA DE CORREIO da óptica e as duas pontas (filho de [`super`] por LOC, B52.R8).

use super::agenda::{Agenda, Momento};
use super::compositor::{Trabalho, compositor};
use super::{CaixaCel, PedidoDeImagem, RespostaDaOtica, SujoDoPedido, caixa_da_grade};
use crate::tool::paint::wetpaint::composite_otica::{EntradaDaOtica, TrabalhoDaOtica};
use crate::tool::paint::wetpaint::fita::MarcaDaAgua;
use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::painter::espelho::Espelho;
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Instant;

/// A caixa de correio entre o fio principal e o worker.
#[derive(Default)]
struct Correio {
    pedido: Option<PedidoDeImagem>,
    /// `(época, faixas)` do último pedido servido: o sujo saiu do motor e a imagem dele vem a caminho (B14.1: as
    /// faixas, não a envolvente).
    servido: Option<(u64, Vec<CaixaCel>)>,
    /// O fio do compositor morreu (B11.A1): o sujo servido voltou ao motor e a tela compõe em casa.
    morto: bool,
    /// A porta de teste: o próximo `compoe` entra em pânico.
    #[cfg(test)]
    mata: bool,
}

/// A ponta do fio principal.
pub(in crate::tool::paint::wetpaint) struct DoFioPrincipal {
    correio: Arc<Mutex<Correio>>,
    imagens: Receiver<RespostaDaOtica>,
    /// O fio do compositor, para os `leak_tests` verem que acabou (a thread é solta de propósito).
    #[cfg(test)]
    pub(in crate::tool::paint::wetpaint::offthread) fio: Option<std::thread::JoinHandle<()>>,
}

impl DoFioPrincipal {
    /// Publica `p`; devolve o pedido ainda por servir que ele substituiu (a dívida dele é de quem pede).
    pub(in crate::tool::paint::wetpaint) fn pede(
        &self,
        p: PedidoDeImagem,
    ) -> Option<PedidoDeImagem> {
        trava(&self.correio).pedido.replace(p)
    }

    /// O sujo que o pedido `epoca` levou do motor: `(a dívida dele por servir ++ as faixas já servidas, a água dele por
    /// servir)`. Só vale com o motor em casa: o worker não está a meio de um `serve`.
    pub(in crate::tool::paint::wetpaint) fn sujo_do_pedido(&self, epoca: u64) -> SujoDoPedido {
        let c = trava(&self.correio);
        let pedido = c.pedido.as_ref().filter(|p| p.epoca == epoca);
        let mut sujo: Vec<CaixaCel> = pedido.map(|p| p.divida.clone()).unwrap_or_default();
        if let Some((_, faixas)) = c.servido.as_ref().filter(|s| s.0 == epoca) {
            sujo.extend_from_slice(faixas);
        }
        (sujo, pedido.and_then(|p| p.agua))
    }

    /// O compositor ainda está de pé? `false` = morreu e o sujo que levava voltou ao motor (B11.A1).
    pub(in crate::tool::paint::wetpaint) fn vivo(&self) -> bool {
        !trava(&self.correio).morto
    }

    /// A porta de teste de B11.A1: o próximo `compoe` entra em pânico e o fio do compositor morre.
    #[cfg(test)]
    pub(in crate::tool::paint::wetpaint) fn mata_no_teste(&self) {
        trava(&self.correio).mata = true;
    }

    /// A próxima resposta, sem esperar.
    pub(in crate::tool::paint::wetpaint) fn try_recv(&self) -> Option<RespostaDaOtica> {
        self.imagens.try_recv().ok()
    }
}

/// A ponta do worker: a caixa de correio, o espelho (quando está com ele) e a previsão da cópia.
pub(in crate::tool::paint::wetpaint::offthread) struct DoWorker {
    correio: Arc<Mutex<Correio>>,
    para_o_compositor: Sender<Trabalho>,
    volta: Receiver<Box<Espelho>>,
    ui: Sender<RespostaDaOtica>,
    espelho: Option<Box<Espelho>>,
    agenda: Agenda,
    relogio: Instant,
    morto: bool,
}

fn trava<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Sobe o compositor e devolve as duas pontas.
pub(in crate::tool::paint::wetpaint::offthread) fn par() -> (DoWorker, DoFioPrincipal) {
    let correio = Arc::new(Mutex::new(Correio::default()));
    let (para_o_compositor, rx) = channel::<Trabalho>();
    let (de_volta, volta) = channel::<Box<Espelho>>();
    let (ui, imagens) = channel::<RespostaDaOtica>();
    let ui_c = ui.clone();
    let fio = std::thread::Builder::new()
        .name("ph2d-wet-otica".into())
        .spawn(move || compositor(&rx, &de_volta, &ui_c))
        .expect("a thread do compositor da optica");
    #[cfg(not(test))]
    drop(fio);
    (
        DoWorker {
            correio: Arc::clone(&correio),
            para_o_compositor,
            volta,
            ui,
            espelho: Some(Box::default()),
            agenda: Agenda::default(),
            relogio: Instant::now(),
            morto: false,
        },
        DoFioPrincipal {
            correio,
            imagens,
            #[cfg(test)]
            fio: Some(fio),
        },
    )
}

impl DoWorker {
    /// O gancho do `worker_loop`, num passo FECHADO: serve o pedido se o espelho está aqui e a [`Agenda`] deixa (a
    /// cópia cabe na folga, ou ocupa ≤ 20 % do worker com a água atrasada). O correio lê-se primeiro: sem pedido
    /// nada anda.
    pub(in crate::tool::paint::wetpaint::offthread) fn serve(
        &mut self,
        engine: &mut Engine,
        acc: f64,
        passos: u64,
    ) {
        if self.morto || engine.sim.step_pending() {
            return;
        }
        if self.espelho.is_none() {
            match self.volta.try_recv() {
                Ok(e) => self.espelho = Some(e),
                // O compositor morreu com o espelho (pânico em `compoe`): o sujo servido perdeu-se com ele.
                Err(TryRecvError::Disconnected) => return self.morre(engine),
                Err(TryRecvError::Empty) => return,
            }
        }
        if trava(&self.correio).pedido.is_none() {
            return;
        }
        let m = Momento {
            agora_s: self.relogio.elapsed().as_secs_f64(),
            acc,
            sim_corre: engine.sim_should_run(),
        };
        if !self.agenda.deve_servir(m) {
            return;
        }
        let Some(p) = trava(&self.correio).pedido.take() else {
            return;
        };
        let i = |v: usize| i32::try_from(v).unwrap_or(i32::MAX);
        for &(x0, y0, x1, y1) in &p.divida {
            engine.merge_dirty_otica(i(x0), i(y0), i(x1), i(y1));
        }
        if let Some((x0, y0, x1, y1)) = p.agua {
            engine.devolve_sujo_do_passo(i(x0), i(y0), i(x1), i(y1));
        }
        let marca = MarcaDaAgua::de(engine);
        // B14.1: o sujo do passo da sim reduz-se às faixas de células que diferem do espelho (a entrada da imagem
        // anterior).
        let sujo = match self.espelho.as_deref() {
            Some(e) => engine.take_dirty_pela_otica_em_faixas(e),
            None => vec![engine.take_dirty()],
        };
        let faixas: Vec<CaixaCel> = sujo
            .into_iter()
            .filter_map(|d| caixa_da_grade(d, p.forma.grade))
            .collect();
        let Some(caixa) = faixas
            .iter()
            .copied()
            .reduce(|a, b| crate::tool::paint::wetpaint::otica_no_tique::une(Some(a), b))
        else {
            let _ = self.ui.send(RespostaDaOtica::Nada {
                epoca: p.epoca,
                passos,
                marca,
            });
            return;
        };
        trava(&self.correio).servido = Some((p.epoca, faixas.clone()));
        let Some(mut espelho) = self.espelho.take() else {
            unreachable!("conferido acima")
        };
        let t = Instant::now();
        engine.copia_para(&mut espelho);
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        self.agenda.copiou(ms, self.relogio.elapsed().as_secs_f64());
        crate::wet_diag::note_copia(ms as f32);
        let trabalho = Trabalho {
            espelho,
            entrada: EntradaDaOtica {
                factos: p.factos,
                marca,
                trabalho: Some(TrabalhoDaOtica {
                    caixa,
                    params: engine.sim.gather_params(&engine.tuning),
                }),
                ratio: p.forma.ratio,
                grade: p.forma.grade,
                tela: p.forma.tela,
                revisao: engine.revisao(),
            },
            faixas,
            epoca: p.epoca,
            passos,
            veu: p.com_veu.then_some(engine.active_layer),
            pixel: p.pixel,
            #[cfg(test)]
            panica: trava(&self.correio).mata,
        };
        if let Err(e) = self.para_o_compositor.send(trabalho) {
            self.espelho = Some(e.0.espelho);
            self.morre(engine);
        }
    }

    /// O compositor morreu: o sujo do pedido servido volta ao motor e a tela passa a compor em casa (B11.A1).
    fn morre(&mut self, engine: &mut Engine) {
        self.morto = true;
        let mut c = trava(&self.correio);
        c.morto = true;
        let i = |v: usize| i32::try_from(v).unwrap_or(i32::MAX);
        for (x0, y0, x1, y1) in c.servido.take().map(|s| s.1).unwrap_or_default() {
            engine.merge_dirty_otica(i(x0), i(y0), i(x1), i(y1));
        }
    }
}
