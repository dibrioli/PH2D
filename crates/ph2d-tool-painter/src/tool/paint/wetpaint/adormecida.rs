//! **A tinta viva adormecida** (dono 06/11, [plano](../../../../../../../docs/Painter/TINTA_VIVA_2026-11-06.md)): sair do
//! Wet ou da camada não larga a sessão. Parada, sem worker nem tique: medido, nem o traço nem o tique dos outros meios
//! mudam acima do ruído, a 1080p e a 4096×2560. Ao acordar, o que outro meio pintou por cima (o que difere de `tela`)
//! enterra as células sob ele: os dois viram tinta antiga na `base`, e a Borracha leva-os juntos (decisão 2 do dono).

use super::*;
use crate::layers::LayerId as RtLayerId;

/// Quantas vezes o orçamento do undo as adormecidas seguram, em RAM. O crescimento é por CAMADA adormecida: uma camada
/// cheia a 4096×2560 retém 577 MB (sonda `sonda_tinta_adormecida_nos_outros_modos`), logo cabem 4 antes de a mais antiga
/// virar tinta morta.
const ORCAMENTOS_DE_UNDO: usize = 4;

/// Uma sessão parada, com a tela de quando parou. O `Arc` deixa a cópia do estado (o tocador de uma macro) sem copiar a
/// grade.
#[derive(Clone)]
pub(crate) struct TintaAdormecida {
    pub(super) sess: Arc<std::sync::Mutex<WetSession>>,
    pub(super) tela: Arc<Vec<u8>>,
    camada: Option<RtLayerId>,
    bytes: usize,
}

impl PainterTool {
    /// A sessão pode parar a meio? Com a macro a gravar ou a tocar, com um traço aberto ou um rascunho, ela acaba como
    /// sempre: a gravação guarda o estado da tela, não a água.
    fn wet_pode_adormecer(&self) -> bool {
        let w = &self.paint.wetpaint;
        w.session.as_ref().is_some_and(|s| {
            !s.stroke_open && s.preview.is_none() && s.fita.is_none() && s.ritmo.is_none()
        }) && w.fita.is_none()
            && w.ritmo.is_none()
            && self.paint.tracos_fora == 0
            && !self.undo.marca.ativa()
    }

    /// A porta de quem sai do Wet: adormece a sessão, ou acaba-a se ela não pode parar.
    pub(in crate::tool::paint) fn wet_adormece_ou_acaba(&mut self) {
        if !self.wet_adormece() {
            self.wetpaint_end_session();
        }
    }

    /// Adormece a sessão viva: a última imagem na tela, o motor em casa e o worker acabado. `false` se não pode.
    pub(crate) fn wet_adormece(&mut self) -> bool {
        self.wetpaint_guard();
        if !self.wet_pode_adormecer() {
            return false;
        }
        self.wetpaint_pousa_adiados();
        self.wet_ultima_imagem();
        let Some(mut sess) = self.paint.wetpaint.session.take() else {
            return false;
        };
        sess.bring_home();
        sess.worker = None;
        sess.seen_steps = 0;
        sess.canvas = Weak::new();
        self.paint.wetpaint.composta = None;
        self.paint.wetpaint.pending_deposit.clear();
        self.undo.marca.acaba_a_molhada();
        let bytes = sess.bytes_retidos();
        let camada = self.layers.active();
        let lista = &mut self.paint.wetpaint.adormecidas;
        lista.retain(|a| a.camada != camada);
        lista.push(TintaAdormecida {
            sess: Arc::new(std::sync::Mutex::new(sess)),
            tela: Arc::clone(&self.canvas_rgba),
            camada,
            bytes,
        });
        let (w, h) = self.source_size;
        let teto = ORCAMENTOS_DE_UNDO * crate::undo::history_budget_bytes(w, h);
        while lista.len() > 1 && lista.iter().map(|a| a.bytes).sum::<usize>() > teto {
            lista.remove(0);
        }
        true
    }

    /// Acorda a tinta da camada activa. `false` se não há, ou se ela já não serve (a grade ou o Blend mudaram com tinta
    /// na tela: a lei de sempre fecha a sessão).
    pub(in crate::tool::paint) fn wet_acorda(&mut self) -> bool {
        let camada = self.layers.active();
        let lista = &mut self.paint.wetpaint.adormecidas;
        let Some(i) = lista.iter().position(|a| a.camada == camada) else {
            return false;
        };
        let a = lista.remove(i);
        if self.paint.wetpaint.session.is_some() || self.undo.marca.ativa() {
            return false;
        }
        let tela = a.tela;
        let mut sess = match Arc::try_unwrap(a.sess) {
            Ok(m) => m.into_inner().expect("o trinco da adormecida"),
            Err(arc) => arc.lock().expect("o trinco da adormecida").clone(),
        };
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
            self.wet_enterra_o_que_mudou(&mut sess, &tela);
        }
        sess.canvas = Arc::downgrade(&self.canvas_rgba);
        self.paint.wetpaint.session = Some(sess);
        true
    }

    /// As células sob o que outro meio pintou acabam: a imagem de agora passa a ser a `base` delas.
    fn wet_enterra_o_que_mudou(&self, sess: &mut WetSession, velha: &[u8]) {
        let nova = &self.canvas_rgba[..];
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let q = usize::from(sess.ratio);
        let (gw, gh) = sess.grid;
        let mut tocada = vec![false; gw * gh];
        for y in 0..h {
            let o = y * w * 4;
            if velha[o..o + w * 4] == nova[o..o + w * 4] {
                continue;
            }
            for x in 0..w {
                let p = o + x * 4;
                if velha[p..p + 4] != nova[p..p + 4] {
                    tocada[(y / q) * gw + x / q] = true;
                }
            }
        }
        let base = Arc::make_mut(&mut sess.base);
        for y in 0..h {
            let o = y * w * 4;
            for x in 0..w {
                if tocada[(y / q) * gw + x / q] {
                    let p = o + x * 4;
                    base[p..p + 4].copy_from_slice(&nova[p..p + 4]);
                }
            }
        }
        sess.engine.enterra_celulas(&tocada, gw);
    }

    /// A troca de camada: no Wet, acorda a tinta da camada que entra.
    pub(crate) fn wet_acorda_da_camada(&mut self) {
        if matches!(self.paint.paint_mode, PaintMode::WetPaint) {
            self.wet_acorda();
        }
    }

    /// A tinta adormecida acaba toda: o documento mudou, ou a macro vai gravar.
    pub(crate) fn wet_esquece_as_adormecidas(&mut self) {
        self.paint.wetpaint.adormecidas.clear();
    }
}
