//! ② **A BARRA DE CONTROLE** (F16, dono 15/10: *«a barra de ajustes no topo como no illustrator,
//! principalmente com os ajustes do stroke»*) — a *Control bar* do Illustrator na fila ②:
//! `nome │ Fill │ Stroke │ Stroke ▾ │ peso │ Perfil ▾ │ Tinta ▾ │ Opacity`, e o painel do traço que o
//! «Stroke ▾» abre POR BAIXO do chip (o *Stroke panel* que o Illustrator abre do «Stroke:»).
//!
//! ⚠️ **Nenhum controlo novo**: cada chip é outra porta para um id que o painel já despacha (o peso é o
//! `VECTOR_WIDTH_NUM`, a opacidade o `VECTOR_OBJ_OPACITY_NUM`, as linhas das listas os perfis e as
//! tintas), e o painel do traço pinta o MESMO corpo da secção Stroke (`BodyCtx::stroke_body`).
//! Plano: `docs/UI_New_and_Simple/vetor_nova_ui/26_plano_F16_barra_de_controle.md`.

use crate::paint_sections::BodyCtx;
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::panel_chrome::{PANEL_HEAD_PAD, paint_panel_surface};
use ph2d_editor_core::widget::{
    Dropdown, DropdownOption, DropdownState, ToolRailEntry, number_input_min_w_px,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ROW_H_PX, Spacing, Theme, TypeToken};
use ph2d_tool_vector::{VectorStyleSnapshot, ids};

/// Os três chips da barra que abrem uma lista ou um painel por baixo deles.
const LISTS: [NodeId; 3] = [
    crate::ids::VECTOR_BAR_STROKE,
    crate::ids::VECTOR_BAR_PROFILE,
    crate::ids::VECTOR_BAR_KIND,
];

/// As tintas do traço, pela ordem do segmentado do painel (id da linha, chave do nome, tinta).
const KINDS: [(NodeId, &str, crate::state::StrokePaintKind); 3] = [
    (
        ids::VECTOR_STROKE_KIND_SOLID,
        "panel.vector.stroke.solid",
        crate::state::StrokePaintKind::Solid,
    ),
    (
        ids::VECTOR_STROKE_KIND_PATTERN,
        "panel.vector.stroke.pattern",
        crate::state::StrokePaintKind::Pattern,
    ),
    (
        ids::VECTOR_STROKE_KIND_BRUSH,
        "panel.vector.stroke.brush",
        crate::state::StrokePaintKind::Brush,
    ),
];

/// Regista os três chips como `Dropdown` do store: o clique no chip abre/fecha, e o light-dismiss
/// genérico (`pointer_down`) fecha-os num clique fora do que publicaram.
pub(crate) fn populate(store: &mut WidgetStore) {
    for id in LISTS {
        store.register_if_absent(
            id,
            InteractiveState::Dropdown {
                state: DropdownState::Normal,
                open: false,
                selected_index: None,
            },
        );
    }
}

fn is_open(store: &WidgetStore, id: NodeId) -> bool {
    matches!(
        store.get(id),
        Some(InteractiveState::Dropdown { open: true, .. })
    )
}

/// O painel do traço está aberto por baixo da barra?
#[must_use]
pub(crate) fn stroke_popover_open(store: &WidgetStore) -> bool {
    is_open(store, crate::ids::VECTOR_BAR_STROKE)
}

/// O perfil de largura DA FORMA selecionada (o Uniform sem perfil), `None` = «Custom».
fn active_profile() -> Option<usize> {
    crate::state::current_selection_profile()
}

fn profile_names() -> impl Iterator<Item = &'static str> {
    ph2d_vec_scene::WIDTH_PRESETS
        .iter()
        .take(crate::ids::MAX_WIDTH_PRESETS)
        .map(|p| tr(p.key))
}

/// O nome à cabeça da barra: a forma selecionada, «N objects», ou «No selection».
fn selection_name(sel: usize) -> String {
    match sel {
        0 => tr("panel.vector.bar.name.none").to_owned(),
        1 => crate::state::current_selection_name()
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| tr("panel.vector.bar.name.many").replace("{n}", "1")),
        n => tr("panel.vector.bar.name.many").replace("{n}", &n.to_string()),
    }
}

/// Um chip da fila que abre uma LISTA (aceso enquanto ela está aberta).
fn list_chip(
    store: &WidgetStore,
    id: NodeId,
    key: &str,
    face: String,
    faces: Vec<String>,
    sub: &str,
) -> ToolRailEntry {
    let e = ToolRailEntry::compound(id, tr(key), face, sub).showing(faces);
    if is_open(store, id) { e.active() } else { e }
}

/// ⭐ **A cabeça da fila ②** — o que só funciona NA fila (o `⋯` leva a cauda): o nome, as amostras,
/// o «Stroke ▾», o peso, e com seleção o perfil, a tinta e a opacidade do objecto.
pub(super) fn control_chips(snap: &VectorStyleSnapshot, store: &WidgetStore) -> Vec<ToolRailEntry> {
    let sel = crate::state::current_selection_count();
    let mut v = vec![
        ToolRailEntry::text(selection_name(sel)),
        ToolRailEntry::swatch(
            ids::VECTOR_FILL_SWATCH,
            tr("panel.vector.bar.fill"),
            snap.fill,
        )
        .mixed(!snap.agrees(ph2d_tool_vector::StyleFields::FILL_RGB))
        .with_sub(tr("panel.vector.bar.sub.fill")),
        ToolRailEntry::swatch(
            ids::VECTOR_STROKE_SWATCH,
            tr("panel.vector.bar.stroke"),
            snap.stroke,
        )
        .mixed(!snap.agrees(ph2d_tool_vector::StyleFields::STROKE_RGB))
        .with_sub(tr("panel.vector.bar.sub.stroke")),
        list_chip(
            store,
            crate::ids::VECTOR_BAR_STROKE,
            "panel.vector.bar.stroke_options",
            tr("panel.vector.bar.stroke_more").to_owned(),
            Vec::new(),
            "",
        ),
        ToolRailEntry::number(
            ids::VECTOR_WIDTH_NUM,
            tr("panel.vector.bar.width"),
            tr("panel.vector.bar.sub.width"),
            // Seleção com espessuras diferentes: o «W» fica em branco (o «misto» dos números).
            (!snap.agrees(ph2d_tool_vector::StyleFields::WIDTH)).then(String::new),
        ),
    ];
    // O perfil é VIVO na seleção (ADR-0148): sem forma selecionada não há a quem o dar.
    if sel > 0 {
        let custom = tr("panel.vector.bar.profile.custom");
        let face = active_profile()
            .and_then(|i| profile_names().nth(i))
            .unwrap_or(custom);
        let mut faces: Vec<String> = profile_names().map(str::to_owned).collect();
        faces.push(custom.to_owned());
        v.push(list_chip(
            store,
            crate::ids::VECTOR_BAR_PROFILE,
            "panel.vector.bar.profile",
            face.to_owned(),
            faces,
            tr("panel.vector.bar.sub.profile"),
        ));
    }
    // A tinta só existe com UMA forma com traço — a mesma condição da fileira do painel.
    if let Some(k) = crate::state::stroke_paint_kind() {
        let face = KINDS
            .iter()
            .find(|(_, _, kind)| *kind == k)
            .map_or("", |(_, key, _)| tr(key));
        v.push(list_chip(
            store,
            crate::ids::VECTOR_BAR_KIND,
            "panel.vector.bar.kind",
            face.to_owned(),
            KINDS.iter().map(|(_, key, _)| tr(key).to_owned()).collect(),
            tr("panel.vector.bar.sub.kind"),
        ));
    }
    if let Some(a) = crate::state::current_appearance() {
        v.push(ToolRailEntry::number(
            crate::ids::VECTOR_OBJ_OPACITY_NUM,
            tr("panel.vector.bar.opacity"),
            tr("panel.vector.bar.sub.opacity"),
            a.mixed_opacity.then(String::new),
        ));
    }
    v
}

/// Um `Click` num dos três chips chega aqui só pela PALETA (o ponteiro abre-os no store sem evento,
/// `dispatch::focus`): abre o pedido e fecha os outros dois — um popover de cada vez.
pub(crate) fn toggle_list(store: &mut WidgetStore, id: NodeId) -> bool {
    if !LISTS.contains(&id) {
        return false;
    }
    for other in LISTS {
        if let Some(InteractiveState::Dropdown { open, .. }) = store.get_mut(other) {
            *open = other == id && !*open;
        }
    }
    true
}

/// Uma linha de uma das duas listas foi escolhida: fecha o chip que a abriu (o light-dismiss não
/// dispara num clique DENTRO da lista). O verbo da linha é o do painel, que corre a seguir.
pub(crate) fn close_list_on_pick(store: &mut WidgetStore, id: NodeId) {
    let profile =
        (0..crate::ids::MAX_WIDTH_PRESETS).any(|i| id == crate::ids::vector_width_preset_id(i));
    let kind = KINDS.iter().any(|(k, _, _)| *k == id);
    let owner = if profile {
        crate::ids::VECTOR_BAR_PROFILE
    } else if kind {
        crate::ids::VECTOR_BAR_KIND
    } else {
        return;
    };
    if let Some(InteractiveState::Dropdown { open, .. }) = store.get_mut(owner) {
        *open = false;
    }
}

/// ⭐⭐ **O que a barra abre, por baixo dela** — o painel do traço e as duas listas. Pintado no passe
/// diferido do painel (por cima das secções), ancorado no chip que o abriu
/// ([`WidgetStore::tool_row_slot`], o quadro anterior: o painel pinta antes da fila).
pub(crate) fn paint_bar_popovers(ctx: &mut PaintCtx, theme: Theme, width: f32) {
    let store = ctx.host.store();
    let stroke = stroke_popover_open(store)
        .then(|| store.tool_row_slot(crate::ids::VECTOR_BAR_STROKE))
        .flatten();
    let profile = is_open(store, crate::ids::VECTOR_BAR_PROFILE)
        .then(|| store.tool_row_slot(crate::ids::VECTOR_BAR_PROFILE))
        .flatten();
    let kind = is_open(store, crate::ids::VECTOR_BAR_KIND)
        .then(|| store.tool_row_slot(crate::ids::VECTOR_BAR_KIND))
        .flatten();
    if let Some(chip) = stroke {
        paint_stroke_popover(ctx, theme, chip, width);
    }
    let region = ctx.layout.popover_region();
    if let Some(chip) = profile {
        let options = profile_names()
            .enumerate()
            .map(|(i, name)| DropdownOption::new(crate::ids::vector_width_preset_id(i), i, name))
            .collect();
        let mut dd = Dropdown::new(crate::ids::VECTOR_BAR_PROFILE, "", options).open(true);
        if let Some(i) = active_profile() {
            dd = dd.selected(i);
        }
        ph2d_editor_core::panel::paint_dropdown_list(ctx, &dd, chip, region, theme, None);
    }
    if let Some(chip) = kind {
        let options = KINDS
            .iter()
            .enumerate()
            .map(|(i, (id, key, _))| DropdownOption::new(*id, i, tr(key)))
            .collect();
        let mut dd = Dropdown::new(crate::ids::VECTOR_BAR_KIND, "", options).open(true);
        if let Some(k) = crate::state::stroke_paint_kind()
            && let Some(i) = KINDS.iter().position(|(_, _, kind)| *kind == k)
        {
            dd = dd.selected(i);
        }
        ph2d_editor_core::panel::paint_dropdown_list(ctx, &dd, chip, region, theme, None);
    }
}

/// O corpo do painel do traço: a secção Stroke inteira e, com seleção, o perfil (o *Profile* do
/// Stroke panel do Illustrator). Devolve o `y` final.
fn stroke_rows(b: &mut BodyCtx<'_>, snap: &VectorStyleSnapshot, y: f32) -> f32 {
    let y = b.stroke_body(snap, y);
    if crate::state::current_selection_count() > 0 {
        b.width_preset_row(active_profile(), y)
    } else {
        y
    }
}

/// ⭐ **O painel do traço** — solto por baixo do «Stroke ▾» (por cima, com a fila embaixo), com a
/// largura do painel docado: as linhas são as mesmas e leem-se iguais nos dois sítios.
fn paint_stroke_popover(ctx: &mut PaintCtx, theme: Theme, chip: Rect, width: f32) {
    let snap = crate::state::current_snapshot();
    let pad = PANEL_HEAD_PAD;
    let inner_w = (width - pad * 2.0).max(0.0);
    // A altura é a do corpo, medida numa passagem a seco (cena e hits descartáveis): o fundo tem de
    // ser pintado ANTES das linhas, e o corpo muda de altura com o que a seleção tem.
    let body_h = {
        let mut scene = ph2d_vector::VectorScene::new();
        let mut hits = ph2d_editor_core::interaction::HitIndex::new();
        let mut b = body_ctx(
            &mut scene,
            ctx.text_system,
            ctx.host.store(),
            &mut hits,
            theme,
            pad,
            inner_w,
        );
        stroke_rows(&mut b, &snap, 0.0)
    };
    let id = crate::ids::VECTOR_BAR_STROKE;
    let region = ctx.layout.popover_region();
    let content_h = body_h + pad * 2.0;
    let gap = Spacing::Xs.px();
    let bottom = ctx.host.store().tool_row_bottom();
    // O painel fica do lado do chip que a fila deixa e encolhe ao que sobra dele na região; o resto
    // ROLA (um ecrã baixo transbordava as linhas por cima da arte e para fora da janela).
    let room = if bottom {
        chip.y - gap - region.y
    } else {
        region.y + region.h - (chip.y + chip.h + gap)
    };
    let h = content_h.min(room.max(0.0));
    let x = chip.x.min(region.x + region.w - width).max(region.x);
    let y = if bottom {
        chip.y - gap - h
    } else {
        chip.y + chip.h + gap
    };
    let rect = Rect::new(x, y, width, h);
    let max_scroll = (content_h - h).max(0.0);
    let scroll = ctx.host.store().panel_scroll(id).clamp(0.0, max_scroll); // CLAMP-OK: 0.0 literal; max_scroll is a non-negative px extent
    paint_panel_surface(rect, ctx.scene, theme);
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    // O fundo ENGOLE o clique (o mesmo da fila): sem ele um clique entre duas linhas caía na arte.
    hit_index.register(ph2d_editor_core::ids::RAIL_BACKDROP, rect);
    // O recorte da cena e o dos hits dizem o mesmo: uma linha rolada para fora nem pinta nem clica.
    ctx.scene.push_clip(&ph2d_vector::Rect::new(
        f64::from(rect.x),
        f64::from(rect.y),
        f64::from(rect.x + rect.w),
        f64::from(rect.y + rect.h),
    ));
    hit_index.push_clip(rect);
    let mut b = body_ctx(
        ctx.scene,
        ctx.text_system,
        store,
        hit_index,
        theme,
        rect.x + pad,
        inner_w,
    );
    stroke_rows(&mut b, &snap, rect.y + pad - scroll);
    hit_index.pop_clip();
    ctx.scene.pop_layer();
    // O rect publicado é o que o light-dismiss, a roda e o polegar leem (a mesma publicação da porta
    // das listas e do popover diferido do Inspector).
    let painted = ph2d_editor_core::widget::PaintedPopover {
        id,
        panel: rect,
        content_h,
        visible_h: h,
        scroll,
    };
    ph2d_editor_core::panel::paint_popover_scrollbar(&painted, store, ctx.scene, theme, hit_index);
    ctx.host.store_mut().publish_dropdown_popover(painted);
}

fn body_ctx<'a>(
    scene: &'a mut ph2d_vector::VectorScene,
    text_system: &'a mut ph2d_text::TextSystem,
    store: &'a WidgetStore,
    hit_index: &'a mut ph2d_editor_core::interaction::HitIndex,
    theme: Theme,
    inner_x: f32,
    inner_w: f32,
) -> BodyCtx<'a> {
    BodyCtx {
        scene,
        text_system,
        store,
        hit_index,
        theme,
        inner_x,
        inner_w,
        row_h: ROW_H_PX,
        row_gap: Spacing::Xs.px(),
        chip_w: number_input_min_w_px(),
        font: TypeToken::Base.px(),
        open_fold: None,
    }
}
