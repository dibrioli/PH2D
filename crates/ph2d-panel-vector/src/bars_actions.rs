//! ④ **A barra das ações da seleção** — as duas linhas (*o que faço a seguir com isto?*). Irmão do
//! [`super`] pelo tecto de 600 LOC dos painéis; o corte é por LUGAR: aqui a ④, no `bars_row` o
//! gesto da fila ②, no `bars_control` a cabeça dela, lá a coluna ① e a ordem.

use super::button;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::{BarButton, BarFamily};
use ph2d_editor_core::widget::ToolRailEntry;
use ph2d_i18n::tr;
use ph2d_tool_vector::DrawMode;
use ph2d_tool_vector::ids;

/// As ferramentas de clique PRECISO sobre a arte: com uma delas na mão a barra ④ não aparece — a
/// barra pousa junto da seleção, e ali ela roubaria o clique que a ferramenta existe para dar.
const PRECISE: [DrawMode; 13] = [
    DrawMode::Rules,
    DrawMode::HudLayout,
    DrawMode::Cut,
    DrawMode::Trim,
    DrawMode::Bucket,
    DrawMode::Build,
    DrawMode::Fillet,
    DrawMode::Chamfer,
    DrawMode::Width,
    DrawMode::Offset,
    DrawMode::Connect,
    DrawMode::PickBlend,
    DrawMode::Eyedropper,
];

/// ⭐ (dono 25/10, F29: *«a tool Cut não exibe suas funções … vamos colocar na barra flutuante»*)
/// **A ④ do Corte**: os verbos dele SÃO o «a seguir» da lâmina que acabou de nascer seleccionada —
/// e só então, porque é só então que o Cortar corta (`apply_cut` exige a lâmina na seleção).
fn cut_actions(sel: usize) -> Vec<Vec<BarButton>> {
    if sel == 0 || !crate::state::cut_line_exists() {
        return Vec::new();
    }
    let b = |id, icon, key: &str| button(id, icon, tr(key).to_owned(), false, BarFamily::Carve);
    vec![vec![
        b(
            crate::ids::VECTOR_CUT_APPLY,
            IconId::Knife,
            "panel.vector.cut.apply",
        ),
        b(
            crate::ids::VECTOR_CUT_DISCARD,
            IconId::Trash,
            "panel.vector.cut.discard",
        ),
    ]]
}

/// ④ **Do Offset Path** (F48d): `Apply` torna a prévia viva numa forma; `Remove` desfaz a prévia.
/// Só com um offset VIVO na seleção — antes dele não há nada a aplicar.
fn offset_actions(sel: usize) -> Vec<Vec<BarButton>> {
    if sel == 0 || !crate::state::offset_live_on() {
        return Vec::new();
    }
    let b = |id, icon, key: &str| button(id, icon, tr(key).to_owned(), false, BarFamily::Stroke);
    vec![vec![
        b(
            ph2d_tool_vector::ids::VECTOR_EXPAND_OFFSET_PATH,
            IconId::Check,
            "panel.vector.expand.apply",
        ),
        b(
            crate::ids::VECTOR_EXPAND_REMOVE,
            IconId::Trash,
            "panel.vector.expand.remove",
        ),
    ]]
}

/// ④ as ações da seleção — *o que faço a seguir com isto?*, com seleção, em toda ferramenta que
/// não é de clique preciso. ⚠️ Os verbos de caminho não entram no Node: lá eles estão na ②.
pub(super) fn actions(mode: DrawMode, sel: usize) -> Vec<Vec<BarButton>> {
    if mode == DrawMode::Cut {
        return cut_actions(sel);
    }
    if mode == DrawMode::Offset {
        return offset_actions(sel);
    }
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
    // ⭐ (dono 15/10) **Dentro de uma booleana viva**: o verbo da forma em mãos (aceso o dela) — com
    // UMA forma selecionada, que é como se aponta um filho; sem isto a ④ não deixava mudar o modo. A regra de quando há fileira (a base não tem verbo…) é da shell (`vec_bool_shape`).
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
    out.push(combine(sel));
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

/// ⭐ F50 (dono 02/11: «faça como o Figma») **O que COMBINA formas**: as quatro booleanas (acesa a
/// do grupo vivo escolhido — trocar é clicar noutra), o `Flatten` que torna permanente, e o
/// `Outline Stroke`. A booleana é sempre viva; com uma forma solta só o Outline Stroke serve.
fn combine(sel: usize) -> Vec<BarButton> {
    let op = crate::state::bool_group_op();
    let mut out = Vec::new();
    if sel >= 2 || op.is_some() {
        let ops = [
            (
                crate::ids::VECTOR_BOOL_UNION,
                IconId::BoolUnion,
                "panel.vector.bar.union",
            ),
            (
                crate::ids::VECTOR_BOOL_SUBTRACT,
                IconId::BoolSubtract,
                "panel.vector.bar.subtract",
            ),
            (
                crate::ids::VECTOR_BOOL_INTERSECT,
                IconId::BoolIntersect,
                "panel.vector.bar.intersect",
            ),
            (
                crate::ids::VECTOR_BOOL_EXCLUDE,
                IconId::BoolExclude,
                "panel.vector.bar.exclude",
            ),
        ];
        out.extend(ops.into_iter().zip(0u8..).map(|((id, icon, key), code)| {
            button(
                id,
                icon,
                tr(key).to_owned(),
                op == Some(code),
                BarFamily::Neutral,
            )
        }));
        out.push(button(
            crate::ids::VECTOR_FLATTEN,
            IconId::Flatten,
            tr("panel.vector.bar.flatten").to_owned(),
            false,
            BarFamily::Neutral,
        ));
    }
    out.push(button(
        ph2d_tool_vector::ids::VECTOR_EXPAND_OUTLINE_STROKE,
        IconId::OutlineStroke,
        tr("panel.vector.bar.outline_stroke").to_owned(),
        false,
        BarFamily::Stroke,
    ));
    out
}

/// ⭐ F50 **O «⋯» da ④** — as raras: as quatro receitas do Pathfinder (com duas formas ou um grupo
/// vivo) e o Compound; o Release com qualquer seleção. Os ids são os do painel (a mesma porta).
pub(super) fn more(mode: DrawMode, sel: usize) -> Vec<ToolRailEntry> {
    if sel == 0 || PRECISE.contains(&mode) || matches!(mode, DrawMode::Cut | DrawMode::Offset) {
        return Vec::new();
    }
    let row = |id, key: &str| ToolRailEntry::icon(id, tr(key), IconId::MoreHorizontal);
    let mut out = Vec::new();
    if sel >= 2 || crate::state::bool_group_selected() {
        out.extend([
            row(
                ph2d_tool_vector::ids::VECTOR_BOOL_MINUS_BACK,
                "panel.vector.more.minus_back",
            ),
            row(
                ph2d_tool_vector::ids::VECTOR_BOOL_TRIM,
                "panel.vector.more.trim",
            ),
            row(
                ph2d_tool_vector::ids::VECTOR_BOOL_CROP,
                "panel.vector.more.crop",
            ),
            row(
                ph2d_tool_vector::ids::VECTOR_BOOL_MERGE,
                "panel.vector.more.merge",
            ),
            ToolRailEntry::Divider,
            row(
                ph2d_editor_core::ids::VECTOR_COMPOUND_MAKE,
                "panel.vector.more.compound",
            ),
        ]);
    }
    out.push(row(
        crate::ids::VECTOR_COMPOUND_RELEASE,
        "panel.vector.more.release",
    ));
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
