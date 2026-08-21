//! **A COSTURA** — amostrar uma vez, montar, alisar, reprojetar.
//!
//! ⚠️ **A regra que governa este ficheiro inteiro:** *um ponto de fronteira
//! pertence ao ARCO, nunca ao patch.* Os dois patches que partilham um arco
//! pedem-lhe os mesmos índices — um deles ao contrário. Amostrar por patch daria
//! dois conjuntos de pontos quase iguais sobre a mesma curva, e a malha sairia
//! **rasgada** ao longo de toda fronteira de patch: um erro pequeno demais para
//! se ver num render e grande demais para uma malha ser usável.

use std::collections::BTreeMap;

use ph2d_mesh::{Face, Mesh};
use ph2d_quantize::Quantization;
use ph2d_trace::PatchLayout;

use crate::fan::{coons, resample, segment};

/// Por que a malha não pôde ser montada.
// ⚠️ Deixou de ser `Eq` quando a `ArcNotOfThisMesh` passou a carregar os dois
// comprimentos: sem eles, a recusa diria *que* não bate e não **quanto**, e a
// diferença entre `1,0001×` e `5,40×` é a diferença entre ruído e catástrofe.
#[derive(Debug, Clone, PartialEq)]
pub enum FillError {
    /// ⚠️ **A lei do patch não bate com a quantização.** `L_i` tinha de ser
    /// `e_{i-1} + e_{i+1}`, e não é. Isto é **bug a montante**, não uma
    /// propriedade da malha: o F4 devolve `e` que satisfazem a lei por
    /// construção, logo ou os lados vieram fora de ordem ou o `e` é de outro
    /// patch. Recusar em vez de remendar é o que impede uma malha torcida.
    Mismatch {
        /// Qual patch.
        patch: usize,
        /// Qual lado.
        side: usize,
        /// O que a lei exigia.
        expected: u32,
        /// O que os arcos somaram.
        got: u32,
    },
    /// Um lado não emenda no seguinte — a fronteira do patch não fecha.
    Broken {
        /// Qual patch.
        patch: usize,
        /// Qual lado.
        side: usize,
    },
    /// A malha resultante não monta.
    Mesh(String),
    /// ⭐⭐ **O LAYOUT NÃO É DESTA MALHA** — o defeito que destruiu o produto em
    /// 2026-08-21, e que nenhum dos 10.515 gates conseguia ver.
    ///
    /// ⚠️ **É a pré-condição mais barata que existe**, e ela existe porque o
    /// sintoma é invisível a jusante: um `arc_chain` de outra malha produz uma
    /// saída com **topologia perfeita** — 100 % quads, característica de Euler
    /// exacta, zero arestas de bordo, contagem de irregulares idêntica — e
    /// geometria destruída. *Nenhum número do [`FillReport`] muda.*
    ///
    /// A régua: o comprimento da polilinha de cada arco, medido **na malha que se
    /// vai amostrar**, tem de bater com o `arc_length` que o F3 declarou e que o
    /// F4 já usou para decidir quantos segmentos aquele arco leva. Medido: no
    /// caminho coerente a razão é **1,000 exacto** (é a mesma soma dos mesmos
    /// `f32`); no caminho destruído foi **5,40×**, com o pior arco a **9,04×**.
    /// *Três ordens de grandeza de margem — não há flake possível.*
    ArcNotOfThisMesh {
        /// Qual arco.
        arc: usize,
        /// O comprimento que o F3 declarou.
        declared: f32,
        /// O que a malha recebida de facto mede — ou `None` se um índice do arco
        /// nem sequer existe nela.
        measured: Option<f32>,
    },
}

/// O que a montagem mediu.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FillReport {
    /// Quantos quads.
    pub quads: usize,
    /// Quantas faces que **não** são quads. ⭐ Tem de ser **zero** — é a promessa
    /// inteira desta família de algoritmos.
    pub non_quads: usize,
    /// Quantos vértices.
    pub verts: usize,
    /// ⭐ Quantos vértices **irregulares** (valência ≠ 4). É a grandeza que o
    /// artista vê, e a que o pivô existiu para derrubar: a família local entregava
    /// 21 a 49 %, o oráculo entrega 0,2 %.
    pub irregular: usize,
    /// ⚠️ Quantas arestas ficaram com **uma** face só. Tem de ser zero numa
    /// superfície fechada: é o instrumento que denuncia a malha rasgada.
    pub boundary_edges: usize,
    /// Quantas rondas de alisamento correram.
    pub smoothing: usize,
    /// ⚠️ Quantas faces tiveram de ser **invertidas** para o volume ficar
    /// positivo. É `0` ou `todas` — qualquer outro número seria orientação
    /// inconsistente, que é outro defeito.
    pub flipped: usize,
    /// ⭐ **DE ONDE vêm os irregulares** — ver [`Provenance`]. Sem esta
    /// decomposição, `irregular: 47` diz que há trabalho e não diz **em que
    /// fase**: um irregular num canto do layout é dívida do F3, um no centro de
    /// um patch é da valência que o F3 entregou, e um no interior de uma grade
    /// seria um bug desta crate.
    pub by_provenance: [usize; Provenance::COUNT],
    /// ⭐⭐ **A ARESTA MAIS LONGA da saída** — e ela é a primeira grandeza
    /// GEOMÉTRICA que este relatório alguma vez teve.
    ///
    /// ⛔ **Todo o resto deste struct é função pura dos ÍNDICES.** `quads`,
    /// `non_quads`, `boundary_edges` e `irregular` saem da combinatória das faces;
    /// uma malha com as posições embaralhadas dá exactamente os mesmos números.
    /// Foi assim que 10.515 gates ficaram verdes sobre um produto destruído
    /// (auditoria de 2026-08-21): *não existia uma única asserção que olhasse uma
    /// coordenada.*
    ///
    /// A régua do chamador é a razão para o alvo dele. Medido: caminho correcto
    /// **≤ 4× o alvo**; caminho destruído **18×** — que era o **diâmetro da peça**,
    /// uma aresta a atravessar a esfera de lado a lado.
    pub edge_max: f32,
    /// A aresta mediana. ⭐ É a que diz se a DENSIDADE saiu no alvo — a máxima diz
    /// se alguma coisa se partiu, esta diz se a grade tem o passo pedido.
    pub edge_median: f32,
}

/// **De onde um vértice da saída veio** — a chave para saber de quem é a dívida.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provenance {
    /// Um **canto do layout**: onde três ou mais arcos se encontram. A valência
    /// dele é o número de arcos, e ⭐ **é o F3 que a decide** — cada junção em T
    /// que o traçado cria é um canto a mais.
    Corner,
    /// O interior de um **arco** partilhado. Deviam ser todos regulares.
    Arc,
    /// O **centro** de um patch. A valência é a do patch, logo um patch de 3 ou 5
    /// lados produz aqui um irregular **por construção** — é o preço do leque.
    Center,
    /// O interior de um **raio** do leque, do centro ao corte de um lado.
    Spoke,
    /// O interior de uma **grade** de Coons. ⛔ Um irregular aqui seria bug desta
    /// crate: uma grade regular não tem nenhum.
    Grid,
}

impl Provenance {
    /// Quantas classes existem.
    pub const COUNT: usize = 5;
    /// Os nomes, na ordem do array de [`FillReport::by_provenance`].
    pub const NAMES: [&'static str; Self::COUNT] =
        ["canto (F3)", "arco", "centro (F3)", "raio", "grade"];
}

/// **OS PONTOS DA SAÍDA, com a origem de cada um.**
///
/// ⭐ **Existe para que a posição e a proveniência não possam divergir.** A
/// primeira versão eram dois `Vec` a crescer lado a lado com um comentário a pedir
/// que assim continuassem — e um `push` esquecido num dos cinco sítios daria uma
/// decomposição deslocada, que **soma certo** e atribui a dívida à fase errada.
/// Aqui há um único `push`, e ele exige as duas coisas.
struct Points {
    pos: Vec<[f32; 3]>,
    prov: Vec<Provenance>,
}

impl Points {
    fn new() -> Self {
        Self {
            pos: Vec::new(),
            prov: Vec::new(),
        }
    }

    /// Acrescenta um ponto e devolve o índice dele.
    fn push(&mut self, p: [f32; 3], from: Provenance) -> u32 {
        self.pos.push(p);
        self.prov.push(from);
        u32::try_from(self.pos.len() - 1).unwrap_or(u32::MAX)
    }

    /// **Acrescenta um ponto POUSADO na superfície.**
    ///
    /// ⭐⭐ **É a diferença entre construir a grade NO ESPAÇO e construí-la SOBRE a
    /// forma**, e ela vale mais do que qualquer alisamento posterior.
    ///
    /// ⛔ **A primeira versão interpolava tudo em linha reta e deixava a
    /// reprojecção para o alisamento no fim.** Numa esfera de raio 1,0 a corda de
    /// um raio de leque mergulha para dentro, o Coons construído sobre cordas
    /// mergulhadas fica pior ainda, e as faces **dobram sobre si mesmas** —
    /// exactamente as fendas escuras que o Enio fotografou em 2026-08-21.
    ///
    /// Medido nessa esfera (4 922 quads), faces dobradas contra rondas de
    /// alisamento: `0 → 405 · 1 → 403 · 3 → 289 · 6 → 205 · 12 → 135`. ⭐ **O
    /// alisamento REPARA e não CAUSA** — ele nunca chega a zero porque o estrago
    /// já veio pronto da construção. *Um remédio que melhora monotonicamente e não
    /// cura está a tratar o sintoma.*
    fn push_on(&mut self, mesh: &Mesh, p: [f32; 3], seed: f32, from: Provenance) -> u32 {
        self.push(ph2d_remesh_iso::project_onto(mesh, p, seed), from)
    }
}

/// ⚠️ **Quantas rondas de alisamento por omissão.** Elas não mudam a topologia —
/// só onde os pontos ficam — e cada uma **reprojeta** sobre a superfície de
/// referência, então a forma não escorre. O número está aqui e não numa opinião:
/// ver a tabela do `PLAN.md` §4-sexies.
pub const SMOOTHING_ROUNDS: usize = 6;

/// **A TOLERÂNCIA da pré-condição**, em fração do comprimento declarado.
///
/// ⚠️ **Ela é folga de ARREDONDAMENTO e nada mais.** No caminho correto os dois
/// números são a **mesma soma dos mesmos `f32`**, então a razão é `1,000` exacto;
/// `1e-3` cobre uma reordenação de soma e ainda deixa **três ordens de grandeza**
/// até o `5,40×` que o defeito produziu. ⛔ Alargá-la não compra robustez nenhuma:
/// compra o direito de voltar a montar uma malha sobre índices de outra.
const ARC_LENGTH_TOLERANCE: f32 = 1.0e-3;

/// **O LAYOUT É DESTA MALHA?** — a pré-condição do [`fill`].
///
/// ⭐ **Ela responde à única pergunta que a montagem não pode responder sozinha**,
/// e responde-a com aritmética que já está paga: o F3 mediu o comprimento de cada
/// arco quando o traçou, e o F4 usou esse número para decidir a quantização. Se a
/// malha que chega aqui medir outra coisa, o `arc_chain` **não é dela**.
///
/// ⚠️ **E ela absorve de graça o segundo defeito da mesma família:** quando o F1
/// REFINA em vez de grosseirar (toda entrada mais grossa que ~2.500 vértices), o
/// índice sai do alcance e o `src[v]` **panica** — a janela morre com a peça por
/// gravar. Aqui o mesmo `get` devolve uma recusa nomeada. *Reproduzido: o SEGUNDO
/// clique do botão era panic certo.*
fn check_arcs_belong_to(mesh: &Mesh, layout: &PatchLayout) -> Result<(), FillError> {
    let pos = mesh.positions();
    for (a, chain) in layout.arc_chain.iter().enumerate() {
        let declared = layout.arc_length.get(a).copied().unwrap_or(0.0);
        let mut measured = 0.0f32;
        for w in chain.windows(2) {
            // ⚠️ `get` e não `[]`: um índice fora do alcance é a MESMA doença, e
            // um panic no meio de um gesto do artista é a pior forma de a dizer.
            let (Some(a0), Some(a1)) = (pos.get(w[0] as usize), pos.get(w[1] as usize)) else {
                return Err(FillError::ArcNotOfThisMesh {
                    arc: a,
                    declared,
                    measured: None,
                });
            };
            let d = [a1[0] - a0[0], a1[1] - a0[1], a1[2] - a0[2]];
            measured += d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt();
        }
        if (measured - declared).abs() > ARC_LENGTH_TOLERANCE * declared.max(1.0e-6) {
            return Err(FillError::ArcNotOfThisMesh {
                arc: a,
                declared,
                measured: Some(measured),
            });
        }
    }
    Ok(())
}

/// **MONTA A MALHA DE QUADS.**
///
/// # Os DOIS mesh, e por que eles são dois parâmetros
///
/// ⭐ **`indexed` é a malha que o `layout` INDEXA; `surface` é onde o resultado
/// pousa.** Elas são a mesma coisa em quase todo chamador — e na cadeia do produto
/// **não são**: o traçado corre sobre a saída da remalha isotrópica (F1), que tem
/// espaço de índice próprio, enquanto a forma que o artista esculpiu vive na malha
/// original.
///
/// ⛔ **Isto era UM parâmetro chamado `reference`, e a confusão custou o produto
/// inteiro** (auditoria de 2026-08-21). A porta do shell passou-lhe a malha
/// original — raciocinando, **corretamente**, sobre o papel de reprojeção — e com
/// isso cada `arc_chain[i]`, que é um índice da malha remalhada, foi ler a posição
/// de um vértice **arbitrário** da original. Medido: aresta mediana **4,6× o
/// alvo**, aresta máxima **2,01 numa peça de raio 1,0** — o diâmetro, uma aresta a
/// atravessar a esfera de lado a lado. *E os quatro números do relatório saíram
/// **bit-a-bit iguais** aos da corrida correta.*
///
/// ⚠️ **A cura não foi trocar o argumento: foi partir o parâmetro**, porque a
/// assinatura antiga **não permitia exprimir** a intenção certa. Um erro que a
/// assinatura torna inexprimível não precisa de gate.
///
/// # Errors
/// [`FillError`] quando a estrutura a montante não fecha — ver as variantes.
pub fn fill(
    indexed: &Mesh,
    surface: &Mesh,
    layout: &PatchLayout,
    quant: &Quantization,
    smoothing: usize,
) -> Result<(Mesh, FillReport), FillError> {
    // ⭐⭐ **A PRÉ-CONDIÇÃO, e ela é a mais barata que existe.** Ver
    // [`check_arcs_belong_to`].
    check_arcs_belong_to(indexed, layout)?;
    let src = indexed.positions();
    // ⭐⭐ **TODO ponto de INTERIOR nasce POUSADO na superfície.** Ver
    // [`Points::push_on`].
    let seed = bbox_seed(surface);
    let mut pts = Points::new();
    let mut faces: Vec<Face> = Vec::new();

    // ── 1. Os pontos de cada ARCO, amostrados UMA vez.
    let mut corner_vid: BTreeMap<u32, u32> = BTreeMap::new();
    let mut arc_points: Vec<Vec<u32>> = Vec::with_capacity(layout.arc_chain.len());
    for (a, chain) in layout.arc_chain.iter().enumerate() {
        let n = quant.arc[a].max(1) as usize;
        let curve: Vec<[f32; 3]> = chain.iter().map(|&v| src[v as usize]).collect();
        let sampled = resample(&curve, n);
        let mut ids = Vec::with_capacity(n + 1);
        for (k, p) in sampled.iter().enumerate() {
            // As pontas são cantos de malha, e um canto é de TODOS os arcos que
            // lá chegam — por isso a chave é o vértice, não o par (arco, índice).
            let id = if k == 0 || k == n {
                let v = if k == 0 {
                    chain[0]
                } else {
                    *chain.last().unwrap_or(&chain[0])
                };
                *corner_vid
                    .entry(v)
                    .or_insert_with(|| pts.push(src[v as usize], Provenance::Corner))
            } else {
                pts.push(*p, Provenance::Arc)
            };
            ids.push(id);
        }
        arc_points.push(ids);
    }

    // ── 2. Cada patch vira o seu leque.
    for (p, sides) in layout.side_arcs.iter().enumerate() {
        let n = sides.len();
        let e = &quant.corners[p];
        if n < 3 || e.len() != n {
            return Err(FillError::Mismatch {
                patch: p,
                side: 0,
                expected: u32::try_from(n).unwrap_or(0),
                got: u32::try_from(e.len()).unwrap_or(0),
            });
        }
        // Os pontos de cada lado, do canto de entrada ao de saída.
        let mut side_pts: Vec<Vec<u32>> = Vec::with_capacity(n);
        for (i, side) in sides.iter().enumerate() {
            let mut pts: Vec<u32> = Vec::new();
            for &(a, rev) in side {
                let mut ids = arc_points[a as usize].clone();
                if rev {
                    ids.reverse();
                }
                if pts.is_empty() {
                    pts = ids;
                } else {
                    if *pts.last().unwrap_or(&u32::MAX) != ids[0] {
                        return Err(FillError::Broken { patch: p, side: i });
                    }
                    pts.extend_from_slice(&ids[1..]);
                }
            }
            side_pts.push(pts);
        }
        // ⚠️ A lei, conferida: se ela não bate aqui, o resto é geometria torcida.
        for i in 0..n {
            let want = e[(i + n - 1) % n] + e[(i + 1) % n];
            let got = u32::try_from(side_pts[i].len() - 1).unwrap_or(u32::MAX);
            if want != got {
                return Err(FillError::Mismatch {
                    patch: p,
                    side: i,
                    expected: want,
                    got,
                });
            }
            if side_pts[i].last() != side_pts[(i + 1) % n].first() {
                return Err(FillError::Broken { patch: p, side: i });
            }
        }

        // O centro: a média dos pontos de fronteira, reprojetada.
        let mut c = [0.0f32; 3];
        let mut count = 0usize;
        for side in &side_pts {
            for &v in &side[..side.len() - 1] {
                let q = pts.pos[v as usize];
                for k in 0..3 {
                    c[k] += q[k];
                }
                count += 1;
            }
        }
        #[allow(clippy::cast_precision_loss)]
        let inv = if count == 0 { 0.0 } else { 1.0 / count as f32 };
        let center_vid = pts.push_on(
            surface,
            [c[0] * inv, c[1] * inv, c[2] * inv],
            seed,
            Provenance::Center,
        );

        // Os raios: do centro ao corte de cada lado.
        let mut spoke: Vec<Vec<u32>> = Vec::with_capacity(n);
        for i in 0..n {
            let cut = e[(i + n - 1) % n] as usize;
            let tip = side_pts[i][cut];
            let steps = e[i] as usize;
            let line = segment(pts.pos[center_vid as usize], pts.pos[tip as usize], steps);
            let mut ids = Vec::with_capacity(steps + 1);
            ids.push(center_vid);
            for q in line.iter().take(steps).skip(1) {
                ids.push(pts.push_on(surface, *q, seed, Provenance::Spoke));
            }
            ids.push(tip);
            spoke.push(ids);
        }

        // As `n` grades.
        for i in 0..n {
            let j = (i + 1) % n;
            let (s, t) = (e[j] as usize, e[i] as usize);
            let cut_i = e[(i + n - 1) % n] as usize;
            let cut_j = e[i] as usize;
            let bottom = &side_pts[i][cut_i..];
            let right = &side_pts[j][..=cut_j];
            let top = &spoke[j];
            let left: Vec<u32> = spoke[i].iter().rev().copied().collect();
            let grid = build_grid(&mut pts, surface, seed, bottom, top, &left, right, s, t);
            for k in 0..s {
                for l in 0..t {
                    faces.push(Face::quad(
                        grid[k][l],
                        grid[k + 1][l],
                        grid[k + 1][l + 1],
                        grid[k][l + 1],
                    ));
                }
            }
        }
    }

    // ── 3. A malha, com a orientação conferida.
    let mut flipped = 0usize;
    if signed_volume(&pts.pos, &faces) < 0.0 {
        // ⚠️ **Ou todas ou nenhuma.** A orientação da grade sai da orientação da
        // fronteira do patch, que é a mesma para todos os patches; se ela estiver
        // ao contrário, está ao contrário em bloco. Inverter face a face por
        // teste local é que produziria uma malha inconsistente.
        for f in &mut faces {
            let v = f.verts().to_vec();
            *f = Face::quad(v[3], v[2], v[1], v[0]);
        }
        flipped = faces.len();
    }
    let mut mesh =
        Mesh::from_parts(pts.pos, faces).map_err(|e| FillError::Mesh(format!("{e:?}")))?;

    // ── 4. Alisar, reprojetando sempre.
    for _ in 0..smoothing {
        smooth_once(&mut mesh, surface);
    }

    let report = measure(&mesh, &pts.prov, smoothing, flipped);
    Ok((mesh, report))
}

/// Os índices da grade: bordos vindos das curvas, interior por Coons.
#[allow(clippy::too_many_arguments)]
fn build_grid(
    pts: &mut Points,
    surface: &Mesh,
    seed: f32,
    bottom: &[u32],
    top: &[u32],
    left: &[u32],
    right: &[u32],
    s: usize,
    t: usize,
) -> Vec<Vec<u32>> {
    let at = |ids: &[u32], k: usize, pos: &[[f32; 3]]| pos[ids[k] as usize];
    let b: Vec<[f32; 3]> = (0..=s).map(|k| at(bottom, k, &pts.pos)).collect();
    let tp: Vec<[f32; 3]> = (0..=s).map(|k| at(top, k, &pts.pos)).collect();
    let l: Vec<[f32; 3]> = (0..=t).map(|k| at(left, k, &pts.pos)).collect();
    let r: Vec<[f32; 3]> = (0..=t).map(|k| at(right, k, &pts.pos)).collect();
    let inner = coons(&b, &tp, &l, &r);
    let mut grid = vec![vec![0u32; t + 1]; s + 1];
    for (k, row) in grid.iter_mut().enumerate() {
        for (l_i, cell) in row.iter_mut().enumerate() {
            *cell = if l_i == 0 {
                bottom[k]
            } else if l_i == t {
                top[k]
            } else if k == 0 {
                left[l_i]
            } else if k == s {
                right[l_i]
            } else {
                pts.push_on(surface, inner[k][l_i], seed, Provenance::Grid)
            };
        }
    }
    grid
}

/// Um passo de Laplaciano tangencial, seguido de reprojeção.
fn smooth_once(mesh: &mut Mesh, reference: &Mesh) {
    let n = mesh.vert_count();
    let neighbours: Vec<Vec<u32>> = {
        let adj = mesh.adjacency();
        (0..n)
            .map(|v| adj.vert_verts.neighbours(v).to_vec())
            .collect()
    };
    let normals: Vec<[f32; 3]> = mesh.normals().to_vec();
    let seed = bbox_seed(reference);
    let mut next = vec![[0.0f32; 3]; n];
    {
        let pos = mesh.positions();
        for v in 0..n {
            let ns = &neighbours[v];
            if ns.len() < 3 {
                next[v] = pos[v];
                continue;
            }
            let mut sum = [0.0f32; 3];
            for &w in ns {
                let q = pos[w as usize];
                for k in 0..3 {
                    sum[k] += q[k];
                }
            }
            #[allow(clippy::cast_precision_loss)]
            let inv = 1.0 / ns.len() as f32;
            let p = pos[v];
            let d = [
                sum[0].mul_add(inv, -p[0]),
                sum[1].mul_add(inv, -p[1]),
                sum[2].mul_add(inv, -p[2]),
            ];
            // ⚠️ **Só a parte TANGENTE.** A componente normal encolheria a peça a
            // cada ronda, e a reprojeção a seguir esconderia o encolhimento sem o
            // desfazer.
            let nv = normals[v];
            let along = d[0].mul_add(nv[0], d[1].mul_add(nv[1], d[2] * nv[2]));
            next[v] = [
                LAMBDA.mul_add(along.mul_add(-nv[0], d[0]), p[0]),
                LAMBDA.mul_add(along.mul_add(-nv[1], d[1]), p[1]),
                LAMBDA.mul_add(along.mul_add(-nv[2], d[2]), p[2]),
            ];
        }
    }
    for q in &mut next {
        *q = ph2d_remesh_iso::project_onto(reference, *q, seed);
    }
    mesh.positions_mut().copy_from_slice(&next);
    mesh.rebuild();
}

/// Meio passo — o amortecimento que o torna monótono.
const LAMBDA: f32 = 0.5;

/// O raio inicial da busca de reprojeção: uma fração da diagonal da caixa.
fn bbox_seed(mesh: &Mesh) -> f32 {
    let b = mesh.bounds();
    let d = [
        b.max[0] - b.min[0],
        b.max[1] - b.min[1],
        b.max[2] - b.min[2],
    ];
    d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt() * 0.02
}

/// Volume com sinal — o que diz se a orientação saiu ao contrário.
fn signed_volume(pos: &[[f32; 3]], faces: &[Face]) -> f32 {
    let mut total = 0.0f32;
    for f in faces {
        let v = f.verts();
        for k in 1..v.len() - 1 {
            let (a, b, c) = (
                pos[v[0] as usize],
                pos[v[k] as usize],
                pos[v[k + 1] as usize],
            );
            total += a[0].mul_add(
                b[1].mul_add(c[2], -(b[2] * c[1])),
                a[1].mul_add(
                    b[2].mul_add(c[0], -(b[0] * c[2])),
                    a[2] * b[0].mul_add(c[1], -(b[1] * c[0])),
                ),
            );
        }
    }
    total / 6.0
}

/// As grandezas que o relatório carrega.
fn measure(mesh: &Mesh, prov: &[Provenance], smoothing: usize, flipped: usize) -> FillReport {
    let faces = mesh.faces();
    let quads = faces.iter().filter(|f| !f.is_tri()).count();
    let mut count: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for f in faces {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            *count.entry((a.min(b), a.max(b))).or_default() += 1;
        }
    }
    let boundary_edges = count.values().filter(|&&c| c == 1).count();
    let adj = mesh.adjacency();
    // ⭐ **As duas grandezas geométricas**, medidas sobre as arestas da saída.
    let mut lens: Vec<f32> = Vec::with_capacity(count.len());
    let pos = mesh.positions();
    for (a, b) in count.keys() {
        let (p, q) = (pos[*a as usize], pos[*b as usize]);
        let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
        lens.push(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt());
    }
    lens.sort_by(f32::total_cmp);
    let edge_max = lens.last().copied().unwrap_or(0.0);
    let edge_median = lens.get(lens.len() / 2).copied().unwrap_or(0.0);

    let mut by_provenance = [0usize; Provenance::COUNT];
    let irregular = (0..mesh.vert_count())
        .filter(|&v| !adj.is_border(v) && adj.valence(v) != 4)
        .inspect(|&v| {
            if let Some(p) = prov.get(v) {
                by_provenance[*p as usize] += 1;
            }
        })
        .count();
    FillReport {
        by_provenance,
        edge_max,
        edge_median,
        quads,
        non_quads: faces.len() - quads,
        verts: mesh.vert_count(),
        irregular,
        boundary_edges,
        smoothing,
        flipped,
    }
}
