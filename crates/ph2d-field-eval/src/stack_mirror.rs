//! ⭐⭐⭐ **O ESPELHO** — a dobra num plano, e a quiralidade que um plano deslocado introduz.
//!
//! # Por que um módulo irmão
//!
//! Ele é o terceiro corte do [`crate::stack`] (depois da dobra e da inclinação), e pela mesma razão:
//! as duas leis daqui carregam a medição que as escolheu por extenso, e com elas o arquivo passava
//! dos **700** do gate de LOC da workspace. ⚠️ **A cura é partir para irmão, nunca uma entrada na
//! allowlist.**

use fidget::context::Tree;
use ph2d_field::Unary;

/// ⭐⭐⭐ **A DOBRA NUM PLANO** — a lei do espelho, com o plano onde o artista o pôs.
///
/// `t → |t − c| + c`: guarda a metade `t ≥ c` e reflecte-a. ⚠️ **Com `c = 0` devolve `|t|` AO BIT**
/// (o caminho de omissão de toda peça anterior à v17 do formato), e por isso o ramo é explícito em
/// vez de subtrair e somar zero: `x − 0 + 0` constrói dois nós de árvore que a marcha depois avalia.
///
/// ⚠️ **Continua uma distância exacta**: uma reflexão é uma isometria, e `|·|` sobre uma distância
/// assinada é a distância à mesma superfície vista dos dois lados — é o mesmo argumento que a casca
/// e a booleana já usam. Medido: o espelho **sozinho** dá `‖∇f‖ = 1,0000` nos três eixos e em quatro
/// posições do plano (dentro da peça, na face dela, e para lá dela).
pub(crate) fn dobra(t: Tree, plano: f32) -> Tree {
    if plano == 0.0 {
        return t.abs();
    }
    let c = Tree::constant(f64::from(plano));
    (t - c.clone()).abs() + c
}

/// ⭐⭐⭐ **ESTE MODIFICADOR TORNA A SECÇÃO QUIRAL?** — a metade do espelho na bandeira que decide
/// quantas fatias uma repetição avalia (2026-09-04).
///
/// # ⛔⛔ O defeito que ela fecha, alcançável em DOIS cliques
///
/// A repetição radial avalia **duas** fatias, e isso basta *«enquanto a forma é a mesma vista de
/// qualquer lado»* — a premissa está escrita no [`crate::stack`], para as torções. ⚠️ **Um espelho
/// no plano `0` AUMENTA a simetria e é inofensivo; um plano DESLOCADO empurra a matéria para um
/// lado só, e isso é exactamente a mesma quiralidade que uma torção produz.**
///
/// Medido com o plano de nascimento (na face da peça) e `Radial` a seguir:
///
/// | pilha | antes | depois |
/// |---|---:|---:|
/// | `[MirrorY, Radial]` | **`223,8962`** | `1,0000` |
/// | `[Radial, MirrorY]` · `[Mirror, Radial]` · `[MirrorZ, Radial]` | `1,0000` | `1,0000` |
/// | espelho **sozinho**, os três eixos, quatro posições do plano | `1,0000` | `1,0000` |
///
/// ⇒ a bandeira deixa de perguntar *«há um deformador?»* e passa a perguntar *«a secção ainda é a
/// mesma vista de qualquer lado?»*, que é o que ela sempre significou. O instrumento é a
/// `tests/probe_mirror_pairs.rs`, que responde em **8 s** contra os `380` do gate dos trios.
pub(crate) fn desloca_a_seccao(m: &Unary) -> bool {
    matches!(
        m,
        Unary::Mirror { offset } | Unary::MirrorY { offset } | Unary::MirrorZ { offset }
            if *offset != 0.0
    )
}
