//! **A tinta viva adormecida** (dono 06/11, [plano](../../../../../../../docs/Painter/TINTA_VIVA_2026-11-06.md)): sair do
//! Wet ou da camada não larga a sessão. Parada, sem worker nem tique: medido, nem o traço nem o tique dos outros meios
//! mudam acima do ruído, a 1080p e a 4096×2560. Ao acordar, o que outro meio pintou por cima (o que difere de `tela`)
//! enterra as células sob ele: os dois viram tinta antiga na `base`, e a Borracha leva-os juntos (decisão 2 do dono).
//!
//! Dentro do Wet a lei é a mesma: o que outra ferramenta escreve (Fill e Cut da selecção, a Borracha plana) enterra só o
//! que mudou e a sessão continua. O enterro é um passo do undo ([`super::undo_vivo`]): desfeito, o óleo volta.

use super::*;
use crate::layers::LayerId as RtLayerId;

#[cfg(test)]
mod documento_tests;
mod enterro;
mod silencio;
#[cfg(test)]
mod silencio_tests;
pub(crate) use silencio::marca_gesto;
#[cfg(test)]
pub(crate) use silencio::{ENCOLHE_NO_SILENCIO, ULTIMO_FIO};
#[cfg(test)]
mod troca_tests;
pub(crate) use enterro::Congelada;

/// A sessão fora do caminho de um undo sem fotografia, com a tela e a camada de antes.
pub(crate) struct SessaoNoRestauro {
    sess: WetSession,
    tela: Arc<Vec<u8>>,
}

/// Quantas vezes o orçamento do undo DA MÁQUINA ([`crate::undo::history_budget_here`]) as adormecidas seguram. Recurso:
/// a RAM (4/32 dela; o piso do documento sem leitura). B42.3: a calibração antiga dizia «577 MB por camada», mas eram
/// 5 081 MB (IN-18); a 4K o piso (2,14 GB) não segurava UMA camada cheia, e a de outra camada morria. Com 123 GiB o
/// tecto é 15,4 GiB: três camadas cheias de 5 GB.
const ORCAMENTOS_DE_UNDO: usize = 4;

/// Uma sessão parada, com a tela de quando parou. O `Arc` deixa a cópia do estado (o tocador de uma macro) sem copiar a
/// grade.
#[derive(Clone)]
pub(crate) struct TintaAdormecida {
    pub(super) sess: Arc<std::sync::Mutex<WetSession>>,
    pub(super) tela: Arc<Vec<u8>>,
    camada: Option<RtLayerId>,
    /// O que ela retém agora: desce quando o fio da compressão acaba.
    pub(super) bytes: Arc<std::sync::atomic::AtomicUsize>,
    aquecendo: Arc<std::sync::atomic::AtomicBool>,
    /// A compressão à espera do silêncio ([`silencio`]); acordar cancela-a.
    espera: Option<silencio::Espera>,
}

/// Quanto se conta para uma tinta ainda a comprimir: medido 7,3× a 1080p e 10× a 4096×2560 (Dabber cheio).
const ENCOLHE_PELO_MENOS: usize = 7;

/// Abaixo disto não vale um fio: uma camada pequena pesa menos do que o custo de a esperar ao acordar.
const COMPRIME_A_PARTIR_DE: usize = 64 << 20;

#[cfg(test)]
thread_local! {
    /// n.º 10 F5j (B18.7): a sonda baixa o limiar para ligar o fio numa sessão pequena (`PH2D_SONDA_COMPRIME_MB`).
    pub(crate) static COMPRIME_A_PARTIR_NO_TESTE: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) };
}

fn comprime_a_partir_de() -> usize {
    #[cfg(test)]
    if let Some(b) = COMPRIME_A_PARTIR_NO_TESTE.get() {
        return b;
    }
    COMPRIME_A_PARTIR_DE
}

impl TintaAdormecida {
    /// Comprime aqui e agora (o fio faz o mesmo).
    #[cfg(test)]
    pub(super) fn encolhe_ja(&self) {
        let n = self.sess.lock().expect("o trinco da adormecida").encolhe();
        self.bytes.store(n, std::sync::atomic::Ordering::Relaxed);
    }

    fn encolhe_num_fio(&mut self) {
        let (sess, bytes) = (Arc::downgrade(&self.sess), Arc::downgrade(&self.bytes));
        self.espera = silencio::lanca(move || {
            let Some(s) = sess.upgrade() else {
                return;
            };
            let n = s.lock().expect("o trinco da adormecida").encolhe();
            if let Some(b) = bytes.upgrade() {
                b.store(n, std::sync::atomic::Ordering::Relaxed);
            }
        });
    }

    /// Acordar agora não espera ninguém: nenhum fio nela, nada por descomprimir.
    fn pronta(&self) -> bool {
        let por_comecar = self.espera.as_ref().is_some_and(silencio::Espera::pendente);
        (por_comecar || Arc::weak_count(&self.sess) == 0)
            && self.sess.try_lock().is_ok_and(|s| s.dormente.is_none())
    }

    /// Descomprime num fio à parte: a troca de modo não engasga, e o quadro acorda-a quando ficar pronta.
    fn aquece(&self) {
        use std::sync::atomic::Ordering::SeqCst;
        if self.aquecendo.swap(true, SeqCst) {
            return;
        }
        let (sess, aquecendo) = (Arc::downgrade(&self.sess), Arc::clone(&self.aquecendo));
        std::thread::spawn(move || {
            if let Some(s) = sess.upgrade() {
                s.lock().expect("o trinco da adormecida").reidrata();
            }
            aquecendo.store(false, SeqCst);
        });
    }

    /// A sessão de volta: espera o fio, leva-a sem copiar, e descomprime.
    fn acorda(self) -> WetSession {
        use std::sync::atomic::Ordering::Relaxed;
        let _ = self.bytes.load(Relaxed);
        if let Some(e) = &self.espera {
            e.cancela();
        }
        let mut arc = self.sess;
        let mut sess = loop {
            // O fio solta o trinco e larga o `Arc` logo a seguir: esperar o trinco é esperar a compressão.
            drop(arc.lock().expect("o trinco da adormecida"));
            match Arc::try_unwrap(arc) {
                Ok(m) => break m.into_inner().expect("o trinco da adormecida"),
                Err(a) if Arc::weak_count(&a) == 0 && Arc::strong_count(&a) > 1 => {
                    // Uma cópia do estado segura-a: copiar é o caminho certo.
                    break a.lock().expect("o trinco da adormecida").clone();
                }
                Err(a) => {
                    arc = a;
                    std::thread::yield_now();
                }
            }
        };
        sess.reidrata();
        sess
    }
}

impl PainterTool {
    /// A sessão pode parar a meio? Com a macro a gravar ou a tocar, com um traço aberto ou um rascunho, ela acaba como
    /// sempre: a gravação guarda o estado da tela, não a água.
    fn wet_pode_adormecer(&self) -> bool {
        self.wet_sem_gesto_nem_macro() && self.paint.tracos_fora == 0
    }

    fn wet_sem_gesto_nem_macro(&self) -> bool {
        let w = &self.paint.wetpaint;
        w.session.as_ref().is_some_and(|s| {
            !s.stroke_open && s.preview.is_none() && s.fita.is_none() && s.ritmo.is_none()
        }) && w.fita.is_none()
            && w.ritmo.is_none()
            && !self.undo.marca.ativa()
    }

    /// O guarda viu uma tela que a sessão não pintou. `true` = a sessão continua: enterrou o que mudou, ou espera a
    /// operação aberta acabar (a tela é dela até lá). `false` = não sabe o que mudou, e acaba.
    pub(super) fn wet_atravessa_a_troca(&mut self) -> bool {
        if !self.wet_sem_gesto_nem_macro() {
            return false;
        }
        if self.paint.stroke_undo.is_some() || self.paint.tracos_fora > 0 {
            return true;
        }
        let Some(velha) = self.undo.write_state.tela_que_saiu() else {
            return false;
        };
        let Some(mut sess) = self.paint.wetpaint.session.take() else {
            return false;
        };
        // A tela retida é sempre a da sessão: `vigia_a_tela` e `sess.canvas` andam juntos em todos os sítios que um
        // deles muda, e `a_tela_sai` só retém a vigiada. Por isso não há guarda de ponteiro aqui (mutação provada
        // equivalente: nenhum caminho a decide).
        if velha.len() != self.canvas_rgba.len() || sess.camada != self.layers.active() {
            return false;
        }
        sess.bring_home();
        self.wet_enterra_como_passo(&mut sess, &velha, None);
        sess.canvas = Arc::downgrade(&self.canvas_rgba);
        self.undo.write_state.vigia_a_tela(Some(&self.canvas_rgba));
        self.paint.wetpaint.session = SessaoDoWet::aqui(sess);
        true
    }

    /// Antes de um undo/redo sem fotografia: a sessão sai do caminho do restore, que a acabaria.
    pub(crate) fn wet_tira_para_o_restauro(&mut self) -> Option<SessaoNoRestauro> {
        self.wetpaint_guard();
        if !self.wet_pode_adormecer() || self.wet_tela_alheia() {
            return None;
        }
        let mut sess = self.paint.wetpaint.session.take()?;
        sess.bring_home();
        self.undo.write_state.vigia_a_tela(None);
        Some(SessaoNoRestauro {
            sess,
            tela: Arc::clone(&self.canvas_rgba),
        })
    }

    /// Depois do undo/redo: a mesma tela (um passo sem pixels), outra camada (a tinta adormece com a dela), um redo (a
    /// tela nova enterra-se como qualquer escrita), ou um undo de um passo que já saiu do histórico (o que voltou atrás
    /// enterra-se fora da história).
    pub(crate) fn wet_volta_do_restauro(&mut self, p: SessaoNoRestauro, refaz: bool) {
        let SessaoNoRestauro { mut sess, tela } = p;
        // Um undo que não se aplicou deixa a mesma `Arc`: o `ptr_eq` poupa a comparação da tela inteira (e não decide
        // nada: com a mesma `Arc` o conteúdo é igual por definição).
        let mesma_tela = Arc::ptr_eq(&tela, &self.canvas_rgba) || *tela == *self.canvas_rgba;
        if sess.camada != self.layers.active() {
            self.wet_guarda_adormecida(sess, tela);
            self.wet_acorda_da_camada();
            return;
        }
        if !mesma_tela {
            if tela.len() != self.canvas_rgba.len() {
                return;
            }
            let casco =
                super::undo_vivo::casco_de(&sess, self.undo.chave_do_topo() - usize::from(refaz));
            if let Some(c) = casco {
                self.wet_fora_do_casco_fica(&sess, &tela, c);
            }
            if refaz {
                self.wet_enterra_como_passo(&mut sess, &tela, casco);
            } else {
                self.wet_enterra_sem_passo(&mut sess, &tela, casco);
            }
        }
        sess.canvas = Arc::downgrade(&self.canvas_rgba);
        self.undo.write_state.vigia_a_tela(Some(&self.canvas_rgba));
        self.paint.wetpaint.session = SessaoDoWet::aqui(sess);
    }

    /// A porta de quem sai do Wet: adormece a sessão, ou acaba-a se ela não pode parar.
    pub(in crate::tool::paint) fn wet_adormece_ou_acaba(&mut self) {
        if !self.wet_adormece() {
            self.wetpaint_end_session();
        }
    }

    /// Adormece a sessão viva: a última imagem na tela, o motor em casa e o worker acabado. `false` se não pode.
    pub(crate) fn wet_adormece(&mut self) -> bool {
        crate::sonda_partes::parte("flush.antes");
        self.wetpaint_guard();
        crate::sonda_partes::parte("flush.guard");
        if !self.wet_pode_adormecer() || self.wet_tela_alheia() {
            return false;
        }
        crate::sonda_partes::parte("flush.pode");
        self.wetpaint_pousa_adiados();
        crate::sonda_partes::parte("flush.pousa_adiados");
        self.wet_ultima_imagem();
        crate::sonda_partes::parte("flush.ultima_imagem");
        let Some(sess) = self.paint.wetpaint.session.take() else {
            return false;
        };
        self.wet_guarda_adormecida(sess, Arc::clone(&self.canvas_rgba));
        true
    }

    /// A sessão pára, com a tela `tela` da camada dela.
    fn wet_guarda_adormecida(&mut self, mut sess: WetSession, tela: Arc<Vec<u8>>) {
        sess.bring_home();
        crate::sonda_partes::parte("flush.bring_home");
        sess.worker = None;
        crate::sonda_partes::parte("flush.worker_none");
        sess.seen_steps = 0;
        sess.otica = Default::default(); // o compositor acabou com o worker: nenhuma imagem está no ar
        sess.canvas = Weak::new();
        self.undo.write_state.vigia_a_tela(None);
        self.paint.wetpaint.composta = None;
        self.paint.wetpaint.pending_deposit.clear();
        self.undo.marca.acaba_a_molhada();
        let bytes = sess.bytes_retidos();
        let limiar = comprime_a_partir_de();
        let camada = sess.camada;
        let lista = &mut self.paint.wetpaint.adormecidas;
        lista.retain(|a| a.camada != camada);
        let mut tinta = TintaAdormecida {
            sess: Arc::new(std::sync::Mutex::new(sess)),
            tela,
            camada,
            bytes: Arc::new(std::sync::atomic::AtomicUsize::new(if bytes >= limiar {
                bytes / ENCOLHE_PELO_MENOS
            } else {
                bytes
            })),
            aquecendo: Arc::default(),
            espera: None,
        };
        crate::sonda_partes::parte("flush.guarda");
        if bytes >= limiar {
            tinta.encolhe_num_fio();
        }
        lista.push(tinta);
        let (w, h) = self.source_size;
        let teto = ORCAMENTOS_DE_UNDO * crate::undo::history_budget_here(w, h);
        let soma = |l: &[TintaAdormecida]| -> usize {
            l.iter()
                .map(|a| a.bytes.load(std::sync::atomic::Ordering::Relaxed))
                .sum()
        };
        while lista.len() > 1 && soma(lista) > teto {
            lista.remove(0);
        }
        self.wet_poda_as_adormecidas();
    }

    /// [`Self::wet_acorda`] de quem não pode engasgar (a troca de modo, de camada, o quadro): se ainda há trabalho, ele
    /// corre num fio e o quadro seguinte volta a tentar. O pen-down e o resto esperam, pela de sempre.
    pub(in crate::tool::paint) fn wet_acorda_sem_esperar(&mut self) -> bool {
        let camada = self.layers.active();
        // B18.5: num relógio que não é o da parede (gates, miniaturas, a macro) o quadro do acordar não pode depender
        // do fio da compressão: espera-o, e a mesma sequência dá o mesmo bit.
        let w = &self.paint.wetpaint;
        let sem_parede = w.relogio_fixo.is_some() || w.fita.is_some() || w.ritmo.is_some();
        if !sem_parede
            && let Some(a) = w.adormecidas.iter().find(|a| a.camada == camada)
            && !a.pronta()
        {
            a.aquece();
            return false;
        }
        self.wet_acorda()
    }

    /// Acorda a tinta da camada activa. `false` se não há, ou se ela já não serve (a grade ou o Blend mudaram com tinta
    /// na tela: a lei de sempre fecha a sessão).
    pub(in crate::tool::paint) fn wet_acorda(&mut self) -> bool {
        // As recusas antes do `remove` (B02.3, O2-16): recusar não leva a tinta.
        if self.paint.wetpaint.session.is_some() || self.undo.marca.ativa() {
            return false;
        }
        let camada = self.layers.active();
        let lista = &mut self.paint.wetpaint.adormecidas;
        let Some(i) = lista.iter().position(|a| a.camada == camada) else {
            return false;
        };
        let a = lista.remove(i);
        let tela = Arc::clone(&a.tela);
        let mut sess = a.acorda();
        let w = &self.paint.wetpaint;
        let grade_igual = sess.ratio == grid_map::clamp_ratio(w.grid_ratio)
            && sess.engine.flow_ratio
                == ph2d_wet_paint::flow::clamp_ratio(usize::from(w.flow_ratio.max(1)));
        let modo = super::modo_da_sessao(self.paint.brush.blend);
        let blend_igual = sess.blend == modo && sess.vazio == self.paint.brush.blend_sobre_vazio;
        if !grade_igual
            || sess.base.len() != self.canvas_rgba.len()
            || (!blend_igual && sess.ha_tinta())
        {
            return false;
        }
        sess.blend = modo;
        sess.vazio = self.paint.brush.blend_sobre_vazio;
        if !Arc::ptr_eq(&tela, &self.canvas_rgba) && *tela != *self.canvas_rgba {
            self.wet_enterra_como_passo(&mut sess, &tela, None);
        }
        sess.canvas = Arc::downgrade(&self.canvas_rgba);
        self.undo.write_state.vigia_a_tela(Some(&self.canvas_rgba));
        self.paint.wetpaint.session = SessaoDoWet::aqui(sess);
        true
    }

    /// A troca de camada: no Wet, acorda a tinta da camada que entra.
    pub(crate) fn wet_acorda_da_camada(&mut self) {
        if matches!(self.paint.paint_mode, PaintMode::WetPaint) {
            self.wet_acorda_sem_esperar();
        }
    }

    /// A tinta adormecida acaba toda: o documento mudou, ou a macro vai gravar.
    pub(crate) fn wet_esquece_as_adormecidas(&mut self) {
        self.paint.wetpaint.adormecidas.clear();
    }

    /// **A troca de documento** (B02.1): a sessão viva adormece (ou acaba) e as adormecidas saem com o documento —
    /// os ids de camada colidem entre documentos, então nenhuma fica para acordar no seguinte.
    pub(crate) fn wet_sai_do_documento(&mut self) -> Vec<TintaAdormecida> {
        if self.paint.wetpaint.session.is_some() {
            self.wet_adormece_ou_acaba();
        }
        std::mem::take(&mut self.paint.wetpaint.adormecidas)
    }

    /// As adormecidas do documento que entra substituem as do que saiu.
    pub(crate) fn wet_entra_no_documento(&mut self, tintas: Vec<TintaAdormecida>) {
        self.paint.wetpaint.adormecidas = tintas;
        self.wet_poda_as_adormecidas();
    }

    /// A tinta de uma camada que já não existe acaba: o undo repõe o `next_id`, e a camada seguinte herdaria o id.
    pub(crate) fn wet_poda_as_adormecidas(&mut self) {
        let l = &self.layers;
        self.paint
            .wetpaint
            .adormecidas
            .retain(|a| a.camada.is_none_or(|c| l.get(c).is_some()));
    }

    /// As camadas com tinta adormecida (o gate de que elas seguem a pilha).
    #[cfg(test)]
    pub(crate) fn camadas_adormecidas(&self) -> Vec<RtLayerId> {
        let a = &self.paint.wetpaint.adormecidas;
        a.iter().filter_map(|a| a.camada).collect()
    }
}
