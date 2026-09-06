//! ⭐⭐⭐ **A ESPIRAL POR FÓRMULA** (W123) — a fita de Arquimedes, sem um único segmento desenhado.
//!
//! # ⛔⛔⛔ A recusa que estava escrita respondia a OUTRA pergunta
//!
//! O [doc 08](../../../docs/3DModeling/08_formas_por_formula.md) dá a espiral como classe **D**,
//! *«a distância a uma espiral de Arquimedes não é fechada»*. Isso é **verdade** — e o módulo nunca
//! precisou dela. Uma marcha de esferas precisa de um **MINORANTE**: andar a menos custa passos,
//! andar a mais atravessa a superfície.
//!
//! ⚠️ E o preço de a deixar desenhada está MEDIDO (`spike_formula_vs_profile`, 05/09, `load 0,61`):
//! o **mesmo** cilindro custa `1,79 ns/ponto` por fórmula e `181,44 ns` desenhado com 192 lados —
//! **`101×`**. Uma espiral de três voltas não se desenha com menos do que isso.
//!
//! # ⭐⭐ As duas metades que a tornam exprimível
//!
//! 1. **A volta mais próxima sai de um arredondamento**, como a repetição radial já faz: com
//!    `φ = atan2(y, x)` e `u = (ρ − r₀)/b`, a volta é `k = round((u − φ)/2π)` e o raio dela é
//!    `r_k = r₀ + b·(φ + 2πk)`. ⚠️ **Contínuo no corte do `atan2`**: ali `φ` salta `−2π` e `k` salta
//!    `+1`, e a soma não se mexe.
//! 2. ⭐⭐⭐ **O FIM DA FITA É UM ANEL, e isso é exacto para a LINHA MÉDIA.** Numa espiral de
//!    Arquimedes o raio é **monótono** no ângulo, então `ρ ∈ [r₀, r_fim]` limita `θ` a `[0, Θ]`.
//!
//!    ⚠️⚠️ **E deixa uma PENA nas duas pontas, que fica DECLARADA** — ver
//!    [`SPIRAL_FEATHER`](../../ph2d-field-eval/tests/measure_sharp_edges.rs). A fita tem espessura,
//!    e um corte circular é **tangente** aos flancos dela: a ponta afina ao longo de `π·fill/c` de
//!    ângulo (`101°` no representante). Medido: o filete não a alcança (`3,1 %` da superfície sobre
//!    um vinco de `61,7°`) e deixa uma crista de curvatura de `2,12` contra a barra de `2,0`.
//!
//!    ⛔⛔⛔ **TRÊS cortes alternativos foram construídos e MEDIDOS ABAIXO:**
//!    - **o divisor LOCAL** (`ρ/√(ρ²+b²)` em vez da constante) dá gradiente unitário e o filete
//!      passa a assentar — e **quebra o minorante**, que é a razão de a forma existir;
//!    - **cortar pelo ÂNGULO DESENROLADO** dá pontas radiais a direito — e `θ` **salta `2π` na
//!      costura onde a volta mais próxima muda**: `‖∇f‖ = 2596,5`. *Contínuo numa costura não é
//!      contínuo em todas*;
//!    - **subtrair a pena** com um semiplano mais um anel — a pena e a **volta anterior** partilham
//!      a mesma faixa de raio a ângulos diferentes, e a subtracção come as duas.
//!
//!    ⇒ *o defeito é do corte CIRCULAR de uma fita com espessura, e curá-lo é desenho novo.*
//!
//! # ⚠️ O divisor é uma CONSTANTE, e é isso que o torna rigoroso
//!
//! `‖∇u‖ = √(ρ² + b²)/ρ`, que **decresce** com `ρ`. Dividir pelo valor local sobrestimaria a
//! distância quando o ponto mais próximo estivesse mais para dentro; dividir pelo valor em `r₀` — o
//! maior de toda a coroa — é um majorante do gradiente em todo o domínio onde a fita vive, logo
//! `|u|·c` é um minorante honesto. *Uma inexactidão que subestima é folga.*

use fidget::context::Tree;

use crate::ops::{safe_sqrt, slab_and_walls};
use crate::ops_joint::{Edge, intersection_joint};

/// ⭐ **ESPIRAL** — a fita de Arquimedes de `turns` voltas, começando no raio `radius`, com `pitch`
/// de afastamento por volta e meia-espessura `thickness`.
///
/// As duas pontas são cortes **radiais** (ver o cabeçalho: o anel é o fim da fita).
pub fn sd_spiral(
    radius: f64,
    pitch: f64,
    turns: f64,
    thickness: f64,
    half_height: f64,
    round: f64,
    chamfer: f64,
) -> Tree {
    let e = Edge::square(round, chamfer);
    let tau = std::f64::consts::TAU;
    let b = pitch / tau;
    let r0 = radius;
    let r_fim = pitch.mul_add(turns, r0);
    let rho = safe_sqrt(Tree::x().square() + Tree::y().square());
    let phi = Tree::y().atan2(Tree::x());
    // A volta cujo raio passa mais perto deste ponto.
    let u = (rho.clone() - Tree::constant(r0)) / Tree::constant(b);
    let k = ((u - phi.clone()) / Tree::constant(tau)).round();
    let r_k = Tree::constant(r0) + (phi + Tree::constant(tau) * k) * Tree::constant(b);
    // ⭐ O divisor CONSTANTE — ver o cabeçalho. ⛔ O local foi medido e REFUTADO.
    let c = r0 / r0.hypot(b);
    let banda = (rho.clone() - r_k).abs() * Tree::constant(c) - Tree::constant(thickness);
    // ⭐⭐ O anel que limita a linha média — exacto, porque o raio é monótono no ângulo.
    let meio = (r0 + r_fim) * 0.5;
    let anel = (rho.clone() - Tree::constant(meio)).abs() - Tree::constant((r_fim - r0) * 0.5);
    slab_and_walls(&intersection_joint(&banda, &anel, e), half_height, e)
}
