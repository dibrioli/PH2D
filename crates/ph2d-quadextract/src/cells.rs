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

    // ── A malha: só os nós que uma célula sobrevivente usa.
    let mut slot = vec![u32::MAX; nodes.len()];
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut faces: Vec<Face> = Vec::new();
    for cell in &cells {
        let ring = reduce_ring(cell, &mut uf);
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

    Ok((Mesh::from_parts(positions, faces)?, st))
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
