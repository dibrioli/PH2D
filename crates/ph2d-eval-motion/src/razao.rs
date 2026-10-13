//! **O razão da RAM** (doc 132 §2 (b)): o tecto dos caches do scrub é uma FRAÇÃO da RAM física,
//! lida uma vez, e PARTILHADO por todas as instâncias do processo — cem objectos não pedem cem
//! tectos. Um [`Razao`] por família (a fita das entradas; o anel de checkpoints da CPU); cada
//! membro declara o que retém por uma [`Conta`], e quem grava dobra/afina o SEU quando o total passa.
//!
//! A fração e a tabela que a escolheu: doc 132 §7 (P2).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock, OnceLock};

/// **Cada cache do scrub leva `1/32` da RAM física** — a fita e o anel juntos, `1/16`. Recurso: a
/// RAM física da máquina (8 GiB ⇒ 256 MiB cada; 128 GB ⇒ ~3,9 GiB cada). Tabela: doc 132 §7 (P2).
pub const FRACAO_DA_RAM: u64 = 32;

/// O tecto da fita pela linha de comando, em bytes (sufixos `k`/`m`/`g` aceites) — a cena `=139`
/// usa-o para a dobra chegar em segundos. Lido UMA vez, no arranque.
pub const VAR_DO_TECTO_DA_FITA: &str = "PH2D_MOTION_FITA_BYTES";

/// Um razão: o tecto e o que os membros declaram.
#[derive(Debug)]
pub struct Razao {
    tecto: usize,
    usado: AtomicUsize,
}

impl Razao {
    /// Um razão novo (os gates e as sondas usam um próprio, para não partilhar o do processo).
    #[must_use]
    pub fn new(tecto: usize) -> Arc<Self> {
        Arc::new(Self {
            tecto,
            usado: AtomicUsize::new(0),
        })
    }

    #[must_use]
    pub fn tecto(&self) -> usize {
        self.tecto
    }

    /// O que os membros vivos declaram agora.
    #[must_use]
    pub fn usado(&self) -> usize {
        self.usado.load(Ordering::Relaxed)
    }

    /// O razão de TODAS as fitas das entradas do processo.
    #[must_use]
    pub fn da_fita() -> Arc<Self> {
        static R: LazyLock<Arc<Razao>> = LazyLock::new(|| {
            let var = std::env::var(VAR_DO_TECTO_DA_FITA).ok();
            Razao::new(tecto_da_fita(var.as_deref(), ram_fisica()))
        });
        Arc::clone(&R)
    }

    /// O razão de TODOS os anéis de checkpoints da CPU do processo.
    #[must_use]
    pub fn dos_aneis() -> Arc<Self> {
        static R: LazyLock<Arc<Razao>> =
            LazyLock::new(|| Razao::new(tecto_pela_ram(ram_fisica(), crate::CPU_RING_BYTES)));
        Arc::clone(&R)
    }
}

/// A RAM física da máquina, lida uma vez — `None` onde não se sabe ler sem dependência nova.
#[must_use]
pub fn ram_fisica() -> Option<u64> {
    static RAM: OnceLock<Option<u64>> = OnceLock::new();
    *RAM.get_or_init(le_a_ram)
}

#[cfg(target_os = "linux")]
fn le_a_ram() -> Option<u64> {
    let m = std::fs::read_to_string("/proc/meminfo").ok()?;
    let kb: u64 = m
        .lines()
        .find_map(|l| l.strip_prefix("MemTotal:"))?
        .trim()
        .trim_end_matches("kB")
        .trim()
        .parse()
        .ok()?;
    kb.checked_mul(1024)
}

#[cfg(target_os = "macos")]
fn le_a_ram() -> Option<u64> {
    let out = std::process::Command::new("sysctl")
        .args(["-n", "hw.memsize"])
        .output()
        .ok()?;
    String::from_utf8(out.stdout).ok()?.trim().parse().ok()
}

/// ⚠️ Windows: o `GlobalMemoryStatusEx` pede o `windows-sys`, que esta crate não tem — vale a
/// reserva (o número de antes, 128 MiB por cache).
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn le_a_ram() -> Option<u64> {
    None
}

/// `ram / FRACAO_DA_RAM`, ou a `reserva` sem leitura.
#[must_use]
pub fn tecto_pela_ram(ram: Option<u64>, reserva: usize) -> usize {
    ram.and_then(|r| usize::try_from(r / FRACAO_DA_RAM).ok())
        .unwrap_or(reserva)
}

/// O tecto da fita: o da linha de comando se ela o diz, senão a fração da RAM.
#[must_use]
pub fn tecto_da_fita(var: Option<&str>, ram: Option<u64>) -> usize {
    var.and_then(bytes_de)
        .unwrap_or_else(|| tecto_pela_ram(ram, crate::FITA_BYTES))
}

/// `"262144"`, `"256k"`, `"4m"`, `"1g"` → bytes.
fn bytes_de(texto: &str) -> Option<usize> {
    let t = texto.trim().to_ascii_lowercase();
    let (n, mult) = match t.chars().last()? {
        'k' => (&t[..t.len() - 1], 1usize << 10),
        'm' => (&t[..t.len() - 1], 1 << 20),
        'g' => (&t[..t.len() - 1], 1 << 30),
        _ => (t.as_str(), 1),
    };
    n.trim().parse::<usize>().ok()?.checked_mul(mult)
}

/// O que UM membro declarou no seu razão; ao morrer, devolve-o.
#[derive(Debug)]
pub(crate) struct Conta {
    razao: Arc<Razao>,
    declarado: usize,
}

impl Conta {
    pub(crate) fn em(razao: Arc<Razao>) -> Self {
        Self {
            razao,
            declarado: 0,
        }
    }

    pub(crate) fn razao(&self) -> &Arc<Razao> {
        &self.razao
    }

    /// O membro retém agora `bytes`.
    pub(crate) fn declara(&mut self, bytes: usize) {
        let u = &self.razao.usado;
        if bytes >= self.declarado {
            u.fetch_add(bytes - self.declarado, Ordering::Relaxed);
        } else {
            u.fetch_sub(self.declarado - bytes, Ordering::Relaxed);
        }
        self.declarado = bytes;
    }

    /// O razão INTEIRO passou o tecto?
    pub(crate) fn acima(&self) -> bool {
        self.razao.usado() > self.razao.tecto
    }
}

impl Drop for Conta {
    fn drop(&mut self) {
        self.razao
            .usado
            .fetch_sub(self.declarado, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn o_tecto_e_a_fracao_da_ram_e_a_linha_de_comando_manda() {
        let gib = 1u64 << 30;
        assert_eq!(tecto_pela_ram(Some(8 * gib), 7), 256 << 20);
        assert_eq!(tecto_pela_ram(None, 7), 7, "sem leitura, a reserva");
        assert_eq!(tecto_da_fita(Some("256k"), Some(8 * gib)), 256 << 10);
        assert_eq!(tecto_da_fita(Some(" 3M "), None), 3 << 20);
        assert_eq!(tecto_da_fita(Some("1000"), None), 1000);
        assert_eq!(
            tecto_da_fita(Some("muito"), Some(8 * gib)),
            256 << 20,
            "o que não se entende não vale"
        );
    }

    #[test]
    fn uma_conta_que_morre_devolve_o_que_declarou() {
        let r = Razao::new(100);
        let mut a = Conta::em(Arc::clone(&r));
        let mut b = Conta::em(Arc::clone(&r));
        a.declara(60);
        b.declara(30);
        assert_eq!(r.usado(), 90);
        assert!(!a.acima());
        b.declara(50);
        assert!(a.acima() && b.acima(), "o tecto é do razão, não da conta");
        a.declara(10);
        assert_eq!(r.usado(), 60);
        drop(b);
        assert_eq!(r.usado(), 10);
    }

    /// A leitura desta máquina (Linux): existe e é plausível.
    #[cfg(target_os = "linux")]
    #[test]
    fn a_ram_le_se_no_linux() {
        let r = ram_fisica().expect("MemTotal");
        assert!(r > 512 << 20, "{r}");
    }
}
