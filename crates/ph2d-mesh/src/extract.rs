//! **O EXTRACT — a máscara vira uma PEÇA.**
//!
//! Adaptado de `reference/sculptgl/src/editing/tools/Masking.js` (`extract`),
//! MIT — ver `LICENSES/sculptgl-MIT.txt`. É o gesto que o ZBrush chama
//! *Subtool > Extract* e o Blender *Mesh > Extract Mask*: pinte a região,
//! aperte, e ela sai como um objeto novo — a armadura, a roupa, a casca.
//!
//! ⚠️ **É a única forma do módulo de fazer uma peça NOVA a partir de uma que já
//! existe.** `Add` faz primitivas e `Duplicate` copia inteiro; até aqui não
//! havia gesto que recortasse.
//!
//! # A COSTURA vem da topologia, não da máscara — e é aqui que isto deixa de ser
//! um port
//!
//! A referência decide que uma aresta é fronteira quando **as duas pontas dela**
//! estão fora da máscara ou na beira da malha. É uma heurística sobre VÉRTICES,
//! e ela erra numa faixa fina: num trecho de duas fileiras de largura *todo*
//! vértice é de fronteira, então uma aresta do MIOLO tem as duas pontas
//! marcadas e ganha uma ponte que atravessa a peça pelo meio.
//!
//! A pergunta exata é sobre ARESTAS: *quantas faces recortadas contêm esta
//! aresta?* Uma é fronteira, duas é miolo. Isso é a definição de fronteira de
//! uma superfície, não uma aproximação dela — e ainda absorve de graça o caso
//! que a referência trata à parte (uma aresta de beira da malha original, dentro
//! do recorte, tem UMA face e portanto **é** costurada, que é o certo: a casca
//! tem de fechar ali também).
//!
//! # O que a espessura decide
//!
//! * `thickness == 0` — **uma folha**. O trecho sai com a mesma orientação da
//!   superfície de onde veio, aberto. É o recorte para trabalhar em cima.
//! * `thickness > 0` — casca para FORA. A camada de trás fica mais longe da
//!   superfície, então ela é a face externa; o enrolamento inteiro é invertido
//!   para que a de fora olhe para fora.
//! * `thickness < 0` — casca para DENTRO, e aí a camada da frente já é a
//!   externa: nada a inverter.

use crate::face::{Face, TRI};
use crate::mesh::Mesh;
use crate::smooth::ring_average;

/// A partir de que valor um vértice conta como mascarado.
///
/// ⚠️ **A convenção desta casa é `0 = livre`, `1 = protegido`** — o INVERSO da
/// referência, e é a armadilha nº 1 de todo port desta área (ver `mask_ops`).
/// O extract leva o que está **protegido**, que é o que o artista acabou de
/// pintar e é o que o ZBrush leva.
const MASK_CLAMP: f32 = 0.5;

/// O erguimento da casca acima da superfície de onde ela saiu, em **fração da
/// aresta mediana do trecho**.
///
/// ⚠️ **Ele não pode ser um comprimento absoluto, e a medição diz por quê**
/// (`tests/measure_extract.rs`): entre as fixtures desta casa a aresta mediana
/// varia **5,3×** — 0,13081 na esfera de 738 vértices, 0,04908 na de 6.050 e
/// 0,02454 na de 24.386. O `eps = 0.01` da referência vale **7,6% de uma
/// aresta** na grosseira e **41%** na fina: o mesmo número significando duas
/// coisas, e na fina a casca visivelmente FLUTUA acima do barro.
///
/// A única régua que a geometria tem é a aresta dela, e é ela que este número
/// mede. A 5% a casca se separa em profundidade (não briga com o original) e
/// fica uma ordem de grandeza abaixo do detalhe que a própria malha resolve,
/// então ela lê como *encostada*.
const LIFT_FRACTION_OF_EDGE: f32 = 0.05;

/// O que o artista escolheu antes de apertar o botão.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Extract {
    /// Espessura da casca, no espaço LOCAL da malha. **Zero é uma folha só.**
    pub thickness: f32,
    /// Quantas passadas de relaxamento a COSTURA recebe.
    ///
    /// ⚠️ Ela faz duas coisas certas com uma lei só, e é o
    /// [`ring_average`](crate::ring_average) que decide qual: numa casca fechada
    /// a costura não é borda, então o anel inteiro entra e o lábio **arredonda**;
    /// numa folha a costura **é** a borda, então ela desliza ao longo de si
    /// mesma e a fronteira serrilhada que a máscara pintada à mão deixou se
    /// acalma **sem** o trecho encolher.
    pub smooth: u32,
}

impl Default for Extract {
    fn default() -> Self {
        Self {
            thickness: 0.05,
            smooth: 3,
        }
    }
}

/// **Recorta a região mascarada numa malha nova.**
///
/// Devolve `None` quando não há o que recortar — sem plano de máscara, com a
/// máscara toda em zero, ou com o recorte sem uma face inteira.
///
/// ⚠️ **A malha de origem NÃO é tocada**, nem a máscara dela: extrair é copiar.
/// O artista extrai de novo com outra espessura sem repintar nada, e é o que o
/// ZBrush faz.
#[must_use]
pub fn extract_masked(mesh: &Mesh, opts: Extract) -> Option<Mesh> {
    let masks = mesh.masks()?;
    let sel: Vec<bool> = masks.iter().map(|&m| m >= MASK_CLAMP).collect();

    // ── 1. O RECORTE: toda face que toca a máscara, e os vértices dela ───────
    //
    // ⚠️ A face entra tocando UM vértice mascarado, o que arrasta junto uma
    // franja de um anel de vértices não-mascarados. É ela que garante que o
    // recorte COBRE a máscara em vez de parar um anel antes dela.
    let mut tag = vec![u32::MAX; mesh.vert_count()];
    let mut old: Vec<u32> = Vec::new();
    let mut kept: Vec<u32> = Vec::new();
    for (fi, f) in mesh.faces().iter().enumerate() {
        if !f.verts().iter().any(|&v| sel[v as usize]) {
            continue;
        }
        kept.push(fi as u32);
        for &v in f.verts() {
            if tag[v as usize] == u32::MAX {
                tag[v as usize] = u32::try_from(old.len()).ok()?;
                old.push(v);
            }
        }
    }
    if kept.is_empty() {
        return None;
    }
    let n = u32::try_from(old.len()).ok()?;

    // ── 2. A FRONTEIRA: aresta com UMA face recortada ────────────────────────
    let bound = Boundary::of(mesh, &kept);

    // ── 3. As posições: a frente erguida, e o verso à espessura ──────────────
    let lift = LIFT_FRACTION_OF_EDGE
        * median_edge(mesh, &kept)
        * if opts.thickness < 0.0 { -1.0 } else { 1.0 };
    let shell = opts.thickness != 0.0;
    let (np, nn) = (mesh.positions(), mesh.normals());
    let mut pos: Vec<[f32; 3]> = Vec::with_capacity(old.len() * if shell { 2 } else { 1 });
    for &v in &old {
        pos.push(along(np[v as usize], nn[v as usize], lift));
    }
    if shell {
        for &v in &old {
            pos.push(along(np[v as usize], nn[v as usize], lift + opts.thickness));
        }
    }

    // ── 4. As faces: a frente, o verso invertido, e a costura ────────────────
    let mut faces: Vec<Face> = Vec::with_capacity(kept.len() * if shell { 3 } else { 1 });
    for &fi in &kept {
        faces.push(remap(mesh.faces()[fi as usize], &tag, 0));
    }
    if shell {
        for &fi in &kept {
            faces.push(flipped(remap(mesh.faces()[fi as usize], &tag, n)));
        }
        // ⚠️ **A ponte usa a aresta ao CONTRÁRIO de como a tampa da frente a
        // usa**, e é isso que faz o enrolamento fechar: a tampa da frente
        // contém `a → b`, então a ponte tem de conter `b → a`; a tampa do verso
        // é a inversa da frente, logo contém `b → a` no bloco de trás, e a
        // ponte a devolve como `a → b`. Sem essa troca a casca sai com duas
        // faces olhando para o mesmo lado da mesma aresta, e a luz a desenha
        // rasgada.
        for &fi in &kept {
            let v = mesh.faces()[fi as usize].verts();
            for k in 0..v.len() {
                let (a, b) = (v[k], v[(k + 1) % v.len()]);
                if !bound.is_boundary(a, b) {
                    continue;
                }
                let (af, bf) = (tag[a as usize], tag[b as usize]);
                faces.push(Face::quad(bf, af, af + n, bf + n));
            }
        }
    }
    // Espessura para FORA põe o verso do lado de fora, e ele nasceu invertido:
    // a peça inteira vira de uma vez.
    if opts.thickness > 0.0 {
        for f in &mut faces {
            *f = flipped(*f);
        }
    }

    let mut out = Mesh::from_parts(pos, faces).ok()?;

    // ── 5. O relaxamento da costura ──────────────────────────────────────────
    if opts.smooth > 0 {
        let seam = seam_verts(&out, &bound, &tag, n, shell);
        relax(&mut out, &seam, opts.smooth);
    }
    Some(out)
}

// ── A fronteira ─────────────────────────────────────────────────────────────

/// As arestas do recorte, ordenadas, para a pergunta *"quantas faces
/// recortadas te contêm?"*.
///
/// ⚠️ **Ordenar e contar corridas, e não um mapa de hash:** a resposta tem de
/// ser a mesma em toda máquina (a costura decide a ordem das faces da peça
/// nova, que viaja no documento), e uma varredura sobre um `Vec` ordenado não
/// tem opinião sobre semente nem sobre ordem de iteração.
struct Boundary {
    keys: Vec<u64>,
}

impl Boundary {
    fn of(mesh: &Mesh, kept: &[u32]) -> Self {
        let mut keys = Vec::with_capacity(kept.len() * 4);
        for &fi in kept {
            let v = mesh.faces()[fi as usize].verts();
            for k in 0..v.len() {
                keys.push(key(v[k], v[(k + 1) % v.len()]));
            }
        }
        keys.sort_unstable();
        Self { keys }
    }

    /// Uma aresta é fronteira do recorte quando **exatamente uma** face
    /// recortada a contém.
    fn is_boundary(&self, a: u32, b: u32) -> bool {
        let k = key(a, b);
        let lo = self.keys.partition_point(|&x| x < k);
        // `lo + 1` existe sempre que houve ao menos uma ocorrência, e ela houve:
        // a pergunta só é feita sobre arestas de faces recortadas.
        self.keys.get(lo + 1) != Some(&k)
    }

    /// Todo vértice que toca a fronteira, em índices da malha ORIGINAL.
    fn verts(&self) -> Vec<u32> {
        let mut out = Vec::new();
        let mut i = 0;
        while i < self.keys.len() {
            let k = self.keys[i];
            let run = self.keys[i..].iter().take_while(|&&x| x == k).count();
            if run == 1 {
                out.push((k >> 32) as u32);
                out.push((k & 0xffff_ffff) as u32);
            }
            i += run;
        }
        out.sort_unstable();
        out.dedup();
        out
    }
}

/// A aresta `{a, b}` sem direção, empacotada num inteiro ordenável.
fn key(a: u32, b: u32) -> u64 {
    let (lo, hi) = if a < b { (a, b) } else { (b, a) };
    (u64::from(lo) << 32) | u64::from(hi)
}

// ── O relaxamento ───────────────────────────────────────────────────────────

/// Os vértices da peça NOVA que a costura toca, expandidos por um anel.
///
/// ⚠️ **A expansão não é folga:** relaxar só o anel da costura deixa um vinco
/// exatamente um anel para dentro — o problema muda de lugar em vez de sair. É
/// o mesmo `expandsVertices(…, 1)` da referência.
fn seam_verts(out: &Mesh, bound: &Boundary, tag: &[u32], n: u32, shell: bool) -> Vec<u32> {
    let mut seed: Vec<u32> = Vec::new();
    for v in bound.verts() {
        let f = tag[v as usize];
        debug_assert!(f != u32::MAX, "a fronteira so' cita vertices do recorte");
        seed.push(f);
        if shell {
            seed.push(f + n);
        }
    }
    let adj = out.adjacency();
    let mut seen = vec![false; out.vert_count()];
    let mut all = Vec::with_capacity(seed.len() * 4);
    for &v in &seed {
        for &w in std::iter::once(&v).chain(adj.vert_verts.neighbours(v as usize)) {
            if !seen[w as usize] {
                seen[w as usize] = true;
                all.push(w);
            }
        }
    }
    all
}

/// `passes` passadas de laplaciano sobre `verts`, **em forma de Jacobi**.
///
/// ⚠️ **Jacobi e não Gauss-Seidel:** cada passada lê o estado do INÍCIO dela, e
/// o resultado deixa de depender da ordem em que os vértices aparecem na lista.
/// É a mesma lei que o solver do Wet Paint pagou para aprender — uma varredura
/// que lê o que ela mesma acabou de escrever desloca massa na direção em que
/// varre.
fn relax(out: &mut Mesh, verts: &[u32], passes: u32) {
    for _ in 0..passes {
        let before: Vec<[f32; 3]> = out.positions().to_vec();
        let moved: Vec<(u32, [f32; 3])> = verts
            .iter()
            .map(|&v| {
                let base = before[v as usize];
                (
                    v,
                    ring_average(out.adjacency(), v, base, |nb| before[nb as usize]),
                )
            })
            .collect();
        let p = out.positions_mut();
        for (v, q) in moved {
            p[v as usize] = q;
        }
    }
    out.rebuild();
}

// ── Aritmética ──────────────────────────────────────────────────────────────

/// A mediana das arestas do recorte — a régua do erguimento.
fn median_edge(mesh: &Mesh, kept: &[u32]) -> f32 {
    let p = mesh.positions();
    let mut len: Vec<f32> = Vec::with_capacity(kept.len() * 4);
    for &fi in kept {
        let v = mesh.faces()[fi as usize].verts();
        for k in 0..v.len() {
            let (a, b) = (v[k] as usize, v[(k + 1) % v.len()] as usize);
            let d = [p[b][0] - p[a][0], p[b][1] - p[a][1], p[b][2] - p[a][2]];
            len.push(d[2].mul_add(d[2], d[0].mul_add(d[0], d[1] * d[1])).sqrt());
        }
    }
    len.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    len.get(len.len() / 2).copied().unwrap_or(0.0)
}

fn along(p: [f32; 3], n: [f32; 3], k: f32) -> [f32; 3] {
    [
        n[0].mul_add(k, p[0]),
        n[1].mul_add(k, p[1]),
        n[2].mul_add(k, p[2]),
    ]
}

/// A face com os índices traduzidos para a malha nova, deslocados por `off`.
fn remap(f: Face, tag: &[u32], off: u32) -> Face {
    let mut out = [TRI; 4];
    for (k, &v) in f.verts().iter().enumerate() {
        out[k] = tag[v as usize] + off;
    }
    Face(out)
}

/// A mesma face com o enrolamento ao contrário.
fn flipped(f: Face) -> Face {
    if f.is_tri() {
        Face::tri(f.0[2], f.0[1], f.0[0])
    } else {
        Face::quad(f.0[3], f.0[2], f.0[1], f.0[0])
    }
}

#[cfg(test)]
#[path = "extract_tests.rs"]
mod tests;
