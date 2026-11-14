//! **A sessão do Wet que pode estar com o operário do depósito** (N10 F2d) — o campo
//! `WetPaintState::session` (filho de [`super`]). O padrão do `EngineSlot`: quem lê a sessão traz o lote
//! em voo de volta primeiro ([`super::dab_route`]'s `fila_do_dab`), então nenhum dos ~400 acessos vê uma
//! sessão a meio do depósito nem a «perde» enquanto ela está fora.
//!
//! ⚠️ Os leitores `&self` (os painéis, o `Tool` congelado do §6) também drenam: a sessão mora numa `OnceLock`
//! que o dreno preenche por `&self`, e o lote em voo num `Mutex` — interior, sem `unsafe`, sem mudar uma
//! assinatura. Fora de voo o custo é um `get` atómico. Desenho e números: `docs/Painter/missao_4k/medicoes/N10_cerdas_mistura.md`.

use super::WetSession;
use super::dab_route::fila_do_dab::{self, Composta, EmVoo};
use super::session::LoteAdiado;
use std::sync::{Mutex, OnceLock, PoisonError};

#[cfg(test)]
thread_local! {
    /// Quantos lotes voaram neste fio (o gate prova que a fila adiada correu).
    pub(in crate::tool::paint) static VOOS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    /// Quantos lotes ficaram ATRÁS de um voo neste fio (F2f: o gate prova que o encadeado correu).
    pub(in crate::tool::paint) static ATRAS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    /// Quantas ópticas do operário o fio da tela colou (F2h: o gate prova que o quadro não compôs a óptica delas).
    pub(in crate::tool::paint) static COLADAS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// A sessão viva: em casa, com o operário, ou nenhuma.
#[derive(Default)]
pub(in crate::tool::paint) struct SessaoDoWet {
    aqui: OnceLock<WetSession>,
    voo: Mutex<Option<EmVoo>>,
    /// Os lotes que chegaram com a sessão em voo (F2f), na ordem da mão: o dreno põe-nos no FIM de
    /// `WetSession::adiados`, e a rota pousa-os um por oportunidade (evento ou tique) sem esperar o operário.
    atras: Mutex<Vec<LoteAdiado>>,
    /// A `WetSession::marca` da sessão em voo (o operário não a muda): a rota lê-a sem drenar.
    marca_em_voo: Option<u64>,
    /// N10 F2h: as ópticas que os voos compuseram no operário, por ordem; o próximo composite corre o laço de píxel
    /// delas ([`super::por_quadro`]). Morrem com a sessão.
    compostas: Mutex<Vec<Composta>>,
}

impl SessaoDoWet {
    /// Uma sessão em casa.
    pub(in crate::tool::paint) fn aqui(s: WetSession) -> Self {
        Self {
            aqui: OnceLock::from(s),
            voo: Mutex::new(None),
            atras: Mutex::new(Vec::new()),
            marca_em_voo: None,
            compostas: Mutex::new(Vec::new()),
        }
    }

    /// A sessão sai ao operário ([`fila_do_dab::enfileira`]); a próxima leitura espera-a.
    pub(in crate::tool::paint) fn voa(&mut self, voo: EmVoo, marca: Option<u64>) {
        self.marca_em_voo = marca;
        debug_assert!(
            self.aqui.get().is_none(),
            "a sessão voa sem estar fora de casa"
        );
        debug_assert!(
            self.atras
                .get_mut()
                .unwrap_or_else(PoisonError::into_inner)
                .is_empty(),
            "a sessão voa com lotes atrás de um voo anterior"
        );
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

    /// F2f: com um lote ainda com o operário, `lote()` fica ATRÁS dele (`true`) e o evento volta sem esperar;
    /// sem voo, ou com o voo já pronto, nada muda (`false`) e a rota segue (drenando, sem espera).
    pub(in crate::tool::paint) fn fica_atras(&mut self, lote: impl FnOnce() -> LoteAdiado) -> bool {
        let voo = self.voo.get_mut().unwrap_or_else(PoisonError::into_inner);
        if !voo.as_mut().is_some_and(|v| !v.pronto()) {
            return false;
        }
        #[cfg(test)]
        ATRAS.set(ATRAS.get() + 1);
        self.atras
            .get_mut()
            .unwrap_or_else(PoisonError::into_inner)
            .push(lote());
        true
    }

    /// F2f: os lotes que ainda faltam de uma vez de adiados vão atrás do voo que acabou de sair (antes de qualquer
    /// lote de evento: `atras` está vazio, a sessão estava em casa). Sem voo, voltam a `adiados`.
    pub(in crate::tool::paint) fn resto_atras(&mut self, resto: Vec<LoteAdiado>) {
        if resto.is_empty() {
            return;
        }
        if self.em_voo() {
            let a = self.atras.get_mut().unwrap_or_else(PoisonError::into_inner);
            debug_assert!(
                a.is_empty(),
                "o resto dos adiados passaria à frente de um lote de evento"
            );
            let novos = std::mem::replace(a, resto);
            a.extend(novos);
        } else if let Some(s) = self.as_mut() {
            s.adiados.extend(resto);
        }
    }

    /// A marca de undo da sessão, em casa ou em voo, SEM drenar (F2f). `None` = sem sessão.
    pub(in crate::tool::paint) fn marca(&self) -> Option<Option<u64>> {
        if let Some(s) = self.aqui.get() {
            return Some(s.marca);
        }
        self.em_voo().then_some(self.marca_em_voo)
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
    #[track_caller]
    fn drena(&self) {
        let mut voo = self.voo.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(v) = voo.take() {
            let t0 = std::time::Instant::now();
            let dep = fila_do_dab::drena_os_dabs(v);
            diag_espera("dreno", t0, std::panic::Location::caller());
            let mut sess = dep.sess;
            if let Some(s) = dep.saida {
                self.compostas
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .push(s);
            }
            sess.adiados
                .append(&mut self.atras.lock().unwrap_or_else(PoisonError::into_inner));
            if self.aqui.set(sess).is_err() {
                unreachable!("um lote voltou com a sessão em casa");
            }
        }
    }

    #[track_caller]
    pub(in crate::tool::paint) fn as_ref(&self) -> Option<&WetSession> {
        if self.aqui.get().is_none() {
            self.drena();
        }
        self.aqui.get()
    }

    #[track_caller]
    pub(in crate::tool::paint) fn as_mut(&mut self) -> Option<&mut WetSession> {
        if self.aqui.get().is_none() {
            self.drena();
        }
        self.aqui.get_mut()
    }

    #[track_caller]
    pub(in crate::tool::paint) fn take(&mut self) -> Option<WetSession> {
        if self.aqui.get().is_none() {
            self.drena();
        }
        self.compostas
            .get_mut()
            .unwrap_or_else(PoisonError::into_inner)
            .clear();
        self.aqui.take()
    }

    /// F2h: as ópticas compostas no operário por colar (a sessão já em casa: quem chama trouxe-a).
    pub(in crate::tool::paint) fn tem_compostas(&self) -> bool {
        !self
            .compostas
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_empty()
    }

    pub(super) fn tira_as_compostas(&mut self) -> Vec<Composta> {
        std::mem::take(
            self.compostas
                .get_mut()
                .unwrap_or_else(PoisonError::into_inner),
        )
    }

    pub(in crate::tool::paint) fn is_some(&self) -> bool {
        self.aqui.get().is_some() || self.em_voo()
    }

    pub(in crate::tool::paint) fn is_none(&self) -> bool {
        !self.is_some()
    }
}

/// ⚠️ Medição, nunca produto (F2g): com `PH2D_QUADRO_LENTO_MS` posto, cada espera do fio da UI > 5 ms imprime
/// `[wet-espera] <o quê> ms=<n> em <ficheiro:linha>` — o `[quadro-lento]` diz QUE o quadro parou, isto diz ONDE.
pub(in crate::tool::paint) fn diag_espera(
    oque: &str,
    t0: std::time::Instant,
    onde: &std::panic::Location<'_>,
) {
    static ON: OnceLock<bool> = OnceLock::new();
    if !*ON.get_or_init(|| std::env::var_os("PH2D_QUADRO_LENTO_MS").is_some()) {
        return;
    }
    let ms = t0.elapsed().as_secs_f64() * 1e3;
    if ms > 5.0 {
        eprintln!("[wet-espera] {oque} ms={ms:.1} em {onde}");
    }
}

/// ⚠️ Medição (F2h, a régua da latência caneta→tinta): com `PH2D_QUADRO_LENTO_MS` posto, cada óptica do operário
/// colada na tela imprime `[wet-latencia] ms=<n>` — do evento da caneta que pousou o lote ao quadro que o mostra.
pub(in crate::tool::paint) fn diag_latencia(evento: std::time::Instant) {
    static ON: OnceLock<bool> = OnceLock::new();
    if *ON.get_or_init(|| std::env::var_os("PH2D_QUADRO_LENTO_MS").is_some()) {
        eprintln!(
            "[wet-latencia] ms={:.1}",
            evento.elapsed().as_secs_f64() * 1e3
        );
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
