//! **A sessão do Wet que pode estar com o operário do depósito** (N10 F2d) — o campo
//! `WetPaintState::session` (filho de [`super`]). O padrão do `EngineSlot`: quem lê a sessão traz o lote
//! em voo de volta primeiro ([`super::dab_route`]'s `fila_do_dab`), então nenhum dos ~400 acessos vê uma
//! sessão a meio do depósito nem a «perde» enquanto ela está fora.
//!
//! ⚠️ Os leitores `&self` (os painéis, o `Tool` congelado do §6) também drenam: a sessão mora numa `OnceLock`
//! que o dreno preenche por `&self`, e o lote em voo num `Mutex` — interior, sem `unsafe`, sem mudar uma
//! assinatura. Fora de voo o custo é um `get` atómico. Desenho e números: `docs/Painter/missao_4k/medicoes/N10_cerdas_mistura.md`.

use super::WetSession;
use super::dab_route::fila_do_dab::{self, EmVoo};
use std::sync::{Mutex, OnceLock, PoisonError};

#[cfg(test)]
thread_local! {
    /// Quantos lotes voaram neste fio (o gate prova que a fila adiada correu).
    pub(in crate::tool::paint) static VOOS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// A sessão viva: em casa, com o operário, ou nenhuma.
#[derive(Default)]
pub(in crate::tool::paint) struct SessaoDoWet {
    aqui: OnceLock<WetSession>,
    voo: Mutex<Option<EmVoo>>,
}

impl SessaoDoWet {
    /// Uma sessão em casa.
    pub(in crate::tool::paint) fn aqui(s: WetSession) -> Self {
        Self {
            aqui: OnceLock::from(s),
            voo: Mutex::new(None),
        }
    }

    /// A sessão sai ao operário ([`fila_do_dab::enfileira`]); a próxima leitura espera-a.
    pub(in crate::tool::paint) fn voa(&mut self, voo: EmVoo) {
        debug_assert!(self.aqui.get().is_none(), "a sessão voa sem estar fora de casa");
        #[cfg(test)]
        VOOS.set(VOOS.get() + 1);
        *self.voo.get_mut().unwrap_or_else(PoisonError::into_inner) = Some(voo);
    }

    /// `true` = há um lote com o operário.
    pub(in crate::tool::paint) fn em_voo(&self) -> bool {
        self.voo
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_some()
    }

    /// Traz o lote de volta SEM esperar: `true` = a sessão está em casa (ou não há nenhuma em voo).
    pub(in crate::tool::paint) fn assentou(&mut self) -> bool {
        let voo = self.voo.get_mut().unwrap_or_else(PoisonError::into_inner);
        let Some(v) = voo.as_mut() else {
            return true;
        };
        if !v.pronto() {
            return false;
        }
        self.drena();
        true
    }

    /// A ÚNICA espera (Bug #66: nunca num job do rayon — `drena_os_dabs` confere).
    fn drena(&self) {
        let mut voo = self.voo.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(v) = voo.take() {
            let dep = fila_do_dab::drena_os_dabs(v);
            if self.aqui.set(dep.sess).is_err() {
                unreachable!("um lote voltou com a sessão em casa");
            }
        }
    }

    pub(in crate::tool::paint) fn as_ref(&self) -> Option<&WetSession> {
        if self.aqui.get().is_none() {
            self.drena();
        }
        self.aqui.get()
    }

    pub(in crate::tool::paint) fn as_mut(&mut self) -> Option<&mut WetSession> {
        if self.aqui.get().is_none() {
            self.drena();
        }
        self.aqui.get_mut()
    }

    pub(in crate::tool::paint) fn take(&mut self) -> Option<WetSession> {
        if self.aqui.get().is_none() {
            self.drena();
        }
        self.aqui.take()
    }

    pub(in crate::tool::paint) fn is_some(&self) -> bool {
        self.aqui.get().is_some() || self.em_voo()
    }

    pub(in crate::tool::paint) fn is_none(&self) -> bool {
        !self.is_some()
    }
}

impl From<Option<WetSession>> for SessaoDoWet {
    fn from(s: Option<WetSession>) -> Self {
        s.map_or_else(Self::default, Self::aqui)
    }
}

/// Copiar drena primeiro: a cópia é sempre de uma sessão em casa.
impl Clone for SessaoDoWet {
    fn clone(&self) -> Self {
        self.as_ref().cloned().into()
    }
}
