//! **A TOPOLOGIA da região de simulação** — dobradiças, incidência e cores.
//!
//! ⚠️ **Tudo aqui é construído UMA vez, no pen-down.** A pegada de um pincel muda
//! a cada evento de ponteiro, mas a região que SIMULA não: ela é escolhida quando
//! o traço começa e vive até o pen-up. Reconstruir esta estrutura por evento seria
//! `O(pegada)` a 200 Hz — e, pior, **mudaria a lei no meio do traço**: a ordem de
//! Gauss-Seidel é a coloração, então recolorir é trocar o solver debaixo da mão do
//! artista.

use crate::bending::Hinge;

/// Uma lista de listas, achatada — `starts[i]..starts[i+1]` indexa `items`.
#[derive(Clone, Debug, Default)]
pub(crate) struct Csr {
    starts: Vec<u32>,
    items: Vec<u32>,
}

impl Csr {
    fn build(count: usize, pairs: &mut [(u32, u32)]) -> Self {
        // ⚠️ **A ordenação é a metade DETERMINISTA desta estrutura.** A ordem em
        // que os elementos incidentes são acumulados decide os últimos bits da
        // soma em `f64`; um percurso que dependesse de iteração de mapa daria
        // saídas diferentes para a mesma entrada, e a casa tem hash de replay.
        pairs.sort_unstable();
        let mut starts = vec![0u32; count + 1];
        for (v, _) in pairs.iter() {
            starts[*v as usize + 1] += 1;
        }
        for i in 0..count {
            starts[i + 1] += starts[i];
        }
        Self {
            starts,
            items: pairs.iter().map(|(_, e)| *e).collect(),
        }
    }

    pub(crate) fn of(&self, v: usize) -> &[u32] {
        let (a, b) = (self.starts[v] as usize, self.starts[v + 1] as usize);
        &self.items[a..b]
    }
}

/// **A REGIÃO que simula** — a malha do tecido, já preparada.
#[derive(Clone, Debug)]
pub struct ClothTopology {
    /// Os triângulos, em índices LOCAIS da região.
    pub(crate) tris: Vec<[u32; 3]>,
    /// As dobradiças — uma por aresta interior (duas faces).
    pub(crate) hinges: Vec<Hinge>,
    /// Vértice → triângulos incidentes.
    pub(crate) tri_of: Csr,
    /// Vértice → dobradiças incidentes, empacotadas como `dobradiça·4 + slot`.
    pub(crate) hinge_of: Csr,
    /// Cor → os vértices dela. A ordem de Gauss-Seidel.
    pub(crate) bins: Vec<Vec<u32>>,
    /// Quantos vértices a região tem.
    pub(crate) verts: usize,
}

impl ClothTopology {
    /// Monta a região a partir dos triângulos.
    ///
    /// ⚠️ **Uma aresta com TRÊS ou mais faces não vira dobradiça.** Uma escultura
    /// tem não-manifold (o quad remesh desta casa passou uma jornada inteira a
    /// medi-lo), e escolher duas das três faces seria inventar uma lei. Ali o pano
    /// fica sem resistência a dobrar, o que é **menos** errado do que uma dobra
    /// arbitrária — e a membrana continua valendo.
    #[must_use]
    pub fn build(tris: &[[u32; 3]], verts: usize) -> Self {
        let tris: Vec<[u32; 3]> = tris
            .iter()
            .copied()
            .filter(|t| {
                (t[0] != t[1] && t[1] != t[2] && t[0] != t[2])
                    && t.iter().all(|v| (*v as usize) < verts)
            })
            .collect();

        // As arestas, cada uma com a face e o ápice oposto.
        let mut edges: Vec<(u32, u32, u32, u32)> = Vec::with_capacity(tris.len() * 3);
        for t in &tris {
            for k in 0..3 {
                let (a, b, apex) = (t[k], t[(k + 1) % 3], t[(k + 2) % 3]);
                // A chave é ordenada; o SENTIDO em que a face percorre a aresta
                // fica no par `(a, b)` original, e é ele que orienta o ápice.
                if a < b {
                    edges.push((a, b, apex, 0));
                } else {
                    edges.push((b, a, apex, 1));
                }
            }
        }
        edges.sort_unstable();

        let mut hinges = Vec::new();
        let mut i = 0;
        while i < edges.len() {
            let mut j = i + 1;
            while j < edges.len() && edges[j].0 == edges[i].0 && edges[j].1 == edges[i].1 {
                j += 1;
            }
            // ⚠️ Exatamente DUAS faces, e uma de cada sentido: é isso que faz o
            // ângulo diedro ter sinal. Duas faces do mesmo lado são uma malha
            // dobrada sobre si mesma, e ali o sinal não existe.
            if j - i == 2 && edges[i].3 != edges[i + 1].3 {
                let (e0, e1) = (edges[i].0, edges[i].1);
                let (front, back) = if edges[i].3 == 0 {
                    (edges[i].2, edges[i + 1].2)
                } else {
                    (edges[i + 1].2, edges[i].2)
                };
                hinges.push(Hinge {
                    edge: [e0, e1],
                    apex: [front, back],
                });
            }
            i = j;
        }

        let mut tp: Vec<(u32, u32)> = Vec::with_capacity(tris.len() * 3);
        for (ti, t) in tris.iter().enumerate() {
            for v in t {
                tp.push((*v, u32::try_from(ti).unwrap_or(u32::MAX)));
            }
        }
        let tri_of = Csr::build(verts, &mut tp);

        let mut hp: Vec<(u32, u32)> = Vec::with_capacity(hinges.len() * 4);
        for (hi, h) in hinges.iter().enumerate() {
            let hi = u32::try_from(hi).unwrap_or(u32::MAX);
            for (slot, v) in h.verts().into_iter().enumerate() {
                hp.push((v, hi * 4 + u32::try_from(slot).unwrap_or(0)));
            }
        }
        let hinge_of = Csr::build(verts, &mut hp);

        let bins = color(&tris, &hinges, verts);
        Self {
            tris,
            hinges,
            tri_of,
            hinge_of,
            bins,
            verts,
        }
    }

    /// As cores, cada uma com os vértices dela.
    #[must_use]
    pub fn color_bins(&self) -> &[Vec<u32>] {
        &self.bins
    }

    /// As dobradiças achadas.
    #[must_use]
    pub fn hinge_count(&self) -> usize {
        self.hinges.len()
    }
}

/// **A COLORAÇÃO de VÉRTICE** — o que torna o Gauss-Seidel paralelizável.
///
/// Dois vértices ligados por um elemento (triângulo **ou** dobradiça) não podem
/// partilhar cor: dentro de uma cor ninguém lê o que o vizinho está escrevendo.
///
/// ⭐ **Colorir VÉRTICE dá muito menos cores que colorir ELEMENTO** — no paper do
/// VBD, `8` cores para `3 891` vértices contra `76` para `14 802` tetraedros. É o
/// que faz a varredura ter poucas passadas sequenciais.
///
/// ⚠️ **A dobradiça acopla os QUATRO**, e os dois ápices dela podem não partilhar
/// triângulo nenhum. Colorir só pela aresta de triângulo poria os dois ápices na
/// mesma cor, e aí os dois escreveriam ao mesmo tempo sobre a mesma dobra.
///
/// ⚠️ **Guloso em ordem CRESCENTE de índice, de propósito.** A ordem das cores é a
/// ordem de Gauss-Seidel, logo ela muda o resultado nos últimos bits; derivá-la do
/// índice a torna função da malha, e não da ordem em que um mapa foi iterado.
fn color(tris: &[[u32; 3]], hinges: &[Hinge], verts: usize) -> Vec<Vec<u32>> {
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); verts];
    let mut link = |a: u32, b: u32| {
        if a != b {
            adj[a as usize].push(b);
        }
    };
    for t in tris {
        for a in *t {
            for b in *t {
                link(a, b);
            }
        }
    }
    for h in hinges {
        let vs = h.verts();
        for a in vs {
            for b in vs {
                link(a, b);
            }
        }
    }
    for n in &mut adj {
        n.sort_unstable();
        n.dedup();
    }

    let mut of = vec![u32::MAX; verts];
    let mut taken = Vec::new();
    for v in 0..verts {
        taken.clear();
        for n in &adj[v] {
            let c = of[*n as usize];
            if c != u32::MAX {
                taken.push(c);
            }
        }
        taken.sort_unstable();
        let mut c = 0u32;
        for t in &taken {
            match (*t).cmp(&c) {
                core::cmp::Ordering::Equal => c += 1,
                core::cmp::Ordering::Less => {}
                core::cmp::Ordering::Greater => break,
            }
        }
        of[v] = c;
    }

    let n = of.iter().filter(|c| **c != u32::MAX).map(|c| *c + 1).max();
    let mut bins = vec![Vec::new(); n.unwrap_or(0) as usize];
    for (v, c) in of.iter().enumerate() {
        if *c != u32::MAX {
            bins[*c as usize].push(u32::try_from(v).unwrap_or(u32::MAX));
        }
    }
    bins
}
