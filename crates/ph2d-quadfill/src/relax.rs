//! ⭐⭐⭐ **O ALISAMENTO QUE OLHA PARA O ÂNGULO** — a relaxação por ajuste de
//! quadrado, e a cura medida do defeito que o artista chamou «péssimo»
//! (2026-08-22).
//!
//! # ⛔ Porque o Laplaciano não podia curar isto
//!
//! [`crate::stitch`] aplica um **Laplaciano tangencial**: cada vértice anda na
//! direção do centróide dos vizinhos. ⚠️ **Ele trata a malha de quads como um
//! GRAFO** — só sabe quem é vizinho de quem — e por isso iguala **comprimentos de
//! aresta** e não sabe o que é um ângulo. *Um losango perfeito de 30° tem todas as
//! arestas iguais e é um ponto FIXO do Laplaciano.*
//!
//! ⭐ **E o defeito medido é exactamente esse.** Na `wrinkled_sphere`, com a
//! quantização a encomendar células praticamente quadradas (razão entre lados
//! vizinhos `1,14` na mediana), a saída entregava:
//!
//! | grandeza | oráculo | nós, com o Laplaciano |
//! |---|---|---|
//! | aspecto p50 | `1,08` | `1,28` — **quase certo** |
//! | ⛔ enviesamento p50 | `5°` | `18°` |
//! | ⛔ enviesamento p99 | `17°` | **`87°`** |
//! | ⛔ faces com um canto pior que 60° | **`0`** | **`8 281` de `29 468` (28 %)** |
//!
//! *O comprimento estava certo e o ângulo estava destruído* — que é a assinatura
//! de um alisador cego ao ângulo, e a razão de esta crate ter passado semanas com
//! réguas verdes sobre uma malha que o artista recusava.
//!
//! # ⭐ A lei: o quadrado mais próximo de quatro pontos tem forma FECHADA
//!
//! No plano do quad, com os quatro cantos vistos como números complexos
//! `z₀..z₃`, o conjunto dos quadrados com aquela ordem de cantos é o subespaço
//! gerado por `u = (1,1,1,1)` (a translação) e `v = (1, i, −1, −i)` (a forma).
//! ⚠️ **Os dois são ortogonais**, então a projecção de mínimos quadrados é uma
//! média pesada e não uma iteração:
//!
//! ```text
//!     c = ¼ Σ zₖ                          (o centro)
//!     a = ¼ (z₀ − i·z₁ − z₂ + i·z₃)       (a forma)
//!     wₖ = c + a·iᵏ                        (o quadrado mais próximo)
//! ```
//!
//! ⚠️ **Isto é a transformada discreta de Fourier de quatro pontos**, e `a` é o
//! primeiro harmónico. Um quadrado perfeito devolve-se a si mesmo (`|a|` é o
//! raio, os outros harmónicos são zero); um losango de 30° tem harmónico de
//! ordem 2 grande, e é ele que esta projecção **deita fora**.
//!
//! ⚠️ **A mão do quad decide-se medindo, não assumindo.** O harmónico da volta ao
//! contrário é `b = ¼ (z₀ + i·z₁ − z₂ − i·z₃)`, e para um quad bem formado ele é
//! zero. ⛔ **Um quad DOBRADO tem a volta invertida no plano dele** e `|b| > |a|`
//! — pedir-lhe o quadrado da mão errada puxaria os cantos para o lado oposto e
//! agravaria a dobra. *Escolher pelo maior módulo não é esconder um sinal: é a
//! pergunta «de que lado este quad está virado», respondida.*
//!
//! # A malha inteira: local-global
//!
//! Cada face diz onde gostaria que os seus quatro cantos estivessem; cada vértice
//! vai para a **média** dos pedidos que recebeu. ⚠️ É o esquema local-global das
//! famílias ARAP/shape-matching, e a ronda é uma contracção — por isso ela
//! converge e por isso [`LAMBDA`] amortece em vez de saltar.
//!
//! ⚠️ **Só a parte TANGENTE anda, e reprojecta-se sempre** — as duas leis que o
//! [`crate::stitch`] já pagou: a componente normal encolheria a peça a cada ronda,
//! e a reprojecção sem direcção é deliberada (uma normal estimada sobre a malha
//! que a própria ronda está a mexer realimenta-se; medido em 2026-08-22, as dobras
//! foram de 1 para 10).

use ph2d_mesh::Mesh;

/// ⛔⛔ **ZERO — MEDIDO E REJEITADO como cura** (2026-08-22). Ver este módulo,
/// que fica vivo e testado porque a **medição** é que é o resultado, não o código.
///
/// ⚠️ **O número sai da tabela, não de uma opinião** — e a tabela mede a grandeza
/// que o artista viu (`enviesamento`), não a que era fácil de medir.
///
/// # A hipótese, e porque era boa
///
/// O alisador de [`crate::stitch`] é um **Laplaciano**: trata a malha como um grafo,
/// iguala **comprimentos de aresta** e é cego ao ângulo — *um losango perfeito é
/// ponto fixo dele*. A relaxação por ajuste de quadrado ataca exactamente o que
/// falta: cada face pede o quadrado mais próximo de si (forma fechada, ver
/// [`nearest_square`]) e cada vértice vai para a média dos pedidos.
///
/// # ⛔ A tabela — orelha, `d = 1,0`, 78 403 quads
///
/// | rondas | aspecto p99 | aspecto max | `> 4×` | ⭐ **enviesamento p50** | `> 60°` | ⛔ **dobras** | ms |
/// |---|---|---|---|---|---|---|---|
/// | **0** | `7,4` | `122,7` | 3 558 | **`27°`** | 9 159 | **171** | 5 063 |
/// | 2 | `6,3` | `80,9` | 3 346 | `27°` | 8 801 | 306 | 6 346 |
/// | 4 | `5,4` | `38,6` | 3 032 | `26°` | 8 587 | 395 | 7 617 |
/// | 8 | `4,9` | `32,7` | 2 595 | `26°` | 8 276 | 497 | 10 275 |
/// | 16 | `4,6` | `30,3` | 2 143 | **`26°`** | 7 886 | **576** | 16 009 |
///
/// ⭐ **A cauda melhora muito** (o aspecto máximo cai `4×`) ⛔ **e a mediana do
/// enviesamento não se mexe: `27°` → `26°` em dezasseis rondas.** O preço são
/// `3,4×` mais dobras e `3,2×` o relógio.
///
/// # ⭐⭐⭐ O que a tabela PROVA, e é mais valioso que a feature
///
/// **Uma relaxação move vértices e mais nada.** Se dezasseis rondas de um método
/// cuja função-objectivo *é* a esquadria não movem a mediana, então endireitar um
/// quad desendireita o vizinho — ⇒ **o esmagamento está na CONECTIVIDADE**, em que
/// direcção as linhas da grade correm, e nenhum alisador lhe toca.
///
/// ⚠️ **E há um mecanismo para as dobras a mais:** num vértice irregular o pedido é
/// *contraditório* — três quads a pedir 90° cada somam 270° e têm de fechar 360°.
/// A relaxação puxa com força onde não existe solução, e a reprojecção (sem
/// direcção, deliberadamente — ver [`crate::stitch`]) aterra do lado errado do vinco.
///
/// ⭐ **A cura verdadeira ficou NOMEADA** pela sonda irmã (`sculpt3d_field_follow`):
/// medindo o desvio da grade ao campo cruzado **por família de linhas**, a nossa
/// primeira família segue o campo (`9,9°` no gancho) e a segunda não fica ortogonal
/// a ela (`19,2°` com as duas), enquanto no oráculo as duas quase coincidem
/// (`5,1°` → `7,6°`). É a assinatura da interpolação transfinita: casa com a
/// fronteira do patch e **enviesa no meio**. ⇒ *o interior de um patch tem de nascer
/// de uma parametrização alinhada ao campo* — e note-se que [`crate::fill_with`] nem sequer
/// **recebe** o campo.
///
/// ⛔ **Não volte a subir este número sem uma tabela nova.** Ligá-lo compra cauda e
/// paga dobras; o defeito que o artista fotografa é a mediana.
pub const SQUARE_ROUNDS: usize = 0;

/// **O amortecimento.** ⚠️ Meio passo, como no irmão Laplaciano: a projecção dá o
/// alvo, não o destino desta ronda.
const LAMBDA: f32 = 0.5;

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1].mul_add(b[2], -(a[2] * b[1])),
        a[2].mul_add(b[0], -(a[0] * b[2])),
        a[0].mul_add(b[1], -(a[1] * b[0])),
    ]
}

fn norm(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}

/// ⭐⭐⭐ **O QUADRADO MAIS PRÓXIMO de quatro pontos do plano** — a lei do módulo,
/// em forma fechada e sem iteração.
///
/// Recebe os quatro cantos **já centrados** (`Σ zₖ = 0`) e devolve os quatro
/// cantos do quadrado de mínimos quadrados, na mesma ordem.
///
/// ⚠️ **É `pub(crate)` e separada da ronda de propósito:** ela é a única parte
/// desta crate que é matemática pura, e uma troca de sinal aqui produziria uma
/// malha *plausível* e errada. *Uma lei que se pode testar sem malha nenhuma
/// testa-se sem malha nenhuma.*
pub(crate) fn nearest_square(z: [[f32; 2]; 4]) -> [[f32; 2]; 4] {
    // `a` = harmónico da mão directa, `b` = da mão inversa. `i·(x,y) = (−y, x)`.
    let a = [
        0.25 * (z[0][0] + z[1][1] - z[2][0] - z[3][1]),
        0.25 * (z[0][1] - z[1][0] - z[2][1] + z[3][0]),
    ];
    let b = [
        0.25 * (z[0][0] - z[1][1] - z[2][0] + z[3][1]),
        0.25 * (z[0][1] + z[1][0] - z[2][1] - z[3][0]),
    ];
    let ccw = a[0].mul_add(a[0], a[1] * a[1]) >= b[0].mul_add(b[0], b[1] * b[1]);
    let h = if ccw { a } else { b };
    let mut out = [[0.0f32; 2]; 4];
    for (k, o) in out.iter_mut().enumerate() {
        // `w = h · iᵏ` (ou `h · (−i)ᵏ` na mão inversa), em componentes.
        *o = match (k, ccw) {
            (0, _) => h,
            (1, true) | (3, false) => [-h[1], h[0]],
            (2, _) => [-h[0], -h[1]],
            _ => [h[1], -h[0]],
        };
    }
    out
}

/// **UMA RONDA de relaxação por ajuste de quadrado**, seguida de reprojecção.
///
/// ⚠️ **Faces que não são quads contribuem com a posição que já têm** — neutras,
/// não ausentes. Um vértice que só toca faces não-quad ficaria com `cnt = 0` e o
/// código teria de o tratar à parte; assim a lei é uma só. *A promessa desta
/// família é `100 %` de quads e o `non_quads` já a guarda; isto é a rede.*
pub(crate) fn square_once(mesh: &mut Mesh, reference: &Mesh, seed: f32) {
    let n = mesh.vert_count();
    let mut acc = vec![[0.0f32; 3]; n];
    let mut cnt = vec![0u32; n];
    {
        let pos = mesh.positions();
        for f in mesh.faces() {
            let v = f.verts();
            if v.len() != 4 {
                for &i in v {
                    let p = pos[i as usize];
                    for k in 0..3 {
                        acc[i as usize][k] += p[k];
                    }
                    cnt[i as usize] += 1;
                }
                continue;
            }
            let p = [
                pos[v[0] as usize],
                pos[v[1] as usize],
                pos[v[2] as usize],
                pos[v[3] as usize],
            ];
            let c3 = [
                0.25 * (p[0][0] + p[1][0] + p[2][0] + p[3][0]),
                0.25 * (p[0][1] + p[1][1] + p[2][1] + p[3][1]),
                0.25 * (p[0][2] + p[1][2] + p[2][2] + p[3][2]),
            ];
            // ⚠️ **Newell, não o produto de duas arestas.** Um quad alabeado — e
            // quase todos são, sobre uma superfície curva — não tem normal única;
            // Newell dá a do plano de mínimos quadrados, que é o plano onde o
            // ajuste tem de correr.
            let mut nrm = [0.0f32; 3];
            for k in 0..4 {
                let (a, b) = (p[k], p[(k + 1) % 4]);
                nrm[0] += (a[1] - b[1]) * (a[2] + b[2]);
                nrm[1] += (a[2] - b[2]) * (a[0] + b[0]);
                nrm[2] += (a[0] - b[0]) * (a[1] + b[1]);
            }
            let nl = norm(nrm);
            // Quad degenerado: sem plano não há ajuste, e forçar um seria inventar
            // uma direcção. Contribui neutro.
            if nl < 1.0e-12 {
                for &i in v {
                    let q = pos[i as usize];
                    for k in 0..3 {
                        acc[i as usize][k] += q[k];
                    }
                    cnt[i as usize] += 1;
                }
                continue;
            }
            let nu = [nrm[0] / nl, nrm[1] / nl, nrm[2] / nl];
            // A base do plano. ⚠️ `e2 = n × e1` faz `e1 × e2 = n`, e é isso que
            // garante que um quad enrolado no sentido directo em 3D se lê no
            // sentido directo em 2D — sem essa escolha o harmónico da mão certa
            // seria o outro.
            let r = sub(p[0], c3);
            let along = dot(r, nu);
            let e1r = [
                along.mul_add(-nu[0], r[0]),
                along.mul_add(-nu[1], r[1]),
                along.mul_add(-nu[2], r[2]),
            ];
            let e1l = norm(e1r);
            if e1l < 1.0e-12 {
                for &i in v {
                    let q = pos[i as usize];
                    for k in 0..3 {
                        acc[i as usize][k] += q[k];
                    }
                    cnt[i as usize] += 1;
                }
                continue;
            }
            let e1 = [e1r[0] / e1l, e1r[1] / e1l, e1r[2] / e1l];
            let e2 = cross(nu, e1);
            let mut z = [[0.0f32; 2]; 4];
            for k in 0..4 {
                let d = sub(p[k], c3);
                z[k] = [dot(d, e1), dot(d, e2)];
            }
            let w = nearest_square(z);
            for k in 0..4 {
                let i = v[k] as usize;
                for t in 0..3 {
                    acc[i][t] += w[k][1].mul_add(e2[t], w[k][0].mul_add(e1[t], c3[t]));
                }
                cnt[i] += 1;
            }
        }
    }
    let normals: Vec<[f32; 3]> = mesh.normals().to_vec();
    let mut next = vec![[0.0f32; 3]; n];
    {
        let pos = mesh.positions();
        for v in 0..n {
            let p = pos[v];
            if cnt[v] == 0 {
                next[v] = p;
                continue;
            }
            #[allow(clippy::cast_precision_loss)]
            let inv = 1.0 / cnt[v] as f32;
            let d = [
                acc[v][0].mul_add(inv, -p[0]),
                acc[v][1].mul_add(inv, -p[1]),
                acc[v][2].mul_add(inv, -p[2]),
            ];
            let nv = normals[v];
            let along = dot(d, nv);
            next[v] = [
                LAMBDA.mul_add(along.mul_add(-nv[0], d[0]), p[0]),
                LAMBDA.mul_add(along.mul_add(-nv[1], d[1]), p[1]),
                LAMBDA.mul_add(along.mul_add(-nv[2], d[2]), p[2]),
            ];
        }
    }
    for q in &mut next {
        *q = ph2d_remesh_iso::project_onto(reference, *q, seed);
    }
    mesh.positions_mut().copy_from_slice(&next);
    mesh.rebuild();
}

#[cfg(test)]
#[path = "relax_tests.rs"]
mod tests;
