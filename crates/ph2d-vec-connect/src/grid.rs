//! **O grafo de visibilidade ortogonal** (Wybrow/Marriott/Stuckey, GD 2009).
//!
//! A ideia: uma rota ortogonal ótima **nunca precisa dobrar num lugar arbitrário**. Ela só
//! dobra alinhada com a borda de algum obstáculo (ou com um dos endpoints). Então em vez de
//! buscar numa grade densa de pixels — cara e cheia de nós inúteis — constrói-se uma grade
//! **esparsa**: as coordenadas interessantes de `x`, as de `y`, e o produto cartesiano.
//!
//! Com dois obstáculos são `(2·2 + 2)² = 36` nós no pior caso. A busca custa microssegundos.
//! Com vinte obstáculos ainda são poucos milhares — e é por isso que o desvio de obstáculo,
//! quando chegar, é só um slice maior, não um algoritmo novo.
//!
//! ## A linha do meio
//!
//! Há uma coordenada que **não** vem de nenhuma borda e mesmo assim é indispensável: o ponto
//! médio entre as duas pontas. Sem ela, um "Z" entre duas caixas lado a lado não tem por onde
//! dobrar no meio do vão — só nas bordas das caixas —, e a rota, embora ótima, sai **feia**
//! (colada numa das caixas). É o que a §"correto vs. bonito" do módulo explica.

use crate::{Aabb, Dir, EPS};

/// Um nó do grafo: um ponto do produto cartesiano das coordenadas interessantes.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Node {
    pub at: [f64; 2],
}

/// O grafo esparso: os nós, e a vizinhança ortogonal entre eles.
pub(crate) struct Graph {
    pub nodes: Vec<Node>,
    /// `adj[i]` = os vizinhos de `i` em (E, N, W, S) — `None` onde o passo é bloqueado.
    pub adj: Vec<[Option<usize>; 4]>,
    /// O ponto médio entre as duas pontas — a **linha do meio**, contra a qual o desempate
    /// mede o quanto a rota se afasta do centro.
    pub mid: [f64; 2],
}

/// Ordena e deduplica um eixo de coordenadas. Determinístico (a CI compara bytes).
fn axis(mut v: Vec<f64>) -> Vec<f64> {
    v.sort_by(f64::total_cmp);
    v.dedup_by(|a, b| (*a - *b).abs() < EPS);
    v
}

impl Graph {
    /// Constrói o grafo para uma busca de `s0` a `s1` desviando de `obstacles` (já inflados).
    ///
    /// `s0`/`s1` são as pontas dos **stubs** — o ponto onde a linha já saiu da forma e pode
    /// finalmente dobrar. Eles entram no grafo por construção: estão fora das caixas infladas
    /// (é essa a função do jetty).
    pub(crate) fn build(s0: [f64; 2], s1: [f64; 2], obstacles: &[Aabb]) -> Self {
        let mid = [(s0[0] + s1[0]) * 0.5, (s0[1] + s1[1]) * 0.5];

        let mut xs = vec![s0[0], s1[0], mid[0]];
        let mut ys = vec![s0[1], s1[1], mid[1]];
        for o in obstacles {
            xs.push(o.min[0]);
            xs.push(o.max[0]);
            ys.push(o.min[1]);
            ys.push(o.max[1]);
        }
        let (xs, ys) = (axis(xs), axis(ys));

        // Os nós: o produto cartesiano, menos o que cai DENTRO de um obstáculo. (A borda não
        // conta como dentro — é exatamente por ela que as rotas passam, rentes ao obstáculo.)
        let mut nodes = Vec::with_capacity(xs.len() * ys.len());
        let mut index = vec![vec![usize::MAX; ys.len()]; xs.len()];
        for (ix, &x) in xs.iter().enumerate() {
            for (iy, &y) in ys.iter().enumerate() {
                let at = [x, y];
                if obstacles.iter().any(|o| o.contains(at)) {
                    continue;
                }
                index[ix][iy] = nodes.len();
                nodes.push(Node { at });
            }
        }

        // As arestas: cada nó liga ao vizinho IMEDIATO em cada direção, se o segmento aberto
        // entre os dois não atravessa nenhum obstáculo. "Imediato" é o que mantém o grafo
        // esparso: ligar a todos os visíveis daria o mesmo resultado com muito mais arestas.
        let mut adj = vec![[None; 4]; nodes.len()];
        for (ix, _) in xs.iter().enumerate() {
            for (iy, _) in ys.iter().enumerate() {
                let i = index[ix][iy];
                if i == usize::MAX {
                    continue;
                }
                for (d, dir) in Dir::ALL.iter().enumerate() {
                    let (jx, jy) = match dir {
                        Dir::East if ix + 1 < xs.len() => (ix + 1, iy),
                        Dir::North if iy + 1 < ys.len() => (ix, iy + 1),
                        Dir::West if ix > 0 => (ix - 1, iy),
                        Dir::South if iy > 0 => (ix, iy - 1),
                        _ => continue,
                    };
                    let j = index[jx][jy];
                    if j == usize::MAX {
                        continue;
                    }
                    if !blocked(nodes[i].at, nodes[j].at, obstacles) {
                        adj[i][d] = Some(j);
                    }
                }
            }
        }

        Self { nodes, adj, mid }
    }

    /// **O desempate, como uma INTEGRAL.** O quanto o segmento `a`–`b` corre longe da linha do
    /// meio: `comprimento × distância`.
    ///
    /// Somar a distância POR NÓ (que foi a minha primeira tentativa) não funciona: premia a
    /// rota com menos vértices, não a mais central. Duas rotas de mesmo custo — o "Z" pelo
    /// meio do vão e o "L" colado na caixa-alvo — empatam em comprimento e em dobras, e o L
    /// vencia só por ter um nó a menos.
    ///
    /// A integral mede o que o olho vê: um trecho comprido correndo longe do centro pesa mais
    /// que um curto. O Z, cuja vertical passa exatamente no meio, integra zero ali.
    pub(crate) fn off_center(&self, a: [f64; 2], b: [f64; 2]) -> f64 {
        let (dx, dy) = ((b[0] - a[0]).abs(), (b[1] - a[1]).abs());
        if dy < EPS {
            dx * (a[1] - self.mid[1]).abs() // horizontal: desvia em y
        } else {
            dy * (a[0] - self.mid[0]).abs() // vertical: desvia em x
        }
    }

    /// O nó exatamente em `p` (ele existe por construção: `p` é uma das coordenadas do eixo).
    pub(crate) fn find(&self, p: [f64; 2]) -> Option<usize> {
        self.nodes
            .iter()
            .position(|n| (n.at[0] - p[0]).abs() < EPS && (n.at[1] - p[1]).abs() < EPS)
    }
}

/// O segmento **aberto** `a`–`b` (ortogonal) atravessa o interior de algum obstáculo?
///
/// Testa o ponto médio e mais alguns: como o segmento liga dois nós ADJACENTES da grade, e as
/// bordas de todo obstáculo são coordenadas da grade, nenhum obstáculo pode "começar e
/// terminar" dentro dele. Logo o meio decide — e é por isso que a grade esparsa é correta.
fn blocked(a: [f64; 2], b: [f64; 2], obstacles: &[Aabb]) -> bool {
    let mid = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
    obstacles.iter().any(|o| o.contains(mid))
}
