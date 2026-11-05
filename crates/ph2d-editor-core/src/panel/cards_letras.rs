//! ⭐ **Duas letras iguais lado a lado não dizem nada** (`docs/CartoesUI/18`, família X): Size e Spacing
//! davam «S S». Numa fila, o campo da direita mostra a letra que o distingue: a inicial de outra palavra do
//! rótulo, senão a letra seguinte da palavra dele. A regra mora na fila, porque quem a pinta conhece os dois.

use super::corpo::{MAIUSCULAS, SEM_LETRA, letra_da_grandeza};
use super::{Cell, Widget};
use crate::widget::card_field::Glyph;

fn maiuscula(c: char) -> Option<&'static str> {
    let u = c.to_ascii_uppercase();
    u.is_ascii_uppercase().then(|| {
        let i = usize::from(u as u8 - b'A');
        &MAIUSCULAS[i..=i]
    })
}

/// A letra de `rotulo` que não está em `ocupadas`: a dele, a inicial de outra palavra, a seguinte da palavra.
#[must_use]
pub fn letra_que_distingue<'a>(rotulo: &'a str, ocupadas: &[&str]) -> &'a str {
    let propria = letra_da_grandeza(rotulo);
    if !ocupadas.contains(&propria) {
        return propria;
    }
    let palavras: Vec<&str> = rotulo
        .split(|c: char| c.is_whitespace() || c == '_')
        .filter(|p| p.starts_with(char::is_alphanumeric))
        .collect();
    let iniciais = palavras
        .iter()
        .rev()
        .filter(|p| !SEM_LETRA.contains(&p.to_ascii_lowercase().as_str()))
        .filter_map(|p| p.chars().next());
    let resto = palavras
        .iter()
        .rev()
        .filter(|p| !SEM_LETRA.contains(&p.to_ascii_lowercase().as_str()))
        .flat_map(|p| p.chars().skip(1));
    iniciais
        .chain(resto)
        .filter_map(maiuscula)
        .find(|l| !ocupadas.contains(l))
        .unwrap_or(propria)
}

/// O glifo da célula `i` na fila: quem tem nome próprio não repete a letra de um vizinho à esquerda.
pub(super) fn glifo_na_fila<'a>(cells: &[Cell<'a>], i: usize, glyph: Glyph<'a>) -> Glyph<'a> {
    let (Glyph::Text(_), Some(rotulo)) = (glyph, cells[i].label) else {
        return glyph;
    };
    let ocupadas: Vec<&str> = cells[..i]
        .iter()
        .enumerate()
        .filter_map(|(j, c)| match c.widget {
            Widget::Field { glyph, .. } => match glifo_na_fila(cells, j, glyph) {
                Glyph::Text(l) => Some(l),
                Glyph::Icon(_) => None,
            },
            _ => None,
        })
        .collect();
    match glyph {
        Glyph::Text(l) if ocupadas.contains(&l) => Glyph::Text(letra_que_distingue(rotulo, &ocupadas)),
        outro => outro,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_a11y::NodeId;

    /// Os casos da foto: cada par sai com duas letras. *Mutação: devolver `propria` sempre ⇒ vermelho.*
    #[test]
    fn os_pares_da_foto_saem_com_duas_letras() {
        for (a, b) in [
            ("Size", "Spacing"),
            ("Spacing", "Slide"),
            ("Temperature", "Tint"),
            ("Cover", "Dry cover"),
            ("Fall-off", "Full for"),
            ("Random 2nd", "Pressure 2nd"),
            ("Max Bounces", "Bounciness"),
            ("Lasting", "Lasts"),
            ("Radius", "Out Radius"),
        ] {
            let la = letra_da_grandeza(a);
            let lb = letra_que_distingue(b, &[la]);
            assert_ne!(la, lb, "{a} e {b}");
            assert_eq!(lb.len(), 1, "{b}: {lb}");
        }
        assert_eq!(letra_que_distingue("Dry cover", &["C"]), "D");
        assert_eq!(letra_que_distingue("Spacing", &["S"]), "P");
        assert_eq!(letra_que_distingue("Full for", &["F"]), "U");
    }

    fn campo<'a>(id: u64, nome: &'a str) -> Cell<'a> {
        Cell::new(Widget::Field {
            id: NodeId(id),
            glyph: Glyph::Text(letra_da_grandeza(nome)),
        })
        .label(nome)
    }

    fn letras<'a>(cells: &[Cell<'a>]) -> Vec<&'a str> {
        cells
            .iter()
            .enumerate()
            .map(|(i, c)| match c.widget {
                Widget::Field { glyph, .. } => match glifo_na_fila(cells, i, glyph) {
                    Glyph::Text(l) => l,
                    Glyph::Icon(_) => "",
                },
                _ => "",
            })
            .collect()
    }

    /// Na fila: a da esquerda fica; a da direita muda só se repete. *Mutação: tirar o `contains` ⇒ vermelho.*
    #[test]
    fn a_fila_so_muda_a_letra_que_repete() {
        assert_eq!(letras(&[campo(1, "Size"), campo(2, "Spacing")]), ["S", "P"]);
        assert_eq!(letras(&[campo(1, "Width"), campo(2, "Height")]), ["W", "H"]);
        assert_eq!(
            letras(&[campo(1, "Size"), campo(2, "Spacing"), campo(3, "Slide")]),
            ["S", "P", "L"]
        );
    }

    /// Letras de eixo de um rótulo só (X · Y) não passam pela regra: a célula seguinte não tem nome.
    #[test]
    fn as_letras_de_um_grupo_nao_mudam() {
        let sem_nome = Cell::new(Widget::Field {
            id: NodeId(2),
            glyph: Glyph::Text("X"),
        });
        assert_eq!(letras(&[campo(1, "X"), sem_nome]), ["X", "X"]);
    }
}
