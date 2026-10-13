//! ⭐ W5i — **a ESTUFA das pipelines da luz**: o fio do quadro nunca compila a frio. Medido (W5f §3,
//! smoke do dono «o app demora a abrir»): o 1.º `gravar_com` custava 3,5 s — o nascimento do solver a
//! compilar as pipelines no fio do quadro, com a janela presa; e cada nível novo 2–3 s (W5e, ABERTO 1).
//!
//! As pipelines dependem só do shader e das constantes, nunca do tamanho da grade (o portão
//! `redimensionar_a_janela_nao_recompila_a_luz`): um solver de grade MÍNIMA com a mesma configuração,
//! montado noutro fio sobre a mesma [`ph2d_light2d_gpu::Fabrica`], deixa-as todas prontas. Enquanto a
//! configuração pedida não está pronta, o quadro sai sem luz (os receptores neutros) e a janela segue.

use std::collections::{BTreeSet, VecDeque};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::JoinHandle;

use ph2d_gpu::GpuContext;
use ph2d_light2d_gpu::{Config, Fabrica, Grade, Solver};

/// O lado da grade com que a estufa monta (a menor que nasce com todas as passagens: o miolo pede
/// 8 texels finos).
const LADO: u32 = 32;

/// A chave das pipelines de `cfg`: tudo menos a grade.
fn chave(cfg: &Config) -> String {
    let grade = Grade {
        origem_m: [0.0; 2],
        texel_m: 1.0,
        largura: 1,
        altura: 1,
    };
    format!("{:?}", Config { grade, ..*cfg })
}

#[derive(Default)]
pub(crate) struct Forno {
    /// `false` (a omissão): o quadro monta no próprio fio — os portões medem um quadro síncrono.
    pub(crate) ligado: bool,
    prontas: Arc<Mutex<BTreeSet<String>>>,
    fila: VecDeque<Config>,
    /// ⭐ W5o: várias configurações montam ao mesmo tempo (a frio, cada uma leva segundos no
    /// compilador do driver — rodada `medida_luz2d_w5o_abertura`), até [`paralelas`].
    em_curso: Vec<(JoinHandle<()>, bool)>,
    /// ⭐ W5o: as configurações que o quadro PEDIU (espera por elas, apagado): montam sozinhas, com os
    /// núcleos todos; as adiantadas só começam quando nenhuma pedida está em curso.
    pedidas: BTreeSet<String>,
}

/// Quantas configurações a estufa monta ao mesmo tempo: cada uma já compila as pipelines pesadas
/// em 2–4 fios (`Par::em_paralelo`, a fina com o anel), por isso um quarto dos núcleos, de 1 a 4.
fn paralelas() -> usize {
    std::thread::available_parallelism().map_or(1, |n| (n.get() / 4).clamp(1, 4))
}

impl Forno {
    fn prontas(&self) -> std::sync::MutexGuard<'_, BTreeSet<String>> {
        self.prontas.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// As pipelines de `cfg` estão prontas (ou a estufa está desligada)?
    pub(crate) fn pronta(&self, cfg: &Config) -> bool {
        !self.ligado || self.prontas().contains(&chave(cfg))
    }

    /// Pede `cfg` à frente da fila (o quadro espera por ela).
    pub(crate) fn pedir(&mut self, cfg: Config) {
        if self.pronta(&cfg) {
            return;
        }
        let k = chave(&cfg);
        self.fila.retain(|c| chave(c) != k);
        self.fila.push_front(cfg);
        self.pedidas.insert(k);
    }

    /// Adianta `cfg` no fim da fila (um nível que o quadro ainda não pediu).
    pub(crate) fn adiantar(&mut self, cfg: Config) {
        let k = chave(&cfg);
        if !self.pronta(&cfg) && self.fila.iter().all(|c| chave(c) != k) {
            self.fila.push_back(cfg);
        }
    }

    /// Nada na fila nem a montar.
    #[must_use]
    pub(crate) fn ociosa(&self) -> bool {
        self.fila.is_empty() && self.em_curso.iter().all(|(h, _)| h.is_finished())
    }

    /// Um passo, a cada quadro: recolhe as montagens acabadas e começa as próximas da fila.
    pub(crate) fn andar(&mut self, gpu: &GpuContext, fabrica: &Fabrica) {
        let (acabadas, seguem): (Vec<_>, Vec<_>) = std::mem::take(&mut self.em_curso)
            .into_iter()
            .partition(|(h, _)| h.is_finished());
        self.em_curso = seguem;
        for (h, _) in acabadas {
            let _ = h.join();
        }
        while self.em_curso.len() < paralelas()
            && !self.em_curso.iter().any(|(_, pedida)| *pedida)
            && let Some(cfg) = self.fila.pop_front()
        {
            let k = chave(&cfg);
            let pedida = self.pedidas.remove(&k);
            if self.pronta(&cfg) {
                continue;
            }
            let (gpu, fabrica, prontas) = (gpu.clone(), fabrica.clone(), self.prontas.clone());
            let pequena = Config {
                grade: Grade {
                    largura: LADO,
                    altura: LADO,
                    ..cfg.grade
                },
                ..cfg
            };
            let fio = std::thread::Builder::new()
                .name("luz2d-estufa".into())
                .spawn(move || {
                    // A recusa (memória, validação) volta no quadro, que monta a grade de verdade.
                    let _ = Solver::novo_com(&gpu, pequena, &fabrica);
                    prontas
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .insert(k);
                })
                .ok();
            match fio {
                Some(h) => self.em_curso.push((h, pedida)),
                None => {
                    // Sem fio, o quadro monta sozinho (síncrono): nunca fica apagado à espera.
                    self.ligado = false;
                    return;
                }
            }
        }
    }
}
