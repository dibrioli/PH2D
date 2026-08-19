//! **A EXTRAÇÃO** — os dois campos viram uma malha (ADR-0160 §5, Q3).
//!
//! O campo de posição coloca uma origem de retícula por vértice, consistente **a
//! menos de passos inteiros** — e é **aqui** que se quocienta pela retícula. Três
//! passos, e cada um responde a uma pergunta da malha nova:
//!
//! 1. **as CÉLULAS** — que vértices da entrada descrevem o mesmo nó da grade?
//!    (união sobre as arestas; um `union-find`);
//! 2. **as ARESTAS** — que células são vizinhas? (as arestas da entrada que
//!    atravessam células);
//! 3. **as FACES** — que ciclos o grafo fecha? (o *sistema de rotação*: em cada
//!    célula os vizinhos são ordenados por ângulo no plano tangente, e a face é
//!    o passeio que sempre vira para o mesmo lado).
//!
//! ⚠️ **O passo 3 é o que faz a saída ser uma MALHA e não um grafo.** Um grafo de
//! quads não tem faces: quem as tem é a *imersão*, e a ordem angular em torno de
//! cada célula é exatamente ela. Procurar ciclos de comprimento 4 no grafo
//! (a alternativa óbvia) acharia também as diagonais de um cubo — quatro
//! vértices que se ligam em ciclo **sem** delimitar face nenhuma.
//!
//! # ⚠️ O que esta onda NÃO garante, de propósito
//!
//! **`all-quad` ainda não é verdade aqui, e o ADR-0160 §5 diz que a Q3 fecha com
//! um NÚMERO em vez de um zero.** Onde os índices de singularidade não fecham, o
//! passeio devolve ciclos de 3, 5 ou mais — é o defeito conhecido da família
//! Instant Meshes, e é exatamente o que o fluxo de custo mínimo da **Q4** existe
//! para curar. Declarar zero antes daquele passo seria declarar que a técnica
//! base não tem o defeito que a literatura inteira nomeia.
//!
//! A [`Quadrangulation`] carrega a contagem, e é ela que a Q4 tem de baixar.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_mesh::{Face, Mesh, MeshError};

use crate::orientation::OrientationField;
use crate::position::PositionField;
use crate::scale::ScaleField;

/// **A malha nova, com a conta do que não saiu quad.**
///
/// ⚠️ **A contagem viaja com o resultado e não num log**: ela é o número que a
/// Q4 tem de baixar, e um número que só existe num `eprintln!` não é comparável
/// entre duas corridas.
#[derive(Clone, Debug)]
pub struct Quadrangulation {
    /// A malha extraída.
    pub mesh: Mesh,
    /// Quantas faces saíram com **quatro** lados.
    pub quads: usize,
    /// Quantas **não** saíram — o alvo da Q4.
    pub non_quads: usize,
    /// O maior número de lados que um ciclo teve.
    pub max_sides: usize,
}

impl Quadrangulation {
    /// A fração das FACES DA MALHA que saiu quad — a régua de uma corrida contra
    /// a seguinte.
    ///
    /// ⚠️ **Sobre as faces EMITIDAS, e não sobre os ciclos, e a diferença já
    /// mentiu uma vez.** A primeira versão dividia `quads / (quads + ciclos
    /// não-quad)` — e um ciclo de **31 lados** contava como **um** não-quad
    /// enquanto virava **29 triângulos** na malha. A régua *melhorava* quando as
    /// falhas ficavam maiores: uma mudança que trocou 582 triângulos por 918
    /// aparecia como uma subida de 60,9 % para 71,9 %. *Uma régua que premeia o
    /// defeito por ele ser grande é pior que nenhuma.*
    #[must_use]
    pub fn quad_fraction(&self) -> f64 {
        let total = self.mesh.faces().len();
        if total == 0 {
            0.0
        } else {
            self.quads as f64 / total as f64
        }
    }
}

/// **EXTRAI a malha** a partir da entrada e dos dois campos.
///
/// ⚠️ **`BTreeMap`/`BTreeSet` e nunca `HashMap`** — a saída tem de ser
/// byte-idêntica entre corridas (HR-5, e a asserção A7 do ADR-0160). Uma tabela
/// de hash faria a ordem das arestas e das faces depender da semente do
/// processo, e a malha do artista mudaria ao reabrir o projeto.
///
/// # Erros
///
/// Devolve [`MeshError`] se o conjunto de faces extraído não formar uma malha
/// bem formada (um índice fora de alcance) — o que só acontece se a fusão de
/// células tiver produzido um grafo degenerado.
pub fn extract(
    mesh: &Mesh,
    orient: &OrientationField,
    pos: &PositionField,
    scale: &ScaleField,
) -> Result<Quadrangulation, MeshError> {
    // ⚠️ **`Seed`, e a alternativa está MEDIDA e REJEITADA** — ver o doc da
    // [`Clustering::Lattice`].
    extract_with(mesh, orient, pos, scale, Clustering::Seed)
}

/// **Como os vértices viram células** — as duas leis, medidas uma contra a outra.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clustering {
    /// Bola no campo em torno de uma semente. Não precisa de platôs.
    Seed,
    /// **O QUOCIENTE PELA RETÍCULA** — dois vizinhos são a mesma célula quando o
    /// passo inteiro entre as retículas deles é `(0,0)`.
    ///
    /// ⛔ **MEDIDO E REJEITADO (2026-08-19).** Ele COLAPSA: a esfera de 3 072
    /// vértices sai com **1 célula**, o toro com **5** — e a hierarquia não muda
    /// isso (2 e 9). A razão é aritmética e mata a ideia inteira: o campo fica a
    /// menos de `s/√2` do seu vértice (gate `the_field_never_leaves_its_own_cell`)
    /// e os vértices vizinhos distam **muito menos que uma célula** (0,098 contra
    /// 0,18 na fixture) — então o passo inteiro entre duas retículas vizinhas é
    /// `(0,0)` em toda parte, a relação é universal, e o `union-find` funde tudo.
    ///
    /// ⚠️ **Fica no código como recusa com ENDEREÇO**, não como opção: quem
    /// voltar a esta ideia — e ela é a leitura natural da referência — encontra o
    /// número antes de a reconstruir.
    Lattice,
}

/// A extração com a lei de agrupamento aberta — a porta da sonda.
///
/// # Erros
/// Ver [`extract`].
pub fn extract_with(
    mesh: &Mesh,
    orient: &OrientationField,
    pos: &PositionField,
    scale: &ScaleField,
    how: Clustering,
) -> Result<Quadrangulation, MeshError> {
    let cells = match how {
        Clustering::Seed => cluster(mesh, pos, scale),
        Clustering::Lattice => cluster_lattice(mesh, orient, pos, scale),
    };
    let c = collapse(mesh, pos, orient, &cells);
    let graph = neighbour_graph(mesh, &c, scale);
    let cycles = trace_faces(&graph, &c.verts, &c.normals, orient);
    let verts = c.verts.clone();

    let (mut faces, mut quads, mut non_quads, mut max_sides) = (Vec::new(), 0usize, 0usize, 0usize);
    for c in &cycles {
        max_sides = max_sides.max(c.len());
        match c.len() {
            4 => {
                quads += 1;
                faces.push(Face::quad(c[0], c[1], c[2], c[3]));
            }
            3 => {
                non_quads += 1;
                faces.push(Face::tri(c[0], c[1], c[2]));
            }
            n if n > 4 => {
                // ⚠️ **O leque é HONESTO e não uma cura:** um n-gon vira `n−2`
                // triângulos porque a [`Face`] só carrega tri e quad, e cada um
                // deles CONTA como não-quad. Escondê-lo (descartar o ciclo)
                // deixaria um buraco na malha e um número bonito ao lado.
                non_quads += 1;
                for k in 1..c.len() - 1 {
                    faces.push(Face::tri(c[0], c[k], c[k + 1]));
                }
            }
            // Um ciclo de 2 ou menos não delimita área: ele é um artefato da
            // fusão, e entrar na malha seria uma face degenerada.
            _ => non_quads += 1,
        }
    }

    Ok(Quadrangulation {
        mesh: Mesh::from_parts(verts, faces)?,
        quads,
        non_quads,
        max_sides,
    })
}

/// **AS CÉLULAS** — crescimento por SEMENTE, com diâmetro limitado.
///
/// Cada célula nasce num vértice ainda livre e cresce pela malha enquanto o campo
/// dos candidatos ficar a **menos de meia célula da SEMENTE** — nunca do vizinho.
///
/// ⚠️ **A distinção é a wave inteira, e a primeira versão morreu nela.** Ela
/// unia `i` e `j` quando os campos DELES estavam perto, num `union-find` — e
/// união é **transitiva**: uma corrente de arestas curtas funde tudo o que ela
/// tocar. Medido: a esfera de 3 072 vértices saiu com **4 células** (`χ = 4`
/// sobre uma entrada que vale 2). E a corrente **sempre existe** quando o quad
/// pedido é maior que a aresta da malha — que é o caso normal de um remesh.
///
/// Comparar contra a SEMENTE limita o diâmetro a uma célula por construção: o
/// que cresce é uma bola no campo, não uma componente conexa de uma relação.
///
/// ⚠️ **Ordem de sementes = ordem de índice**, e é o que torna a rotulagem
/// determinística (A7). Uma escolha "melhor" de semente (a de maior valência, a
/// mais central) seria uma heurística a mais para justificar e medir; a Q4 é que
/// tem o instrumento para dizer se ela paga.
///
/// ⚠️ **Isto NÃO é o que a referência faz.** O Instant Meshes quocienta pela
/// retícula porque o campo de posição dele forma PLATÔS — e os platôs vêm da
/// hierarquia multirresolução, que a Q2 não construiu (o gate
/// `the_field_never_leaves_its_own_cell` mede o campo a variar continuamente).
/// Sem platôs não há retícula partilhada a que agarrar, e o crescimento por
/// semente é a resposta honesta a *"que células existem"* com o campo que se
/// tem. **A hierarquia é pré-requisito da Q4**, e este doc é o registro de por
/// quê.
fn cluster(mesh: &Mesh, pos: &PositionField, scale: &ScaleField) -> Vec<u32> {
    let n = mesh.vert_count();
    let adj = mesh.adjacency();
    let mut cell = vec![u32::MAX; n];
    let mut next = 0u32;
    let mut queue: Vec<usize> = Vec::new();

    for seed in 0..n {
        if cell[seed] != u32::MAX {
            continue;
        }
        let c = next;
        next += 1;
        cell[seed] = c;
        let o0 = pos.at(seed);
        let radius = 0.5 * scale.at(seed);

        queue.clear();
        queue.push(seed);
        let mut head = 0;
        while head < queue.len() {
            let v = queue[head];
            head += 1;
            for &w in adj.vert_verts.neighbours(v) {
                let w = w as usize;
                if cell[w] != u32::MAX {
                    continue;
                }
                // ⚠️ Contra a SEMENTE (`o0`), nunca contra `v`.
                if dist(pos.at(w), o0) < radius {
                    cell[w] = c;
                    queue.push(w);
                }
            }
        }
    }
    cell
}

/// **As células, colapsadas** — o que o [`collapse`] devolve.
///
/// ⚠️ Um struct e não uma tupla de quatro `Vec`: os quatro campos são indexados
/// pela MESMA coisa (a célula), menos o último, e uma tupla deixa isso por
/// descobrir em cada sítio de uso.
struct Cells {
    /// A posição do nó de cada célula.
    verts: Vec<[f32; 3]>,
    /// A normal média de cada célula.
    normals: Vec<[f32; 3]>,
    /// A direção da cruz de cada célula — a da semente, re-projetada.
    dirs: Vec<[f32; 3]>,
    /// Vértice da ENTRADA → célula de saída.
    of: Vec<u32>,
}

/// **AS CÉLULAS pelo QUOCIENTE DA RETÍCULA** — a lei da referência.
///
/// Dois vértices vizinhos são a **mesma** célula quando o passo inteiro entre as
/// retículas deles é `(0,0)` — ou seja, quando `position_round_4` do campo de um,
/// ancorado no campo do outro, **não anda**. É uma relação de equivalência, e o
/// `union-find` a fecha.
///
/// ⚠️ **Ela SÓ funciona sobre um campo com PLATÔS**, e é essa dependência que
/// liga esta função à hierarquia: sem platôs a relação é *"os campos estão
/// perto"*, que é transitiva e funde o modelo inteiro (medido: 4 células numa
/// esfera de 3 072 vértices). Com platôs, as células são limitadas pelos degraus
/// do próprio campo.
fn cluster_lattice(
    mesh: &Mesh,
    orient: &OrientationField,
    pos: &PositionField,
    scale: &ScaleField,
) -> Vec<u32> {
    let n = mesh.vert_count();
    let normals = mesh.normals();
    let adj = mesh.adjacency();
    let mut uf: Vec<u32> = (0..n as u32).collect();
    for (v, nv) in normals.iter().enumerate().take(n) {
        for &w in adj.vert_verts.neighbours(v) {
            let w = w as usize;
            if w <= v {
                continue;
            }
            // O campo de `w`, reduzido à retícula de `v`: se ele não andou, os
            // dois descrevem o MESMO nó.
            let snapped = crate::position::position_round_4(
                pos.at(v),
                orient.dir(v),
                *nv,
                pos.at(w),
                scale.at(v),
            );
            if dist(snapped, pos.at(v)) < 1.0e-4 * scale.at(v).max(1.0) {
                let (a, b) = (uf_find(&mut uf, v as u32), uf_find(&mut uf, w as u32));
                if a != b {
                    let (lo, hi) = if a < b { (a, b) } else { (b, a) };
                    uf[hi as usize] = lo;
                }
            }
        }
    }
    (0..n as u32).map(|v| uf_find(&mut uf, v)).collect()
}

fn uf_find(uf: &mut [u32], mut x: u32) -> u32 {
    while uf[x as usize] != x {
        uf[x as usize] = uf[uf[x as usize] as usize];
        x = uf[x as usize];
    }
    x
}

/// **O NÓ DE CADA CÉLULA** — a média das origens que caíram nela, e a normal
/// média.
///
/// Devolve as [`Cells`].
///
/// ⚠️ **A média das ORIGENS e não a dos vértices.** As origens já são a resposta
/// do campo sobre onde o nó da grade está; mediar os vértices devolveria o
/// centroide do retalho, que é outro ponto — e a grade encolheria para dentro da
/// forma em toda quina convexa.
fn collapse(mesh: &Mesh, pos: &PositionField, orient: &OrientationField, cells: &[u32]) -> Cells {
    let mut order: BTreeMap<u32, u32> = BTreeMap::new();
    let mut of = vec![0u32; cells.len()];
    for (v, &c) in cells.iter().enumerate() {
        let next = order.len() as u32;
        of[v] = *order.entry(c).or_insert(next);
    }

    let k = order.len();
    let (mut p, mut nrm, mut count) = (vec![[0.0f32; 3]; k], vec![[0.0f32; 3]; k], vec![0.0f32; k]);
    // ⚠️ **A orientação da célula é a da SEMENTE, não uma média.** Médias de
    // campos 4-RoSy exigem reduzir cada termo ao representante mais próximo do
    // acumulador — trabalho que o `solve_orientation` já fez sobre a malha
    // inteira. Reduzi-lo outra vez aqui seria a segunda resposta a *"que direção
    // tem esta região?"*, e as duas divergiriam nas singularidades, que é
    // exatamente onde a grade decide.
    let mut dir = vec![[0.0f32; 3]; k];
    let mut seeded = vec![false; k];
    let mesh_n = mesh.normals();
    for (v, &c) in of.iter().enumerate() {
        let c = c as usize;
        if !seeded[c] {
            dir[c] = orient.dir(v);
            seeded[c] = true;
        }
        for i in 0..3 {
            p[c][i] += pos.at(v)[i];
            nrm[c][i] += mesh_n[v][i];
        }
        count[c] += 1.0;
    }
    for c in 0..k {
        let inv = 1.0 / count[c];
        for i in 0..3 {
            p[c][i] *= inv;
            nrm[c][i] *= inv;
        }
        nrm[c] = normalize_or(nrm[c], [0.0, 0.0, 1.0]);
        // A direção da semente, re-projetada na normal MÉDIA da célula: ela foi
        // tangente ao vértice-semente, e a célula tem plano próprio.
        let d = dot(dir[c], nrm[c]);
        dir[c] = normalize_or(
            [
                d.mul_add(-nrm[c][0], dir[c][0]),
                d.mul_add(-nrm[c][1], dir[c][1]),
                d.mul_add(-nrm[c][2], dir[c][2]),
            ],
            tangent_of(nrm[c]),
        );
    }
    Cells {
        verts: p,
        normals: nrm,
        dirs: dir,
        of,
    }
}

/// **AS ARESTAS — as da RETÍCULA, não as da entrada.**
///
/// Cada célula liga-se à melhor candidata em **cada uma das quatro direções** da
/// cruz local (`±q`, `±(n×q)`), entre as células que a malha de entrada torna
/// vizinhas.
///
/// ⚠️ **A primeira versão ligava toda aresta da entrada que atravessasse
/// células, e o resultado foi medido: 7,2 % de quads.** É aritmética, não azar —
/// a entrada é uma triangulação, cada célula herdava ~6 vizinhas, e o passeio de
/// faces devolvia **triângulos**. Uma grade de quads tem quatro vizinhas por nó,
/// e quem as escolhe é a **cruz**: é essa a única coisa nesta crate que sabe o
/// que "as duas famílias perpendiculares" quer dizer.
///
/// ⚠️ **Os dois limiares são derivados, não escolhidos:**
/// - o **ângulo** (`cos 45°`) é o que reparte o plano em quatro quadrantes sem
///   sobreposição nem buraco: com mais, duas direções disputariam a mesma
///   candidata; com menos, sobraria plano sem dono.
/// - a **distância** (`[0,5 s, 1,7 s]`) é uma célula, com folga para os dois
///   lados: abaixo do piso a candidata é a própria célula mal separada, acima do
///   teto ela é a **segunda** célula na mesma direção — e ligar a segunda salta
///   uma linha da grade.
///
/// ⚠️ **O grafo sai SIMÉTRICO** (as duas pontas inserem), porque uma aresta que
/// só um lado reivindica quebra o sistema de rotação: o passeio de faces chega
/// por ela e não a encontra na ordem angular do destino.
fn neighbour_graph(mesh: &Mesh, c: &Cells, scale: &ScaleField) -> Vec<BTreeSet<u32>> {
    let (verts, normals, dirs, of) = (&c.verts, &c.normals, &c.dirs, &c.of);
    let k = verts.len();

    // As candidatas: as células que a entrada torna vizinhas.
    let mut near = vec![BTreeSet::new(); k];
    let adj = mesh.adjacency();
    for v in 0..mesh.vert_count() {
        for &w in adj.vert_verts.neighbours(v) {
            let (a, b) = (of[v], of[w as usize]);
            if a != b {
                near[a as usize].insert(b);
                near[b as usize].insert(a);
            }
        }
    }

    // A escala de cada célula — a do primeiro vértice que caiu nela.
    let mut cell_scale = vec![0.0f32; k];
    let mut seen = vec![false; k];
    for (v, &c) in of.iter().enumerate() {
        if !seen[c as usize] {
            cell_scale[c as usize] = scale.at(v);
            seen[c as usize] = true;
        }
    }

    // ⚠️ **Primeiro cada célula ESCOLHE, e só depois as escolhas se confrontam.**
    // A versão anterior inseria a aresta nos dois lados assim que UM deles a
    // queria — e a valência estourava: medido na esfera, **390 células com 5 ou
    // mais vizinhas** (até 11), sobre uma grade cujo nó tem quatro. Uma célula
    // com 8 vizinhas não delimita quads: o passeio de faces sai em triângulos, e
    // era daí que vinham os **582** deles.
    let mut choice: Vec<Vec<u32>> = vec![Vec::new(); k];
    for c in 0..k {
        let (o, n, q) = (verts[c], normals[c], dirs[c]);
        let t = cross(n, q);
        let s = cell_scale[c].max(crate::scale::MIN_EDGE);
        let axes = [q, t, [-q[0], -q[1], -q[2]], [-t[0], -t[1], -t[2]]];
        for axis in axes {
            let mut best: Option<(f32, u32)> = None;
            for &cand in &near[c] {
                let d = sub(verts[cand as usize], o);
                let len = dot(d, d).sqrt();
                if len < 0.5 * s || len > 1.7 * s {
                    continue;
                }
                let cosang = dot(d, axis) / len;
                if cosang < core::f32::consts::FRAC_1_SQRT_2 {
                    continue;
                }
                // A melhor é a mais ALINHADA, e o desempate é o índice — nunca a
                // ordem de visita.
                let score = cosang;
                if best.is_none_or(|(b, bi)| score > b || (score == b && cand < bi)) {
                    best = Some((score, cand));
                }
            }
            if let Some((_, w)) = best {
                choice[c].push(w);
            }
        }
    }

    // ⚠️ **A aresta vale se UM dos lados a escolheu — a MUTUALIDADE foi MEDIDA e
    // REJEITADA.** Exigir que as duas pontas se escolhessem limita a valência a
    // quatro por construção, o que parecia a cura do histograma (390 células com
    // 5+ vizinhas). Medido: ela **remove** arestas de mais, o passeio de faces
    // passa a atravessar os buracos, e os ciclos chegam a **31 lados** — a malha
    // sai com **918** triângulos contra 582, e a fração honesta de quads cai de
    // **53,3 % para 35,0 %**.
    //
    // ⚠️ E ela quase passou por melhoria: a régua de então contava CICLOS, e um
    // ciclo de 31 lados contava como **um** não-quad enquanto virava 29
    // triângulos — a métrica subia de 60,9 % para 71,9 % enquanto a malha
    // piorava. *Foi a régua que se corrigiu primeiro, não o algoritmo.*
    let mut g = vec![BTreeSet::new(); k];
    for c in 0..k {
        for &w in &choice[c] {
            g[c].insert(w);
            g[w as usize].insert(c as u32);
        }
    }
    g
}

/// **AS FACES, pelo SISTEMA DE ROTAÇÃO.**
///
/// Em cada célula os vizinhos são ordenados por ângulo no plano tangente. Uma
/// face é o passeio que, ao chegar a `v` vindo de `u`, sai pelo vizinho
/// **seguinte** de `u` na ordem angular de `v` — sempre virando para o mesmo
/// lado. Numa superfície orientável fechada isso parte as `2E` arestas dirigidas
/// exatamente nas `F` faces.
///
/// ⚠️ **É por isso que a normal da célula é load-bearing**: ela dá o *sentido*
/// da ordem angular. Uma normal virada numa célula inverte o passeio ali, e a
/// face que passa por ela sai com o comprimento errado — não com o sinal
/// errado, o que a torna difícil de reconhecer.
fn trace_faces(
    graph: &[BTreeSet<u32>],
    verts: &[[f32; 3]],
    normals: &[[f32; 3]],
    _orient: &OrientationField,
) -> Vec<Vec<u32>> {
    // A ordem angular de cada célula, e o índice inverso.
    let mut order: Vec<Vec<u32>> = Vec::with_capacity(graph.len());
    let mut index: Vec<BTreeMap<u32, usize>> = Vec::with_capacity(graph.len());
    for (c, ns) in graph.iter().enumerate() {
        let n = normals[c];
        let e1 = tangent_of(n);
        let e2 = cross(n, e1);
        let mut ns: Vec<u32> = ns.iter().copied().collect();
        ns.sort_by(|a, b| {
            let ang = |x: u32| {
                let d = sub(verts[x as usize], verts[c]);
                dot(d, e2).atan2(dot(d, e1))
            };
            // ⚠️ `total_cmp` e não `partial_cmp`: um `NaN` numa coordenada
            // tornaria a ordenação não-determinística (ou um pânico), e a saída
            // desta função tem de ser byte-reprodutível.
            ang(*a).total_cmp(&ang(*b))
        });
        let mut m = BTreeMap::new();
        for (i, &w) in ns.iter().enumerate() {
            m.insert(w, i);
        }
        order.push(ns);
        index.push(m);
    }

    let mut seen: BTreeSet<(u32, u32)> = BTreeSet::new();
    let mut faces = Vec::new();
    for u in 0..graph.len() as u32 {
        for &v in &order[u as usize] {
            if seen.contains(&(u, v)) {
                continue;
            }
            let mut cycle = Vec::new();
            let (mut a, mut b) = (u, v);
            loop {
                if !seen.insert((a, b)) {
                    break;
                }
                cycle.push(a);
                let deg = order[b as usize].len();
                let Some(&i) = index[b as usize].get(&a) else {
                    break;
                };
                let next = order[b as usize][(i + 1) % deg];
                (a, b) = (b, next);
                if (a, b) == (u, v) {
                    break;
                }
                // Uma face maior que o grafo é um passeio que não fecha — só
                // acontece com o sistema de rotação inconsistente, e parar aqui
                // impede o laço infinito de esconder isso.
                if cycle.len() > graph.len() {
                    break;
                }
            }
            if cycle.len() >= 3 {
                faces.push(cycle);
            }
        }
    }
    faces
}

fn tangent_of(n: [f32; 3]) -> [f32; 3] {
    let sign = 1.0f32.copysign(n[2]);
    let a = -1.0 / (sign + n[2]);
    let b = n[0] * n[1] * a;
    normalize_or(
        [sign.mul_add(n[0] * n[0] * a, 1.0), sign * b, -sign * n[0]],
        [1.0, 0.0, 0.0],
    )
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1].mul_add(b[2], -(a[2] * b[1])),
        a[2].mul_add(b[0], -(a[0] * b[2])),
        a[0].mul_add(b[1], -(a[1] * b[0])),
    ]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = sub(a, b);
    dot(d, d).sqrt()
}

fn normalize_or(a: [f32; 3], fallback: [f32; 3]) -> [f32; 3] {
    let len = dot(a, a).sqrt();
    if len > 1.0e-6 {
        [a[0] / len, a[1] / len, a[2] / len]
    } else {
        fallback
    }
}

#[cfg(test)]
#[path = "extract_tests.rs"]
mod tests;
