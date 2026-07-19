//! **A pergunta que decide se a cor vira geometria do PRÓPRIO traço** — módulo irmão do
//! `flip_fill.rs` (que estava no teto de LOC do shell, 600).
//!
//! Separado por RESPONSABILIDADE, não por tamanho: aqui mora um critério só — *"a região
//! que o balde traçou É o interior deste traço?"* — e ele tem história própria
//! (BUGS #16 → #17 → **#19**), que é o que o doc-comment abaixo carrega.

use crate::flip_fill::{ring_area, ring_contains};
use ph2d_core::Vec2;
use ph2d_flip::FlipDrawing;

/// **A região preenchida É o interior de um traço fechado?** Se for, o índice dele.
///
/// É a pergunta que muda tudo (smoke do Enio 2026-07-13, com o Suzanne ao lado):
///
/// > *"nem todo vertex da linha está conectado ao vertex de fill… isso cria áreas de
/// > dessincronização e gaps"*
///
/// Exato — e a causa é que o contorno do balde sai do **raster** (marching squares +
/// RDP), então os vértices dele não têm relação com os da linha: nas quinas ele chanfra,
/// nas retas ele desliza, e como o erro é assado em unidades de DOCUMENTO enquanto a
/// linha é absoluta em px de TELA, **o zoom o amplia**.
///
/// A cura não é aproximar melhor: é **não vetorizar**. Quando a região é o interior de
/// uma forma fechada, o preenchimento é o **fill do próprio traço** (a triangulação dos
/// pontos DELE — exatamente o que o Grease Pencil faz num material `stroke + fill`, que
/// é como o Suzanne é desenhado). Aí não há dois conjuntos de vértices para
/// dessincronizar: **há um só**. Esculpir a linha move a cor junto, de graça, para
/// sempre, em qualquer zoom.
///
/// **`closed` NÃO entra no critério** — e é aqui que o 1º corte morreu no produto (smoke
/// do Enio 2026-07-13). Um traço desenhado À MÃO é `closed = false`: o `flip_draw::build_stroke`
/// só fecha o traço no modo `Shape: Filled`, e a mão que encosta a ponta no começo **não**
/// muda esse bit. Exigir `closed` fazia o auto-preenchimento **nunca disparar** fora daquele
/// modo — todo fill do usuário caía no contorno vetorizado, que é justamente o que
/// dessincroniza. O polígono de um traço aberto fecha **implicitamente** (é o que o GP faz:
/// a triangulação dos pontos da curva não pergunta se ela é cíclica), e é assim que o
/// `ring_contains`/`ring_area` já o leem.
///
/// O critério é conservador — os três têm de valer:
/// 1. o traço é **line-art** (não uma região) e tem polígono (≥ 3 pontos);
/// 2. o **clique** cai dentro dele;
/// 3. a área do contorno que o solver traçou **bate** com a do traço (≤ `AREA_TOL`) — é
///    isso que separa "preencheu a forma" de "preencheu um pedaço entre ela e outra".
///
/// Quando não bate (uma região delimitada por VÁRIOS traços — colorir entre linhas), não
/// existe "a curva" para carregar a cor, e o contorno vetorizado é o caminho — que é o
/// que o balde do GP também faz.
pub(crate) fn filled_shape_target(
    drawing: &FlipDrawing,
    outer: &[Vec2],
    click: Vec2,
    hug_tol: f32,
) -> Option<usize> {
    /// Pré-filtro barato de área (fração). **Não é o critério** — ver `hug_tol`.
    const AREA_TOL: f32 = 0.15; // LITERAL-PX-OK: fracao, nao metrica de design
    let target_area = ring_area(outer).abs();
    if target_area <= 0.0 {
        return None;
    }
    drawing
        .strokes
        .iter()
        .enumerate()
        .filter(|(_, s)| !s.hide_stroke && s.len() >= 3)
        .filter(|(_, s)| ring_contains(s.positions(), click))
        .find(|(_, s)| {
            let a = ring_area(s.positions()).abs();
            (a - target_area).abs() <= AREA_TOL * a.max(target_area)
                // **O critério de verdade: as duas curvas se ABRAÇAM.**
                //
                // E os DOIS sentidos, porque cada um sozinho deixa passar um caso
                // real: só "o traço está sobre o contorno" aceita um traço que
                // acompanha um PEDAÇO da fronteira (a região delimitada por vários
                // traços — e aí a cor sairia com a forma do traço errado); só "o
                // contorno está sobre o traço" aceita um traço com pontas e voltas
                // sobrando (a gota que se cruza).
                && max_dist_to_ring(s.positions(), outer) <= hug_tol
                && max_dist_to_ring(outer, s.positions()) <= hug_tol
        })
        .map(|(i, _)| i)
}

/// A maior distância de um ponto de `pts` ao anel `ring` (segmentos, fecho implícito).
///
/// `pub(crate)` porque a R1 da wave da região-por-curvas faz a MESMA pergunta — *"estas
/// duas descrições são da mesma fronteira?"* — entre o anel do raster e o do arranjo. Uma
/// 2ª cópia derivaria, e o critério de abraço já custou caro uma vez (BUGS #19).
pub(crate) fn max_dist_to_ring(pts: &[Vec2], ring: &[Vec2]) -> f32 {
    let n = ring.len();
    if n < 2 {
        return f32::INFINITY;
    }
    pts.iter()
        .map(|p| {
            (0..n)
                .map(|i| {
                    let (a, b) = (ring[i], ring[(i + 1) % n]);
                    let ab = Vec2::new(b.x - a.x, b.y - a.y);
                    let l2 = ab.x * ab.x + ab.y * ab.y;
                    let t = if l2 <= 0.0 {
                        0.0
                    } else {
                        (((p.x - a.x) * ab.x + (p.y - a.y) * ab.y) / l2).clamp(0.0, 1.0)
                    };
                    let (dx, dy) = (p.x - (a.x + t * ab.x), p.y - (a.y + t * ab.y));
                    dx * dx + dy * dy
                })
                .fold(f32::INFINITY, f32::min)
        })
        .fold(0.0f32, f32::max)
        .sqrt()
}

#[cfg(test)]
#[path = "flip_fill_target_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "flip_fill_curve_route_tests.rs"]
mod curve_route_tests;

/// **A fronteira feita das CURVAS**, quando ela serve — a fatia R1 da wave
/// `docs/Flip/10_regiao_por_curvas.md`.
///
/// Devolve o anel do arranjo planar (vértices das próprias linhas + as interseções) se, e
/// só se, ele descreve **a mesma região** que o raster escolheu. Senão, `None`, e o
/// chamador fica com o contorno vetorizado de sempre.
///
/// # As três recusas, e por que cada uma
///
/// 0. **O usuário mexeu no GROW.** Ele é, por definição, um deslocamento a partir do
///    eixo (BUGS #13/#14) — e esta rota põe a fronteira **exatamente** no eixo. Não há
///    como honrar os dois. Recusar devolve o caso ao caminho que sabe deslocar; **aceitar
///    seria ignorar um slider em silêncio**, que é o defeito que a memória
///    `feedback_a_parameter_that_changes_nothing_is_discarded_downstream` descreve — e
///    era o que estava acontecendo aqui até um gate perguntar (o `style` de teste já vinha
///    com `grow: 2.0` e a rota disparava assim mesmo, jogando o valor fora).
/// 1. **As duas rotas discordam sobre os BURACOS.** Esta recusa já foi *"a região tem
///    buracos"* — o motor da R0 só devolvia o anel externo, e entregá-lo sem eles pintaria
///    por cima. Desde 2026-07-18 ele os produz (`arrange::holes_of`: um buraco é uma
///    **componente conexa** que cai dentro da face, e a caminhada de half-edge nunca as
///    enxerga porque nunca atravessa entre componentes). A pergunta certa deixou de ser
///    *se* há buraco e passou a ser se as duas rotas **concordam** — em quantos, e em
///    onde. Discordar em silêncio pintaria por cima de uma forma que devia ficar de fora.
/// 2. **Os dois anéis não se ABRAÇAM.** O arranjo acha a face que contém o clique; o
///    raster acha a região que o flood alcançou. Elas quase sempre coincidem — mas se a
///    tinta rala deixou o flood escapar por onde o arranjo vê parede (ou o contrário), as
///    duas respostas são de regiões DIFERENTES, e pintar a do arranjo seria pintar o que o
///    usuário não pediu.
/// 3. **O anel é degenerado** (< 3 pontos ou área ~zero).
///
/// ⚠️ A comparação é nos **DOIS sentidos**, pela lição do BUGS #19: um sentido só aceita
/// um anel que acompanha só um PEDAÇO do outro.
pub(crate) fn curve_region(
    strokes: &[(Vec<Vec2>, Vec<f32>, bool)],
    solved: &ph2d_flip_fill::FillResult,
    click: Vec2,
    hug_tol: f32,
    grow: i32,
) -> Option<(Vec<Vec2>, Vec<Vec<Vec2>>)> {
    if grow != 0 {
        return None; // recusa 0
    }
    // Os fechamentos de gap viram linhas de largura zero: eles delimitam, mas não são tinta
    // (é o mesmo papel que já têm no documento).
    let mut lines: Vec<(Vec<Vec2>, Vec<f32>, bool)> = strokes.to_vec();
    for c in &solved.closures {
        lines.push((vec![c.a, c.b], vec![0.0, 0.0], false));
    }
    let region = ph2d_flip_fill::region_at(&lines, click)?;
    if region.outer.len() < 3 || ring_area(&region.outer).abs() <= 0.0 {
        return None; // recusa 3
    }
    // ⚠️ **Os BURACOS entram no abraço, não só o anel externo.** A recusa 1 (*"tem buraco?
    // então o raster que resolva"*) existia porque o arranjo não os produzia; agora produz
    // (`arrange::holes_of`), e a pergunta certa não é *se* há buraco, é se as duas rotas
    // **concordam sobre eles**. Sem isto, uma região onde o raster vê um buraco e o
    // arranjo vê outro (ou nenhum) passaria no abraço do anel externo e a cor cobriria uma
    // forma que devia ficar de fora — em silêncio, que é a pior forma.
    if region.holes.len() != solved.holes.len() {
        return None; // recusa 1: as duas rotas discordam sobre QUANTOS buracos há
    }
    let hug = max_dist_to_ring(&region.outer, &solved.outer)
        .max(max_dist_to_ring(&solved.outer, &region.outer));
    if hug > hug_tol {
        return None; // recusa 2
    }
    // Cada buraco do raster tem de achar um par no arranjo (a ORDEM das duas listas não é
    // a mesma: uma sai do marching squares, a outra do percurso de faces).
    for sh in &solved.holes {
        let near = region
            .holes
            .iter()
            .map(|rh| max_dist_to_ring(sh, rh).max(max_dist_to_ring(rh, sh)))
            .fold(f32::MAX, f32::min);
        if near > hug_tol {
            return None; // recusa 2b: um buraco do raster não tem correspondente
        }
    }
    Some((region.outer, region.holes))
}
