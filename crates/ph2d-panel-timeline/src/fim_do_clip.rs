//! ⭐ **O fim do clipe na barra** (dono 13/11): `[vizinha] [● escolhida ▾] [vizinha]` encostado à
//! direita da 2.ª fileira, a escolhida na cor do chip escolhido. O nome abre a lista; o ícone de uma
//! vizinha escolhe-a direto (as setinhas `‹ ›` saíram: supérfluas, dono 13/11). Ao trocar, ícones e nome deslizam de lugar (seco com movimento reduzido).
//! Os eventos são os das caixas de antes (`TIMELINE_LOOP` / `TIMELINE_PINGPONG`).

use std::cell::Cell;

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_icon, paint_text_elided, rect_to_vello};
use ph2d_editor_core::panel::cards::banda::metrics as m;
use ph2d_editor_core::panel::cards::{CardCtx, dicas};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, PanelHostInternal};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::TimelineViewSnapshot;
use ph2d_tokens::{ColorToken, StrokeToken, Theme};

use crate::ids;
use crate::state::{self, TimelinePanelState};

/// O que o relógio faz no fim do clipe, na ordem da roda.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Fim {
    Para,
    Laco,
    PingPong,
}

const RODA: [Fim; 3] = [Fim::Para, Fim::Laco, Fim::PingPong];

impl Fim {
    fn i(self) -> isize {
        self as isize
    }

    /// A `d` passos na roda (`-1` = a da esquerda, `+1` = a da direita).
    fn vizinha(self, d: isize) -> Fim {
        RODA[(self.i() + d).rem_euclid(3) as usize]
    }

    /// Onde `self` está em relação à escolhida `c`: `-1`, `0` ou `+1`.
    fn fenda_de(self, c: Fim) -> isize {
        match (self.i() - c.i()).rem_euclid(3) {
            2 => -1,
            k => k,
        }
    }

    pub(crate) fn id(self) -> NodeId {
        match self {
            Fim::Para => ids::TIMELINE_END_STOP,
            Fim::Laco => ids::TIMELINE_END_LOOP,
            Fim::PingPong => ids::TIMELINE_END_PINGPONG,
        }
    }

    fn de_id(id: NodeId) -> Option<Fim> {
        RODA.into_iter().find(|f| f.id() == id)
    }

    fn icone(self) -> IconId {
        match self {
            Fim::Para => IconId::StopSign,
            Fim::Laco => IconId::Loop,
            Fim::PingPong => IconId::PingPong,
        }
    }

    fn nome(self) -> &'static str {
        ph2d_i18n::tr(match self {
            Fim::Para => "panel.timeline.card.end_stop",
            Fim::Laco => "panel.timeline.loop",
            Fim::PingPong => "panel.timeline.ping_pong",
        })
    }
}

/// As dicas do widget, para a tabela da barra.
pub(crate) const DICAS: [(NodeId, &str); 4] = [
    (ids::TIMELINE_END_DD, "panel.timeline.tip.end_dd"),
    (ids::TIMELINE_END_STOP, "panel.timeline.tip.end_stop"),
    (ids::TIMELINE_END_LOOP, "panel.timeline.tip.end_loop"),
    (
        ids::TIMELINE_END_PINGPONG,
        "panel.timeline.tip.end_pingpong",
    ),
];

pub(crate) fn fim_de(snap: &TimelineViewSnapshot) -> Fim {
    match (snap.loop_range.is_some(), snap.loop_ping_pong) {
        (false, _) => Fim::Para,
        (true, false) => Fim::Laco,
        (true, true) => Fim::PingPong,
    }
}

/// O evento que a caixa de antes pedia para ir de `agora` a `quer` — `None` se já está lá.
fn toggle_que_pede(agora: Fim, quer: Fim) -> Option<(NodeId, bool)> {
    if agora == quer {
        return None;
    }
    Some(match quer {
        // Desmarcar a caixa que estava marcada.
        Fim::Para if agora == Fim::PingPong => (ids::TIMELINE_PINGPONG, false),
        Fim::Para => (ids::TIMELINE_LOOP, false),
        Fim::Laco => (ids::TIMELINE_LOOP, true),
        Fim::PingPong => (ids::TIMELINE_PINGPONG, true),
    })
}

/// O clique no widget ou na lista dele. `None` = não é comigo.
pub(crate) fn apply_event(
    state: &mut TimelinePanelState,
    host: &mut dyn PanelHostInternal,
    id: NodeId,
) -> Option<EventOutcome> {
    if id == ids::TIMELINE_END_DD {
        state.fim_aberto = !state.fim_aberto;
        state.opcoes_abertas = false;
        state.menu_do_clip = false;
        state.add_track_open = false;
        return Some(EventOutcome::Consumed);
    }
    let quer = Fim::de_id(id)?;
    let agora = fim_de(&state::current_snapshot());
    state.fim_aberto = false;
    if let Some((caixa, on)) = toggle_que_pede(agora, quer) {
        if let Some(InteractiveState::Toggle { on: no_store, .. }) = host.store_mut().get_mut(caixa)
        {
            *no_store = on;
        }
        host.bus_mut()
            .push(EditorAction::TimelinePanelEvent(PanelEvent::Toggle(
                caixa, on,
            )));
    }
    Some(EventOutcome::Consumed)
}

// ── a troca animada ─────────────────────────────────────────────────────────────────────────────

#[derive(Copy, Clone)]
struct Troca {
    de: Fim,
    para: Fim,
    t0: u128,
}

thread_local! {
    static TROCA: Cell<Option<Troca>> = const { Cell::new(None) };
}

/// A troca em curso: anota uma nova quando a escolhida mudou. A primeira vista não anima.
fn troca_de(fim: Fim, agora_ns: u128) -> Troca {
    TROCA.with(|c| {
        let t = match c.get() {
            None => Troca {
                de: fim,
                para: fim,
                t0: agora_ns,
            },
            Some(t) if t.para != fim => Troca {
                de: t.para,
                para: fim,
                t0: agora_ns,
            },
            Some(t) => t,
        };
        c.set(Some(t));
        t
    })
}

/// O andamento `0..=1` da troca, já suavizado; `1` com movimento reduzido.
fn andamento(t: Troca, agora_ns: u128, reduzido: bool) -> f32 {
    if reduzido || t.de == t.para {
        return 1.0;
    }
    let dur = f64::from(ph2d_tokens::Duration::Default.ms()) * 1e6;
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    let x = ((agora_ns.saturating_sub(t.t0)) as f64 / dur).clamp(0.0, 1.0) as f32;
    x * x * (3.0 - 2.0 * x) // LITERAL-PX-OK: smoothstep (matemática)
}

/// ⭐ Onde a opção `op` está a meio da troca, em fendas (`-1` esquerda · `0` centro · `+1` direita), e
/// com que opacidade. A que dá a volta à roda sai por um lado e entra pelo outro.
fn fenda(op: Fim, (de, para): (Fim, Fim), t: f32) -> (f32, f32) {
    #[allow(clippy::cast_precision_loss)]
    let (a, b) = (op.fenda_de(de) as f32, op.fenda_de(para) as f32);
    if (a - b).abs() <= 1.0 {
        return (a + (b - a) * t, 1.0);
    }
    if t < 0.5 {
        (a + a.signum() * t, 1.0 - 2.0 * t)
    } else {
        (b + b.signum() * (1.0 - t), 2.0 * t - 1.0)
    }
}

// ── a cara ──────────────────────────────────────────────────────────────────────────────────────

/// O nome mais largo das três.
fn nome_w(ctx: &mut PaintCtx) -> f32 {
    RODA.iter()
        .map(|f| ctx.text_system.prefix_width(f.nome(), m::VALUE_PX).ceil())
        .fold(0.0, f32::max)
}

/// A largura do centro: ícone, o nome mais largo e a seta (não muda ao trocar).
fn centro_w(ctx: &mut PaintCtx) -> f32 {
    m::FIELD_PAD_R * 3.0 + m::ICON_GLYPH * 2.0 + m::SEG_ICON_GAP + nome_w(ctx) // LITERAL-PX-OK: CONTAGEM de folgas e ícones
}

/// A largura do widget inteiro.
pub(crate) fn largura(ctx: &mut PaintCtx) -> f32 {
    m::FIELD_H * 2.0 + centro_w(ctx)
}

/// Pinta o widget em `(x, y)` e devolve o rect do centro (a âncora da lista). O centro leva a tinta do
/// chip escolhido da barra (dono 13/11: «a opção selecionada fica mais destacada, com cor»).
pub(crate) fn pinta(
    ctx: &mut PaintCtx,
    theme: Theme,
    (x, y): (f32, f32),
    (snap, aberto): (&TimelineViewSnapshot, bool),
) -> Rect {
    let ink = CardInk::of_list(theme);
    let tons = crate::transport::cartoes::tons_da_barra(theme);
    let h = m::FIELD_H;
    let cw = centro_w(ctx);
    let nw = nome_w(ctx);
    let agora = ctx.host.store().clock_ns();
    let fim = fim_de(snap);
    let reduzido = ctx.host.store().reduced_motion();
    let troca = troca_de(fim, agora);
    let t = andamento(troca, agora, reduzido);
    let caixa = Rect::new(x, y, h * 2.0 + cw, h);
    let centro = Rect::new(x + h, y, cw, h);
    let lados = [
        (fim.vizinha(-1).id(), Rect::new(x, y, h, h)),
        (fim.vizinha(1).id(), Rect::new(centro.x + cw, y, h, h)),
    ];
    let (store, hit) = ctx.host.store_and_hit_index_mut();
    let scene = &mut *ctx.scene;
    fill_rounded_rect(
        scene,
        caixa,
        m::FIELD_RADIUS,
        ink.color(ColorToken::CardField),
    );
    let por_dentro = |r: Rect| {
        Rect::new(
            r.x + m::SEG_PAD,
            r.y + m::SEG_PAD,
            r.w - m::SEG_PAD * 2.0,
            r.h - m::SEG_PAD * 2.0,
        )
    };
    for (id, r) in lados {
        hit.register(id, r);
        dicas::icone(id);
        if store.hover_live(id) > 0.0 {
            let cor = ink.color(ColorToken::CardFieldHover);
            fill_rounded_rect(scene, por_dentro(r), m::SEG_RADIUS, cor);
        }
    }
    hit.register(ids::TIMELINE_END_DD, centro);
    let aceso = aberto || store.hover_live(ids::TIMELINE_END_DD) > 0.0;
    let fundo = if aceso {
        tons.sob_o_rato
    } else {
        tons.ativa_fundo
    };
    fill_rounded_rect(scene, por_dentro(centro), m::SEG_RADIUS, fundo);
    if aceso {
        // Sob o rato ou aberta, a cor continua: o contorno do escolhido por cima do degrau acima.
        // FRAME-RAW-OK: o contorno é a MENSAGEM do escolhido (estado), não a moldura de repouso.
        let fio = StrokeToken::Thin.px();
        ph2d_editor_core::paint::stroke_rounded_rect(
            scene,
            por_dentro(centro),
            m::SEG_RADIUS,
            fio,
            tons.ativa_fundo,
        );
    }
    let g = m::ICON_GLYPH;
    let traco = StrokeToken::Default.px();
    let seta = Rect::new(centro.x + cw - m::FIELD_PAD_R - g, y + (h - g) * 0.5, g, g);
    paint_icon(scene, IconId::ChevronDown, seta, tons.ativa_texto, traco);
    // A x do glifo em cada fenda; para lá de ±1 continua meia célula (sai pela borda do recorte).
    let gx = |s: f32| {
        let (e, c, d) = (
            x + (h - g) * 0.5,
            centro.x + m::FIELD_PAD_R,
            centro.x + cw + (h - g) * 0.5,
        );
        if s <= -1.0 {
            e + (s + 1.0) * h
        } else if s <= 0.0 {
            c + (c - e) * s
        } else if s <= 1.0 {
            c + (d - c) * s
        } else {
            d + (s - 1.0) * h
        }
    };
    scene.push_clip(&rect_to_vello(caixa));
    for op in RODA {
        let (s, alfa) = fenda(op, (troca.de, troca.para), t);
        let perto = (1.0 - s.abs()).clamp(0.0, 1.0);
        let tinta = if perto >= 0.5 {
            tons.ativa_texto
        } else if store.hover_live(op.id()) > 0.0 {
            ink.color(ColorToken::Text1)
        } else {
            ink.color(ColorToken::Text2)
        };
        let gl = gx(s);
        let b = Rect::new(gl, y + (h - g) * 0.5, g, g);
        paint_icon(scene, op.icone(), b, tinta.multiply_alpha(alfa), traco);
        // O nome só se vê dentro do centro, antes da seta, e cruza com o que entra (meia fenda).
        let nome_alfa = (1.0 - 2.0 * s.abs()).clamp(0.0, 1.0);
        if nome_alfa > 0.0 {
            scene.push_clip(&rect_to_vello(Rect::new(centro.x, y, seta.x - centro.x, h)));
            paint_text_elided(
                &mut *ctx.text_system,
                scene,
                op.nome(),
                gl + g + m::SEG_ICON_GAP,
                y + ph2d_editor_core::paint::folga_do_texto(h, m::VALUE_PX),
                m::VALUE_PX,
                nw,
                tons.ativa_texto.multiply_alpha(alfa * nome_alfa),
            );
            scene.pop_layer();
        }
    }
    scene.pop_layer();
    centro
}

// ── a lista ─────────────────────────────────────────────────────────────────────────────────────

/// As três linhas da lista aberta. Devolve o `y` depois da última.
fn corpo(c: &mut CardCtx<'_>, y: f32, fim: Fim) -> f32 {
    let g = m::ICON_GLYPH;
    let mut y = y;
    for op in RODA {
        let r = Rect::new(c.x, y, c.w, m::FIELD_H);
        c.hit_index.register(op.id(), r);
        let cor = if op == fim {
            Some(ColorToken::KanbanCardRaised)
        } else {
            (c.store.hover_live(op.id()) > 0.0).then_some(ColorToken::CardFieldHover)
        };
        if let Some(cor) = cor {
            fill_rounded_rect(c.scene, r, m::SEG_RADIUS, c.ink.color(cor));
        }
        let b = Rect::new(r.x + m::FIELD_PAD_R, r.y + (r.h - g) * 0.5, g, g);
        let tinta = c.ink.color(ColorToken::Text1);
        paint_icon(c.scene, op.icone(), b, tinta, StrokeToken::Default.px());
        let tx = b.x + g + m::SEG_ICON_GAP;
        paint_text_elided(
            c.text_system,
            c.scene,
            op.nome(),
            tx,
            r.y + ph2d_editor_core::paint::folga_do_texto(r.h, m::VALUE_PX),
            m::VALUE_PX,
            r.x + r.w - tx,
            tinta,
        );
        y += m::FIELD_H + m::FIELD_ROW_GAP;
    }
    y - m::FIELD_ROW_GAP
}

/// A lista aberta, pendurada no centro do widget; fechada, retira a lista publicada.
pub(crate) fn paint_lista(
    ctx: &mut PaintCtx,
    theme: Theme,
    state: &TimelinePanelState,
    snap: &TimelineViewSnapshot,
    centro: Option<Rect>,
) {
    let Some(a) = centro.filter(|_| state.fim_aberto) else {
        ctx.host
            .store_mut()
            .publish_own_list(ids::TIMELINE_END_DD, None);
        return;
    };
    let fim = fim_de(snap);
    let dentro = m::FIELD_PAD_R * 2.0 + m::ICON_GLYPH + m::SEG_ICON_GAP + nome_w(ctx);
    crate::barra_menus::flutua(
        ctx,
        theme,
        (ids::TIMELINE_END_DD, a, true),
        dentro,
        &|c, y| corpo(c, y, fim),
    );
}

#[cfg(test)]
#[path = "fim_do_clip_tests.rs"]
mod tests;
