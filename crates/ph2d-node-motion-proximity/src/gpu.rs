//! O kernel de `motion.proximity` — a mesma grade de vizinhança do `motion.collide`
//! (ADR-0140 Fase 5), com **UM** dispatch: nada se move, então não há varredura a repetir
//! (`GridSpec::sweeps_param: None`) e nem a grade a reconstruir.
//!
//! ⚠️ **A ordem de travessia da grade NÃO é observável — mas isso não torna a paridade
//! bit-exacta, e a medição corrigiu esta prosa.** As duas operações do kernel são `max` (exacto
//! em qualquer ordem) e uma CONTAGEM (inteira), então nada aqui depende de quem foi visitado
//! primeiro, ao contrário do `motion.collide`, que **soma** correcções. O que diverge é o
//! VALOR que alimenta o `max`: o `dot(d, d)` do WGSL pode ser **FUNDIDO num `fma`** pelo
//! compilador de shader — mais exacto que os dois arredondamentos da CPU, e por isso diferente.
//! Medido com fixture bit-exacta: **contagem exacta, fracção a um ULP** (`5,96e-8`).
//! ⇒ `neighbours` é uma DECISÃO e é comparada por igualdade; `overlap` é um valor aritmético e
//! carrega o ε de toda a engine (`gpu_proximity::OVERLAP_EPS`).
//!
//! ⚠️ **O alcance é DERIVADO e depois CULLADO por disco**, o verbatim do collide: a célula
//! é `radius` mas o contacto acontece a `r_i + r_j`, então o alcance é
//! `ceil((r_i + r_max)/radius)` células — e o `r_max` vem do `Max` reduce, sem o qual um
//! vizinho GRANDE a duas células de distância seria perdido em silêncio. Cada célula é
//! então perguntada se o seu ponto MAIS PRÓXIMO está sequer em alcance antes de um único
//! disco dela ser tocado: exacto, nunca aproximado.

use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel, GridSpec};
use ph2d_nodegraph::port::Dim;
use ph2d_nodegraph::reduce_meta::{ReduceOp, ReduceSpec};

/// `P` e `size` são LIDAS (o nó não move nada); `neighbours` e `overlap` são escritas
/// — `Write`, materializadas na saída, exactamente como o `eval` as põe no `Stream`.
static BINDINGS: &[ColumnBinding] = &[
    ColumnBinding {
        column: "P",
        dim: Dim::Vec2,
        access: ColumnAccess::Read,
        identity: [0.0; 4],
        port: 0,
    },
    ColumnBinding {
        // A escala por elemento — a coluna que o RENDERER desenha. Ausente ⇒ `[1, 1]`,
        // que é o próprio fallback do `radius_scale`, então uma nuvem sem `size` mede
        // exactamente o que sempre mediria.
        column: "size",
        dim: Dim::Vec2,
        access: ColumnAccess::Read,
        identity: [1.0, 1.0, 0.0, 0.0],
        port: 0,
    },
    ColumnBinding {
        column: "neighbours",
        dim: Dim::Scalar,
        access: ColumnAccess::Write,
        identity: [0.0; 4],
        port: 0,
    },
    ColumnBinding {
        column: "overlap",
        dim: Dim::Scalar,
        access: ColumnAccess::Write,
        identity: [0.0; 4],
        port: 0,
    },
];

/// **O maior disco do conjunto**, que a varredura precisa para limitar o alcance: `i` só
/// pode tocar `j` dentro de `r_i + r_j <= r_i + r_max`.
///
/// ⚠️ `Max` é **bit-exacto em qualquer ordem** (o próprio doc do `ReduceOp`: associativo
/// *e* exacto sobre floats), então a redução do device e o `fold(max)` da CPU concordam
/// por matemática — o que é o que permite ao gate de paridade afirmar IGUALDADE sobre um
/// conjunto de tamanhos mistos.
///
/// ⚠️ A expressão do `value` carrega o saneador INLINE porque o passe de map da redução é
/// o seu próprio módulo gerado e não alcança o [`WGSL_LIB`]. Uma componente NaN faz toda
/// comparação ser falsa, então ela cai em `1.0` — a mesma resposta do `radius_scale`.
pub static REDUCES: &[ReduceSpec] = &[ReduceSpec {
    name: "rmax",
    column: "size",
    dim: Dim::Vec2,
    port: 0,
    identity: [1.0, 1.0, 0.0, 0.0],
    op: ReduceOp::Max,
    value: "select(1.0, max(abs(v.x), abs(v.y)), abs(v.x) < 3.4028235e38 && abs(v.y) < 3.4028235e38)",
    params: &[],
}];

/// O `EPS` e o `radius_scale` da CPU, à letra.
const WGSL_LIB: &str = r#"
const PROX_EPS: f32 = 1e-9;

// O verbatim do `crate::radius_scale`: o disco que CONTEM a arte, sem sinal, e
// nao-finito lendo como a identidade `1` -- ou seja, como se a coluna estivesse ausente.
fn prox_scale(v: vec2<f32>) -> f32 {
    if (abs(v.x) < 3.4028235e38 && abs(v.y) < 3.4028235e38) {
        return max(abs(v.x), abs(v.y));
    }
    return 1.0;
}
"#;

/// Um dispatch, gathered: cada thread conta os SEUS vizinhos e guarda o pior.
///
/// ⚠️ Os acessores são `read_P`/`read_size` e **não** `read_in_P` — o `accessor_suffix` do
/// codegen só prefixa o nome da porta quando o nó tem MAIS DE UMA, e este tem só a `in`.
/// (O irmão `motion.collide` tem duas — `in` e `spread` — e por isso escreve `read_in_P`.)
const WGSL: &str = r#"
    let pi = read_P(i);
    let base = params.radius;
    let ri = base * prox_scale(read_size(i));
    // O maior disco do conjunto, do `Max` reduce.
    let r_max = base * reduce_rmax();
    // A identidade da CPU, a letra: sem raio nenhum (ou menos de dois discos) nenhum par
    // pode ter `min_dist > 0`, entao ninguem toca ninguem.
    if (r_max <= 0.0 || params.count < 2u) {
        write_neighbours(i, 0.0);
        write_overlap(i, 0.0);
        return;
    }
    var count = 0u;
    var worst = 0.0;
    // O mais longe que ESTE disco pode tocar alguem: o proprio raio mais o maior do
    // conjunto. Com tamanhos uniformes isto e' exatamente `2 * radius`.
    let reach_d = ri + r_max;
    let reach_d2 = reach_d * reach_d;
    let reach = max(1, i32(ceil(reach_d / max(params.radius, 1e-20))));
    let cell = params.grid_cell;
    let ci = grid_cell_of(pi);
    for (var dy = -reach; dy <= reach; dy = dy + 1) {
        for (var dx = -reach; dx <= reach; dx = dx + 1) {
            let c = ci + vec2<i32>(dx, dy);
            // `reach` e' o caso PIOR -- um disco encostado a borda longe da propria
            // celula. O que ESTE disco precisa depende de onde dentro da celula ele
            // esta, entao pula-se a celula cujo ponto MAIS PROXIMO ja' esta' fora de
            // alcance, antes de tocar num unico disco dela. Exato, nunca aproximado.
            let lo = vec2<f32>(f32(c.x), f32(c.y)) * cell;
            let gap = max(max(lo - pi, pi - (lo + vec2<f32>(cell, cell))), vec2<f32>(0.0, 0.0));
            if (dot(gap, gap) >= reach_d2) { continue; }
            let b = grid_bucket_of(c);
            let hi = grid_starts[b + 1u];
            for (var s = grid_starts[b]; s < hi; s = s + 1u) {
                let j = grid_sorted[s];
                if (j == i) { continue; }
                let pj = read_P(j);
                // Dedup exato por celula: duas celulas podem cair no mesmo bucket, entao
                // conta-se `j` so' enquanto se visita a celula em que ele de facto mora.
                let cj = grid_cell_of(pj);
                if (cj.x != c.x || cj.y != c.y) { continue; }
                let min_dist = ri + base * prox_scale(read_size(j));
                if (min_dist <= 0.0) { continue; }
                let d = pj - pi;
                let d2 = dot(d, d);
                if (d2 >= min_dist * min_dist) { continue; }
                var frac = 1.0;
                if (d2 > PROX_EPS) { frac = 1.0 - sqrt(d2) / min_dist; }
                count = count + 1u;
                worst = max(worst, frac);
            }
        }
    }
    write_neighbours(i, f32(count));
    write_overlap(i, worst);
"#;

/// O kernel registado. `radius` é o único param: ele é ao mesmo tempo o raio do disco e a
/// célula da grade, e a grade lê-o pelo [`GRID`] — uma cópia no uniform seria uma segunda
/// resposta à mesma pergunta, mas aqui o corpo PRECISA do número, então é a MESMA entrada
/// lida por dois consumidores, não dois números.
pub const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: WGSL,
    wgsl_lib: WGSL_LIB,
    bindings: BINDINGS,
    params: &["radius"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// A grade de vizinhança: sobre `P` na porta 0, célula = `radius`, **sem varreduras** —
/// nada se move, então a grade construída uma vez continua a responder *"quem está perto
/// de ti?"* até ao fim do dispatch.
pub const GRID: GridSpec = GridSpec {
    column: "P",
    port: 0,
    cell_param: "radius",
    sweeps_param: None,
};
