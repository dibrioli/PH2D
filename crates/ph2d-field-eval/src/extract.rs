//! **A extração da malha** — Dual Contouring sobre grade uniforme, em **quads**.
//!
//! # ⭐ Por que existe um extrator da casa, quando o motor já traz um
//!
//! O extrator da `fidget` (*Manifold Dual Contouring* com leque) foi medido duas vezes por este
//! módulo e reprovou nas duas, **pelo mesmo mecanismo**:
//!
//! | quando | sintoma | medição |
//! |---|---|---|
//! | W0 | aresta viva **serrilhada** | o desvio é IGUAL à fração de célula em que a face cai (`01_resultados_spike.md` §2) |
//! | W19 | faces **dobradas** — manchas em *shade smooth* | 3,5 % dos triângulos com a normal ao contrário, área média 0,15 célula² |
//!
//! ⚠️ **São DOIS mecanismos, e um deles nem era do extrator.** Separá-los custou quatro medições, e
//! o caminho é o conteúdo:
//!
//! | hipótese | veredito | como se soube |
//! |---|---|---|
//! | o QEF da `fidget` escapa da célula | ✅ **é a face dobrada** | `qef.rs::solve` diz *"increase the **likelihood** that the vertex is bounded in the cell"*, e o `bounds.contains` só existe no caminho de colapso — a aresta mais longa de uma invertida media **4,11 células** |
//! | o **leque** da `fidget` serrilha a quina | ⛔ **REFUTADA** | este extrator não tem leque nenhum e reproduzia o **mesmo** desvio |
//! | a **interpolação linear** da travessia serrilha a quina | ⛔ **REFUTADA** | apertar a travessia com 10 bisseções não mexeu **um dígito** na tabela — e piorou a esfera em 25 %, porque empurra `f` para dentro do ruído do `f32` |
//! | perguntar a normal **sobre** a superfície | ⛔ **REFUTADA** | afastar o ponto 1/1000 de aresta para dentro não mexeu no 5º dígito |
//! | `sqrt(0)` tem derivada **infinita** | ✅ **é a quina serrilhada** | `0/49` faixas capturadas → **116/116**, desvio `0,80` célula → `0,00` |
//!
//! ⭐ A quina nunca foi um problema de triangulação: `box_raw` é `length3(max(q,0)…)`, e **dentro da
//! peça inteira** os três termos são zero, então o gradiente automático era `NaN`. Sem normal não há
//! QEF, a célula caía no **baricentro** das travessias, e o desvio medido era `0,72 × (fração de
//! célula em que a face cai)` — que é literalmente esse baricentro. A cura está em
//! [`crate::ops::safe_sqrt`], e é do **campo**, não daqui.
//!
//! # O que este extrator faz de diferente, em três linhas
//!
//! 1. **Um vértice por célula, PRESO à célula.** É a propriedade que apaga a face dobrada — não uma
//!    correção a jusante dela.
//! 2. **Quad por aresta da grade**, com a rotação vinda do sinal, e a diagonal escolhida por
//!    rotação do quad.
//! 3. **Quads de verdade** ([`ph2d_mesh::Face::quad`]) — valência 4 quase em toda parte, que é o que
//!    subdivide bem e o que um *remesh* a jusante consegue comer.
//!
//! # ⚠️ O que ele NÃO é
//!
//! Não é um octree: a grade é **uniforme**, e o custo é o cubo da resolução. É uma escolha MEDIDA
//! (`docs/3DModeling/06_resultados_cena_e_gizmo.md` §21) e não um descuido — a avaliação em lote com
//! JIT é barata o bastante. Em troca, `depth` passa a ser **a resolução** e não um teto: os vértices
//! quadruplicam a cada degrau e o erro cai por 4, o que o extrator adaptativo não fazia. O dia em
//! que a tabela disser o contrário, o eixo a abrir é a poda por aritmética de intervalos, que a
//! `fidget` já expõe.

use ph2d_mesh::{Face, Mesh};

use crate::MeshError;

/// As 12 arestas do cubo, como pares de cantos `b = dx + 2·dy + 4·dz`.
const CUBE_EDGES: [(usize, usize); 12] = [
    (0, 1),
    (2, 3),
    (4, 5),
    (6, 7),
    (0, 2),
    (1, 3),
    (4, 6),
    (5, 7),
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
];

/// Nenhum vértice nesta célula.
const NO_VERT: u32 = u32::MAX;

/// Abaixo de que fração do maior autovalor o QEF ignora uma direção.
///
/// ⭐ **É o número que decide quantas faces saem dobradas — e a suspeita de que ele decidisse
/// também a QUINA está REFUTADA.** Autovalor pequeno significa direção mal determinada; contá-la
/// atira o vértice para longe, e a prisão à célula converte a fuga num vértice colado à parede, que
/// é o que torce o quad. Varrido nas seis fixtures, profundidades 6 e 7 (total de faces dobradas), e
/// contra a captura da aresta viva do cubo:
///
/// | corte | faces dobradas | aresta viva | erro médio da esfera (prof. 5) |
/// |---|---|---|---|
/// | 1e-1 | **0** | 116/116, desvio 0,00 | 9,010e-4 |
/// | **3e-2** | **0** | 116/116, desvio 0,00 | 9,010e-4 |
/// | 1e-2 | **0** | 116/116, desvio 0,00 | 9,010e-4 |
/// | 1e-3 | 32 | 116/116, desvio 0,00 | 9,353e-4 |
/// | 1e-4 | 328 | 116/116, desvio 0,00 | — |
/// | 1e-6 | 624 | 116/116, desvio 0,00 | — |
///
/// ⚠️ **A coluna da aresta viva não se mexe** — a quina é assunto de [`crate::ops::safe_sqrt`], e
/// não deste corte; era a hipótese oposta, e a tabela fecha-a. E o erro da esfera **melhora** com o
/// corte maior, então não há troca a fazer: o patamar de zero dobradas vai de 1e-2 a 1e-1, e o valor
/// escolhido fica no meio dele, com uma ordem de grandeza de folga para cada lado.
///
/// ⛔ O `1e-3` que estava aqui veio do valor *"somewhat arbitrarily"* da `fidget` (`qef.rs`), sem
/// medição nossa — e é justamente o primeiro degrau que reprova.
const QEF_RANK_CUTOFF: f64 = 3.0e-2;

/// Quantas varreduras de Jacobi. Uma 3×3 simétrica converge em 4–6; 12 é folga barata e **fixa**,
/// que é o que mantém a extração determinística (HR-5) sem um critério de paragem que dependa dos
/// dados.
const JACOBI_SWEEPS: usize = 12;

/// Quanto a prisão da célula RECUA da parede, em frações da aresta da célula.
///
/// ⭐ **Não é folga de segurança: é o que torna as caixas de duas células DISJUNTAS**, e daí que dois
/// vértices vizinhos não possam coincidir — que era como nasciam triângulos de área **exatamente**
/// zero e vértices repetidos (medido nas cenas 4 e 5, §21). Prender à parede é prender ao ponto que
/// a célula ao lado também pode reclamar.
///
/// ⚠️ **O piso é de REPRESENTAÇÃO, não de gosto.** A posição sai em `f32` e o ULP de uma coordenada
/// de ordem 1 é 1,19e-7; a célula a prof 9 mede 3,9e-3, então 1 % dela são 3,9e-5 — cerca de **330
/// ULP**, larga o suficiente para sobreviver ao arredondamento e estreita o suficiente para que
/// mover um vértice fugido 1 % de célula não seja visível em geometria nenhuma.
const CELL_INSET: f64 = 0.01;

/// A grade que a extração percorre.
struct Grid {
    /// Células por eixo.
    n: usize,
    /// Aresta da célula.
    step: f64,
}

impl Grid {
    fn new(depth: u8) -> Self {
        // ⚠️ `[-1, 1]` é a caixa do motor — a mesma que o `Octree::build` assume por omissão. Trocar
        // por uma caixa apertada à peça multiplica a resolução efetiva e é wave própria: ela mexe
        // no significado da profundidade, que já está escrita na tabela de exportação.
        let n = 1usize << depth;
        Self {
            n,
            step: 2.0 / n as f64,
        }
    }

    fn coord(&self, i: usize) -> f64 {
        (i as f64).mul_add(self.step, -1.0)
    }

    /// Amostras por eixo — uma a mais que as células.
    fn samples(&self) -> usize {
        self.n + 1
    }
}

/// Autovalores e autovetores (em colunas) de uma simétrica 3×3, por Jacobi cíclico.
fn jacobi3(mut a: [[f64; 3]; 3]) -> ([f64; 3], [[f64; 3]; 3]) {
    let mut v = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    for _ in 0..JACOBI_SWEEPS {
        for (p, q) in [(0usize, 1usize), (0, 2), (1, 2)] {
            let apq = a[p][q];
            if apq.abs() < 1.0e-18 {
                continue;
            }
            let theta = (a[q][q] - a[p][p]) / (2.0 * apq);
            let t = theta.signum() / (theta.abs() + theta.mul_add(theta, 1.0).sqrt());
            let c = 1.0 / t.mul_add(t, 1.0).sqrt();
            let s = t * c;
            for row in a.iter_mut() {
                let (akp, akq) = (row[p], row[q]);
                row[p] = c * akp - s * akq;
                row[q] = s * akp + c * akq;
            }
            let (rp, rq) = (a[p], a[q]);
            for k in 0..3 {
                a[p][k] = c * rp[k] - s * rq[k];
                a[q][k] = s * rp[k] + c * rq[k];
            }
            for row in v.iter_mut() {
                let (vkp, vkq) = (row[p], row[q]);
                row[p] = c * vkp - s * vkq;
                row[q] = s * vkp + c * vkq;
            }
        }
    }
    ([a[0][0], a[1][1], a[2][2]], v)
}

/// O vértice da célula: o mínimo do QEF, **preso à caixa da célula**.
///
/// ⚠️ **A prisão é a lei deste arquivo.** Ela não é uma salvaguarda contra o improvável: é o que
/// torna o quad em torno de cada aresta da grade convexo por construção, e é por isso que a face
/// dobrada deixa de existir em vez de ser consertada depois.
fn qef_vertex(
    ata: [[f64; 3]; 3],
    atb: [f64; 3],
    center: [f64; 3],
    lo: [f64; 3],
    hi: [f64; 3],
) -> [f64; 3] {
    // Minimiza em torno do ponto de massa: b' = Aᵀb − AᵀA·c.
    let mut b = [0.0f64; 3];
    for (r, slot) in b.iter_mut().enumerate() {
        let mut s = 0.0;
        for c in 0..3 {
            s += ata[r][c] * center[c];
        }
        *slot = atb[r] - s;
    }

    let (w, v) = jacobi3(ata);
    let max = w.iter().fold(0.0f64, |m, x| m.max(x.abs()));
    let cut = max * QEF_RANK_CUTOFF;

    // x = c + V·diag(1/w truncado)·Vᵀ·b'
    let mut out = center;
    for i in 0..3 {
        if w[i].abs() <= cut {
            continue;
        }
        let dot = v[2][i].mul_add(b[2], v[0][i].mul_add(b[0], v[1][i] * b[1]));
        let f = dot / w[i];
        for r in 0..3 {
            out[r] += v[r][i] * f;
        }
    }

    for r in 0..3 {
        if !out[r].is_finite() {
            out[r] = center[r];
        }
        out[r] = out[r].clamp(lo[r], hi[r]);
    }
    out
}

/// As travessias de uma célula, acrescentadas a `out`. Devolve a faixa `[início, fim)`.
fn crossings_of(
    grid: &Grid,
    c: &[f32; 8],
    cell: [usize; 3],
    out: &mut Vec<[f64; 3]>,
) -> (usize, usize) {
    let corner = |b: usize| {
        [
            grid.coord(cell[0] + (b & 1)),
            grid.coord(cell[1] + ((b >> 1) & 1)),
            grid.coord(cell[2] + ((b >> 2) & 1)),
        ]
    };
    let start = out.len();
    for &(a, b) in &CUBE_EDGES {
        let (fa, fb) = (f64::from(c[a]), f64::from(c[b]));
        if (fa < 0.0) == (fb < 0.0) {
            continue;
        }
        // ⚠️ **Interpolação linear, e ela é a MELHOR das três medidas** — ver a nota do módulo sobre
        // as duas hipóteses refutadas. Apertar a travessia por bisseção antes de interpolar piora a
        // esfera em 25 %, porque empurra `f` para dentro do ruído do `f32`.
        let d = fa - fb;
        let t = if d == 0.0 {
            0.5
        } else {
            (fa / d).clamp(0.0, 1.0)
        };
        let (pa, pb) = (corner(a), corner(b));
        out.push([
            t.mul_add(pb[0] - pa[0], pa[0]),
            t.mul_add(pb[1] - pa[1], pa[1]),
            t.mul_add(pb[2] - pa[2], pa[2]),
        ]);
    }
    (start, out.len())
}

/// ⭐ **A extração**: documento → malha em quads, na profundidade pedida.
///
/// # Errors
/// Ver [`MeshError`]. A malha sai **vazia** (e não em erro) quando o nível zero não cruza a caixa.
pub fn extract(
    doc: &ph2d_field::FieldDoc,
    reg: &crate::hybrid::Registry,
    depth: u8,
) -> Result<Mesh, MeshError> {
    let mut field = crate::hybrid::Hybrid::new(doc, reg);
    let grid = Grid::new(depth);
    let m = grid.samples();

    let (mut xs, mut ys, mut zs) = (Vec::new(), Vec::new(), Vec::new());
    let plane_coords = |k: usize, xs: &mut Vec<f32>, ys: &mut Vec<f32>, zs: &mut Vec<f32>| {
        xs.clear();
        ys.clear();
        zs.clear();
        let z = grid.coord(k) as f32;
        for j in 0..m {
            let y = grid.coord(j) as f32;
            for i in 0..m {
                xs.push(grid.coord(i) as f32);
                ys.push(y);
                zs.push(z);
            }
        }
    };

    let mut plane_lo = Vec::new();
    let mut plane_hi = Vec::new();
    plane_coords(0, &mut xs, &mut ys, &mut zs);
    plane_lo.extend_from_slice(field.eval(&xs, &ys, &zs)?);

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut faces: Vec<Face> = Vec::new();
    let mut vidx_prev = vec![NO_VERT; grid.n * grid.n];
    let mut vidx_cur = vec![NO_VERT; grid.n * grid.n];

    // Reaproveitados a cada camada — a alocação mora fora do laço de propósito.
    let mut cross: Vec<[f64; 3]> = Vec::new();
    let mut spans: Vec<(usize, usize, usize, usize)> = Vec::new();
    let (mut gx, mut gy, mut gz): (Vec<f32>, Vec<f32>, Vec<f32>) =
        (Vec::new(), Vec::new(), Vec::new());
    let mut grads: Vec<[f32; 3]> = Vec::new();

    for k in 0..grid.n {
        plane_coords(k + 1, &mut xs, &mut ys, &mut zs);
        plane_hi.clear();
        plane_hi.extend_from_slice(field.eval(&xs, &ys, &zs)?);

        // — Passo 1: as travessias de cada célula da camada.
        cross.clear();
        spans.clear();
        for j in 0..grid.n {
            for i in 0..grid.n {
                let c = [
                    plane_lo[j * m + i],
                    plane_lo[j * m + i + 1],
                    plane_lo[(j + 1) * m + i],
                    plane_lo[(j + 1) * m + i + 1],
                    plane_hi[j * m + i],
                    plane_hi[j * m + i + 1],
                    plane_hi[(j + 1) * m + i],
                    plane_hi[(j + 1) * m + i + 1],
                ];
                let inside = c.iter().filter(|v| **v < 0.0).count();
                if inside == 0 || inside == 8 {
                    continue;
                }
                let (a, b) = crossings_of(&grid, &c, [i, j, k], &mut cross);
                if a < b {
                    spans.push((i, j, a, b));
                }
            }
        }

        // — Passo 2: os gradientes das travessias, numa chamada.
        grads.clear();
        if !cross.is_empty() {
            gx.clear();
            gy.clear();
            gz.clear();
            for p in &cross {
                gx.push(p[0] as f32);
                gy.push(p[1] as f32);
                gz.push(p[2] as f32);
            }
            // ⚠️ O passo da diferença central só é usado quando há escultura (ver
            // [`Hybrid::gradients`]); um documento analítico continua a ter gradiente EXATO, que é
            // o que prende a quina viva.
            field.gradients(&gx, &gy, &gz, (grid.step * 0.01) as f32, &mut grads)?;
        }

        // — Passo 3: um vértice por célula, preso à célula.
        vidx_cur.fill(NO_VERT);
        for &(i, j, a, b) in &spans {
            let v = cell_vertex(&grid, [i, j, k], &cross[a..b], &grads[a..b]);
            vidx_cur[j * grid.n + i] = positions.len() as u32;
            positions.push([v[0] as f32, v[1] as f32, v[2] as f32]);
        }

        // — Passo 4: um quad por aresta da grade que troca de sinal.
        emit_faces(
            &grid,
            m,
            k,
            (&plane_lo, &plane_hi),
            (&vidx_prev, &vidx_cur),
            &positions,
            &mut faces,
        );

        std::mem::swap(&mut plane_lo, &mut plane_hi);
        std::mem::swap(&mut vidx_prev, &mut vidx_cur);
    }

    Mesh::from_parts(positions, faces).map_err(|e| MeshError::Rejected(format!("{e:?}")))
}

/// O vértice de uma célula, das suas travessias e das normais nelas.
fn cell_vertex(grid: &Grid, cell: [usize; 3], cross: &[[f64; 3]], grads: &[[f32; 3]]) -> [f64; 3] {
    let mut ata = [[0.0f64; 3]; 3];
    let mut atb = [0.0f64; 3];
    let mut mass = [0.0f64; 3];
    for (&p, g) in cross.iter().zip(grads) {
        for r in 0..3 {
            mass[r] += p[r];
        }
        let mut nrm = [f64::from(g[0]), f64::from(g[1]), f64::from(g[2])];
        let len = nrm[2]
            .mul_add(nrm[2], nrm[0].mul_add(nrm[0], nrm[1] * nrm[1]))
            .sqrt();
        // ⚠️ Sem normal utilizável a travessia ainda conta para o ponto de massa: ela sabe ONDE a
        // superfície está, só não sabe para onde ela olha.
        if !len.is_finite() || len < 1.0e-12 {
            continue;
        }
        for slot in &mut nrm {
            *slot /= len;
        }
        let d = nrm[2].mul_add(p[2], nrm[0].mul_add(p[0], nrm[1] * p[1]));
        for r in 0..3 {
            for c in 0..3 {
                ata[r][c] += nrm[r] * nrm[c];
            }
            atb[r] += nrm[r] * d;
        }
    }
    let count = cross.len() as f64;
    let center = [mass[0] / count, mass[1] / count, mass[2] / count];
    let inset = grid.step * CELL_INSET;
    let lo = [
        grid.coord(cell[0]) + inset,
        grid.coord(cell[1]) + inset,
        grid.coord(cell[2]) + inset,
    ];
    let hi = [
        lo[0] + grid.step - 2.0 * inset,
        lo[1] + grid.step - 2.0 * inset,
        lo[2] + grid.step - 2.0 * inset,
    ];
    qef_vertex(ata, atb, center, lo, hi)
}

fn cross3(u: [f64; 3], v: [f64; 3]) -> [f64; 3] {
    [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ]
}

fn dot3(u: [f64; 3], v: [f64; 3]) -> f64 {
    u[2].mul_add(v[2], u[0].mul_add(v[0], u[1] * v[1]))
}

/// O quadrado do comprimento de uma diagonal.
fn d2(pos: &[[f32; 3]], a: u32, b: u32) -> f64 {
    let (p, q) = (
        pos[a as usize].map(f64::from),
        pos[b as usize].map(f64::from),
    );
    (q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2) + (q[2] - p[2]).powi(2)
}

/// ⭐ **Quão bem a partição `a–c` deste quad concorda com o próprio quad** — o mínimo de `n̂ᵢ · N̂`
/// entre as duas metades, com `N` a normal do quad inteiro (Newell). `1` é uma face plana; **um
/// valor negativo é uma metade virada do avesso**.
///
/// ⚠️ **Prender o vértice à célula garante que o quad é simples e que a área total tem o sinal
/// certo; não garante que ele seja CONVEXO nem PLANO.** Num quad côncavo, a diagonal que passa fora
/// dele parte a face num triângulo bom e num invertido — a soma continua certa e a tela mostra uma
/// mancha escura. Medido na cena 5, prof. 7: 32 faces assim no fundo chato do vaso, e as quatro
/// células que as formavam estavam **cada uma no seu lugar**.
///
/// ⛔ **Duas regras foram tentadas antes, e as duas são insuficientes** — é por isso que a escolha é
/// uma PONTUAÇÃO e não um predicado:
/// - *a diagonal mais curta*: nas 32 ela escolhia justamente a de fora (0,0113 contra 0,0175);
/// - *`n₁ · n₂ < 0`*: num quad em **sela** isso dispara nas DUAS diagonais e a regra não sabe qual
///   delas está errada — foi assim que o copo do torno apareceu com 32 faces dobradas ao entrar na
///   fixture.
///
/// Comparar com a normal do quad responde às duas de uma vez: ela é a média, e quem discorda dela é
/// quem está virado.
fn split_score(pos: &[[f32; 3]], quad: [u32; 4]) -> f64 {
    let p = quad.map(|i| pos[i as usize].map(f64::from));
    let sub = |u: [f64; 3], v: [f64; 3]| [u[0] - v[0], u[1] - v[1], u[2] - v[2]];
    // Newell: vale para qualquer polígono, plano ou não.
    let mut n = [0.0f64; 3];
    for k in 0..4 {
        let (a, b) = (p[k], p[(k + 1) % 4]);
        n[0] += (a[1] - b[1]) * (a[2] + b[2]);
        n[1] += (a[2] - b[2]) * (a[0] + b[0]);
        n[2] += (a[0] - b[0]) * (a[1] + b[1]);
    }
    let nl = dot3(n, n).sqrt();
    if nl <= 0.0 {
        return -1.0;
    }
    let nn = [n[0] / nl, n[1] / nl, n[2] / nl];
    let half = |u: [f64; 3], v: [f64; 3]| {
        let c = cross3(u, v);
        let l = dot3(c, c).sqrt();
        if l <= 0.0 { -1.0 } else { dot3(c, nn) / l }
    };
    half(sub(p[1], p[0]), sub(p[2], p[0])).min(half(sub(p[2], p[0]), sub(p[3], p[0])))
}

/// Os quads da camada `k`.
///
/// ⚠️ **A ROTAÇÃO é a metade que não compila errado.** Cada quad dá a volta ao eixo da aresta na
/// ordem do referencial de mão direita daquele eixo, e **inverte quando o canto mínimo está fora**:
/// a normal tem de apontar para onde `f` cresce. Um sinal trocado aqui não levanta erro nenhum —
/// sai uma peça inteira do avesso, que em *shade smooth* é exatamente o que se estava a curar.
///
/// ⚠️ **E a DIAGONAL do quad também é escolhida aqui, girando-o.** Prender o vértice à célula apaga
/// a dobra grande; sobra a pequena, do quad **não-plano** que o consumidor parte sempre por `a–c`
/// ([`ph2d_mesh::Face::tri_at`], que é a única resposta da casa a *"em que triângulos este quad se
/// parte?"* e **não** se toca). Um quad girado de um é o MESMO quad com a outra diagonal — então a
/// escolha cabe a quem o escreve, e o critério é a diagonal **mais curta**, que é a que minimiza o
/// empeno. Medido (§21): 72 triângulos ao contrário na cena 1 a prof 6 viram **0**.
fn emit_faces(
    grid: &Grid,
    m: usize,
    k: usize,
    planes: (&[f32], &[f32]),
    vidx: (&[u32], &[u32]),
    positions: &[[f32; 3]],
    faces: &mut Vec<Face>,
) {
    let n = grid.n;
    let (plane_lo, plane_hi) = planes;
    let (vidx_prev, vidx_cur) = vidx;
    let cell = |layer: &[u32], i: usize, j: usize| layer[j * n + i];
    let mut push = |quad: [u32; 4], flip: bool| {
        if quad.contains(&NO_VERT) {
            return;
        }
        let [a, b, c, d] = if flip {
            [quad[3], quad[2], quad[1], quad[0]]
        } else {
            quad
        };
        // Girar de um troca a diagonal `a–c` pela `b–d`, e preserva a rotação. A prioridade é a
        // metade que menos discorda do quad; havendo empate (o quad é plano), ganha a diagonal mais
        // curta, que é a que dá os triângulos menos finos.
        let (ac, bd) = (
            split_score(positions, [a, b, c, d]),
            split_score(positions, [b, c, d, a]),
        );
        // ⚠️ A tolerância existe para que um quad PLANO — em que as duas pontuações são iguais até
        // ao último bit — caia no critério de forma, e não no ruído do `f64`.
        let rotate = if (ac - bd).abs() > 1.0e-9 {
            bd > ac
        } else {
            d2(positions, b, d) < d2(positions, a, c)
        };
        if rotate {
            faces.push(Face::quad(b, c, d, a));
        } else {
            faces.push(Face::quad(a, b, c, d));
        }
    };

    for j in 0..n {
        for i in 0..n {
            let base = plane_lo[j * m + i];

            // Aresta em +z: as 4 células estão TODAS nesta camada. Mão direita de +z é (x, y).
            if i >= 1 && j >= 1 && (base < 0.0) != (plane_hi[j * m + i] < 0.0) {
                push(
                    [
                        cell(vidx_cur, i - 1, j - 1),
                        cell(vidx_cur, i, j - 1),
                        cell(vidx_cur, i, j),
                        cell(vidx_cur, i - 1, j),
                    ],
                    base >= 0.0,
                );
            }

            if k == 0 {
                continue;
            }

            // Aresta em +x: células nas camadas k−1 e k. Mão direita de +x é (y, z).
            if j >= 1 && (base < 0.0) != (plane_lo[j * m + i + 1] < 0.0) {
                push(
                    [
                        cell(vidx_prev, i, j - 1),
                        cell(vidx_prev, i, j),
                        cell(vidx_cur, i, j),
                        cell(vidx_cur, i, j - 1),
                    ],
                    base >= 0.0,
                );
            }

            // Aresta em +y: células nas camadas k−1 e k. Mão direita de +y é (z, x).
            if i >= 1 && (base < 0.0) != (plane_lo[(j + 1) * m + i] < 0.0) {
                push(
                    [
                        cell(vidx_prev, i - 1, j),
                        cell(vidx_cur, i - 1, j),
                        cell(vidx_cur, i, j),
                        cell(vidx_prev, i, j),
                    ],
                    base >= 0.0,
                );
            }
        }
    }
}
