//! As 256 cores do GIF. Decisão do dono (05/11): reduz às mais próximas, sem pontilhado, e avisa.
//! Alfa: abaixo de 128 é buraco, o resto é opaco (medido no LibreSprite, 119 → buraco, 136 → opaco).

use std::collections::BTreeMap;

const LIMIAR_ALFA: u8 = 128;
const MAX_CORES: usize = 256;

/// Uma paleta para a animação inteira, os quadros em índices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reduzido {
    pub paleta: Vec<[u8; 3]>,
    pub transparente: Option<u8>,
    pub quadros: Vec<Vec<u8>>,
    /// `Some(n)`: havia `n` cores e a imagem mudou. `None`: saiu idêntica.
    pub reduziu_de: Option<usize>,
}

#[derive(Clone)]
struct Caixa {
    /// `(cor, quantas células)`.
    cores: Vec<([u8; 3], u64)>,
}

impl Caixa {
    fn eixo_maior(&self) -> (usize, u8) {
        let mut melhor = (0, 0);
        for e in 0..3 {
            let (lo, hi) = self
                .cores
                .iter()
                .fold((255, 0), |(lo, hi), (c, _)| (c[e].min(lo), c[e].max(hi)));
            if hi - lo > melhor.1 {
                melhor = (e, hi - lo);
            }
        }
        melhor
    }

    fn media(&self) -> [u8; 3] {
        let total: u64 = self.cores.iter().map(|(_, n)| n).sum::<u64>().max(1);
        let mut s = [0_u64; 3];
        for (c, n) in &self.cores {
            for e in 0..3 {
                s[e] += u64::from(c[e]) * n;
            }
        }
        [0, 1, 2].map(|e| ((s[e] + total / 2) / total) as u8)
    }
}

/// Corte pela mediana: determinístico; cada corte divide a caixa de maior extensão no meio do PESO.
fn corte_mediano(cores: &BTreeMap<[u8; 3], u64>, alvo: usize) -> Vec<[u8; 3]> {
    let mut caixas = vec![Caixa {
        cores: cores.iter().map(|(c, n)| (*c, *n)).collect(),
    }];
    while caixas.len() < alvo {
        let Some(i) = (0..caixas.len())
            .filter(|&i| caixas[i].cores.len() > 1)
            .max_by_key(|&i| (caixas[i].eixo_maior().1, std::cmp::Reverse(i)))
        else {
            break;
        };
        let mut c = caixas.swap_remove(i);
        let (e, _) = c.eixo_maior();
        c.cores.sort_by_key(|(cor, _)| (cor[e], *cor));
        let total: u64 = c.cores.iter().map(|(_, n)| n).sum();
        let mut soma = 0;
        let mut corte = 1;
        for (k, (_, n)) in c.cores.iter().enumerate() {
            soma += n;
            if soma * 2 >= total {
                corte = (k + 1).clamp(1, c.cores.len() - 1);
                break;
            }
        }
        let resto = c.cores.split_off(corte);
        caixas.push(c);
        caixas.push(Caixa { cores: resto });
    }
    let mut p: Vec<[u8; 3]> = caixas.iter().map(Caixa::media).collect();
    p.sort_unstable();
    p.dedup();
    p
}

fn mais_proxima(p: &[[u8; 3]], c: [u8; 3]) -> u8 {
    let d = |q: &[u8; 3]| -> i32 {
        (0..3)
            .map(|e| (i32::from(q[e]) - i32::from(c[e])).pow(2))
            .sum()
    };
    p.iter()
        .enumerate()
        .min_by_key(|(_, q)| d(q))
        .map_or(0, |(i, _)| i as u8)
}

/// **Uma paleta só para os quadros todos.** `quadros` em RGBA.
#[must_use]
pub fn reduzir(quadros: &[&[u8]]) -> Reduzido {
    let mut cores: BTreeMap<[u8; 3], u64> = BTreeMap::new();
    let mut buraco = false;
    for q in quadros {
        for c in q.as_chunks::<4>().0 {
            if c[3] < LIMIAR_ALFA {
                buraco = true;
            } else {
                *cores.entry([c[0], c[1], c[2]]).or_insert(0) += 1;
            }
        }
    }
    let lugares = MAX_CORES - usize::from(buraco);
    let exato = cores.len() <= lugares;
    let reduziu_de = (!exato).then_some(cores.len());
    let cheias = if exato {
        cores.keys().copied().collect()
    } else {
        corte_mediano(&cores, lugares)
    };
    // O transparente fica no fim; o GIF pede tabela de ao menos 2 entradas.
    let mut paleta = cheias.clone();
    let transparente = buraco.then(|| {
        paleta.push([0, 0, 0]);
        (paleta.len() - 1) as u8
    });
    if paleta.len() < 2 {
        paleta.resize(2, [0, 0, 0]);
    }
    let mut cache: BTreeMap<[u8; 3], u8> = BTreeMap::new();
    let quadros = quadros
        .iter()
        .map(|q| {
            q.as_chunks::<4>()
                .0
                .iter()
                .map(|c| {
                    if c[3] < LIMIAR_ALFA {
                        return transparente.unwrap_or(0);
                    }
                    let rgb = [c[0], c[1], c[2]];
                    *cache
                        .entry(rgb)
                        .or_insert_with(|| mais_proxima(&cheias, rgb))
                })
                .collect()
        })
        .collect();
    Reduzido {
        paleta,
        transparente,
        quadros,
        reduziu_de,
    }
}
