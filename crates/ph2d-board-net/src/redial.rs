//! ⭐ **J7 — a lei de voltar depois de cair**, UMA para as duas salas (a do Quadro e a do Kanban
//! sozinho): a 1.ª tentativa a [`RETRY_FIRST`], depois o dobro de cada vez até [`RETRY_MAX`]; passado
//! [`RECONNECT_FOR`] desiste. Porquê estes números: `docs/TrelloClone/loop/MEDICOES.md` §J4.

use std::time::{Duration, Instant};

/// A 1.ª espera: um soluço da rede local reabre em menos de um RTT de TCP + o aperto de mão.
pub const RETRY_FIRST: Duration = Duration::from_millis(250);
/// O tecto da espera: no minuto são ≤ 12 tentativas, cada uma um fio (e, por link, até 30 s).
pub const RETRY_MAX: Duration = Duration::from_secs(8);
/// Quanto se tenta voltar: o Wi-Fi da casa que soluça volta em segundos; o portátil fechado não.
pub const RECONNECT_FOR: Duration = Duration::from_secs(60);

/// A espera antes da tentativa `n` (0 = a primeira).
#[must_use]
pub fn backoff(n: u32) -> Duration {
    RETRY_FIRST
        .saturating_mul(1u32.checked_shl(n.min(16)).unwrap_or(u32::MAX))
        .min(RETRY_MAX)
}

/// Uma ligação caída a voltar: desde quando, quantas tentativas e quando a próxima.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Redial {
    since: Instant,
    tries: u32,
    next: Instant,
}

impl Redial {
    /// Caiu agora (`now`): a 1.ª tentativa daqui a [`backoff`]`(0)`.
    #[must_use]
    pub fn new(now: Instant) -> Self {
        Self {
            since: now,
            tries: 0,
            next: now + backoff(0),
        }
    }

    /// Passou o minuto: desiste.
    #[must_use]
    pub fn expired(&self, now: Instant) -> bool {
        now.saturating_duration_since(self.since) > RECONNECT_FOR
    }

    /// `true` = tente agora (e a seguinte fica marcada com o dobro da espera).
    pub fn due(&mut self, now: Instant) -> bool {
        if now < self.next {
            return false;
        }
        self.tries = self.tries.saturating_add(1);
        self.next = now + backoff(self.tries);
        true
    }

    /// As tentativas já feitas.
    #[must_use]
    pub fn tries(&self) -> u32 {
        self.tries
    }
}

/// A voltar: a porta RECUSADA (a máquina responde, ninguém ouve — a sala fechou) ou um código
/// errado/travado (ali abriu OUTRA sala) dizem que a sala já não é esta; o resto pode ser o Wi-Fi.
#[must_use]
pub fn room_is_gone(e: &crate::JoinError) -> bool {
    matches!(
        e,
        crate::JoinError::WrongCode
            | crate::JoinError::Locked(_)
            | crate::JoinError::Unreachable(std::io::ErrorKind::ConnectionRefused)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O retrocesso: 250 ms, depois o dobro, até ao tecto (e nunca transborda).
    #[test]
    fn the_backoff_doubles_up_to_its_ceiling() {
        let ms = |n| backoff(n).as_millis();
        assert_eq!(
            [ms(0), ms(1), ms(3), ms(5), ms(40), ms(u32::MAX)],
            [250, 500, 2000, 8000, 8000, 8000]
        );
    }

    /// ⭐ O relógio de uma queda: a 1.ª tentativa aos 250 ms, as seguintes a dobrar, ≤ 12 no minuto,
    /// e desiste depois dele.
    #[test]
    fn a_redial_tries_at_the_doubling_waits_and_gives_up_after_the_minute() {
        let t0 = Instant::now();
        let mut r = Redial::new(t0);
        assert!(!r.due(t0), "logo ao cair ainda não");
        let at = |ms| t0 + Duration::from_millis(ms);
        let mut times = Vec::new();
        let mut ms = 0;
        while ms <= 60_000 {
            if r.due(at(ms)) {
                times.push(ms);
            }
            ms += 50;
        }
        assert_eq!(&times[..4], &[250, 750, 1750, 3750], "{times:?}");
        assert!(times.len() <= 12, "{} tentativas no minuto", times.len());
        assert_eq!(r.tries() as usize, times.len());
        assert!(!r.expired(at(60_000)));
        assert!(r.expired(at(60_001)));
    }
}
