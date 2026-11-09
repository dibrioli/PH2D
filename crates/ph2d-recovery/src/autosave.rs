//! O relógio da cópia: a cada [`Period`], **só se os bytes mudaram** desde a última cópia (ou
//! desde o último Save).
//!
//! ⚠️ **A regra «mudou» é a impressão dos BYTES do Save, não um carimbo do undo.** O projecto tem
//! várias histórias (a fila por diff da cena, a do Pixel, a do Painter, a dos quadros e abas) e
//! nenhum carimbo único as cobre; os bytes do Save cobrem-nas todas, por definição.
//!
//! ⚠️ **A impressão e a escrita vão num fio à parte** ([`Autosave::offer`] só entrega o `Vec`):
//! `write_atomic` de 64 MB = 40–44 ms (btrfs/NVMe, `a_escrita_de_um_projeto_grande_custa`) e a
//! impressão SipHash de 128 MiB = 33 ms (`a_serializacao_de_um_projeto_grande_custa`), ambos acima
//! dos 16 ms de um quadro. Medido no MESMO processo, intercalado, mínimo de 5 (`--profile smoke`,
//! `o_fio_da_janela_so_entrega_os_bytes`, 08/11; P12_plano §4.3):
//!
//! | 128 MiB, no fio da janela (load ~6) | antes (impressão + entrega) | depois (só entrega) |
//! |---|---|---|
//! | mínimo de 5, corrida 1 | 32,5 ms | 35 µs |
//! | mínimo de 5, corrida 2 | 34,1 ms | 43 µs |
//!
//! Fica no fio da janela o codec do Pixel (~80 ms) e o `postcard` do `ProjectFile` (~23 ms), só na
//! primeira PAUSA do artista depois da hora ([`crate::Recovery::due`]).

use std::hash::{Hash, Hasher};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::Session;
use crate::session::Meta;

/// O intervalo, nos valores do Aseprite (O15): 10 s, 30 s, 1, 2, 5, 10, 15, 30 min.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Period(Duration);

impl Period {
    /// As opções do O15, em segundos.
    pub const OPTIONS_SECS: [u64; 8] = [10, 30, 60, 120, 300, 600, 900, 1800];

    /// Desligada (`recovery.minutes=0`, Settings ▸ Recovery ▸ Off): nunca é hora.
    pub const OFF: Self = Self(Duration::ZERO);

    /// A opção de `secs` segundos (`0` = [`Self::OFF`]; fora das opções = a mais próxima).
    #[must_use]
    pub fn from_secs(secs: u64) -> Self {
        if secs == 0 {
            return Self::OFF;
        }
        let near = |x: u64| x.abs_diff(secs);
        let best = Self::OPTIONS_SECS.iter().copied().min_by_key(|&x| near(x));
        Self(Duration::from_secs(best.unwrap_or(120)))
    }

    /// O intervalo (`ZERO` = desligada).
    #[must_use]
    pub fn duration(self) -> Duration {
        self.0
    }

    /// Desligada?
    #[must_use]
    pub fn is_off(self) -> bool {
        self.0.is_zero()
    }

    /// `recovery.minutes=<n>` de `prefs.txt` (fracção aceita: `0.1667` = 10 s, como o `.ini` do
    /// Aseprite): o valor vai à opção mais próxima; `0` = [`Self::OFF`]; ilegível = a de fábrica.
    #[must_use]
    pub fn from_minutes(text: &str) -> Self {
        let Ok(min) = text.trim().parse::<f64>() else {
            return Self::default();
        };
        if min == 0.0 {
            return Self::OFF;
        }
        if !min.is_finite() || min <= 0.0 {
            return Self::default();
        }
        let secs = min * 60.0;
        let best = Self::OPTIONS_SECS
            .iter()
            .copied()
            .min_by(|a, b| {
                let d = |x: u64| ((x as f64) - secs).abs();
                d(*a).total_cmp(&d(*b))
            })
            .unwrap_or(120);
        Self(Duration::from_secs(best))
    }

    /// O texto de `recovery.minutes`.
    #[must_use]
    pub fn to_minutes(self) -> String {
        let m = self.0.as_secs_f64() / 60.0;
        if m.fract() == 0.0 {
            format!("{m:.0}")
        } else {
            format!("{m:.4}")
        }
    }
}

impl Default for Period {
    fn default() -> Self {
        Self(Duration::from_secs(120))
    }
}

/// O que o fio fez com os bytes que recebeu (devolvido por [`Autosave::finish`]).
#[derive(Debug, PartialEq, Eq)]
pub enum Offer {
    /// Iguais à última cópia (ou ao último Save): nada se escreveu.
    Unchanged,
    /// A cópia nova está no disco.
    Written,
    /// A escrita (ou a serialização) falhou: o trabalho continua por copiar.
    Failed(String),
}

/// O que o fio devolve: a impressão dos bytes e se os escreveu.
type Job = JoinHandle<std::io::Result<(u64, bool)>>;

/// Uma impressão de vários sinais baratos ([`crate::Recovery::pulse`]).
#[must_use]
pub fn fold(parts: impl IntoIterator<Item = u64>) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    parts.into_iter().for_each(|p| p.hash(&mut h));
    h.finish()
}

/// A impressão dos bytes (o «mudou?» do relógio e do fecho).
#[must_use]
pub fn print_of(bytes: &[u8]) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut h);
    h.finish()
}

/// O relógio e a escrita em curso.
#[derive(Debug)]
pub struct Autosave {
    /// O intervalo.
    pub period: Period,
    next: Instant,
    last: Option<u64>,
    /// A impressão do que está no disco do artista (o último Save ou Open); `None` = por gravar.
    saved: Option<u64>,
    writing: Option<Job>,
}

impl Autosave {
    /// Começa a contar a partir de `now`.
    #[must_use]
    pub fn new(period: Period, now: Instant) -> Self {
        Self {
            period,
            next: now + period.duration(),
            last: None,
            saved: None,
            writing: None,
        }
    }

    /// Chegou a hora (e não há escrita em curso)? Só então a shell serializa.
    #[must_use]
    pub fn due(&self, now: Instant) -> bool {
        !self.period.is_off()
            && now >= self.next
            && self.writing.as_ref().is_none_or(JoinHandle::is_finished)
    }

    /// Passou o prazo duro (2 × o intervalo desde a última hora): grava mesmo sem pausa do artista.
    #[must_use]
    pub fn overdue(&self, now: Instant) -> bool {
        now >= self.next + self.period.duration()
    }

    /// A hora passou sem nada por copiar: a próxima é daqui a um intervalo.
    pub fn postpone(&mut self, now: Instant) {
        self.next = now + self.period.duration();
    }

    /// Muda o intervalo agora (Settings ▸ Recovery): a próxima hora conta a partir de `now`.
    pub fn set_period(&mut self, period: Period, now: Instant) {
        if period != self.period {
            self.period = period;
            self.next = now + period.duration();
        }
    }

    /// O Save gravou estes bytes: lembra a impressão
    /// (o disco do artista passou a ser a verdade) e apaga a cópia.
    pub fn mark_saved(&mut self, bytes: &[u8], session: &Session, now: Instant) {
        self.finish();
        self.last = Some(print_of(bytes));
        self.saved = self.last;
        self.next = now + self.period.duration();
        session.clear_copy();
    }

    /// Abriu-se um projecto com estes bytes: a próxima hora só escreve se mudarem. `restored` = veio
    /// de uma cópia (fica por gravar); senão é o disco do artista.
    pub fn opened(&mut self, bytes: &[u8], restored: bool, now: Instant) {
        self.finish();
        self.last = Some(print_of(bytes));
        self.saved = if restored { None } else { self.last };
        self.next = now + self.period.duration();
    }

    /// Estes bytes diferem do que está no disco do artista (ou o projecto nunca foi gravado)?
    #[must_use]
    pub fn is_unsaved(&self, bytes: &[u8]) -> bool {
        self.saved != Some(print_of(bytes))
    }

    /// Esquece a impressão (o projecto restaurado está por gravar: a próxima hora escreve).
    pub fn forget(&mut self) {
        self.last = None;
    }

    /// A hora chegou e a shell serializou: ENTREGA os bytes ao fio, que tira a impressão e só
    /// escreve se mudaram. No fio da janela não se lê um byte (o resultado vem em [`Self::finish`]).
    pub fn offer(&mut self, bytes: Vec<u8>, meta: Meta, session: &Session, now: Instant) {
        self.offer_with(move || Some(bytes), meta, session, now);
    }

    /// [`Self::offer`] com os bytes ainda por JUNTAR: `make` corre no fio da escrita (P12 S — a
    /// serialização do projecto sai do fio da janela). `None` = recusado, nada se grava.
    pub fn offer_with<F>(&mut self, make: F, meta: Meta, session: &Session, now: Instant)
    where
        F: FnOnce() -> Option<Vec<u8>> + Send + 'static,
    {
        self.next = now + self.period.duration();
        self.finish();
        let last = self.last;
        let dir = session.dir().to_path_buf();
        self.writing = Some(std::thread::spawn(move || {
            let bytes =
                make().ok_or_else(|| std::io::Error::other("the project did not serialize"))?;
            let p = print_of(&bytes);
            if last == Some(p) {
                return Ok((p, false));
            }
            crate::session::write_copy_at(&dir, &bytes, &meta).map(|()| (p, true))
        }));
    }

    /// O fio da escrita acabou (há um resultado por colher sem esperar)?
    #[must_use]
    pub fn finished(&self) -> bool {
        self.writing.as_ref().is_some_and(JoinHandle::is_finished)
    }

    /// Espera o fio em curso (antes de apagar a cópia, e no fecho) e diz o que ele fez.
    pub fn finish(&mut self) -> Option<Offer> {
        let h = self.writing.take()?;
        match h.join() {
            Ok(Ok((p, wrote))) => {
                self.last = Some(p);
                Some(if wrote {
                    Offer::Written
                } else {
                    Offer::Unchanged
                })
            }
            // Falhou: esquece a impressão, para a próxima hora tentar de novo.
            Ok(Err(e)) => {
                eprintln!("[recovery] a cópia não se gravou: {e}");
                self.last = None;
                Some(Offer::Failed(e.to_string()))
            }
            Err(_) => {
                eprintln!("[recovery] o fio da cópia caiu");
                self.last = None;
                Some(Offer::Failed("the writer thread panicked".to_owned()))
            }
        }
    }
}
