//! O RAIO — a pergunta *"o que está sob o cursor?"*.
//!
//! Sem ela não existe **gesto**: a W1 pôs a malha na tela e deu uma câmera, mas
//! um pincel precisa saber onde a superfície está sob o ponteiro. É o
//! pré-requisito da W2 inteira, e por isso ele vem antes dos verbos.
//!
//! Interseção raio-triângulo por **Möller–Trumbore** (1997) — a forma padrão,
//! sem pré-computar plano por face (que custaria 16 B/face de memória residente
//! para poupar meia dúzia de flops num teste que já é podado pelo octree).
//!
//! ⚠️ **Sem culling, e a decisão é a mesma do renderer.** O `MeshRenderer` shipa
//! `cull_mode: None` porque uma casca aberta e um OBJ de terceiro com winding
//! misto são normais neste módulo; um raio que descartasse faces de costas
//! atravessaria exatamente essas superfícies e o pincel cairia no vazio. O
//! acerto que interessa é **o mais próximo**, e ele é o mais próximo com ou sem
//! orientação.

use crate::face::Face;
use crate::mesh::Mesh;

/// Abaixo disto o raio é paralelo ao plano do triângulo (ou o triângulo é
/// degenerado) e não há interseção a computar. É comparação com `|det|`, não com
/// `det`: o sinal do determinante é a ORIENTAÇÃO, e descartá-lo por sinal seria
/// o culling que este módulo recusa.
const PARALLEL_EPS: f32 = 1e-12;

/// Um raio no mundo. A direção é **normalizada na construção**, e é isso que
/// torna o `t` uma distância em unidades de mundo — a mesma régua com que o
/// octree poda e com que o chamador decide o raio do pincel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray {
    origin: [f32; 3],
    dir: [f32; 3],
    inv_dir: [f32; 3],
}

impl Ray {
    /// Constrói normalizando `dir`.
    ///
    /// Direção de comprimento zero (ou não-finita) produz um raio **inválido**,
    /// que [`Mesh::raycast`] recusa de saída. Recusar é mais honesto que
    /// escolher um eixo qualquer: um pick que devolve a geometria de uma direção
    /// que ninguém pediu é pior que um pick que não devolve nada.
    #[must_use]
    pub fn new(origin: [f32; 3], dir: [f32; 3]) -> Self {
        let len2 = dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2];
        let len = len2.sqrt();
        let d = if len > 0.0 && len.is_finite() {
            [dir[0] / len, dir[1] / len, dir[2] / len]
        } else {
            [0.0; 3]
        };
        Self {
            origin,
            dir: d,
            inv_dir: [1.0 / d[0], 1.0 / d[1], 1.0 / d[2]],
        }
    }

    #[must_use]
    pub fn origin(&self) -> [f32; 3] {
        self.origin
    }

    /// A direção, unitária.
    #[must_use]
    pub fn dir(&self) -> [f32; 3] {
        self.dir
    }

    /// O recíproco por eixo, pronto para o teste de *slab* das caixas.
    #[must_use]
    pub fn inv_dir(&self) -> [f32; 3] {
        self.inv_dir
    }

    /// O ponto a `t` unidades de mundo da origem.
    #[must_use]
    pub fn at(&self, t: f32) -> [f32; 3] {
        [
            self.origin[0] + self.dir[0] * t,
            self.origin[1] + self.dir[1] * t,
            self.origin[2] + self.dir[2] * t,
        ]
    }

    #[must_use]
    fn is_valid(&self) -> bool {
        self.dir != [0.0; 3]
    }
}

/// Onde o raio encontrou a superfície.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hit {
    /// A face acertada.
    pub face: u32,
    /// Distância ao longo do raio — na régua do ESPAÇO em que o raio foi dado.
    ///
    /// ⚠️ Desde a W8.1 esse espaço pode não ser o mundo: quem consulta uma malha
    /// com pose leva o raio ao espaço LOCAL dela (`Pose::ray_to_local`), e o
    /// `Ray` normaliza a direção na construção — então este `t` mede em unidades
    /// locais. Comparar dois acertos de objetos com escalas diferentes por `t`
    /// dá a resposta errada; compare os PONTOS, convertidos de volta.
    pub t: f32,
    /// O ponto acertado, no espaço em que o raio foi dado (ver [`Self::t`]).
    pub point: [f32; 3],
    /// A normal **geométrica** da face (não a interpolada dos vértices): é a
    /// direção da superfície que de fato foi tocada, que é o que um pincel e um
    /// cursor querem. A suave é `Mesh::normals()`, e quem a quiser interpola.
    ///
    /// ⚠️ **NÃO é garantidamente unitária, e o vão tem nome.** Um TRIÂNGULO
    /// degenerado nunca chega aqui — o `PARALLEL_EPS` do Möller–Trumbore o
    /// recusa antes. Mas este campo é a normal da **FACE**, e um quad "gravata"
    /// (diagonais paralelas) tem Newell **exatamente zero** com os dois
    /// triângulos de área cheia: o raio acerta, e a normal sai `[0, 0, 0]`.
    ///
    /// Fica assim de propósito: hoje **nenhum consumidor de produto lê este
    /// campo** (só o gate `a_bowtie_quad_is_hit_and_its_face_normal_is_the_named_gap`,
    /// que o pina), e resolvê-lo obrigaria o laço quente do [`Mesh::raycast`] a
    /// carregar QUAL triângulo venceu — estado novo no caminho de pick por causa
    /// de um caso que ninguém lê. **O gatilho que acorda o conserto é o primeiro
    /// leitor de produto**, e aí a cura é cair na normal do triângulo acertado.
    ///
    /// ⚠️ A afirmação anterior deste doc — *"está PROVADO que a face degenerada
    /// não chega aqui"* — era verdadeira por TRIÂNGULO e falsa por FACE, e a
    /// diferença é exatamente o quad.
    pub normal: [f32; 3],
}

/// A folga das barycêntricas — **a MESMA de [`crate::tri_geom`]**, e pela mesma
/// razão de tipo: `f32::EPSILON` é `1,19e-7` e as barycêntricas são O(1), então
/// `1e-6` são umas oito casas de ruído.
///
/// ⚠️ **Ela nasceu aqui em 2026-08-16, e o que a trouxe foi a REFUTAÇÃO da
/// justificativa que o irmão escrevia.** O cabeçalho do `tri_geom.rs` defendia a
/// estreiteza deste teste assim:
///
/// > *"um falso positivo na aresta partilhada elege o triângulo vizinho, e o
/// > artista não distingue"*
///
/// A frase raciocina sobre os DOIS triângulos aceitarem — e **nunca sobre
/// NENHUM aceitar**. Um teste estrito numa aresta exata recusa dos dois lados, e
/// o resultado não é uma escolha ambígua: é um **BURACO**. E o buraco não é
/// hipotético nesta malha — [`Mesh::raycast`] parte cada QUAD em `(0,1,2)` e
/// `(0,2,3)`, que partilham a diagonal e a testam com **ordens de vértice
/// diferentes**, então cada uma das 98.304 faces da esfera de fábrica carrega
/// uma aresta interna por onde vazar.
///
/// ⚠️ **MEDIDO antes de mudar, porque a magnitude decide se vale o risco:** um
/// leque de 4096 raios apontados ao centro **não vazava** (0 misses nas duas
/// esferas), e o que vazava era a ESTABILIDADE — perturbar a direção em **um
/// ULP** trocava acerto por erro em `1 de 6144` empurrões. É pouco, e é o
/// bastante: um pick que muda de resposta no último bit da direção é um pick que
/// pisca sob a mão do artista.
///
/// ⚠️ **A folga não pode fabricar um acerto onde não há superfície**, e é a
/// geometria que garante isso: `1e-6` de barycêntrica sobre um triângulo de
/// aresta `e` alcança `~e·1e-6` de mundo — quatro ordens abaixo de um texel de
/// qualquer malha que este módulo esculpe. Ela alarga a aceitação **para dentro
/// da vizinhança do próprio triângulo**, nunca para o vazio.
const BARY_SLACK: f32 = 1e-6;

/// Möller–Trumbore. Devolve a distância ao longo de `dir` (unitária), ou `None`.
///
/// ⚠️ **Tolerante nas bordas, como o irmão [`crate::TriEdges::ray_hit`]** — ver
/// [`BARY_SLACK`] para a medição que juntou os dois. `pub(crate)` só para o gate
/// que compara o par: sem ele, um deles pode mudar de resposta sem ninguém
/// notar.
///
/// ⚠️ **O `t` NÃO ganha folga.** As barycêntricas dizem *o raio passa por dentro
/// do triângulo?* e é ali que a aresta partilhada mora; o `t` diz *a que
/// distância, e para que lado?* — afrouxá-lo aceitaria superfície **ATRÁS** da
/// origem do raio, que é o cursor a agarrar o outro lado do modelo.
pub(crate) fn ray_triangle(
    origin: [f32; 3],
    dir: [f32; 3],
    a: [f32; 3],
    b: [f32; 3],
    c: [f32; 3],
) -> Option<f32> {
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let p = cross(dir, e2);
    let det = dot(e1, p);
    if det.abs() < PARALLEL_EPS {
        return None;
    }
    let inv_det = 1.0 / det;
    let tv = [origin[0] - a[0], origin[1] - a[1], origin[2] - a[2]];
    let u = dot(tv, p) * inv_det;
    if !(-BARY_SLACK..=1.0 + BARY_SLACK).contains(&u) {
        return None;
    }
    let q = cross(tv, e1);
    let v = dot(dir, q) * inv_det;
    if v < -BARY_SLACK || u + v > 1.0 + BARY_SLACK {
        return None;
    }
    let t = dot(e2, q) * inv_det;
    // Atrás do olho não é acerto. Sem isto, girar a câmera para o outro lado
    // acenderia o pincel na geometria que está às costas.
    if t < 0.0 { None } else { Some(t) }
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

impl Mesh {
    /// O acerto mais próximo do raio, ou `None`.
    ///
    /// A varredura é **front-to-back com poda** (ver
    /// [`crate::Octree::ray_visit_leaves`]): o melhor `t` já achado corta os nós
    /// atrás dele, então o custo é o da superfície visível e não o da malha.
    ///
    /// Um quad é testado como os **mesmos dois triângulos** que
    /// [`Face::triangles`] entrega à GPU (diagonal `a-c`). Se as duas
    /// triangulações divergissem, o pick acertaria a um lado de um vinco e o
    /// desenho mostraria o outro — o tipo de discordância que aparece como *"o
    /// pincel erra por um triângulo"* e não tem sintoma nenhum nos testes de
    /// cada metade.
    #[must_use]
    pub fn raycast(&self, ray: &Ray) -> Option<Hit> {
        if !ray.is_valid() || self.face_count() == 0 {
            return None;
        }
        let (origin, dir) = (ray.origin(), ray.dir());
        let mut best_t = f32::INFINITY;
        let mut best_face = u32::MAX;
        self.octree()
            .ray_visit_leaves(origin, ray.inv_dir(), |faces| {
                for &fi in faces {
                    let f: Face = self.faces()[fi as usize];
                    let vs = f.verts();
                    let p = |k: usize| self.positions()[vs[k] as usize];
                    let mut hit = ray_triangle(origin, dir, p(0), p(1), p(2));
                    if vs.len() == 4 {
                        let second = ray_triangle(origin, dir, p(0), p(2), p(3));
                        hit = match (hit, second) {
                            (Some(a), Some(b)) => Some(a.min(b)),
                            (a, b) => a.or(b),
                        };
                    }
                    if let Some(t) = hit
                        && t < best_t
                    {
                        best_t = t;
                        best_face = fi;
                    }
                }
                best_t
            });
        if best_face == u32::MAX {
            return None;
        }
        Some(Hit {
            face: best_face,
            t: best_t,
            point: ray.at(best_t),
            normal: self.face_normals()[best_face as usize],
        })
    }
}

#[cfg(test)]
#[path = "ray_tests.rs"]
mod tests;
