//! ⭐⭐ **UMA PEÇA → O QUE A PLACA DESENHA** — triângulos, normais, material e oclusão assada.
//!
//! - **Normais por ÂNGULO** (o *auto smooth* do Blender, `30°`): lisas onde a peça é curva, vivas
//!   na quina — o vértice da quina parte-se em tantos quantos os lados que ela separa.
//! - **Material por TRIÂNGULO**: a folha dona do centro dele; o vértice parte-se na fronteira de
//!   cor, então a cor muda a pique onde a peça muda de folha, como no modelador.
//! - **Oclusão ASSADA por vértice, do próprio campo** (o AO de 5 amostras do Quilez): custo ZERO por
//!   quadro, que é o que um jogo de celular faz com a oclusão de cada objeto.

use ph2d_field_eval::hybrid::Hybrid;
use ph2d_field_eval::owners::Owners;

/// O ângulo acima do qual a aresta é viva.
pub const AUTO_SMOOTH_DEG: f32 = 30.0;

/// Onde o AO pergunta ao campo, em células ao longo da normal (dobra a cada amostra).
pub const AO_PASSOS: [f32; 5] = [1.5, 3.0, 6.0, 12.0, 24.0];

/// ⭐ **A malha pronta para a placa** — indexada, em triângulos, com um vértice por
/// (posição, lado da quina, material).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MalhaPronta {
    pub posicoes: Vec<[f32; 3]>,
    pub normais: Vec<[f32; 3]>,
    /// `1` = céu aberto, `0` = fechado.
    pub ao: Vec<f32>,
    /// O índice do material (o da folha, em [`crate::materials::folhas`]).
    pub material: Vec<u32>,
    pub indices: Vec<u32>,
}

impl MalhaPronta {
    #[must_use]
    pub fn triangulos(&self) -> usize {
        self.indices.len() / 3
    }
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(u: [f32; 3], v: [f32; 3]) -> [f32; 3] {
    [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ]
}

fn dot(u: [f32; 3], v: [f32; 3]) -> f32 {
    u[0] * v[0] + u[1] * v[1] + u[2] * v[2]
}

fn unit(v: [f32; 3]) -> [f32; 3] {
    let l = dot(v, v).sqrt();
    if l > 0.0 { v.map(|c| c / l) } else { [0.0, 1.0, 0.0] }
}

/// ⭐ **Prepara uma peça.** Devolve as unidades (índices globais) cuja superfície ela tem, e a
/// malha.
///
/// `mapa` leva o índice LOCAL das `donos` (as folhas do grupo) ao global de
/// [`crate::materials::folhas`]; `unidade_da_folha` leva o global à unidade.
#[must_use]
pub fn prepara(
    m: &ph2d_mesh::Mesh,
    donos: &Owners,
    mapa: &[usize],
    unidade_da_folha: &[Option<usize>],
    campo: &mut Hybrid,
    cell: f32,
) -> (Vec<usize>, MalhaPronta) {
    let pos = m.positions();
    let mut tris: Vec<[u32; 3]> = Vec::with_capacity(m.faces().len() * 2);
    for f in m.faces() {
        let v = f.verts();
        tris.push([v[0], v[1], v[2]]);
        if v.len() == 4 {
            tris.push([v[0], v[2], v[3]]);
        }
    }
    let n_area: Vec<[f32; 3]> = tris
        .iter()
        .map(|t| {
            let [a, b, c] = t.map(|i| pos[i as usize]);
            cross(sub(b, a), sub(c, a))
        })
        .collect();
    let n_unit: Vec<[f32; 3]> = n_area.iter().map(|&n| unit(n)).collect();

    let mut unidades = Vec::new();
    let mat: Vec<u32> = tris
        .iter()
        .map(|t| {
            let [a, b, c] = t.map(|i| pos[i as usize]);
            let centro = [
                (a[0] + b[0] + c[0]) / 3.0,
                (a[1] + b[1] + c[1]) / 3.0,
                (a[2] + b[2] + c[2]) / 3.0,
            ];
            let g = donos.at(centro).and_then(|l| mapa.get(l).copied());
            if let Some(u) = g.and_then(|g| unidade_da_folha.get(g).copied().flatten()) {
                unidades.push(u);
            }
            g.map_or(0, |g| g as u32)
        })
        .collect();
    unidades.sort_unstable();
    unidades.dedup();

    let normais_de_canto = gradientes_dos_cantos(campo, pos, &tris, &n_unit, cell);
    let cos_lim = AUTO_SMOOTH_DEG.to_radians().cos();
    let mut out = MalhaPronta::default();
    // Por vértice original: (material, normal-semente, soma das normais, índice novo).
    let mut emitidos: Vec<Vec<(u32, [f32; 3], [f32; 3], u32)>> = vec![Vec::new(); pos.len()];
    for (ti, t) in tris.iter().enumerate() {
        for (c, &v) in t.iter().enumerate() {
            let g = normais_de_canto[ti * 3 + c];
            let k = mat[ti];
            let slot = emitidos[v as usize]
                .iter_mut()
                .find(|(mk, semente, _, _)| *mk == k && dot(*semente, g) >= cos_lim);
            let idx = if let Some(e) = slot {
                e.2 = [e.2[0] + g[0], e.2[1] + g[1], e.2[2] + g[2]];
                e.3
            } else {
                let i = out.posicoes.len() as u32;
                out.posicoes.push(pos[v as usize]);
                out.normais.push(g);
                out.material.push(k);
                emitidos[v as usize].push((k, g, g, i));
                i
            };
            out.indices.push(idx);
        }
    }
    for lista in &emitidos {
        for &(_, _, soma, i) in lista {
            out.normais[i as usize] = unit(soma);
        }
    }
    out.ao = ao(campo, &out.posicoes, &out.normais, cell);
    (unidades, out)
}

/// ⭐ **A normal de cada CANTO de triângulo é o GRADIENTE do campo**, perguntado um pouco para
/// DENTRO do triângulo (`30 %` do caminho até ao centro).
///
/// ⛔ A normal geométrica (a média das faces) MENTE perto da quina: o vértice do Dual Contouring
/// fica preso à célula e senta `~0,1` célula fora do plano, e a face plana de uma caixa saía com
/// triângulos tortos de `7°` (medido: `1 172` de `27 232` vértices). O gradiente exato não sabe que a
/// malha existe — numa face plana é o eixo, e perguntado do lado de DENTRO do triângulo ele é o da
/// face a que o triângulo pertence, o que mantém a quina viva.
fn gradientes_dos_cantos(
    campo: &mut Hybrid,
    pos: &[[f32; 3]],
    tris: &[[u32; 3]],
    n_face: &[[f32; 3]],
    cell: f32,
) -> Vec<[f32; 3]> {
    let (mut xs, mut ys, mut zs) = (Vec::new(), Vec::new(), Vec::new());
    for t in tris {
        let p = t.map(|i| pos[i as usize]);
        let centro = [0, 1, 2].map(|a| (p[0][a] + p[1][a] + p[2][a]) / 3.0);
        for v in p {
            xs.push(v[0] + 0.3 * (centro[0] - v[0]));
            ys.push(v[1] + 0.3 * (centro[1] - v[1]));
            zs.push(v[2] + 0.3 * (centro[2] - v[2]));
        }
    }
    let mut g = Vec::new();
    if campo.gradients(&xs, &ys, &zs, cell * 0.01, &mut g).is_err() {
        g.clear();
    }
    (0..tris.len() * 3)
        .map(|k| {
            let n = g.get(k).copied().unwrap_or([0.0; 3]);
            let l = dot(n, n).sqrt();
            if l.is_finite() && l > 1.0e-6 {
                n.map(|c| c / l)
            } else {
                n_face[k / 3]
            }
        })
        .collect()
}

/// O AO de 5 amostras ao longo da normal, normalizado a `0..=1`: em cada passo `h`, quanto do
/// caminho até à superfície está tapado (`(h − f)/h`), com o peso a cair para metade por passo.
fn ao(campo: &mut Hybrid, pos: &[[f32; 3]], nrm: &[[f32; 3]], cell: f32) -> Vec<f32> {
    let (mut xs, mut ys, mut zs) = (Vec::new(), Vec::new(), Vec::new());
    for (p, n) in pos.iter().zip(nrm) {
        for &k in &AO_PASSOS {
            let h = k * cell;
            xs.push(p[0] + n[0] * h);
            ys.push(p[1] + n[1] * h);
            zs.push(p[2] + n[2] * h);
        }
    }
    let Ok(f) = campo.eval(&xs, &ys, &zs) else {
        return vec![1.0; pos.len()];
    };
    let pesos: f32 = (0..AO_PASSOS.len()).map(|i| 0.5f32.powi(i as i32)).sum();
    f.chunks(AO_PASSOS.len())
        .map(|amostras| {
            let occ: f32 = amostras
                .iter()
                .zip(AO_PASSOS)
                .enumerate()
                .map(|(i, (&d, k))| {
                    let h = k * cell;
                    0.5f32.powi(i as i32) * ((h - d as f32) / h).clamp(0.0, 1.0)
                })
                .sum();
            1.0 - occ / pesos
        })
        .collect()
}
