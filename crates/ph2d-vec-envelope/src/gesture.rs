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

use crate::{CageEdges, QuadWarp, cage_folds, rest_edges};

/// Quantas alças a gaiola oferece no gesto **Mesh**: 4 cantos + 2 controles por lado.
///
/// **Um espaço de índices só** (`0..4` = cantos, `4 + 2·lado + j` = controle) e não dois enums: o
/// arrasto vivo carrega `(entidade, índice)` e um índice que precisasse de tag seria uma segunda
/// pergunta ("canto ou controle?") feita em cada sítio que toca o arrasto.
pub const MESH_HANDLE_COUNT: usize = 12;

/// O índice da alça do controle `j` (0 ou 1) do lado `side` (0..4).
#[must_use]
pub fn edge_handle_index(side: usize, j: usize) -> usize {
    4 + 2 * side + j
}

/// A gaiola depois de um movimento aceito — os cantos E os controles, porque mover um canto no Mesh
/// **leva os controles vizinhos junto** (as alças pertencem ao canto, como em qualquer Bézier).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CageMove {
    pub corners: [[f64; 2]; 4],
    pub edges: CageEdges,
}

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

/// A alça sob o cursor, no espaço de índices único (`0..4` cantos · `4..12` controles de lado).
///
/// `edges` presente = gesto **Mesh** (os 12 candidatos); `None` = gesto **Perspective** (só os 4
/// cantos — em Perspective os lados são retos por invariante, e oferecer um controle que o mapa
/// ignora seria alça morta). **Cantos primeiro:** num empate de distância o canto vence, porque ele é
/// a alça que existe nos dois gestos e a que o artista mira.
#[must_use]
pub fn nearest_handle(
    corners: &[[f64; 2]; 4],
    edges: Option<&CageEdges>,
    p: [f64; 2],
    radius: f64,
) -> Option<usize> {
    if let Some(c) = nearest_corner(corners, p, radius) {
        return Some(c);
    }
    let edges = edges?;
    let r2 = radius * radius;
    let mut best: Option<(usize, f64)> = None;
    for (side, pair) in edges.iter().enumerate() {
        for (j, c) in pair.iter().enumerate() {
            let d2 = (c[0] - p[0]).powi(2) + (c[1] - p[1]).powi(2);
            if d2 <= r2 && best.is_none_or(|(_, bd)| d2 < bd) {
                best = Some((edge_handle_index(side, j), d2));
            }
        }
    }
    best.map(|(i, _)| i)
}

/// Move a alça `idx` para `to`, devolvendo a gaiola nova — ou `None` se o movimento for **recusado**
/// (o host então preserva a gaiola atual e a alça *para na fronteira*, como no arrasto de canto).
///
/// # Dois gestos, dois guards — e a diferença é epistemológica
///
/// **Perspective** (`mesh == false`): só os cantos, e o guard é a **convexidade**, que tem um teorema
/// atrás (ADR-0129 §5). Os lados são re-emitidos RETOS por [`rest_edges`] — em Perspective os lados
/// *são* retos, então o que fica guardado não pode dizer outra coisa; sem isso, trocar para Mesh
/// depois mostraria controles pendurados onde a gaiola já não está.
///
/// **Mesh** (`mesh == true`): as 12 alças, e o guard é *"o patch dobra?"* ([`cage_folds`]), que é
/// **amostrado** — não há critério fechado para um patch de Coons. Mover um canto **arrasta os 2
/// controles vizinhos pela mesma translação**: eles pertencem ao canto (é o que uma alça de Bézier
/// faz), e deixá-los para trás esticaria os lados de um jeito que ninguém pediu.
#[must_use]
pub fn move_handle(
    corners: [[f64; 2]; 4],
    edges: CageEdges,
    mesh: bool,
    idx: usize,
    to: [f64; 2],
) -> Option<CageMove> {
    if !mesh {
        let corners = move_corner_convex(corners, idx, to)?;
        return Some(CageMove {
            edges: rest_edges(&corners),
            corners,
        });
    }
    let (corners, edges) = if idx < 4 {
        let d = [to[0] - corners[idx][0], to[1] - corners[idx][1]];
        let mut corners = corners;
        corners[idx] = to;
        let mut edges = edges;
        // O canto `idx` começa o lado `idx` e termina o lado `idx+3`: os dois controles que o tocam.
        edges[idx][0] = [edges[idx][0][0] + d[0], edges[idx][0][1] + d[1]];
        let prev = (idx + 3) % 4;
        edges[prev][1] = [edges[prev][1][0] + d[0], edges[prev][1][1] + d[1]];
        (corners, edges)
    } else if idx < MESH_HANDLE_COUNT {
        let mut edges = edges;
        edges[(idx - 4) / 2][(idx - 4) % 2] = to;
        (corners, edges)
    } else {
        return None;
    };
    (!cage_folds(&corners, &edges)).then_some(CageMove { corners, edges })
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
        assert!(
            move_handle(
                rect(),
                rest_edges(&rect()),
                true,
                MESH_HANDLE_COUNT,
                [1.0, 1.0]
            )
            .is_none(),
            "alça além das 12 devia ser recusada, não indexar fora"
        );
    }

    /// **EM PERSPECTIVE OS LADOS SÃO RETOS — E O QUE FICA GUARDADO DIZ ISSO.** Depois de arrastar um
    /// canto, os controles são exatamente os canônicos da gaiola NOVA.
    ///
    /// Não é higiene: sem re-emitir, os controles ficariam pendurados na gaiola VELHA, e trocar para
    /// Mesh depois mostraria alças fora dos lados — o *"funciona e depois esquece"* de chapéu novo.
    #[test]
    fn a_perspective_move_keeps_the_sides_straight() {
        let m = move_handle(rect(), rest_edges(&rect()), false, 2, [8.0, 7.0])
            .expect("movimento convexo devia ser aceito");
        assert_eq!(m.corners[2], [8.0, 7.0]);
        assert_eq!(
            m.edges,
            rest_edges(&m.corners),
            "os lados guardados não são os retos da gaiola nova"
        );
    }

    /// **No Mesh, o canto leva os DOIS controles que o tocam — e só eles.** Uma alça de Bézier
    /// pertence à sua âncora; deixá-la para trás esticaria o lado sozinho.
    #[test]
    fn a_mesh_corner_carries_its_two_handles() {
        let before = rest_edges(&rect());
        let m = move_handle(rect(), before, true, 0, [-1.0, -1.0]).expect("aceito");
        let d = [-1.0, -1.0];
        assert_eq!(
            m.edges[0][0],
            [before[0][0][0] + d[0], before[0][0][1] + d[1]]
        );
        assert_eq!(
            m.edges[3][1],
            [before[3][1][0] + d[0], before[3][1][1] + d[1]]
        );
        // Os outros 6 controles não se mexem.
        assert_eq!(m.edges[0][1], before[0][1]);
        assert_eq!(m.edges[3][0], before[3][0]);
        assert_eq!(m.edges[1], before[1]);
        assert_eq!(m.edges[2], before[2]);
    }

    /// **Um movimento que DOBRARIA o patch é recusado** — e o vizinho que não dobra é aceito, senão
    /// o guard poderia ser "recuse sempre" e o gate ficaria verde.
    #[test]
    fn a_mesh_move_that_folds_is_refused() {
        let e = rest_edges(&rect());
        assert!(
            move_handle(rect(), e, true, edge_handle_index(0, 0), [3.0, 30.0]).is_none(),
            "empurrar o lado de baixo para muito acima do de cima devia dobrar e ser recusado"
        );
        assert!(
            move_handle(rect(), e, true, edge_handle_index(0, 0), [3.0, -2.0]).is_some(),
            "uma barriga moderada devia ser aceita"
        );
    }

    /// **Os controles de lado só existem no Mesh** (ausência) — e o MESMO cursor os acha quando eles
    /// são oferecidos (presença). Sem o segundo ramo, o primeiro ficaria verde num hit-test que nunca
    /// acha nada.
    #[test]
    fn the_side_handles_exist_only_in_mesh() {
        let c = rect();
        let e = rest_edges(&c);
        let on_control = e[0][0];
        assert_eq!(
            nearest_handle(&c, None, on_control, 0.5),
            None,
            "Perspective ofereceu um controle de lado"
        );
        assert_eq!(
            nearest_handle(&c, Some(&e), on_control, 0.5),
            Some(edge_handle_index(0, 0)),
            "Mesh não achou o controle sob o cursor"
        );
    }

    /// **O canto vence o controle num empate** — ele é a alça que existe nos dois gestos, e é a que
    /// o artista mira quando as duas caem sob o mesmo pixel.
    #[test]
    fn a_corner_wins_over_a_side_handle() {
        let c = rect();
        let e = rest_edges(&c);
        // Raio grande o bastante para alcançar o canto 0 E o controle vizinho.
        assert_eq!(nearest_handle(&c, Some(&e), [1.0, 0.0], 5.0), Some(0));
    }
}
