//! ⭐ **A PORTA PARA A EXTRACÇÃO** — do mapa por patch para o mapa por CANTO.
//!
//! ⚠️ **A imagem é por CANTO, não por vértice**, e é isso que dá a cada triângulo a
//! sua **carta**: o mesmo vértice global tem uma imagem por patch que lhe toca, e é de
//! comparar as duas imagens de uma aresta partilhada que sai a função de transição.
//!
//! ⚠️ **Esta função devolve DADOS SIMPLES e não um tipo de outra crate**, de
//! propósito: assim a `ph2d-gridmap` não passa a depender da `ph2d-quadextract` nem o
//! contrário. *Uma seta a mais no grafo de crates é uma seta que alguém tem de
//! desfazer mais tarde.*

use crate::cut::CutMesh;
use crate::solve::GridMap;

/// **O mapa por canto**: os triângulos em índices **globais**, e a imagem de cada
/// canto no domínio.
///
/// ⚠️ **A ordem dos cantos é a da face original** — o corte preserva-a, e trocá-la
/// inverteria a orientação de metade dos triângulos sem erro nenhum a acusar.
#[must_use]
pub fn corner_map(cut: &CutMesh, map: &GridMap) -> (Vec<[u32; 3]>, Vec<[[f64; 2]; 3]>) {
    let n: usize = cut.tris.iter().map(Vec::len).sum();
    let mut tris = Vec::with_capacity(n);
    let mut uv = Vec::with_capacity(n);
    for (p, ts) in cut.tris.iter().enumerate() {
        let (Some(origin), Some(z)) = (cut.origin.get(p), map.uv.get(p)) else {
            continue;
        };
        for t in ts {
            let (Some(a), Some(b), Some(c)) = (
                origin.get(t[0] as usize),
                origin.get(t[1] as usize),
                origin.get(t[2] as usize),
            ) else {
                continue;
            };
            let img = |l: u32| -> [f64; 2] {
                let w = z.get(l as usize).copied().unwrap_or([0.0; 2]);
                [f64::from(w[0]), f64::from(w[1])]
            };
            tris.push([*a, *b, *c]);
            uv.push([img(t[0]), img(t[1]), img(t[2])]);
        }
    }
    (tris, uv)
}
