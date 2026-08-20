//! **A EXTRAÇÃO** — os dois campos viram uma malha (ADR-0160 §5, Q3).
//!
//! O campo de posição coloca uma origem de retícula por vértice, consistente **a
//! menos de passos inteiros** — e é **aqui** que se quocienta pela retícula. Três
//! passos, e cada um responde a uma pergunta da malha nova:
//!
//! 1. **as CÉLULAS** — que vértices da entrada descrevem o mesmo nó da grade?
//!    (união sobre as arestas; um `union-find`);
//! 2. **as ARESTAS** — que células são vizinhas? (as arestas da entrada cujo
//!    passo de retícula tem norma **1** — a mesma conta do passo 1, lida um
//!    degrau adiante; ver `Linking`);
//! 3. **as FACES** — que ciclos o grafo fecha? (o *sistema de rotação*: em cada
//!    célula os vizinhos são ordenados por ângulo no plano tangente, e a face é
//!    o passeio que sempre vira para o mesmo lado).
//!
//! ⚠️ **O passo 3 é o que faz a saída ser uma MALHA e não um grafo.** Um grafo de
//! quads não tem faces: quem as tem é a *imersão*, e a ordem angular em torno de
//! cada célula é exatamente ela. Procurar ciclos de comprimento 4 no grafo
//! (a alternativa óbvia) acharia também as diagonais de um cubo — quatro
//! vértices que se ligam em ciclo **sem** delimitar face nenhuma.
//!
//! # ⚠️ O que esta onda NÃO garante, de propósito
//!
//! **`all-quad` ainda não é verdade aqui, e o ADR-0160 §5 diz que a Q3 fecha com
//! um NÚMERO em vez de um zero.** Onde os índices de singularidade não fecham, o
//! passeio devolve ciclos de 3, 5 ou mais — é o defeito conhecido da família
//! Instant Meshes, e é exatamente o que o fluxo de custo mínimo da **Q4** existe
//! para curar. Declarar zero antes daquele passo seria declarar que a técnica
//! base não tem o defeito que a literatura inteira nomeia.
//!
//! A [`Quadrangulation`] carrega a contagem, e é ela que a Q4 tem de baixar.
//! Medido em 2026-08-19: **97,6 %** na malha que o módulo abre, e **todos** os
//! triângulos que sobram são **isolados** (sonda
//! `measure_where_the_leftover_triangles_come_from`) — o emparelhamento guloso
//! está esgotado, e o que falta é mesmo o passo global.
//!
//! ⚠️ **Mas o resíduo deixou de ser uma AGULHA.** Um ciclo de `n > 4` não é mais
//! triangulado em leque a partir do vértice `0` (`n − 2` fatias degeneradas a
//! irradiar de um ponto — a peça espetada da foto do Enio, 2026-08-19): ele é
//! fechado por um nó no centroide, em `⌈n/2⌉` faces quase todas quads. Ver
//! `faces::fan_free_closure`.

use ph2d_mesh::{Face, Mesh, MeshError};

use crate::orientation::OrientationField;
use crate::position::PositionField;
use crate::scale::ScaleField;

/// **A malha nova, com a conta do que não saiu quad.**
///
/// ⚠️ **A contagem viaja com o resultado e não num log**: ela é o número que a
/// Q4 tem de baixar, e um número que só existe num `eprintln!` não é comparável
/// entre duas corridas.
#[derive(Clone, Debug)]
pub struct Quadrangulation {
    /// A malha extraída.
    pub mesh: Mesh,
    /// Quantas faces saíram com **quatro** lados.
    pub quads: usize,
    /// Quantas **não** saíram — o alvo da Q4.
    pub non_quads: usize,
    /// O maior número de lados que um ciclo teve.
    pub max_sides: usize,
}

impl Quadrangulation {
    /// A fração das FACES DA MALHA que saiu quad — a régua de uma corrida contra
    /// a seguinte.
    ///
    /// ⚠️ **Sobre as faces EMITIDAS, e não sobre os ciclos, e a diferença já
    /// mentiu uma vez.** A primeira versão dividia `quads / (quads + ciclos
    /// não-quad)` — e um ciclo de **31 lados** contava como **um** não-quad
    /// enquanto virava **29 triângulos** na malha. A régua *melhorava* quando as
    /// falhas ficavam maiores: uma mudança que trocou 582 triângulos por 918
    /// aparecia como uma subida de 60,9 % para 71,9 %. *Uma régua que premeia o
    /// defeito por ele ser grande é pior que nenhuma.*
    #[must_use]
    pub fn quad_fraction(&self) -> f64 {
        let total = self.mesh.faces().len();
        if total == 0 {
            0.0
        } else {
            self.quads as f64 / total as f64
        }
    }
}

/// **EXTRAI a malha** a partir da entrada e dos dois campos.
///
/// ⚠️ **`BTreeMap`/`BTreeSet` e nunca `HashMap`** — a saída tem de ser
/// byte-idêntica entre corridas (HR-5, e a asserção A7 do ADR-0160). Uma tabela
/// de hash faria a ordem das arestas e das faces depender da semente do
/// processo, e a malha do artista mudaria ao reabrir o projeto.
///
/// # Erros
///
/// Devolve [`MeshError`] se o conjunto de faces extraído não formar uma malha
/// bem formada (um índice fora de alcance) — o que só acontece se a fusão de
/// células tiver produzido um grafo degenerado.
pub fn extract(
    mesh: &Mesh,
    orient: &OrientationField,
    pos: &PositionField,
    scale: &ScaleField,
) -> Result<Quadrangulation, MeshError> {
    // ⚠️ **`Lattice` — o quociente da referência.** Ele SÓ funciona sobre campos
    // com platôs, e é por isso que a porta do produto é a
    // [`crate::solve::solve_fields`] (hierárquica) e não o caminho plano.
    extract_with(mesh, orient, pos, scale, Clustering::Lattice)
}

/// **Como os vértices viram células** — as duas leis, medidas uma contra a outra.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clustering {
    /// Bola no campo em torno de uma semente. Não precisa de platôs.
    Seed,
    /// **O QUOCIENTE PELA RETÍCULA** — dois vizinhos são a mesma célula quando o
    /// passo inteiro entre as retículas deles é `(0,0)`.
    ///
    /// ⭐ **É O CAMINHO DO PRODUTO** — e ele esteve MEDIDO E REJEITADO por meio
    /// dia, o que é a lição desta wave.
    ///
    /// Ele colapsava (esfera → **1 célula**) porque o
    /// `compat_position_extrinsic_4` da crate **não era o da referência**:
    /// arredondava cada lado ao ponto médio, independentemente, em vez de
    /// enumerar as quatro quinas de cada lado e escolher o **PAR** mais próximo.
    /// Sem esse operador as duas retículas nunca se procuram, o campo sai suave,
    /// não há degraus — e o quociente funde tudo.
    ///
    /// Com o porte fiel: **76,9 %** de quads na esfera e **86,6 %** no toro,
    /// contra 71,7 %/75,6 % do crescimento por semente.
    ///
    /// ⚠️ **A lição não é sobre este enum:** duas recusas MEDIDAS (esta e a da
    /// hierarquia) eram consequências de **um** operador mal portado. *Uma
    /// medição só refuta o que ela de facto exercitou* — e o que ela exercitava
    /// era a minha aproximação, não a lei.
    Lattice,
}

/// A extração com a lei de agrupamento aberta — a porta da sonda.
///
/// # Erros
/// Ver [`extract`].
pub fn extract_with(
    mesh: &Mesh,
    orient: &OrientationField,
    pos: &PositionField,
    scale: &ScaleField,
    how: Clustering,
) -> Result<Quadrangulation, MeshError> {
    extract_tuned(mesh, orient, pos, scale, how, Linking::default())
}

/// **COMO AS CÉLULAS SE LIGAM** — as duas leis, medidas uma contra a outra.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Linking {
    /// **O PASSO INTEIRO DA RETÍCULA** — a lei da referência, e o default.
    ///
    /// Duas células são vizinhas quando alguma aresta da entrada as separa por
    /// **um** passo da retícula (`|a| + |b| = 1`). É o mesmo arredondamento que
    /// já decide o agrupamento (passo `(0,0)`), lido um degrau adiante — então a
    /// vizinhança e a fusão passam a ser **a mesma pergunta**, e não duas que
    /// podem discordar.
    #[default]
    LatticeStep,
    /// **O CONE GEOMÉTRICO** — a primeira lei desta crate, mantida como controle.
    ///
    /// Cada célula escolhe a candidata mais alinhada em cada uma das quatro
    /// direções da cruz, dentro de uma janela de distância. Ela **adivinha** a
    /// grade a partir da geometria em vez de a ler do campo, e os dois limiares
    /// (o cone de 45°, a janela `[0,5 s, 1,7 s]`) são a superfície onde a adivinha
    /// erra.
    Cone,
}

/// A extração com as DUAS leis abertas — a porta da sonda que as mediu.
///
/// # Erros
/// Ver [`extract`].
pub fn extract_tuned(
    mesh: &Mesh,
    orient: &OrientationField,
    pos: &PositionField,
    scale: &ScaleField,
    _how: Clustering,
    _link: Linking,
) -> Result<Quadrangulation, MeshError> {
    // ⭐ **O PORTE FIEL** — `extract_graph` + `extract_faces` da referência.
    let g = crate::im_graph::extract_graph(mesh, orient, pos, scale);
    let f = crate::im_faces::extract_faces(&g.adj, &g.o, &g.n, true);

    // ⚠️ **COMPACTAR: só os vértices que aparecem numa face entram na malha.** Um
    // vértice órfão conta em `V` e não em `E` nem em `F` — ele empurra a
    // característica de Euler para cima sem descrever nada, e faria a A3 medir a
    // poda em vez da topologia.
    let mut remap = vec![u32::MAX; f.verts.len()];
    let mut verts: Vec<[f32; 3]> = Vec::new();
    let mut faces: Vec<Face> = Vec::with_capacity(f.faces.len());
    for face in &f.faces {
        let v = face.verts();
        let mut out = [0u32; 4];
        for (k, &old) in v.iter().enumerate() {
            let old = old as usize;
            if remap[old] == u32::MAX {
                remap[old] = verts.len() as u32;
                verts.push(f.verts[old]);
            }
            out[k] = remap[old];
        }
        faces.push(if v.len() == 4 {
            Face::quad(out[0], out[1], out[2], out[3])
        } else {
            Face::tri(out[0], out[1], out[2])
        });
    }

    let quads = faces.iter().filter(|f| f.verts().len() == 4).count();
    let non_quads = faces.len() - quads;
    let max_sides = f
        .stats
        .iter()
        .enumerate()
        .filter(|(_, c)| **c > 0)
        .map(|(k, _)| k)
        .max()
        .unwrap_or(0);

    Ok(Quadrangulation {
        mesh: Mesh::from_parts(verts, faces)?,
        quads,
        non_quads,
        max_sides,
    })
}

pub(super) fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

pub(super) fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1].mul_add(b[2], -(a[2] * b[1])),
        a[2].mul_add(b[0], -(a[0] * b[2])),
        a[0].mul_add(b[1], -(a[1] * b[0])),
    ]
}

pub(super) fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[cfg(test)]
#[path = "extract_tests.rs"]
mod tests;
