//! **Too many colors.** Lei: o cel usa mais cores opacas distintas (RGBA exacto) do que o limite.
//! As cores ordenam-se por uso (mais usada primeiro; empate pelo RGBA) e o aviso aponta as células
//! das que passam do limite — as mais raras, as primeiras a cortar. A QUASE-igualdade (duas cores
//! que o olho não separa) fica de fora: precisa de OKLab, que este núcleo não tem (P13 §12, A).

use crate::Img;
use crate::img::cel;
use std::collections::BTreeMap;

pub fn acha(img: &Img<'_>, limite: usize) -> Vec<Vec<[u32; 2]>> {
    let mut uso: BTreeMap<[u8; 4], u32> = BTreeMap::new();
    for i in 0..img.len() {
        let (x, y) = img.xy(i);
        if let Some(c) = img.cor(x, y) {
            *uso.entry(c).or_default() += 1;
        }
    }
    if uso.len() <= limite {
        return Vec::new();
    }
    let mut ordem: Vec<([u8; 4], u32)> = uso.into_iter().collect();
    ordem.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let mut extra: Vec<[u8; 4]> = ordem[limite..].iter().map(|e| e.0).collect();
    extra.sort_unstable();
    let celulas = (0..img.len())
        .filter_map(|i| {
            let (x, y) = img.xy(i);
            let c = img.cor(x, y)?;
            extra.binary_search(&c).is_ok().then(|| cel(x, y))
        })
        .collect();
    vec![celulas]
}
