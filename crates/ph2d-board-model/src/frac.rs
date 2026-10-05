//! **Ordem por índice fraccionário** — a posição em z de um elemento é uma chave que ordena
//! lexicograficamente, e entre duas chaves cabe sempre uma terceira. Inserir entre vizinhos não
//! renumera ninguém: é o que deixa duas pessoas reordenarem ao mesmo tempo sem conflito (Etapa 2).
//!
//! O esquema publicado do `fractional-indexing` (rocicorp, domínio público — porta do ALGORITMO):
//! a chave é uma PARTE INTEIRA de comprimento variável (a 1.ª letra diz o comprimento: `a..z`
//! positivos, `A..Z` negativos) seguida de uma FRACÇÃO opcional sem zero final, tudo em base 62
//! (`0-9A-Za-z`, que ordena por byte).
//!
//! ⛔ **Porquê a parte inteira, medido:** só com a fracção, acrescentar no FIM faz a chave crescer
//! um dígito a cada ~6 acrescentos — 100 mil elementos davam chaves de milhares de caracteres.
//! Com ela, o acrescento incrementa um inteiro e o comprimento cresce com log₆₂(n).

use serde::{Deserialize, Serialize};

const DIGITS: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const ZERO: u8 = b'0';
const MAX_DIGIT: u8 = b'z';

/// Uma posição na ordem de um quadro. Compara-se como texto.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FracKey(String);

impl FracKey {
    /// A chave estritamente entre `lo` e `hi` (`None` = a ponta aberta).
    ///
    /// # Panics
    /// Se `lo >= hi`, ou se uma das duas não é uma chave válida — os dois são defeitos de quem chama.
    #[must_use]
    pub fn between(lo: Option<&FracKey>, hi: Option<&FracKey>) -> FracKey {
        if let (Some(a), Some(b)) = (lo, hi) {
            assert!(
                a < b,
                "FracKey::between: lo ({a:?}) não é menor que hi ({b:?})"
            );
        }
        let a = lo.map(|k| k.0.as_bytes());
        let b = hi.map(|k| k.0.as_bytes());
        let out = match (a, b) {
            (None, None) => b"a0".to_vec(),
            (None, Some(b)) => {
                let (ib, fb) = split(b);
                if ib.iter().skip(1).all(|&d| d == ZERO) && ib[0] == b'A' {
                    [ib, &midpoint(&[], Some(fb))].concat()
                } else if ib.len() < b.len() {
                    ib.to_vec()
                } else {
                    decrement(ib).expect("FracKey: abaixo da menor chave possível")
                }
            }
            (Some(a), None) => {
                let (ia, fa) = split(a);
                increment(ia).unwrap_or_else(|| [ia, &midpoint(fa, None)].concat())
            }
            (Some(a), Some(b)) => {
                let (ia, fa) = split(a);
                let (ib, fb) = split(b);
                if ia == ib {
                    [ia, &midpoint(fa, Some(fb))].concat()
                } else {
                    match increment(ia) {
                        Some(i) if i.as_slice() < b => i,
                        _ => [ia, &midpoint(fa, None)].concat(),
                    }
                }
            }
        };
        FracKey(String::from_utf8(out).expect("base 62 é ASCII"))
    }

    /// A chave como texto.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn digit_value(c: u8) -> u8 {
    let i = DIGITS
        .iter()
        .position(|&d| d == c)
        .expect("FracKey: carácter fora da base 62");
    u8::try_from(i).expect("< 62")
}

/// Comprimento da parte inteira (incluindo a letra de cabeça).
fn integer_len(head: u8) -> usize {
    match head {
        b'a'..=b'z' => usize::from(head - b'a') + 2,
        b'A'..=b'Z' => usize::from(b'Z' - head) + 2,
        _ => panic!("FracKey: cabeça inválida {:?}", char::from(head)),
    }
}

fn split(key: &[u8]) -> (&[u8], &[u8]) {
    let n = integer_len(key[0]);
    assert!(key.len() >= n, "FracKey: parte inteira truncada");
    key.split_at(n)
}

/// O inteiro seguinte, ou `None` se não há (o maior inteiro possível).
fn increment(int: &[u8]) -> Option<Vec<u8>> {
    let (head, digs) = (int[0], &int[1..]);
    let mut d = digs.to_vec();
    for i in (0..d.len()).rev() {
        if d[i] == MAX_DIGIT {
            d[i] = ZERO;
        } else {
            d[i] = DIGITS[usize::from(digit_value(d[i]) + 1)];
            return Some([&[head][..], &d].concat());
        }
    }
    // Transbordou: muda o comprimento.
    match head {
        b'Z' => Some(b"a0".to_vec()),
        b'z' => None,
        _ => {
            let h = head + 1;
            if h > b'a' {
                d.push(ZERO);
            } else {
                d.pop();
            }
            Some([&[h][..], &d].concat())
        }
    }
}

/// O inteiro anterior, ou `None` se não há (o menor inteiro possível).
fn decrement(int: &[u8]) -> Option<Vec<u8>> {
    let (head, digs) = (int[0], &int[1..]);
    let mut d = digs.to_vec();
    for i in (0..d.len()).rev() {
        if d[i] == ZERO {
            d[i] = MAX_DIGIT;
        } else {
            d[i] = DIGITS[usize::from(digit_value(d[i]) - 1)];
            return Some([&[head][..], &d].concat());
        }
    }
    match head {
        b'a' => Some(b"Zz".to_vec()),
        b'A' => None,
        _ => {
            let h = head - 1;
            if h < b'Z' {
                d.push(MAX_DIGIT);
            } else {
                d.pop();
            }
            Some([&[h][..], &d].concat())
        }
    }
}

/// Fracção estritamente entre `a` e `b` (`None` = 1), em caracteres base 62. Nenhuma das duas
/// termina em zero, e o resultado também não.
fn midpoint(a: &[u8], b: Option<&[u8]>) -> Vec<u8> {
    if let Some(b) = b {
        let n = b
            .iter()
            .enumerate()
            .take_while(|&(i, &d)| a.get(i).copied().unwrap_or(ZERO) == d)
            .count();
        if n > 0 {
            let mut out = b[..n].to_vec();
            out.extend(midpoint(a.get(n..).unwrap_or(&[]), Some(&b[n..])));
            return out;
        }
    }
    let da = a.first().map_or(0, |&c| digit_value(c));
    let db = b.map_or(62, |b| digit_value(b[0]));
    if db - da > 1 {
        // O meio ARREDONDADO (meio para cima), como no algoritmo publicado — os vectores dele
        // apanharam o `floor` que aqui esteve (`a0V..a1` dava `a0k`, o publicado é `a0l`).
        return vec![DIGITS[usize::from((da + db).div_ceil(2))]];
    }
    // Dígitos vizinhos: se `b` continua, o seu primeiro dígito sozinho já fica entre os dois.
    if let Some(b) = b.filter(|b| b.len() > 1) {
        return vec![b[0]];
    }
    let mut out = vec![DIGITS[usize::from(da)]];
    out.extend(midpoint(a.get(1..).unwrap_or(&[]), None));
    out
}

#[cfg(test)]
#[path = "frac_tests.rs"]
mod tests;
