//! **Banding.** Lei: um pixel de cor `B` cujos dois vizinhos OPOSTOS num eixo (cima/baixo ou
//! esquerda/direita) são opacos, de cores `A ≠ C` diferentes de `B`, com o brilho de `B` ESTRITAMENTE
//! entre os deles: `B` é o tom de transição numa faixa de 1 px encostada a duas fronteiras
//! paralelas. Acusa-se cada grupo 8-ligado de tais pixels da mesma cor com ≥ 3 células. Uma linha
//! escura entre dois tons claros (traço) não é intermédia e não conta; faixas de largura ≥ 2 também
//! não (fora desta lei). As DUAS fronteiras são paralelas: o grupo partilha o eixo e o par
//! `(A, C)`, cada cor do mesmo lado (o anel de tom base à volta de um brilho não é faixa).

use crate::Img;
use crate::img::{VIZ8, cel, luma};

/// O comprimento mínimo da faixa para ser lida como faixa (e não como um degrau isolado).
const MIN_CELULAS: usize = 3;

/// A chave da faixa: o eixo e as duas cores vizinhas, cada uma do seu lado.
type Chave = (u8, [u8; 4], [u8; 4]);

fn intermedio(img: &Img<'_>, x: i32, y: i32) -> Option<Chave> {
    let b = img.cor(x, y)?;
    [(1, 0), (0, 1)]
        .iter()
        .zip(0u8..)
        .find_map(|(&(dx, dy), eixo)| {
            let (a, c) = (img.cor(x - dx, y - dy)?, img.cor(x + dx, y + dy)?);
            if a == c || a == b || c == b {
                return None;
            }
            let (la, lb, lc) = (luma(a), luma(b), luma(c));
            ((la < lb && lb < lc) || (lc < lb && lb < la)).then_some((eixo, a, c))
        })
}

pub fn acha(img: &Img<'_>) -> Vec<Vec<[u32; 2]>> {
    let marca: Vec<Option<Chave>> = (0..img.len())
        .map(|i| {
            let (x, y) = img.xy(i);
            intermedio(img, x, y)
        })
        .collect();
    let mut visto = vec![false; img.len()];
    let mut out = Vec::new();
    for i in 0..img.len() {
        if marca[i].is_none() || visto[i] {
            continue;
        }
        let cor = img.cor(img.xy(i).0, img.xy(i).1);
        let mut pilha = vec![i];
        visto[i] = true;
        let mut grupo = Vec::new();
        while let Some(j) = pilha.pop() {
            let (x, y) = img.xy(j);
            grupo.push(cel(x, y));
            for (dx, dy) in VIZ8 {
                if let Some(k) = img.idx(x + dx, y + dy)
                    && marca[k] == marca[i]
                    && !visto[k]
                    && img.cor(x + dx, y + dy) == cor
                {
                    visto[k] = true;
                    pilha.push(k);
                }
            }
        }
        if grupo.len() >= MIN_CELULAS {
            out.push(grupo);
        }
    }
    out
}
