//! ⭐⭐ **UMA PEÇA → O QUE A PLACA DESENHA** — triângulos, normais, material e oclusão assada.
//!
//! - **Normais por ÂNGULO** (o *auto smooth* do Blender, `30°`): lisas onde a peça é curva, vivas
//!   na quina — o vértice da quina parte-se em tantos quantos os lados que ela separa.
//! - **Material por TRIÂNGULO**: a folha dona do centro dele; o vértice parte-se na fronteira de
//!   cor, então a cor muda a pique onde a peça muda de folha, como no modelador.
//! - **Oclusão ASSADA por vértice, do próprio campo** (o AO de 5 amostras do Quilez): custo ZERO por
//!   quadro, que é o que um jogo de celular faz com a oclusão de cada objeto.

use ph2d_field::FieldDoc;
use ph2d_field_eval::hybrid::Registry;
use ph2d_field_eval::owners::Owners;
use ph2d_field_eval::par;

/// O ângulo acima do qual a aresta é viva.
pub const AUTO_SMOOTH_DEG: f32 = 30.0;

/// Onde o AO pergunta ao campo, em unidades do MUNDO ao longo da normal (dobra a cada amostra). ⛔
/// Em células ele encolhia com a resolução: a prof `8` tinha metade do alcance da `7`.
pub const AO_PASSOS: [f32; 5] = [0.01, 0.02, 0.04, 0.08, 0.16];

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
    /// ⭐ Quantos triângulos ficaram VIRADOS contra o campo (a normal geométrica contra a média dos
    /// gradientes dos cantos) — o sintoma dos espinhos de uma grade grossa demais para a forma.
    pub virados: usize,
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
    if l > 0.0 {
        v.map(|c| c / l)
    } else {
        [0.0, 1.0, 0.0]
    }
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
    (doc, reg): (&FieldDoc, &Registry),
    cell: f32,
) -> (Vec<usize>, MalhaPronta) {
    let pos_proj = projeta_na_superficie(doc, reg, m.positions(), cell);
    let pos = &pos_proj[..];
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
    let centros: Vec<[f32; 3]> = tris
        .iter()
        .map(|t| {
            let [a, b, c] = t.map(|i| pos[i as usize]);
            [0, 1, 2].map(|k| (a[k] + b[k] + c[k]) / 3.0)
        })
        .collect();

    let mut unidades = Vec::new();
    let mat: Vec<u32> = centros
        .iter()
        .map(|&centro| {
            let g = donos.at(centro).and_then(|l| mapa.get(l).copied());
            if let Some(u) = g.and_then(|g| unidade_da_folha.get(g).copied().flatten()) {
                unidades.push(u);
            }
            g.map_or(0, |g| g as u32)
        })
        .collect();
    unidades.sort_unstable();
    unidades.dedup();

    let cantos = gradientes_dos_cantos(doc, reg, pos, &tris, &n_area, cell);
    let virados = (0..tris.len())
        .filter(|&t| {
            let m = [0, 1, 2]
                .map(|a| cantos[t * 3].0[a] + cantos[t * 3 + 1].0[a] + cantos[t * 3 + 2].0[a]);
            dot(n_area[t], m) < 0.0
        })
        .count();
    let cos_lim = AUTO_SMOOTH_DEG.to_radians().cos();
    let mut out = MalhaPronta {
        virados,
        ..MalhaPronta::default()
    };
    // Por vértice original, um LADO por grupo de normais.
    let mut emitidos: Vec<Vec<Lado>> = vec![Vec::new(); pos.len()];
    for (ti, t) in tris.iter().enumerate() {
        for (c, &v) in t.iter().enumerate() {
            let (g, modulo, q) = cantos[ti * 3 + c];
            let k = mat[ti];
            let slot = emitidos[v as usize]
                .iter_mut()
                .find(|e| e.material == k && dot(e.semente, g) >= cos_lim);
            let idx = if let Some(e) = slot {
                e.normal = [e.normal[0] + g[0], e.normal[1] + g[1], e.normal[2] + g[2]];
                e.ponto = [e.ponto[0] + q[0], e.ponto[1] + q[1], e.ponto[2] + q[2]];
                e.modulos += modulo;
                e.n += 1.0;
                e.indice
            } else {
                let i = out.posicoes.len() as u32;
                out.posicoes.push(pos[v as usize]);
                out.normais.push(g);
                out.material.push(k);
                emitidos[v as usize].push(Lado {
                    material: k,
                    semente: g,
                    normal: g,
                    ponto: q,
                    modulos: modulo,
                    n: 1.0,
                    indice: i,
                });
                i
            };
            out.indices.push(idx);
        }
    }
    let mut escala = vec![1.0f32; out.posicoes.len()];
    for (v, lista) in emitidos.iter().enumerate() {
        for e in lista {
            let l = dot(e.normal, e.normal).sqrt();
            out.normais[e.indice as usize] = if l > 0.0 {
                e.normal.map(|c| c / l)
            } else {
                e.semente
            };
            escala[e.indice as usize] = (e.modulos / e.n).clamp(0.05, 1.0);
        }
        // ⭐ A QUINA: o vértice vai ao encontro dos planos dos seus lados.
        if let Some(x) = na_quina(pos[v], lista, cos_lim, cell) {
            for e in lista {
                out.posicoes[e.indice as usize] = x;
            }
        }
    }
    out.ao = ao(doc, reg, &out.posicoes, &out.normais, &escala);
    (unidades, out)
}

/// Um LADO de um vértice: os cantos de triângulo que concordam na normal (e no material).
#[derive(Clone, Copy)]
struct Lado {
    material: u32,
    semente: [f32; 3],
    /// A soma das normais dos cantos.
    normal: [f32; 3],
    /// A soma dos pontos onde os cantos foram perguntados (dentro dos triângulos deste lado).
    ponto: [f32; 3],
    modulos: f32,
    n: f32,
    indice: u32,
}

/// ⭐⭐ **O vértice de QUINA vai ao encontro dos planos dos seus lados** — a correcção do *Extended
/// Marching Cubes* (Kobbelt et al., SIGGRAPH 2001).
///
/// ⛔ Medido (02/10, foto da cena 37): a mordida de uma esfera numa caixa saía SERRILHADA, com dentes
/// do tamanho de uma célula, e sem o Newton ela era IGUAL — logo é da extracção: o vértice do Dual
/// Contouring fica preso à célula, e quando a quina passa perto da parede dela o mínimo do QEF cai
/// fora e é puxado para dentro, para fora da quina. Aqui cada lado (um grupo de normais que diferem
/// mais que o ângulo do *auto smooth*) é um plano — a normal média e o ponto médio dos cantos dele —,
/// e o vértice é projectado alternadamente nos planos até à intersecção, preso a uma célula de onde
/// estava. Um lado só (superfície lisa) não é quina: devolve `None`.
fn na_quina(v: [f32; 3], lados: &[Lado], cos_lim: f32, cell: f32) -> Option<[f32; 3]> {
    let mut planos: Vec<([f32; 3], f32)> = Vec::new();
    for e in lados {
        let l = dot(e.normal, e.normal).sqrt();
        if l <= 0.0 {
            continue;
        }
        let n = e.normal.map(|c| c / l);
        if planos.iter().any(|(m, _)| dot(*m, n) >= cos_lim) {
            continue; // o mesmo plano partido só por material: não é quina
        }
        let q = e.ponto.map(|c| c / e.n);
        planos.push((n, dot(n, q)));
    }
    if planos.len() < 2 {
        return None;
    }
    let mut x = v;
    for _ in 0..16 {
        for (n, d) in &planos {
            let e = dot(*n, x) - d;
            x = [x[0] - e * n[0], x[1] - e * n[1], x[2] - e * n[2]];
        }
    }
    let delta = sub(x, v);
    let l = dot(delta, delta).sqrt();
    if !l.is_finite() {
        return None;
    }
    Some(if l > cell {
        [0, 1, 2].map(|a| v[a] + delta[a] * cell / l)
    } else {
        x
    })
}

/// Quantos passos de Newton levam o vértice à superfície, e quanto cada um pode andar (em células).
pub const NEWTON_PASSOS: usize = 4;
pub const NEWTON_PASSO_MAX: f32 = 0.5;

/// ⭐⭐ **O vértice vai PARA CIMA da superfície** — `p ← p − f·∇f/|∇f|²`, com o passo preso a meia
/// célula; quem já chegou (`|f| ≤ 10⁻⁴` célula) sai da conta.
///
/// ⛔ A travessia linear do Dual Contouring assume um campo que é DISTÂNCIA (`|∇f| = 1`), e a peça
/// não é obrigada a sê-lo: medido (02/10) na cena 28, o nó de toro tem `|∇f|` de `0` a `0,72`, e o
/// vértice saía até uma célula fora — caroços e espinhos de `~10 px` a 1080p, que o traçado (que
/// procura a superfície raio a raio) nunca mostrou. O Newton não sabe da grade: converge para o
/// zero do campo VERDADEIRO. ⚠️ O passo preso impede o salto para outro tubo vizinho, e onde o
/// gradiente some o vértice fica onde estava.
fn projeta_na_superficie(
    doc: &FieldDoc,
    reg: &Registry,
    pos: &[[f32; 3]],
    cell: f32,
) -> Vec<[f32; 3]> {
    let mut p = pos.to_vec();
    let (max, tol) = (NEWTON_PASSO_MAX * cell, 1.0e-4 * cell);
    let mut vivos: Vec<usize> = (0..p.len()).collect();
    for _ in 0..NEWTON_PASSOS {
        if vivos.is_empty() {
            break;
        }
        let pts: Vec<[f32; 3]> = vivos.iter().map(|&i| p[i]).collect();
        let Ok(fg) = par::valores_e_gradientes(doc, reg, &pts, cell * 0.01) else {
            break;
        };
        let mut seguem = Vec::with_capacity(vivos.len());
        for (&i, &(f, g)) in vivos.iter().zip(&fg) {
            let g2 = dot(g, g);
            if !f.is_finite() || f.abs() <= tol || g2.is_nan() || g2 <= 1.0e-12 {
                continue;
            }
            let mut passo = g.map(|c| c * f / g2);
            let l = dot(passo, passo).sqrt();
            if l > max {
                passo = passo.map(|c| c * max / l);
            }
            p[i] = [p[i][0] - passo[0], p[i][1] - passo[1], p[i][2] - passo[2]];
            seguem.push(i);
        }
        vivos = seguem;
    }
    p
}

/// ⭐ **A normal de cada CANTO de triângulo é o GRADIENTE do campo**, perguntado um pouco para
/// DENTRO do triângulo (`30 %` do caminho até ao centro) — e o MÓDULO dele (a escala do AO).
///
/// ⛔ A normal geométrica (a média das faces) MENTE perto da quina: o vértice do Dual Contouring
/// fica preso à célula e senta `~0,1` célula fora do plano, e a face plana de uma caixa saía com
/// triângulos tortos de `7°` (medido: `1 172` de `27 232` vértices). O gradiente exato não sabe que a
/// malha existe — numa face plana é o eixo, e perguntado do lado de DENTRO do triângulo ele é o da
/// face a que o triângulo pertence, o que mantém a quina viva.
///
/// ⛔⛔ **Medido e recusado (02/10, fotos da cena 28):** UM gradiente por triângulo (no centro) ou o
/// gradiente NO vértice martelavam a superfície do nó de toro, cujo campo não é distância; os três
/// cantos, somados à volta do vértice, são a média que a alisa. O preço (`3×` as avaliações) é pago
/// em paralelo ([`par`]).
fn gradientes_dos_cantos(
    doc: &FieldDoc,
    reg: &Registry,
    pos: &[[f32; 3]],
    tris: &[[u32; 3]],
    n_area: &[[f32; 3]],
    cell: f32,
) -> Vec<([f32; 3], f32, [f32; 3])> {
    let mut pts = Vec::with_capacity(tris.len() * 3);
    for t in tris {
        let p = t.map(|i| pos[i as usize]);
        let centro = [0, 1, 2].map(|a| (p[0][a] + p[1][a] + p[2][a]) / 3.0);
        for v in p {
            pts.push([0, 1, 2].map(|a| v[a] + 0.3 * (centro[a] - v[a])));
        }
    }
    let g = par::gradientes(doc, reg, &pts, cell * 0.01).unwrap_or_default();
    (0..tris.len() * 3)
        .map(|k| {
            let n = g.get(k).copied().unwrap_or([0.0; 3]);
            let l = dot(n, n).sqrt();
            if l.is_finite() && l > 1.0e-6 {
                (n.map(|c| c / l), l, pts[k])
            } else {
                (unit(n_area[k / 3]), 1.0, pts[k])
            }
        })
        .collect()
}

/// O AO de 5 amostras ao longo da normal, normalizado a `0..=1`: em cada passo `h`, quanto do
/// caminho até à superfície está tapado (`(h − d)/h`), com o peso a cair para metade por passo.
///
/// ⚠️ **`d = f / |∇f|`, e não `f`**: o AO do Quilez assume um campo que é DISTÂNCIA, e um que a
/// subestima (o nó de toro, `|∇f| ≈ 0,5`) lia-se «tapado» onde não está — uma faixa escura ao longo
/// de todo o tubo (foto de 02/10). `escala` é o `|∇f|` no vértice.
fn ao(
    doc: &FieldDoc,
    reg: &Registry,
    pos: &[[f32; 3]],
    nrm: &[[f32; 3]],
    escala: &[f32],
) -> Vec<f32> {
    let mut pts = Vec::with_capacity(pos.len() * AO_PASSOS.len());
    for (p, n) in pos.iter().zip(nrm) {
        for &h in &AO_PASSOS {
            pts.push([p[0] + n[0] * h, p[1] + n[1] * h, p[2] + n[2] * h]);
        }
    }
    let Ok(f) = par::valores(doc, reg, &pts) else {
        return vec![1.0; pos.len()];
    };
    let pesos: f32 = (0..AO_PASSOS.len()).map(|i| 0.5f32.powi(i as i32)).sum();
    f.chunks(AO_PASSOS.len())
        .zip(escala)
        .map(|(amostras, &s)| {
            let occ: f32 = amostras
                .iter()
                .zip(AO_PASSOS)
                .enumerate()
                .map(|(i, (&f, h))| 0.5f32.powi(i as i32) * ((h - f / s) / h).clamp(0.0, 1.0))
                .sum();
            1.0 - occ / pesos
        })
        .collect()
}

/// ⭐⭐ **As duas curvaturas de cada vértice** — `[material, estilo]`, `H` com sinal (`1/mundo`),
/// pela MESMA conta do Render traçado ([`ph2d_field_render::curvatura::curvaturas_por`]) com o
/// avaliador paralelo. Um passo `≤ 0` = ninguém lê aquela: fica `0` sem pagar amostra.
#[must_use]
pub fn curvaturas(
    doc: &FieldDoc,
    reg: &Registry,
    pos: &[[f32; 3]],
    (eps_material, eps_estilo): (f32, f32),
) -> Vec<[f32; 2]> {
    let mede = |eps: f32| {
        if eps <= 0.0 {
            return vec![0.0; pos.len()];
        }
        ph2d_field_render::curvatura::curvaturas_por(pos, eps, |xs, ys, zs| {
            let pts: Vec<[f32; 3]> = (0..xs.len()).map(|i| [xs[i], ys[i], zs[i]]).collect();
            par::valores(doc, reg, &pts).ok()
        })
    };
    let (m, e) = (mede(eps_material), mede(eps_estilo));
    m.into_iter().zip(e).map(|(a, b)| [a, b]).collect()
}
