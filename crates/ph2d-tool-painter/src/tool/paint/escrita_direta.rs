//! **A guarda da escrita DIRETA** (B40.A1): Color Fill, Cut e Colar escrevem na tela sem passar pelo
//! carimbo, e por isso perguntam aqui o que o carimbo já respeita — a proteção da máscara e o trinco de
//! alfa da camada ativa.
//!
//! A lei é a do carimbo, termo a termo: a proteção é o `keep` do `project_gate_region` (o mesmo
//! `mask_value`, a mesma `painted·keep + orig·(1−keep)`), e o trinco devolve o alfa de antes
//! (`out[3] = dst[3]`, como o `composite_linhas`). A seleção NÃO entra: cada ação já a leva na sua
//! própria cobertura (o Colar não é recortado por ela).

use super::PainterTool;
use std::sync::Arc;

/// O que guarda a camada ativa contra uma escrita direta; [`Self::neutra`] quando nada guarda.
pub(in crate::tool::paint) struct GuardaDaEscrita {
    protecao: Option<Arc<Vec<u8>>>,
    trinco: bool,
}

impl GuardaDaEscrita {
    /// Nada guarda: o chamador segue o caminho de sempre, ao bit.
    pub(in crate::tool::paint) fn neutra(&self) -> bool {
        self.protecao.is_none() && !self.trinco
    }

    /// O `keep` da proteção no texel `i`: 1 = livre, 0 = congelado. Fora do buffer da máscara, livre.
    pub(in crate::tool::paint) fn keep(&self, i: usize) -> f32 {
        match &self.protecao {
            Some(s) if i < s.len() / 4 => crate::compositor::mask_value(s, i),
            _ => 1.0,
        }
    }

    /// A fração do CORPO que chega ao texel `i`: o `keep`, e com o trinco só onde já há tinta (`alfa`
    /// de antes) — corpo num texel transparente seria espessura sem pigmento.
    pub(in crate::tool::paint) fn fator_do_corpo(&self, i: usize, alfa: u8) -> f32 {
        let k = self.keep(i);
        if self.trinco { k * f32::from(alfa) / 255.0 } else { k }
    }

    /// Assenta o texel `i` já escrito em `px` contra o que ele era (`orig`).
    pub(in crate::tool::paint) fn assenta(&self, i: usize, orig: [u8; 4], px: &mut [u8]) {
        let keep = self.keep(i);
        if keep < 1.0 {
            for c in 0..4 {
                let (painted, o) = (f32::from(px[c]), f32::from(orig[c]));
                px[c] = (painted * keep + o * (1.0 - keep)).round().clamp(0.0, 255.0) as u8;
            }
        }
        if self.trinco {
            px[3] = orig[3];
        }
    }

    /// A cobertura `cov` que DEPOSITA corpo (Color Fill), com o fator do corpo; `alfa(i)` = o de antes.
    pub(in crate::tool::paint) fn cobertura_que_deposita(&self, cov: &[u8], alfa: impl Fn(usize) -> u8) -> Vec<u8> {
        escala(cov, |i| self.fator_do_corpo(i, alfa(i)))
    }

    /// A cobertura `cov` que RETIRA corpo (Cut): com o trinco o alfa fica, e o corpo fica com ele.
    pub(in crate::tool::paint) fn cobertura_que_retira(&self, cov: &[u8]) -> Vec<u8> {
        escala(cov, |i| if self.trinco { 0.0 } else { self.keep(i) })
    }
}

fn escala(cov: &[u8], f: impl Fn(usize) -> f32) -> Vec<u8> {
    cov.iter()
        .enumerate()
        .map(|(i, &c)| {
            if c == 0 {
                return 0;
            }
            (f32::from(c) * f(i)).round().clamp(0.0, 255.0) as u8
        })
        .collect()
}

impl PainterTool {
    /// A guarda da camada ativa para uma escrita direta (a mesma pergunta do carimbo).
    pub(in crate::tool::paint) fn guarda_da_escrita_direta(&self) -> GuardaDaEscrita {
        #[cfg(test)]
        if CAMINHO_ANTIGO.get() {
            return GuardaDaEscrita {
                protecao: None,
                trinco: false,
            };
        }
        GuardaDaEscrita {
            protecao: self
                .mask_protection_active()
                .then(|| Arc::clone(&self.paint.mask_scratch_rgba)),
            trinco: self.active_alpha_locked(),
        }
    }
}

#[cfg(test)]
thread_local! {
    /// O ORÁCULO do antes (prancha e prova red): a guarda neutra é o caminho de antes do B40.A1, verbatim.
    pub(in crate::tool::paint) static CAMINHO_ANTIGO: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
