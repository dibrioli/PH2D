//! **LER E PERCORRER a retícula** — onde cada amostra CAI na superfície, e que
//! amostras um ponto da face lê.
//!
//! ⭐ A posição de uma amostra é **afim** nos cantos da face: `(i·A + j·B +
//! k·C)/L`. Nada aqui projecta, nada aqui achata — e é essa ausência que faz a
//! densidade de amostras por área ser exactamente uniforme dentro de uma face.

use crate::{Sitio, Tinta, indice, sitio_quad, sitio_tri};

/// A posição de um ponto `(i, j, k)` da retícula de um triângulo `(a, b, c)`.
#[must_use]
pub fn posicao_tri(
    a: [f32; 3],
    b: [f32; 3],
    c: [f32; 3],
    lado: u32,
    ijk: (u32, u32, u32),
) -> [f32; 3] {
    let inv = 1.0 / lado as f32;
    let (i, j, k) = (ijk.0 as f32, ijk.1 as f32, ijk.2 as f32);
    [
        (i * a[0] + j * b[0] + k * c[0]) * inv,
        (i * a[1] + j * b[1] + k * c[1]) * inv,
        (i * a[2] + j * b[2] + k * c[2]) * inv,
    ]
}

/// A posição de `(i, j)` na retícula bilinear de um quad `(a, b, c, d)`.
#[must_use]
pub fn posicao_quad(q: [[f32; 3]; 4], lado: u32, ij: (u32, u32)) -> [f32; 3] {
    let inv = 1.0 / lado as f32;
    let (u, v) = (ij.0 as f32 * inv, ij.1 as f32 * inv);
    let mut out = [0.0; 3];
    for (e, o) in out.iter_mut().enumerate() {
        let baixo = q[0][e] * (1.0 - u) + q[1][e] * u;
        let cima = q[3][e] * (1.0 - u) + q[2][e] * u;
        *o = baixo * (1.0 - v) + cima * v;
    }
    out
}

/// ⭐⭐ **Os TRÊS pontos da retícula que um ponto baricêntrico lê, e os pesos.**
///
/// ⛔⛔ **A retícula de um triângulo tem sub-triângulos de DUAS orientações**, e
/// esquecer os invertidos não dá um erro — dá uma tinta que se lê **em degraus**
/// em metade da superfície, porque aqueles pontos cairiam no vizinho errado.
/// O discriminador é a soma dos pisos: `L−1` é o sub-triângulo **direito** e
/// `L−2` o **invertido**. *A soma das partes fraccionárias é sempre um inteiro,
/// logo só existem estes dois casos e o caso exacto.*
#[must_use]
pub fn leitura_tri(lado: u32, bar: [f32; 3]) -> [((u32, u32, u32), f32); 3] {
    // ⚠️ **Primeiro o ponto entra no simplexo.** Um `bar` com uma coordenada
    // negativa é um ponto FORA do triângulo, e o `floor` dele seria negativo —
    // que num `as u32` satura a zero **em silêncio** e devolve um endereço
    // plausível e errado. Cortar e renormalizar dá o ponto mais perto DENTRO,
    // que é a resposta sã para quem pergunta pela borda.
    let b = [bar[0].max(0.0), bar[1].max(0.0), bar[2].max(0.0)];
    let s = b[0] + b[1] + b[2];
    let b = if s > 0.0 {
        [b[0] / s, b[1] / s, b[2] / s]
    } else {
        [1.0, 0.0, 0.0]
    };
    let l = lado as f32;
    let sc = [b[0] * l, b[1] * l, b[2] * l];
    let p = [
        sc[0].floor().clamp(0.0, l),
        sc[1].floor().clamp(0.0, l),
        sc[2].floor().clamp(0.0, l),
    ];
    let f = [sc[0] - p[0], sc[1] - p[1], sc[2] - p[2]];
    let (i, j, k) = (p[0] as u32, p[1] as u32, p[2] as u32);
    // ⭐ As partes fraccionárias somam um INTEIRO (porque `b` soma `1`), logo
    // `lado − Σpisos` só pode ser `0`, `1` ou `2`: o ponto é da retícula, ou
    // está num sub-triângulo direito, ou num invertido.
    match lado - (i + j + k).min(lado) {
        0 => [((i, j, k), 1.0), ((i, j, k), 0.0), ((i, j, k), 0.0)],
        1 => [
            ((i + 1, j, k), f[0]),
            ((i, j + 1, k), f[1]),
            ((i, j, k + 1), f[2]),
        ],
        _ => [
            ((i, j + 1, k + 1), 1.0 - f[0]),
            ((i + 1, j, k + 1), 1.0 - f[1]),
            ((i + 1, j + 1, k), 1.0 - f[2]),
        ],
    }
}

/// ⭐⭐ **Os QUATRO pontos da retícula que um ponto de um QUAD lê, e os pesos.**
///
/// ⛔⛔ **Ela NÃO existia, e a ausência era do tamanho do produto:** a
/// [`leitura_tri`] é a única leitura desta crate, e a malha de escultura desta
/// casa é **quase toda de quads** (a `uv_sphere` só tem triângulos nos dois
/// pólos). *Uma lei de leitura que só sabe ler um terço da superfície do
/// produto não é uma lei de leitura* — e a mesma ausência já tinha mordido uma
/// vez, do lado da ESCRITA, quando o laço do dab só tratava triângulos e o
/// pincel não pintava nada.
///
/// ⭐ **Aqui não há sub-triângulos invertidos** (a razão de a irmã ser
/// complicada): a retícula de um quad é uma grelha, e uma célula tem sempre
/// quatro cantos. A leitura é a **bilinear** deles.
///
/// ⚠️ **O `clamp` do piso em `L−1` é load-bearing e não defesa:** em `u = 1`
/// exacto o `floor` dá `L`, e sem o corte a célula seria `[L, L+1]` — um
/// endereço FORA da face. Com o corte a célula é a última e `fu` vale `1`,
/// que devolve o canto. *A borda de uma face é o sítio onde uma leitura erra
/// sem que ninguém veja, porque ali o vizinho tem quase a mesma cor.*
#[must_use]
pub fn leitura_quad(lado: u32, uv: [f32; 2]) -> [((u32, u32), f32); 4] {
    // ⚠️ Primeiro o ponto entra no quadrado, pela razão que a irmã escreve: um
    // `floor` negativo satura a zero **em silêncio** num `as u32` e devolve um
    // endereço plausível e errado.
    let u = uv[0].clamp(0.0, 1.0);
    let v = uv[1].clamp(0.0, 1.0);
    let l = lado as f32;
    let (su, sv) = (u * l, v * l);
    let i = (su.floor() as u32).min(lado - 1);
    let j = (sv.floor() as u32).min(lado - 1);
    let fu = su - i as f32;
    let fv = sv - j as f32;
    [
        ((i, j), (1.0 - fu) * (1.0 - fv)),
        ((i + 1, j), fu * (1.0 - fv)),
        ((i + 1, j + 1), fu * fv),
        ((i, j + 1), (1.0 - fu) * fv),
    ]
}

impl Tinta {
    /// ⭐ **Percorre TODAS as amostras de uma face, uma vez cada.**
    ///
    /// ⚠️ **«Uma vez cada» é por FACE, não por malha:** uma amostra de aresta é
    /// visitada pelas DUAS faces que a partilham, e uma de canto por todas as
    /// faces do anel. Quem acumula (um pincel que mistura com peso) tem de se
    /// lembrar de quem já visitou — ver o cabeçalho do `dab` do consumidor.
    /// *Guardar essa memória aqui obrigaria esta crate a alocar por chamada.*
    pub fn para_cada_amostra_tri(
        &self,
        face: usize,
        cantos: &[u32],
        mut f: impl FnMut(u32, (u32, u32, u32)),
    ) {
        let l = self.lado_da_face(face);
        for i in 0..=l {
            for j in 0..=(l - i) {
                let k = l - i - j;
                f(
                    indice(self.topologia(), face, sitio_tri(l, i, j, k), cantos),
                    (i, j, k),
                );
            }
        }
    }

    /// A irmã para QUADS.
    pub fn para_cada_amostra_quad(
        &self,
        face: usize,
        cantos: &[u32],
        mut f: impl FnMut(u32, (u32, u32)),
    ) {
        let l = self.lado_da_face(face);
        for j in 0..=l {
            for i in 0..=l {
                f(
                    indice(self.topologia(), face, sitio_quad(l, i, j), cantos),
                    (i, j),
                );
            }
        }
    }

    /// ⭐⭐⭐ **A COR num ponto baricêntrico de um triângulo** — a porta que o
    /// renderizador e o oráculo partilham.
    ///
    /// ⛔ Ela existe para que a lei da interpolação tenha **um** dono: o gémeo
    /// em WGSL é conferido contra esta função, e não contra uma segunda
    /// redacção da mesma aritmética.
    #[must_use]
    pub fn cor_tri(&self, face: usize, cantos: &[u32], bar: [f32; 3]) -> [f32; 3] {
        let l = self.lado_da_face(face);
        let mut out = [0.0f32; 3];
        for (ijk, peso) in leitura_tri(l, bar) {
            let idx = indice(
                self.topologia(),
                face,
                sitio_tri(l, ijk.0, ijk.1, ijk.2),
                cantos,
            ) as usize;
            let c = self.amostras()[idx];
            for e in 0..3 {
                out[e] += c[e] * peso;
            }
        }
        out
    }

    /// ⭐⭐ **A COR num ponto `(u, v)` de um QUAD** — a irmã da [`Self::cor_tri`],
    /// e o outro dono da lei que o gémeo em WGSL confere.
    #[must_use]
    pub fn cor_quad(&self, face: usize, cantos: &[u32], uv: [f32; 2]) -> [f32; 3] {
        let l = self.lado_da_face(face);
        let mut out = [0.0f32; 3];
        for (ij, peso) in leitura_quad(l, uv) {
            let idx = indice(self.topologia(), face, sitio_quad(l, ij.0, ij.1), cantos) as usize;
            let c = self.amostras()[idx];
            for e in 0..3 {
                out[e] += c[e] * peso;
            }
        }
        out
    }

    /// O sítio de um ponto da retícula de `face`, resolvido em índice global.
    #[must_use]
    pub fn indice_de(&self, face: usize, cantos: &[u32], sitio: Sitio) -> u32 {
        indice(self.topologia(), face, sitio, cantos)
    }
}
