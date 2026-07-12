//! **A busca** — A\* sobre o grafo esparso, com a escada de fallback que garante que a rota
//! nunca falha.
//!
//! O estado da busca é `(nó, rumo)`, não só `(nó)`: chegar num ponto **indo para o norte** é
//! diferente de chegar nele indo para o leste, porque a próxima dobra custa. Sem o rumo no
//! estado, o custo de dobra é inexprimível.

use std::collections::BinaryHeap;

use crate::grid::Graph;
use crate::loops::self_loop;
use crate::{Aabb, BEND_K, Dir, EPS, MARGIN_K, RouteInput, RouteKind};

/// O comprimento de Manhattan — a métrica natural de uma rota ortogonal (e uma heurística
/// admissível: nenhuma rota ortogonal é mais curta que ela).
fn manhattan(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).abs() + (a[1] - b[1]).abs()
}

/// **A rota.** Função TOTAL: qualquer entrada devolve uma polilinha finita, não-vazia. Nunca
/// entra em pânico, nunca produz `NaN`, nunca devolve vazio — porque ela é chamada a cada
/// frame enquanto o usuário arrasta uma caixa, e uma rota que "falha" é uma linha que some.
#[must_use]
pub fn route(input: &RouteInput) -> Vec<[f64; 2]> {
    let (p0, p1) = (input.start.at, input.end.at);

    // Um laço não se roteia: constrói-se. (É o que o mxGraph faz — o `isLoopStyleEnabled`
    // intercepta antes de qualquer edge style.)
    if let Some(bbox) = input.self_loop {
        return self_loop(bbox, input.start.dir, input.jetty, input.spread);
    }
    // Degenerado: as duas pontas no mesmo lugar.
    if manhattan(p0, p1) < EPS {
        return vec![p0, p1];
    }
    if input.kind == RouteKind::Straight {
        return vec![p0, p1];
    }

    let j = input.jetty.max(EPS);
    let (d0, d1) = (input.start.dir, input.end.dir);
    // Os stubs: o ponto onde a linha já saiu da forma e finalmente pode dobrar. É o que faz
    // um conector "sair para cima antes de virar", em vez de dobrar colado na caixa.
    let s0 = step(p0, d0, j);
    let s1 = step(p1, d1, j);

    let inflated = obstacles_for(input.obstacles, MARGIN_K * j, s0, s1);

    // **A escada de fallback.** Cada degrau afrouxa uma restrição; o último é uma reta. A
    // rota nunca falha — no máximo fica feia, e uma linha feia é infinitamente melhor que uma
    // linha que sumiu.
    let attempts: [&[Aabb]; 2] = [&inflated, &[]];
    for obs in attempts {
        if let Some(path) = search(s0, d0, s1, d1, obs) {
            return finish(p0, p1, &path);
        }
    }
    // Ainda nada (caixas sobrepostas, uma dentro da outra): o "L" cru, ortogonalizado.
    let elbow = vec![s0, [s1[0], s0[1]], s1];
    finish(p0, p1, &elbow)
}

/// Anda `d` unidades de `p` na direção `dir`.
fn step(p: [f64; 2], dir: Dir, d: f64) -> [f64; 2] {
    let v = dir.vec();
    [p[0] + v[0] * d, p[1] + v[1] * d]
}

/// Os obstáculos como o grafo os vê: inflados pela margem — **exceto quando isso engoliria a
/// ponta de um stub**.
///
/// A regra parece um detalhe e é a espinha do roteador. A caixa de onde a linha SAI é ela
/// própria um obstáculo; se ela for inflada pela margem (1,5 × jetty), ela passa a conter o
/// ponto onde o stub termina — que é justamente o único lugar por onde a rota pode começar.
/// A ponta do stub deixa de ser um nó do grafo, a busca não acha caminho nenhum, e a escada
/// de fallback entrega uma **reta que atravessa as duas formas**. Foi exatamente o que o teste
/// pegou.
///
/// Então: um obstáculo que engoliria um stub é usado **sem inflar** (a ponta do stub fica
/// fora dele por construção — ela nasce na borda e anda para fora). E se nem assim couber (a
/// forma-alvo contém a forma-fonte), ele fica **transparente para esta rota**: não faz sentido
/// desviar de uma caixa de dentro da qual a linha começa. É o que a libavoid faz com um
/// checkpoint inalcançável — ela o pula.
fn obstacles_for(raw: &[Aabb], margin: f64, s0: [f64; 2], s1: [f64; 2]) -> Vec<Aabb> {
    raw.iter()
        .filter_map(|o| {
            let big = o.inflate(margin);
            if !big.contains(s0) && !big.contains(s1) {
                return Some(big);
            }
            if !o.contains(s0) && !o.contains(s1) {
                return Some(*o); // sem folga, mas ainda um obstáculo
            }
            None // transparente: a rota começa dentro dele
        })
        .collect()
}

/// O estado do A\*: nó + rumo de chegada.
#[derive(Copy, Clone, PartialEq)]
struct State {
    f: f64,
    g: f64,
    node: usize,
    dir: usize,
    /// O desempate: **quanta linha corre longe do meio**, integrado ao longo da rota
    /// (`Σ comprimento × distância`, ver [`Graph::off_center`]).
    ///
    /// Duas tentativas anteriores falharam, e as duas por medir a coisa errada: a centralidade
    /// do NÓ ATUAL não desempata (os dois caminhos terminam no mesmo nó), e a SOMA da
    /// centralidade dos nós premia o caminho com menos vértices — que é justamente o "L"
    /// colado na caixa. A integral mede o que o olho vê.
    tb: f64,
}

impl Eq for State {}

impl Ord for State {
    /// **A ordem que decide se a rota é bonita.**
    ///
    /// Entre duas caixas lado a lado, TODA linha vertical dentro do vão dá uma rota com o
    /// mesmo comprimento e as mesmas dobras — elas são todas ótimas. Um A\* que desempatasse
    /// arbitrariamente escolheria uma qualquer, e ela costuma grudar na borda de uma das
    /// caixas: correto, e feio.
    ///
    /// O terceiro critério (`centrality`, menor é melhor) resolve isso: entre iguais, vence a
    /// que passa mais perto do meio do vão — que é onde o olho espera o "Z" dobrar. Nenhum
    /// paper menciona; o mxGraph o esconde na tabela (o `CENTER_MASK`).
    ///
    /// O quarto critério (o índice do nó) torna a ordem **total** — sem ele a busca não é
    /// determinística, e a CI que compara bytes quebra.
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // `BinaryHeap` é um max-heap; invertemos para que o MENOR `f` saia primeiro.
        //
        // **O `tb` vem logo depois do `f`, e o `g` NÃO entra na ordem.** Este detalhe custou
        // um teste vermelho: com o desempate clássico de A\* (preferir o maior `g`, "mais
        // fundo"), o `tb` nunca era consultado — as duas rotas têm o mesmo `f` e `g`
        // DIFERENTE, porque a que hugueia a caixa esconde uma dobra no stub final (ela entra
        // no `h`, não no `g`). O `g` decidia sozinho, e decidia pelo feio.
        other
            .f
            .total_cmp(&self.f)
            .then(other.tb.total_cmp(&self.tb))
            .then(other.node.cmp(&self.node))
            .then(other.dir.cmp(&self.dir))
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// O número MÍNIMO de dobras para ir de `(p, rumo)` até `q` chegando com o rumo `want`.
/// Fechado, sem busca — é a tabela do Wybrow. Mantém a heurística admissível.
fn min_bends(p: [f64; 2], dir: Dir, q: [f64; 2], want: Dir) -> f64 {
    let (dx, dy) = (q[0] - p[0], q[1] - p[1]);
    // Quantas dobras para o rumo atual virar o rumo desejado, no mínimo?
    let turn = |a: Dir, b: Dir| -> f64 {
        if a == b {
            0.0
        } else if a == b.opposite() {
            2.0
        } else {
            1.0
        }
    };
    // O rumo que o deslocamento exige em cada eixo (0 se já está alinhado).
    let need_x = if dx.abs() < EPS {
        None
    } else if dx > 0.0 {
        Some(Dir::East)
    } else {
        Some(Dir::West)
    };
    let need_y = if dy.abs() < EPS {
        None
    } else if dy > 0.0 {
        Some(Dir::North)
    } else {
        Some(Dir::South)
    };
    match (need_x, need_y) {
        // Já em cima do alvo: só falta acertar o rumo de chegada.
        (None, None) => turn(dir, want),
        // Alinhado num eixo: no mínimo, virar para lá, e depois para o rumo de chegada.
        (Some(nx), None) => turn(dir, nx) + turn(nx, want),
        (None, Some(ny)) => turn(dir, ny) + turn(ny, want),
        // Nos dois eixos: tem de percorrer os dois, em alguma ordem. Vale a melhor.
        (Some(nx), Some(ny)) => {
            let via_x = turn(dir, nx) + 1.0 + turn(ny, want);
            let via_y = turn(dir, ny) + 1.0 + turn(nx, want);
            via_x.min(via_y)
        }
    }
}

/// O A\* propriamente dito. `None` = não há rota (o chamador desce a escada de fallback).
fn search(
    s0: [f64; 2],
    d0: Dir,
    s1: [f64; 2],
    d1: Dir,
    obstacles: &[Aabb],
) -> Option<Vec<[f64; 2]>> {
    let g = Graph::build(s0, s1, obstacles);
    let (start, goal) = (g.find(s0)?, g.find(s1)?);

    // O peso de uma dobra: uma FRAÇÃO do tamanho da rota, não um número em pixels. Constante
    // durante a busca (`D` não muda), o que mantém a heurística admissível.
    let d = manhattan(s0, s1);
    let w = BEND_K * d;

    // O rumo com que a rota tem de CHEGAR em s1: contra a direção de saída do alvo (ela
    // aponta para fora dele; a linha chega vindo de fora).
    let want = d1.opposite();

    let n = g.nodes.len();
    // O custo dominante é `(g, tb)` — LEXICOGRÁFICO. Guardar só o `g` faria a busca podar o
    // caminho mais central quando ele chegasse em segundo com o MESMO custo: exatamente o
    // empate que o desempate existe para resolver.
    let mut best = vec![[(f64::INFINITY, f64::INFINITY); 4]; n];
    let mut from = vec![[usize::MAX; 4]; n]; // (nó, rumo) anterior, empacotado
    let mut heap = BinaryHeap::new();

    // O primeiro passo SÓ pode ser na direção de saída: é o stub. (Restrição DURA, não
    // penalidade — o Excalidraw acertou aqui, e é mais barato: a alternativa é inexprimível.)
    let d0i = dir_index(d0);
    best[start][d0i] = (0.0, 0.0);
    heap.push(State {
        f: manhattan(s0, s1) + w * min_bends(s0, d0, s1, want),
        g: 0.0,
        node: start,
        dir: d0i,
        tb: 0.0,
    });

    while let Some(cur) = heap.pop() {
        let b = best[cur.node][cur.dir];
        if cur.g > b.0 + EPS || (cur.g > b.0 - EPS && cur.tb > b.1 + EPS) {
            continue; // entrada obsoleta
        }
        // **Chegou.** E o rumo de chegada NÃO precisa ser o do stub: chegar por cima e dobrar
        // para dentro é o cotovelo clássico, e é legal. A única chegada proibida é a que vem
        // de DENTRO do alvo — e essa é barrada na inserção da aresta, abaixo.
        //
        // Isto é ótimo, e não por sorte: a heurística já cobra a dobra final
        // (`min_bends(v, dir, s1, want)`), então no nó-alvo `f = g + w·turn(dir, want)` é o
        // custo TOTAL exato. Como o A\* estoura em ordem de `f`, o primeiro pop no alvo é o
        // melhor. (Exigir o rumo do stub, que era o que eu fazia, deixava a busca sem saída
        // quando não havia nó atrás do alvo — e a escada de fallback entregava uma reta que
        // atravessava as duas formas. O teste pegou.)
        if cur.node == goal {
            return Some(unwind(&g, &from, cur.node, cur.dir, start, d0i));
        }
        for (di, dir) in Dir::ALL.iter().enumerate() {
            let Some(next) = g.adj[cur.node][di] else {
                continue;
            };
            // Nunca voltar por onde veio.
            if *dir == Dir::ALL[cur.dir].opposite() {
                continue;
            }
            // **Nunca chegar em s1 pela direção +d1**: isso seria chegar vindo de DENTRO do
            // alvo. Restrição dura, de novo.
            if next == goal && *dir == d1 {
                continue;
            }
            let seg = manhattan(g.nodes[cur.node].at, g.nodes[next].at);
            let bend = if di == cur.dir { 0.0 } else { w };
            let ng = cur.g + seg + bend;
            let ntb = cur.tb + g.off_center(g.nodes[cur.node].at, g.nodes[next].at);
            // Domina se for mais barato — ou, no EMPATE de custo, mais central.
            let b = best[next][di];
            if ng > b.0 + EPS || (ng > b.0 - EPS && ntb + EPS >= b.1) {
                continue;
            }
            best[next][di] = (ng, ntb);
            from[next][di] = pack(cur.node, cur.dir);
            heap.push(State {
                f: ng
                    + manhattan(g.nodes[next].at, s1)
                    + w * min_bends(g.nodes[next].at, *dir, s1, want),
                g: ng,
                node: next,
                dir: di,
                tb: ntb,
            });
        }
    }
    None
}

fn dir_index(d: Dir) -> usize {
    Dir::ALL.iter().position(|x| *x == d).unwrap_or(0)
}

fn pack(node: usize, dir: usize) -> usize {
    node * 4 + dir
}

/// Refaz o caminho de trás para a frente.
fn unwind(
    g: &Graph,
    from: &[[usize; 4]],
    mut node: usize,
    mut dir: usize,
    start: usize,
    start_dir: usize,
) -> Vec<[f64; 2]> {
    let mut out = vec![g.nodes[node].at];
    while !(node == start && dir == start_dir) {
        let p = from[node][dir];
        if p == usize::MAX {
            break;
        }
        node = p / 4;
        dir = p % 4;
        out.push(g.nodes[node].at);
    }
    out.reverse();
    out
}

/// Cola os stubs nas pontas e funde os segmentos colineares.
fn finish(p0: [f64; 2], p1: [f64; 2], mid: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let mut pts = Vec::with_capacity(mid.len() + 2);
    pts.push(p0);
    pts.extend_from_slice(mid);
    pts.push(p1);
    simplify(pts)
}

/// Funde pontos duplicados e segmentos colineares: uma polilinha com uma dobra de 0° não é
/// uma dobra, é ruído — e ela vira uma âncora a mais no path editável.
fn simplify(pts: Vec<[f64; 2]>) -> Vec<[f64; 2]> {
    let mut out: Vec<[f64; 2]> = Vec::with_capacity(pts.len());
    for p in pts {
        if out
            .last()
            .is_some_and(|l| (l[0] - p[0]).abs() < EPS && (l[1] - p[1]).abs() < EPS)
        {
            continue; // duplicado
        }
        // Três colineares? O do meio não é dobra.
        if out.len() >= 2 {
            let (a, b) = (out[out.len() - 2], out[out.len() - 1]);
            let cross = (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
            if cross.abs() < EPS {
                out.pop();
            }
        }
        out.push(p);
    }
    if out.len() < 2 {
        out.push(*out.first().unwrap_or(&[0.0, 0.0]));
    }
    out
}
