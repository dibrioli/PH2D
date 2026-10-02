//! ⭐⭐ **O número que não cabe perde CASAS, nunca dígitos** (escolha do dono, 2026-10-02).
//!
//! ⛔ O piso da caixa ([`super::MIN_W_PX`]) deixa `44 px` ao número e o `12345.67` pede `52,6` na
//! fonte de fábrica: o campo recortava à direita e o artista lia `12345.` ou `12345.…`. Alargar a
//! caixa custava, medido, `+13 px` e o dobro dos nomes cortados no painel mais estreito (o Inspector
//! a `220`: `18 → 35`) — recusado. ⇒ o número mostra-se **arredondado** a menos casas
//! (`12345.67 → 12345.7 → 12346`); o valor guardado não muda, e a edição mostra-o inteiro.

use std::borrow::Cow;

use ph2d_text::{FontWeight, TextSystem};

/// O texto que o pintor pousa: `texto` tal como veio se cabe em `largura`, senão o mesmo texto com
/// o seu número decimal ARREDONDADO à maior quantidade de casas que cabe (até zero).
///
/// ⚠️ Só o PRIMEIRO número decimal do texto (`-12.345 px` → o `-12.345`) é tocado — a unidade e o
/// resto ficam. Sem número decimal, ou quando nem o inteiro cabe, devolve o texto como veio (o
/// pintor recorta, como antes).
///
/// ⚠️ Mede como a elisão mede ([`crate::text_elide`]: peso `MEDIUM`), para o texto escolhido aqui
/// ser exactamente o que a elisão do pintor deixa passar inteiro.
pub fn numero_que_cabe<'a>(
    text_system: &mut TextSystem,
    texto: &'a str,
    font_size: f32,
    largura: f32,
) -> Cow<'a, str> {
    let cabe = |ts: &mut TextSystem, s: &str| {
        ts.prefix_width_weighted(s, font_size, FontWeight::MEDIUM) <= largura
    };
    if cabe(text_system, texto) {
        return Cow::Borrowed(texto);
    }
    let Some((ini, fim, casas)) = decimal(texto) else {
        return Cow::Borrowed(texto);
    };
    let Ok(v) = texto[ini..fim].parse::<f64>() else {
        return Cow::Borrowed(texto);
    };
    for p in (0..casas).rev() {
        let mut n = format!("{v:.p$}");
        // `-0.4` a zero casas dá `-0`: um zero não tem sinal que se leia.
        if n.parse::<f64>().is_ok_and(|x| x == 0.0) {
            n = n.trim_start_matches('-').to_owned();
        }
        let s = format!("{}{n}{}", &texto[..ini], &texto[fim..]);
        if cabe(text_system, &s) {
            return Cow::Owned(s);
        }
    }
    Cow::Borrowed(texto)
}

/// O primeiro número DECIMAL do texto: `(início, fim, casas)` em bytes, com o sinal incluído.
fn decimal(texto: &str) -> Option<(usize, usize, usize)> {
    let b = texto.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i].is_ascii_digit() {
            let ini = if i > 0 && b[i - 1] == b'-' { i - 1 } else { i };
            let mut j = i;
            while j < b.len() && b[j].is_ascii_digit() {
                j += 1;
            }
            if j + 1 < b.len() && b[j] == b'.' && b[j + 1].is_ascii_digit() {
                let mut k = j + 1;
                while k < b.len() && b[k].is_ascii_digit() {
                    k += 1;
                }
                return Some((ini, k, k - j - 1));
            }
            i = j;
        } else {
            i += 1;
        }
    }
    None
}

#[cfg(test)]
#[path = "casas_tests.rs"]
mod tests;
