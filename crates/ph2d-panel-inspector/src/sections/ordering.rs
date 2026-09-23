//! Ordering / Sorting — Inspector §7 section painter (Sprite Inspector
//! v2 W3). Renders the render-ready optional sorting components. Control
//! VALUES come from the `WidgetStore` (seeded by `sync_ordering_fields`)
//! or the snapshot; `info` drives the conditional rows.

use super::*;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::screens::hero::InspectorOrderingInfo;
use ph2d_editor_core::widget::SectionFold;
use ph2d_editor_core::widget::{Dropdown, DropdownOption, paint_dropdown_chip};
use ph2d_i18n::TextKey;
use ph2d_i18n::tr;

/// Canonical project sorting layers (spec §5.2 default set). Value ==
/// label so the dropdown's `selected` matches by name. Custom project
/// layer lists are a follow-up (no Project Settings UI yet).
pub(crate) const LAYER_LABELS: [TextKey; 5] = [
    TextKey::new("panel.inspector.ordering.background"),
    TextKey::new("panel.inspector.ordering.midground"),
    TextKey::new("panel.inspector.ordering.default"),
    TextKey::new("panel.inspector.ordering.foreground"),
    TextKey::new("panel.inspector.ordering.ui"),
];

/// ⭐ **O rótulo de um CAMPO, lido do descritor** (ADR-0166 · plano F0 — a §7 é a seção
/// piloto).
///
/// Antes desta wave o rótulo vivia como literal aqui e o nome do campo vivia no descritor:
/// **duas fontes para a mesma string**, e elas já tinham divergido (*"Sort At Root"* ×
/// *"Sort at Root"*; *"Y-Sort"* × *"Enabled"*). Quem manda é o produto — o descritor foi
/// corrigido para o que o Inspector pinta —, e agora há uma fonte só.
///
/// ⚠️ **Faltar não é opção silenciosa:** um `field_id` sem descrição faz o painel pintar
/// `??` em vez de uma linha muda ou de um rótulo inventado. É a lei do *zero no-op
/// silencioso* (DIRETIVA §2) aplicada a uma string.
fn field_label(canonical_name: &'static str, field_id: u16) -> &'static str {
    ph2d_component_desc::desc_for(canonical_name)
        .and_then(|d| d.field(field_id))
        .map_or("??", |f| tr(f.label_key))
}

/// O rótulo de um **marcador de tamanho zero** — nele a presença É o valor, então a linha
/// mostra o nome do COMPONENTE (`Show Behind Parent`), não o de um campo que não existe.
fn marker_label(canonical_name: &'static str) -> &'static str {
    ph2d_component_desc::desc_for(canonical_name).map_or("??", |d| tr(d.display_key))
}

/// ⭐⭐⭐ **A COLUNA desta secção — DERIVADA do descritor, como os rótulos dela já eram.**
///
/// ⛔⛔ Até 2026-09-15 as três famílias de linha desta secção (número · marcar · escolher) pediam
/// cada uma a **metade cega** ([`property_row_columns`] sem nome). Elas concordavam por acidente —
/// *três derivações da mesma coluna, e nenhuma sabia o que ia pintar* —, e o nome mais largo
/// (`Show Behind Parent`) não tinha como pedir a folga que o campo ao lado não usa (spec §6).
///
/// ⚠️ **A lista é o que a secção PINTA, incluindo as linhas gateadas** (*Sort at Root* só aparece
/// com o *Sorting Group* ligado, o *Axis X/Y* só com *Custom*): uma coluna que muda quando uma
/// linha aparece salta debaixo do olho do artista — a lei está no doc de [`Seccao::medida`].
///
/// ⛔ E ela é **derivada**, nunca escrita à mão: esta é a secção-piloto do ADR-0166, e uma segunda
/// lista de rótulos ao lado da primeira é a que envelhece.
fn seccao(text_system: &mut TextSystem) -> ph2d_editor_core::property_row::Seccao {
    ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &[
            field_label("ph2d::ecs::ZIndexOverride", 1),
            field_label("ph2d::ecs::ZAsRelative", 1),
            marker_label("ph2d::ecs::ShowBehindParent"),
            field_label("ph2d::ecs::OrderInLayer", 1),
            tr("panel.inspector.ordering.sorting_layer"),
            field_label("ph2d::ecs::YSort", 1),
            tr("panel.inspector.ordering.sort_point"),
            tr("panel.inspector.ordering.axis_x"),
            tr("panel.inspector.ordering.axis_y"),
            marker_label("ph2d::ecs::SortingGroup"),
            field_label("ph2d::ecs::SortingGroup", 1),
            marker_label("ph2d::ecs::TopLevel"),
        ],
    )
}

fn label_color(theme: Theme) -> VelloColor {
    resolve(ColorToken::Text2, theme)
}

/// One left-label + checkbox row. Returns the next `y`.
#[allow(clippy::too_many_arguments)]
fn check_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    id: NodeId,
    label: &str,
    sec: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    let (_, value) = store
        .checkbox(id)
        .unwrap_or((CheckboxState::Normal, CheckboxValue::Unchecked));
    ph2d_editor_core::property_row::paint_check_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        (id, label, matches!(value, CheckboxValue::Checked)),
        sec,
    )
}

/// One left-label + single NumberInput row. Returns the next `y`.
#[allow(clippy::too_many_arguments)]
fn number_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    id: NodeId,
    label: &str,
    sec: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    // ⭐ **A coluna do rótulo deixou de ser um literal** (14/09): ela era `96 px` escritos aqui, e
    // a mesma pergunta tinha SEIS respostas no app (`96` · `78` · `84` · `76` · `72` · `150`).
    // ⛔ Uma largura fixa está errada por construção — a coluna docada é arrastável.
    // ⭐⭐ E desde 15/09 a coluna é a da SECÇÃO, medida sobre os onze nomes que ela pinta.
    let h = ROW_H_PX;
    let row = ph2d_editor_core::property_row::colunas_da_linha(x, w, y, h, sec);
    ph2d_editor_core::widget::paint_property_label(
        text_system,
        scene,
        label,
        row.label.x,
        row.label.y + (h - TypeToken::Sm.px()) * 0.5,
        TypeToken::Sm.px(),
        row.label.w,
        label_color(theme),
    );
    let (dot, rect) = (row.dot, row.control);
    hit_index.register(id, rect);
    let (state, value, buffer, caret, anchor) = read_number_input(store, id);
    let input = NumberInput::new(id, "", value)
        .step(1.0)
        .visual((state, store.hover_live(id)));
    paint_number_input_with_buffer(
        &input,
        Some(buffer),
        caret,
        anchor,
        rect,
        scene,
        text_system,
        theme,
    );
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, dot);
    y + h + ph2d_tokens::control_gap_px()
}

/// Sorting Layer dropdown row. Stashes the open popover for the deferred
/// pass in `paint_inspector`. Returns the next `y`.
#[allow(clippy::too_many_arguments)]
fn layer_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    fallback_idx: usize,
    sec: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    // ⭐ Irmã da row acima: a coluna do rótulo sai da porta, e é a da SECÇÃO.
    let h = ROW_H_PX;
    let row = ph2d_editor_core::property_row::colunas_da_linha(x, w, y, h, sec);
    ph2d_editor_core::widget::paint_property_label(
        text_system,
        scene,
        tr("panel.inspector.ordering.sorting_layer"),
        row.label.x,
        row.label.y + (h - TypeToken::Sm.px()) * 0.5,
        TypeToken::Sm.px(),
        row.label.w,
        label_color(theme),
    );
    let (dot, rect) = (row.dot, row.control);
    hit_index.register(ids::INSP_ORDER_SORTING_LAYER, rect);
    let (open, sel) = match store.get(ids::INSP_ORDER_SORTING_LAYER) {
        Some(InteractiveState::Dropdown {
            open,
            selected_index,
            ..
        }) => (*open, selected_index.unwrap_or(fallback_idx)),
        _ => (false, fallback_idx),
    };
    let visual = store.dropdown_visual(ids::INSP_ORDER_SORTING_LAYER);
    let label = LAYER_LABELS
        .get(sel)
        .map_or(tr("panel.inspector.ordering.default"), |k| k.tr());
    let dd = Dropdown::new(ids::INSP_ORDER_SORTING_LAYER, "", layer_options())
        .selected(label)
        .open(open)
        .visual(visual);
    paint_dropdown_chip(&dd, rect, scene, text_system, theme);
    if open {
        crate::state_popovers::set_pending_ordering_dd(Some((sel, rect)));
    }
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, dot);
    y + h + ph2d_tokens::control_gap_px()
}

/// Canonical dropdown options (value == label).
pub(crate) fn layer_options() -> Vec<DropdownOption<&'static str>> {
    LAYER_LABELS
        .iter()
        .enumerate()
        .map(|(i, name)| DropdownOption::new(ids::INSP_ORDER_LAYER_OPT[i], name.tr(), name.tr()))
        .collect()
}

/// Y-Sort Sort Point segmented (Center/Pivot/Custom) + Custom Axis
/// (shown only on Custom). Snapshot-driven selection. Returns next `y`.
#[allow(clippy::too_many_arguments)]
fn ysort_point_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorOrderingInfo,
    sec: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    // ⛔⛔ **Ela não tinha NOME** — três botões (`Center · Pivot · Custom`) a toda a largura por baixo
    //    da caixa do `Y Sort`, e o artista tinha de adivinhar a que pergunta respondiam. Hoje é uma
    //    linha de escolha com o nome `Sort Point`, pela porta, na coluna da secção.
    let sel = info.y_sort_point as usize;
    let segmentos = [
        (
            tr("panel.inspector.ordering.center"),
            sel == 0,
            ids::INSP_ORDER_SP_CENTER,
        ),
        (
            tr("panel.inspector.ordering.pivot"),
            sel == 1,
            ids::INSP_ORDER_SP_PIVOT,
        ),
        (
            tr("panel.inspector.ordering.custom"),
            sel == 2,
            ids::INSP_ORDER_SP_CUSTOM,
        ),
    ];
    let mut cur_y = ph2d_editor_core::property_row::paint_choice_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        tr("panel.inspector.ordering.sort_point"),
        &segmentos,
        sec,
    );
    // Custom Axis — only when Sort Point = Custom (tag 2).
    if info.y_sort_point == 2 {
        cur_y = number_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            ids::INSP_ORDER_AXIS_X,
            tr("panel.inspector.ordering.axis_x"),
            sec,
        );
        cur_y = number_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            ids::INSP_ORDER_AXIS_Y,
            tr("panel.inspector.ordering.axis_y"),
            sec,
        );
    }
    cur_y
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_ordering_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorOrderingInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: section header band height
    let color_id = core_ids::INSP_LIVE_ORDERING_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: neutral default section accent
    let header = section_header(
        store,
        core_ids::INSP_LIVE_ORDERING_SECTION,
        tr("panel.inspector.ordering.ordering"),
    )
    .color(rgba);
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    if let Some(circle_rect) = ph2d_editor_core::widget::color_circle_hit_rect(&header, header_rect)
    {
        hit_index.register(color_id, circle_rect);
    }
    // ⚠️ **A DOBRA do corpo** — o escopo recorta a cena E o hit, e escala o `y` de saída, para
    //    que tudo o que está por baixo suba junto. Ver `SectionFold`.
    // ⚠️ **Pergunta o `t`, e NUNCA o `is_collapsed`:** ao clicar para fechar o flag semântico vira
    //    neste mesmo quadro enquanto o `t` ainda desce, então um corpo gateado no flag sumiria de
    //    repente por baixo de um chevron a rodar — as duas metades a discordar outra vez.
    let Some(fold) = SectionFold::begin(
        store,
        core_ids::INSP_LIVE_ORDERING_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };
    let mut yy = y + header_h;
    let sec = seccao(text_system);
    let live =
        |store: &WidgetStore, id: NodeId, snap: bool| match store.checkbox(id).map(|(_, v)| v) {
            Some(CheckboxValue::Checked) => true,
            Some(CheckboxValue::Unchecked) => false,
            _ => snap,
        };
    // Macros (not closures) so each row is a direct fn call that
    // re-borrows `scene`/`text_system`/`hit_index` per invocation.
    macro_rules! cb {
        ($yy:expr, $id:expr, $l:expr) => {
            check_row(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                x,
                w,
                $yy,
                $id,
                $l,
                sec,
            )
        };
    }
    macro_rules! ni {
        ($yy:expr, $id:expr, $l:expr) => {
            number_row(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                x,
                w,
                $yy,
                $id,
                $l,
                sec,
            )
        };
    }

    // Z Index — the field IS the override (no separate toggle, Godot-
    // style): a non-zero value attaches `ZIndexOverride`, `0` detaches it
    // (= default, pure DFS / hierarchy order — `0` and absent sort
    // identically). Z as Relative pairs with it.
    yy = ni!(
        yy,
        ids::INSP_ORDER_Z_INDEX,
        field_label("ph2d::ecs::ZIndexOverride", 1)
    );
    yy = cb!(
        yy,
        ids::INSP_ORDER_Z_RELATIVE,
        field_label("ph2d::ecs::ZAsRelative", 1)
    );
    yy = cb!(
        yy,
        ids::INSP_ORDER_SHOW_BEHIND,
        marker_label("ph2d::ecs::ShowBehindParent")
    );
    yy = ni!(
        yy,
        ids::INSP_ORDER_ORDER_IN_LAYER,
        field_label("ph2d::ecs::OrderInLayer", 1)
    );
    yy = layer_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        info.sorting_layer as usize,
        sec,
    );
    yy = cb!(
        yy,
        ids::INSP_ORDER_YSORT_ENABLED,
        field_label("ph2d::ecs::YSort", 1)
    );
    if live(store, ids::INSP_ORDER_YSORT_ENABLED, info.y_sort_enabled) {
        yy = ysort_point_rows(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            info,
            sec,
        );
    }
    // ⚠️ A linha-mãe mostra o nome do COMPONENTE (a presença do `SortingGroup` é o que ela
    // liga) e a filha mostra o nome do CAMPO dele — duas perguntas, dois rótulos.
    yy = cb!(
        yy,
        ids::INSP_ORDER_SORTING_GROUP,
        marker_label("ph2d::ecs::SortingGroup")
    );
    if live(store, ids::INSP_ORDER_SORTING_GROUP, info.sorting_group) {
        yy = cb!(
            yy,
            ids::INSP_ORDER_SORT_AT_ROOT,
            field_label("ph2d::ecs::SortingGroup", 1)
        );
    }
    yy = cb!(
        yy,
        ids::INSP_ORDER_TOP_LEVEL,
        marker_label("ph2d::ecs::TopLevel")
    );

    fold.finish(
        store,
        scene,
        hit_index,
        yy - Spacing::Sm.px() + SECTION_BOTTOM_PAD_PX,
    )
}
