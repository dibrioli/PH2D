//! **Ordem por índice fraccionário** — a posição em z de um elemento é uma chave que ordena
//! lexicograficamente, e entre duas chaves cabe sempre uma terceira. Inserir entre vizinhos não
//! renumera ninguém: é o que deixa duas pessoas reordenarem ao mesmo tempo sem conflito (Etapa 2).
//!
//! A chave é uma fracção em `[0, 1)` escrita em base 62 (`0-9A-Za-z`, que ordena por byte), sem
//! zero final — invariante que garante espaço entre quaisquer duas chaves.

use serde::{Deserialize, Serialize};

const DIGITS: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const BASE: u8 = 62;

/// Uma posição na ordem de um quadro. Compara-se como texto.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FracKey(String);

impl FracKey {
    /// A chave estritamente entre `lo` e `hi` (`None` = a ponta aberta).
    ///
    /// # Panics
    /// Se `lo >= hi` — pedir uma chave entre vizinhos fora de ordem é um defeito de quem chama.
    #[must_use]
    pub fn between(lo: Option<&FracKey>, hi: Option<&FracKey>) -> FracKey {
        let a = lo.map(|k| digits_of(&k.0)).unwrap_or_default();
        let b = hi.map(|k| digits_of(&k.0));
        if let Some(b) = &b {
            assert!(
                a < *b,
                "FracKey::between: lo ({lo:?}) não é menor que hi ({hi:?})"
            );
        }
        let mid = midpoint(&a, b.as_deref());
        FracKey(
            mid.iter()
                .map(|&d| char::from(DIGITS[usize::from(d)]))
                .collect(),
        )
    }

    /// A chave como texto.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn digits_of(s: &str) -> Vec<u8> {
    s.bytes()
        .map(|c| {
            let i = DIGITS.iter().position(|&d| d == c);
            u8::try_from(i.expect("FracKey com carácter fora da base 62")).expect("< 62")
        })
        .collect()
}

/// Fracção estritamente entre `a` e `b` (`None` = 1). Nenhuma das duas termina em zero.
fn midpoint(a: &[u8], b: Option<&[u8]>) -> Vec<u8> {
    if let Some(b) = b {
        let n = b
            .iter()
            .enumerate()
            .take_while(|&(i, &d)| a.get(i).copied().unwrap_or(0) == d)
            .count();
        if n > 0 {
            let mut out = b[..n].to_vec();
            out.extend(midpoint(a.get(n..).unwrap_or(&[]), Some(&b[n..])));
            return out;
        }
    }
    let da = a.first().copied().unwrap_or(0);
    let db = b.map_or(BASE, |b| b[0]);
    if db - da > 1 {
        return vec![da + (db - da) / 2];
    }
    // Dígitos vizinhos: se `b` continua, o seu primeiro dígito sozinho já fica entre os dois.
    if let Some(b) = b.filter(|b| b.len() > 1) {
        return vec![b[0]];
    }
    let mut out = vec![da];
    out.extend(midpoint(a.get(1..).unwrap_or(&[]), None));
    out
}

#[cfg(test)]
#[path = "frac_tests.rs"]
mod tests;
