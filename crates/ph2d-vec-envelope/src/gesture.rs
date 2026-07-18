//! O gesto de **arrastar os cantos da gaiola** (ADR-0129, Fatia 1) — a parte PURA.
//!
//! O envelope já deforma uma forma pela gaiola de 4 cantos (`QuadWarp`); esta fatia torna os
//! cantos arrastáveis no modo Node. A geometria de "que canto o dedo pegou" e "para onde ele pode
//! ir" mora aqui, sem `sim`, sem câmera e sem Vello — o host (`shells/desktop/src/envelope_gesture`)
//! é um adaptador fino que lê/escreve o componente ECS e delega a decisão a estas duas funções.
//!
//! Duas regras, uma cada:
//!
//! 1. [`nearest_corner`] — qual dos 4 cantos está sob o cursor. É a MESMA função para o hit-test e
//!    (via o raio) para o desenho: um segundo critério faria o dedo pegar um canto e a bolinha
//!    acender noutro. (O raio em si é UI — vive no renderer, `ph2d_vec_render::ENVELOPE_HANDLE_R_PX`;
//!    aqui ele chega já em unidades de MUNDO.)
//!
//! 2. [`move_corner_convex`] — mover um canto **só se a gaiola continuar convexa**. É o §5 do ADR: a
//!    homografia de retângulo→quad estritamente convexo não põe a linha de fuga dentro da gaiola,
//!    então recusar o não-convexo torna o caso degenerado **inalcançável pelo gesto** — sem clipping,
//!    sem epsilon. O canto simplesmente **para na fronteira** enquanto o cursor está na zona proibida
//!    (o clamp que o LPE de perspectiva do Inkscape também faz), e volta a seguir quando o cursor
//!    reentra na região convexa. Recusar é `None`; o host mantém os cantos como estavam.

use crate::QuadWarp;

/// O índice do canto (0..4, ordem `[BL, BR, TR, TL]`) **mais próximo** de `p` dentro de `radius`, ou
/// `None` se nenhum está ao alcance. Empate → o menor índice (determinístico).
///
/// `radius` é distância em coordenadas de MUNDO (o host converte o raio em pixels do renderer pela
/// escala px→mundo, exatamente como o hit-test da alça de raio de quina faz).
#[must_use]
pub fn nearest_corner(corners: &[[f64; 2]; 4], p: [f64; 2], radius: f64) -> Option<usize> {
    let r2 = radius * radius;
    let mut best: Option<(usize, f64)> = None;
    for (i, c) in corners.iter().enumerate() {
        let d2 = (c[0] - p[0]).powi(2) + (c[1] - p[1]).powi(2);
        if d2 <= r2 && best.is_none_or(|(_, bd)| d2 < bd) {
            best = Some((i, d2));
        }
    }
    best.map(|(i, _)| i)
}

/// Move o canto `idx` para `to`, devolvendo os 4 cantos novos **apenas se a gaiola continuar
/// estritamente convexa** ([`QuadWarp::is_convex`]). `None` = o movimento tornaria a gaiola
/// não-convexa (ou `idx` fora de 0..4) — o host então preserva os cantos atuais.
///
/// Convexo, e não meramente não-degenerado, de propósito: é a convexidade que mantém o horizonte
/// fora da gaiola (ADR-0129 §5). Um quad reflexo (bowtie) reprovaria também — `is_convex` pega os
/// dois pela mudança de sinal das viradas consecutivas.
#[must_use]
pub fn move_corner_convex(
    mut corners: [[f64; 2]; 4],
    idx: usize,
    to: [f64; 2],
) -> Option<[[f64; 2]; 4]> {
    if idx >= 4 {
        return None;
    }
    corners[idx] = to;
    QuadWarp::is_convex(&corners).then_some(corners)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cantos de um retângulo `[BL, BR, TR, TL]`. Convexo em repouso.
    fn rect() -> [[f64; 2]; 4] {
        [[0.0, 0.0], [10.0, 0.0], [10.0, 6.0], [0.0, 6.0]]
    }

    /// **O canto sob o dedo é o mais próximo dentro do raio.** Um ponto quase em cima do TR pega o
    /// índice 2, e um ponto no meio da gaiola (longe de todos) não pega ninguém.
    #[test]
    fn nearest_corner_picks_the_one_under_the_cursor() {
        let c = rect();
        assert_eq!(nearest_corner(&c, [10.1, 6.1], 0.5), Some(2), "TR");
        assert_eq!(nearest_corner(&c, [0.05, -0.05], 0.5), Some(0), "BL");
        assert_eq!(
            nearest_corner(&c, [5.0, 3.0], 0.5),
            None,
            "o centro não está perto de canto nenhum"
        );
    }

    /// **O raio é uma cerca:** um canto logo além do raio não é pego, o mesmo canto logo aquém é.
    /// Sem isto, `nearest_corner` seria "o mais próximo, sempre" e o dedo pegaria um canto do outro
    /// lado da tela.
    #[test]
    fn the_radius_is_a_fence() {
        let c = rect();
        // BR está a 0.3 de distância do ponto; raio 0.2 recusa, 0.4 aceita.
        let p = [10.0, 0.3];
        assert_eq!(nearest_corner(&c, p, 0.2), None, "0.3 > 0.2: fora");
        assert_eq!(nearest_corner(&c, p, 0.4), Some(1), "0.3 < 0.4: dentro");
    }

    /// **Empate → o menor índice** (determinístico). Um ponto equidistante de dois cantos não pode
    /// escolher por acaso.
    #[test]
    fn a_tie_goes_to_the_lower_index() {
        // Quadrado; o centro do lado inferior é equidistante de BL(0) e BR(1).
        let c = [[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]];
        assert_eq!(nearest_corner(&c, [1.0, 0.0], 2.0), Some(0));
    }

    /// **Um empurrão pequeno mantém a convexidade — e é aceito.** É o caso comum do arrasto.
    #[test]
    fn a_small_push_stays_convex() {
        let c = rect();
        let moved = move_corner_convex(c, 2, [8.0, 7.0]).expect("ainda convexo");
        assert_eq!(moved[2], [8.0, 7.0], "o canto foi para onde o cursor pediu");
        assert!(QuadWarp::is_convex(&moved));
    }

    /// **Puxar um canto para DENTRO do triângulo dos outros três é recusado.** Isso é o reflexo
    /// (côncavo) que o §5 proíbe — o único jeito de a homografia meter o horizonte na gaiola.
    /// Recusar aqui é o que torna o caso degenerado inalcançável pelo gesto.
    #[test]
    fn pulling_a_corner_into_a_reflex_is_refused() {
        let c = rect();
        // TR (índice 2) puxado para o interior, perto do centro → quad reflexo.
        assert!(
            move_corner_convex(c, 2, [3.0, 2.0]).is_none(),
            "um quad côncavo passou pelo guard de convexidade"
        );
    }

    /// **Cruzar um canto para o lado oposto (bowtie) também é recusado.** A ordem do polígono é
    /// pressuposta; um canto que atravessa vira sinais mistos nas viradas, que `is_convex` reprova.
    #[test]
    fn a_bowtie_is_refused() {
        let c = rect();
        // BR (índice 1) levado para cima e à esquerda de TL → os lados se cruzam.
        assert!(move_corner_convex(c, 1, [-1.0, 7.0]).is_none());
    }

    /// **Índice fora de 0..4 é `None`, não pânico.** O host controla o índice, mas uma função pura
    /// não confia — e um `corners[idx]` cru entraria em pânico.
    #[test]
    fn an_out_of_range_index_is_none() {
        assert!(move_corner_convex(rect(), 4, [0.0, 0.0]).is_none());
    }
}
