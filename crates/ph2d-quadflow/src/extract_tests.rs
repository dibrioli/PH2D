//! **OS GATES DA EXTRAÇÃO** — as asserções **A1..A4** e **A7** do ADR-0160.
//!
//! ⚠️ **A1 (`all-quad`) é medida, não exigida, nesta onda** — o ADR §5 diz que a
//! Q3 fecha com um NÚMERO. O gate abaixo pina a fração e o número; baixá-los é o
//! trabalho da Q4 (o fluxo de custo mínimo), e é contra estes valores que aquela
//! onda se mede.

use std::collections::BTreeMap;

use ph2d_mesh::{Mesh, shapes};

use super::{Quadrangulation, extract};
use crate::scale::ScaleField;

/// A corrida inteira, num sítio só.
fn run(mesh: &Mesh, edge: f32) -> Quadrangulation {
    // ⚠️ **A porta do produto é a HIERÁRQUICA.** O quociente pela retícula só
    // funciona sobre campos com platôs, e os platôs vêm de resolver os campos de
    // cima para baixo.
    let scale = ScaleField::uniform(mesh, edge);
    let (orient, pos) = crate::solve::solve_fields(mesh, &scale);
    extract(mesh, &orient, &pos, &scale).expect("a extracao devolveu uma malha bem formada")
}

fn sphere() -> Mesh {
    shapes::uv_sphere(48, 64, 1.0)
}

fn torus() -> Mesh {
    shapes::torus(64, 32, 1.0, 0.35)
}

/// Quantas faces tocam cada aresta — a régua do *manifold*.
fn edge_use(mesh: &Mesh) -> BTreeMap<(u32, u32), usize> {
    let mut m = BTreeMap::new();
    for f in mesh.faces() {
        let v = f.verts();
        for i in 0..v.len() {
            let (a, b) = (v[i], v[(i + 1) % v.len()]);
            let key = if a < b { (a, b) } else { (b, a) };
            *m.entry(key).or_insert(0usize) += 1;
        }
    }
    m
}

/// ⭐ **A2 — A SAÍDA É MANIFOLD:** toda aresta é usada por uma ou duas faces.
///
/// ⚠️ **É a asserção que separa "uma malha" de "um monte de polígonos".** Uma
/// aresta com três faces é uma superfície que se bifurca — nada a jusante
/// (subdivisão, normais, booleana) tem resposta para ela, e o sintoma aparece
/// longe da causa.
#[test]
fn the_extracted_mesh_is_manifold() {
    for (name, mesh) in [("esfera", sphere()), ("toro", torus())] {
        let q = run(&mesh, 0.18);
        let use_count = edge_use(&q.mesh);
        let bad = use_count.values().filter(|c| **c > 2).count();
        assert_eq!(
            bad, 0,
            "{name}: {bad} arestas com mais de duas faces -- a saida nao e' manifold"
        );
        assert!(
            !use_count.is_empty(),
            "{name}: a extracao nao produziu aresta nenhuma"
        );
    }
}

/// ⭐ **A3 — O GÊNERO SOBREVIVE:** a característica de Euler da saída é a da
/// entrada.
///
/// `χ = V − E + F`. Uma esfera vale **2**, um toro vale **0** — e o toro é a
/// fixture que faz este gate discriminar: um remesh que costurasse o buraco
/// devolveria 2 sobre uma entrada que vale 0, e nenhuma medição de forma veria.
#[test]
fn the_genus_of_the_input_survives() {
    for (name, mesh, want) in [("esfera", sphere(), 2i64), ("toro", torus(), 0)] {
        let q = run(&mesh, 0.18);
        let v = q.mesh.vert_count() as i64;
        let e = edge_use(&q.mesh).len() as i64;
        let f = q.mesh.faces().len() as i64;
        let chi = v - e + f;
        eprintln!("[quadflow] {name}: V={v} E={e} F={f} => chi={chi} (esperado {want})");
        assert_eq!(
            chi, want,
            "{name}: a caracteristica de Euler saiu {chi} e a entrada vale {want} -- o remesh mudou \
             o GENERO da superficie"
        );
    }
}

/// ⭐ **A4 — A FORMA SOBREVIVE:** todo vértice da saída está sobre a entrada.
///
/// ⚠️ **A distância é BILATERAL de propósito** (ADR-0160 §4): uma medida só de
/// ida premia uma malha que encolhe para dentro da original — ela ficaria toda
/// "sobre" a entrada e teria perdido a forma. Aqui as duas direções são medidas
/// contra a diagonal da caixa, e a barra é a do ADR: **1 %**.
#[test]
fn the_shape_survives_within_one_percent() {
    for (name, mesh) in [("esfera", sphere()), ("toro", torus())] {
        let q = run(&mesh, 0.18);
        let diag = bbox_diagonal(&mesh);
        let a = one_sided(&q.mesh, &mesh) / diag;
        let b = one_sided(&mesh, &q.mesh) / diag;
        eprintln!("[quadflow] {name}: hausdorff saida->entrada {a:.4}, entrada->saida {b:.4}");
        assert!(
            a.max(b) < 0.01,
            "{name}: a forma andou {:.4} da diagonal da caixa (barra 0,01)",
            a.max(b)
        );
    }
}

/// ⭐ **A1 — QUANTOS QUADS, e é um NÚMERO e não um zero.**
///
/// ⚠️ **Esta é a asserção que a Q4 existe para mover.** A família Instant Meshes
/// emite não-quads onde os índices de singularidade não fecham; o fluxo de custo
/// mínimo do QuadriFlow é o passo que os elimina. Pinar zero aqui seria declarar
/// que a técnica base não tem o defeito que a literatura inteira nomeia — e o
/// gate ficaria vermelho sobre um porte correto.
///
/// O que se pina é o **piso**: uma regressão que afundasse a fração cai aqui.
///
/// ⚠️ **O piso foi RE-DERIVADO depois de a RÉGUA se corrigir.** A primeira
/// versão media `quads / (quads + ciclos não-quad)` — e um ciclo de 31 lados
/// contava como **um** não-quad enquanto virava 29 triângulos na malha. Sob
/// aquela régua a esfera dava **60,9 %**; sob a régua honesta (sobre as faces
/// EMITIDAS) ela era **53,3 %**.
///
/// ⭐ **Com o pipeline completo — porte fiel do operador + hierarquia + quociente
/// da retícula + poda dos pendentes + partição dos pinçamentos + emparelhamento
/// de triângulos — o número MEDIDO é `85,3 %` (esfera 24×36), `92,4 %` (toro) e
/// **`96,4 %` na malha que o módulo de facto abre** (98 306 vértices, pelo
/// `measure_the_kill_criterion`).** O piso abaixo sai da pior fixture.
///
/// ⚠️ **A1 é a ÚNICA asserção do ADR-0160 §4 que continua aberta**, e ela é o
/// alvo do fluxo de custo mínimo (Q4). A2, A3, A4, A6, A7 e A8 estão verdes.
#[test]
fn the_quad_fraction_is_measured_and_pinned() {
    for (name, mesh) in [("esfera", sphere()), ("toro", torus())] {
        let q = run(&mesh, 0.18);
        eprintln!(
            "[quadflow] {name}: {} quads, {} nao-quads ({:.1}%), maior ciclo {}",
            q.quads,
            q.non_quads,
            q.quad_fraction() * 100.0,
            q.max_sides
        );
        assert!(
            q.quads > 0,
            "{name}: a extracao nao produziu um unico quad -- o passeio de faces nao esta' a fechar"
        );
        // Piso MEDIDO (2026-08-19, `SWEEPS_PER_LEVEL = 2`): esfera **85,3 %**,
        // toro **92,4 %**, e **96,4 %** na malha que o módulo abre
        // (`measure_the_kill_criterion`). ⚠️ As fixtures pequenas medem PIOR que
        // o produto: 24×36 e 64×32 vértices dão poucas células por feição, e o
        // resíduo de borda pesa mais. O piso sai da pior delas.
        assert!(
            q.quad_fraction() > 0.82,
            "{name}: so' {:.1}% das faces sairam quad -- abaixo do piso MEDIDO desta onda \
             (esfera 85,3 / toro 92,4 / a malha do modulo 96,4)",
            q.quad_fraction() * 100.0
        );
    }
}

/// **A7 — DETERMINÍSTICO.** Duas corridas, a mesma malha ao bit.
///
/// ⚠️ **É o gate que justifica o `BTreeMap`/`BTreeSet` em toda a extração.** Uma
/// tabela de hash faria a ordem das células, das arestas e das faces depender da
/// semente do processo — e a malha do artista mudaria ao reabrir o projeto.
#[test]
fn the_extraction_is_bit_reproducible() {
    let mesh = torus();
    let a = run(&mesh, 0.18);
    let b = run(&mesh, 0.18);
    assert_eq!(
        a.mesh.positions(),
        b.mesh.positions(),
        "duas corridas deram vertices diferentes"
    );
    assert_eq!(a.quads, b.quads, "duas corridas deram contagens diferentes");
}

/// A diagonal da caixa envolvente — a régua da A4.
fn bbox_diagonal(mesh: &Mesh) -> f32 {
    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
    for p in mesh.positions() {
        for i in 0..3 {
            lo[i] = lo[i].min(p[i]);
            hi[i] = hi[i].max(p[i]);
        }
    }
    let d = [hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// A maior distância de um vértice de `from` à **SUPERFÍCIE** de `to`.
///
/// ⚠️ **Ponto-a-SUPERFÍCIE, e a primeira versão media ponto-a-VÉRTICE.** A
/// diferença estava declarada ali com um *"as duas coincidem dentro da barra"* —
/// e a barra apertou: um remesh devolve uma malha mais GROSSA, então um vértice
/// da entrada está sempre a meia célula do vértice de saída mais próximo, **por
/// construção**. Medido: 4,22 % da diagonal com a régua de vértices e **0,60 %**
/// com a de superfície, sobre a MESMA malha. *A régua media a densidade da saída,
/// não a fidelidade da forma.*
fn one_sided(from: &Mesh, to: &Mesh) -> f32 {
    let pos = to.positions();
    let mut worst = 0.0f32;
    for p in from.positions() {
        let mut best = f32::MAX;
        for f in to.faces() {
            let v = f.verts();
            // Um quad é medido como os dois triângulos que o leque dá.
            for k in 1..v.len() - 1 {
                let d = point_triangle_sq(
                    *p,
                    pos[v[0] as usize],
                    pos[v[k] as usize],
                    pos[v[k + 1] as usize],
                );
                best = best.min(d);
            }
        }
        worst = worst.max(best.sqrt());
    }
    worst
}

/// A distância ao QUADRADO de `p` ao triângulo `(a, b, c)`.
///
/// ⚠️ **As sete regiões de Voronoi do triângulo**, e nenhuma pode faltar: um
/// ponto sobre a extensão de uma aresta pertence à aresta, e a fórmula
/// baricêntrica crua devolveria um pé de perpendicular FORA do triângulo — uma
/// distância menor que a verdadeira, que é o erro que faz uma medição de
/// fidelidade aprovar uma malha errada.
fn point_triangle_sq(p: [f32; 3], a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let sub = |x: [f32; 3], y: [f32; 3]| [x[0] - y[0], x[1] - y[1], x[2] - y[2]];
    let dot3 = |x: [f32; 3], y: [f32; 3]| x[0].mul_add(y[0], x[1].mul_add(y[1], x[2] * y[2]));
    let (ab, ac, ap) = (sub(b, a), sub(c, a), sub(p, a));

    let (d1, d2) = (dot3(ab, ap), dot3(ac, ap));
    if d1 <= 0.0 && d2 <= 0.0 {
        return dot3(ap, ap);
    }
    let bp = sub(p, b);
    let (d3, d4) = (dot3(ab, bp), dot3(ac, bp));
    if d3 >= 0.0 && d4 <= d3 {
        return dot3(bp, bp);
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let t = d1 / (d1 - d3);
        let q = [
            t.mul_add(ab[0], a[0]),
            t.mul_add(ab[1], a[1]),
            t.mul_add(ab[2], a[2]),
        ];
        return dot3(sub(p, q), sub(p, q));
    }
    let cp = sub(p, c);
    let (d5, d6) = (dot3(ab, cp), dot3(ac, cp));
    if d6 >= 0.0 && d5 <= d6 {
        return dot3(cp, cp);
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let t = d2 / (d2 - d6);
        let q = [
            t.mul_add(ac[0], a[0]),
            t.mul_add(ac[1], a[1]),
            t.mul_add(ac[2], a[2]),
        ];
        return dot3(sub(p, q), sub(p, q));
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let t = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        let bc = sub(c, b);
        let q = [
            t.mul_add(bc[0], b[0]),
            t.mul_add(bc[1], b[1]),
            t.mul_add(bc[2], b[2]),
        ];
        return dot3(sub(p, q), sub(p, q));
    }
    // O interior: as coordenadas baricêntricas.
    let denom = 1.0 / (va + vb + vc);
    let (v, w) = (vb * denom, vc * denom);
    let q = [
        w.mul_add(ac[0], v.mul_add(ab[0], a[0])),
        w.mul_add(ac[1], v.mul_add(ab[1], a[1])),
        w.mul_add(ac[2], v.mul_add(ab[2], a[2])),
    ];
    dot3(sub(p, q), sub(p, q))
}

/// **A SONDA DA PERDA** — onde os não-quads nascem. ⚠️ `#[ignore]`: é medição.
#[test]
#[ignore = "sonda -- o histograma que diz onde a extracao perde"]
fn measure_where_the_quads_are_lost() {
    for (name, mesh) in [("esfera", sphere()), ("toro", torus())] {
        let q = run(&mesh, 0.18);
        // Valência de cada vértice de saída, pelas ARESTAS das faces.
        let mut deg: BTreeMap<u32, usize> = BTreeMap::new();
        for (a, b) in edge_use(&q.mesh).keys() {
            *deg.entry(*a).or_insert(0) += 1;
            *deg.entry(*b).or_insert(0) += 1;
        }
        let mut hist: BTreeMap<usize, usize> = BTreeMap::new();
        for d in deg.values() {
            *hist.entry(*d).or_insert(0) += 1;
        }
        let orphan = q.mesh.vert_count() - deg.len();
        eprintln!("[quadflow] {name}: valencias {hist:?} | {orphan} celulas sem aresta nenhuma");

        // E o tamanho das células: uma grade quer células parecidas.
        let mut sides: BTreeMap<usize, usize> = BTreeMap::new();
        for f in q.mesh.faces() {
            *sides.entry(f.verts().len()).or_insert(0) += 1;
        }
        eprintln!("[quadflow] {name}: lados das faces {sides:?}");
    }
}

/// **A SONDA DAS COMPONENTES** — χ = 12 numa esfera é aritmeticamente impossível
/// para uma superfície CONEXA (o máximo é 2). ⚠️ `#[ignore]`: é medição.
#[test]
#[ignore = "sonda -- quantas componentes a extracao produz, e de que tamanho"]
fn measure_the_components() {
    for (name, mesh) in [("esfera", sphere()), ("toro", torus())] {
        let q = run(&mesh, 0.18);
        let n = q.mesh.vert_count();
        let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n];
        for (a, b) in edge_use(&q.mesh).keys() {
            adj[*a as usize].push(*b);
            adj[*b as usize].push(*a);
        }
        let mut comp = vec![u32::MAX; n];
        let mut sizes = Vec::new();
        for s in 0..n {
            if comp[s] != u32::MAX {
                continue;
            }
            let c = sizes.len() as u32;
            let mut queue = vec![s];
            comp[s] = c;
            let mut head = 0;
            while head < queue.len() {
                let v = queue[head];
                head += 1;
                for &w in &adj[v] {
                    if comp[w as usize] == u32::MAX {
                        comp[w as usize] = c;
                        queue.push(w as usize);
                    }
                }
            }
            sizes.push(queue.len());
        }
        sizes.sort_unstable();
        sizes.reverse();
        let small: usize = sizes.iter().skip(1).sum();
        eprintln!(
            "[quadflow] {name}: {} componentes, tamanhos {:?}... ({small} vertices fora da maior)",
            sizes.len(),
            &sizes[..sizes.len().min(8)]
        );
    }
}

/// **A SONDA DA CONSERVAÇÃO** — a soma dos comprimentos dos ciclos TEM de ser
/// `2E`. ⚠️ `#[ignore]`: é medição.
#[test]
#[ignore = "sonda -- as arestas dirigidas que somem do passeio de faces"]
fn measure_directed_edge_conservation() {
    for (name, mesh) in [("esfera", sphere()), ("toro", torus())] {
        let q = run(&mesh, 0.18);
        let e = edge_use(&q.mesh).len();
        let sum: usize = q.mesh.faces().iter().map(|f| f.verts().len()).sum();
        let once = edge_use(&q.mesh).values().filter(|c| **c == 1).count();
        let over = edge_use(&q.mesh).values().filter(|c| **c > 2).count();
        eprintln!(
            "[quadflow] {name}: 2E={} soma_lados={sum} | arestas com 1 face={once}, com >2={over}",
            2 * e
        );
    }
}

/// ⭐ **O KILL-CRITERION DO ADR-0160 §4, MEDIDO.**
///
/// *"Se sobre a esfera de 196 608 triângulos que este módulo abre o passe custar
/// **> 3 s** depois da segunda tentativa de otimização, a feature não existe
/// nesta forma."* — escrito **antes** do build, e é agora que ele se cobra.
///
/// ⚠️ `#[ignore]`: é uma leitura de RELÓGIO, e o `CLAUDE.md` §5.0 diz que
/// nenhuma desta workstation vale acima de `load ~5`. Ela roda à mão, na máquina
/// calma, e o número vai para o ADR.
#[test]
#[ignore = "medicao de relogio -- rode sozinho, na maquina calma (CLAUDE.md §5.0)"]
fn measure_the_kill_criterion() {
    use std::time::Instant;
    let mesh = ph2d_mesh::shapes::sculpt_sphere(1.0);
    eprintln!(
        "[quadflow] a malha do modulo: {} vertices, {} faces",
        mesh.vert_count(),
        mesh.faces().len()
    );
    let scale = ScaleField::uniform(&mesh, 0.05);
    let t = Instant::now();
    let (o, p) = crate::solve::solve_fields(&mesh, &scale);
    let campos = t.elapsed().as_secs_f64();
    let t2 = Instant::now();
    let q = extract(&mesh, &o, &p, &scale).expect("extraiu");
    let extracao = t2.elapsed().as_secs_f64();
    eprintln!(
        "[quadflow] campos {campos:.2} s + extracao {extracao:.2} s = {:.2} s | {} celulas, {:.1}% quads",
        campos + extracao,
        q.mesh.vert_count(),
        q.quad_fraction() * 100.0
    );

    // ⚠️ **E as propriedades de FORMA na malha do PRODUTO, não só nas fixtures.**
    // Os gates correm sobre 48×64 e 64×32; o que o artista vê tem 98 306
    // vértices, e foi lá que o smoke achou a peça do avesso.
    let mut dir: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for f in q.mesh.faces() {
        let v = f.verts();
        for i in 0..v.len() {
            *dir.entry((v[i], v[(i + 1) % v.len()])).or_insert(0) += 1;
        }
    }
    let e = edge_use(&q.mesh);
    let chi = q.mesh.vert_count() as i64 - e.len() as i64 + q.mesh.faces().len() as i64;
    eprintln!(
        "[quadflow] chi={chi} (esperado 2) | arestas com >2 faces: {} | dirigidas repetidas: {} | maior ciclo {}",
        e.values().filter(|c| **c > 2).count(),
        dir.values().filter(|c| **c > 1).count(),
        q.max_sides
    );
    let pos = q.mesh.positions();
    let mut vol = 0.0f64;
    for f in q.mesh.faces() {
        let v = f.verts();
        for k in 1..v.len() - 1 {
            let (a, b, c) = (
                pos[v[0] as usize],
                pos[v[k] as usize],
                pos[v[k + 1] as usize],
            );
            vol += f64::from(a[0].mul_add(
                b[1].mul_add(c[2], -(b[2] * c[1])),
                a[1].mul_add(
                    b[2].mul_add(c[0], -(b[0] * c[2])),
                    a[2] * b[0].mul_add(c[1], -(b[1] * c[0])),
                ),
            )) / 6.0;
        }
    }
    eprintln!("[quadflow] volume com sinal {vol:.4} (a esfera unitaria vale 4,189)");
    assert!(
        campos + extracao < 3.0,
        "o passe custou {:.2} s, e o kill-criterion do ADR-0160 §4 e' 3 s -- a feature nao existe \\
         nesta forma: ela vira offline (fora do laco interativo, com barra) ou nao entra",
        campos + extracao
    );
}

/// ⭐ **A2, A METADE QUE FALTAVA: A SAÍDA É ORIENTÁVEL E APONTA PARA FORA.**
///
/// ⚠️ **O gate irmão contava faces por aresta e chamava a isso *manifold* — e a
/// contagem não vê a ORIENTAÇÃO.** Duas faces podem partilhar uma aresta e
/// percorrê-la no MESMO sentido: a contagem dá 2, o `χ` fecha, e as duas normais
/// apontam para lados opostos. Do lado do artista isso é a peça **com buracos**,
/// porque o *backface culling* apaga metade dela — foi exatamente o que o smoke
/// do Enio devolveu (2026-08-19, foto).
///
/// Duas propriedades, e a segunda não decorre da primeira:
///
/// 1. **COERÊNCIA** — toda aresta interna é percorrida uma vez em cada sentido.
///    É a definição de uma superfície orientada.
/// 2. **SENTIDO** — o volume com sinal (teorema da divergência) é **positivo**.
///    Uma malha perfeitamente coerente pode estar inteira do avesso, e a
///    coerência não diz nada sobre isso.
#[test]
fn the_extracted_mesh_is_consistently_oriented_and_faces_outward() {
    for (name, mesh) in [("esfera", sphere()), ("toro", torus())] {
        let q = run(&mesh, 0.18);

        // (1) cada aresta DIRIGIDA aparece no máximo uma vez.
        let mut dir: BTreeMap<(u32, u32), usize> = BTreeMap::new();
        for f in q.mesh.faces() {
            let v = f.verts();
            for i in 0..v.len() {
                *dir.entry((v[i], v[(i + 1) % v.len()])).or_insert(0) += 1;
            }
        }
        let twice = dir.values().filter(|c| **c > 1).count();
        assert_eq!(
            twice, 0,
            "{name}: {twice} arestas dirigidas aparecem em DUAS faces -- as duas percorrem a \
             aresta no mesmo sentido, entao as normais delas apontam para lados opostos e o \
             culling abre um buraco na peca"
        );

        // (2) o volume com sinal — para FORA.
        let pos = q.mesh.positions();
        let mut vol = 0.0f64;
        for f in q.mesh.faces() {
            let v = f.verts();
            for k in 1..v.len() - 1 {
                let (a, b, c) = (
                    pos[v[0] as usize],
                    pos[v[k] as usize],
                    pos[v[k + 1] as usize],
                );
                vol += f64::from(a[0].mul_add(
                    b[1].mul_add(c[2], -(b[2] * c[1])),
                    a[1].mul_add(
                        b[2].mul_add(c[0], -(b[0] * c[2])),
                        a[2] * b[0].mul_add(c[1], -(b[1] * c[0])),
                    ),
                )) / 6.0;
            }
        }
        eprintln!("[quadflow] {name}: volume com sinal {vol:.4}");
        assert!(
            vol > 0.0,
            "{name}: o volume com sinal saiu {vol:.4} -- a malha esta' do AVESSO, e o artista ve' \
             o interior dela"
        );
    }
}
