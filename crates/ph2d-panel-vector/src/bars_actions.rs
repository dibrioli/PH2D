//! ④ **A barra das ações da seleção** — as duas linhas (*o que faço a seguir com isto?*). Irmão do
//! [`super`] pelo tecto de 600 LOC dos painéis; o corte é por LUGAR: aqui a ④, no `bars_row` o
//! gesto da fila ②, no `bars_control` a cabeça dela, lá a coluna ① e a ordem.

use super::button;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::{BarButton, BarFamily};
use ph2d_i18n::tr;
use ph2d_tool_vector::DrawMode;
use ph2d_tool_vector::ids;

/// As ferramentas de clique PRECISO sobre a arte: com uma delas na mão a barra ④ não aparece — a
/// barra pousa junto da seleção, e ali ela roubaria o clique que a ferramenta existe para dar.
const PRECISE: [DrawMode; 10] = [
    DrawMode::Cut,
    DrawMode::Trim,
    DrawMode::Bucket,
    DrawMode::Build,
    DrawMode::Fillet,
    DrawMode::Chamfer,
    DrawMode::Width,
    DrawMode::Connect,
    DrawMode::PickBlend,
    DrawMode::Eyedropper,
];

/// ④ as ações da seleção — *o que faço a seguir com isto?*, com seleção, em toda ferramenta que
/// não é de clique preciso. ⚠️ Os verbos de caminho não entram no Node: lá eles estão na ②.
pub(super) fn actions(mode: DrawMode, sel: usize) -> Vec<Vec<BarButton>> {
    if sel == 0 || PRECISE.contains(&mode) {
        return Vec::new();
    }
    let b = |id, icon, key: &str| button(id, icon, tr(key).to_owned(), false, BarFamily::Neutral);
    let mut out = Vec::new();
    // ⭐ O Edit Shape (F7e) À FRENTE: é o «a seguir» de uma forma que acabou de nascer (o botão que
    // o Procreate mostra ao soltar). Aceso com os nós à vista.
    let (offered, open) = crate::state::shape_edit();
    if offered {
        out.push(vec![button(
            crate::ids::VECTOR_PENCIL_EDIT_SHAPE,
            IconId::VectorNode,
            tr("panel.vector.pencil.edit_shape").to_owned(),
            open,
            BarFamily::Neutral,
        )]);
    }
    // ⭐ (dono 15/10) **Dentro de uma booleana viva**: o verbo da forma em mãos (aceso o dela) e o
    // Apply — com UMA forma selecionada, que é como se aponta um filho; sem isto a ④ não deixava
    // mudar o modo. A regra de quando há fileira (a base não tem verbo…) é da shell (`vec_bool_shape`).
    if let Some((code, name)) = crate::state::bool_shape_row() {
        let shape = if name.is_empty() {
            tr("panel.vector.bool.shape").to_owned()
        } else {
            name
        };
        let verbs = [
            (
                crate::ids::VECTOR_BOOL_SHAPE_UNION,
                IconId::BoolUnion,
                "panel.vector.bool.shape.union",
            ),
            (
                crate::ids::VECTOR_BOOL_SHAPE_SUBTRACT,
                IconId::BoolSubtract,
                "panel.vector.bool.shape.subtract",
            ),
            (
                crate::ids::VECTOR_BOOL_SHAPE_INTERSECT,
                IconId::BoolIntersect,
                "panel.vector.bool.shape.intersect",
            ),
            (
                crate::ids::VECTOR_BOOL_SHAPE_EXCLUDE,
                IconId::BoolExclude,
                "panel.vector.bool.shape.exclude",
            ),
        ];
        out.push(
            verbs
                .into_iter()
                .zip(0u8..)
                .map(|((id, icon, key), i)| {
                    let label = tr("panel.vector.bar.bool_shape")
                        .replace("{shape}", &shape)
                        .replace("{verb}", tr(key));
                    button(id, icon, label, code == i, BarFamily::Neutral)
                })
                .collect(),
        );
    }
    if crate::state::bool_group_selected() {
        out.push(vec![b(
            crate::ids::VECTOR_BOOL_APPLY,
            IconId::Check,
            "panel.vector.bool.apply",
        )]);
    }
    if sel >= 2 {
        // ⭐ (dono 15/10) A BOOLEANA VIVA à frente das quatro: é o MODO delas (`On` cria um grupo
        // cujos filhos continuam editáveis). Um interruptor: aceso = ligada, e o id é o do estado
        // OPOSTO — o clique pede o que ainda não é (a mesma lei dos chips do encaixe).
        let live = crate::state::bool_live_on();
        out.push(vec![
            button(
                if live {
                    crate::ids::VECTOR_BOOL_LIVE_OFF
                } else {
                    crate::ids::VECTOR_BOOL_LIVE_ON
                },
                IconId::Link,
                tr("panel.vector.bar.bool_live").to_owned(),
                live,
                BarFamily::Neutral,
            ),
            b(
                crate::ids::VECTOR_BOOL_UNION,
                IconId::BoolUnion,
                "panel.vector.bool.union",
            ),
            b(
                crate::ids::VECTOR_BOOL_SUBTRACT,
                IconId::BoolSubtract,
                "panel.vector.bool.subtract",
            ),
            b(
                crate::ids::VECTOR_BOOL_INTERSECT,
                IconId::BoolIntersect,
                "panel.vector.bool.intersect",
            ),
            b(
                crate::ids::VECTOR_BOOL_EXCLUDE,
                IconId::BoolExclude,
                "panel.vector.bool.exclude",
            ),
        ]);
    }
    // ⭐ O ARRANJO inteiro (dono 15/10: a secção Arrange saiu do painel): a ordem na pilha, da
    // frente para trás, e a orientação. O Z numérico é do Inspector no modo Objeto (a ordenação).
    // ⛔ O Duplicate saiu (dono 15/10, F16): ficam o Ctrl+D e o menu da Hierarchy.
    out.push(vec![
        b(
            ids::VECTOR_ARRANGE_TO_FRONT,
            IconId::ArrangeFront,
            "panel.vector.arrange.to_front",
        ),
        b(
            ids::VECTOR_ARRANGE_FORWARD,
            IconId::ArrangeForward,
            "panel.vector.arrange.forward",
        ),
        b(
            ids::VECTOR_ARRANGE_BACKWARD,
            IconId::ArrangeBackward,
            "panel.vector.arrange.backward",
        ),
        b(
            ids::VECTOR_ARRANGE_TO_BACK,
            IconId::ArrangeBack,
            "panel.vector.arrange.to_back",
        ),
    ]);
    out.push(vec![
        b(
            ids::VECTOR_ARRANGE_FLIP_H,
            IconId::FlipH,
            "panel.vector.arrange.flip_h",
        ),
        b(
            ids::VECTOR_ARRANGE_FLIP_V,
            IconId::FlipV,
            "panel.vector.arrange.flip_v",
        ),
        b(
            ids::VECTOR_ARRANGE_ROTATE_CCW,
            IconId::RotateCcw,
            "panel.vector.arrange.rotate_ccw",
        ),
        b(
            ids::VECTOR_ARRANGE_ROTATE_CW,
            IconId::RotateCw,
            "panel.vector.arrange.rotate_cw",
        ),
    ]);
    if mode != DrawMode::Node {
        let mut path = Vec::new();
        if crate::state::convertible() {
            path.push(b(
                crate::ids::VECTOR_CONVERT_TO_CURVES,
                IconId::ConvertCurves,
                "panel.vector.arrange.convert_to_curves",
            ));
        }
        if sel >= 2 {
            path.push(b(
                crate::ids::VECTOR_PATH_JOIN,
                IconId::PathJoin,
                "panel.vector.arrange.join",
            ));
        }
        path.push(b(
            crate::ids::VECTOR_PATH_WELD,
            IconId::PathWeld,
            "panel.vector.arrange.weld",
        ));
        out.push(path);
    }
    out
}

/// ④ a 2.ª linha — os verbos que REMODELAM o caminho (F16: eram o pulldown «Path» da fila ②, que
/// virou a barra de controle) e o ALINHAR (dono 15/10: saiu do painel para a barra): com duas formas
/// ou mais, e o Distribuir com três (as duas das pontas ficam onde estão).
pub(super) fn actions_below(mode: DrawMode, sel: usize) -> Vec<Vec<BarButton>> {
    if sel == 0 || PRECISE.contains(&mode) {
        return Vec::new();
    }
    let b = |id, icon, key: &str| button(id, icon, tr(key).to_owned(), false, BarFamily::Neutral);
    let mut out = Vec::new();
    if matches!(mode, DrawMode::Select | DrawMode::Node) {
        let mut path = vec![
            b(
                ids::VECTOR_PATH_SMOOTH,
                IconId::PathSmooth,
                "panel.vector.arrange.smooth",
            ),
            b(
                ids::VECTOR_PATH_SHARPEN,
                IconId::PathSharpen,
                "panel.vector.arrange.sharpen",
            ),
            b(
                ids::VECTOR_PATH_SIMPLIFY,
                IconId::PathSimplify,
                "panel.vector.arrange.simplify",
            ),
            b(
                ids::VECTOR_PATH_SUBDIVIDE,
                IconId::PathSubdivide,
                "panel.vector.arrange.subdivide",
            ),
        ];
        // No Select o Fechar e o Inverter não estão na ② (são do gesto do Node): moram aqui.
        if mode == DrawMode::Select {
            path.push(b(
                crate::ids::VECTOR_PATH_CLOSE,
                IconId::PathClose,
                "panel.vector.arrange.close_path",
            ));
            path.push(b(
                crate::ids::VECTOR_PATH_REVERSE,
                IconId::PathReverse,
                "panel.vector.arrange.reverse",
            ));
        }
        out.push(path);
    }
    if sel < 2 {
        return out;
    }
    out.extend([
        vec![
            b(
                ids::VECTOR_ALIGN_LEFT,
                IconId::AlignLeft,
                "panel.vector.arrange.left",
            ),
            b(
                ids::VECTOR_ALIGN_HCENTER,
                IconId::AlignCenter,
                "panel.vector.arrange.center",
            ),
            b(
                ids::VECTOR_ALIGN_RIGHT,
                IconId::AlignRight,
                "panel.vector.arrange.right",
            ),
        ],
        vec![
            b(
                ids::VECTOR_ALIGN_TOP,
                IconId::AlignTop,
                "panel.vector.arrange.top",
            ),
            b(
                ids::VECTOR_ALIGN_VCENTER,
                IconId::AlignMiddle,
                "panel.vector.arrange.middle",
            ),
            b(
                ids::VECTOR_ALIGN_BOTTOM,
                IconId::AlignBottom,
                "panel.vector.arrange.bottom",
            ),
        ],
    ]);
    if sel >= 3 {
        out.push(vec![
            b(
                ids::VECTOR_DISTRIBUTE_H,
                IconId::DistributeH,
                "panel.vector.arrange.dist_h",
            ),
            b(
                ids::VECTOR_DISTRIBUTE_V,
                IconId::DistributeV,
                "panel.vector.arrange.dist_v",
            ),
        ]);
    }
    out
}
