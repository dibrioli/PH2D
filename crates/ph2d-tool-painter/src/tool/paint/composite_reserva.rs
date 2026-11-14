//! **B17.8: a reserva dos planos da pilha entre traços** (molde `SparePlanes`).
//!
//! Cada camada viva abria um plano do tamanho da tela por traço (33 MB a 4K, as páginas faltam no
//! primeiro toque). O `fecha` guarda o plano (se ninguém mais o segura) com o chão dele e a caixa que
//! o depósito MARCOU nele ([`super::stamp_preview`]: `mark_dirty` com `acumulando_no_plano`); o
//! traço seguinte repõe o chão SÓ nessa caixa. ⛔ `clear()+resize` reescreveria a tela inteira (RC-10).

use super::Region;
use super::composite::N_CAMADAS;
use std::sync::Arc;

/// O chão de um plano e a caixa que o depósito escreveu nele, `None` = intacto.
#[derive(Copy, Clone, Default, PartialEq, Eq, Debug)]
pub(super) struct SujoDoPlano {
    pub(super) chao: u8,
    pub(super) caixa: Option<Region>,
}

impl SujoDoPlano {
    pub(super) fn une(&mut self, r: Region) {
        self.caixa = Some(self.caixa.map_or(r, |a| super::region::union_region(a, r)));
    }
}

struct Guardado {
    plano: Vec<u8>,
    sujo: SujoDoPlano,
}

/// ⚠️ O `Clone` dá uma reserva VAZIA: é cache, e clonar o estado não deve duplicar 33 MB por plano.
#[derive(Default)]
pub(super) struct ReservaDePlanos {
    guardados: Vec<Guardado>,
}

impl Clone for ReservaDePlanos {
    fn clone(&self) -> Self {
        Self::default()
    }
}

#[cfg(test)]
thread_local! {
    /// A ablação do gate: sem reserva, todo plano nasce do alocador (a rota de antes).
    pub(super) static SEM_RESERVA: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// A mutação do gate: repõe nada (o plano volta com a tinta do traço anterior).
    pub(super) static NAO_REPOE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

impl ReservaDePlanos {
    /// Guarda o plano se o `Arc` é só dele; vazio ou partilhado larga-se.
    pub(super) fn guarda(&mut self, plano: Arc<Vec<u8>>, sujo: SujoDoPlano) {
        if plano.is_empty() || self.guardados.len() >= N_CAMADAS {
            return;
        }
        #[cfg(test)]
        if SEM_RESERVA.with(std::cell::Cell::get) {
            return;
        }
        if let Ok(plano) = Arc::try_unwrap(plano) {
            self.guardados.push(Guardado { plano, sujo });
        }
    }

    #[cfg(test)]
    pub(super) fn quantos(&self) -> usize {
        self.guardados.len()
    }

    /// Um plano de `len` bytes todo no `chao`, de `w` píxeis por linha — ou `None` (nasce do alocador).
    pub(super) fn tira(&mut self, len: usize, chao: u8, w: u32) -> Option<Vec<u8>> {
        #[cfg(test)]
        if SEM_RESERVA.with(std::cell::Cell::get) {
            return None;
        }
        // A tela mudou de tamanho: o que estava guardado já não serve a ninguém.
        self.guardados.retain(|g| g.plano.len() == len);
        let i = self.guardados.iter().position(|g| g.sujo.chao == chao)?;
        let Guardado { mut plano, sujo } = self.guardados.swap_remove(i);
        #[cfg(test)]
        if NAO_REPOE.with(std::cell::Cell::get) {
            return Some(plano);
        }
        if let Some(r) = sujo.caixa {
            repoe(&mut plano, w, r, chao);
        }
        Some(plano)
    }
}

fn repoe(plano: &mut [u8], w: u32, r: Region, chao: u8) {
    let stride = w as usize * 4;
    let (x0, x1) = (r.x as usize * 4, (r.x + r.w) as usize * 4);
    for y in r.y as usize..(r.y + r.h) as usize {
        plano[y * stride + x0..y * stride + x1].fill(chao);
    }
}

/// N10 F11b: `PH2D_PILHA_PELOS_FIOS=1` = a abertura do B17.8 (`par_clone`/`par_fill`), lida uma vez (A×B).
pub(super) fn pilha_pelos_fios() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| std::env::var("PH2D_PILHA_PELOS_FIOS").is_ok_and(|v| v == "1"))
}

/// O `pre` da pilha: a tela inteira ANTES do traço.
pub(super) fn abre_o_pre(tela: &[u8]) -> Vec<u8> {
    if pilha_pelos_fios() {
        crate::plane_copy::par_clone(tela)
    } else {
        tela.to_vec()
    }
}
