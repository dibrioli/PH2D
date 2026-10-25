//! **Adivinhar colunas × linhas de uma folha** pelas faixas transparentes entre as figuras
//! ([plano 07](../../../docs/GameEngineFuncional/07_animacao_num_cartao_plano.md) §3: largar uma folha
//! PROPÕE a grelha, o artista confirma).
//!
//! A grelha do PH2D (`SpriteGrid`) divide a textura em partes IGUAIS, então só se procuram divisores
//! exatos da largura/altura. Por eixo: o perfil de ocupação (a coluna tem algum píxel não
//! transparente?), o número de figuras `R` (corridas de colunas ocupadas) e o MENOR divisor `n ≥ R`
//! cujas fronteiras não atravessam figura nenhuma. ⚠️ A folha do herói não tem calhas desenhadas — as
//! células estão juntas e a transparência é a margem à volta do boneco; é o caso que o gate mede.

/// A grelha proposta.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GridGuess {
    pub cols: u32,
    pub rows: u32,
}

/// Uma célula mais estreita que isto não é uma figura, é ruído.
const CELL_MIN_PX: u32 = 2;

/// Propõe a grelha de uma imagem `w × h` dada a sua opacidade (`alpha`, uma entrada por píxel, por
/// linhas). `None` para uma imagem vazia, totalmente transparente, ou sem divisão que não parta figuras.
#[must_use]
pub fn guess_grid(alpha: &[u8], w: u32, h: u32) -> Option<GridGuess> {
    if w == 0 || h == 0 || alpha.len() != (w as usize) * (h as usize) {
        return None;
    }
    let (wu, hu) = (w as usize, h as usize);
    let mut cols = vec![false; wu];
    let mut rows = vec![false; hu];
    for y in 0..hu {
        for x in 0..wu {
            if alpha[y * wu + x] > 0 {
                cols[x] = true;
                rows[y] = true;
            }
        }
    }
    Some(GridGuess {
        cols: guess_axis(&cols)?,
        rows: guess_axis(&rows)?,
    })
}

/// O menor divisor `n ≥ figuras` do comprimento cujas fronteiras caem fora das figuras.
fn guess_axis(occupied: &[bool]) -> Option<u32> {
    let len = occupied.len();
    let runs = occupied
        .iter()
        .enumerate()
        .filter(|&(i, &o)| o && (i == 0 || !occupied[i - 1]))
        .count();
    if runs == 0 {
        return None;
    }
    (runs..=len / CELL_MIN_PX as usize)
        .filter(|n| len % n == 0)
        .find(|&n| {
            let cell = len / n;
            (1..n).all(|k| {
                let b = k * cell;
                !(occupied[b - 1] && occupied[b])
            })
        })
        .map(|n| n as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Uma folha `cols × rows` de células `cell` px com um quadrado opaco de `fig` px no meio de cada
    /// célula ocupada.
    fn sheet(cols: u32, rows: u32, cell: u32, fig: u32, filled: usize) -> (Vec<u8>, u32, u32) {
        let (w, h) = (cols * cell, rows * cell);
        let mut a = vec![0u8; (w * h) as usize];
        let pad = (cell - fig) / 2;
        for i in 0..filled {
            let (cx, cy) = (i as u32 % cols, i as u32 / cols);
            for y in 0..fig {
                for x in 0..fig {
                    let (px, py) = (cx * cell + pad + x, cy * cell + pad + y);
                    a[(py * w + px) as usize] = 255;
                }
            }
        }
        (a, w, h)
    }

    #[test]
    fn figuras_separadas_por_transparencia_dao_a_grelha() {
        let (a, w, h) = sheet(6, 3, 32, 20, 18);
        assert_eq!(guess_grid(&a, w, h), Some(GridGuess { cols: 6, rows: 3 }));
    }

    #[test]
    fn celulas_vazias_no_fim_nao_mudam_a_grelha() {
        // A última linha só com duas figuras: as colunas vazias dela não contam (as outras linhas
        // ocupam-nas).
        let (a, w, h) = sheet(5, 2, 16, 8, 7);
        assert_eq!(guess_grid(&a, w, h), Some(GridGuess { cols: 5, rows: 2 }));
    }

    #[test]
    fn uma_figura_so_e_uma_celula_e_o_vazio_nao_propoe_nada() {
        let (a, w, h) = sheet(1, 1, 40, 30, 1);
        assert_eq!(guess_grid(&a, w, h), Some(GridGuess { cols: 1, rows: 1 }));
        let (a, w, h) = sheet(4, 4, 8, 4, 0);
        assert_eq!(guess_grid(&a, w, h), None);
        assert_eq!(guess_grid(&[], 0, 0), None);
        assert_eq!(guess_grid(&[255; 3], 2, 2), None, "tamanho errado");
    }

    #[test]
    fn uma_figura_que_atravessa_a_fronteira_impede_a_divisao() {
        // Duas figuras: separadas pelo meio dão 2; se a segunda cruza o meio, nenhum divisor serve.
        let w = 20;
        let mut a = vec![false; w];
        a[2..5].fill(true);
        a[12..15].fill(true);
        assert_eq!(guess_axis(&a), Some(2));
        let mut b = vec![false; w];
        b[2..5].fill(true);
        b[8..19].fill(true);
        assert_eq!(guess_axis(&b), None);
    }
}
