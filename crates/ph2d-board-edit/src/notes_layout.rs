//! As contas SEM estado das notas: a grelha (colunas da mais larga, linhas da mais alta), a ordem
//! de leitura de uma selecção, e a planilha colada (o texto que o Excel e as Folhas põem na área de
//! transferência).

use ph2d_board_model::Element;

/// As posições `[x, y]` de `sizes` (`[w, h]`) em `cols` colunas a partir de `origin`, com `gap`
/// entre elas: cada coluna da largura da maior, cada linha da altura da maior dela.
#[must_use]
pub fn grid_layout(sizes: &[[f64; 2]], cols: usize, origin: [f64; 2], gap: f64) -> Vec<[f64; 2]> {
    let cols = cols.max(1);
    let col_w = sizes.iter().map(|s| s[0]).fold(0.0, f64::max);
    let mut out = Vec::with_capacity(sizes.len());
    let mut y = origin[1];
    for row in sizes.chunks(cols) {
        for (c, _) in row.iter().enumerate() {
            out.push([origin[0] + c as f64 * (col_w + gap), y]);
        }
        y += row.iter().map(|s| s[1]).fold(0.0, f64::max) + gap;
    }
    out
}

/// Os elementos por ordem de LEITURA: linhas de cima para baixo (uma linha nova começa quando o
/// centro desce mais de meia altura abaixo do primeiro da linha), cada linha da esquerda à direita.
#[must_use]
pub fn reading_order(els: &[Element]) -> Vec<Element> {
    let mut v = els.to_vec();
    v.sort_by(|a, b| a.center()[1].total_cmp(&b.center()[1]));
    let mut rows: Vec<Vec<Element>> = Vec::new();
    for el in v {
        match rows.last_mut() {
            Some(row) if el.center()[1] <= row[0].center()[1] + row[0].h / 2.0 => row.push(el),
            _ => rows.push(vec![el]),
        }
    }
    rows.into_iter()
        .flat_map(|mut r| {
            r.sort_by(|a, b| a.center()[0].total_cmp(&b.center()[0]));
            r
        })
        .collect()
}

/// ⭐ **Uma planilha colada** (o texto que o Excel e as Folhas põem na área de transferência):
/// linhas por `\n`, células por `\t`; uma célula entre aspas pode ter `\n` e `\t` dentro, e `""` é
/// uma aspa. Devolve as linhas com as suas células.
#[must_use]
pub fn parse_cells(text: &str) -> Vec<Vec<String>> {
    let mut rows = vec![Vec::new()];
    let mut cell = String::new();
    let mut chars = text.chars().peekable();
    let mut quoted = false;
    let mut at_start = true;
    while let Some(ch) = chars.next() {
        if quoted {
            match ch {
                '"' if chars.peek() == Some(&'"') => {
                    chars.next();
                    cell.push('"');
                }
                '"' => quoted = false,
                _ => cell.push(ch),
            }
            continue;
        }
        match ch {
            '"' if at_start => {
                quoted = true;
                at_start = false;
            }
            '\t' => {
                rows.last_mut()
                    .expect("há sempre uma")
                    .push(std::mem::take(&mut cell));
                at_start = true;
            }
            '\r' if chars.peek() == Some(&'\n') => {}
            '\n' | '\r' => {
                rows.last_mut()
                    .expect("há sempre uma")
                    .push(std::mem::take(&mut cell));
                rows.push(Vec::new());
                at_start = true;
            }
            _ => {
                cell.push(ch);
                at_start = false;
            }
        }
    }
    rows.last_mut().expect("há sempre uma").push(cell);
    // A última linha vazia de quem termina em `\n` não é uma linha.
    while rows.last().is_some_and(|r| r.iter().all(String::is_empty)) {
        rows.pop();
    }
    rows
}
