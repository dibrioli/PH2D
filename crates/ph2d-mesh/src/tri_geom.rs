//! Um triângulo **preparado para muitas perguntas** — distância de ponto e
//! travessia de raio.
//!
//! Adaptado de `src/math3d/Geometry.js` do SculptGL (MIT):
//! `distance2PointTriangleEdges` (o particionamento de 7 regiões do Eberly) e
//! `intersectionRayTriangleEdges` (Möller–Trumbore). Licença em
//! `LICENSES/sculptgl-MIT.txt`.
//!
//! # Por que um TIPO e não duas funções soltas
//!
//! O consumidor é o voxelizador: ele prepara **um** triângulo e depois pergunta
//! a ele por **dezenas de voxels**. As cinco grandezas que só dependem do
//! triângulo (as duas arestas e os três produtos internos) são computadas uma
//! vez — que é exatamente o que a referência faz à mão, com cinco variáveis
//! locais recicladas no laço de fora.
//!
//! # ⚠️ Já existe um Möller–Trumbore nesta crate, e o VIÉS deles é OPOSTO
//!
//! O [`crate::ray::ray_triangle`] (privado, o do *picking*) recusa em bordas
//! exatas: `u` e `v` são testados contra `0.0..=1.0` sem folga. Isso é o certo
//! **para um cursor** — um falso positivo na aresta partilhada elege o triângulo
//! vizinho, e o artista não distingue.
//!
//! Aqui o custo é o oposto e não é cosmético. Um falso **negativo** na aresta
//! partilhada deixa de marcar uma travessia de aresta de voxel, e o *flood fill*
//! que decide o SINAL do campo escapa por esse furo: a malha inteira sai do
//! avesso. A referência diz isso no próprio comentário — *"we favor false
//! positive just in case... mainly because of the voxel-remesh"* — então
//! [`TriEdges::ray_hit`] carrega folga nas barycêntricas.
//!
//! ⚠️ **E a folga NÃO pode ser copiada da referência.** Lá ela é `1e-15`, num
//! motor de `f64`, onde vale uns 5 ulp perto de 1.0. Em `f32` a nossa precisão
//! é `1.2e-7`, e `1.0f32 + 1e-15` **é** `1.0f32` — copiar o literal daria uma
//! função estrita com um comentário dizendo que é tolerante. O número tem de ser
//! re-derivado da precisão do tipo, e há gate pinando que o literal do original
//! seria inerte.

/// A folga das coordenadas barycêntricas.
///
/// `f32::EPSILON` é `1.19e-7` e as barycêntricas são O(1), então `1e-6` são umas
/// oito casas de ruído — largo o bastante para uma aresta partilhada acertar dos
/// dois lados, estreito o bastante para não alcançar um voxel vizinho (o passo
/// da grade é ordens de grandeza maior).
const BARY_SLACK: f32 = 1e-6;

/// O mesmo guard de paralelismo do irmão de picking: abaixo disto o raio corre
/// no plano do triângulo e não há travessia que signifique alguma coisa.
const PARALLEL_EPS: f32 = 1e-12;

/// Um triângulo com as grandezas que não dependem do ponto já computadas.
#[derive(Clone, Copy, Debug)]
pub struct TriEdges {
    v1: [f32; 3],
    e1: [f32; 3],
    e2: [f32; 3],
    a00: f32,
    a01: f32,
    a11: f32,
}

impl TriEdges {
    /// Prepara o triângulo `(v1, v2, v3)`.
    pub fn new(v1: [f32; 3], v2: [f32; 3], v3: [f32; 3]) -> Self {
        let e1 = sub(v2, v1);
        let e2 = sub(v3, v1);
        Self {
            v1,
            e1,
            e2,
            a00: dot(e1, e1),
            a01: dot(e1, e2),
            a11: dot(e2, e2),
        }
    }

    /// O ponto do triângulo mais próximo de `point`, e a distância **ao
    /// quadrado** até ele.
    ///
    /// Devolve o quadrado porque o chamador quente compara distâncias e a raiz
    /// só é tirada quando o número vai ser guardado — a referência faz o mesmo,
    /// e é a diferença entre uma raiz por voxel-triângulo e uma por escrita.
    pub fn closest_to(&self, point: [f32; 3]) -> (f32, [f32; 3]) {
        let (sq, s, t) = self.closest_bary(point);
        (sq, self.at(s, t))
    }

    /// O ponto reconstruído a partir das barycêntricas `(s, t)`.
    fn at(&self, s: f32, t: f32) -> [f32; 3] {
        [
            self.v1[0] + s * self.e1[0] + t * self.e2[0],
            self.v1[1] + s * self.e1[1] + t * self.e2[1],
            self.v1[2] + s * self.e1[2] + t * self.e2[2],
        ]
    }

    /// A MESMA busca, devolvendo as **barycêntricas** `(s, t)` do ponto mais
    /// próximo em vez do ponto — a distância ao quadrado vem junto.
    ///
    /// ⚠️ **Ela é o CORPO, e a [`Self::closest_to`] passou a delegar** — de
    /// propósito. Quem leva um canal por-vértice de uma malha para outra precisa
    /// dos PESOS (o valor no ponto mais próximo é `w1·a + s·b + t·c`), e derivá-los
    /// do ponto devolvido seria resolver de volta um sistema que esta função
    /// acabou de resolver: uma segunda resposta para a mesma pergunta, com a
    /// aritmética das sete regiões do Eberly de um lado e um solve 2×2 do outro.
    /// O peso do vértice `v1` é `1 − s − t`.
    pub fn closest_bary(&self, point: [f32; 3]) -> (f32, f32, f32) {
        // `diff` aponta do PONTO para o vértice, e é essa orientação que dá os
        // sinais de `b0`/`b1` que o particionamento abaixo espera.
        let diff = sub(self.v1, point);
        let b0 = dot(diff, self.e1);
        let b1 = dot(diff, self.e2);
        let c = dot(diff, diff);
        let (a00, a01, a11) = (self.a00, self.a01, self.a11);
        let det = (a00 * a11 - a01 * a01).abs();
        let mut s = a01 * b1 - a11 * b0;
        let mut t = a01 * b0 - a00 * b1;
        let sq;

        if s + t <= det {
            if s < 0.0 {
                if t < 0.0 {
                    // região 4 — o canto em v1
                    if b0 < 0.0 {
                        t = 0.0;
                        if -b0 >= a00 {
                            s = 1.0;
                            sq = a00 + 2.0 * b0 + c;
                        } else {
                            s = -b0 / a00;
                            sq = b0 * s + c;
                        }
                    } else {
                        s = 0.0;
                        let (nt, nsq) = clamp_edge(b1, a11, c);
                        t = nt;
                        sq = nsq;
                    }
                } else {
                    // região 3 — a aresta v1→v3
                    s = 0.0;
                    let (nt, nsq) = clamp_edge(b1, a11, c);
                    t = nt;
                    sq = nsq;
                }
            } else if t < 0.0 {
                // região 5 — a aresta v1→v2
                t = 0.0;
                let (ns, nsq) = clamp_edge(b0, a00, c);
                s = ns;
                sq = nsq;
            } else {
                // região 0 — o mínimo cai DENTRO do triângulo
                let inv = 1.0 / det;
                s *= inv;
                t *= inv;
                sq = s * (a00 * s + a01 * t + 2.0 * b0) + t * (a01 * s + a11 * t + 2.0 * b1) + c;
            }
        } else if s < 0.0 {
            // região 2
            let tmp0 = a01 + b0;
            let tmp1 = a11 + b1;
            if tmp1 > tmp0 {
                let (ns, nt, nsq) = slide_hypotenuse(tmp1 - tmp0, a00, a01, a11, b0, b1, c);
                s = ns;
                t = nt;
                sq = nsq;
            } else {
                s = 0.0;
                if tmp1 <= 0.0 {
                    t = 1.0;
                    sq = a11 + 2.0 * b1 + c;
                } else {
                    let (nt, nsq) = clamp_edge(b1, a11, c);
                    t = nt;
                    sq = nsq;
                }
            }
        } else if t < 0.0 {
            // região 6
            let tmp0 = a01 + b1;
            let tmp1 = a00 + b0;
            if tmp1 > tmp0 {
                // ⚠️ Esta é a hipotenusa vista pelo OUTRO eixo, e ela NÃO passa
                // pelo `slide_hypotenuse`. A forma quadrática casa `a00` com `s`
                // e `a11` com `t`, então trocar os dois papéis **não** é uma
                // simetria dela: reusar o helper aqui devolveria um número
                // errado com o código parecendo compartilhado. As regiões 1 e 2
                // são a mesma expressão; a 6 é a espelhada, e escreve-se.
                let numer = tmp1 - tmp0;
                let denom = a00 - 2.0 * a01 + a11;
                if numer >= denom {
                    t = 1.0;
                    s = 0.0;
                    sq = a11 + 2.0 * b1 + c;
                } else {
                    t = numer / denom;
                    s = 1.0 - t;
                    sq =
                        s * (a00 * s + a01 * t + 2.0 * b0) + t * (a01 * s + a11 * t + 2.0 * b1) + c;
                }
            } else {
                t = 0.0;
                if tmp1 <= 0.0 {
                    s = 1.0;
                    sq = a00 + 2.0 * b0 + c;
                } else {
                    let (ns, nsq) = clamp_edge(b0, a00, c);
                    s = ns;
                    sq = nsq;
                }
            }
        } else {
            // região 1 — a hipotenusa v2→v3
            let numer = a11 + b1 - a01 - b0;
            if numer <= 0.0 {
                s = 0.0;
                t = 1.0;
                sq = a11 + 2.0 * b1 + c;
            } else {
                let (ns, nt, nsq) = slide_hypotenuse(numer, a00, a01, a11, b0, b1, c);
                s = ns;
                t = nt;
                sq = nsq;
            }
        }

        // O arredondamento pode empurrar uma soma de termos que se cancelam para
        // baixo de zero; a distância ao quadrado de um ponto a um triângulo não
        // tem como ser negativa, e quem consome tira a raiz.
        (sq.max(0.0), s, t)
    }

    /// A distância ao longo de `dir` até o triângulo, ou `None`.
    ///
    /// ⚠️ **Tolerante nas bordas de propósito** — leia o cabeçalho do módulo: o
    /// modo de falha que importa aqui é o falso NEGATIVO.
    pub fn ray_hit(&self, origin: [f32; 3], dir: [f32; 3]) -> Option<f32> {
        let p = cross(dir, self.e2);
        let det = dot(self.e1, p);
        if det.abs() < PARALLEL_EPS {
            return None;
        }
        let inv = 1.0 / det;
        let tv = sub(origin, self.v1);
        let u = dot(tv, p) * inv;
        if !(-BARY_SLACK..=1.0 + BARY_SLACK).contains(&u) {
            return None;
        }
        let q = cross(tv, self.e1);
        let v = dot(dir, q) * inv;
        if v < -BARY_SLACK || u + v > 1.0 + BARY_SLACK {
            return None;
        }
        let t = dot(self.e2, q) * inv;
        if t < -BARY_SLACK { None } else { Some(t) }
    }
}

/// O mínimo sobre uma aresta que sai de `v1`, com o parâmetro grampeado a
/// `[0, 1]`. É o mesmo bloco em quatro regiões do particionamento — escrito uma
/// vez porque quatro cópias é como uma delas ganha um sinal trocado.
fn clamp_edge(b: f32, a: f32, c: f32) -> (f32, f32) {
    if b >= 0.0 {
        (0.0, c)
    } else if -b >= a {
        (1.0, a + 2.0 * b + c)
    } else {
        let x = -b / a;
        (x, b * x + c)
    }
}

/// O mínimo sobre a hipotenusa `s + t = 1`, com o parâmetro grampeado.
fn slide_hypotenuse(
    numer: f32,
    a00: f32,
    a01: f32,
    a11: f32,
    b0: f32,
    b1: f32,
    c: f32,
) -> (f32, f32, f32) {
    let denom = a00 - 2.0 * a01 + a11;
    if numer >= denom {
        (1.0, 0.0, a00 + 2.0 * b0 + c)
    } else {
        let s = numer / denom;
        let t = 1.0 - s;
        let sq = s * (a00 * s + a01 * t + 2.0 * b0) + t * (a01 * s + a11 * t + 2.0 * b1) + c;
        (s, t, sq)
    }
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[cfg(test)]
#[path = "tri_geom_tests.rs"]
mod tests;
