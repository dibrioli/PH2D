//! **AS FRONTEIRAS DA FOLHA** — o que fica dentro, como se sai, e o que está mal.
//!
//! Pedido do Enio (2026-08-19): *"Quando a imagem está numa sheet não permita que ela seja
//! arrastada para fora das margens na sheet"*, *"Se assim fizer automaticamente a sprite deve se
//! deslocar para dentro das fronteiras da sheet"* e *"Se houver sobreposição ou se as sprites não
//! couberem na sheet, sua moldura deve ficar vermelha e um aviso deve aparecer"*.
//!
//! ## Uma lei, três gestos
//!
//! Confinar é a mesma conta em três sítios que não se parecem: arrastar uma peça, largar um sprite
//! dentro da folha na hierarquia, e (no futuro) encolher a folha. Escrevê-la três vezes seria como
//! ela passa a discordar de si própria — a versão do arrasto ganharia a correção do arredondamento
//! e a do reparent não. Por isso [`confine`] é **um verbo**, e os dois chamadores só decidem
//! *quando*.
//!
//! ## O confinamento é POR GESTO, nunca por quadro
//!
//! ⚠️ Isto escreve `Transform`, e a lei do [ADR-0153] diz que *"o passe publica onde as coisas
//! ficam; ele não escreve onde elas estão"*. Ela proíbe um passe **por-quadro** de tocar poses —
//! senão cada quadro de um resize vira um passo de undo, e o histórico enche-se de ruído. Aqui é
//! sempre por-gesto: um `CursorMoved` dentro de um arrasto, ou um largar na hierarquia. A
//! distinção é *por-quadro vs. por-gesto*, e é a mesma que o `arrange_children` invoca.
//!
//! ## A saúde é DERIVADA, e por isso não pode mentir
//!
//! [`health`] não guarda nada: lê as peças e responde. Guardar um sinalizador «esta folha está
//! sobreposta» seria um segundo estado a manter em passo com o primeiro — e o dia em que alguém
//! movesse uma peça sem o atualizar, a moldura ficaria vermelha sobre uma folha correta (ou, pior,
//! verde sobre uma errada). *O que se pode contar, conta-se.*
//!
//! [ADR-0153]: ../../../docs/architecture/decisions/0153-vector-auto-layout-is-taffy-behind-one-leaf-crate-and-the-pose-is-derived.md

use ph2d_ecs::{ChildOf, Entity, SimWorld, SpriteSheetFrame, Transform, VecShape};
use ph2d_render::Sprite;

/// A caixa de uma peça no referencial LOCAL da folha: o centro do quad e as meias-extensões.
///
/// ⚠️ **O centro do quad não é a translação** quando o artista mexeu no pivô: o `Sprite::anchor` é
/// um deslocamento em metros locais, e o quad centra-se em `translation + R·(scale ⊙ anchor)` — a
/// mesma composição que o `picking` usa para decidir se o rato acertou. Confinar a *translação*
/// deixaria a peça a sair pela borda na exata medida do pivô deslocado, e o defeito só apareceria
/// nos sprites que alguém tivesse repivotado. *Uma segunda resposta a «onde está isto» é como as
/// duas passam a discordar.*
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PieceBox {
    /// Centro do quad, em unidades locais do pai.
    pub(crate) center: [f32; 2],
    /// Meias-extensões da AABB do quad (já com rotação e escala), em unidades locais do pai.
    pub(crate) half: [f32; 2],
    /// `center - translation` — o que é preciso subtrair para voltar da caixa à pose.
    pub(crate) pivot_offset: [f32; 2],
}

/// A caixa de uma peça, a partir do seu `Sprite` + `Transform` locais.
///
/// PURA de propósito: é aqui que mora a única trigonometria deste módulo, e um teste consegue
/// varrê-la sem montar um mundo.
pub(crate) fn piece_box(
    size: [f32; 2],
    t_translation: [f32; 2],
    t_rotation: f32,
    t_scale: [f32; 2],
    anchor: [f32; 2],
) -> PieceBox {
    let (sin, cos) = t_rotation.sin_cos();
    let (hx, hy) = (
        (size[0] * t_scale[0] * 0.5).abs(),
        (size[1] * t_scale[1] * 0.5).abs(),
    );
    // A AABB de um retângulo rodado: cada meia-extensão é a soma das projeções das duas metades.
    // Com rotação zero isto colapsa em `(hx, hy)`, que é o caso de todas as peças que o
    // empacotador coloca — a conta geral não custa nada e não tem caso especial.
    let half = [
        cos.abs() * hx + sin.abs() * hy,
        sin.abs() * hx + cos.abs() * hy,
    ];
    // O deslocamento do pivô, rodado e escalado como o quad.
    let ax = anchor[0] * t_scale[0];
    let ay = anchor[1] * t_scale[1];
    let pivot_offset = [cos * ax - sin * ay, sin * ax + cos * ay];
    PieceBox {
        center: [
            t_translation[0] + pivot_offset[0],
            t_translation[1] + pivot_offset[1],
        ],
        half,
        pivot_offset,
    }
}

/// Confina um centro dentro de uma caixa centrada na origem. PURA.
///
/// ⚠️ **Quando a peça é MAIOR que a folha o intervalo é vazio**, e a única resposta honesta é
/// centrá-la: qualquer outra escolheria arbitrariamente uma borda para transbordar. Não é um erro
/// silencioso — a [`health`] vê a mesma condição e acende a moldura, que é onde o artista a lê.
/// *Um `clamp` com `min > max` devolve lixo em silêncio; este eixo responde `0.0` e deixa o aviso
/// para quem o sabe mostrar.*
pub(crate) fn clamp_center(center: [f32; 2], half: [f32; 2], bounds_half: [f32; 2]) -> [f32; 2] {
    let axis = |c: f32, h: f32, bh: f32| {
        let limit = bh - h;
        if limit <= 0.0 {
            0.0
        } else {
            c.clamp(-limit, limit)
        }
    };
    [
        axis(center[0], half[0], bounds_half[0]),
        axis(center[1], half[1], bounds_half[1]),
    ]
}

/// O pai desta entidade, se — e só se — ele for uma folha.
pub(crate) fn sheet_parent(sim: &SimWorld, piece: Entity) -> Option<Entity> {
    let parent = sim.world().get::<ChildOf>(piece)?.parent();
    sim.world()
        .get::<SpriteSheetFrame>(parent)
        .is_some()
        .then_some(parent)
}

/// As meias-extensões do interior da folha, em unidades locais dela.
///
/// ⚠️ Do `VecShape`, que **é** o tamanho: a folha recusa ter um campo próprio de tamanho (vide
/// [`SpriteSheetFrame`]), e ler daqui é o que faz o redimensionamento pelo gizmo mudar as
/// fronteiras sem ninguém propagar coisa nenhuma.
pub(crate) fn sheet_half_local(sim: &SimWorld, sheet: Entity) -> Option<[f32; 2]> {
    match sim.world().get::<VecShape>(sheet)? {
        VecShape::Param { w, h, .. } => Some([(*w as f32 * 0.5).abs(), (*h as f32 * 0.5).abs()]),
        VecShape::Text(_) => None,
    }
}

/// A caixa de uma peça, lida do mundo. `None` se ela não for um sprite posicionado.
pub(crate) fn piece_box_of(sim: &SimWorld, piece: Entity) -> Option<PieceBox> {
    let world = sim.world();
    let sprite = world.get::<Sprite>(piece)?;
    let t = world.get::<Transform>(piece)?;
    Some(piece_box(
        sprite.size,
        [t.translation.x, t.translation.y],
        t.rotation,
        [t.scale.x, t.scale.y],
        sprite.anchor,
    ))
}

/// **Confina uma peça dentro da folha-mãe.** Devolve `true` se a pose mudou.
///
/// Não faz nada — e devolve `false` — quando a entidade não é filha de uma folha, quando não é um
/// sprite, ou quando já estava dentro. É esse silêncio que a torna segura de chamar a cada
/// `CursorMoved` de um arrasto: sem movimento não há escrita, e sem escrita o undo (que regista
/// por DIFF do mundo) não vê passo nenhum.
pub(crate) fn confine(sim: &mut SimWorld, piece: Entity) -> bool {
    let Some(sheet) = sheet_parent(sim, piece) else {
        return false;
    };
    let (Some(bounds_half), Some(bx)) = (sheet_half_local(sim, sheet), piece_box_of(sim, piece))
    else {
        return false;
    };
    let clamped = clamp_center(bx.center, bx.half, bounds_half);
    if clamped == bx.center {
        return false;
    }
    let Some(mut t) = sim.world_mut().get_mut::<Transform>(piece) else {
        return false;
    };
    t.translation = ph2d_core::Vec2::new(
        clamped[0] - bx.pivot_offset[0],
        clamped[1] - bx.pivot_offset[1],
    );
    true
}

/// O que está mal numa folha — as duas condições que o Enio quer ver na moldura.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct SheetHealth {
    /// Duas peças ocupam o mesmo pixel. No bake, uma taparia a outra.
    pub(crate) overlap: bool,
    /// Alguma peça sai da folha — porque a folha foi encolhida, ou porque a peça é maior que ela.
    pub(crate) overflow: bool,
}

impl SheetHealth {
    pub(crate) fn is_ok(self) -> bool {
        !self.overlap && !self.overflow
    }
}

/// Um retângulo `[x0, y0, x1, y1]` em unidades locais da folha.
type Aabb = [f32; 4];

/// As caixas dos filhos de uma folha, em unidades locais dela.
fn child_boxes(sim: &SimWorld, sheet: Entity) -> Vec<Aabb> {
    let Some(children) = sim.world().get::<bevy_ecs::hierarchy::Children>(sheet) else {
        return Vec::new();
    };
    children
        .iter()
        .filter(|c| {
            // ⚠️ Uma peça ESCONDIDA não conta para nenhuma das duas condições. Esconder é como o
            // artista põe uma peça de lado enquanto arruma as outras; acusá-la de sobrepor seria
            // acender a moldura por causa de algo que ele não vê.
            !sim.world()
                .get::<ph2d_ecs::Visibility>(**c)
                .is_some_and(|v| v.hidden)
        })
        .filter_map(|c| piece_box_of(sim, *c))
        .map(|b| {
            [
                b.center[0] - b.half[0],
                b.center[1] - b.half[1],
                b.center[0] + b.half[0],
                b.center[1] + b.half[1],
            ]
        })
        .collect()
}

/// **A saúde de uma folha, contada agora.**
///
/// ⚠️ **Varrimento, não par-a-par.** As caixas são ordenadas por `x0` e cada uma só é comparada
/// com as seguintes enquanto estas começarem antes de ela acabar — o clássico *sweep and prune* de
/// um eixo. O ingénuo é `O(n²)` e roda **por quadro, por folha**: com as peças que um atlas real
/// carrega isso é trabalho a mais no caminho do desenho, e o custo cresce onde ninguém olha. Aqui
/// é `O(n log n)` mais os pares que de facto se cruzam em `x`.
pub(crate) fn health(sim: &SimWorld, sheet: Entity) -> SheetHealth {
    let Some(bounds_half) = sheet_half_local(sim, sheet) else {
        return SheetHealth::default();
    };
    let mut boxes = child_boxes(sim, sheet);
    let overflow = boxes.iter().any(|b| {
        b[0] < -bounds_half[0] - EPS
            || b[1] < -bounds_half[1] - EPS
            || b[2] > bounds_half[0] + EPS
            || b[3] > bounds_half[1] + EPS
    });
    boxes.sort_by(|a, b| a[0].total_cmp(&b[0]));
    let mut overlap = false;
    'outer: for (i, a) in boxes.iter().enumerate() {
        for b in &boxes[i + 1..] {
            // Ordenadas por `x0`: assim que uma começa depois de `a` acabar, nenhuma das
            // seguintes pode cruzá-la em `x`. É isto que troca o `n²` por um varrimento.
            if b[0] >= a[2] - EPS {
                continue 'outer;
            }
            if a[1] < b[3] - EPS && b[1] < a[3] - EPS {
                overlap = true;
                break 'outer;
            }
        }
    }
    SheetHealth { overlap, overflow }
}

/// A folga que impede uma peça encostada à borda — que é onde o empacotador as põe — de se
/// denunciar como transbordo por causa do último bit de um `f32`.
///
/// ⚠️ **Relativa não seria melhor aqui:** as unidades são metros de cena, e um sprite de 32 px a
/// 100 px/m mede 0,32 — um épsilon relativo a essa grandeza seria menor que o erro que a cadeia
/// `px → metro → arranjo → pose` acumula. Este é o mesmo piso absoluto que o resto da shell usa.
const EPS: f32 = 1e-5;

#[cfg(test)]
mod tests {
    use super::*;

    /// Sem rotação nem pivô, a caixa é o que se espera — e é este o caso de toda peça que o
    /// empacotador coloca.
    #[test]
    fn an_unrotated_piece_is_its_own_box() {
        let b = piece_box([2.0, 4.0], [1.0, -1.0], 0.0, [1.0, 1.0], [0.0, 0.0]);
        assert_eq!(b.center, [1.0, -1.0]);
        assert_eq!(b.half, [1.0, 2.0]);
        assert_eq!(b.pivot_offset, [0.0, 0.0]);
    }

    /// A 90° a caixa TROCA de eixos. É a metade que um `size` cru erraria.
    #[test]
    fn a_quarter_turn_swaps_the_half_extents() {
        let b = piece_box(
            [2.0, 4.0],
            [0.0, 0.0],
            std::f32::consts::FRAC_PI_2,
            [1.0, 1.0],
            [0.0, 0.0],
        );
        assert!((b.half[0] - 2.0).abs() < 1e-5, "half={:?}", b.half);
        assert!((b.half[1] - 1.0).abs() < 1e-5, "half={:?}", b.half);
    }

    /// A escala entra na caixa. Arranjar pela caixa INTRÍNSECA deixaria uma peça escalada a
    /// sobrepor a vizinha — o mesmo motivo que o `collect_pieces` do empacotador já documenta.
    #[test]
    fn scale_enters_the_box() {
        let b = piece_box([2.0, 2.0], [0.0, 0.0], 0.0, [3.0, 0.5], [0.0, 0.0]);
        assert_eq!(b.half, [3.0, 0.5]);
    }

    /// ⚠️ **O pivô move o CENTRO, e o `pivot_offset` é o caminho de volta.** É o que permite
    /// confinar a caixa e escrever a pose.
    #[test]
    fn a_moved_pivot_offsets_the_centre_and_the_way_back() {
        let b = piece_box([2.0, 2.0], [10.0, 0.0], 0.0, [1.0, 1.0], [0.5, -0.25]);
        assert_eq!(b.center, [10.5, -0.25]);
        assert_eq!(b.pivot_offset, [0.5, -0.25]);
        // O caminho de volta: centro confinado − offset == a pose a escrever.
        let clamped = clamp_center(b.center, b.half, [2.0, 2.0]);
        assert_eq!(clamped, [1.0, -0.25]);
        assert_eq!(
            [
                clamped[0] - b.pivot_offset[0],
                clamped[1] - b.pivot_offset[1]
            ],
            [0.5, 0.0]
        );
    }

    #[test]
    fn a_piece_already_inside_is_left_alone() {
        assert_eq!(clamp_center([1.0, 1.0], [0.5, 0.5], [4.0, 4.0]), [1.0, 1.0]);
    }

    #[test]
    fn a_piece_pushed_out_comes_back_to_the_edge() {
        // A folha vai de -4 a +4; uma peça de meia-largura 1 pode centrar-se no máximo em 3.
        assert_eq!(
            clamp_center([99.0, -99.0], [1.0, 1.0], [4.0, 4.0]),
            [3.0, -3.0]
        );
    }

    /// ⚠️ O caso que um `clamp` ingénuo trata como lixo: a peça é MAIOR que a folha, o intervalo
    /// é vazio, e `f32::clamp(min > max)` entra em pânico. Aqui responde `0.0` — centrada — e é a
    /// [`health`] que acende a moldura.
    #[test]
    fn a_piece_bigger_than_the_sheet_centres_instead_of_panicking() {
        assert_eq!(
            clamp_center([5.0, -5.0], [10.0, 10.0], [4.0, 4.0]),
            [0.0, 0.0]
        );
    }

    /// Cada eixo decide sozinho: caber em X e não caber em Y é um caso real (uma faixa larga).
    #[test]
    fn the_axes_are_independent() {
        assert_eq!(
            clamp_center([0.0, 9.0], [0.5, 10.0], [4.0, 4.0]),
            [0.0, 0.0]
        );
        assert_eq!(
            clamp_center([9.0, 0.0], [0.5, 10.0], [4.0, 4.0]),
            [3.5, 0.0]
        );
    }
}
