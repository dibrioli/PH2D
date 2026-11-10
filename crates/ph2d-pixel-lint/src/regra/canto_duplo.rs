//! **Doubles.** Lei: na linha de 1 px de uma cor, o pixel `b` cujos ÚNICOS vizinhos 4 da mesma cor
//! são `a = b + d1` e `c = b + d2` (direcções perpendiculares), com o quarto pixel `a + d2` de outra
//! cor, e com um dos braços (`a` ou `c`) de 1 px só: o L onde a diagonal `a`–`c` bastava. Cantos de
//! braços ≥ 2 px (o canto de uma caixa) são intencionais. Os dois cantos de um mesmo degrau em Z
//! viram UM aviso (apagar qualquer um dos dois cura).

use crate::Img;
use crate::img::cel;

const PARES: [((i32, i32), (i32, i32)); 4] = [
    ((1, 0), (0, 1)),
    ((1, 0), (0, -1)),
    ((-1, 0), (0, 1)),
    ((-1, 0), (0, -1)),
];

fn braco(img: &Img<'_>, x: i32, y: i32, d: (i32, i32), c: [u8; 4]) -> u32 {
    let mut k = 0;
    while img.e(x + d.0 * (k as i32 + 1), y + d.1 * (k as i32 + 1), c) {
        k += 1;
    }
    k
}

fn e_canto(img: &Img<'_>, x: i32, y: i32) -> bool {
    let Some(c) = img.cor(x, y) else { return false };
    PARES.iter().any(|&(d1, d2)| {
        img.e(x + d1.0, y + d1.1, c)
            && img.e(x + d2.0, y + d2.1, c)
            && !img.e(x - d1.0, y - d1.1, c)
            && !img.e(x - d2.0, y - d2.1, c)
            && !img.e(x + d1.0 + d2.0, y + d1.1 + d2.1, c)
            && braco(img, x, y, d1, c).min(braco(img, x, y, d2, c)) == 1
    })
}

pub fn acha(img: &Img<'_>) -> Vec<Vec<[u32; 2]>> {
    let cantos: Vec<bool> = (0..img.len())
        .map(|i| {
            let (x, y) = img.xy(i);
            e_canto(img, x, y)
        })
        .collect();
    let mut visto = vec![false; img.len()];
    let mut out = Vec::new();
    for i in 0..img.len() {
        if !cantos[i] || visto[i] {
            continue;
        }
        let (x, y) = img.xy(i);
        let mut grupo = vec![cel(x, y)];
        visto[i] = true;
        let c = img.cor(x, y);
        for (dx, dy) in [(1, 0), (0, 1)] {
            if let Some(j) = img.idx(x + dx, y + dy)
                && cantos[j]
                && !visto[j]
                && img.cor(x + dx, y + dy) == c
            {
                visto[j] = true;
                grupo.push(cel(x + dx, y + dy));
            }
        }
        out.push(grupo);
    }
    out
}
