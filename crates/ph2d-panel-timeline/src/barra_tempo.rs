//! ⭐ **A barra de rolagem do tempo com alças de zoom** (dono 16/11: «Godot faz assim. Mas veja se
//! alguém faz melhor»). Fica no pé da fita, sob a coluna do tempo, e serve as chaves E o gráfico: os
//! dois lêem a MESMA vista (`view_start_s`, `px_per_s`) — esta barra só a escreve, sem estado seu de
//! tempo. Pesquisa e escolha: `docs/Timeline/20_barra_do_tempo.md`.
//!
//! - **o polegar é a janela visível** sobre o tempo todo (`0` → o fim do clipe ou da última chave, ou
//!   o fim da vista, o que for maior);
//! - **pontas = zoom** (After Effects «Time Navigator», Premiere, Blender): a ponta arrastada move-se,
//!   a OUTRA fica parada;
//! - **meio = rolar**; **calha = salta** (a janela centra-se ali e o mesmo arrasto continua a rolar);
//! - **duplo clique = caber tudo; outro duplo clique volta** à vista de antes (a alternância do AE);
//! - **roda sobre a barra** rola o tempo (sem Shift), Ctrl+roda dá zoom pelo meio da vista;
//! - **marcas das chaves e o playhead** desenhados na calha: com zoom, vê-se onde está o resto.
//!
//! O mapa (tempo ↔ px da barra) fica CONGELADO durante o arrasto: o «tempo todo» cresce quando a
//! vista passa do fim, e um mapa que mudasse debaixo do rato fazia o polegar fugir dele.

use ph2d_editor_core::interaction::{
    GesturePhase, InteractiveState, TimelineGesture, TimelineHitKind, TimelineWheel, WidgetStore,
};
use ph2d_editor_core::paint::{fill_rounded_rect, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::ferramenta::cards_on;
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::TimelineViewSnapshot;
use ph2d_tokens::chrome::{
    TIMELINE_BARRA_TEMPO_ALCA_PX as ALCA, TIMELINE_BARRA_TEMPO_POLEGAR_MIN_PX as POLEGAR_MIN,
    TIMELINE_BARRA_TEMPO_RECUO_PX as RECUO,
};
use ph2d_tokens::{ColorToken, StrokeToken, Theme, TimelineTinta};

use crate::alca_estado::{self, EstadoAlca};
use crate::folha_tinta;
use crate::ids::TIMELINE_TIME_BAR;
use crate::state::{MAX_PX_PER_S, MIN_PX_PER_S, TimelinePanelState};

/// A ponta esquerda do polegar (zoom; a direita fica).
pub(crate) const PONTA_E: u8 = 0;
/// A ponta direita do polegar (zoom; a esquerda fica).
pub(crate) const PONTA_D: u8 = 1;
/// O meio do polegar (rola).
pub(crate) const MEIO: u8 = 2;
/// A calha fora do polegar (salta e rola).
pub(crate) const CALHA: u8 = 3;

/// O tempo ↔ px da barra: `x` e `w` são a calha, `total_s` o tempo que ela inteira representa.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Mapa {
    pub x: f32,
    pub w: f32,
    pub total_s: f64,
}

impl Mapa {
    /// O tempo sob o px `x` da barra.
    pub(crate) fn t(self, x: f32) -> f64 {
        f64::from(x - self.x) / f64::from(self.w) * self.total_s
    }

    /// O px da barra do tempo `t`.
    pub(crate) fn x(self, t: f64) -> f32 {
        self.x + (t / self.total_s * f64::from(self.w)) as f32
    }
}

/// Um arrasto na barra: a parte agarrada, o mapa congelado, a vista no aperto e o tempo agarrado.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Arrasto {
    pub part: u8,
    pub mapa: Mapa,
    pub inicio: f64,
    pub fim: f64,
    /// A largura da coluna do tempo em px (a vista mede `largura / px_per_s` segundos).
    pub largura: f64,
    pub pega: f64,
}

/// O estado da barra no painel (nunca gravado, nunca desfeito: é câmara, não documento).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BarraEstado {
    /// O mapa do último quadro pintado (o gesto chega antes da pintura do quadro dele).
    pub(crate) mapa: Option<Mapa>,
    pub(crate) arrasto: Option<Arrasto>,
    /// O «caber tudo» do duplo clique: `(vista de antes, vista que ele deixou)`, para o segundo voltar.
    pub(crate) volta: Option<((f64, f64), (f64, f64))>,
}

/// O fim do tempo que a barra cobre: o maior entre o fim do clipe, o conteúdo da aba e o fim da vista.
pub(crate) fn total_s(state: &TimelinePanelState, snap: &TimelineViewSnapshot) -> f64 {
    let conteudo = crate::view::content_extent(state, snap).map_or(0.0, |(_, b)| b);
    snap.view_length_seconds
        .max(conteudo)
        .max(state.view_start_s + state.view_span_s)
        .max(f64::EPSILON)
}

/// O polegar em px: a janela `[inicio, inicio+span]` no mapa, nunca mais estreito que
/// [`POLEGAR_MIN`] (alarga-se à volta do centro) e sempre dentro da calha.
pub(crate) fn polegar(mapa: Mapa, inicio: f64, span: f64) -> (f32, f32) {
    let fim_x = mapa.x + mapa.w;
    let (a, b) = (mapa.x(inicio), mapa.x(inicio + span));
    let min = POLEGAR_MIN.min(mapa.w);
    let (a, b) = if b - a < min {
        let c = (a + b) * 0.5 - min * 0.5;
        let a = c.max(mapa.x).min(fim_x - min);
        (a, a + min)
    } else {
        (a, b)
    };
    (a.max(mapa.x), b.min(fim_x))
}

/// A vista `(view_start_s, px_per_s)` que um arrasto deixa com o rato no tempo `t` (do mapa congelado).
pub(crate) fn arrasta(d: Arrasto, t: f64) -> (f64, f64) {
    let dt = t - d.pega;
    let min_span = d.largura / MAX_PX_PER_S;
    let zoom = |span: f64| (d.largura / span).clamp(MIN_PX_PER_S, MAX_PX_PER_S); // CLAMP-OK: const bounds, min<max
    match d.part {
        PONTA_E => {
            let inicio = (d.inicio + dt).min(d.fim - min_span).max(0.0);
            let px = zoom(d.fim - inicio);
            ((d.fim - d.largura / px).max(0.0), px)
        }
        PONTA_D => {
            let fim = (d.fim + dt).max(d.inicio + min_span);
            (d.inicio, zoom(fim - d.inicio))
        }
        _ => ((d.inicio + dt).max(0.0), d.largura / (d.fim - d.inicio)),
    }
}

/// Um gesto na barra (Primary): aperto, arrasto, largada, duplo clique.
pub(crate) fn apply(
    state: &mut TimelinePanelState,
    snap: &TimelineViewSnapshot,
    part: u8,
    g: TimelineGesture,
) {
    match g.phase {
        GesturePhase::Begin => {
            let Some(mapa) = state.barra.mapa else {
                return;
            };
            let largura = state.view_span_s * state.px_per_s;
            if largura <= 0.0 || state.view_span_s <= 0.0 {
                return;
            }
            if part == CALHA {
                state.view_start_s = (mapa.t(g.x) - state.view_span_s * 0.5).max(0.0);
            }
            state.barra.arrasto = Some(Arrasto {
                part,
                mapa,
                inicio: state.view_start_s,
                fim: state.view_start_s + state.view_span_s,
                largura,
                pega: mapa.t(g.x),
            });
        }
        GesturePhase::Update => {
            if let Some(d) = state.barra.arrasto {
                (state.view_start_s, state.px_per_s) = arrasta(d, d.mapa.t(g.x));
            }
        }
        GesturePhase::End | GesturePhase::Click => state.barra.arrasto = None,
        GesturePhase::DoubleClick => {
            state.barra.arrasto = None;
            caber_tudo(state, snap);
        }
    }
}

/// O duplo clique: enquadra o clipe inteiro (o mesmo enquadrar de abrir a fita); se a vista ainda é a
/// que ele deixou, volta à de antes.
pub(crate) fn caber_tudo(state: &mut TimelinePanelState, snap: &TimelineViewSnapshot) {
    let agora = (state.view_start_s, state.px_per_s);
    match state.barra.volta {
        Some((antes, deixou)) if deixou == agora => {
            (state.view_start_s, state.px_per_s) = antes;
            state.barra.volta = None;
        }
        _ => {
            let largura = (state.view_span_s * state.px_per_s) as f32;
            crate::view::apply_fit_clip(state, largura, snap);
            state.barra.volta = Some((agora, (state.view_start_s, state.px_per_s)));
        }
    }
}

/// A roda com o rato sobre a barra: a roda simples rola o TEMPO (não as linhas) e o Ctrl+roda dá zoom
/// pelo meio da vista (o tempo sob o rato não está na coluna do tempo, está na barra).
pub(crate) fn roda(
    state: &TimelinePanelState,
    store: &WidgetStore,
    w: &mut TimelineWheel,
    time_x: f32,
) {
    if !store
        .hot_id()
        .is_some_and(|h| TIMELINE_TIME_BAR.contains(&h))
    {
        return;
    }
    w.pan_delta += w.scroll_delta;
    w.scroll_delta = 0.0;
    w.anchor_x = time_x + (state.view_span_s * state.px_per_s * 0.5) as f32;
}

/// Pinta a barra em `bar` e regista as quatro partes (calha, meio, pontas — a última ganha).
pub(crate) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    bar: Rect,
    state: &mut TimelinePanelState,
    snap: &TimelineViewSnapshot,
) {
    if bar.w <= POLEGAR_MIN || bar.h <= RECUO * 2.0 {
        state.barra.mapa = None;
        return;
    }
    let mapa = state.barra.arrasto.map_or_else(
        || Mapa {
            x: bar.x,
            w: bar.w,
            total_s: total_s(state, snap),
        },
        |d| d.mapa,
    );
    state.barra.mapa = Some(mapa);
    let corpo = Rect::new(bar.x, bar.y + RECUO, bar.w, bar.h - RECUO * 2.0);
    let raio = corpo.h * 0.5;
    fill_rounded_rect(
        ctx.scene,
        corpo,
        raio,
        folha_tinta::cor(ColorToken::Bg2, theme),
    );

    let (a, b) = polegar(mapa, state.view_start_s, state.view_span_s);
    let polegar_r = Rect::new(a, corpo.y, (b - a).max(0.0), corpo.h);
    let store = ctx.host.store();
    let estado = |part: u8| EstadoAlca::de(store, TIMELINE_TIME_BAR[usize::from(part)]);
    let (e_meio, e_esq, e_dir) = (estado(MEIO), estado(PONTA_E), estado(PONTA_D));
    // Cinza em repouso, o cinza dos valores (o `BorderEmph` lê-se azul nos cartões e a barra gritava); o azul é só
    // da mão a apertar (`alca_estado::cor`).
    let repouso = folha_tinta::tinta(TimelineTinta::TextoValor, theme);
    let cor_polegar = alca_estado::cor(e_meio, repouso, theme);
    fill_rounded_rect(ctx.scene, polegar_r, raio, cor_polegar);
    // As pontas: um traço vertical dentro de cada uma, a dizer «agarre aqui para o zoom».
    let traco = alca_estado::tinta_sobre(cor_polegar, theme);
    let pontas = alvos_das_pontas(polegar_r);
    for (e, r) in [(e_esq, pontas[0]), (e_dir, pontas[1])] {
        let w = StrokeToken::Thin.px() * 2.0 + e.cresce();
        let h = corpo.h * 0.7;
        let fio = Rect::new(r.x + (r.w - w) * 0.5, corpo.y + (corpo.h - h) * 0.5, w, h);
        fill_rounded_rect(ctx.scene, fio, w * 0.5, alca_estado::cor(e, traco, theme));
    }
    pinta_marcas(ctx, theme, corpo, (a, b), traco, mapa, snap);
    pinta_playhead(ctx, theme, bar, mapa, snap);

    for (part, r) in [
        (CALHA, bar),
        (MEIO, Rect::new(a, bar.y, (b - a).max(0.0), bar.h)),
        (PONTA_E, Rect::new(pontas[0].x, bar.y, pontas[0].w, bar.h)),
        (PONTA_D, Rect::new(pontas[1].x, bar.y, pontas[1].w, bar.h)),
    ] {
        regista(ctx, part, r);
    }
}

/// Os alvos das duas pontas dentro do polegar `p`: cada um `ALCA` de largura (ou um terço do
/// polegar, se ele for mais estreito — o meio nunca desaparece).
pub(crate) fn alvos_das_pontas(p: Rect) -> [Rect; 2] {
    let w = ALCA.min(p.w / 3.0);
    [
        Rect::new(p.x, p.y, w, p.h),
        Rect::new(p.x + p.w - w, p.y, w, p.h),
    ]
}

/// As marcas das chaves na calha (uma por px), para se ver onde está o resto com o zoom perto — por
/// cima do polegar, na tinta que se lê sobre ele (`sobre`) entre `polegar.0` e `polegar.1`.
fn pinta_marcas(
    ctx: &mut PaintCtx,
    theme: Theme,
    corpo: Rect,
    polegar: (f32, f32),
    sobre: ph2d_vector::Color,
    mapa: Mapa,
    snap: &TimelineViewSnapshot,
) {
    let mut xs: Vec<i32> = snap
        .tracks
        .iter()
        .flat_map(|t| &t.keys)
        .map(|k| mapa.x(k.t_seconds).round() as i32)
        .collect();
    xs.sort_unstable();
    xs.dedup();
    let cor = folha_tinta::cor(ColorToken::TimelineRulerTick, theme);
    let w = StrokeToken::Thin.px();
    let h = corpo.h * 0.5;
    for x in xs {
        let x = x as f32;
        if x >= corpo.x && x <= corpo.x + corpo.w {
            let r = Rect::new(x - w * 0.5, corpo.y + (corpo.h - h) * 0.5, w, h);
            let dentro = x >= polegar.0 && x <= polegar.1;
            fill_rounded_rect(ctx.scene, r, 0.0, if dentro { sobre } else { cor });
        }
    }
}

/// O playhead na barra: onde se está, mesmo com a vista longe dali.
fn pinta_playhead(
    ctx: &mut PaintCtx,
    theme: Theme,
    bar: Rect,
    mapa: Mapa,
    snap: &TimelineViewSnapshot,
) {
    let x = mapa.x(snap.time_seconds);
    if x < bar.x || x > bar.x + bar.w {
        return;
    }
    let cor = if cards_on() {
        folha_tinta::tinta(TimelineTinta::Cursor, theme)
    } else {
        resolve(ColorToken::TimelinePlayhead, theme)
    };
    let w = StrokeToken::Thin.px() * 2.0;
    fill_rounded_rect(ctx.scene, Rect::new(x - w * 0.5, bar.y, w, bar.h), 0.0, cor);
}

/// Regista uma parte como superfície da Timeline (o despacho entrega o gesto ao painel).
fn regista(ctx: &mut PaintCtx, part: u8, r: Rect) {
    let id = TIMELINE_TIME_BAR[usize::from(part)];
    ctx.host.store_mut().register(
        id,
        InteractiveState::TimelineSurface {
            parent: ph2d_editor_core::ids::TIMELINE_PANEL,
            kind: TimelineHitKind::TimeBar { part },
            canvas: r,
        },
    );
    ctx.host.hit_index_mut().register(id, r);
}

#[cfg(test)]
#[path = "barra_tempo_tests.rs"]
mod tests;
