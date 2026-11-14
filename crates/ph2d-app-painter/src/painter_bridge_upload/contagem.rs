//! n.º 10 F5j (B20.5): **os contadores do envio** da prévia à placa. `PH2D_ENVIO=1` (lida uma vez) soma, por janela de
//! um segundo, os quadros, os envios inteiros e parciais, as janelas parciais, os bytes e os ms de CPU do envio, e
//! imprime uma linha `ENVIO\t…` no stderr. Desligada, o custo é um `OnceLock` lido.

use std::cell::RefCell;
use std::sync::OnceLock;
use std::time::Instant;

pub(crate) fn ligada() -> bool {
    static L: OnceLock<bool> = OnceLock::new();
    *L.get_or_init(|| std::env::var("PH2D_ENVIO").is_ok_and(|v| v == "1"))
}

/// O que um envio fez: `None` = nada (Skip); `Some(true)` = inteiro; `Some(false)` = parcial.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub(crate) struct Contagem {
    pub quadros: u32,
    pub inteiros: u32,
    pub parciais: u32,
    pub janelas: u32,
    pub bytes: u64,
    pub ms: f64,
}

impl Contagem {
    pub(crate) fn soma(&mut self, tipo: Option<bool>, janelas: u32, bytes: u64, ms: f64) {
        self.quadros += 1;
        match tipo {
            Some(true) => self.inteiros += 1,
            Some(false) => self.parciais += 1,
            None => {}
        }
        self.janelas += janelas;
        self.bytes += bytes;
        self.ms += ms;
    }

    pub(crate) fn linha(&self, segundos: f64) -> String {
        format!(
            "ENVIO\t{segundos:.2} s\tquadros {}\tinteiros {}\tparciais {} ({} janelas)\t{:.1} MB\t{:.2} ms de CPU (média/quadro {:.2})",
            self.quadros,
            self.inteiros,
            self.parciais,
            self.janelas,
            self.bytes as f64 / 1e6,
            self.ms,
            if self.quadros == 0 {
                0.0
            } else {
                self.ms / f64::from(self.quadros)
            },
        )
    }
}

thread_local! {
    static JANELA: RefCell<Option<(Instant, Contagem)>> = const { RefCell::new(None) };
}

/// Soma um envio à janela corrente; passado um segundo, imprime-a e abre outra.
pub(crate) fn conta(tipo: Option<bool>, janelas: u32, bytes: u64, ms: f64) {
    JANELA.with_borrow_mut(|j| {
        let (t0, c) = j.get_or_insert_with(|| (Instant::now(), Contagem::default()));
        c.soma(tipo, janelas, bytes, ms);
        let s = t0.elapsed().as_secs_f64();
        if s >= 1.0 {
            eprintln!("{}", c.linha(s));
            *j = None;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::Contagem;

    #[test]
    fn a_contagem_do_envio_separa_inteiros_parciais_e_skip() {
        let mut c = Contagem::default();
        c.soma(Some(true), 0, 33_177_600, 2.0);
        c.soma(Some(false), 3, 4096, 0.5);
        c.soma(None, 0, 0, 0.0);
        assert_eq!(
            (c.quadros, c.inteiros, c.parciais, c.janelas, c.bytes),
            (3, 1, 1, 3, 33_181_696)
        );
        assert!(c.linha(1.0).contains("inteiros 1\tparciais 1 (3 janelas)"));
    }
}
