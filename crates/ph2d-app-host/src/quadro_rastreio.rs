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
            "[quadro] quem: {} | minflt/quadro={faltas:.1} (fio da UI)",
            quem.join(" ")
        );
        *self = Self {
            minflt0: minflt,
            ..Self::default()
        };
        Some(linha)
    }
}

/// As faltas de página menores do fio que chama (`/proc/thread-self/stat`, campo 10).
#[must_use]
pub fn minflt_do_fio() -> Option<u64> {
    let s = std::fs::read_to_string("/proc/thread-self/stat").ok()?;
    s.rsplit_once(')')?
        .1
        .split_whitespace()
        .nth(7)?
        .parse()
        .ok()
}

thread_local! {
    static RASTREIO: std::cell::RefCell<Option<Rastreio>> = std::cell::RefCell::new(
        ["PH2D_PAINT_PERF", "PH2D_FLUID_PROFILE"]
            .iter()
            .any(|k| std::env::var(k).is_ok_and(|v| v != "0"))
            .then(Rastreio::default),
    );
}

/// O laço chama isto com cada pedido; desligado custa um `borrow`.
pub(super) fn rastreia(p: &Pedido) {
    RASTREIO.with_borrow_mut(|r| {
        if let Some(linha) = r.as_mut().and_then(|r| r.nota(p, minflt_do_fio())) {
            eprintln!("{linha}");
        }
    });
}
