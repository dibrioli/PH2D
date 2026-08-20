//! **A EXTRAÇÃO** — os dois campos viram uma malha (ADR-0160 §5, Q3).
//!
//! O campo de posição coloca uma origem de retícula por vértice, consistente **a
//! menos de passos inteiros** — e é **aqui** que se quocienta pela retícula. Três
//! passos, e cada um responde a uma pergunta da malha nova:
//!
//! 1. **as CÉLULAS** — que vértices da entrada descrevem o mesmo nó da grade?
//!    (união sobre as arestas; um `union-find`);
//! 2. **as ARESTAS** — que células são vizinhas? (as arestas da entrada cujo
//!    passo de retícula tem norma **1** — a mesma conta do passo 1, lida um
//!    degrau adiante; ver `Linking`);
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
//! Medido em 2026-08-19: **97,6 %** na malha que o módulo abre, e **todos** os
//! triângulos que sobram são **isolados** (sonda
//! `measure_where_the_leftover_triangles_come_from`) — o emparelhamento guloso
//! está esgotado, e o que falta é mesmo o passo global.
//!
//! ⚠️ **Mas o resíduo deixou de ser uma AGULHA.** Um ciclo de `n > 4` não é mais
//! triangulado em leque a partir do vértice `0` (`n − 2` fatias degeneradas a
//! irradiar de um ponto — a peça espetada da foto do Enio, 2026-08-19): ele é
//! fechado por um nó no centroide, em `⌈n/2⌉` faces quase todas quads. Ver
//! `faces::fan_free_closure`.

use std::collections::BTreeMap;

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
    // ⚠️ **`Lattice` — o quociente da referência.** Ele SÓ funciona sobre campos
    // com platôs, e é por isso que a porta do produto é a
    // [`crate::solve::solve_fields`] (hierárquica) e não o caminho plano.
    extract_with(mesh, orient, pos, scale, Clustering::Lattice)
}

/// **Como os vértices viram células** — as duas leis, medidas uma contra a outra.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clustering {
    /// Bola no campo em torno de uma semente. Não precisa de platôs.
    Seed,
    /// **O QUOCIENTE PELA RETÍCULA** — dois vizinhos são a mesma célula quando o
    /// passo inteiro entre as retículas deles é `(0,0)`.
    ///
    /// ⭐ **É O CAMINHO DO PRODUTO** — e ele esteve MEDIDO E REJEITADO por meio
    /// dia, o que é a lição desta wave.
    ///
    /// Ele colapsava (esfera → **1 célula**) porque o
    /// `compat_position_extrinsic_4` da crate **não era o da referência**:
    /// arredondava cada lado ao ponto médio, independentemente, em vez de
    /// enumerar as quatro quinas de cada lado e escolher o **PAR** mais próximo.
    /// Sem esse operador as duas retículas nunca se procuram, o campo sai suave,
    /// não há degraus — e o quociente funde tudo.
    ///
    /// Com o porte fiel: **76,9 %** de quads na esfera e **86,6 %** no toro,
    /// contra 71,7 %/75,6 % do crescimento por semente.
    ///
    /// ⚠️ **A lição não é sobre este enum:** duas recusas MEDIDAS (esta e a da
    /// hierarquia) eram consequências de **um** operador mal portado. *Uma
    /// medição só refuta o que ela de facto exercitou* — e o que ela exercitava
    /// era a minha aproximação, não a lei.
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
    extract_tuned(mesh, orient, pos, scale, how, Linking::default())
}

/// **COMO AS CÉLULAS SE LIGAM** — as duas leis, medidas uma contra a outra.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Linking {
    /// **O PASSO INTEIRO DA RETÍCULA** — a lei da referência, e o default.
    ///
    /// Duas células são vizinhas quando alguma aresta da entrada as separa por
    /// **um** passo da retícula (`|a| + |b| = 1`). É o mesmo arredondamento que
    /// já decide o agrupamento (passo `(0,0)`), lido um degrau adiante — então a
    /// vizinhança e a fusão passam a ser **a mesma pergunta**, e não duas que
    /// podem discordar.
    #[default]
    LatticeStep,
    /// **O CONE GEOMÉTRICO** — a primeira lei desta crate, mantida como controle.
    ///
    /// Cada célula escolhe a candidata mais alinhada em cada uma das quatro
    /// direções da cruz, dentro de uma janela de distância. Ela **adivinha** a
    /// grade a partir da geometria em vez de a ler do campo, e os dois limiares
    /// (o cone de 45°, a janela `[0,5 s, 1,7 s]`) são a superfície onde a adivinha
    /// erra.
    Cone,
}

/// A extração com as DUAS leis abertas — a porta da sonda que as mediu.
///
/// # Erros
/// Ver [`extract`].
pub fn extract_tuned(
    mesh: &Mesh,
    orient: &OrientationField,
    pos: &PositionField,
    scale: &ScaleField,
    how: Clustering,
    link: Linking,
) -> Result<Quadrangulation, MeshError> {
    let cells = match how {
        Clustering::Seed => cluster(mesh, pos, scale),
        Clustering::Lattice => cluster_lattice(mesh, orient, pos, scale),
    };
    let c = collapse(mesh, pos, orient, &cells);
    let mut graph = match link {
        Linking::LatticeStep => lattice_graph(mesh, orient, pos, scale, &c),
        Linking::Cone => neighbour_graph(mesh, &c, scale),
    };
    prune_dangling(&mut graph);
    let cycles: Vec<Vec<u32>> = trace_faces(&graph, &c.verts, &c.normals, orient)
        .into_iter()
        .flat_map(|cy| split_pinches(&cy))
        .collect();

    // ⚠️ **COMPACTAR ANTES de construir as faces: só as células que aparecem num
    // ciclo entram na malha.** Um vértice órfão conta em `V` e não em `E` nem em
    // `F` — ele empurra a característica de Euler para cima sem descrever nada, e
    // faria a A3 medir a PODA em vez da topologia.
    let mut remap = vec![u32::MAX; c.verts.len()];
    let mut verts: Vec<[f32; 3]> = Vec::new();
    let cycles: Vec<Vec<u32>> = cycles
        .into_iter()
        .map(|cy| {
            cy.into_iter()
                .map(|v| {
                    let old = v as usize;
                    if remap[old] == u32::MAX {
                        remap[old] = verts.len() as u32;
                        verts.push(c.verts[old]);
                    }
                    remap[old]
                })
                .collect()
        })
        .collect();

    // ⚠️ **A contagem NÃO é acumulada aqui**, e o clippy o disse: ela sai da
    // lista FINAL, depois do emparelhamento (ver abaixo). Duas contagens da mesma
    // coisa divergem; uma derivada da fonte, não.
    let (mut faces, mut max_sides) = (Vec::<Face>::new(), 0usize);
    for c in &cycles {
        max_sides = max_sides.max(c.len());
        match c.len() {
            4 => faces.push(Face::quad(c[0], c[1], c[2], c[3])),
            3 => faces.push(Face::tri(c[0], c[1], c[2])),
            n if n > 4 => fan_free_closure(c, &mut verts, &mut faces),
            // Um ciclo de 2 ou menos não delimita área: ele é um artefato da
            // fusão, e entrar na malha seria uma face degenerada.
            _ => {}
        }
    }

    // ⚠️ **O EMPARELHAMENTO: dois triângulos vizinhos SÃO um quad com uma
    // diagonal a mais.** Os não-quads que sobram não são singularidades do campo
    // — as singularidades de uma grade vivem na VALÊNCIA dos vértices (3 ou 5),
    // nunca no número de lados de uma face. Eles são resíduo da extração, e um
    // par que partilha aresta reconstitui o quad que a diagonal partiu.
    //
    // ⚠️ **A operação preserva χ por construção**: some uma aresta e some uma
    // face, e `V − (E−1) + (F−1)` é `V − E + F`.
    let (paired, _merged) = pair_triangles(&faces, &verts);
    faces = paired;

    // ⚠️ **A CONTAGEM SAI DA LISTA FINAL, e não de aritmética sobre os ciclos.**
    // A primeira versão fazia `quads += pares; non_quads -= pares * 2` — e
    // `non_quads` contava CICLOS enquanto os pares consomem TRIÂNGULOS. Um ciclo
    // de `n` lados vira `n − 2` triângulos, então as duas grandezas nunca foram
    // a mesma, e o `usize` deu a volta: **18 446 744 073 709 551 613**
    // não-quads num gate. *Duas contagens da mesma coisa divergem no dia em que
    // uma delas ganha um consumidor novo; uma contagem derivada da fonte, não.*
    let quads = faces.iter().filter(|f| f.verts().len() == 4).count();
    let non_quads = faces.len() - quads;

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

    // ⚠️ **Ordem de índice — e a amostragem de POISSON foi MEDIDA e REJEITADA.**
    // Semear pelo ponto mais DISTANTE das sementes já postas espalha-as por
    // construção e dá células de tamanhos parecidos; parecia a cura da valência
    // irregular. Medido: **48,9 %** de quads na esfera contra 53,3 %, e 38,5 %
    // contra 39,7 % no toro. Células mais regulares não são grades melhores —
    // o que a grade quer é que as vizinhas caiam nas QUATRO direções da cruz, e a
    // regularidade métrica não fala sobre isso.
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
            // **O campo de `w`, trazido à retícula de `v`, ANDA?** Se o passo
            // inteiro é `(0,0)`, os dois descrevem o mesmo nó da grade.
            //
            // ⚠️ **É o critério da referência expresso num referencial só.** O
            // `extract_graph` compara os dois índices inteiros
            // (`shift.first − shift.second == 0`), e o porte literal disso vive
            // aqui ao lado (`compat_position_extrinsic_index_4`) — MEDIDO, ele dá
            // **19,3 %** de quads contra os **76,9 %** desta forma, com χ = 280 e
            // ciclos de 63 lados. Os índices de cada lado vivem na retícula do
            // SEU vértice, e igualá-los exige que os dois campos já partilhem o
            // referencial — o que a nossa otimização, sem o limiar de `error` e
            // sem os passes de limpeza do `extract_graph`, ainda não garante.
            // *A função fica portada e gateada; ligá-la é trabalho da Q4.*
            //
            // ⚠️ E o teste **não** pode ser a distância entre as quinas que o
            // `compat_position_extrinsic_4` devolve: ele escolhe o par mais
            // PRÓXIMO por construção, então aquela distância é sempre minúscula e
            // o `union-find` funde o modelo inteiro (medido: **1 célula**).
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

pub(super) fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

pub(super) fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1].mul_add(b[2], -(a[2] * b[1])),
        a[2].mul_add(b[0], -(a[0] * b[2])),
        a[0].mul_add(b[1], -(a[1] * b[0])),
    ]
}

pub(super) fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
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

/// **DOS CICLOS ÀS FACES** — ver [`faces`].
#[path = "extract_faces.rs"]
mod faces;
use faces::{
    fan_free_closure, pair_triangles, prune_dangling, split_pinches, tangent_of, trace_faces,
};

/// **QUE CÉLULAS SÃO VIZINHAS** — ver [`graph`].
#[path = "extract_graph.rs"]
mod graph;
use graph::{lattice_graph, neighbour_graph};

#[cfg(test)]
#[path = "extract_tests.rs"]
mod tests;
