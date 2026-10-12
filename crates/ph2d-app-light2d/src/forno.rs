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
    em_curso: Option<JoinHandle<()>>,
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
        self.fila.is_empty() && self.em_curso.as_ref().is_none_or(JoinHandle::is_finished)
    }

    /// Um passo, a cada quadro: recolhe a montagem acabada e começa a próxima da fila.
    pub(crate) fn andar(&mut self, gpu: &GpuContext, fabrica: &Fabrica) {
        if self.em_curso.as_ref().is_some_and(|h| !h.is_finished()) {
            return;
        }
        if let Some(h) = self.em_curso.take() {
            let _ = h.join();
        }
        while let Some(cfg) = self.fila.pop_front() {
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
            let k = chave(&cfg);
            self.em_curso = std::thread::Builder::new()
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
            if self.em_curso.is_none() {
                // Sem fio, o quadro monta sozinho (síncrono): nunca fica apagado à espera.
                self.ligado = false;
            }
            return;
        }
    }
}
