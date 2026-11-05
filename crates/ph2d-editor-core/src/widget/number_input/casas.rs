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
        return zero_sem_sinal(texto);
    }
    let Some((ini, fim, casas)) = decimal(texto) else {
        return zero_sem_sinal(texto);
    };
    let Ok(v) = texto[ini..fim].parse::<f64>() else {
        return zero_sem_sinal(texto);
    };
    for p in (0..casas).rev() {
        // `-0.4` a zero casas dá `-0`.
        let n = format!("{v:.p$}");
        let s = format!("{}{}{}", &texto[..ini], zero_sem_sinal(&n), &texto[fim..]);
        if cabe(text_system, &s) {
            return Cow::Owned(s);
        }
    }
    zero_sem_sinal(texto)
}

/// ⭐ **Um zero não tem sinal que se leia**: `-0.00 px` lê-se `0.00 px`. A porta de todo número
/// mostrado: as duas `format_number`, as casas fixas e, no pintor das caixas, o texto pronto que o
/// painel entrega. Só o PRIMEIRO número conta; um texto em edição nunca passa por aqui.
#[must_use]
pub fn zero_sem_sinal(texto: &str) -> Cow<'_, str> {
    let b = texto.as_bytes();
    let Some(ini) = b.iter().position(u8::is_ascii_digit) else {
        return Cow::Borrowed(texto);
    };
    let fim = b[ini..]
        .iter()
        .position(|c| !c.is_ascii_digit() && *c != b'.')
        .map_or(b.len(), |n| ini + n);
    if ini == 0 || b[ini - 1] != b'-' || b[ini..fim].iter().any(|c| !matches!(c, b'0' | b'.')) {
        return Cow::Borrowed(texto);
    }
    Cow::Owned(format!("{}{}", &texto[..ini - 1], &texto[ini..]))
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

/// [`format_number`] com as casas pedidas: `Some(0)` = inteiro, `Some(n)` = `n` casas fixas.
#[must_use]
pub fn format_number_com_casas(v: f64, casas: Option<usize>) -> String {
    match casas {
        Some(n) if v.is_finite() => zero_sem_sinal(&format!("{v:.n$}")).into_owned(),
        _ => super::format_number(v),
    }
}
