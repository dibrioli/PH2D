//! **Pillow shading.** Lei: a profundidade de um pixel é 1 se algum dos 8 vizinhos é transparente
//! ou fora do cel, senão 1 + a menor dos vizinhos (distância de xadrez ao contorno). Numa região
//! opaca (4-ligada) com profundidade ≥ 3, se CADA nível de profundidade tem uma só cor e há ≥ 3
//! cores entre os níveis, a cor depende só da distância ao contorno: a sombra segue a borda a toda
//! a volta, sem luz. Aponta-se o anel intermédio (nem a cor do contorno, nem a do miolo).

use crate::Img;
use crate::img::{VIZ4, VIZ8, cel};
use std::collections::VecDeque;

fn profundidades(img: &Img<'_>) -> Vec<u32> {
    let mut d = vec![0u32; img.len()];
    let mut fila = VecDeque::new();
    for (i, di) in d.iter_mut().enumerate() {
        let (x, y) = img.xy(i);
        if img.opaco(x, y) && VIZ8.iter().any(|&(dx, dy)| !img.opaco(x + dx, y + dy)) {
            *di = 1;
            fila.push_back(i);
        }
    }
    while let Some(i) = fila.pop_front() {
        let (x, y) = img.xy(i);
        for (dx, dy) in VIZ8 {
            if let Some(j) = img.idx(x + dx, y + dy)
                && d[j] == 0
                && img.opaco(x + dx, y + dy)
            {
                d[j] = d[i] + 1;
                fila.push_back(j);
            }
        }
    }
    d
}

pub fn acha(img: &Img<'_>) -> Vec<Vec<[u32; 2]>> {
    let d = profundidades(img);
    let mut visto = vec![false; img.len()];
    let mut out = Vec::new();
    for i in 0..img.len() {
        let (x, y) = img.xy(i);
        if visto[i] || !img.opaco(x, y) {
            continue;
        }
        let mut regiao = Vec::new();
        let mut pilha = vec![i];
        visto[i] = true;
        while let Some(j) = pilha.pop() {
            regiao.push(j);
            let (x, y) = img.xy(j);
            for (dx, dy) in VIZ4 {
                if let Some(k) = img.idx(x + dx, y + dy)
                    && !visto[k]
                    && img.opaco(x + dx, y + dy)
                {
                    visto[k] = true;
                    pilha.push(k);
                }
            }
        }
        if let Some(anel) = anel_de_almofada(img, &d, &regiao) {
            out.push(anel);
        }
    }
    out
}

fn anel_de_almofada(img: &Img<'_>, d: &[u32], regiao: &[usize]) -> Option<Vec<[u32; 2]>> {
    let fundo = regiao.iter().map(|&j| d[j]).max()?;
    if fundo < 3 {
        return None;
    }
    let mut nivel: Vec<Option<[u8; 4]>> = vec![None; fundo as usize + 1];
    for &j in regiao {
        let (x, y) = img.xy(j);
        let c = img.cor(x, y)?;
        let n = &mut nivel[d[j] as usize];
        match n {
            None => *n = Some(c),
            Some(k) if *k != c => return None,
            Some(_) => {}
        }
    }
    let cores: Vec<[u8; 4]> = nivel[1..].iter().map(|c| c.unwrap_or_default()).collect();
    let mut distintas = cores.clone();
    distintas.sort_unstable();
    distintas.dedup();
    if distintas.len() < 3 {
        return None;
    }
    let (borda, miolo) = (cores[0], cores[cores.len() - 1]);
    Some(
        regiao
            .iter()
            .filter_map(|&j| {
                let (x, y) = img.xy(j);
                let c = img.cor(x, y)?;
                (c != borda && c != miolo).then(|| cel(x, y))
            })
            .collect(),
    )
}
