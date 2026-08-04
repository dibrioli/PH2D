//! **TOPOLOGIA DINÂMICA** — o traço acrescenta detalhe onde o pincel toca.
//!
//! A promessa é a do SculptGL, e ela é sobre *não ficar preso a uma resolução*:
//! você começa numa esfera grosseira, puxa um dedo, e a malha ganha triângulos
//! exatamente ali — em vez de subdividir o modelo inteiro para poder detalhar
//! uma unha.
//!
//! # A lei, numa frase
//!
//! Uma aresta mais longa que o alvo, **dentro da esfera do dab**, é partida ao
//! meio; toda face que toca uma aresta partida é re-triangulada. A segunda
//! metade não é detalhe de implementação — é o que torna o refino
//! **livre de rachaduras por construção**: a vizinha que ninguém pediu para
//! refinar *tem* de aprender o vértice novo, senão sobra um T-vértice e a malha
//! abre um buraco fino que só aparece na luz.
//!
//! # O alvo de aresta é DERIVADO do raio, e a fórmula é a da referência
//!
//! `d2Max = radius² × (1.1 − detail) × 0.2` (`SculptBase.js::dynamicTopology`).
//! Ou seja: o detalhe que o artista escolhe é uma **fração do pincel**, nunca um
//! comprimento de mundo — um pincel pequeno detalha fino e um grande detalha
//! grosso, que é o que a mão espera. Ver [`edge_target`].
//!
//! # Duas recusas, e as duas são geometria
//!
//! * **Quads não entram** ([`Refine::NotTriangles`]): partir uma aresta de um
//!   quad devolve um triângulo e um pentágono. O SculptGL resolve isto
//!   triangulando ao entrar no modo (`MeshDynamic`, *"triangles only"*), e é o
//!   que [`crate::Mesh::triangulate`] faz — a recusa aqui existe para o motor
//!   nunca depender de quem chamou ter lembrado.
//! * **Alvo não-positivo** é no-op: um alvo zero pediria infinitos vértices, e
//!   *"o app travou"* é a pior forma de dizer que um número está errado.
//!
//! # O que esta wave NÃO faz, e o número que decide quando fará
//!
//! ⚠️ **Cada passe termina num [`crate::Mesh::rebuild`] inteiro** — anéis,
//! octree e normais. O plano chamava a estrutura mutável de *pré-requisito*, e
//! a medição (`tests/measure_dyntopo.rs`) diz que ela é a wave SEGUINTE:
//!
//! | vértices | rebuild | cabe no dab (8 ms)? |
//! |---|---|---|
//! | 6 050 | 0,59 ms | sim |
//! | 24 386 | 1,55 ms | sim |
//! | 97 922 | 5,50 ms | sim, apertado |
//!
//! E o mesmo dab toca **0,33% das faces** a 98k ⇒ o `rebuild` faz ~300× o
//! trabalho que a mudança pede. Ele cabe hoje e **deixa de caber** quando o
//! artista cruzar ~100k vértices — que é exatamente o que a topologia dinâmica
//! existe para fazer. O gatilho é um número medido, não um palpite.

use crate::face::Face;
use crate::mesh::Mesh;

/// O que o refino fez, ou por que não fez nada.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refine {
    /// Refinou. Os números são o que **cresceu**, não o total.
    Done {
        verts_added: usize,
        faces_added: usize,
        passes: usize,
    },
    /// Nenhuma aresta na esfera passou do alvo — a malha já tem a densidade
    /// pedida ali. É o desfecho NORMAL do meio de um traço.
    Enough,
    /// A malha tem quads. Ver o cabeçalho.
    NotTriangles,
}

/// **O alvo de comprimento de aresta**, em unidades de OBJETO.
///
/// `detail` anda em `[0, 1]`: `0` é o mais grosso, `1` o mais fino. A fórmula é
/// a do `SculptBase.js` (`d2Max = radius² × (1.1 − sub) × 0.2`), tirada da raiz
/// para virar comprimento — a razão de a raiz vir aqui e não no chamador é que
/// **o quadrado é detalhe do laço**, e um alvo em comprimento é o que se pode
/// comparar com uma aresta num log ou num gate.
///
/// Os extremos, para quem for reafinar: `detail = 0` dá `0,469 × raio` e
/// `detail = 1` dá `0,141 × raio` — o mais fino que a referência oferece é
/// cerca de **um sétimo do pincel**.
#[must_use]
pub fn edge_target(radius: f32, detail: f32) -> f32 {
    let d = detail.clamp(0.0, 1.0);
    radius * ((1.1 - d) * 0.2).sqrt()
}

/// Quantos passes um único dab pode gastar.
///
/// ⚠️ **É um teto de RECURSO, e o recurso é o quadro:** cada passe custa um
/// `rebuild`, e um dab que gastasse dez deles perderia o frame. Três é o que
/// leva uma aresta de `8×` o alvo até ele (cada passe a divide por dois), e uma
/// aresta assim é a que aparece quando o artista aumenta o detalhe de uma vez
/// no meio do traço — o caso raro. O comum termina em **um**.
const MAX_PASSES: usize = 3;

/// **Refina a malha dentro da esfera** até nenhuma aresta ali passar do alvo.
///
/// `center` e `radius` são do espaço da MALHA (o dab já os traz assim), e o
/// `edge_max` sai do [`edge_target`] — ele é passado em vez de derivado aqui
/// porque quem conhece o `detail` é o dono do pincel, e derivá-lo duas vezes é
/// como o log e a geometria passam a discordar.
pub fn refine_in_sphere(mesh: &mut Mesh, center: [f32; 3], radius: f32, edge_max: f32) -> Refine {
    if !mesh.faces().iter().all(Face::is_tri) {
        return Refine::NotTriangles;
    }
    if edge_max <= 0.0 || radius <= 0.0 || !edge_max.is_finite() {
        return Refine::Enough;
    }
    let (v0, f0) = (mesh.vert_count(), mesh.face_count());
    let mut passes = 0;
    while passes < MAX_PASSES {
        if !one_pass(mesh, center, radius, edge_max) {
            break;
        }
        passes += 1;
    }
    if passes == 0 {
        return Refine::Enough;
    }
    Refine::Done {
        verts_added: mesh.vert_count() - v0,
        faces_added: mesh.face_count() - f0,
        passes,
    }
}

/// Um passe. `true` se alguma coisa foi partida.
fn one_pass(mesh: &mut Mesh, center: [f32; 3], radius: f32, edge_max: f32) -> bool {
    let r2 = radius * radius;
    let emax2 = edge_max * edge_max;

    // As candidatas saem do octree — é ele que torna o dab local, e é o mesmo
    // caminho que o pincel usa para achar os vértices que move.
    let mut hits = Vec::new();
    mesh.octree().faces_in_sphere(center, radius, &mut hits);
    if hits.is_empty() {
        return false;
    }

    // ⚠️ **A aresta tem NÚMERO, e é por isso que não há mapa aqui.** O
    // `Edges` já dá um id canônico por par de vértices, então marcar é escrever
    // num vetor indexado por ele — determinístico por construção, enquanto a
    // ordem de iteração de um `HashMap` decidiria em que ordem os vértices
    // novos nascem e, com ela, os índices da malha inteira.
    let edges = mesh.edges();
    // Duas listas e não uma: `pending` diz *esta aresta parte*, `split_at` diz
    // *o vértice em que ela partiu*. Colapsá-las exigiria um valor sentinela que
    // também é um índice válido, e o dia em que o vértice 0 for um meio o refino
    // silenciosamente pula uma face.
    let mut pending = vec![false; edges.len()];
    let mut split_at = vec![u32::MAX; edges.len()];
    let positions = mesh.positions();
    let mut marked: Vec<u32> = Vec::new();

    for &f in &hits {
        let fi = f as usize;
        let Some(face) = mesh.faces().get(fi) else {
            continue;
        };
        let v = face.verts();
        for k in 0..v.len() {
            let Some(e) = edges.face_edge(fi, k) else {
                continue;
            };
            if pending[e as usize] {
                continue;
            }
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            let (pa, pb) = (positions[a as usize], positions[b as usize]);
            let d = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
            if d[0] * d[0] + d[1] * d[1] + d[2] * d[2] <= emax2 {
                continue;
            }
            // ⚠️ **O MEIO tem de estar na esfera, não um dos extremos.** É o
            // que mantém o refino dentro da pegada: uma aresta enorme que só
            // encosta na esfera com uma ponta partiria para fora do dab, e o
            // artista veria detalhe nascer onde ele não passou.
            let m = [
                (pa[0] + pb[0]) * 0.5,
                (pa[1] + pb[1]) * 0.5,
                (pa[2] + pb[2]) * 0.5,
            ];
            let dm = [m[0] - center[0], m[1] - center[1], m[2] - center[2]];
            if dm[0] * dm[0] + dm[1] * dm[1] + dm[2] * dm[2] > r2 {
                continue;
            }
            pending[e as usize] = true;
            marked.push(e);
        }
    }
    if marked.is_empty() {
        return false;
    }

    // ⚠️ **Aqui havia um `sort` + `dedup`, e o doc dizia que eles eram o que
    // torna o refino determinístico. A mutação provou a frase FALSA:** tirá-los
    // deixa os dez gates verdes, porque quem garante que uma aresta entra na
    // lista UMA vez é o `pending`, e quem garante a ordem é a travessia do
    // octree, que é determinística. Ordenar era trabalho que nenhum gate podia
    // ver — e uma linha justificada por um efeito que ela não tem é pior do que
    // linha nenhuma, porque a próxima pessoa a defende.

    let mut positions = mesh.positions().to_vec();
    let normals = mesh.normals().to_vec();
    let mut colors = mesh.colors().map(<[[f32; 3]]>::to_vec);
    let mut masks = mesh.masks().map(<[f32]>::to_vec);
    let mut ends: Vec<(u32, u32)> = vec![(0, 0); edges.len()];

    // Para saber os EXTREMOS de cada aresta marcada sem um segundo grafo: as
    // faces já os dão, e a face que contém a aresta é a que a marcou.
    for &f in &hits {
        let fi = f as usize;
        let Some(face) = mesh.faces().get(fi) else {
            continue;
        };
        let v = face.verts();
        for k in 0..v.len() {
            if let Some(e) = edges.face_edge(fi, k).filter(|&e| pending[e as usize]) {
                ends[e as usize] = (v[k], v[(k + 1) % v.len()]);
            }
        }
    }

    for &e in &marked {
        let (a, b) = ends[e as usize];
        let new = u32::try_from(positions.len()).unwrap_or(u32::MAX);
        split_at[e as usize] = new;
        let m = midpoint(&positions, &normals, a, b);
        positions.push(m);
        if let Some(c) = colors.as_mut() {
            let (ca, cb) = (c[a as usize], c[b as usize]);
            c.push([
                (ca[0] + cb[0]) * 0.5,
                (ca[1] + cb[1]) * 0.5,
                (ca[2] + cb[2]) * 0.5,
            ]);
        }
        if let Some(m) = masks.as_mut() {
            let v = (m[a as usize] + m[b as usize]) * 0.5;
            m.push(v);
        }
    }

    // ⚠️ **Toda face é examinada, não só as do dab — e a razão que eu escrevi
    // aqui primeiro estava ERRADA.** Dizia que a varredura é o que fecha a
    // rachadura; a mutação (*re-triangular só o que está em `hits`*) passou nos
    // dez gates, e ao ler o `faces_in_sphere` a causa apareceu: ele devolve
    // TODA face das folhas que a esfera toca, e uma aresta só é marcada com o
    // MEIO dentro da esfera ⇒ as duas faces que a dividem estão sempre nas
    // candidatas. Quem fecha a rachadura é o **padrão**, que é total sobre
    // qualquer subconjunto de arestas partidas (gate `every_split_pattern…`).
    //
    // A varredura FICA como camada, e agora pelo motivo verdadeiro: ela torna a
    // ausência de rachadura independente da REGRA DE MARCAÇÃO. O dia em que
    // alguém marcar por outro critério — curvatura, uma máscara, um pincel de
    // detalhe — a garantia continua de pé sem ninguém ter de reprovar este
    // raciocínio. Ela é `O(faces)` e o `rebuild` logo abaixo é `O(faces)` e ~20×
    // mais caro; quando ele sair (a estrutura mutável), esta varredura sai com
    // ele. ⚠️ Mutação registrada e NÃO sangrando, de propósito.
    let mut faces = Vec::with_capacity(mesh.face_count() + marked.len() * 2);
    for (fi, face) in mesh.faces().iter().enumerate() {
        let v = face.verts();
        let mid = |k: usize| -> Option<u32> {
            let e = edges.face_edge(fi, k)?;
            let m = split_at[e as usize];
            (m != u32::MAX).then_some(m)
        };
        emit_triangle(&mut faces, [v[0], v[1], v[2]], [mid(0), mid(1), mid(2)]);
    }

    let mut out = Mesh::from_parts(positions, faces).expect("o refino não inventa índice");
    if let Some(c) = colors {
        out.colors_mut().copy_from_slice(&c);
    }
    if let Some(m) = masks {
        out.put_masks(m);
    }
    *mesh = out;
    true
}

/// O vértice do meio, **deslocado ao longo da normal média**.
///
/// ⚠️ **Sem o deslocamento o refino ACHATA a superfície**: partir uma aresta
/// curva no meio geométrico põe o vértice novo *dentro* da curva, e refinar
/// muitas vezes lixa a forma que o artista acabou de esculpir. A correção é a do
/// `Subdivision.js`: o desvio é proporcional ao ÂNGULO entre as duas normais
/// (superfície plana ⇒ ângulo zero ⇒ meio exato) e ao comprimento da aresta.
fn midpoint(pos: &[[f32; 3]], nor: &[[f32; 3]], a: u32, b: u32) -> [f32; 3] {
    let (pa, pb) = (pos[a as usize], pos[b as usize]);
    let m = [
        (pa[0] + pb[0]) * 0.5,
        (pa[1] + pb[1]) * 0.5,
        (pa[2] + pb[2]) * 0.5,
    ];
    let (na, nb) = (unit(nor[a as usize]), unit(nor[b as usize]));
    let dot = (na[0] * nb[0] + na[1] * nb[1] + na[2] * nb[2]).clamp(-1.0, 1.0);
    let angle = dot.acos();
    if !angle.is_finite() || angle <= 0.0 {
        return m;
    }
    let d = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    // O `0.12` é da referência. Ele é o que separa "o refino segue a forma" de
    // "o refino a arredonda", e mudá-lo é decisão de LOOK, com smoke.
    let off = angle * 0.12 * len;
    let n = unit([na[0] + nb[0], na[1] + nb[1], na[2] + nb[2]]);
    [m[0] + n[0] * off, m[1] + n[1] * off, m[2] + n[2] * off]
}

fn unit(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l > 0.0 {
        [v[0] / l, v[1] / l, v[2] / l]
    } else {
        [0.0, 0.0, 1.0]
    }
}

/// Re-triangula um triângulo dados os meios das suas três arestas.
///
/// Os quatro padrões (nenhum · um · dois · três meios) são o corte clássico, e
/// os três casos de UM meio são o MESMO caso girado — escrevê-los à mão seria
/// três chances de trocar a ordem e inverter a face, que é um buraco preto na
/// luz e nada mais.
fn emit_triangle(out: &mut Vec<Face>, v: [u32; 3], mid: [Option<u32>; 3]) {
    let n = mid.iter().filter(|m| m.is_some()).count();
    match n {
        0 => out.push(Face::tri(v[0], v[1], v[2])),
        1 => {
            // Gira até o meio ser o da aresta 0.
            let k = mid.iter().position(Option::is_some).unwrap_or(0);
            let (a, b, c) = (v[k], v[(k + 1) % 3], v[(k + 2) % 3]);
            let m = mid[k].unwrap_or(a);
            out.push(Face::tri(a, m, c));
            out.push(Face::tri(m, b, c));
        }
        2 => {
            // Gira até o canto SEM meio ser o `a`: as duas arestas partidas são
            // então (a,b) e (b,c)? Não — são as que NÃO tocam `a` só num caso.
            // A forma robusta é achar a aresta não partida e girar por ela.
            let k = mid.iter().position(Option::is_none).unwrap_or(0);
            // aresta `k` = (v[k], v[k+1]) inteira ⇒ o canto compartilhado pelas
            // duas partidas é v[k+2].
            let (a, b, c) = (v[k], v[(k + 1) % 3], v[(k + 2) % 3]);
            let p = mid[(k + 1) % 3].unwrap_or(b); // meio de (b, c)
            let q = mid[(k + 2) % 3].unwrap_or(c); // meio de (c, a)
            out.push(Face::tri(p, c, q));
            out.push(Face::tri(a, b, p));
            out.push(Face::tri(a, p, q));
        }
        _ => {
            let p = mid[0].unwrap_or(v[0]);
            let q = mid[1].unwrap_or(v[1]);
            let r = mid[2].unwrap_or(v[2]);
            out.push(Face::tri(v[0], p, r));
            out.push(Face::tri(p, v[1], q));
            out.push(Face::tri(r, q, v[2]));
            out.push(Face::tri(p, q, r));
        }
    }
}

#[cfg(test)]
#[path = "dyntopo_tests.rs"]
mod tests;
