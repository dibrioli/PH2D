//! O código curto que o anfitrião mostra e quem entra escreve, e o travão da força bruta.

use std::time::{Duration, Instant};

use rand_core::{OsRng, RngCore};

/// Sem os ambíguos (`0`/`O`, `1`/`I`/`L`): 31 símbolos.
pub const ALPHABET: &[u8; 31] = b"23456789ABCDEFGHJKMNPQRSTUVWXYZ";
pub const CODE_LEN: usize = 6;

/// Códigos errados SEGUIDOS antes do bloqueio.
pub const MAX_FAILS: u32 = 5;
/// O bloqueio. Com PAKE a única força bruta possível é ONLINE (uma tentativa por ligação):
/// 31⁶ ≈ 8,9e8 códigos (~29,7 bits) a 5 por 30 s = 10 por minuto ⇒ ~85 anos para metade do espaço.
pub const LOCKOUT: Duration = Duration::from_secs(30);

/// Um código sorteado (6 símbolos do [`ALPHABET`], maiúsculas, sem hífen).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Code([u8; CODE_LEN]);

impl Code {
    pub(crate) fn random() -> Self {
        let mut out = [0u8; CODE_LEN];
        let mut i = 0;
        while i < CODE_LEN {
            let mut b = [0u8; 16];
            OsRng.fill_bytes(&mut b);
            // Rejeição: só bytes abaixo de 31·8 = 248, para o resto não favorecer símbolo nenhum.
            for &x in b.iter().filter(|&&x| x < 248) {
                if i == CODE_LEN {
                    break;
                }
                out[i] = ALPHABET[usize::from(x % 31)];
                i += 1;
            }
        }
        Self(out)
    }

    /// O que a pessoa escreveu: minúsculas, espaços e hífens aceites.
    pub(crate) fn parse(text: &str) -> Option<Self> {
        let mut out = [0u8; CODE_LEN];
        let mut i = 0;
        for c in text.chars().filter(|c| !c.is_whitespace() && *c != '-') {
            let c = u8::try_from(c.to_ascii_uppercase()).ok()?;
            if i == CODE_LEN || !ALPHABET.contains(&c) {
                return None;
            }
            out[i] = c;
            i += 1;
        }
        (i == CODE_LEN).then_some(Self(out))
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        &self.0
    }

    /// `ABC-DEF`.
    pub(crate) fn display(&self) -> String {
        let s = |r: std::ops::Range<usize>| String::from_utf8_lossy(&self.0[r]).into_owned();
        format!("{}-{}", s(0..3), s(3..6))
    }
}

/// O travão: [`MAX_FAILS`] errados seguidos ⇒ [`LOCKOUT`]. Os apertos de mão EM CURSO contam como
/// errados até acabarem — sem isso, 100 ligações em paralelo eram 100 palpites antes do 1.º erro.
#[derive(Debug, Default)]
pub(crate) struct Gate {
    fails: u32,
    inflight: u32,
    until: Option<Instant>,
}

impl Gate {
    /// Pode começar um aperto de mão? `Err` = quanto falta.
    pub(crate) fn begin(&mut self, now: Instant) -> Result<(), Duration> {
        if let Some(until) = self.until {
            if now < until {
                return Err(until - now);
            }
            self.until = None;
            self.fails = 0;
        }
        if self.fails + self.inflight >= MAX_FAILS {
            // Só apertos em curso: acabam (ou expiram) depressa.
            return Err(Duration::from_secs(1));
        }
        self.inflight += 1;
        Ok(())
    }

    /// Acabou um aperto começado por [`Gate::begin`].
    pub(crate) fn end(&mut self, ok: bool, now: Instant) {
        self.inflight = self.inflight.saturating_sub(1);
        if ok {
            self.fails = 0;
        } else {
            self.fails += 1;
            if self.fails >= MAX_FAILS {
                self.until = Some(now + LOCKOUT);
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn fails(&self) -> u32 {
        self.fails
    }

    /// Um aperto que não chegou a testar o código (lixo, versão): não é palpite.
    pub(crate) fn abandon(&mut self) {
        self.inflight = self.inflight.saturating_sub(1);
    }
}
