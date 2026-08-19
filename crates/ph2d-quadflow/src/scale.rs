//! **A ESCALA — quão grande é um quad, e ONDE** (ADR-0160 §3-ii, asserção A6).
//!
//! É aqui que mora a metade *adaptativa* do pedido. Um remesh de escala única
//! devolve a mesma grade num nariz e numa barriga; um adaptativo põe **quads
//! menores onde a curvatura é alta** e maiores onde a forma é chapada — que é o
//! que o Houdini chama de *"more smaller quads in regions with many local
//! features"*.
//!
//! ⚠️ **A escala é um CAMPO por-vértice, e não um número, mesmo no modo
//! uniforme.** Uma porta única (`ScaleField`) que às vezes é constante custa um
//! `Vec` de `f32` e apaga o caso especial; duas portas (um `f32` e um campo)
//! seriam a pergunta *"qual das duas manda?"* respondida em cada consumidor.

use ph2d_mesh::Mesh;

/// **Quantas vezes o quad menor cabe no maior** — o teto da adaptação.
///
/// ⚠️ **É um limite de REPRESENTAÇÃO, não de conforto** (`CLAUDE.md` §0.0): a
/// extração liga células vizinhas da retícula, e duas células cujas escalas
/// diferem por mais do que isto deixam de ter aresta comum — a grade rasga em
/// vez de transitar. O número é a razão que a literatura de campo cruzado usa
/// para o *sizing field* graduado, e ele **tem gate**
/// (`the_adaptive_range_is_bounded`).
pub const MAX_ADAPTIVE_RATIO: f32 = 4.0;

/// **A escala de cada vértice** — o lado do quad que se quer ali, em unidades de
/// objeto.
#[derive(Clone, Debug, PartialEq)]
pub struct ScaleField {
    per_vertex: Vec<f32>,
}

impl ScaleField {
    /// O lado do quad pedido no vértice `v`.
    #[must_use]
    pub fn at(&self, v: usize) -> f32 {
        self.per_vertex[v]
    }

    /// Quantos vértices o campo cobre.
    #[must_use]
    pub fn len(&self) -> usize {
        self.per_vertex.len()
    }

    /// Um campo sem vértices.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.per_vertex.is_empty()
    }

    /// O par (menor, maior) — a régua da asserção A6.
    #[must_use]
    pub fn range(&self) -> (f32, f32) {
        self.per_vertex
            .iter()
            .fold((f32::MAX, f32::MIN), |(lo, hi), s| (lo.min(*s), hi.max(*s)))
    }

    /// **UNIFORME** — o mesmo lado em toda parte.
    ///
    /// ⚠️ **A razão do [`Self::range`] tem de sair `1,0` EXATO aqui**, e é o
    /// controle da A6: um modo "uniforme" que variasse um por cento seria um
    /// adaptativo fraco a fingir-se de uniforme, e nenhum gate de aparência veria.
    #[must_use]
    pub fn uniform(mesh: &Mesh, edge: f32) -> Self {
        Self {
            per_vertex: vec![edge.max(MIN_EDGE); mesh.vert_count()],
        }
    }

    /// **ADAPTATIVA** — o lado encolhe onde a curvatura aperta.
    ///
    /// A lei é a da literatura de *sizing field*: o lado do quad é proporcional
    /// ao **raio de curvatura** (`1/|κ|`), porque é o raio que diz quantos quads
    /// uma feição precisa para não sair facetada. `strength = 0` devolve o campo
    /// uniforme **ao bit**; `1` usa a faixa inteira.
    ///
    /// ⚠️ **A curvatura entra NORMALIZADA pelo percentil, não pelo máximo.** Um
    /// único vértice patológico (um pico de uma importação, um polo de uma
    /// esfera UV) tem curvatura ordens de grandeza acima do resto, e dividir pelo
    /// máximo esmagaria o modelo inteiro contra o piso — a adaptação inteira
    /// ficaria a servir um vértice. A mediana é a estatística que não se move com
    /// ele.
    ///
    /// ⚠️ **E o resultado é CLAMPADO pela [`MAX_ADAPTIVE_RATIO`]**: não é
    /// conforto, é o que impede a grade de rasgar entre células de escalas
    /// incompatíveis.
    #[must_use]
    pub fn adaptive(mesh: &Mesh, edge: f32, strength: f32) -> Self {
        let edge = edge.max(MIN_EDGE);
        let s = strength.clamp(0.0, 1.0);
        if s == 0.0 {
            // ⚠️ **Saída ANTECIPADA e não `mix(uniform, adaptive, 0)`:** o
            // caminho aritmético devolveria `edge * (1 - 0) + x * 0`, que em
            // `f32` é `edge` só quando `x` é finito. Uma curvatura `NaN` numa
            // malha importada envenenaria o modo uniforme por um caminho que
            // ninguém suspeitaria.
            return Self::uniform(mesh, edge);
        }

        let curv = mesh.curvatures();
        // A mediana do |κ| — a régua que um pico não move.
        let mut mags: Vec<f32> = curv.iter().map(|k| k.abs()).collect();
        mags.sort_by(f32::total_cmp);
        let median = mags.get(mags.len() / 2).copied().unwrap_or(0.0);

        let lo = edge / MAX_ADAPTIVE_RATIO.sqrt();
        let hi = edge * MAX_ADAPTIVE_RATIO.sqrt();
        let per_vertex = curv
            .iter()
            .map(|k| {
                // `r = 1/|κ|` normalizado pela mediana: 1 no vértice mediano,
                // menor onde aperta, maior onde a forma é chapada.
                let rel = if median > 1.0e-9 {
                    (median / k.abs().max(1.0e-9))
                        .clamp(1.0 / MAX_ADAPTIVE_RATIO.sqrt(), MAX_ADAPTIVE_RATIO.sqrt())
                } else {
                    1.0
                };
                // `strength` interpola entre o uniforme e a lei cheia.
                let f = s.mul_add(rel - 1.0, 1.0);
                (edge * f).clamp(lo, hi)
            })
            .collect();
        Self { per_vertex }
    }
}

/// O piso do lado de um quad, em unidades de objeto.
///
/// ⚠️ **Guarda de RECURSO e não de gosto:** o inverso da escala multiplica cada
/// coordenada na retícula do campo de posição, e um zero ali é uma divisão por
/// zero que sai como `inf` no meio de um campo — envenenando a suavização
/// inteira na varredura seguinte, sem erro nenhum.
pub const MIN_EDGE: f32 = 1.0e-6;

#[cfg(test)]
#[path = "scale_tests.rs"]
mod tests;
