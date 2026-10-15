//! **Os razões da RAM do Motion** (doc 132 §2 (b)): a lei — o tecto é uma fração da RAM física,
//! partilhado pelo processo — é a de [`ph2d_core::razao`]; aqui moram os DOIS razões desta família (a
//! fita das entradas; o anel de checkpoints da CPU), cada um com `1/32`, e o tecto da fita pela linha de
//! comando.

pub use ph2d_core::razao::{Conta, FRACAO_DA_RAM, Razao, ram_fisica, tecto_pela_ram};
use std::sync::{Arc, LazyLock};

/// O tecto da fita pela linha de comando, em bytes (sufixos `k`/`m`/`g` aceites) — a cena `=139`
/// usa-o para a dobra chegar em segundos. Lido UMA vez, no arranque.
pub const VAR_DO_TECTO_DA_FITA: &str = "PH2D_MOTION_FITA_BYTES";

/// O razão de TODAS as fitas das entradas do processo.
#[must_use]
pub fn da_fita() -> Arc<Razao> {
    static R: LazyLock<Arc<Razao>> = LazyLock::new(|| {
        let var = std::env::var(VAR_DO_TECTO_DA_FITA).ok();
        Razao::new(tecto_da_fita(var.as_deref(), ram_fisica()))
    });
    Arc::clone(&R)
}

/// O razão de TODOS os anéis de checkpoints da CPU do processo.
#[must_use]
pub fn dos_aneis() -> Arc<Razao> {
    static R: LazyLock<Arc<Razao>> =
        LazyLock::new(|| Razao::new(tecto_pela_ram(ram_fisica(), crate::CPU_RING_BYTES)));
    Arc::clone(&R)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_linha_de_comando_manda_no_tecto_da_fita() {
        let gib = 1u64 << 30;
        assert_eq!(tecto_da_fita(Some("256k"), Some(8 * gib)), 256 << 10);
        assert_eq!(tecto_da_fita(Some(" 3M "), None), 3 << 20);
        assert_eq!(tecto_da_fita(Some("1000"), None), 1000);
        assert_eq!(
            tecto_da_fita(Some("muito"), Some(8 * gib)),
            256 << 20,
            "o que não se entende não vale"
        );
        assert_eq!(
            tecto_da_fita(None, Some(8 * gib)),
            256 << 20,
            "sem linha, a fração"
        );
    }
}
