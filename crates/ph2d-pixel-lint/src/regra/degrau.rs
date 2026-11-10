//! **Jaggies.** Lei: uma linha de 1 px (componente de uma cor em que cada pixel tem ≤ 2 vizinhos de
//! linha — 4-vizinhos, e a diagonal só sem 4-vizinho comum) parte-se em trechos de eixo e sentido
//! constantes; os degraus de cada trecho (comprimentos `s` das corridas ao longo do eixo) estão
//! certos se `max − min ≤ 1` (recta digital, também com as pontas cortadas), ou se a sequência sem
//! repetições tem no máximo UM extremo interior e esse muda só 1 em cada lado (curva ou inflexão
//! suave). Senão (ex. 3-1-3, 2-1-2-3-1) acusam-se as corridas que são extremos.

use crate::Img;
use crate::img::{VIZ8, cel};

fn vizinhos_de_linha(img: &Img<'_>, x: i32, y: i32, c: [u8; 4]) -> Vec<(i32, i32)> {
    VIZ8.iter()
        .filter(|&&(dx, dy)| {
            img.e(x + dx, y + dy, c)
                && (dx == 0 || dy == 0 || (!img.e(x + dx, y, c) && !img.e(x, y + dy, c)))
        })
        .map(|&(dx, dy)| (x + dx, y + dy))
        .collect()
}

pub fn acha(img: &Img<'_>) -> Vec<Vec<[u32; 2]>> {
    let mut visto = vec![false; img.len()];
    let mut out = Vec::new();
    for i in 0..img.len() {
        let (x0, y0) = img.xy(i);
        let Some(c) = img.cor(x0, y0) else { continue };
        if visto[i] {
            continue;
        }
        // O componente, em ordem de linha: o primeiro é o de cima à esquerda.
        let mut comp = Vec::new();
        let mut pilha = vec![(x0, y0)];
        visto[i] = true;
        let mut simples = true;
        while let Some((x, y)) = pilha.pop() {
            comp.push((x, y));
            let v = vizinhos_de_linha(img, x, y, c);
            simples &= v.len() <= 2;
            for (nx, ny) in v {
                if let Some(k) = img.idx(nx, ny)
                    && !visto[k]
                {
                    visto[k] = true;
                    pilha.push((nx, ny));
                }
            }
        }
        if !simples || comp.len() < 3 {
            continue;
        }
        let ponta = comp
            .iter()
            .copied()
            .filter(|&(x, y)| vizinhos_de_linha(img, x, y, c).len() == 1)
            .min_by_key(|&(x, y)| (y, x))
            .unwrap_or((x0, y0));
        let caminho = percorre(img, ponta, c);
        for trecho in trechos(&caminho) {
            out.extend(avalia(&trecho));
        }
    }
    out
}

fn percorre(img: &Img<'_>, inicio: (i32, i32), c: [u8; 4]) -> Vec<(i32, i32)> {
    let mut caminho = vec![inicio];
    let mut antes: Option<(i32, i32)> = None;
    let mut agora = inicio;
    loop {
        let prox = vizinhos_de_linha(img, agora.0, agora.1, c)
            .into_iter()
            .find(|&p| Some(p) != antes && p != inicio);
        let Some(p) = prox else { break };
        caminho.push(p);
        antes = Some(agora);
        agora = p;
    }
    caminho
}

/// Parte o caminho onde o eixo das corridas ou o sentido mudam; o pixel da junta fica nos dois.
fn trechos(caminho: &[(i32, i32)]) -> Vec<Vec<(i32, i32)>> {
    let mut out = Vec::new();
    let mut atual = vec![caminho[0]];
    // (eixo: 0 = x, 1 = y; sentido em x; sentido em y)
    let mut estado: (Option<u8>, i32, i32) = (None, 0, 0);
    for w in caminho.windows(2) {
        let (dx, dy) = (w[1].0 - w[0].0, w[1].1 - w[0].1);
        let mut novo = estado;
        let ok = encaixa(&mut novo, dx, dy);
        if ok {
            estado = novo;
            atual.push(w[1]);
        } else {
            out.push(std::mem::replace(&mut atual, vec![w[0], w[1]]));
            estado = (None, 0, 0);
            encaixa(&mut estado, dx, dy);
        }
    }
    out.push(atual);
    out
}

fn encaixa(e: &mut (Option<u8>, i32, i32), dx: i32, dy: i32) -> bool {
    let eixo = match (dx, dy) {
        (_, 0) => Some(0),
        (0, _) => Some(1),
        _ => None,
    };
    if let Some(a) = eixo {
        if e.0.is_some_and(|b| b != a) {
            return false;
        }
        e.0 = Some(a);
    }
    for (s, d) in [(&mut e.1, dx), (&mut e.2, dy)] {
        if d != 0 {
            if *s != 0 && *s != d {
                return false;
            }
            *s = d;
        }
    }
    true
}

fn avalia(trecho: &[(i32, i32)]) -> Option<Vec<[u32; 2]>> {
    // Corridas: grupos ligados por passos ortogonais (ao longo do eixo).
    let mut corridas: Vec<Vec<(i32, i32)>> = vec![vec![trecho[0]]];
    for w in trecho.windows(2) {
        let diagonal = w[1].0 != w[0].0 && w[1].1 != w[0].1;
        if diagonal {
            corridas.push(vec![w[1]]);
        } else if let Some(c) = corridas.last_mut() {
            c.push(w[1]);
        }
    }
    let s: Vec<usize> = corridas.iter().map(Vec::len).collect();
    if s.len() < 3 || recta(&s) {
        return None;
    }
    let mut cortada = &s[..];
    if cortada[0] < cortada[1] {
        cortada = &cortada[1..];
    }
    if cortada.len() >= 2 && cortada[cortada.len() - 1] < cortada[cortada.len() - 2] {
        cortada = &cortada[..cortada.len() - 1];
    }
    if recta(cortada) {
        return None;
    }
    // Sem repetições: (valor, corridas do grupo).
    let mut grupos: Vec<(usize, Vec<usize>)> = Vec::new();
    for (k, &v) in s.iter().enumerate() {
        match grupos.last_mut() {
            Some(g) if g.0 == v => g.1.push(k),
            _ => grupos.push((v, vec![k])),
        }
    }
    let extremos: Vec<usize> = (1..grupos.len().saturating_sub(1))
        .filter(|&g| {
            let (a, b, c) = (grupos[g - 1].0, grupos[g].0, grupos[g + 1].0);
            (b > a && b > c) || (b < a && b < c)
        })
        .collect();
    let suave = |g: usize| {
        grupos[g].0.abs_diff(grupos[g - 1].0) <= 1 && grupos[g].0.abs_diff(grupos[g + 1].0) <= 1
    };
    if extremos.is_empty() || (extremos.len() == 1 && suave(extremos[0])) {
        return None;
    }
    Some(
        extremos
            .iter()
            .flat_map(|&g| grupos[g].1.iter().flat_map(|&k| corridas[k].iter()))
            .map(|&(x, y)| cel(x, y))
            .collect(),
    )
}

fn recta(s: &[usize]) -> bool {
    let (mn, mx) = (s.iter().min(), s.iter().max());
    matches!((mn, mx), (Some(a), Some(b)) if b - a <= 1)
}
