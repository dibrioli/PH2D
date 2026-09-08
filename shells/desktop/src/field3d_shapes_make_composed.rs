//! ⭐⭐⭐ **AS FORMAS COMPOSTAS** (W138) — as que o catálogo entrega como uma **árvore**, e não como
//! uma primitiva.
//!
//! # ⛔ A fila fechou com ZERO variantes novas, e foi a medição que o decidiu
//!
//! O [plano](../../../docs/3DModeling/09_plano_das_dez_que_faltam.md) pedia duas primitivas — a
//! *Death Star* e o *Vesica Segment* —, com a justificação de que *«a nossa subtracção não dá a
//! distância exacta na cratera»*. O `CLAUDE.md` §5.0 manda medir isso **antes** de construir, e a
//! medição (`ph2d-field-eval/tests/probe_composed_shapes.rs`) devolveu:
//!
//! | peça | `‖∇f‖` | pior campo/verdade | passos de 1,0 | de 4,0 |
//! |---|---:|---:|---:|---:|
//! | esfera (controlo) | 1,000 | 1,0000 | 6 | 6 |
//! | esfera com cratera | 1,000 | **0,5495** | 5 | 9 |
//! | lente | 1,000 | **0,9674** | 10 | 8 |
//!
//! ⇒ o campo composto é **exactamente 1-Lipschitz** nas duas (a marcha nunca atravessa), a lente é
//! exacta a menos da amostragem do oráculo, e o que a fórmula fechada compraria na cratera são
//! `9` passos contra `6`. *Uma primitiva que custa uma variante, um degrau de formato e uma linha
//! na tabela de dimensões para comprar três passos de marcha numa região pequena não se escreve.*
//!
//! # ⚠️ O que a receita entrega e uma primitiva não entregaria
//!
//! A cratera continua a **ser uma esfera na Hierarquia**: move-se, redimensiona-se, duplica-se, e
//! podem pôr-se três. Uma `DeathStar { ra, rb, d }` congelaria a peça em três números.

use ph2d_field::{Blend, Op, Primitive};

use super::Recipe;

/// ⭐⭐ **A ESFERA COM CRATERA** — a bola menos outra bola encostada a ela.
///
/// ⚠️ **A geometria tem de deixar a cratera VISÍVEL, e isso é uma desigualdade**, não um gosto: o
/// círculo de corte só existe com `|d − rb| < ra < d + rb`. Com `rb = 0,75·r` e `d = 1,10·r` fica
/// `0,35 < 1 < 1,85` — folgado dos dois lados, e a cratera abre a pouco menos de metade do raio.
/// *Uma forma nova nasce no sítio em que ela é ELA* (doc 06 §128): com a segunda esfera longe
/// demais isto é uma esfera, e com ela por cima do centro é uma casca.
///
/// ⚠️ **A junta é `Sharp` e é escolhida, não herdada.** O verbo de omissão do módulo é
/// [`Blend::Exact`], e uma cratera de borda arredondada não é a forma que o nome promete — quem
/// quiser arredondá-la sobe o raio da junta no nó, que continua lá.
pub(crate) fn a_cratered_sphere(r: f32) -> Recipe {
    Recipe {
        op: Op::Difference(Blend::Sharp),
        parts: vec![
            (Primitive::Sphere { radius: r }, [0.0, 0.0, 0.0]),
            (Primitive::Sphere { radius: r * 0.75 }, [r * 1.10, 0.0, 0.0]),
        ],
    }
}

/// ⭐⭐ **A LENTE** — o sólido de revolução que duas esferas iguais partilham.
///
/// ⚠️ **Os centros afastam-se de exactamente um raio**, que é a *vesica piscis* canónica: a meia
/// altura fica `√3/2 = 0,866·r` e o bico é uma aresta viva de `120°` (as normais das duas esferas
/// no aro fazem `60°` entre si). Afastá-los mais afina a lente até ela desaparecer; menos, e ela
/// engorda até ser uma esfera com dois cortes.
///
/// ⚠️ **NÃO é a [`super::a_vesica`]**, e a diferença é a razão de as duas existirem: aquela é uma
/// **chapa** (a lente 2D puxada em Z, com `half_height`), esta é o sólido de revolução. O rótulo
/// separa-as pela primeira palavra de propósito — ver `panel.model3d.add.lens`.
pub(crate) fn a_lens(r: f32) -> Recipe {
    Recipe {
        op: Op::Intersection(Blend::Sharp),
        parts: vec![
            (Primitive::Sphere { radius: r }, [-r * 0.5, 0.0, 0.0]),
            (Primitive::Sphere { radius: r }, [r * 0.5, 0.0, 0.0]),
        ],
    }
}
