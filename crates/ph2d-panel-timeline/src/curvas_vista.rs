//! **«Keys | Curves»** (doc 17, item 5; maquetes B1/B2) — a vista da área das faixas na aba Keys.
//!
//! Em Curves a área inteira vira UM grafo: a linha escolhida (a da chave escolhida, senão a primeira
//! com chaves) pinta-se pelo MESMO [`crate::graph_paint::paint_track`] do grafo por linha — eixo
//! esquerdo na unidade dela, âncoras e alças com os mesmos gestos e intents; as outras linhas com chaves
//! e olho aberto só a curva, tracejada e normalizada 0–1 pelo próprio intervalo, no eixo direito.
//! Estado de sessão: não grava, não desfaz; mudar de aba volta a Keys; `PH2D_CARDS=0` nunca a liga.

use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{paint_text, rect_to_vello, resolve, stroke_polyline};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::Seg;
use ph2d_editor_core::widget::ButtonState;
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::{TimelineViewSnapshot, TrackView, drawn_extent, sample_keys};
use ph2d_tokens::{Spacing, Theme, TimelineTinta, TypeToken};

use crate::graph::{Band, TimeView};
use crate::ids;
use crate::state::TimelinePanelState;

/// Amostragem da polilinha, a mesma densidade do grafo por linha.
const PASSO_PX: f32 = 2.0; // LITERAL-PX-OK: curve polyline sample spacing
/// A curva secundária: 1,5 px, traço 4/4 a 60 % (maquete B2).
const SEC_W: f32 = 1.5; // LITERAL-PX-OK: secondary curve stroke width
const TRACO_PX: f32 = 4.0; // LITERAL-PX-OK: secondary curve dash length (B2: 4/4)
const SEC_ALFA: f32 = 0.6; // LITERAL-PX-OK: secondary curve opacity (B2: 60 %)
const EIXO_ALFA: f32 = 0.8; // LITERAL-PX-OK: right-axis label opacity (B2: 80 %)

/// A vista da área das faixas na aba Keys.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Vista {
    /// As faixas, cada uma com as suas chaves (e o grafo por linha do twirl).
    #[default]
    Keys,
    /// Um grafo grande com as curvas de todas as linhas com chaves.
    Curves,
}

/// A vista Curves está a pintar: nos cartões, na aba Keys, ligada.
pub(crate) fn ativa(state: &TimelinePanelState) -> bool {
    state.vista == Vista::Curves
        && state.tab == crate::tab::Tab::Keys
        && ph2d_editor_core::panel::cards::ferramenta::cards_on()
}

/// As duas células do segmentado da barra.
pub(crate) fn segs(curvas: bool) -> [Seg<'static>; 2] {
    [
        Seg {
            id: ids::TIMELINE_VIEW_KEYS,
            icon: None,
            text: ph2d_i18n::tr("panel.timeline.view.keys"),
            on: !curvas,
        },
        Seg {
            id: ids::TIMELINE_VIEW_CURVES,
            icon: None,
            text: ph2d_i18n::tr("panel.timeline.view.curves"),
            on: curvas,
        },
    ]
}

/// O clique numa célula da vista ou num olho. `true` = era nosso.
pub(crate) fn route(state: &mut TimelinePanelState, id: ph2d_a11y::NodeId) -> bool {
    let want = if id == ids::TIMELINE_VIEW_KEYS {
        Vista::Keys
    } else if id == ids::TIMELINE_VIEW_CURVES {
        Vista::Curves
    } else {
        let snap = crate::state::current_snapshot();
        let Some(t) = snap
            .tracks
            .iter()
            .map(|t| t.target.get())
            .find(|&t| ids::timeline_curve_eye_id(t) == id)
        else {
            return false;
        };
        match state.curvas_escondidas.iter().position(|&x| x == t) {
            Some(i) => {
                state.curvas_escondidas.remove(i);
            }
            None => state.curvas_escondidas.push(t),
        }
        return true;
    };
    if state.vista != want {
        state.vista = want;
        state.curvas_do_clip = Some(crate::state::current_snapshot().active_clip);
        // A área muda por baixo de qualquer gesto: o mesmo fecho que mudar de aba.
        crate::state::drop_row_gestures(state);
    }
    true
}

/// ⭐ A vista volta a Keys quando a área deixa de ser a do clipe em que Curves se ligou: outro clipe
/// ou a lista de Containers (mudar de aba é [`crate::state::set_tab`]). Chamado pelo `paint`.
pub(crate) fn volta_a_keys(state: &mut TimelinePanelState, clip: usize, lista: bool) {
    if state.vista == Vista::Curves && (lista || state.curvas_do_clip != Some(clip)) {
        state.vista = Vista::Keys;
        crate::state::drop_row_gestures(state);
    }
}

/// A curva desta linha aparece no grafo grande?
fn visivel(state: &TimelinePanelState, t: &TrackView) -> bool {
    !t.keys.is_empty() && !t.prop.nomeia() && !state.curvas_escondidas.contains(&t.target.get())
}

/// A linha cuja curva manda no eixo esquerdo e se edita: a da chave escolhida, senão a primeira.
pub(crate) fn principal(state: &TimelinePanelState, snap: &TimelineViewSnapshot) -> Option<usize> {
    let vis = |i: &usize| visivel(state, &snap.tracks[*i]);
    (0..snap.tracks.len())
        .filter(vis)
        .find(|&i| snap.tracks[i].keys.iter().any(|k| k.selected))
        .or_else(|| (0..snap.tracks.len()).find(vis))
}

/// O olho da linha `target` na coluna de nomes, no lugar do ícone da propriedade.
pub(crate) fn paint_olho(
    ctx: &mut PaintCtx,
    theme: Theme,
    state: &TimelinePanelState,
    target: u64,
    (x, y): (f32, f32),
) {
    let aberto = !state.curvas_escondidas.contains(&target);
    let (ic, tinta) = if aberto {
        (IconId::Eye, TimelineTinta::TextoValor)
    } else {
        (IconId::EyeClosed, TimelineTinta::OlhoMudo)
    };
    crate::linhas::paint_icone(ctx, ic, (x, y), crate::folha_tinta::tinta(tinta, theme));
    let id = ids::timeline_curve_eye_id(target);
    let hit = Rect::new(
        x,
        y,
        crate::linhas::icone_w(Some(ic)),
        ph2d_tokens::ROW_H_PX,
    );
    ctx.host.store_mut().register(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    ctx.host
        .store_mut()
        .set_tooltip(id, ph2d_i18n::tr("panel.timeline.view.eye_tip"));
    ctx.host.hit_index_mut().register(id, hit);
}

/// O grafo grande sobre `area` (a área de tempo das faixas).
pub(crate) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    state: &mut TimelinePanelState,
    area: Rect,
    view: TimeView,
    snap: &TimelineViewSnapshot,
) {
    let fonte = TypeToken::Xs.px();
    let legenda_h = fonte + Spacing::Sm.px();
    let grafo = Rect::new(
        area.x,
        area.y + Spacing::Sm.px(),
        area.w,
        (area.h - Spacing::Sm.px() - legenda_h).max(0.0),
    );
    let Some(p) = principal(state, snap) else {
        return;
    };
    // O eixo direito: a faixa interior que a moldura do grafo por linha pinta.
    let banda = crate::graph_paint::banda_de(grafo, view);
    // A principal pelo pintor do grafo por linha (moldura, gestos e intents incluídos); as outras por
    // cima da moldura dela, a 60 % e tracejadas.
    let label_w = Spacing::Xl.px() * 2.0;
    let rect = Rect::new(
        view.time_x,
        grafo.y,
        grafo.w - (view.time_x - grafo.x),
        grafo.h,
    );
    crate::graph_paint::paint_track(
        ctx,
        theme,
        state,
        rect,
        label_w,
        view,
        &snap.tracks[p],
        &snap.values,
    );
    let cor = crate::folha_tinta::tinta(TimelineTinta::CurvaSecundaria, theme);
    ctx.scene.push_clip(&rect_to_vello(banda));
    let mut secundaria = None;
    for (i, t) in snap.tracks.iter().enumerate() {
        if i == p || !visivel(state, t) {
            continue;
        }
        let pontos = curva_normalizada(&banda, view, t);
        tracejada(ctx, &pontos, cor.multiply_alpha(SEC_ALFA));
        secundaria.get_or_insert(i);
    }
    ctx.scene.pop_layer();
    if secundaria.is_some() {
        let cor_eixo = cor.multiply_alpha(EIXO_ALFA);
        let x = banda.x + banda.w - Spacing::Md.px();
        let digito = ph2d_editor_core::interaction::util::format_number;
        for (v, y) in [(1.0, banda.y), (0.0, banda.y + banda.h - fonte)] {
            paint_text(
                ctx.text_system,
                ctx.scene,
                &digito(v),
                x,
                y,
                fonte,
                Spacing::Md.px(),
                cor_eixo,
            );
        }
    }
    legenda(
        ctx,
        theme,
        snap,
        (p, secundaria),
        (area.x, grafo.y + grafo.h + Spacing::Xs.px()),
    );
}

/// A curva de `t` normalizada pelo próprio intervalo visível: o mínimo toca o fundo, o máximo o topo.
pub(crate) fn curva_normalizada(banda: &Rect, view: TimeView, t: &TrackView) -> Vec<(f32, f32)> {
    let (l, r) = (banda.x, banda.x + banda.w);
    let n = ((r - l) / PASSO_PX).ceil() as usize + 1;
    let Some((lo, hi)) = drawn_extent(&t.keys, view.t(l), view.t(r), n) else {
        return Vec::new();
    };
    let b = if hi > lo {
        Band::from_range(*banda, f64::from(lo), f64::from(hi))
    } else {
        Band::fit(*banda, Some((lo, hi)))
    };
    (0..n)
        .filter_map(|i| {
            let x = (l + i as f32 * PASSO_PX).min(r);
            sample_keys(&t.keys, view.t(x)).map(|v| (x, b.y(f64::from(v))))
        })
        .collect()
}

/// Uma polilinha tracejada `TRACO_PX` cheio / `TRACO_PX` vazio, medida ao longo do traço.
fn tracejada(ctx: &mut PaintCtx, pontos: &[(f32, f32)], cor: ph2d_vector::Color) {
    let mut andado = 0.0_f32;
    let mut pedaco: Vec<(f32, f32)> = Vec::new();
    for w in pontos.windows(2) {
        let (a, b) = (w[0], w[1]);
        let cheio = ((andado / TRACO_PX) as u32).is_multiple_of(2);
        if cheio {
            if pedaco.is_empty() {
                pedaco.push(a);
            }
            pedaco.push(b);
        } else if pedaco.len() > 1 {
            stroke_polyline(ctx.scene, &pedaco, SEC_W, cor);
            pedaco.clear();
        } else {
            pedaco.clear();
        }
        andado += (b.0 - a.0).hypot(b.1 - a.1);
    }
    if pedaco.len() > 1 {
        stroke_polyline(ctx.scene, &pedaco, SEC_W, cor);
    }
}

/// A legenda por baixo do grafo: a principal pelo nome, a secundária «· normalized, right axis».
fn legenda(
    ctx: &mut PaintCtx,
    theme: Theme,
    snap: &TimelineViewSnapshot,
    (p, s): (usize, Option<usize>),
    (mut x, y): (f32, f32),
) {
    let fonte = TypeToken::Xs.px();
    let u = crate::tracks_value::Unidades::of(ctx.host.project());
    let texto = crate::folha_tinta::tinta(TimelineTinta::TextoPropriedade, theme);
    let mut item = |ctx: &mut PaintCtx, nome: String, amostra_cor: ph2d_vector::Color| {
        let amostra = Rect::new(x, y + fonte * 0.5 - 1.0, Spacing::Lg.px(), 2.0); // LITERAL-PX-OK: legend swatch
        ph2d_editor_core::paint::fill_rounded_rect(
            ctx.scene,
            amostra,
            1.0, // LITERAL-PX-OK: legend swatch radius
            amostra_cor,
        );
        x += amostra.w + Spacing::Xs.px();
        let w = ctx.text_system.prefix_width(&nome, fonte).ceil();
        paint_text(
            ctx.text_system,
            ctx.scene,
            &nome,
            x,
            y,
            fonte,
            w + 1.0,
            texto,
        );
        x += w + Spacing::Md.px();
    };
    let nome = |i: usize| crate::tracks::propriedade_snap(snap, snap.tracks[i].prop, u);
    // A principal veste o token do grafo por linha (é o mesmo pintor que a desenha).
    item(
        ctx,
        nome(p),
        resolve(ph2d_tokens::ColorToken::TimelineCurve, theme),
    );
    if let Some(s) = s {
        let t = ph2d_i18n::tr("panel.timeline.view.normalized").replace("{name}", &nome(s));
        item(
            ctx,
            t,
            crate::folha_tinta::tinta(TimelineTinta::CurvaSecundaria, theme),
        );
    }
}

#[cfg(test)]
#[path = "curvas_vista_tests.rs"]
mod tests;
