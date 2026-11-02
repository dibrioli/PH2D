//! ⭐⭐ **O painel de propriedades à ESQUERDA do Inspector e a lista do «+»** (F43, dono 01/11:
//! *«Propriedade dos efeitos abre na lateral esquerda»*, referência `img_F43/` do Figma).
//!
//! Pintados no passe DIFERIDO do painel (por cima das secções e fora do recorte do scroll), a
//! partir das âncoras que [`crate::paint_sections::fx_list`] deixou ao pintar a linha aberta e o «+»
//! aberto. O mecanismo é o do painel do traço da barra ([`crate::bars`]): um `Dropdown` aberto que
//! publica o rect do popover — o light-dismiss, a roda e o polegar vêm de graça.
//!
//! ```text
//!  ┌──────────────────────────┐ ┌ Inspector ─────────────┐
//!  │ ▣ Zig Zag       ↑  ↓  ✕  │ │ Effects            +   │
//!  │ Size    ───●────  42.00  │ │ [▣ Zig Zag ]  👁  −    │
//!  │ Ridges  ─●──────   8.00  │ │ [▣ Wave    ]  👁  −    │
//!  └──────────────────────────┘ └────────────────────────┘
//! ```

use crate::paint_sections::BodyCtx;
use crate::paint_sections::fx_list::{Pilha, is_open, take_pendentes};
use crate::state;
use crate::state::filters as fst;
use ph2d_a11y::NodeId;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{paint_icon, paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::panel_chrome::{PANEL_HEAD_PAD, paint_panel_surface};
use ph2d_editor_core::widget::{Dropdown, DropdownOption, PopoverRow};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Spacing, StrokeToken, Theme, TypeToken};

/// O passe diferido: a lista do «+» aberto e o painel da linha aberta, por pilha. Uma linha aberta
/// que o corpo não pintou neste quadro (secção dobrada, seleção vazia) FECHA — senão o painel
/// voltava sozinho ao reaparecer a linha.
pub(crate) fn paint(ctx: &mut PaintCtx, panel: Rect, theme: Theme) {
    let pend = take_pendentes();
    for (i, p) in Pilha::TODAS.into_iter().enumerate() {
        if let Some(chip) = pend.menu[i] {
            paint_menu(ctx, p, chip, theme);
        }
        match pend.props[i] {
            Some((row, line)) => paint_props(ctx, p, row, line, panel, theme),
            None => close_rows(ctx.host.store_mut(), p),
        }
    }
}

/// A lista do «+»: um tipo por linha, com o glifo dele — as linhas são os `Add <tipo>` de sempre.
fn paint_menu(ctx: &mut PaintCtx, p: Pilha, chip: Rect, theme: Theme) {
    let options: Vec<DropdownOption<usize>> = match p {
        Pilha::Efeitos => state::kinds()
            .iter()
            .enumerate()
            .take(p.max_kinds())
            .map(|(k, name)| DropdownOption::new(p.add(k), k, *name).with_icon(state::kind_icon(k)))
            .collect(),
        Pilha::Filtros => fst::kinds()
            .iter()
            .enumerate()
            .take(p.max_kinds())
            .map(|(k, spec)| {
                DropdownOption::new(p.add(k), k, spec.name).with_icon(fst::kind_icon(k))
            })
            .collect(),
    };
    let glifos: Vec<IconId> = options.iter().filter_map(|o| o.icon).collect();
    let dd = Dropdown::new(p.add_menu(), "", options).open(true);
    let region = ctx.layout.popover_region();
    let rotulos: Vec<String> = dd.options.iter().map(|o| o.label.clone()).collect();
    let mut linha = |scene: &mut ph2d_vector::VectorScene,
                     text: &mut ph2d_text::TextSystem,
                     row: PopoverRow| {
        paint_menu_row(scene, text, row, &glifos, &rotulos, theme);
    };
    ph2d_editor_core::panel::paint_dropdown_list(ctx, &dd, chip, region, theme, Some(&mut linha));
}

/// Uma linha da lista do «+»: o GLIFO do tipo e o nome alinhado à esquerda (a lista do Figma).
fn paint_menu_row(
    scene: &mut ph2d_vector::VectorScene,
    text_system: &mut ph2d_text::TextSystem,
    row: PopoverRow,
    glifos: &[IconId],
    rotulos: &[String],
    theme: Theme,
) {
    let r = row.rect;
    let ink = resolve(row.ink(), theme);
    let pad = Spacing::Md.px();
    let font = TypeToken::Base.px();
    let g = ph2d_editor_core::paint::icon_side_for_glyph(font).min(r.h);
    let gr = Rect::new(r.x + pad, r.y + (r.h - g) * 0.5, g, g);
    if let Some(&glyph) = glifos.get(row.index) {
        paint_icon(scene, glyph, gr, ink, StrokeToken::Default.px());
    }
    let tx = gr.x + gr.w + ph2d_tokens::icon_label_gap_px();
    if let Some(label) = rotulos.get(row.index) {
        paint_text(
            text_system,
            scene,
            label,
            tx,
            r.y + (r.h - font) * 0.5,
            font,
            (r.x + r.w - pad - tx).max(1.0),
            ink,
        );
    }
}

/// O nome e o glifo da linha `row`, se ela existe na pilha publicada.
fn head_of(p: Pilha, row: usize) -> Option<(IconId, &'static str)> {
    match p {
        Pilha::Efeitos => state::stack()
            .get(row)
            .map(|fx| (state::row_icon(row), fx.label)),
        Pilha::Filtros => {
            let fx = fst::stack().get(row).cloned()?;
            let spec = fst::kind_spec(fx.kind)?;
            Some((fst::kind_icon(usize::from(fx.kind)), spec.name))
        }
    }
}

/// ⭐ **O cabeçalho de um painel à esquerda**: o glifo (se há), o nome, e da DIREITA ✕ · ↓ · ↑ (as
/// setas só onde mexem — um ícone inerte ensina a desconfiar). Devolve o `y` dos campos.
pub(crate) fn side_head(
    b: &mut BodyCtx<'_>,
    glyph: Option<IconId>,
    name: &str,
    close: NodeId,
    down: Option<NodeId>,
    up: Option<NodeId>,
    y: f32,
) -> f32 {
    let h = b.row_h;
    let ink = resolve(ColorToken::Text1, b.theme);
    let mut tx = b.inner_x;
    if let Some(glyph) = glyph {
        let g = ph2d_editor_core::paint::icon_side_for_glyph(b.font).min(h);
        paint_icon(
            b.scene,
            glyph,
            Rect::new(b.inner_x, y + (h - g) * 0.5, g, g),
            ink,
            StrokeToken::Default.px(),
        );
        tx += g + ph2d_tokens::icon_label_gap_px();
    }
    paint_text(
        b.text_system,
        b.scene,
        name,
        tx,
        y + (h - TypeToken::Md.px()) * 0.5,
        TypeToken::Md.px(),
        (b.inner_x + b.inner_w - tx).max(1.0),
        ink,
    );
    let gap = Spacing::Xs.px();
    let slot = |i: usize| {
        #[allow(clippy::cast_precision_loss)]
        let i = i as f32;
        b.inner_x + b.inner_w - h * (i + 1.0) - gap * i
    };
    let (x0, x1, x2) = (slot(0), slot(1), slot(2));
    b.filter_icon(close, IconId::Close, x0, y, h);
    if let Some(id) = down {
        b.filter_icon(id, IconId::ChevronDown, x1, y, h);
    }
    if let Some(id) = up {
        b.filter_icon(id, IconId::ChevronUp, x2, y, h);
    }
    y + h + ph2d_tokens::control_gap_px()
}

/// O corpo do painel: o cabeçalho (glifo, nome, ↑ ↓ ✕) e os campos do tipo. Devolve o `y` final.
fn body(b: &mut BodyCtx<'_>, p: Pilha, row: usize, glyph: IconId, name: &str, y: f32) -> f32 {
    let total = p.len();
    let down = (row + 1 < total).then(|| p.down(row));
    let up = (row > 0).then(|| p.up(row));
    let y = side_head(b, Some(glyph), name, p.close(), down, up, y);
    match p {
        Pilha::Efeitos => match state::stack().get(row) {
            Some(fx) => b.effect_props(row, fx, y),
            None => y,
        },
        Pilha::Filtros => {
            let Some(fx) = fst::stack().get(row).cloned() else {
                return y;
            };
            match fst::kind_spec(fx.kind) {
                Some(spec) => b.filter_props(row, &fx, &spec, y),
                None => y,
            }
        }
    }
}

/// O painel da linha aberta de uma pilha (Effects/Filters).
fn paint_props(ctx: &mut PaintCtx, p: Pilha, row: usize, line: Rect, panel: Rect, theme: Theme) {
    let Some((glyph, name)) = head_of(p, row) else {
        close_rows(ctx.host.store_mut(), p);
        return;
    };
    side_panel(ctx, p.row(row), line, panel, theme, &mut |b, y| {
        body(b, p, row, glyph, name, y)
    });
    if p == Pilha::Filtros {
        crate::paint::register_ramp_handles(ctx.host.store_mut(), row);
    }
}

/// ⭐ **Um painel à esquerda do Inspector**, no topo alinhado à `line` que o abriu (e dentro da
/// região: encolhe e ROLA num ecrã baixo). Com o Inspector docado à esquerda, abre à direita dele.
/// `id` é o `Dropdown` que o mantém aberto; `body` pinta os campos a partir do `y` e devolve o fim.
/// A porta de F43 (Effects/Filters) e de F44 (Fill, camadas, «Stroke settings»).
pub(crate) fn side_panel(
    ctx: &mut PaintCtx,
    id: NodeId,
    line: Rect,
    panel: Rect,
    theme: Theme,
    body: &mut dyn FnMut(&mut BodyCtx<'_>, f32) -> f32,
) {
    let pad = PANEL_HEAD_PAD;
    let width = panel.w;
    let inner_w = (width - pad * 2.0).max(0.0);
    // A altura medida numa passagem a seco (cena e hits descartáveis): o fundo pinta-se ANTES.
    let body_h = {
        let mut scene = ph2d_vector::VectorScene::new();
        let mut hits = ph2d_editor_core::interaction::HitIndex::new();
        let mut b = crate::bars::body_ctx(
            &mut scene,
            ctx.text_system,
            ctx.host.store(),
            &mut hits,
            theme,
            pad,
            inner_w,
        );
        body(&mut b, 0.0)
    };
    let region = ctx.layout.popover_region();
    let gap = Spacing::Sm.px();
    let content_h = body_h + pad * 2.0;
    let h = content_h.min(region.h);
    let left = panel.x - gap - width;
    let x = if left >= region.x {
        left
    } else {
        panel.x + panel.w + gap
    };
    let y = line.y.min(region.y + region.h - h).max(region.y);
    let rect = Rect::new(x, y, width, h);
    let max_scroll = (content_h - h).max(0.0);
    let scroll = ctx.host.store().panel_scroll(id).clamp(0.0, max_scroll); // CLAMP-OK: 0.0 literal; max_scroll is a non-negative px extent
    paint_panel_surface(rect, ctx.scene, theme);
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    // O fundo ENGOLE o clique: sem ele um clique entre dois campos caía na arte por baixo.
    hit_index.register(ph2d_editor_core::ids::RAIL_BACKDROP, rect);
    ctx.scene.push_clip(&ph2d_vector::Rect::new(
        f64::from(rect.x),
        f64::from(rect.y),
        f64::from(rect.x + rect.w),
        f64::from(rect.y + rect.h),
    ));
    hit_index.push_clip(rect);
    let mut b = crate::bars::body_ctx(
        ctx.scene,
        ctx.text_system,
        store,
        hit_index,
        theme,
        rect.x + pad,
        inner_w,
    );
    body(&mut b, rect.y + pad - scroll);
    hit_index.pop_clip();
    ctx.scene.pop_layer();
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

pub(crate) fn set_open(store: &mut WidgetStore, id: NodeId, to: bool) {
    if let Some(InteractiveState::Dropdown { open, .. }) = store.get_mut(id) {
        *open = to;
    }
}

fn close_rows(store: &mut WidgetStore, p: Pilha) {
    for r in 0..p.max_rows() {
        set_open(store, p.row(r), false);
    }
}

/// ⭐ **Os cliques que mexem no que está aberto**, antes do verbo do painel (que corre a seguir,
/// salvo no X). `true` = consumido.
///
/// - X → fecha o painel; uma linha da lista do «+» → fecha a lista (o light-dismiss não dispara
///   num clique DENTRO dela) e o `Add <tipo>` segue;
/// - «−» → fecha o painel (a linha aberta passa a ser outra coisa);
/// - ↑ / ↓ da linha aberta → o painel ACOMPANHA o efeito para a linha nova.
pub(crate) fn on_click(store: &mut WidgetStore, id: NodeId) -> bool {
    // O X de cada painel, pelo id (o que a régua do consumidor lê); é o mesmo do `Pilha::close`.
    for (x, p) in [
        (ph2d_tool_vector::ids::VECTOR_FX_PROPS_CLOSE, Pilha::Efeitos),
        (crate::ids::FILTER_PROPS_CLOSE, Pilha::Filtros),
    ] {
        debug_assert_eq!(x, p.close());
        if id == x {
            close_rows(store, p);
            return true;
        }
    }
    for p in Pilha::TODAS {
        if (0..p.max_kinds()).any(|k| id == p.add(k)) {
            set_open(store, p.add_menu(), false);
            return false;
        }
        for r in 0..p.max_rows() {
            if id == p.remove(r) {
                close_rows(store, p);
                return false;
            }
            let to = if id == p.up(r) && r > 0 {
                Some(r - 1)
            } else if id == p.down(r) && r + 1 < p.max_rows() {
                Some(r + 1)
            } else {
                None
            };
            if let Some(to) = to {
                if is_open(store, p.row(r)) {
                    set_open(store, p.row(r), false);
                    set_open(store, p.row(to), true);
                }
                return false;
            }
        }
    }
    false
}
