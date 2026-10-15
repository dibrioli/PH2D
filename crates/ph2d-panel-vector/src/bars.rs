//! ⭐⭐ **As BARRAS do vetor** (D10 — `docs/UI_New_and_Simple/vetor_nova_ui/03_plano.md`): o que a
//! coluna ①, a fila ② e a barra ④ mostram em cada modo, e a dica ⑤.
//!
//! **Cada controlo vai para o lugar cuja pergunta ele responde** (estudo §1): *que gesto tenho na
//! mão* → ①; *como o gesto actua* → ②; *o que faço a seguir com a seleção* → ④; *que teclas valem
//! agora* → ⑤. O que a seleção É continua no painel (③).
//!
//! ⚠️ **Os ids são os do painel** — o clique segue pela porta que ele já tem (`event.rs`), e a
//! tecla é um clique no mesmo id ([`crate::tool_keys`]). Nenhum comando ganha um segundo id.
//!
//! ⚠️ **A ORDEM da fila é a PRIORIDADE**: o `⋯` leva a cauda (F0, 07/10: em Edit + Node a fila pede
//! `1 051 px` e o iPad 12,9" dá `730`). Por isso o gesto entra À FRENTE dos chips do gizmo.

use crate::tool_keys;
#[path = "bars_row.rs"]
mod row;
use ph2d_a11y::NodeId;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::{
    AreaMenu, BarButton, BarFace, BarFamily, ToolBars, WidgetStore,
};
use ph2d_editor_core::widget::ToolRailEntry;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tool_vector::ids;
use ph2d_tool_vector::{DrawMode, VectorStyleSnapshot};
use row::{area_menus, gesture_chips, shape_chips, style_chips, symmetry_chips};
use std::cell::Cell;

/// ① **A coluna**, por família — a forma de cada ferramenta é o ícone dela, a cor é a família.
/// `(modo, id, ícone, família, chave do nome)`.
pub const TOOLS: [(DrawMode, NodeId, IconId, BarFamily, &str); 17] = [
    (
        DrawMode::Select,
        ids::VECTOR_MODE_SELECT,
        IconId::Select,
        BarFamily::Neutral,
        "panel.vector.mode.select",
    ),
    (
        DrawMode::Node,
        ids::VECTOR_MODE_NODE,
        IconId::VectorNode,
        BarFamily::Neutral,
        "panel.vector.mode.node",
    ),
    (
        DrawMode::Pen,
        ids::VECTOR_MODE_PEN,
        IconId::VectorPen,
        BarFamily::Draw,
        "panel.vector.mode.pen",
    ),
    (
        DrawMode::Pencil,
        ids::VECTOR_MODE_PENCIL,
        IconId::VectorPencil,
        BarFamily::Draw,
        "panel.vector.mode.pencil",
    ),
    (
        DrawMode::Shape,
        ids::VECTOR_MODE_SHAPE,
        IconId::VectorShape,
        BarFamily::Draw,
        "panel.vector.mode.shape",
    ),
    (
        DrawMode::Text,
        ids::VECTOR_MODE_TEXT,
        IconId::Text,
        BarFamily::Draw,
        "panel.vector.mode.text",
    ),
    (
        DrawMode::Build,
        ids::VECTOR_MODE_BUILD,
        IconId::VectorBuild,
        BarFamily::Build,
        "panel.vector.mode.build",
    ),
    (
        DrawMode::Connect,
        ids::VECTOR_MODE_CONNECT,
        IconId::VectorConnect,
        BarFamily::Build,
        "panel.vector.mode.connect",
    ),
    (
        DrawMode::Bucket,
        ids::VECTOR_MODE_BUCKET,
        IconId::VectorBucket,
        BarFamily::Build,
        "panel.vector.mode.bucket",
    ),
    (
        DrawMode::Frame,
        ids::VECTOR_MODE_FRAME,
        IconId::VectorFrame,
        BarFamily::Build,
        "panel.vector.mode.frame",
    ),
    (
        DrawMode::Cut,
        ids::VECTOR_MODE_CUT,
        IconId::Knife,
        BarFamily::Carve,
        "panel.vector.mode.cut",
    ),
    (
        DrawMode::Trim,
        ids::VECTOR_MODE_TRIM,
        IconId::VectorTrim,
        BarFamily::Carve,
        "panel.vector.mode.trim",
    ),
    (
        DrawMode::Fillet,
        ids::VECTOR_MODE_FILLET,
        IconId::VectorFillet,
        BarFamily::Carve,
        "panel.vector.mode.fillet",
    ),
    (
        DrawMode::Chamfer,
        ids::VECTOR_MODE_CHAMFER,
        IconId::VectorChamfer,
        BarFamily::Carve,
        "panel.vector.mode.chamfer",
    ),
    (
        DrawMode::Width,
        ids::VECTOR_MODE_WIDTH,
        IconId::VectorWidth,
        BarFamily::Stroke,
        "panel.vector.mode.width",
    ),
    (
        DrawMode::PickBlend,
        ids::VECTOR_MODE_PICKBLEND,
        IconId::VectorBlend,
        BarFamily::Stroke,
        "panel.vector.blend.pick_shapes",
    ),
    (
        DrawMode::Eyedropper,
        ids::VECTOR_MODE_EYEDROPPER,
        IconId::Eyedropper,
        BarFamily::Stroke,
        "panel.vector.mode.eyedropper",
    ),
];

/// O id do botão de um modo na coluna — `None` só se a tabela perdeu o modo (o gate o acusa).
#[must_use]
pub fn mode_id(mode: DrawMode) -> Option<NodeId> {
    TOOLS.iter().find(|r| r.0 == mode).map(|r| r.1)
}

/// ⑤ A dica do estado: a do modo, e a do Lápis diz o «segurar» só com o QuickShape ligado.
#[must_use]
pub fn hint_key_for(snap: &VectorStyleSnapshot) -> &'static str {
    if snap.mode == DrawMode::Pencil && crate::state::shape_edit().1 {
        return "panel.vector.hint.pencil_edit_shape";
    }
    if snap.mode == DrawMode::Pencil && snap.pencil_quickshape {
        return "panel.vector.hint.pencil_quickshape";
    }
    // F10: no modo Forma, com a forma VIVA de alças em foco (as alças à vista — armado no catálogo o
    // foco publicado é `None`), a dica ensina-as e diz que o vazio desenha outra.
    if snap.mode == DrawMode::Shape
        && crate::state::current_shape_focus()
            .is_some_and(|k| !ph2d_tool_vector::shape_handles::handles_of(k).is_empty())
    {
        return "panel.vector.hint.shape_handles";
    }
    hint_key(snap.mode)
}

/// ⑤ A chave da dica de um modo.
#[must_use]
pub fn hint_key(mode: DrawMode) -> &'static str {
    match mode {
        DrawMode::Select => "panel.vector.hint.select",
        DrawMode::Node => "panel.vector.hint.node",
        DrawMode::Pen => "panel.vector.hint.pen",
        DrawMode::Pencil => "panel.vector.hint.pencil",
        DrawMode::Shape => "panel.vector.hint.shape",
        DrawMode::Text => "panel.vector.hint.text",
        DrawMode::Build => "panel.vector.hint.build",
        DrawMode::Connect => "panel.vector.hint.connect",
        DrawMode::PickBlend => "panel.vector.hint.pickblend",
        DrawMode::Fillet => "panel.vector.hint.fillet",
        DrawMode::Chamfer => "panel.vector.hint.chamfer",
        DrawMode::Width => "panel.vector.hint.width",
        DrawMode::Trim => "panel.vector.hint.trim",
        DrawMode::Bucket => "panel.vector.hint.bucket",
        DrawMode::Cut => "panel.vector.hint.cut",
        DrawMode::Frame => "panel.vector.hint.frame",
        DrawMode::Eyedropper => "panel.vector.hint.eyedropper",
    }
}

thread_local! {
    /// O FOCO da cor (`X`): o Traço (`true`) ou o Preenchimento. Segue o seletor quando ele abre.
    static FOCUS_STROKE: Cell<bool> = const { Cell::new(false) };
}

/// `X` — troca o foco e devolve o swatch que ficou com ele (o seletor abre nele).
pub(crate) fn toggle_focus() -> NodeId {
    let stroke = !FOCUS_STROKE.with(Cell::get);
    FOCUS_STROKE.with(|c| c.set(stroke));
    if stroke {
        ids::VECTOR_STROKE_SWATCH
    } else {
        ids::VECTOR_FILL_SWATCH
    }
}

/// O nome de um botão com a tecla dele: `Pen · P`.
fn named(key: &str, chord: Option<tool_keys::Chord>) -> String {
    match chord {
        Some(c) => format!("{} \u{00b7} {}", tr(key), c.label()),
        None => tr(key).to_owned(),
    }
}

fn button(id: NodeId, icon: IconId, label: String, on: bool, family: BarFamily) -> BarButton {
    BarButton {
        id,
        face: BarFace::Icon(icon),
        label,
        on,
        family,
    }
}

/// ① a coluna: as ferramentas por família, e a cor em baixo (o padrão do Affinity, estudo §2.2).
fn column(snap: &VectorStyleSnapshot, focus_stroke: bool) -> Vec<Vec<BarButton>> {
    let mut groups: Vec<Vec<BarButton>> = Vec::new();
    let mut last: Option<BarFamily> = None;
    for (mode, id, icon, family, key) in TOOLS {
        if last != Some(family) {
            groups.push(Vec::new());
            last = Some(family);
        }
        let label = named(key, tool_keys::chord_for_mode(mode));
        if let Some(g) = groups.last_mut() {
            g.push(button(id, icon, label, snap.mode == mode, family));
        }
    }
    let swatch = |id, rgba: [u8; 4], ring, key, on| BarButton {
        id,
        face: BarFace::Swatch {
            rgba: (rgba[3] > 0).then_some(rgba),
            ring,
        },
        label: named(key, None),
        on,
        family: BarFamily::Neutral,
    };
    let chord = |l| {
        Some(tool_keys::Chord {
            letter: l,
            shift: false,
            ctrl_alt: false,
        })
    };
    groups.push(vec![
        swatch(
            ids::VECTOR_FILL_SWATCH,
            snap.fill,
            false,
            "panel.vector.bar.fill",
            !focus_stroke,
        ),
        swatch(
            ids::VECTOR_STROKE_SWATCH,
            snap.stroke,
            true,
            "panel.vector.bar.stroke",
            focus_stroke,
        ),
        button(
            ids::VECTOR_COLOUR_FOCUS,
            IconId::ChevronDown,
            named("panel.vector.bar.focus", chord('x')),
            false,
            BarFamily::Neutral,
        ),
        button(
            ids::VECTOR_COLOUR_SWAP,
            IconId::PathReverse,
            named(
                "panel.vector.bar.swap",
                Some(tool_keys::Chord {
                    letter: 'x',
                    shift: true,
                    ctrl_alt: false,
                }),
            ),
            false,
            BarFamily::Neutral,
        ),
        button(
            ids::VECTOR_COLOUR_DEFAULT,
            IconId::Reset,
            named("panel.vector.bar.default", chord('d')),
            false,
            BarFamily::Neutral,
        ),
    ]);
    groups
}

/// ① o flyout da Forma: o catálogo inteiro, pela mesma lista e o mesmo cozido do painel.
fn shape_flyout(snap: &VectorStyleSnapshot) -> Vec<BarButton> {
    ph2d_tool_vector::shapes::SHAPES
        .iter()
        .enumerate()
        .filter_map(|(i, d)| {
            Some(BarButton {
                id: ids::vector_shape_id(i),
                face: BarFace::Outline(crate::shape_outlines::thumb_outline(i)?),
                label: crate::nomes_do_motor::forma(d.label).to_owned(),
                on: snap.shape == d.kind,
                family: BarFamily::Draw,
            })
        })
        .collect()
}

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
fn actions(mode: DrawMode, sel: usize) -> Vec<Vec<BarButton>> {
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
    if sel >= 2 {
        out.push(vec![
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
    out.push(vec![
        b(
            crate::ids::VECTOR_ARRANGE_DUPLICATE,
            IconId::Duplicate,
            "panel.vector.arrange.duplicate",
        ),
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

/// ④ a 2.ª linha — o ALINHAR (dono 15/10: saiu do painel para a barra, numa linha própria): com
/// duas formas ou mais, e o Distribuir com três (as duas das pontas ficam onde estão).
fn actions_below(mode: DrawMode, sel: usize) -> Vec<Vec<BarButton>> {
    if sel < 2 || PRECISE.contains(&mode) {
        return Vec::new();
    }
    let b = |id, icon, key: &str| button(id, icon, tr(key).to_owned(), false, BarFamily::Neutral);
    let mut out = vec![
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
    ];
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

/// ⭐ **As barras deste quadro** — o que a crate do vetor publica no store em todo quadro em que
/// a ferramenta está na mão. `selection` é a seleção no ECRÃ (a âncora da ④); `readout` são as
/// leituras do rodapé já formatadas.
#[must_use]
pub fn tool_bars(store: &WidgetStore, selection: Option<Rect>, readout: Vec<String>) -> ToolBars {
    let snap = crate::state::current_snapshot();
    // O foco segue o seletor quando ele abre num dos dois (o clique numa amostra É escolher o foco).
    match store.picker_target() {
        Some(id) if id == ids::VECTOR_STROKE_SWATCH => FOCUS_STROKE.with(|c| c.set(true)),
        Some(id) if id == ids::VECTOR_FILL_SWATCH => FOCUS_STROKE.with(|c| c.set(false)),
        _ => {}
    }
    let focus = FOCUS_STROKE.with(Cell::get);
    let sel = crate::state::current_selection_count();
    let verts = crate::state::current_vertex_count();
    // ⭐⭐ **A ORDEM é a prioridade, e a 1.ª pergunta é «isto funciona atrás do `⋯`?»** — medido em
    // 07/10 no iPad mini: a largura do traço (um CAMPO) ia para o `⋯`, onde nada se edita; uma
    // amostra lá não abre a cor (o menu não tem o Down que a abre). ⇒ à frente vai o que SÓ
    // funciona na fila (as amostras e todos os campos, os da simetria incluídos); depois os botões
    // do gesto, que atrás do `⋯` continuam clicáveis; na cauda os botões da simetria.
    let is_field = |e: &ToolRailEntry| matches!(e, ToolRailEntry::Number { .. });
    let (mut fields, gesture): (Vec<_>, Vec<_>) = gesture_chips(&snap, sel, verts)
        .into_iter()
        .partition(is_field);
    let (sym_fields, sym_chips): (Vec<_>, Vec<_>) =
        symmetry_chips(&snap).into_iter().partition(is_field);
    fields.extend(sym_fields);
    let mut row_lead = style_chips(&snap);
    row_lead.extend(fields);
    // Os parâmetros da forma em foco (F9) são campos: à frente, num grupo que o separador nomeia.
    let shape = shape_chips(&snap, store);
    if !shape.is_empty() {
        row_lead.push(ToolRailEntry::Divider);
        row_lead.extend(shape);
    }
    let gesture: Vec<ToolRailEntry> = gesture
        .into_iter()
        .skip_while(|e| matches!(e, ToolRailEntry::Divider))
        .collect();
    if !gesture.is_empty() {
        row_lead.push(ToolRailEntry::Divider);
        row_lead.extend(gesture);
    }
    ToolBars {
        column: column(&snap, focus),
        flyout: (snap.mode == DrawMode::Shape)
            .then(|| (ids::VECTOR_MODE_SHAPE, shape_flyout(&snap))),
        row_lead,
        row_tail: sym_chips,
        actions: actions(snap.mode, sel),
        actions_below: actions_below(snap.mode, sel),
        selection,
        hint: tr(hint_key_for(&snap)).to_owned(),
        readout,
    }
}

/// Os pulldowns do módulo deste quadro (o encaixe e o ⚙ do caminho).
#[must_use]
pub fn module_menus() -> Vec<AreaMenu> {
    let snap = crate::state::current_snapshot();
    area_menus(snap.mode, crate::state::current_selection_count())
}

#[cfg(test)]
#[path = "bars_tests.rs"]
mod tests;
