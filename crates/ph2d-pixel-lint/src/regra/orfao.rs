//! **Orphan.** Lei: um pixel opaco cujos 4 vizinhos são TODOS transparentes (ou fora do cel) e
//! nenhum dos 8 vizinhos tem a mesma cor — uma migalha solta do desenho (resto de borracha).
//! Um ponto de brilho de 1 px num FUNDO pintado tem vizinhos 4 opacos e não é acusado; uma ponta
//! de linha tem um vizinho da mesma cor e também não.

use crate::Img;
use crate::img::{VIZ4, VIZ8, cel};

pub fn acha(img: &Img<'_>) -> Vec<Vec<[u32; 2]>> {
    let mut out = Vec::new();
    for i in 0..img.len() {
        let (x, y) = img.xy(i);
        let Some(c) = img.cor(x, y) else { continue };
        let solto = VIZ4.iter().all(|&(dx, dy)| !img.opaco(x + dx, y + dy));
        let sem_par = VIZ8.iter().all(|&(dx, dy)| !img.e(x + dx, y + dy, c));
        if solto && sem_par {
            out.push(vec![cel(x, y)]);
        }
    }
    out
}
