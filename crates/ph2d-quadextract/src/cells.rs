//! ⭐⭐⭐ **FASE 4b — FECHAR as células, FUNDIR os nós repetidos, e montar a malha.**
//!
//! Partindo de qualquer ligação não visitada: siga-a até ao nó seguinte, **vire à
//! esquerda** — que é *a saída seguinte na lista horária daquele nó* — e repita até
//! voltar ao ponto de partida.
//!
//! ⚠️⚠️ **A transição acumulada tem DOIS ingredientes por passo:** a da **ligação**
//! percorrida **e** a do **leque**, ao rodar do talo de chegada para o talo de saída
//! dentro do mesmo nó. ⛔ Esquecer a segunda é o erro que dá células com coordenadas
//! locais impossíveis, e a fusão deixa de convergir.
//!
//! # ⭐⭐ Por que as coordenadas locais são a chave
//!
//! Com traço correcto, os nós de uma célula só podem cair num de **quatro** valores
//! locais — os cantos do quadrado unitário. Um número **par** de mudanças de
//! orientação leva ao valor esperado; **ímpar** devolve ao valor de partida. ⇒
//! *coordenadas locais repetidas são a assinatura, medível, de uma dobra*, e é assim
//! que se limpa sem tocar nos vizinhos.
//!
//! ⭐⭐⭐ **O resultado é um teorema, não uma esperança:** como dentro de uma célula
//! só há no máximo quatro valores locais distintos, depois da fusão só podem existir
//! **quads, bígonos e monógonos** — e estes dois últimos colapsam trivialmente.
//! ⛔ **Triângulos não podem ocorrer:** exigiriam uma ligação **diagonal** no
//! quadrado unitário, e a fusão não cria ligações novas.

use ph2d_mesh::{Face, Mesh, MeshError};

use crate::exact::{P, Xf};
use crate::ingest::Topo;
use crate::nodes::Node;
use crate::ports::Ports;

/// O tecto de lados que um percurso pode ter antes de ser dado por não-fechado.
///
/// ⭐ **Quatro é o número da grade**, e o tecto é MUITO maior de propósito: perto de
/// uma dobra as cartas **sobrepõem-se**, um nó recebe saídas a mais, e a órbita
/// fecha com mais lados. ⛔ **Cortar em `8` não a torna num quad — apaga-a**, e com
/// ela as saídas que ela já tinha consumido; foi assim que seis órbitas do gancho
/// viraram oito arestas de bordo e `χ = 5`. *Meça a distribuição antes de escolher o
/// tecto* — ver [`CellStats::ring_len`].
const MAX_SIDES: usize = 64;

/// O que a extracção de células mediu de si própria.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct CellStats {
    /// ⭐⭐⭐ **ONDE os percursos falharam**, em **raios normalizados** — a mediana da
    /// distância ao centro dos nós que abandonaram ou não fecharam, dividida pela mediana
    /// de todos. `1,0` = no corpo · `1,3` = na ponta.
    ///
    /// ⛔⛔ **Ela existe porque o report do artista é sobre POSIÇÃO** (*«furos nas
    /// pontas»*, 2026-08-25), e toda régua desta crate era um TOTAL: `10 abandonadas` não
    /// diz se elas estão nas pontas ou espalhadas pelo corpo, e são coisas diferentes com
    /// curas diferentes. ⚠️ *Um número que não tem coordenada não pode confirmar nem
    /// desmentir uma frase que tem.*
    ///
    /// ⚠️ Sai **resumida e não como lista** porque a [`crate::ExtractReport`] é `Copy` e
    /// tem dezenas de sítios de construção — *a forma do relatório é um contrato, e alargá-lo
    /// para um diagnóstico é pagar em toda a workspace o que uma mediana responde.*
    pub failed_radius_p50: f32,
    /// O `p99` do raio normalizado de **todos** os nós — a régua com que a linha de cima
    /// se lê. ⛔ Sem ela, `1,3` não diz se é ponta ou se a peça inteira vai a `3,0`.
    pub node_radius_p99: f32,
    /// Células fechadas.
    pub closed: usize,
    /// ⚠️ Percursos abandonados por encontrarem uma saída **pendente**.
    pub abandoned: usize,
    /// ⛔ Percursos que passaram o tecto de lados sem fechar.
    pub unclosed: usize,
    /// Grupos de nós que a fusão colapsou (componentes com mais de um nó).
    pub merged_groups: usize,
    /// Quads emitidos.
    pub quads: usize,
    /// ⭐⭐⭐ **CÉLULAS ESPELHADAS — o mesmo anel de nós percorrido nos DOIS sentidos.**
    ///
    /// ⛔⛔ **Ela existe por uma foto (Enio, 2026-08-28, «faces completamente soltas»).** A
    /// peça dele saiu com o casco **fechado e perfeito** (`χ = 2`, zero bordo, zero
    /// não-manifold, `23 628` faces) **mais** uma ilha de **duas** faces a flutuar numa
    /// ponta: `[68,69,70,71]` e `[71,70,69,68]` — *o mesmo quadrado emitido duas vezes, um
    /// virado ao contrário do outro*, com arestas `3×` a mediana da peça.
    ///
    /// ⚠️ **Nenhuma régua desta linha a via**, e todas passavam: `χ` conta os dois lados de
    /// uma almofada e dá `2`, o bordo é zero, o não-manifold é zero, e a contagem de quads
    /// sobe. *Uma almofada é uma superfície fechada legítima — o que ela não é, é parte
    /// desta.*
    ///
    /// ⭐ **A causa é uma DOBRA do mapa:** uma região coberta duas vezes com orientações
    /// opostas dá dois percursos de célula sobre os mesmos nós. ⇒ nenhum dos dois é uma
    /// face da superfície, e os dois caem.
    pub mirrored_cells: usize,
    /// ⭐⭐⭐ **DOUBLETS dissolvidos** — um vértice interior com **duas** arestas e duas faces.
    ///
    /// ⛔⛔ **Ele é a mordida que o artista fotografou em 2026-08-29** («o remesh amputou
    /// pontas»), e o pior é que ele se **REALIMENTA**: a saída que ele exportou tinha `19`
    /// deles, todos em pontas finas — e ao voltar a entrar na cadeia, a fase zero, que só
    /// sabe remalhar superfície, transforma `χ = 2` em **`χ = 6` com aresta não-manifold**, e
    /// a jusante o `ph2d-gridmap` entra em `index out of bounds`.
    ///
    /// ⭐ **A dissolução é a clássica e é exacta:** as duas faces partilham três cantos, então
    /// fundem-se numa só — `V−1`, `E−2`, `F−1`, **`χ` invariante**. *Não há escolha a fazer
    /// nem geometria a inventar.*
    pub doublets: usize,
    /// Bígonos e monógonos, que colapsam.
    pub degenerate_cells: usize,
    /// ⛔ Células que sobraram com três cantos distintos — o teorema diz que não
    /// existem, e por isso elas são contadas em vez de emitidas.
    pub triangles: usize,
    /// ⚠️ **§6.4** — nós em que duas saídas consecutivas apontam na mesma direcção:
    /// o leque colapsou abaixo de meia volta e o nó ficou com saídas a menos.
    pub collapsed_fans: usize,
    /// ⭐⭐⭐ **O HISTOGRAMA DA ORDEM DAS SAÍDAS** — quantos passos consecutivos
    /// avançam `0`, `1`, `2` ou `3` quartos de volta.
    ///
    /// ⚠️ **A lista é horária, logo TODO passo são desce um quarto**: um mapa são
    /// põe **tudo** no balde `3`. É a prova executável da propriedade de que a
    /// extracção de células depende, e ela é cega a qualquer outra coisa.
    pub port_step: [usize; 4],
    /// O mesmo histograma para os pares em que **alguma** das duas saídas vive numa
    /// carta dobrada. ⚠️ Aí a resposta certa é o balde `1`, não o `3`.
    pub port_step_folded: [usize; 4],
    /// ⭐ **QUANTOS LADOS cada percurso fechado teve**, indexado pelo número de
    /// lados (`0..=16`, com o último balde a somar tudo o que passa). Uma grade sã
    /// põe tudo no balde `4`.
    pub ring_len: [usize; 17],
    /// ⭐ **QUANTOS CANTOS DISTINTOS cada célula tem depois da fusão** — o teorema
    /// diz `4`, `2` ou `1`. Um `3` seria uma ligação diagonal, que a fusão não cria.
    pub ring_distinct: [usize; 17],
}

struct Uf(Vec<u32>);

impl Uf {
    fn new(n: usize) -> Self {
        Self((0..u32::try_from(n).unwrap_or(u32::MAX)).collect())
    }
    fn find(&mut self, mut a: u32) -> u32 {
        while self.0[a as usize] != a {
            let g = self.0[self.0[a as usize] as usize];
            self.0[a as usize] = g;
            a = g;
        }
        a
    }
    fn union(&mut self, a: u32, b: u32) -> bool {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra == rb {
            return false;
        }
        let (lo, hi) = if ra < rb { (ra, rb) } else { (rb, ra) };
        self.0[hi as usize] = lo;
        true
    }
}

/// ⭐ **A EXTRACÇÃO DE CÉLULAS + A FUSÃO + A MALHA.**
pub(crate) fn build(
    topo: &crate::ingest::Topo,
    nodes: &[Node],
    ports: &Ports,
) -> Result<(Mesh, CellStats), MeshError> {
    let mut st = CellStats::default();
    let (collapsed, clean, folded) = probe_fans(topo, ports);
    st.collapsed_fans = collapsed;
    st.port_step = clean;
    st.port_step_folded = folded;

    // ── §6.2 Fechar as células, guardando as coordenadas locais.
    let mut failed: Vec<usize> = Vec::new();
    let mut visited = vec![false; ports.ports.len()];
    let mut cells: Vec<Vec<u32>> = Vec::new();
    let mut uf = Uf::new(nodes.len());
    for start in 0..ports.ports.len() {
        if visited[start] || ports.ports[start].link.is_none() {
            continue;
        }
        #[allow(clippy::cast_possible_truncation)]
        match walk_cell(ports, start as u32, &mut visited) {
            Trip::Closed(ring) => {
                st.closed += 1;
                st.ring_len[ring.len().min(16)] += 1;
                merge_by_local(&ring, &mut uf);
                cells.push(ring.iter().map(|&(n, _)| n).collect());
            }
            Trip::Abandoned => {
                st.abandoned += 1;
                failed.push(ports.ports[start].node as usize);
            }
            Trip::Unclosed => {
                st.unclosed += 1;
                failed.push(ports.ports[start].node as usize);
            }
        }
    }

    // ⭐ **ONDE as falhas moram** — ver [`CellStats::failed_radius_p50`].
    {
        let c = nodes.iter().fold([0.0f64; 3], |a, n| {
            [a[0] + n.pos[0], a[1] + n.pos[1], a[2] + n.pos[2]]
        });
        let inv = 1.0 / nodes.len().max(1) as f64;
        let c = [c[0] * inv, c[1] * inv, c[2] * inv];
        let r = |i: usize| {
            let p = nodes[i].pos;
            let d = [p[0] - c[0], p[1] - c[1], p[2] - c[2]];
            d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
        };
        let mut all: Vec<f64> = (0..nodes.len()).map(r).collect();
        all.sort_by(f64::total_cmp);
        let med = all.get(all.len() / 2).copied().unwrap_or(1.0).max(1.0e-12);
        #[allow(clippy::cast_possible_truncation)]
        {
            st.node_radius_p99 =
                (all.get(all.len() * 99 / 100).copied().unwrap_or(med) / med) as f32;
            let mut f: Vec<f64> = failed.iter().map(|&i| r(i) / med).collect();
            f.sort_by(f64::total_cmp);
            st.failed_radius_p50 = f.get(f.len() / 2).copied().unwrap_or(0.0) as f32;
        }
    }

    // ── §6.3 Cada componente conexo colapsa num nó único, no CENTROIDE.
    let mut acc: Vec<([f64; 3], usize)> = vec![([0.0; 3], 0); nodes.len()];
    for (i, n) in nodes.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        let r = uf.find(i as u32) as usize;
        for k in 0..3 {
            acc[r].0[k] += n.pos[k];
        }
        acc[r].1 += 1;
        if r != i {
            // contado uma vez por membro extra do grupo
        }
    }
    st.merged_groups = acc.iter().filter(|(_, c)| *c > 1).count();

    // ⭐⭐⭐ **AS CÉLULAS ESPELHADAS CAEM ANTES DE VIRAREM FACE** — ver
    // [`CellStats::mirrored_cells`]. A chave é o ciclo **sem sentido**: roda-se o anel para
    // começar no menor nó, compara-se com o mesmo tratamento do anel invertido, e fica o
    // menor dos dois. ⚠️ Duas células com a mesma chave são a mesma região do mapa contada
    // duas vezes — *e os DOIS lados caem, porque uma almofada não tem lado certo.*
    let rings: Vec<Vec<u32>> = cells.iter().map(|c| reduce_ring(c, &mut uf)).collect();
    let mut seen: std::collections::BTreeMap<Vec<u32>, usize> = std::collections::BTreeMap::new();
    for r in &rings {
        *seen.entry(undirected_key(r)).or_default() += 1;
    }
    // ⚠️ **`PH2D_EXTRACT_MIRROR=0` devolve a almofada** — é o A/B, e é o que permite dizer
    // que a cura disparou nesta peça em vez de o supor. Lida **uma vez**, fora do laço.
    let drop_mirrored = std::env::var("PH2D_EXTRACT_MIRROR").as_deref() != Ok("0");

    // ── A malha: só os nós que uma célula sobrevivente usa.
    let mut slot = vec![u32::MAX; nodes.len()];
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut faces: Vec<Face> = Vec::new();
    for ring in &rings {
        let ring = ring.clone();
        if seen.get(&undirected_key(&ring)).copied().unwrap_or(0) > 1 {
            st.mirrored_cells += 1;
            if drop_mirrored {
                continue;
            }
        }
        st.ring_distinct[ring.len().min(16)] += 1;
        match ring.len() {
            4 => {}
            3 => {
                st.triangles += 1;
                continue;
            }
            _ => {
                st.degenerate_cells += 1;
                continue;
            }
        }
        let mut idx = [0u32; 4];
        for (i, &r) in ring.iter().enumerate() {
            let s = &mut slot[r as usize];
            if *s == u32::MAX {
                let (sum, n) = acc[r as usize];
                #[allow(clippy::cast_precision_loss)]
                let inv = 1.0 / n.max(1) as f64;
                #[allow(clippy::cast_possible_truncation)]
                positions.push([
                    (sum[0] * inv) as f32,
                    (sum[1] * inv) as f32,
                    (sum[2] * inv) as f32,
                ]);
                #[allow(clippy::cast_possible_truncation)]
                let id = (positions.len() - 1) as u32;
                *s = id;
            }
            idx[i] = *s;
        }
        faces.push(Face::quad(idx[0], idx[1], idx[2], idx[3]));
        st.quads += 1;
    }

    st.doublets = dissolve_doublets(&mut faces);
    compact_verts(&mut positions, &mut faces);
    // ⚠️ **A CONTAGEM tem de seguir a dissolução.** Cada doublet funde duas faces numa, e o
    // `quads` é lido a jusante como `faces - quads = nao-quads`: sem esta linha a subtracção
    // dá `usize` negativo e o log imprime `18446744073709551613`. *Um contador que descreve
    // uma fase anterior à última é um contador errado.*
    st.quads = faces.len();
    Ok((Mesh::from_parts(positions, faces)?, st))
}

/// ⭐⭐⭐ **REPARA UMA MALHA JÁ CONSTRUÍDA** — a mesma lei, para quem chega com a peça pronta.
///
/// ⛔⛔ **A mordida REALIMENTA-SE, e é por isso que esta porta existe.** A saída que o artista
/// exportou em 2026-08-29 tinha `19` doublets, todos em pontas finas; ao voltar a entrar na
/// cadeia, a fase zero — que só sabe remalhar superfície — transforma `χ = 2` em **`χ = 6`
/// com aresta não-manifold**, e a jusante o `ph2d-gridmap` estoura. ⇒ *fechar só o lado da
/// saída deixa toda peça já gravada a partir o botão para sempre.*
///
/// # Errors
/// Nunca — a fusão é exacta e a malha resultante é sempre construível.
pub fn repair_doublets(mesh: &mut Mesh) -> Result<usize, MeshError> {
    let mut faces = mesh.faces().to_vec();
    let n = dissolve_doublets(&mut faces);
    if n > 0 {
        let mut positions = mesh.positions().to_vec();
        compact_verts(&mut positions, &mut faces);
        *mesh = Mesh::from_parts(positions, faces)?;
    }
    Ok(n)
}

/// ⭐⭐⭐ **DEIXA CAIR OS VÉRTICES ÓRFÃOS** — e ela não é arrumação.
///
/// ⛔⛔ **Sem ela a dissolução PARECE preservar a topologia e não preserva a CONTAGEM.** O
/// vértice preso deixa de ser usado por face nenhuma, mas fica no arquivo — e a
/// característica de Euler é `V − E + F` sobre **todos** os vértices, então ela sobe **`1` por
/// mordida**. Medido: dois gates desta crate reprovaram com `χ = 14` contra `2` e `13` contra
/// `1` — *doze órfãos, doze unidades.*
///
/// ⚠️ *«A superfície está certa» e «o ficheiro está certo» são duas afirmações, e a régua
/// mede a segunda.*
fn compact_verts(positions: &mut Vec<[f32; 3]>, faces: &mut [Face]) {
    let mut used = vec![false; positions.len()];
    for f in faces.iter() {
        for &v in f.verts() {
            if let Some(u) = used.get_mut(v as usize) {
                *u = true;
            }
        }
    }
    if used.iter().all(|u| *u) {
        return;
    }
    let mut slot = vec![u32::MAX; positions.len()];
    let mut next = 0u32;
    for (i, u) in used.iter().enumerate() {
        if *u {
            slot[i] = next;
            next += 1;
        }
    }
    let mut kept: Vec<[f32; 3]> = Vec::with_capacity(next as usize);
    for (i, p) in positions.iter().enumerate() {
        if used[i] {
            kept.push(*p);
        }
    }
    *positions = kept;
    for f in faces.iter_mut() {
        let v = f.verts();
        let m: Vec<u32> = v.iter().map(|&x| slot[x as usize]).collect();
        *f = if v.len() == 4 && m.len() == 4 {
            Face::quad(m[0], m[1], m[2], m[3])
        } else {
            Face::tri(m[0], m[1], m[2])
        };
    }
}

/// ⭐⭐⭐ **DISSOLVE OS DOUBLETS** — ver [`CellStats::doublets`]. Devolve quantos caíram.
///
/// Um **doublet** é um vértice interior com exactamente **duas** arestas e duas faces; as
/// duas partilham três cantos (`a`, `v`, `b`) e a união delas é um quad. ⭐ Fundi-las é
/// exacto: `V−1`, `E−2`, `F−1`, e `χ` não se mexe.
///
/// ⚠️ **A ORDEM da fusão sai do percurso da fronteira, não de um palpite:** com
/// `Q1 = [a, v, b, p]` e `Q2 = [b, v, a, q]`, apagar as arestas `a–v` e `v–b` deixa
/// `a → q → b → p → a`. *Escrever `[a, p, b, q]` daria o quad com os lados trocados.*
///
/// ⚠️ **E ele corre até assentar:** dissolver um doublet pode criar outro no vizinho.
fn dissolve_doublets(faces: &mut Vec<Face>) -> usize {
    use std::collections::BTreeMap;
    let mut total = 0usize;
    for _ in 0..MAX_DOUBLET_ROUNDS {
        let mut inc: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
        let mut ring: BTreeMap<u32, std::collections::BTreeSet<u32>> = BTreeMap::new();
        for (fi, f) in faces.iter().enumerate() {
            let v = f.verts();
            for k in 0..v.len() {
                inc.entry(v[k]).or_default().push(fi);
                ring.entry(v[k]).or_default().insert(v[(k + 1) % v.len()]);
                ring.entry(v[k]).or_default().insert(v[(k + v.len() - 1) % v.len()]);
            }
        }
        let mut dead: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
        let mut round = 0usize;
        for (&v, fs) in &inc {
            if fs.len() != 2 || ring.get(&v).map_or(0, std::collections::BTreeSet::len) != 2 {
                continue;
            }
            let (f0, f1) = (fs[0], fs[1]);
            if dead.contains(&f0) || dead.contains(&f1) {
                continue;
            }
            let (q0, q1) = (faces[f0].verts().to_vec(), faces[f1].verts().to_vec());
            if q0.len() != 4 || q1.len() != 4 {
                continue;
            }
            let (Some(i), Some(j)) = (
                q0.iter().position(|&x| x == v),
                q1.iter().position(|&x| x == v),
            ) else {
                continue;
            };
            let (a, b, p) = (q0[(i + 3) % 4], q0[(i + 1) % 4], q0[(i + 2) % 4]);
            let (b2, a2, q) = (q1[(j + 3) % 4], q1[(j + 1) % 4], q1[(j + 2) % 4]);
            // ⚠️ **A segunda face tem de ver `a` e `b` ao contrário** — se ela os vê na mesma
            // ordem, as duas não são vizinhas por duas arestas e a fusão inventaria um quad.
            if (a2, b2) != (a, b) {
                continue;
            }
            // ⛔ `p == q` é uma ALMOFADA (duas faces coincidentes) e não um doublet — fundi-la
            // daria um quad com dois cantos iguais. Ver [`CellStats::mirrored_cells`].
            if p == q {
                continue;
            }
            faces[f0] = Face::quad(a, q, b, p);
            dead.insert(f1);
            round += 1;
        }
        if round == 0 {
            break;
        }
        let mut keep = 0usize;
        faces.retain(|_| {
            let live = !dead.contains(&keep);
            keep += 1;
            live
        });
        total += round;
    }
    total
}

/// A rede do laço de dissolução — ver [`dissolve_doublets`]. ⚠️ Medido: a peça do artista
/// resolve-se em **uma** ronda; este número é a rede de um caso patológico, não o que manda.
const MAX_DOUBLET_ROUNDS: usize = 8;

/// ⭐ **A CHAVE DE UM CICLO SEM SENTIDO** — roda para começar no menor nó e devolve o menor
/// entre o anel e o seu inverso.
///
/// ⚠️ *Sem a inversão a chave distinguiria as duas metades de uma almofada, que é
/// exactamente o par que se quer ver como um.*
fn undirected_key(ring: &[u32]) -> Vec<u32> {
    if ring.is_empty() {
        return Vec::new();
    }
    let rot = |v: &[u32]| -> Vec<u32> {
        let at = v
            .iter()
            .enumerate()
            .min_by_key(|(_, x)| **x)
            .map_or(0, |(i, _)| i);
        v[at..].iter().chain(&v[..at]).copied().collect()
    };
    let fwd = rot(ring);
    let mut back: Vec<u32> = ring.to_vec();
    back.reverse();
    let rev = rot(&back);
    if rev < fwd { rev } else { fwd }
}

enum Trip {
    Closed(Vec<(u32, P)>),
    Abandoned,
    Unclosed,
}

/// Um percurso de célula: **ligação, virar à esquerda, repetir**.
fn walk_cell(ports: &Ports, start: u32, visited: &mut [bool]) -> Trip {
    let mut ring: Vec<(u32, P)> = Vec::with_capacity(4);
    let mut cur = start;
    // `acc` leva a carta da saída CORRENTE à carta da saída de PARTIDA.
    let mut acc = Xf::IDENTITY;
    for _ in 0..MAX_SIDES {
        let p = ports.ports[cur as usize];
        ring.push((p.node, acc.apply(p.at)));
        visited[cur as usize] = true;
        let Some(q) = p.link else {
            return Trip::Abandoned;
        };
        // A ligação: da carta de `cur` para a de `q`.
        acc = p.link_xf.inverse().then(acc);
        let next = left_turn(ports, q);
        // ⚠️ **A transição do LEQUE**, ao rodar do talo de chegada para o de saída
        // dentro do mesmo nó. Sem ela as coordenadas locais são impossíveis.
        let fan = ports.ports[q as usize]
            .to_ref
            .then(ports.ports[next as usize].to_ref.inverse());
        acc = fan.inverse().then(acc);
        if next == start {
            return Trip::Closed(ring);
        }
        if visited[next as usize] {
            return Trip::Unclosed;
        }
        cur = next;
    }
    Trip::Unclosed
}

/// ⭐ **VIRAR À ESQUERDA** — a saída **seguinte** na lista horária daquele nó.
fn left_turn(ports: &Ports, arrived: u32) -> u32 {
    let node = ports.ports[arrived as usize].node as usize;
    let list = &ports.of_node[node];
    let i = list.iter().position(|&x| x == arrived).unwrap_or(0);
    list[(i + 1) % list.len()]
}

/// Liga dois nós sempre que **partilhem as mesmas coordenadas locais dentro da
/// mesma célula**.
/// ⭐ **A REDUÇÃO DE UM ANEL** — o que *«bígonos e monógonos colapsam trivialmente»*
/// quer dizer quando o anel tem mais de quatro lados.
///
/// ⚠️ **Duplicados consecutivos não bastam.** Um anel `A B C D C B` não tem nenhum
/// par consecutivo igual e mesmo assim é um quad `A B C D` com uma cauda dobrada
/// para trás — a **espora** `X Y X`, em que os dois vizinhos de `Y` são o mesmo nó.
/// Removê-las até estabilizar é o que reduz um percurso perto de uma dobra à célula
/// que ele de facto é. *Parar nos duplicados consecutivos deixava a célula com seis
/// cantos e ela era deitada fora inteira.*
fn reduce_ring(cell: &[u32], uf: &mut Uf) -> Vec<u32> {
    let mut ring: Vec<u32> = cell.iter().map(|&n| uf.find(n)).collect();
    loop {
        let before = ring.len();
        // duplicados consecutivos, incluindo o par que dá a volta
        let mut out: Vec<u32> = Vec::with_capacity(ring.len());
        for &r in &ring {
            if out.last() != Some(&r) {
                out.push(r);
            }
        }
        while out.len() > 1 && out.first() == out.last() {
            out.pop();
        }
        // esporas: os dois vizinhos de um canto são o mesmo nó
        ring = Vec::with_capacity(out.len());
        let n = out.len();
        let mut skip = vec![false; n];
        for i in 0..n {
            if n < 3 {
                break;
            }
            let (prev, next) = (out[(i + n - 1) % n], out[(i + 1) % n]);
            if prev == next && !skip[(i + n - 1) % n] && !skip[(i + 1) % n] {
                skip[i] = true;
            }
        }
        for (i, &r) in out.iter().enumerate() {
            if !skip[i] {
                ring.push(r);
            }
        }
        if ring.len() == before {
            return ring;
        }
    }
}

fn merge_by_local(ring: &[(u32, P)], uf: &mut Uf) {
    for i in 0..ring.len() {
        for j in (i + 1)..ring.len() {
            if ring[i].1 == ring[j].1 {
                uf.union(ring[i].0, ring[j].0);
            }
        }
    }
}

/// ⚠️ **§6.4 — o LEQUE COLAPSADO**, e ⭐ **a ordem das saídas medida ao mesmo
/// tempo**.
///
/// Um leque inteiro de triângulos em torno de um nó pode, com dobras, abranger
/// **menos de meia volta**: aí duas saídas consecutivas apontam na **mesma**
/// direcção sem que nenhuma esteja num triângulo dobrado, e o nó fica com saídas a
/// menos — no limite, uma só.
///
/// ⛔⛔ **A primeira redacção desta sonda acusava toda a singularidade**, e a leitura
/// (`5` em cada peça) parecia um achado. Ela transportava cada direcção para a carta
/// de **referência** do nó e comparava também o par que **dá a volta** — e é
/// precisamente aí que a holonomia entra: num nó de valência 5 as cinco direcções de
/// referência são `0,1,2,3,0`, e o último contra o primeiro leem iguais **por serem
/// uma singularidade**, não por o leque ter colapsado. *Uma sonda que acusa o caso
/// normal não mede nada.*
///
/// ⇒ o par que fecha a volta fica **de fora**, e o que sobra é uma medição forte: num
/// leque são, cada passo consecutivo desce **exactamente um** quarto de volta (a
/// lista é **horária**), e o histograma dos passos é a prova executável dessa ordem.
fn probe_fans(topo: &Topo, ports: &Ports) -> (usize, [usize; 4], [usize; 4]) {
    let mut collapsed = 0usize;
    let mut clean = [0usize; 4];
    let mut folded = [0usize; 4];
    for list in &ports.of_node {
        let mut hit = false;
        for w in list.windows(2) {
            let a = ports.ports[w[0] as usize];
            let b = ports.ports[w[1] as usize];
            let da = a.to_ref.dir(a.dir);
            let db = b.to_ref.dir(b.dir);
            let step = ((db + 4 - da) & 3) as usize;
            // ⚠️ **Uma carta DOBRADA inverte o sentido** — o horário na superfície é
            // o anti-horário no domínio, e o passo lê `+1` **por estar correcto**.
            // Misturar as duas populações num histograma só faria o caso normal de
            // uma dobra parecer um defeito, que foi a primeira leitura desta sonda.
            let unfolded = crate::walk::face_sign(topo, a.face as usize) > 0
                && crate::walk::face_sign(topo, b.face as usize) > 0;
            if unfolded {
                clean[step] += 1;
            } else {
                folded[step] += 1;
            }
            // ⚠️ **O §6.4 exige que NENHUMA das duas esteja numa carta dobrada** —
            // é essa a diferença entre um leque que colapsou abaixo de meia volta e
            // a sobreposição banal de duas cartas numa dobra, que a fusão trata.
            // *Contar as duas juntas dava `5` em cada peça e leria como um achado.*
            if step == 0 && unfolded {
                hit = true;
            }
        }
        if hit {
            collapsed += 1;
        }
    }
    (collapsed, clean, folded)
}

#[cfg(test)]
mod mirror_tests {
    use super::undirected_key;

    /// ⭐⭐⭐ **A CHAVE VÊ O CICLO E IGNORA O SENTIDO** — as duas metades de uma almofada
    /// têm de colidir, e dois quads diferentes não.
    ///
    /// ⛔ **A fixtura é o par real que o artista fotografou** (2026-08-28): a peça dele saiu
    /// com o casco fechado e perfeito **mais** `[68,69,70,71]` e `[71,70,69,68]` a flutuar
    /// numa ponta — *o mesmo quadrado emitido duas vezes, um virado ao contrário do outro*.
    #[test]
    fn the_key_sees_the_cycle_and_ignores_the_direction() {
        let a = undirected_key(&[68, 69, 70, 71]);
        let b = undirected_key(&[71, 70, 69, 68]);
        assert_eq!(a, b, "⛔ as duas metades de uma almofada tem de dar a MESMA chave");
        // ⚠️ E a rotação também não conta: o mesmo ciclo começado noutro canto é o mesmo.
        assert_eq!(a, undirected_key(&[70, 71, 68, 69]));
        assert_eq!(a, undirected_key(&[69, 68, 71, 70]));
        // ⛔ **O CONTROLE, e sem ele a chave podia ser uma constante:** dois quads que
        // partilham três nós e diferem no quarto são células DIFERENTES.
        assert_ne!(
            a,
            undirected_key(&[68, 69, 70, 99]),
            "⛔ a chave nao pode colidir para aneis diferentes"
        );
        // ⛔ E a ORDEM importa quando não é uma inversão: `[68,70,69,71]` percorre outro
        // ciclo sobre os mesmos quatro nós.
        assert_ne!(a, undirected_key(&[68, 70, 69, 71]));
        assert!(undirected_key(&[]).is_empty());
    }
}

#[cfg(test)]
mod doublet_tests {
    use ph2d_mesh::{Face, Mesh};

    /// ⭐⭐⭐ **A MORDIDA DISSOLVE-SE, e a ORDEM da fusão é a do percurso da fronteira.**
    ///
    /// ⛔ A fixtura é o doublet canónico: `v` tem **duas** arestas (`v–a`, `v–b`) e duas
    /// faces, que partilham três cantos. ⚠️ *É a forma exacta dos `19` que a peça do artista
    /// trazia em 2026-08-29, todos em pontas finas* — e é ela que faz a fase zero devolver
    /// `χ = 6` quando aquela peça volta a entrar.
    ///
    /// ⚠️ **O CONTROLE está na asserção da ordem:** apagar `a–v` e `v–b` deixa
    /// `a → q → b → p → a`, e um `[a, p, b, q]` daria os mesmos cantos com os lados trocados
    /// — um quad que se auto-intersecta. *Uma asserção que só contasse faces ficaria verde
    /// sobre ele.*
    #[test]
    fn a_mordida_dissolve_e_a_ordem_e_a_do_percurso() {
        let (a, b, v, p, q) = (0u32, 1, 2, 3, 4);
        let mut mesh = Mesh::from_parts(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [1.0, -1.0, 0.0],
            ],
            vec![Face::quad(a, v, b, p), Face::quad(b, v, a, q)],
        )
        .expect("a fixtura e' construida aqui");
        assert_eq!(mesh.face_count(), 2);

        let n = super::repair_doublets(&mut mesh).expect("a fusao e' exacta");
        assert_eq!(n, 1, "⛔ o doublet tem de ser contado");
        assert_eq!(mesh.face_count(), 1, "⛔ as duas faces fundem-se numa");

        // ⚠️ **A asserção é sobre POSIÇÕES e não sobre índices**: a compactação deixa cair o
        // vértice preso e **renumera** o resto, então um índice esperado seria uma afirmação
        // sobre a arrumação e não sobre a forma. *A pergunta é qual quad saiu, não com que
        // nomes.*
        let at = |i: u32| mesh.positions()[i as usize];
        let got: Vec<[f32; 3]> = mesh.faces()[0].verts().iter().map(|&i| at(i)).collect();
        let want = [
            [0.0f32, 0.0, 0.0],  // a
            [1.0, -1.0, 0.0],    // q
            [2.0, 0.0, 0.0],     // b
            [1.0, 1.0, 0.0],     // p
        ];
        let ok = (0..4).any(|r| (0..4).all(|k| got[(k + r) % 4] == want[k]));
        assert!(
            ok,
            "⛔ o quad fundido saiu {got:?} e tinha de ser uma rotacao de {want:?} -- a ordem \
             e' a do percurso da fronteira, e trocar `p` com `q` da' um quad que se \
             auto-intersecta"
        );
        assert_eq!(
            mesh.vert_count(),
            4,
            "⛔ o vertice preso tem de sair do ARQUIVO, e nao so' das faces -- um orfao \
             move a caracteristica de Euler em 1"
        );
        let _ = (v, p, q);
    }

    /// ⛔ **Uma ALMOFADA não é um doublet** — ver [`super::CellStats::mirrored_cells`].
    ///
    /// ⚠️ Nela os dois quads coincidem, logo `p == q`, e fundi-los daria um quad com **dois
    /// cantos iguais**. *A recusa vive numa linha, e sem este gate ela é invisível.*
    #[test]
    fn uma_almofada_nao_e_dissolvida_como_mordida() {
        let mut mesh = Mesh::from_parts(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![Face::quad(0, 1, 2, 3), Face::quad(3, 2, 1, 0)],
        )
        .expect("a fixtura e' construida aqui");
        let n = super::repair_doublets(&mut mesh).expect("nao ha' fusao a fazer");
        assert_eq!(
            n, 0,
            "⛔ uma almofada nao se funde -- ela DESCARTA-SE, e noutro sitio"
        );
        assert_eq!(mesh.face_count(), 2);
    }
}
