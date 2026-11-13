//! **O fundo do toque compõe-se num fio, fora do clique** (n.º 7 item 3f,
//! [medição](../../../../../docs/Painter/missao_4k/medicoes/N7_ITEM3_mais.md) «Continuação 5»): o «+» e o clique na
//! linha tiram um instantâneo do que o fundo de baixo lê (a pilha, os `Arc` das camadas NÃO activas, o papel) e um fio
//! compõe o fundo e o chão preto do vidro num pool rayon próprio (nunca o do traço). O pen-down colhe: chave igual ⇒
//! espera o fio se ainda corre e usa; chave diferente ⇒ cancela e compõe como antes. As mesmas funções sobre as mesmas
//! entradas: o mesmo bit. O instantâneo não segura a camada activa (vive em `canvas_rgba`), por isso o traço que
//! começa nela não copia o plano por causa dele.

use super::watercolor_backdrop::ChaveDoFundo;
use super::*;
use crate::compositor::{Corte, composite_below_cortavel};
use std::sync::atomic::Ordering::SeqCst;
use std::sync::{Arc, OnceLock};

thread_local! {
    /// **A chave do A×B**: `true` = o aquecimento da troca corre num fio; `false` = no clique, como no item 3e (o
    /// oráculo). Lida no fio da UI. Só a sonda e o gate a desligam.
    #[doc(hidden)]
    pub static FUNDO_AQUECIDO_EM_FUNDO: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

#[cfg(test)]
thread_local! {
    /// Atraso do fio antes de compor (o gate «a meio»).
    pub(super) static ATRASO_DO_AQUECIMENTO_MS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    /// `(colhidos, descartados)` neste fio da UI.
    pub(super) static CONTAS: std::cell::Cell<(u32, u32)> = const { std::cell::Cell::new((0, 0)) };
}

type Feito = (Arc<Vec<u8>>, Option<Arc<Vec<u8>>>);

pub(crate) struct Aquecendo {
    chave: ChaveDoFundo,
    cancela: Arc<Corte>,
    fio: std::thread::JoinHandle<Option<Feito>>,
}

impl Aquecendo {
    /// Desiste: o fio sai antes da fase seguinte e larga o instantâneo sozinho (sem esperar por ele aqui).
    fn descarta(self) {
        self.cancela.para.store(true, SeqCst);
        #[cfg(test)]
        CONTAS.set((CONTAS.get().0, CONTAS.get().1 + 1));
    }
}

/// O aquecimento por colher. Um clone (o estado do pincel copiado) não herda o fio.
#[derive(Default)]
pub(crate) struct AquecimentoPendente(Option<Aquecendo>);

impl Clone for AquecimentoPendente {
    fn clone(&self) -> Self {
        Self(None)
    }
}

/// Pool próprio, do tamanho do global: o fio nunca rouba o pool a um traço ou a um preview.
fn pool() -> &'static rayon::ThreadPool {
    static POOL: OnceLock<rayon::ThreadPool> = OnceLock::new();
    POOL.get_or_init(|| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(rayon::current_num_threads())
            .thread_name(|i| format!("ph2d-aquece-{i}"))
            .build()
            .expect("o pool do aquecimento")
    })
}

impl PainterTool {
    /// A rota em fundo do [`Self::aquece_o_fundo_do_toque`]: `false` = a chave está desligada (o chamador aquece no
    /// clique). `camada` é a activa, não máscara.
    pub(super) fn aquece_em_fundo(&mut self, camada: RtLayerId) -> bool {
        if !FUNDO_AQUECIDO_EM_FUNDO.get() || !super::watercolor_backdrop::FUNDO_EM_CACHE.get() {
            return false;
        }
        let chave = self.chave_do_fundo();
        if let Some(a) = self.paint.aquecimento.0.take() {
            if a.chave.casa(&chave) {
                // A troca de pincel depois do clique (ou duas portas no mesmo gesto) não refaz o que já corre.
                self.paint.aquecimento.0 = Some(a);
                return true;
            }
            a.descarta();
        }
        let pronto = matches!(&self.paint.fundo_em_cache,
            Some((c, f)) if c.casa(&chave) && self.chao_preto_fresco(f));
        if pronto {
            return true;
        }
        let abaixo = self.layers.z_order_bottom_up().first() != Some(&camada);
        let pilha = self.layers.clone();
        let images = self.images.clone();
        let (w, h) = self.source_size;
        let papel = self.cor_do_chao();
        let cancela = Arc::new(Corte::default());
        let parar = Arc::clone(&cancela);
        #[cfg(test)]
        let atraso = ATRASO_DO_AQUECIMENTO_MS.get();
        let corpo = move || {
            #[cfg(test)]
            std::thread::sleep(std::time::Duration::from_millis(atraso));
            let compoe = |chao: [u8; 3]| {
                let src = ToolPixelSource {
                    active_id: camada,
                    active_rgba: &[],
                    images: &images,
                };
                pool()
                    .install(|| composite_below_cortavel(&pilha, &src, w, h, camada, chao, &parar))
                    .map(Arc::new)
            };
            let fundo = compoe(papel)?;
            let preto = if abaixo {
                Some(compoe([0, 0, 0])?)
            } else {
                None
            };
            drop((images, pilha)); // o instantâneo sai antes de o pen-down ver o resultado
            Some((fundo, preto))
        };
        match std::thread::Builder::new()
            .name("ph2d-aquece-fundo".into())
            .spawn(corpo)
        {
            Ok(fio) => {
                self.paint.aquecimento.0 = Some(Aquecendo {
                    chave,
                    cancela,
                    fio,
                });
                true
            }
            Err(_) => false,
        }
    }

    /// O pen-down colhe o aquecimento: a chave de agora casa ⇒ espera o fio (se ainda corre) e guarda o fundo e o chão
    /// preto nas caches de sempre; senão descarta-o e o chamador compõe.
    pub(super) fn colhe_o_aquecimento(&mut self, chave: &ChaveDoFundo) {
        let Some(a) = self.paint.aquecimento.0.take() else {
            return;
        };
        if !a.chave.casa(chave) {
            a.descarta();
            return;
        }
        let Ok(Some((fundo, preto))) = a.fio.join() else {
            return;
        };
        #[cfg(test)]
        CONTAS.set((CONTAS.get().0 + 1, CONTAS.get().1));
        self.paint.fundo_em_cache = Some((a.chave, Arc::clone(&fundo)));
        self.guarda_o_chao_preto(&fundo, preto);
    }

    /// O corte do aquecimento por colher (o gate do fio descartado).
    #[cfg(test)]
    pub(super) fn corte_do_aquecimento(&self) -> Option<Arc<Corte>> {
        self.paint
            .aquecimento
            .0
            .as_ref()
            .map(|a| Arc::clone(&a.cancela))
    }

    /// O fio do aquecimento já acabou (`None` = não há aquecimento por colher).
    #[cfg(test)]
    pub(super) fn aquecimento_acabou(&self) -> Option<bool> {
        self.paint
            .aquecimento
            .0
            .as_ref()
            .map(|a| a.fio.is_finished())
    }
}
