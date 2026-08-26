//! ⭐⭐⭐ **A RÉGUA DOS EDGE LOOPS** — a única queixa do artista que nunca teve número.
//!
//! ```text
//! cargo run --release -p ph2d-quadextract --example loop_census -- <quad.obj> [original.obj]
//! ```
//!
//! ⛔⛔ **Por que ela existe.** Das quatro queixas de 2026-08-25, três foram medidas e duas
//! curadas. A que sobra — *«os edge loops que normalmente são gerados em áreas de transição
//! de topologia ainda não estão no estado da arte»* — **não tinha régua nenhuma**, e uma
//! queixa sem régua não se pode fechar nem refutar. Em 2026-08-26 o artista chamou o
//! resultado de *«pro»* **excluindo os loops**: ⇒ eles são, por palavras dele, o que separa
//! este módulo do nível seguinte.
//!
//! # ⭐ O que é um LOOP, e por que ele PARA
//!
//! Um edge loop atravessa um vértice tomando a aresta **oposta**. Isso só está definido num
//! vértice de **valência 4**: com quatro arestas, a oposta a `e` é a única que não partilha
//! quad nenhum com `e` naquele vértice. ⇒ **um loop morre numa singularidade**, e é
//! exactamente isso que *«áreas de transição de topologia»* quer dizer.
//!
//! ⚠️ **A régua é a DISTRIBUIÇÃO dos comprimentos, não a média.** Uma malha com muitos loops
//! curtos e alguns longos tem a mesma média de uma com todos médios, e as duas leem-se de
//! maneira oposta no visor do artista.
//!
//! ⚠️ **E ela vem em ARESTAS, não em unidades de mundo:** um loop de `40` arestas numa malha
//! de `600` quads e outro de `40` numa de `6 000` descrevem coisas diferentes — por isso a
//! contagem de quads sai ao lado, e a coluna que se compara entre malhas é a **fracção** de
//! arestas em loops longos.

use std::collections::BTreeMap;

fn load(name: &str) -> ph2d_mesh::Mesh {
    let text = std::fs::read_to_string(name).unwrap_or_else(|e| panic!("{name}: {e}"));
    ph2d_mesh::import_obj(&text)
        .unwrap_or_else(|e| panic!("{name} nao e' um OBJ deste leitor: {e:?}"))
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("{name} nao tem peca dentro"))
        .mesh
}

type Edge = (u32, u32);

fn key(a: u32, b: u32) -> Edge {
    if a < b { (a, b) } else { (b, a) }
}

/// ⭐ **A aresta OPOSTA a `e` no vértice `v`** — a única que não partilha quad com ela ali.
///
/// `None` quando `v` não tem valência 4, que é o que faz o loop **parar numa singularidade**.
fn across(
    at_vert: &BTreeMap<u32, Vec<Edge>>,
    faces_of_edge: &BTreeMap<Edge, Vec<u32>>,
    quads: &[[u32; 4]],
    v: u32,
    e: Edge,
) -> Option<Edge> {
    let inc = at_vert.get(&v)?;
    if inc.len() != 4 {
        return None;
    }
    // Os quads que tocam `e` — as arestas deles em `v` são as VIZINHAS de `e`.
    let mut neighbour: std::collections::BTreeSet<Edge> = std::collections::BTreeSet::new();
    for &f in faces_of_edge.get(&e)? {
        let q = quads[f as usize];
        for k in 0..4 {
            let (a, b) = (q[k], q[(k + 1) % 4]);
            if a == v || b == v {
                neighbour.insert(key(a, b));
            }
        }
    }
    inc.iter()
        .copied()
        .find(|c| *c != e && !neighbour.contains(c))
}

fn main() {
    let mut args = std::env::args().skip(1);
    let name = args.next().unwrap_or_else(|| {
        panic!("uso: loop_census <quad.obj> [original.obj]");
    });
    let mesh = load(&name);

    let quads: Vec<[u32; 4]> = mesh
        .faces()
        .iter()
        .filter(|f| f.verts().len() == 4)
        .map(|f| {
            let v = f.verts();
            [v[0], v[1], v[2], v[3]]
        })
        .collect();
    let non_quads = mesh.face_count() - quads.len();

    let mut faces_of_edge: BTreeMap<Edge, Vec<u32>> = BTreeMap::new();
    for (fi, q) in quads.iter().enumerate() {
        for k in 0..4 {
            #[allow(clippy::cast_possible_truncation)]
            faces_of_edge
                .entry(key(q[k], q[(k + 1) % 4]))
                .or_default()
                .push(fi as u32);
        }
    }
    let mut at_vert: BTreeMap<u32, Vec<Edge>> = BTreeMap::new();
    for e in faces_of_edge.keys() {
        at_vert.entry(e.0).or_default().push(*e);
        at_vert.entry(e.1).or_default().push(*e);
    }

    // Cada aresta pertence a UM loop. Percorre-se em ambos os sentidos até parar.
    let mut seen: std::collections::BTreeSet<Edge> = std::collections::BTreeSet::new();
    let mut lengths: Vec<usize> = Vec::new();
    let mut closed = 0usize;
    for &start in faces_of_edge.keys() {
        if seen.contains(&start) {
            continue;
        }
        let mut chain: Vec<Edge> = vec![start];
        seen.insert(start);
        let mut fechou = false;
        // Os dois sentidos, a partir de cada ponta da aresta.
        for &from in &[start.1, start.0] {
            let (mut e, mut v) = (start, from);
            while let Some(next) = across(&at_vert, &faces_of_edge, &quads, v, e) {
                if next == start {
                    fechou = true;
                    break;
                }
                if !seen.insert(next) {
                    break;
                }
                chain.push(next);
                v = if next.0 == v { next.1 } else { next.0 };
                e = next;
            }
            if fechou {
                break;
            }
        }
        if fechou {
            closed += 1;
        }
        lengths.push(chain.len());
    }

    lengths.sort_unstable();
    let total_edges: usize = lengths.iter().sum();
    let at = |q: usize| lengths.get(lengths.len() * q / 100).copied().unwrap_or(0);
    // ⭐ A FRACÇÃO de arestas que vive em loops longos — a coluna comparável entre malhas.
    let frac = |n: usize| {
        let s: usize = lengths.iter().filter(|l| **l >= n).sum();
        100.0 * s as f64 / total_edges.max(1) as f64
    };
    // ⛔ Um loop de UMA aresta é uma aresta cercada de singularidades dos dois lados.
    let solitarias = lengths.iter().filter(|l| **l == 1).count();

    println!(
        "{name}: {} quads ({non_quads} nao-quads), {} arestas, {} LOOPS ({closed} fechados)",
        quads.len(),
        total_edges,
        lengths.len()
    );
    println!(
        "  ⭐⭐⭐ COMPRIMENTO dos loops (arestas): p50 {} p90 {} max {} · ⛔ {solitarias} de UMA aresta",
        at(50),
        at(90),
        lengths.last().copied().unwrap_or(0)
    );
    println!(
        "  ⭐⭐ FRACCAO das arestas em loops de >= 8: {:.1}% · >= 16: {:.1}% · >= 32: {:.1}%",
        frac(8),
        frac(16),
        frac(32)
    );
    // ⭐⭐⭐ **QUANTAS VOLTAS um loop dá — o ESPIRAL, em número.**
    //
    // ⛔⛔ Duas hipóteses caíram antes desta (as singularidades **agrupadas** e o **arranjo**
    // delas): medido, o oráculo tem as singularidades mais espalhadas **e** menos regulares
    // que as nossas, e mesmo assim fecha os anéis. ⇒ *a colocação não explica.*
    //
    // ⚠️ **O que a contagem revela é outra coisa.** Num quad-mesh de esfera, cada vértice de
    // valência 3 termina **três** pontas de loop; com `8` deles são `24` pontas ⇒ **`12`
    // loops abertos, obrigatórios pela topologia**. E é exactamente o que as duas malhas
    // teem: nós `12` **no total**, o oráculo `12` abertos **mais `26` fechados**.
    //
    // ⇒ A pergunta certa deixa de ser *«por que os nossos não fecham?»* e passa a ser
    // **«por que os nossos 12 obrigatórios cobrem a peça INTEIRA?»**. Um loop que dá quatro
    // voltas à peça sem fechar é um **espiral**: ele volta ao pé do sítio de onde partiu,
    // deslocado de uma linha, e recomeça.
    //
    // ⚠️ A régua é o comprimento em **circunferências de grade** (`≈ 2·√quads`), porque um
    // loop de `359` arestas numa malha de `2 152` quads e outro numa de `20 000` não dão o
    // mesmo número de voltas.
    {
        #[allow(clippy::cast_precision_loss)]
        let circ = 2.0 * (quads.len() as f64).sqrt();
        let voltas: Vec<f64> = lengths.iter().map(|l| *l as f64 / circ.max(1.0)).collect();
        let mut v = voltas;
        v.sort_by(f64::total_cmp);
        println!(
            "  ⭐⭐⭐ VOLTAS por loop (comprimento / circunferencia da grade): p50 {:.1}x              p90 {:.1}x max {:.1}x  — ⚠️ >1 significa que ele NAO fechou e recomecou deslocado",
            v.get(v.len() / 2).copied().unwrap_or(0.0),
            v.get(v.len() * 9 / 10).copied().unwrap_or(0.0),
            v.last().copied().unwrap_or(0.0)
        );
    }

    // ⭐⭐⭐ **ONDE VIVEM AS SINGULARIDADES — a propriedade que decide se um anel fecha.**
    //
    // ⛔⛔ Um loop fechado é um circuito que **não encontra singularidade nenhuma** na volta
    // inteira. ⇒ *muitos anéis fechados = as singularidades estão AGRUPADAS, deixando regiões
    // inteiras de grade pura; poucos = elas estão ESPALHADAS, e toda volta topa com uma.*
    //
    // ⚠️ **A régua é a distância GRÁFICA em arestas ao irregular mais próximo**, e não a
    // distância no mundo: um anel percorre a grade, não o espaço. *Duas singularidades a
    // meio milímetro uma da outra mas a vinte arestas de distância na grade não estão
    // agrupadas para efeito nenhum de loop.*
    //
    // ⚠️ E a **contagem** sai ao lado porque as duas leituras se confundem: uma malha com
    // menos irregulares tem-nos mais afastados por construção.
    {
        let valence: BTreeMap<u32, usize> = at_vert.iter().map(|(v, e)| (*v, e.len())).collect();
        let irregular: Vec<u32> = valence
            .iter()
            .filter(|(_, n)| **n != 4)
            .map(|(v, _)| *v)
            .collect();
        let mut adj: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
        for e in faces_of_edge.keys() {
            adj.entry(e.0).or_default().push(e.1);
            adj.entry(e.1).or_default().push(e.0);
        }
        let irr: std::collections::BTreeSet<u32> = irregular.iter().copied().collect();
        // BFS de cada irregular até topar noutro — a distância em arestas.
        let mut nearest: Vec<usize> = Vec::new();
        for &src in &irregular {
            let mut dist: BTreeMap<u32, usize> = BTreeMap::new();
            dist.insert(src, 0);
            let mut queue = std::collections::VecDeque::from([src]);
            let mut best = usize::MAX;
            while let Some(u) = queue.pop_front() {
                let d = dist[&u];
                if d >= best || d > 64 {
                    continue;
                }
                for &w in adj.get(&u).into_iter().flatten() {
                    if dist.contains_key(&w) {
                        continue;
                    }
                    dist.insert(w, d + 1);
                    if irr.contains(&w) {
                        best = best.min(d + 1);
                    } else {
                        queue.push_back(w);
                    }
                }
            }
            if best != usize::MAX {
                nearest.push(best);
            }
        }
        nearest.sort_unstable();
        let q = |k: usize| nearest.get(nearest.len() * k / 100).copied().unwrap_or(0);
        let colados = nearest.iter().filter(|d| **d <= 2).count();
        println!(
            "  ⭐⭐⭐ SINGULARIDADES: {} irregulares ({:.2}% dos vertices) · distancia em ARESTAS \
             a' vizinha: p50 {} p90 {} · ⭐ {colados} a <= 2 arestas ({:.0}% AGRUPADAS)",
            irregular.len(),
            100.0 * irregular.len() as f64 / at_vert.len().max(1) as f64,
            q(50),
            q(90),
            100.0 * colados as f64 / irregular.len().max(1) as f64
        );
        // ⭐⭐⭐ **O ARRANJO, e não a contagem nem o espalhamento.**
        //
        // ⛔⛔ Na `sphere_uv_96x144` as duas malhas têm **8** irregulares numa esfera lisa e
        // o oráculo faz **26** anéis fechados contra os nossos **`0`**. ⇒ *contagem e
        // espalhamento estão iguais; o que difere é ONDE eles ficam uns em relação aos
        // outros.* Oito vértices nos cantos de um cubo têm uma assinatura angular exacta:
        // cada um tem **três** vizinhos a `70,5°` e o oposto a `180°`.
        //
        // ⚠️ **O ângulo, e não a distância**: a peça pode ter qualquer tamanho, e é a
        // direcção a partir do centro que descreve o padrão.
        {
            let pos = mesh.positions();
            let c = irregular.iter().fold([0.0f64; 3], |a, v| {
                let p = pos[*v as usize];
                [
                    a[0] + f64::from(p[0]),
                    a[1] + f64::from(p[1]),
                    a[2] + f64::from(p[2]),
                ]
            });
            #[allow(clippy::cast_precision_loss)]
            let inv = 1.0 / irregular.len().max(1) as f64;
            let c = [c[0] * inv, c[1] * inv, c[2] * inv];
            let dir = |v: u32| {
                let p = pos[v as usize];
                let d = [
                    f64::from(p[0]) - c[0],
                    f64::from(p[1]) - c[1],
                    f64::from(p[2]) - c[2],
                ];
                let l = d[0]
                    .mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2]))
                    .sqrt()
                    .max(1.0e-12);
                [d[0] / l, d[1] / l, d[2] / l]
            };
            let mut ang: Vec<f64> = Vec::new();
            for (i, &a) in irregular.iter().enumerate() {
                for &b in irregular.iter().skip(i + 1) {
                    let (u, w) = (dir(a), dir(b));
                    ang.push(
                        u[0].mul_add(w[0], u[1].mul_add(w[1], u[2] * w[2]))
                            .clamp(-1.0, 1.0)
                            .acos()
                            .to_degrees(),
                    );
                }
            }
            ang.sort_by(f64::total_cmp);
            let sample: Vec<String> = ang.iter().take(12).map(|a| format!("{a:.0}")).collect();
            println!(
                "  ⭐⭐ ARRANJO (angulos entre pares, do centro): {} · \
                 ⚠️ oito cantos de CUBO dariam 70 70 70 (x12) e 180 (x4)",
                sample.join(" ")
            );
        }
    }

    if let Some(orig) = args.next() {
        let src = load(&orig);
        let (relief, conf) = ph2d_quadfill::follows_relief(&src, &mesh);
        let shape = ph2d_quadfill::quad_shape(&mesh);
        println!(
            "  ⭐⭐⭐ OBEDECE AO RELEVO: {relief:.1}° (confianca {conf:.2}) — ⚠️ 22,5° = «nao olhou» \
             | enviesamento p50 {:.1}° p99 {:.1}° (>60: {})",
            shape.skew_p50, shape.skew_p99, shape.skew_over_60
        );
    }
}
