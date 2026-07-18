//! **O ARRANJO PLANAR das linhas** — a fatia R0 da wave `docs/Flip/10_regiao_por_curvas.md`.
//!
//! Entra: as mesmas polilinhas de fronteira que o `fill_at` recebe, e um ponto semente.
//! Sai: a fronteira da região que contém a semente, **feita dos vértices ORIGINAIS das
//! linhas** — pontos novos só nas interseções, e esses caem exatamente sobre as duas
//! linhas que se cruzam.
//!
//! # A ordem vem da TOPOLOGIA, nunca da proximidade
//!
//! O BUGS #16 já tentou o caminho óbvio — projetar os vértices do contorno traçado no eixo
//! da linha e reinserir os dela — e ele **destrói o anel numa quina aguda**: os dois lados
//! do bico estão à mesma distância, a projeção alterna entre eles e a região vira um nó de
//! área zero. O veredito de lá é o alicerce deste módulo:
//!
//! > *proximidade não é ordem.*
//!
//! Aqui a ordem sai do grafo: ao chegar num nó por uma meia-aresta, a seguinte é a
//! **vizinha angular** — uma pergunta puramente topológica, que não tem como alternar
//! entre dois lados de um bico porque não mede distância nenhuma.
//!
//! # O que este módulo NÃO faz
//!
//! Ele não decide *qual* região o usuário quis. Arte de mão tem vãos, e um arranjo não
//! fecha uma face que não está fechada — a resposta honesta nesse caso é `None`, e o
//! chamador cai no caminho de hoje (o raster), que é robusto justamente aí. A divisão de
//! trabalho está no §3 do plano: **o raster escolhe, o arranjo desenha**.

use crate::Vec2;

/// A fronteira de uma região: o anel externo e os buracos.
#[derive(Debug, Clone, Default)]
pub struct Region {
    pub outer: Vec<Vec2>,
    pub holes: Vec<Vec<Vec2>>,
}

/// Quanto dois pontos podem distar e ainda serem **o mesmo nó**, como fração da diagonal
/// do bbox da arte.
///
/// Relativo, e não absoluto, porque a arte do Flip vive em unidades de documento cuja
/// escala é do artista — um épsilon absoluto seria enorme num desenho pequeno e inerte num
/// grande. É a mesma lei do `FILL_TUCK_FRACTION` e do `STROKE_SIMPLIFY_FRACTION`: quando
/// não há unidade natural, a grandeza é uma fração de algo que a arte fornece.
const WELD_FRACTION: f32 = 1e-5;

/// Um segmento de entrada: de que polilinha veio, e onde.
#[derive(Clone, Copy)]
struct Seg {
    line: usize,
    /// Índice do vértice inicial dentro da polilinha.
    at: usize,
    a: Vec2,
    b: Vec2,
}

/// Uma meia-aresta do arranjo: de `from` para `to`, carregando os pontos INTERMEDIÁRIOS
/// (os vértices originais da linha entre as duas interseções).
#[derive(Clone)]
struct Half {
    from: usize,
    /// Pontos entre `from` e o destino, exclusive — os vértices que a linha já tinha.
    ///
    /// (Não há campo `to`: o destino é `halves[twin].from`, e guardá-lo duas vezes é
    /// convidar as duas cópias a divergirem.)
    via: Vec<Vec2>,
    /// Ângulo da saída em `from` (para a ordenação angular no nó).
    angle: f32,
    /// A meia-aresta oposta.
    twin: usize,
}

fn dist2(a: Vec2, b: Vec2) -> f32 {
    let (dx, dy) = (a.x - b.x, a.y - b.y);
    dx * dx + dy * dy
}

/// O parâmetro `(t, u)` da interseção de `p+t·r` com `q+u·s`, se elas se cruzam de fato
/// (ambos em `[0,1]`) e não são paralelas.
fn intersect(p: Vec2, r: Vec2, q: Vec2, s: Vec2) -> Option<(f32, f32)> {
    let denom = r.x * s.y - r.y * s.x;
    if denom == 0.0 {
        return None; // paralelas ou colineares: não geram nó
    }
    let qp = Vec2::new(q.x - p.x, q.y - p.y);
    let t = (qp.x * s.y - qp.y * s.x) / denom;
    let u = (qp.x * r.y - qp.y * r.x) / denom;
    if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
        Some((t, u))
    } else {
        None
    }
}

/// **A fronteira da região que contém `seed`** — ou `None` se ela não estiver fechada
/// pelas linhas dadas (arte com vão), que é quando o chamador deve usar o raster.
#[must_use]
pub fn region_at(strokes: &[(Vec<Vec2>, Vec<f32>, bool)], seed: Vec2) -> Option<Region> {
    let segs = collect_segments(strokes);
    if segs.is_empty() {
        return None;
    }
    let weld = weld_tolerance(&segs);
    let cuts = split_parameters(&segs);
    let (nodes, halves) = build_graph(strokes, &segs, &cuts, weld)?;
    let faces = walk_faces(&nodes, &halves);
    pick_face(faces, seed)
}

fn collect_segments(strokes: &[(Vec<Vec2>, Vec<f32>, bool)]) -> Vec<Seg> {
    let mut out = Vec::new();
    for (li, (pts, _, closed)) in strokes.iter().enumerate() {
        let n = pts.len();
        if n < 2 {
            continue;
        }
        let last = if *closed { n } else { n - 1 };
        for i in 0..last {
            out.push(Seg {
                line: li,
                at: i,
                a: pts[i],
                b: pts[(i + 1) % n],
            });
        }
    }
    out
}

fn weld_tolerance(segs: &[Seg]) -> f32 {
    let (mut lo, mut hi) = (Vec2::new(f32::MAX, f32::MAX), Vec2::new(f32::MIN, f32::MIN));
    for s in segs {
        for p in [s.a, s.b] {
            lo = Vec2::new(lo.x.min(p.x), lo.y.min(p.y));
            hi = Vec2::new(hi.x.max(p.x), hi.y.max(p.y));
        }
    }
    let diag = ((hi.x - lo.x).powi(2) + (hi.y - lo.y).powi(2)).sqrt();
    (diag * WELD_FRACTION).max(f32::MIN_POSITIVE)
}

/// Para cada segmento, os parâmetros `t` onde ele é cortado por outro — **só cortes de
/// verdade**.
///
/// ⚠️ A 1ª versão empurrava `0.0` e `1.0` para toda lista *"por conveniência"*, e com isso
/// **todo vértice original virava nó**: as arestas nasciam sempre com o `via` vazio e o
/// mecanismo que é o coração desta wave (a aresta carregar os vértices da linha) virava
/// código morto — que ainda assim produzia o anel certo, porque os vértices entravam como
/// nós. Funcionava por acidente, e a mutação que apaga o `via` sobrevivia. As fronteiras
/// de trecho são decididas no `build_graph`, que sabe distinguir *"ponta da linha"* de
/// *"interseção"*; aqui só entram interseções.
fn split_parameters(segs: &[Seg]) -> Vec<Vec<f32>> {
    let mut cuts: Vec<Vec<f32>> = vec![Vec::new(); segs.len()];
    for i in 0..segs.len() {
        for j in (i + 1)..segs.len() {
            let (si, sj) = (segs[i], segs[j]);
            // Vizinhos na MESMA polilinha compartilham um vértice: não é interseção.
            if si.line == sj.line && si.at.abs_diff(sj.at) <= 1 {
                continue;
            }
            let r = Vec2::new(si.b.x - si.a.x, si.b.y - si.a.y);
            let s = Vec2::new(sj.b.x - sj.a.x, sj.b.y - sj.a.y);
            if let Some((t, u)) = intersect(si.a, r, sj.a, s) {
                cuts[i].push(t);
                cuts[j].push(u);
            }
        }
    }
    for c in &mut cuts {
        c.sort_by(f32::total_cmp);
    }
    cuts
}

/// Solda um ponto na lista de nós (ou devolve o índice do que já está lá).
fn weld(nodes: &mut Vec<Vec2>, p: Vec2, tol: f32) -> usize {
    let t2 = tol * tol;
    if let Some(i) = nodes.iter().position(|&q| dist2(p, q) <= t2) {
        return i;
    }
    nodes.push(p);
    nodes.len() - 1
}

/// Constrói o grafo: nós **nas interseções** (e nas pontas de linha aberta), meia-arestas
/// carregando os vértices ORIGINAIS de cada trecho.
fn build_graph(
    strokes: &[(Vec<Vec2>, Vec<f32>, bool)],
    segs: &[Seg],
    cuts: &[Vec<f32>],
    weld_tol: f32,
) -> Option<(Vec<Vec2>, Vec<Half>)> {
    let mut nodes: Vec<Vec2> = Vec::new();
    let mut halves: Vec<Half> = Vec::new();

    for (li, (pts, _, closed)) in strokes.iter().enumerate() {
        let n = pts.len();
        if n < 2 {
            continue;
        }
        // A sequência da linha, na ORDEM dela, marcando o que é nó.
        let mut seq: Vec<(Vec2, bool)> = Vec::new();
        let last = if *closed { n } else { n - 1 };
        for i in 0..last {
            let si = segs
                .iter()
                .position(|s| s.line == li && s.at == i)
                .expect("todo segmento entrou na lista");
            let (a, b) = (segs[si].a, segs[si].b);
            // A ponta de uma linha ABERTA é nó (grau 1); num traço fechado, não.
            seq.push((a, !*closed && i == 0));
            for &t in &cuts[si] {
                seq.push((
                    Vec2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t),
                    true,
                ));
            }
        }
        if !*closed {
            seq.push((pts[n - 1], true));
        }
        // Um traço fechado SEM interseção nenhuma não tem nó — e um ciclo sem nó não pode
        // ser percorrido. Ancora-se o primeiro vértice, que é arbitrário e inofensivo (o
        // anel é o mesmo, só começa noutro ponto).
        if !seq.iter().any(|&(_, is_node)| is_node) {
            seq[0].1 = true;
        }
        emit_edges(&seq, *closed, &mut nodes, &mut halves, weld_tol);
    }
    if halves.is_empty() {
        return None;
    }
    Some((nodes, halves))
}

/// Corta a sequência da linha nos nós e emite uma meia-aresta por trecho.
fn emit_edges(
    seq: &[(Vec2, bool)],
    closed: bool,
    nodes: &mut Vec<Vec2>,
    halves: &mut Vec<Half>,
    weld_tol: f32,
) {
    let n = seq.len();
    let Some(start) = seq.iter().position(|&(_, is_node)| is_node) else {
        return;
    };
    let steps = if closed { n } else { n - start };
    let mut open_at: Option<usize> = None;
    let mut via: Vec<Vec2> = Vec::new();
    for k in 0..=steps {
        let idx = if closed { (start + k) % n } else { start + k };
        if !closed && idx >= n {
            break;
        }
        let (p, is_node) = seq[idx];
        if !is_node {
            via.push(p);
            continue;
        }
        let ni = weld(nodes, p, weld_tol);
        match open_at {
            None => open_at = Some(ni),
            // ⚠️ **`prev == ni` é um trecho legítimo, não um caso degenerado.** Um traço
            // fechado SEM interseção nenhuma sai e volta ao seu único nó — é um laço, e
            // recusá-lo fazia o círculo sozinho devolver `None` (o gate pegou). O que de
            // fato não é aresta é chegar ao mesmo nó sem ter andado: `via` vazio.
            Some(prev) if prev != ni || !via.is_empty() => {
                push_pair(halves, prev, ni, &via, nodes);
                via.clear();
                open_at = Some(ni);
            }
            Some(_) => via.clear(),
        }
    }
}

/// Acrescenta a meia-aresta e a gêmea.
fn push_pair(halves: &mut Vec<Half>, from: usize, to: usize, via: &[Vec2], nodes: &[Vec2]) {
    let a = halves.len();
    let first = via.first().copied().unwrap_or(nodes[to]);
    let last = via.last().copied().unwrap_or(nodes[from]);
    let ang = |o: Vec2, d: Vec2| (d.y - o.y).atan2(d.x - o.x);
    halves.push(Half {
        from,
        via: via.to_vec(),
        angle: ang(nodes[from], first),
        twin: a + 1,
    });
    let mut rev: Vec<Vec2> = via.to_vec();
    rev.reverse();
    halves.push(Half {
        from: to,
        via: rev,
        angle: ang(nodes[to], last),
        twin: a,
    });
}

/// Percorre todas as faces do arranjo pela regra do half-edge: ao chegar num nó, a
/// seguinte é a vizinha angular da gêmea.
fn walk_faces(nodes: &[Vec2], halves: &[Half]) -> Vec<Vec<Vec2>> {
    // Saídas por nó, ordenadas por ângulo (CCW).
    let mut out_of: Vec<Vec<usize>> = vec![Vec::new(); nodes.len()];
    for (i, h) in halves.iter().enumerate() {
        out_of[h.from].push(i);
    }
    for v in &mut out_of {
        v.sort_by(|&a, &b| halves[a].angle.total_cmp(&halves[b].angle));
    }
    let next_of = |h: usize| -> usize {
        let twin = halves[h].twin;
        let at = halves[twin].from;
        let ring = &out_of[at];
        let k = ring.iter().position(|&x| x == twin).unwrap_or(0);
        // A ANTERIOR em ordem CCW = a vizinha no sentido horário. É esta escolha que
        // faz o percurso abraçar a face pela esquerda e fechar anéis.
        ring[(k + ring.len() - 1) % ring.len()]
    };

    let mut seen = vec![false; halves.len()];
    let mut faces = Vec::new();
    for start in 0..halves.len() {
        if seen[start] {
            continue;
        }
        let mut ring: Vec<Vec2> = Vec::new();
        let mut h = start;
        loop {
            seen[h] = true;
            ring.push(nodes[halves[h].from]);
            ring.extend_from_slice(&halves[h].via);
            h = next_of(h);
            if h == start || seen[h] {
                break;
            }
        }
        if ring.len() >= 3 {
            faces.push(ring);
        }
    }
    faces
}

/// A face que contém a semente: a de MENOR área entre as que a contêm, e só se ela for
/// limitada (área positiva na convenção do percurso).
fn pick_face(faces: Vec<Vec<Vec2>>, seed: Vec2) -> Option<Region> {
    let mut best: Option<(f32, Vec<Vec2>)> = None;
    for f in faces {
        let a = crate::signed_area(&f);
        // A face EXTERNA percorre no sentido oposto: descartá-la é o que faz uma arte com
        // vão devolver `None` em vez de um anel que engloba o desenho inteiro.
        if a <= 0.0 || !contains(&f, seed) {
            continue;
        }
        if best.as_ref().is_none_or(|(ba, _)| a < *ba) {
            best = Some((a, f));
        }
    }
    best.map(|(_, outer)| Region {
        outer,
        holes: Vec::new(),
    })
}

fn contains(ring: &[Vec2], p: Vec2) -> bool {
    let n = ring.len();
    let mut c = false;
    for i in 0..n {
        let (a, b) = (ring[i], ring[(i + 1) % n]);
        if (a.y > p.y) != (b.y > p.y) {
            let t = (p.y - a.y) / (b.y - a.y);
            if p.x < a.x + t * (b.x - a.x) {
                c = !c;
            }
        }
    }
    c
}

#[cfg(test)]
#[path = "arrange_tests.rs"]
mod tests;
