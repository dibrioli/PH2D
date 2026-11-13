//! ⭐ **O CARTÃO da chave** — o menu do botão direito numa chave da timeline, na cara da maquete do
//! dono (13/11): o título, três blocos grandes (Curve · Straight · Hold), a fileira do ease, a forma
//! da curva e, embaixo, as linhas que a maquete não tem (Nearest · Custom · Rove).
//!
//! Só PINTA e diz o que cada clique quer ([`acao`]); quem aplica é o `chrome::timeline_segment`, pela
//! mesma `TimelineInterpPick` do menu em lista — a shell não muda. O estado (o que está aceso, o
//! título) vem do painel da timeline por `WidgetStore::set_menu_marks`.

use crate::ids as c;
use crate::interaction::{ContextMenuKind, HitIndex, TimelineInterpScope, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_icon, paint_text};
use crate::widget::lista_flutuante::Superficie;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_i18n::{TextKey, tr};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, INLINE_ICON_PX, ROW_H_PX, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

const TILE_W: f32 = 76.0; // LITERAL-PX-OK: bloco da maquete do dono (13/11)
const TILE_H: f32 = 58.0; // LITERAL-PX-OK: bloco da maquete do dono (13/11)
const TILE_ICON: f32 = 22.0; // LITERAL-PX-OK: o desenho da curva no bloco, maior que o ícone de linha

/// Os três blocos: o id, o rótulo, o ícone.
const BLOCOS: [(NodeId, &str, crate::icons::IconId); 3] = [
    (
        c::CTX_MENU_TL_CURVE,
        "chrome.timeline_menu.curve",
        crate::icons::IconId::CardInterpCubic,
    ),
    (
        c::CTX_MENU_TL_LINEAR,
        "chrome.timeline_menu.straight",
        crate::icons::IconId::CardInterpLinear,
    ),
    (
        c::CTX_MENU_TL_HOLD,
        "chrome.timeline_menu.hold",
        crate::icons::IconId::CardInterpHold,
    ),
];

/// A fileira do ease: o botão, a linha do modo que o acende, o modo que ele pede.
const EASES: [(NodeId, NodeId, u8, &str); 3] = [
    (
        c::CTX_MENU_TL_CURVE_IN,
        c::CTX_MENU_TL_EASE_IN,
        c::TL_EASE_MODE_IN,
        "chrome.timeline_menu.chip_ease_in",
    ),
    (
        c::CTX_MENU_TL_CURVE_INOUT,
        c::CTX_MENU_TL_EASE_INOUT,
        c::TL_EASE_MODE_INOUT,
        "chrome.timeline_menu.chip_ease_in_out",
    ),
    (
        c::CTX_MENU_TL_CURVE_OUT,
        c::CTX_MENU_TL_EASE_OUT,
        c::TL_EASE_MODE_OUT,
        "chrome.timeline_menu.chip_ease_out",
    ),
];

/// As linhas de baixo, que a maquete não tem.
const LINHAS: [(NodeId, &str); 3] = [
    (c::CTX_MENU_TL_NEAREST, "chrome.timeline_menu.nearest"),
    (c::CTX_MENU_TL_CUSTOM, "chrome.timeline_menu.custom_b_zier"),
    (c::CTX_MENU_TL_ROVE, "chrome.timeline_menu.rove_across_time"),
];

/// O escopo pinta-se como cartão (uma chave ou uma coluna do Summary; o fade continua em lista).
#[must_use]
pub fn e_cartao(kind: ContextMenuKind) -> bool {
    matches!(
        kind,
        ContextMenuKind::TimelineSegment {
            scope: TimelineInterpScope::Key { .. } | TimelineInterpScope::Column { .. }
        }
    )
}

/// O que um clique no cartão pede.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Acao {
    /// Nada (o fundo): o cartão fica aberto.
    Nada,
    /// A mesma escolha do menu em lista: `(item, mode)` da `TimelineInterpPick`.
    Escolhe { item: NodeId, mode: u8 },
    /// Abre a lista das formas, no modo dado.
    AbreFormas { mode: u8 },
}

/// A forma acesa (ou `Cubic`) e o modo aceso (ou In-Out): o que Curve e o ease mantêm.
fn forma_e_modo(store: &WidgetStore, kind: ContextMenuKind) -> (NodeId, u8) {
    let forma = c::TIMELINE_EASE_MENU
        .iter()
        .map(|&(id, ..)| id)
        .find(|&id| store.menu_row_is_marked(kind, id))
        .unwrap_or(c::CTX_MENU_TL_FAM_CUBIC);
    let modo = EASES
        .iter()
        .find(|e| store.menu_row_is_marked(kind, e.1))
        .map_or(c::TL_EASE_MODE_INOUT, |e| e.2);
    (forma, modo)
}

/// O que o clique em `id` pede, se `id` é do cartão. `None` = não é do cartão (as linhas de baixo,
/// Straight e Hold seguem pelo caminho do menu em lista).
#[must_use]
pub fn acao(id: NodeId, kind: ContextMenuKind, store: &WidgetStore) -> Option<Acao> {
    if !e_cartao(kind) {
        return None;
    }
    let (forma, modo) = forma_e_modo(store, kind);
    if id == c::CTX_MENU_TL_KEY_CARD {
        return Some(Acao::Nada);
    }
    if id == c::CTX_MENU_TL_CURVE {
        return Some(Acao::Escolhe {
            item: forma,
            mode: modo,
        });
    }
    if id == c::CTX_MENU_TL_SHAPE {
        return Some(Acao::AbreFormas { mode: modo });
    }
    EASES.iter().find(|e| e.0 == id).map(|e| Acao::Escolhe {
        item: forma,
        mode: e.2,
    })
}

/// Os rectângulos do cartão ancorado em `(x, y)` — uma conta só para o desenho e o teste.
pub(super) struct Geo {
    pub cartao: Rect,
    pub titulo: Rect,
    pub blocos: [Rect; 3],
    pub eases: [Rect; 3],
    pub forma_rotulo: Rect,
    pub forma: Rect,
    pub risco_y: f32,
    pub linhas: [Rect; 3],
}

pub(super) fn geo(x: f32, y: f32) -> Geo {
    let pad = Spacing::Md.px();
    let gap = Spacing::Sm.px();
    let row = ROW_H_PX;
    let w = pad * 2.0 + TILE_W * 3.0 + gap * 2.0;
    let iw = w - pad * 2.0;
    let x0 = x + pad;
    let titulo = Rect::new(x0, y + pad * 0.5, iw, row);
    let ty = titulo.y + row + gap * 0.5;
    let blocos =
        std::array::from_fn(|i| Rect::new(x0 + (TILE_W + gap) * i as f32, ty, TILE_W, TILE_H));
    let ey = ty + TILE_H + gap;
    let ew = (iw - gap * 2.0) / 3.0;
    let eases = std::array::from_fn(|i| Rect::new(x0 + (ew + gap) * i as f32, ey, ew, row));
    let fy = ey + row + gap;
    let forma = Rect::new(x0 + ew + gap, fy, ew * 2.0 + gap, row);
    let forma_rotulo = Rect::new(x0, fy, ew, row);
    let risco_y = fy + row + gap;
    let ly = risco_y + gap;
    let lx = x + Spacing::Xs.px();
    let lw = w - Spacing::Xs.px() * 2.0;
    let linhas = std::array::from_fn(|i| Rect::new(lx, ly + row * i as f32, lw, row));
    let h = ly + row * 3.0 + Spacing::Xs.px() - y;
    Geo {
        cartao: Rect::new(x, y, w, h),
        titulo,
        blocos,
        eases,
        forma_rotulo,
        forma,
        risco_y,
        linhas,
    }
}

/// O bloco/botão do cartão: o escolhido no azul calmo da maquete (`TimelineTinta::BlocoEscolhido`),
/// os outros como todo botão de cartão. Devolve a tinta do texto.
fn botao(
    sup: Superficie,
    theme: Theme,
    scene: &mut VectorScene,
    r: Rect,
    on: bool,
    hover: bool,
) -> ph2d_vector::Color {
    let tinta = sup.botao(scene, r, false, hover);
    let cor = |t: ph2d_tokens::TimelineTinta| crate::paint::token_to_vello(t.resolve(theme));
    if !on {
        // Na maquete o texto do não escolhido é cinzento; sob o rato acende.
        return if hover || sup.tinta().is_none() {
            tinta
        } else {
            cor(ph2d_tokens::TimelineTinta::TextoPropriedade)
        };
    }
    fill_rounded_rect(
        scene,
        r,
        ph2d_card_stack::metrics::FIELD_RADIUS,
        cor(ph2d_tokens::TimelineTinta::BlocoEscolhido),
    );
    cor(ph2d_tokens::TimelineTinta::TextoCabecalho)
}

/// Texto centrado em `r`.
fn centrado(
    ts: &mut TextSystem,
    scene: &mut VectorScene,
    s: &str,
    r: Rect,
    y: f32,
    cor: ph2d_vector::Color,
) {
    // Peso NORMAL, o da maquete (o app pinta rótulos em MEDIUM).
    let px = TypeToken::Sm.px();
    let peso = ph2d_text::FontWeight::NORMAL;
    let tw = ts.prefix_width_weighted(s, px, peso).min(r.w);
    crate::paint::paint_text_in_weight(
        ts,
        scene,
        s,
        [r.x + (r.w - tw) * 0.5, y],
        px,
        r.w,
        cor,
        peso,
    );
}

/// Pinta o cartão aberto e regista os alvos.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    hit: &mut HitIndex,
    store: &WidgetStore,
    req: crate::interaction::ContextMenuRequest,
    viewport: Rect,
) {
    let kind = req.kind;
    let g0 = geo(0.0, 0.0);
    let r = crate::widget::panel_chrome::clamp_menu_to_viewport(
        req.x,
        super::context_menu_overlay::anchor_y(store, req, g0.cartao.h),
        g0.cartao.w,
        g0.cartao.h,
        viewport,
    );
    let g = geo(r.x, r.y);
    let sup = Superficie::de(theme);
    sup.moldura(scene, g.cartao);
    hit.register(c::CTX_MENU_TL_KEY_CARD, g.cartao);
    let px = TypeToken::Sm.px();
    let ty = |r: Rect| r.y + (r.h - px) * 0.5;
    let marcado = |id| store.menu_row_is_marked(kind, id);
    let hot = |id| store.hot_id() == Some(id);

    // O título um degrau acima e mais forte, como na maquete.
    let tpx = TypeToken::Base.px();
    crate::paint::paint_text_title(
        ts,
        scene,
        store.menu_title(kind),
        g.titulo.x,
        g.titulo.y + (g.titulo.h - tpx) * 0.5,
        tpx,
        g.titulo.w,
        sup.cor(ColorToken::Text1),
    );

    let curva = EASES.iter().any(|e| marcado(e.1));
    for (i, &(id, chave, icone)) in BLOCOS.iter().enumerate() {
        let b = g.blocos[i];
        let on = if id == c::CTX_MENU_TL_CURVE {
            curva
        } else {
            marcado(id)
        };
        let tinta = botao(sup, theme, scene, b, on, hot(id));
        hit.register(id, b);
        let ix = b.x + (b.w - TILE_ICON) * 0.5;
        let iy = b.y + Spacing::Sm.px();
        paint_icon(
            scene,
            icone,
            Rect::new(ix, iy, TILE_ICON, TILE_ICON),
            tinta,
            StrokeToken::Default.px(),
        );
        centrado(
            ts,
            scene,
            tr(chave),
            b,
            b.y + b.h - px - Spacing::Sm.px(),
            tinta,
        );
    }
    for (i, &(id, linha, _, chave)) in EASES.iter().enumerate() {
        let b = g.eases[i];
        let tinta = botao(sup, theme, scene, b, marcado(linha), hot(id));
        hit.register(id, b);
        centrado(ts, scene, tr(chave), b, ty(b), tinta);
    }
    // A forma da curva: o rótulo à esquerda, o botão com o nome à direita (abre a lista).
    paint_text(
        ts,
        scene,
        tr("chrome.timeline_menu.shape"),
        g.forma_rotulo.x,
        ty(g.forma_rotulo),
        px,
        g.forma_rotulo.w,
        sup.cor(ColorToken::Text3),
    );
    let (forma, _) = forma_e_modo(store, kind);
    let nome = c::TIMELINE_EASE_MENU
        .iter()
        .find(|row| row.0 == forma)
        .map_or("", |row: &(NodeId, TextKey, _)| row.1.tr());
    let tinta = sup.botao(scene, g.forma, false, hot(c::CTX_MENU_TL_SHAPE));
    hit.register(c::CTX_MENU_TL_SHAPE, g.forma);
    let ic = INLINE_ICON_PX;
    let pad_x = Spacing::Sm.px();
    paint_text(
        ts,
        scene,
        nome,
        g.forma.x + pad_x,
        ty(g.forma),
        px,
        g.forma.w - ic - pad_x * 2.0,
        tinta,
    );
    paint_icon(
        scene,
        crate::icons::IconId::ChevronDown,
        Rect::new(
            g.forma.x + g.forma.w - ic - pad_x,
            g.forma.y + (g.forma.h - ic) * 0.5,
            ic,
            ic,
        ),
        tinta,
        StrokeToken::Default.px(),
    );

    fill_rounded_rect(
        scene,
        Rect::new(g.cartao.x, g.risco_y, g.cartao.w, 1.0),
        0.0,
        sup.cor(ColorToken::Border),
    );
    for (i, &(id, chave)) in LINHAS.iter().enumerate() {
        let l = g.linhas[i];
        hit.register(id, l);
        if hot(id) {
            sup.realce(scene, l);
        }
        let on = marcado(id);
        let x = l.x + Spacing::Md.px();
        if on {
            let dot = ph2d_vector::Circle::new(
                ph2d_vector::Point::new(f64::from(x), f64::from(l.y + l.h * 0.5)),
                3.0, // LITERAL-PX-OK: a bolinha do estado corrente, a mesma do menu em lista
            );
            scene.inner_mut().fill(
                ph2d_vector::Fill::NonZero,
                ph2d_vector::Affine::IDENTITY,
                &ph2d_vector::Brush::Solid(sup.cor(ColorToken::Accent)),
                None,
                &dot,
            );
        }
        let gx = x + Spacing::Sm.px() + Spacing::Xs.px();
        if let Some(icone) = super::menu_row_look::icone(id) {
            let cor = sup.cor(if on {
                ColorToken::Accent
            } else {
                ColorToken::Text2
            });
            paint_icon(
                scene,
                icone,
                Rect::new(gx, l.y + (l.h - ic) * 0.5, ic, ic),
                cor,
                StrokeToken::Default.px(),
            );
        }
        let tx = gx + ic + ph2d_tokens::icon_label_gap_px();
        let cor = sup.cor(if on {
            ColorToken::Text1
        } else {
            ColorToken::Text2
        });
        paint_text(
            ts,
            scene,
            tr(chave),
            tx,
            ty(l),
            px,
            (l.x + l.w - tx - Spacing::Md.px()).max(0.0),
            cor,
        );
    }
}

#[cfg(test)]
#[path = "key_card_tests.rs"]
mod tests;
