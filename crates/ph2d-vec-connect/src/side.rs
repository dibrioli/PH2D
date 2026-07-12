//! **Por qual lado a linha sai** — a âncora flutuante.
//!
//! Uma âncora flutuante não tem lado fixo: ela **re-escolhe** o lado a cada frame, em função
//! de onde a outra forma está. Arraste a caixa para cima e a linha passa a sair pelo topo,
//! sozinha. É o comportamento que faz o diagrama parecer vivo.
//!
//! Duas sutilezas, e as duas são o que separa uma implementação ingênua de uma boa.

use crate::Dir;

/// A **banda morta** da histerese, como fração. Ver [`side_towards`].
const HYSTERESIS: f64 = 0.05;

/// O lado por onde a linha deve sair de uma caixa de semi-eixos `(hw, hh)` para alcançar um
/// alvo que está no deslocamento `d` (do centro da caixa ao centro do alvo).
///
/// # 1. O critério é o QUADRANTE DA DIAGONAL, não o ângulo
///
/// A comparação ingênua é `|dx| > |dy|` ⇒ sai pelo lado. Isso está errado para uma caixa que
/// não é quadrada: numa caixa **bem larga**, um alvo logo acima do canto tem `|dx| > |dy|` e
/// sairia pela lateral — mas o olho espera que ele saia pelo **topo**, porque o topo é a face
/// que "olha" para lá. O critério certo normaliza pelo semi-eixo: compara `|dx|/hw` contra
/// `|dy|/hh`, ou seja, testa de que lado da **diagonal da própria caixa** o alvo está.
///
/// # 2. A histerese, que ninguém documenta
///
/// Quando o alvo passa **exatamente** pela diagonal, os dois lados empatam — e o menor tremor
/// no arrasto faz a linha **piscar** entre sair pelo topo e sair pela lateral, quadro sim,
/// quadro não. É feio e é tonto.
///
/// `prev` (o lado escolhido no frame anterior) resolve: só se troca de lado quando o novo
/// vence por uma margem. Dentro da banda morta, **fica onde estava**. É o mesmo princípio de
/// um termostato — e a razão de ele existir aqui é exatamente a mesma.
#[must_use]
pub fn side_towards(d: [f64; 2], hw: f64, hh: f64, prev: Option<Dir>) -> Dir {
    let (hw, hh) = (hw.max(1e-9), hh.max(1e-9));
    // Normalizado pelo semi-eixo: é o teste da diagonal da caixa.
    let (ax, ay) = ((d[0] / hw).abs(), (d[1] / hh).abs());

    let horizontal = if let Some(p) = prev {
        // Já havia um lado. Só troca de eixo se o novo vencer com FOLGA — dentro da banda
        // morta, mantém. (Sem isto a linha pisca na diagonal.)
        let was_h = matches!(p, Dir::East | Dir::West);
        if was_h {
            ax >= ay * (1.0 - HYSTERESIS)
        } else {
            ax > ay * (1.0 + HYSTERESIS)
        }
    } else {
        ax >= ay
    };

    if horizontal {
        if d[0] >= 0.0 { Dir::East } else { Dir::West }
    } else if d[1] >= 0.0 {
        Dir::North
    } else {
        Dir::South
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O caso óbvio: o alvo à direita, sai pela direita.
    #[test]
    fn the_line_leaves_by_the_face_that_looks_at_the_target() {
        assert_eq!(side_towards([10.0, 0.0], 2.0, 1.0, None), Dir::East);
        assert_eq!(side_towards([-10.0, 0.0], 2.0, 1.0, None), Dir::West);
        assert_eq!(side_towards([0.0, 10.0], 2.0, 1.0, None), Dir::North);
        assert_eq!(side_towards([0.0, -10.0], 2.0, 1.0, None), Dir::South);
    }

    /// **Uma caixa larga sai pelo TOPO quando o alvo está quase em cima** — mesmo com
    /// `|dx| > |dy|`.
    ///
    /// É o teste que quebra a comparação ingênua. Numa caixa de 10×1, um alvo em `(3, 2)` tem
    /// `|dx| = 3 > |dy| = 2`, então `|dx| > |dy|` mandaria sair pela LATERAL. Mas a caixa é
    /// larguíssima: o alvo está muito acima da diagonal dela, e o olho espera o topo.
    #[test]
    fn a_wide_box_exits_through_the_top_when_the_target_is_nearly_above_it() {
        let (hw, hh) = (10.0, 1.0);
        assert_eq!(
            side_towards([3.0, 2.0], hw, hh, None),
            Dir::North,
            "numa caixa 10x1, um alvo em (3,2) esta MUITO acima da diagonal — sai pelo topo, \
             ainda que |dx| > |dy| (a comparacao ingenua erraria aqui)"
        );
        // E longe o bastante na horizontal, sai pela lateral, como deve.
        assert_eq!(side_towards([30.0, 2.0], hw, hh, None), Dir::East);
    }

    /// **A histerese mata o tremor.** Sobre a diagonal exata, os dois lados empatam — e sem
    /// memória a linha pisca entre eles a cada micro-movimento do mouse. Com `prev`, ela fica
    /// onde estava até o novo lado vencer com folga.
    #[test]
    fn the_exit_side_does_not_flicker_when_the_target_crosses_the_diagonal() {
        let (hw, hh) = (2.0, 2.0);
        // Exatamente na diagonal (|dx|/hw == |dy|/hh).
        let on_diag = [5.0, 5.0];
        // Sem memória, escolhe um (o horizontal, por convenção).
        assert_eq!(side_towards(on_diag, hw, hh, None), Dir::East);

        // Com memória de NORTE, um tremor na diagonal NÃO troca para leste.
        assert_eq!(
            side_towards(on_diag, hw, hh, Some(Dir::North)),
            Dir::North,
            "sobre a diagonal, com o lado anterior = Norte, a linha tem de FICAR no Norte — \
             senao ela pisca entre topo e lateral a cada quadro do arrasto"
        );
        // E com memória de LESTE, também fica.
        assert_eq!(side_towards(on_diag, hw, hh, Some(Dir::East)), Dir::East);

        // Mas quando o alvo se move DE VERDADE para o outro lado, ela troca.
        assert_eq!(
            side_towards([1.0, 9.0], hw, hh, Some(Dir::East)),
            Dir::North,
            "um alvo claramente acima TEM de mudar o lado — a histerese e uma banda morta, \
             nao uma trava"
        );
    }
}
