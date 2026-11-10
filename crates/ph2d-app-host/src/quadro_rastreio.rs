//! B23.1 (PX-26, X-01): quem pediu cada quadro e quantas faltas de página menores o fio da UI pagou por
//! quadro. Liga com `PH2D_PAINT_PERF` ou `PH2D_FLUID_PROFILE` (os mesmos do `[frame]`); uma linha
//! `[quadro] quem:` por [`JANELA`] pedidos.

use super::Pedido;
use std::collections::BTreeMap;

/// Pedidos por linha impressa (a janela do `[frame]`).
pub const JANELA: u32 = 120;

/// A contagem de uma janela: o vencedor da lei (`Ja(<produtor>)`) ou a parada (`dorme:<porquê>`).
#[derive(Default)]
pub struct Rastreio {
    n: u32,
    conta: BTreeMap<String, u32>,
    minflt0: Option<u64>,
    /// As faltas partidas pelas fases do quadro ([`marca`]).
    pub fases: PorFase,
}

/// As fases do quadro do fio da UI, pela ordem em que a shell as [`marca`]: cada marca fecha o troço desde a
/// anterior. `Fora` abre o quadro (fecha o que correu entre quadros: o present no `Drop`, os eventos).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fase {
    Fora,
    Entrada,
    Cena,
    Sim,
    Desenho,
    Acquire,
    Present,
}

const FASES: [(Fase, &str); 7] = [
    (Fase::Fora, "fora"),
    (Fase::Entrada, "entrada"),
    (Fase::Cena, "cena"),
    (Fase::Sim, "sim"),
    (Fase::Desenho, "desenho"),
    (Fase::Acquire, "acquire"),
    (Fase::Present, "present"),
];

/// As somas das faltas por fase numa janela, e quantos quadros (`Fora`) a abriram.
#[derive(Default)]
pub struct PorFase {
    ultimo: Option<u64>,
    soma: [u64; FASES.len()],
    quadros: u32,
}

impl PorFase {
    /// O fio leu `minflt` ao fechar a fase `f`.
    pub fn marca(&mut self, f: Fase, minflt: u64) {
        if let Some(u) = self.ultimo {
            self.soma[f as usize] += minflt.saturating_sub(u);
        }
        self.ultimo = Some(minflt);
        self.quadros += u32::from(f == Fase::Fora);
    }

    /// `| por quadro desenhado (N): fora=… present=…` e zera a janela (vazio sem quadros).
    pub fn resumo(&mut self) -> String {
        let n = std::mem::take(&mut self.quadros);
        let soma = std::mem::take(&mut self.soma);
        if n == 0 {
            return String::new();
        }
        let partes: Vec<String> = FASES
            .iter()
            .map(|&(f, nome)| format!("{nome}={:.1}", soma[f as usize] as f64 / f64::from(n)))
            .collect();
        format!(" | por quadro desenhado ({n}): {}", partes.join(" "))
    }
}

impl Rastreio {
    /// Conta `p`; no fim da janela devolve a linha (com `minflt` = as faltas do fio desde a linha anterior).
    pub fn nota(&mut self, p: &Pedido, minflt: Option<u64>) -> Option<String> {
        let chave = match p {
            Pedido::Ja(q) => format!("{q:?}"),
            Pedido::Ate { porque, .. } => format!("dorme:{porque:?}"),
        };
        *self.conta.entry(chave).or_default() += 1;
        let inicio = *self.minflt0.get_or_insert(minflt.unwrap_or(0));
        self.n += 1;
        if self.n < JANELA {
            return None;
        }
        let quem: Vec<String> = self.conta.iter().map(|(k, v)| format!("{k}={v}")).collect();
        let faltas = minflt.map_or(0, |m| m.saturating_sub(inicio)) as f64 / f64::from(self.n);
        let linha = format!(
            "[quadro] quem: {} | minflt/quadro={faltas:.1} (fio da UI){}",
            quem.join(" "),
            self.fases.resumo()
        );
        *self = Self {
            minflt0: minflt,
            fases: std::mem::take(&mut self.fases),
            ..Self::default()
        };
        Some(linha)
    }
}

thread_local! {
    /// O `stat` DESTE fio, aberto uma vez: o `read_at` num buffer da pilha não aloca (nem paga faltas).
    static STAT: Option<std::fs::File> = std::fs::File::open("/proc/thread-self/stat").ok();
}

/// As faltas de página menores do fio que chama (`/proc/thread-self/stat`, campo 10).
#[cfg(unix)]
#[must_use]
pub fn minflt_do_fio() -> Option<u64> {
    use std::os::unix::fs::FileExt;
    let mut buf = [0u8; 1024];
    let n = STAT.with(|f| f.as_ref()?.read_at(&mut buf, 0).ok())?;
    let s = std::str::from_utf8(&buf[..n]).ok()?;
    s.rsplit_once(')')?
        .1
        .split_whitespace()
        .nth(7)?
        .parse()
        .ok()
}

#[cfg(not(unix))]
#[must_use]
pub fn minflt_do_fio() -> Option<u64> {
    None
}

thread_local! {
    static RASTREIO: std::cell::RefCell<Option<Rastreio>> = std::cell::RefCell::new(
        ["PH2D_PAINT_PERF", "PH2D_FLUID_PROFILE"]
            .iter()
            .any(|k| std::env::var(k).is_ok_and(|v| v != "0"))
            .then(Rastreio::default),
    );
}

/// A shell fecha a fase `f` do quadro; devolve o `Instant` do fecho (os relógios do `[frame]` armam nele).
/// Desligado custa um `borrow`.
pub fn marca(f: Fase) -> std::time::Instant {
    RASTREIO.with_borrow_mut(|r| {
        if let Some(r) = r.as_mut()
            && let Some(m) = minflt_do_fio()
        {
            r.fases.marca(f, m);
        }
    });
    std::time::Instant::now()
}

/// O laço chama isto com cada pedido; desligado custa um `borrow`.
pub(super) fn rastreia(p: &Pedido) {
    RASTREIO.with_borrow_mut(|r| {
        if let Some(linha) = r.as_mut().and_then(|r| r.nota(p, minflt_do_fio())) {
            eprintln!("{linha}");
        }
    });
}
