//! ⭐ **Quem ouve o sinal de um marcador** (doc 17 item 8, maquete B5), só nos cartões: o clique parado
//! na alça de um marcador COM sinal vai até ele (como sempre) e abre o cartão «Signal at frame 24 · land»,
//! o nome só de leitura e a lista «Heard by». A lista vem da shell por esta porta de dados (o idioma do
//! `set_signal_census` do Inspector): o painel diz o nome aberto ([`signal_asked`]), a shell calcula o
//! censo dos ouvintes e devolve-o ([`set_signal_listeners`]). Fecha com Esc, clique de fora, marcador
//! sumido ou sem sinal, outra aba, ou o painel escondido.

use std::cell::RefCell;

use ph2d_editor_core::paint::{folga_do_texto, paint_text_elided};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::banda::metrics as m;
use ph2d_editor_core::panel::cards::{CardCtx, Widget};
use ph2d_editor_core::zones::Rect;
use ph2d_signal_ends::{Quem, SignalListeners};
use ph2d_timeline::TimelineViewSnapshot;
use ph2d_tokens::{ColorToken, Theme};

use crate::ids;
use crate::state::{self, TimelinePanelState};
use crate::tab::Tab;

/// O cartão aberto: o marcador (índice no armazenamento) e a aba em que abriu.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Aberto {
    pub index: usize,
    pub tab: Tab,
    /// Aberto neste quadro: o Down que o abriu (na alça) também «fechou pelo clique de fora».
    novo: bool,
}

thread_local! {
    static PEDIDO: RefCell<Option<String>> = const { RefCell::new(None) };
    static OUVINTES: RefCell<Option<(String, SignalListeners)>> = const { RefCell::new(None) };
}

/// ⭐ O nome do sinal do cartão aberto (a shell lê-o a cada quadro); `None` = nenhum cartão aberto.
#[must_use]
pub fn signal_asked() -> Option<String> {
    PEDIDO.with(|p| p.borrow().clone())
}

/// ⭐ A shell entrega quem ouve `name` (o que [`signal_asked`] pediu).
pub fn set_signal_listeners(name: &str, l: SignalListeners) {
    OUVINTES.with(|o| *o.borrow_mut() = Some((name.to_owned(), l)));
}

fn tem_sinal(snap: &TimelineViewSnapshot, index: usize) -> Option<&str> {
    snap.markers
        .get(index)
        .and_then(|m| m.2.as_deref())
        .filter(|s| !s.is_empty())
}

/// O clique parado na alça do marcador `index`: com sinal (e nos cartões) abre o cartão; sem, fecha.
pub(crate) fn abre(state: &mut TimelinePanelState, snap: &TimelineViewSnapshot, index: usize) {
    let cartoes = ph2d_editor_core::panel::cards::ferramenta::cards_on();
    state.sinal_aberto = (cartoes && tem_sinal(snap, index).is_some()).then_some(Aberto {
        index,
        tab: state.tab,
        novo: true,
    });
    if state.sinal_aberto.is_some() {
        state.opcoes_abertas = false;
        state.menu_do_clip = false;
    }
}

/// O Esc: fecha o cartão; devolve se havia um.
pub(crate) fn esc(state: &mut TimelinePanelState) -> bool {
    state.sinal_aberto.take().is_some()
}

/// O painel escondido larga o cartão e o pedido.
pub(crate) fn larga(state: &mut TimelinePanelState) {
    state.sinal_aberto = None;
    PEDIDO.with(|p| *p.borrow_mut() = None);
}

/// O clique de fora, o marcador sumido (ou sem sinal) e a troca de aba fecham o cartão.
pub(crate) fn fecha_se_preciso(state: &mut TimelinePanelState, ctx: &mut PaintCtx) {
    let fora = ctx
        .host
        .store_mut()
        .take_own_list_dismissed(ids::TIMELINE_SIGNAL_CARD);
    let tab = state.tab;
    let Some(a) = state.sinal_aberto.as_mut() else {
        return;
    };
    let novo = std::mem::take(&mut a.novo);
    let vivo = tem_sinal(&state::current_snapshot(), a.index).is_some();
    if (fora && !novo) || a.tab != tab || !vivo {
        state.sinal_aberto = None;
    }
}

/// O rótulo de quem ouve: o nome do componente (ou documento) para o artista. Os padrões são os NOMES
/// DE TIPO que o censo entrega (`stringify!` da tabela `CAMPOS`, o `doc` da `FORA`): identidade, não frase.
fn rotulo(q: Quem) -> &'static str {
    use ph2d_i18n::tr;
    match q {
        Quem::Script => tr("panel.timeline.signal.who.script"),
        Quem::Component("SignalActions") => tr("panel.timeline.signal.who.signal_actions"),
        Quem::Component("WeaponFire") => tr("panel.timeline.signal.who.weapon_fire"),
        Quem::Component("Factory") => tr("panel.timeline.signal.who.factory"),
        Quem::Component("StateMachine") => tr("panel.timeline.signal.who.state_machine"),
        Quem::Component("ShakeEmitter") => tr("panel.timeline.signal.who.shake_emitter"),
        Quem::Component("ParticleEmitter") => tr("panel.timeline.signal.who.particle_emitter"),
        Quem::Component("AnimRules") => tr("panel.timeline.signal.who.anim_rules"),
        Quem::Doc("MotionGraph") => tr("panel.timeline.signal.who.motion_graph"),
        Quem::Doc("StateSets") => tr("panel.timeline.signal.who.state_sets"),
        // Uma linha nova da tabela do censo sem rótulo: o nome do tipo (dado, não frase).
        Quem::Component(t) | Quem::Doc(t) => t,
    }
}

/// As linhas «Heard by»: `None` = a shell ainda não respondeu por este nome.
fn linhas(nome: &str) -> Option<(Vec<String>, bool)> {
    OUVINTES.with(|o| {
        let o = o.borrow();
        let (n, l) = o.as_ref().filter(|(n, _)| n == nome)?;
        debug_assert_eq!(n, nome);
        let rows = l
            .list
            .iter()
            .map(|x| match &x.object {
                Some(obj) => ph2d_i18n::tr_with(
                    "panel.timeline.signal.row",
                    &[("object", obj), ("what", &rotulo(x.what))],
                ),
                None => rotulo(x.what).to_owned(),
            })
            .collect();
        Some((rows, l.script_may))
    })
}

/// Uma linha numa tinta (o «Nobody listens» na do aviso): uma linha só, com «…» se não cabe.
fn linha_tinta(c: &mut CardCtx<'_>, text: &str, cor: ColorToken, y: f32) -> f32 {
    let top = y + m::FIELD_ROW_GAP;
    let ty = top + folga_do_texto(m::FIELD_H, m::VALUE_PX);
    let tinta = c.ink.color(cor);
    paint_text_elided(
        c.text_system,
        c.scene,
        text,
        c.x,
        ty,
        m::VALUE_PX,
        c.w,
        tinta,
    );
    top + m::FIELD_H
}

/// Pinta o cartão aberto por cima de tudo, acima da alça do marcador, alinhado à direita da linha
/// dele e preso dentro do painel; publica o pedido para a shell.
pub(crate) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    state: &TimelinePanelState,
    snap: &TimelineViewSnapshot,
) {
    let aberto = state
        .sinal_aberto
        .filter(|_| ph2d_editor_core::panel::cards::ferramenta::cards_on())
        .and_then(|a| Some((a.index, tem_sinal(snap, a.index)?.to_owned())));
    PEDIDO.with(|p| *p.borrow_mut() = aberto.as_ref().map(|a| a.1.clone()));
    let alca = aberto.as_ref().and_then(|(i, _)| {
        ctx.host
            .hit_index_mut()
            .rect_for(ids::timeline_marker_hit_id(*i))
    });
    let (Some((index, nome)), Some(alca)) = (aberto, alca) else {
        ctx.host
            .store_mut()
            .publish_own_list(ids::TIMELINE_SIGNAL_CARD, None);
        return;
    };
    let quadro = (snap.markers[index].0 * snap.fps.max(0.0)).round() as i64;
    let titulo = ph2d_i18n::tr_with(
        "panel.timeline.signal.title",
        &[("frame", &quadro.to_string()), ("name", &nome)],
    );
    let ouvintes = linhas(&nome);
    let tr = ph2d_i18n::tr;
    let corpo = |c: &mut CardCtx<'_>, y: f32| {
        let mut y = c.full(None, Widget::Readout { text: &titulo }, y);
        y = c.full(
            Some(tr("panel.timeline.signal.name")),
            Widget::Readout { text: &nome },
            y,
        );
        let Some((rows, may)) = &ouvintes else {
            return y;
        };
        y = linha_tinta(
            c,
            tr("panel.timeline.signal.heard_by"),
            ColorToken::CardLabel,
            y,
        );
        for r in rows {
            y = c.full(None, Widget::Readout { text: r }, y);
        }
        if rows.is_empty() {
            y = linha_tinta(c, tr("panel.timeline.signal.nobody"), ColorToken::Warn, y);
        }
        if *may {
            y = linha_tinta(
                c,
                tr("panel.timeline.signal.script_may"),
                ColorToken::CardLabel,
                y,
            );
        }
        y
    };
    let painel = ctx
        .host
        .store()
        .panel_rect(ph2d_editor_core::ids::TIMELINE_PANEL)
        .unwrap_or(ctx.viewport);
    let mut textos = vec![titulo.as_str(), nome.as_str()];
    if let Some((rows, _)) = &ouvintes {
        textos.extend(rows.iter().map(String::as_str));
        textos.extend([
            tr("panel.timeline.signal.nobody"),
            tr("panel.timeline.signal.script_may"),
        ]);
    }
    let larga = textos
        .iter()
        .map(|t| ctx.text_system.prefix_width(t, m::VALUE_PX).ceil())
        .fold(0.0, f32::max);
    let dentro = larga.min((painel.w - m::CARD_PAD_X * 4.0).max(0.0));
    let w = dentro + m::CARD_PAD_X * 2.0;
    let h = crate::barra_menus::altura(ctx, theme, dentro, &corpo);
    let x_linha = alca.x + alca.w * 0.5;
    let x = (x_linha - w).min(painel.x + painel.w - w).max(painel.x);
    let y = (alca.y - m::FIELD_ROW_GAP - h).max(painel.y);
    let rect = Rect::new(x, y, w, h);
    crate::barra_menus::flutua_em(
        ctx,
        theme,
        (ids::TIMELINE_SIGNAL_CARD, rect),
        dentro,
        &corpo,
    );
}

/// ⭐ A porta da shell num gesto: com um cartão aberto, `calcula` quem ouve o nome dele e entrega-o.
pub fn ouvintes(calcula: impl FnOnce(&str) -> SignalListeners) {
    if let Some(n) = signal_asked() {
        set_signal_listeners(&n, calcula(&n));
    }
}
