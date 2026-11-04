//! ② **A BARRA DE CONTROLE** (F16, dono 15/10: *«a barra de ajustes no topo como no illustrator,
//! principalmente com os ajustes do stroke»*) — a *Control bar* do Illustrator na fila ②:
//! `nome │ Fill │ Stroke │ Stroke ▾ │ peso │ Perfil ▾ │ Tinta ▾ │ Opacity`, e o painel do traço que o
//! «Stroke ▾» abre POR BAIXO do chip (o *Stroke panel* que o Illustrator abre do «Stroke:»).
//!
//! ⚠️ **Nenhum controlo novo**: cada chip é outra porta para um id que o painel já despacha (o peso é o
//! `VECTOR_WIDTH_NUM`, a opacidade o `VECTOR_OBJ_OPACITY_NUM`, as linhas das listas os perfis e as
//! tintas), e o painel do traço pinta o MESMO corpo da secção Stroke (`BodyCtx::stroke_lines` + `stroke_settings_body`, F44).
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

/// Os chips da barra que abrem uma lista ou um painel por baixo deles.
const LISTS: [NodeId; 5] = [
    crate::ids::VECTOR_BAR_STROKE,
    crate::ids::VECTOR_BAR_PROFILE,
    crate::ids::VECTOR_BAR_KIND,
    crate::ids::VECTOR_BAR_HOLES,
    crate::ids::VECTOR_BAR_RESULT,
];

/// ⭐ As duas listas do Offset Path (F48d): `(chip, rótulo, legenda, [(linha, nome, código)])`.
type OffsetList = (NodeId, &'static str, &'static [(NodeId, &'static str, u8)]);
const HOLES: OffsetList = (
    crate::ids::VECTOR_BAR_HOLES,
    "panel.vector.expand.holes",
    &[
        (
            ids::VECTOR_EXPAND_SIDE_BOTH,
            "panel.vector.expand.holes_move",
            2,
        ),
        (
            ids::VECTOR_EXPAND_SIDE_OUTER,
            "panel.vector.expand.holes_keep",
            0,
        ),
    ],
);
const RESULT: OffsetList = (
    crate::ids::VECTOR_BAR_RESULT,
    "panel.vector.expand.result",
    &[
        (
            crate::ids::VECTOR_EXPAND_RESULT_BEHIND,
            "panel.vector.expand.behind",
            1,
        ),
        (
            crate::ids::VECTOR_EXPAND_RESULT_FRONT,
            "panel.vector.expand.front",
            2,
        ),
        (
            crate::ids::VECTOR_EXPAND_RESULT_REPLACE,
            "panel.vector.expand.replace",
            0,
        ),
    ],
);

/// O código escolhido de uma lista do Offset Path (`Holes` lê o `side`, `Result` o `result`).
fn offset_code(list: OffsetList) -> u8 {
    let k = crate::state::offset_knobs();
    if list.0 == crate::ids::VECTOR_BAR_HOLES {
        k.side
    } else {
        k.result
    }
}

/// A face (a escolha de agora) e as faces (todas) de uma lista do Offset Path.
fn offset_faces(list: OffsetList) -> (String, Vec<String>) {
    let code = offset_code(list);
    let face = list.2.iter().find(|r| r.2 == code).map_or("", |r| tr(r.1));
    (
        face.to_owned(),
        list.2.iter().map(|r| tr(r.1).to_owned()).collect(),
    )
}

/// ⭐ `Holes ▾` e `Result ▾` para a fila: a face é a escolha de agora, a largura a da mais larga.
pub(super) fn offset_lists(store: &WidgetStore) -> [ToolRailEntry; 2] {
    let (hf, hs) = offset_faces(HOLES);
    let (rf, rs) = offset_faces(RESULT);
    [
        list_chip(
            store,
            crate::ids::VECTOR_BAR_HOLES,
            HOLES.1,
            hf,
            hs,
            tr(HOLES.1),
        ),
        list_chip(
            store,
            crate::ids::VECTOR_BAR_RESULT,
            RESULT.1,
            rf,
            rs,
            tr(RESULT.1),
        ),
    ]
}

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
            .unwrap_or_else(|| ph2d_i18n::tr_with("panel.vector.bar.name.many", &[("n", &1)])),
        n => ph2d_i18n::tr_with("panel.vector.bar.name.many", &[("n", &n)]),
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
    let offset = [HOLES, RESULT]
        .into_iter()
        .find(|l| l.2.iter().any(|r| r.0 == id));
    let owner = if profile {
        crate::ids::VECTOR_BAR_PROFILE
    } else if kind {
        crate::ids::VECTOR_BAR_KIND
    } else if let Some(l) = offset {
        l.0
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
    for list in [HOLES, RESULT] {
        let store = ctx.host.store();
        let Some(chip) = is_open(store, list.0)
            .then(|| store.tool_row_slot(list.0))
            .flatten()
        else {
            continue;
        };
        let options = list
            .2
            .iter()
            .enumerate()
            .map(|(i, (id, key, _))| DropdownOption::new(*id, i, tr(key)))
            .collect();
        let code = offset_code(list);
        let mut dd = Dropdown::new(list.0, "", options).open(true);
        if let Some(i) = list.2.iter().position(|r| r.2 == code) {
            dd = dd.selected(i);
        }
        ph2d_editor_core::panel::paint_dropdown_list(ctx, &dd, chip, region, theme, None);
    }
}

/// O corpo do painel do traço: as linhas da secção Stroke e o «Stroke settings» (o perfil incluído)
/// — o mesmo conteúdo que o Inspector abre pelo ⚙ (F44). Devolve o `y` final.
fn stroke_rows(b: &mut BodyCtx<'_>, snap: &VectorStyleSnapshot, y: f32) -> f32 {
    let y = b.stroke_lines(snap, y, true);
    b.stroke_settings_body(snap, y)
}

/// ⭐ **O painel do traço** — solto por baixo do «Stroke ▾» (por cima, com a fila embaixo), com a
/// largura do painel docado: as linhas são as mesmas e leem-se iguais nos dois sítios.
fn paint_stroke_popover(ctx: &mut PaintCtx, theme: Theme, chip: Rect, width: f32) {
    let snap = crate::state::current_snapshot();
    if crate::cards::cards_on() {
        let region = ctx.layout.popover_region();
        let bottom = ctx.host.store().tool_row_bottom();
        crate::cards_lado::flutua(
            ctx,
            crate::ids::VECTOR_BAR_STROKE,
            ph2d_tool_vector::ids::VECTOR_SECTION_FILL,
            width,
            theme,
            &|conteudo| lugar_do_painel(chip, width, conteudo, region, bottom),
            None,
            &mut |c, y| {
                let fim = crate::cards::linhas_do_cartao(c).stroke_lines(&snap, y, true);
                crate::cards_traco::stroke_settings(c, &snap, fim)
            },
        );
        return;
    }
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
    let content_h = body_h + pad * 2.0;
    let rect = stroke_popover_rect(ctx, chip, width, content_h);
    let h = rect.h;
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

fn stroke_popover_rect(ctx: &PaintCtx, chip: Rect, width: f32, content_h: f32) -> Rect {
    let region = ctx.layout.popover_region();
    let bottom = ctx.host.store().tool_row_bottom();
    lugar_do_painel(chip, width, content_h, region, bottom)
}

/// O painel fica do lado do chip que a fila deixa e encolhe ao que sobra dele na região; o resto ROLA
/// (um ecrã baixo transbordava as linhas por cima da arte e para fora da janela).
fn lugar_do_painel(chip: Rect, width: f32, content_h: f32, region: Rect, bottom: bool) -> Rect {
    let gap = Spacing::Xs.px();
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
    Rect::new(x, y, width, h)
}

pub(crate) fn body_ctx<'a>(
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
