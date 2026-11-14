//! B21.1 — o ritmo do quadro quando a apresentação NÃO bloqueia (Mailbox/Immediate).
//!
//! Em `Fifo` o `acquire` espera o vblank e dá o ritmo; em Mailbox nada espera e o laço contínuo
//! correria sem tecto. A janela publica o refresco do monitor, a superfície publica se o modo
//! bloqueia, e o laço do quadro lê [`intervalo`]: o recurso que limita é o MONITOR.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;

static REFRESCO_MHZ: AtomicU32 = AtomicU32::new(0);
static NAO_BLOQUEIA: AtomicBool = AtomicBool::new(false);

/// O refresco do monitor da janela, em milihertz (`None` = o SO não o diz).
pub fn publica_refresco(mhz: Option<u32>) {
    REFRESCO_MHZ.store(mhz.unwrap_or(0), Ordering::Relaxed);
}

#[must_use]
pub fn refresco_mhz() -> Option<u32> {
    Some(REFRESCO_MHZ.load(Ordering::Relaxed)).filter(|m| *m > 0)
}

/// A superfície diz se o modo activo deixa de esperar o vblank.
pub fn publica_nao_bloqueia(sim: bool) {
    NAO_BLOQUEIA.store(sim, Ordering::Relaxed);
}

/// O período de um refresco; refresco desconhecido ⇒ 60 Hz (o monitor mais lento comum).
#[must_use]
pub fn periodo(mhz: Option<u32>) -> Duration {
    Duration::from_nanos(1_000_000_000_000 / u64::from(mhz.unwrap_or(60_000).max(1_000)))
}

/// O intervalo mínimo entre quadros: `None` quando o modo bloqueia (o `acquire` já dá o ritmo).
#[must_use]
pub fn intervalo() -> Option<Duration> {
    NAO_BLOQUEIA
        .load(Ordering::Relaxed)
        .then(|| periodo(refresco_mhz()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn o_periodo_e_o_do_monitor() {
        assert_eq!(periodo(Some(144_000)), Duration::from_nanos(6_944_444));
        assert_eq!(periodo(Some(60_000)), Duration::from_nanos(16_666_666));
        assert_eq!(periodo(None), periodo(Some(60_000)));
    }
}
