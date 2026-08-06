//! **O AO ASSADO** — quanto do céu cada vértice enxerga.
//!
//! `docs/3D/05.1` §3 fixa as duas metades do desenho: *cone tracing contra o
//! campo SDF, guardado por vértice*, e **assar contra o CAMPO, não contra a
//! malha** — ray-casting contra triângulos quer um BVH, contra um campo é
//! **marcha de esfera**, e a maquinaria do remesh já tem o campo.
//!
//! ⚠️ **O doc diz que isso é "de graça", e essa frase é sobre a ESTRUTURA, não
//! sobre o relógio.** De graça quer dizer *não precisa de um BVH novo*; o que um
//! bake custa é a sonda `measure_ao` que decide, e o campo sozinho já mede
//! 231-386 ms na malha que a cena `=16` abre.
//!
//! # A lei
//!
//! Para cada vértice, cones distribuídos por **cosseno** no hemisfério da
//! normal. Cada cone marcha contra o campo e devolve a fração dele que ficou
//! livre; o AO é a média.
//!
//! ```text
//! vis(cone) = clamp( min sobre a marcha de  k · d(p + t·dir) / t ,  0, 1)
//! ao(v)     = média dos vis
//! ```
//!
//! `d/t` é o seno do meio-ângulo que o espaço livre subtende àquela distância —
//! é o **teste de cone** do soft shadow de Inigo Quilez ([*Soft shadows in
//! raymarched SDFs*](https://iquilezles.org/articles/rmshadows/)), e `k` é o
//! inverso da tangente do meio-ângulo do nosso cone.
//!
//! **AO aqui é VISIBILIDADE:** `1` é céu aberto, `0` é enterrado. É a convenção
//! de multiplicador — o shader multiplica o ambiente por ele — e é por isso que
//! o nome não é "oclusão".
//!
//! # As três constantes que pareceriam mágicas, e de onde cada uma sai
//!
//! O `CLAUDE.md` §0 recusa fator solto. Nenhuma destas é escolhida:
//!
//! - **A abertura do cone** é função de **quantos cones** você atira: eles têm
//!   de ladrilhar o hemisfério, então `2π(1 − cos θ) = 2π/cones`, ou seja
//!   `cos θ = 1 − 1/cones`. Pedir mais cones os afina sozinho.
//! - **O viés** (onde a marcha começa) é **um passo de voxel**. Abaixo da
//!   resolução do campo não há o que resolver, então começar mais perto é ler
//!   ruído de quantização como sombra.
//! - **O piso do passo da marcha** é o mesmo passo de voxel, pela mesma razão:
//!   uma marcha de esfera que avança menos que uma célula não aprendeu nada e
//!   só gasta amostra.
//!
//! # O viés que sobra, e por que ele ENCOLHE com a qualidade
//!
//! Num plano infinito a resposta verdadeira é `1` em toda parte. A nossa dá
//! menos, e dá para dizer quanto: a origem sai do vértice, então a distância ao
//! plano ao longo de um cone a `θ` da normal é `t·cos θ`, e `k·d/t = k·cos θ`.
//! O cone só é cortado quando `cos θ < 1/k`, isto é `θ` maior que o meio-ângulo
//! — os cones **rasantes**, que a distribuição por cosseno já pesa pouco.
//!
//! ⚠️ **E a fração deles cai quando os cones sobem** (`k` cresce com a contagem),
//! então este viés é o raro que *melhora com a qualidade em vez de trocar de
//! forma*. O número medido está no gate `um_convexo_isolado_enxerga_o_ceu`.

use ph2d_mesh::Aabb;
use rayon::prelude::*;

use crate::VoxelField;

/// Quanto do céu um vértice enxerga, e com que orçamento perguntar.
#[derive(Clone, Copy, Debug)]
pub struct AoParams {
    /// Quantos cones por vértice. Manda na QUALIDADE e na abertura (ver módulo).
    pub cones: usize,
    /// Teto de amostras por cone. É um teto de custo, não de alcance: a marcha
    /// de esfera costuma escapar bem antes dele em espaço aberto.
    pub max_steps: usize,
    /// Até onde procurar um oclusor, em unidades de mundo.
    pub radius: f32,
}

impl AoParams {
    /// Os defaults, com o alcance **semeado pelo MODELO**.
    ///
    /// ⚠️ **É a lição da W10.2 no eixo do alcance:** uma escala absoluta é a
    /// unidade certa e um literal absoluto não é — o mesmo `0,5` é o corpo
    /// inteiro numa peça pequena e um poro numa grande. O alcance nasce como
    /// fração do maior lado da caixa, que é a régua que o artista de fato vê.
    ///
    /// ⚠️ **A fração é uma SEMENTE de LOOK, não um teto de recurso**, e o smoke
    /// é quem a decide: grande o bastante para atravessar a fresta entre dois
    /// membros, pequena o bastante para o AO não virar uma medida da sala.
    #[must_use]
    pub fn for_bounds(bounds: Aabb) -> Self {
        let ext = [
            bounds.max[0] - bounds.min[0],
            bounds.max[1] - bounds.min[1],
            bounds.max[2] - bounds.min[2],
        ];
        let longest = ext[0].max(ext[1]).max(ext[2]).max(f32::MIN_POSITIVE);
        Self {
            cones: 32,
            max_steps: 24,
            radius: longest * 0.125,
        }
    }

    /// `1/tan(meio-ângulo)`, com o meio-ângulo derivado da contagem de cones —
    /// ver o módulo.
    #[must_use]
    fn cone_k(self) -> f32 {
        let n = self.cones.max(1) as f32;
        let cos = 1.0 - 1.0 / n;
        // `tan = sin/cos`, e `sin = sqrt(1 - cos²)`: sem transcendental, e sem
        // o `atan` que só existiria para ser desfeito na linha seguinte.
        let sin = (1.0 - cos * cos).max(f32::MIN_POSITIVE).sqrt();
        cos / sin
    }
}

/// As direções de cone, distribuídas por **cosseno** no hemisfério `+Z`.
///
/// Rede de Fibonacci esférica: `r² = (i + ½)/n` uniforme dá `sin²θ` uniforme,
/// que é exatamente a densidade `cos θ · sin θ dθ` da amostragem por
/// importância de cosseno. ⚠️ **É isso que torna a MÉDIA simples o integral
/// certo** — o `cos θ` da definição de AO já está na escolha das direções, e
/// pesar de novo na soma o contaria duas vezes.
///
/// Roda **uma vez por bake** (`cones` entradas), não por vértice: é onde os
/// dois transcendentais do módulo vivem, e eles não alcançam o laço quente.
#[must_use]
fn cone_directions(n: usize) -> Vec<[f32; 3]> {
    // O ângulo áureo, a constante da rede de Fibonacci.
    let ga = core::f32::consts::PI * (3.0 - 5.0f32.sqrt());
    (0..n)
        .map(|i| {
            let r2 = (i as f32 + 0.5) / n as f32;
            let r = r2.sqrt();
            let phi = ga * i as f32;
            [r * phi.cos(), r * phi.sin(), (1.0 - r2).max(0.0).sqrt()]
        })
        .collect()
}

/// Uma base ortonormal com `n` no terceiro eixo.
///
/// Port de **Duff et al. 2017, *Building an Orthonormal Basis, Revisited***
/// (JCGT 6.1). ⚠️ **O `copysign` é a coisa toda:** a construção ingênua divide
/// por `1 + n.z` e explode no polo `n = (0,0,−1)`; escolher o sinal faz o
/// denominador ser `−2` justamente lá. Sem ramo, sem caso degenerado, e é por
/// isso que vale portar em vez de escrever a versão com `if`.
#[must_use]
fn basis(n: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let s = if n[2] >= 0.0 { 1.0f32 } else { -1.0 };
    let a = -1.0 / (s + n[2]);
    let b = n[0] * n[1] * a;
    (
        [1.0 + s * n[0] * n[0] * a, s * b, -s * n[0]],
        [b, s + n[1] * n[1] * a, -n[1]],
    )
}

/// A fração de um cone que ficou livre — ver a lei no módulo.
#[must_use]
fn cone_visibility(
    field: &VoxelField,
    p: [f32; 3],
    dir: [f32; 3],
    radius: f32,
    k: f32,
    max_steps: usize,
) -> f32 {
    let step = field.step();
    let mut vis = 1.0f32;
    let mut t = step;
    for _ in 0..max_steps {
        let q = [p[0] + dir[0] * t, p[1] + dir[1] * t, p[2] + dir[2] * t];
        let d = field.sample(q);
        let v = k * d / t;
        if v < vis {
            vis = v;
        }
        if vis <= 0.0 {
            // Enterrado: nenhum passo adiante pode reabrir o cone, porque o
            // mínimo já é o mínimo.
            return 0.0;
        }
        // A marcha de esfera: avança pelo raio da maior bola vazia conhecida,
        // com piso na resolução do campo (ver o módulo).
        t += d.max(step);
        if t > radius {
            break;
        }
    }
    vis.clamp(0.0, 1.0)
}

/// Quanto do céu o vértice `p` com normal `n` enxerga, em `[0, 1]`.
///
/// ⚠️ **Privada de propósito.** A porta componível é o próprio [`bake_ao`] sobre
/// uma FATIA — é o que a sonda de paralelismo usa, e é o que um `rayon` faria
/// por baixo. Um `pub` aqui seria uma segunda resposta esperando quem a chame,
/// e esta linha já pagou essa lição.
#[must_use]
fn ao_at(
    field: &VoxelField,
    dirs: &[[f32; 3]],
    k: f32,
    params: AoParams,
    p: [f32; 3],
    n: [f32; 3],
) -> f32 {
    if dirs.is_empty() {
        // Sem cone não há pergunta feita; `1` é *"nada foi observado ocluindo"*,
        // que é a leitura honesta e a que não escurece a peça por acidente.
        return 1.0;
    }
    let (t1, t2) = basis(n);
    let mut sum = 0.0f32;
    for d in dirs {
        let world = [
            t1[0] * d[0] + t2[0] * d[1] + n[0] * d[2],
            t1[1] * d[0] + t2[1] * d[1] + n[1] * d[2],
            t1[2] * d[0] + t2[2] * d[1] + n[2] * d[2],
        ];
        sum += cone_visibility(field, p, world, params.radius, k, params.max_steps);
    }
    sum / dirs.len() as f32
}

/// Assa o AO de todos os vértices.
///
/// **Paralelo por [ADR-0156](../../../docs/architecture/decisions/0156-sculpt3d-ao-trace-is-a-per-vertex-gather-rayon-exception.md).**
/// O laço é um **gather**: cada vértice escreve só o seu, contra um campo
/// **imutável**, e a soma sobre os cones é **privada do vértice e de ordem
/// fixa** — então a saída é byte-idêntica ao serial. Isso não é argumentado, é
/// **medido**: 2, 4, 8, 16 e 32 threads dão os mesmos bytes (18,49× a 32), e o
/// gate `o_bake_paralelo_e_byte_identico_ao_serial` compara contra a rota
/// serial congelada.
///
/// ⚠️ **A exceção é ESTREITA.** O voxelizador e o flood fill continuam seriais
/// pelo mecanismo que o `Cargo.toml` desta crate escreve: as caixas de dois
/// triângulos se sobrepõem ⇒ a escrita não é disjunta. Paralelizar aquela
/// metade exige ADR próprio.
///
/// ⚠️ **Sem piso de pool, e é uma decisão declarada:** o custo por vértice aqui
/// é ~1 850 ns (contra dezenas nas normais), então o `PAR_MIN` da `ph2d-mesh`
/// não é a régua certa — e o piso honesto sai de uma varredura, não de um
/// palpite. Até ela existir, o pior caso é uma malha minúscula pagar o overhead
/// do pool, que é seguro.
#[must_use]
pub fn bake_ao(
    field: &VoxelField,
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    params: AoParams,
) -> Vec<f32> {
    let dirs = cone_directions(params.cones);
    let k = params.cone_k();
    // O `min` preserva a semântica do `zip` da rota serial: entradas de
    // comprimentos diferentes produzem o menor dos dois, nunca um pânico.
    let n = positions.len().min(normals.len());
    let mut out = vec![0.0f32; n];
    out.par_iter_mut().enumerate().for_each(|(v, o)| {
        *o = ao_at(field, &dirs, k, params, positions[v], normals[v]);
    });
    out
}

/// A rota **SERIAL, CONGELADA** — o oráculo do gate de identidade.
///
/// ⚠️ É o código que shipava antes do ADR-0156, verbatim. Ele existe para que a
/// byte-identidade seja uma **comparação**, não um argumento sobre invariantes;
/// e vive sob `cfg(test)` porque um `pub` sem chamador de produto seria uma
/// **segunda resposta** esperando quem a chame — a lição que esta casa já pagou
/// com o `warp_axis` e o `serial_side`.
#[cfg(test)]
#[must_use]
fn bake_ao_serial(
    field: &VoxelField,
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    params: AoParams,
) -> Vec<f32> {
    let dirs = cone_directions(params.cones);
    let k = params.cone_k();
    positions
        .iter()
        .zip(normals)
        .map(|(&p, &n)| ao_at(field, &dirs, k, params, p, n))
        .collect()
}

#[cfg(test)]
#[path = "ao_tests.rs"]
mod ao_tests;
