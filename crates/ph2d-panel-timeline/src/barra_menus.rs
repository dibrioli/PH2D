//! ⭐ **As duas janelinhas da barra em cartões** (`docs/Timeline/15_plano_redesenho_molde_rive.md`, onda 1):
//! «Timeline options» (a engrenagem) e o menu do clipe (o «⋯» do chip escolhido). Nenhuma linha é função
//! nova: cada uma é o controlo que antes vivia solto na barra, com o mesmo id e o mesmo evento. Só as
//! três células de «When the clip ends» têm ids próprios — pedem os eventos de `TIMELINE_LOOP` e
//! `TIMELINE_PINGPONG`, exatamente o que a caixa de antes pedia.
//!
//! Abrem-se pelo clique na porta e fecham-se pelo clique de fora (`publish_own_list`, a lei do «+ Track»).

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{HitIndex, InteractiveState, WidgetEvent};
use ph2d_editor_core::paint::fill_rounded_rect;
use ph2d_editor_core::panel::cards::banda::metrics as m;
use ph2d_editor_core::panel::cards::corpo::letra_da_grandeza;
use ph2d_editor_core::panel::cards::{CardCtx, Seg, Widget};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, PanelHostInternal, camada_das_listas};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::card_field::{self, Glyph};
use ph2d_editor_core::widget::card_ink::{self, CardInk};
use ph2d_editor_core::widget::key_mark::Rotulo;
use ph2d_editor_core::widget::panel_chrome::clamp_menu_to_viewport;
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::TimelineViewSnapshot;
use ph2d_tokens::{ColorToken, StrokeToken, Theme};
use ph2d_vector::VectorScene;

use crate::ids;
use crate::state::{self, TimelinePanelState};
use crate::transport::BarView;

/// O que o relógio faz no fim do clipe — a leitura das duas caixas de antes.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Fim {
    Para,
    Laco,
    PingPong,
}

fn fim_de(snap: &TimelineViewSnapshot) -> Fim {
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

/// O que a caixa da Length lê, se não é o do store: sem duração autorada, ∞; o store fica com o
/// finito, que é o que se edita.
fn valor_lido_da_dur(snap: &TimelineViewSnapshot) -> Option<f64> {
    (!snap.view_length_explicit).then_some(f64::INFINITY)
}

/// Os controlos do menu do clipe: um clique neles fecha o menu (e segue para o despacho de sempre).
const DO_MENU: [NodeId; 5] = [
    ids::TIMELINE_RENAME_CLIP,
    ids::TIMELINE_DUP_CLIP,
    ids::TIMELINE_REVERSE_CLIP,
    ids::TIMELINE_DELETE_CLIP,
    ids::TIMELINE_CLIP_AUTOPLAY,
];

/// As portas e as células do fim. `None` = não é comigo — e um clique numa linha do menu fecha-o
/// antes de seguir para o despacho de sempre.
pub(crate) fn apply_event(
    state: &mut TimelinePanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> Option<EventOutcome> {
    let id = match ev {
        WidgetEvent::Click(id) | WidgetEvent::Toggled(id) => id,
        _ => return None,
    };
    if DO_MENU.contains(&id) {
        state.menu_do_clip = false;
        return None;
    }
    if id == ids::TIMELINE_ONION_SETTINGS {
        state.opcoes_abertas = false;
        return None;
    }
    if id == ids::TIMELINE_OPTIONS {
        state.opcoes_abertas = !state.opcoes_abertas;
        state.menu_do_clip = false;
        state.add_track_open = false;
        return Some(EventOutcome::Consumed);
    }
    if id == ids::TIMELINE_CLIP_MENU {
        state.menu_do_clip = !state.menu_do_clip;
        state.opcoes_abertas = false;
        state.add_track_open = false;
        return Some(EventOutcome::Consumed);
    }
    let quer = [
        (ids::TIMELINE_END_STOP, Fim::Para),
        (ids::TIMELINE_END_LOOP, Fim::Laco),
        (ids::TIMELINE_END_PINGPONG, Fim::PingPong),
    ]
    .into_iter()
    .find_map(|(i, f)| (i == id).then_some(f))?;
    if let Some((caixa, on)) = toggle_que_pede(fim_de(&state::current_snapshot()), quer) {
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

/// Os dois cliques de fora deste quadro fecham o que estava aberto.
pub(crate) fn fecha_pelo_clique_de_fora(state: &mut TimelinePanelState, ctx: &mut PaintCtx) {
    let store = ctx.host.store_mut();
    if store.take_own_list_dismissed(ids::TIMELINE_OPTIONS) {
        state.opcoes_abertas = false;
    }
    if store.take_own_list_dismissed(ids::TIMELINE_CLIP_MENU) {
        state.menu_do_clip = false;
    }
}

fn tr(chave: &str) -> &'static str {
    ph2d_i18n::tr(chave)
}

/// As linhas de «Timeline options». Devolve o `y` depois da última.
fn corpo_das_opcoes(
    c: &mut CardCtx<'_>,
    y: f32,
    snap: &TimelineViewSnapshot,
    view: BarView,
) -> f32 {
    let fim = fim_de(snap);
    let segs = [
        Seg {
            id: ids::TIMELINE_END_STOP,
            icon: None,
            text: tr("panel.timeline.card.end_stop"),
            on: fim == Fim::Para,
        },
        Seg {
            id: ids::TIMELINE_END_LOOP,
            icon: None,
            text: tr("panel.timeline.loop"),
            on: fim == Fim::Laco,
        },
        Seg {
            id: ids::TIMELINE_END_PINGPONG,
            icon: None,
            text: tr("panel.timeline.ping_pong"),
            on: fim == Fim::PingPong,
        },
    ];
    let y = c.full(
        Some(tr("panel.timeline.card.when_ends")),
        Widget::Choice(&segs),
        y,
    );
    let nome = tr("panel.timeline.card.length_long");
    let campo = Widget::Field {
        id: ids::TIMELINE_LENGTH_NUM,
        glyph: Glyph::Text(letra_da_grandeza(nome)),
    };
    let mut y = match valor_lido_da_dur(snap) {
        Some(lido) => card_field::com_valores(&[(ids::TIMELINE_LENGTH_NUM, lido)], || {
            c.full(Some(nome), campo, y)
        }),
        None => c.full(Some(nome), campo, y),
    };
    let valor = |id| crate::transport::cartoes::valor_da_caixa(snap, view, id);
    for (id, chave) in [
        (ids::TIMELINE_SNAP, "panel.timeline.card.snap"),
        (ids::TIMELINE_PHYSICS, "panel.timeline.card.physics"),
        (ids::TIMELINE_MOTION_PATH, "panel.timeline.card.motion_path"),
        (ids::TIMELINE_SPEED, "panel.timeline.card.speed"),
        (ids::TIMELINE_ONION, "panel.timeline.card.onion"),
        (ids::TIMELINE_ONION_MODE, "panel.timeline.card.onion_keys"),
    ] {
        y = c.check(id, Rotulo::vista_de(tr(chave)), valor(id), y);
    }
    c.full(
        None,
        Widget::Button {
            id: ids::TIMELINE_ONION_SETTINGS,
            text: tr("panel.timeline.card.onion_settings"),
            primary: false,
        },
        y,
    )
}

/// As linhas do menu do clipe. Devolve o `y` depois da última.
fn corpo_do_menu(c: &mut CardCtx<'_>, y: f32, snap: &TimelineViewSnapshot) -> f32 {
    let mut y = y;
    for (id, chave) in [
        (ids::TIMELINE_RENAME_CLIP, "panel.timeline.card.rename"),
        (ids::TIMELINE_DUP_CLIP, "panel.timeline.card.duplicate"),
        (ids::TIMELINE_REVERSE_CLIP, "panel.timeline.card.reverse"),
    ] {
        let text = tr(chave);
        y = c.full(
            None,
            Widget::Button {
                id,
                text,
                primary: false,
            },
            y,
        );
    }
    y = c.check(
        ids::TIMELINE_CLIP_AUTOPLAY,
        Rotulo::vista_de(tr("panel.timeline.card.starts_with")),
        crate::transport_entry::ligado(snap),
        y,
    );
    let text = tr("panel.timeline.card.delete");
    // O lixo só existe acima de um clipe: um documento tem sempre um para editar.
    let w = if snap.clips.len() > 1 {
        Widget::Button {
            id: ids::TIMELINE_DELETE_CLIP,
            text,
            primary: false,
        }
    } else {
        Widget::ButtonOff { text }
    };
    c.full(None, w, y)
}

/// A largura de dentro: a frase mais larga com a caixa de marcar à frente.
fn largura_de_dentro(ctx: &mut PaintCtx, chaves: &[&str]) -> f32 {
    let mais_larga = chaves
        .iter()
        .map(|k| ctx.text_system.prefix_width(tr(k), m::VALUE_PX).ceil())
        .fold(0.0, f32::max);
    m::CHECK_BOX + m::CHECK_GAP + mais_larga + m::KEY_MARK_BOX
}

/// Uma janelinha de cartão pendurada em `ancora`, alinhada pela ESQUERDA (`a_esquerda`) ou pela direita.
fn flutua(
    ctx: &mut PaintCtx,
    theme: Theme,
    (dono, ancora, a_esquerda): (NodeId, Rect, bool),
    dentro: f32,
    corpo: &dyn Fn(&mut CardCtx<'_>, f32) -> f32,
) {
    let ink = CardInk::of_list(theme);
    let w = dentro + m::CARD_PAD_X * 2.0;
    let medida = {
        let mut cena = VectorScene::new();
        let mut alvos = HitIndex::new();
        let mut c = CardCtx::flutuante(
            &mut cena,
            ctx.text_system,
            ctx.host.store(),
            &mut alvos,
            ink,
            m::CARD_PAD_X,
            dentro,
        );
        card_ink::com(ink, || corpo(&mut c, m::CARD_PAD_TOP))
    };
    let h = medida + m::CARD_PAD_BOTTOM;
    let x = if a_esquerda {
        ancora.x
    } else {
        ancora.x + ancora.w - w
    };
    let rect = clamp_menu_to_viewport(
        x,
        ancora.y + ancora.h + m::FIELD_ROW_GAP,
        w,
        h,
        ctx.viewport,
    );
    ctx.host.store_mut().publish_own_list(dono, Some(rect));
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let marca = hit_index.len();
    camada_das_listas::por_cima(ctx.scene, |scene| {
        fill_rounded_rect(
            scene,
            rect,
            m::CARD_RADIUS,
            ink.color(ColorToken::KanbanCard),
        );
        ph2d_editor_core::paint::stroke_frame(
            scene,
            rect,
            m::CARD_RADIUS,
            theme,
            ph2d_tokens::visuals::Feel::Rest,
            StrokeToken::Thin.px(),
            ink.color(ColorToken::Border),
        );
        // O fundo ENGOLE o clique: sem ele um clique entre duas linhas caía na folha por baixo.
        hit_index.register(ph2d_editor_core::ids::RAIL_BACKDROP, rect);
        let mut c = CardCtx::flutuante(
            &mut *scene,
            &mut *text_system,
            store,
            &mut *hit_index,
            ink,
            rect.x + m::CARD_PAD_X,
            dentro,
        );
        card_ink::com(ink, || corpo(&mut c, rect.y + m::CARD_PAD_TOP));
    });
    camada_das_listas::alvos_desde(hit_index, marca);
}

/// Pinta o que estiver aberto, sobre tudo (depois da folha). Fechado, retira a lista publicada.
pub(crate) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    state: &TimelinePanelState,
    (snap, view): (&TimelineViewSnapshot, BarView),
    (opcoes, menu): (Option<Rect>, Option<Rect>),
) {
    match opcoes.filter(|_| state.opcoes_abertas) {
        Some(a) => {
            let dentro = largura_de_dentro(
                ctx,
                &[
                    "panel.timeline.card.snap",
                    "panel.timeline.card.physics",
                    "panel.timeline.card.motion_path",
                    "panel.timeline.card.speed",
                ],
            );
            flutua(
                ctx,
                theme,
                (ids::TIMELINE_OPTIONS, a, false),
                dentro,
                &|c, y| corpo_das_opcoes(c, y, snap, view),
            );
        }
        None => ctx
            .host
            .store_mut()
            .publish_own_list(ids::TIMELINE_OPTIONS, None),
    }
    match menu.filter(|_| state.menu_do_clip) {
        Some(a) => {
            let dentro = largura_de_dentro(ctx, &["panel.timeline.card.starts_with"]);
            flutua(
                ctx,
                theme,
                (ids::TIMELINE_CLIP_MENU, a, true),
                dentro,
                &|c, y| corpo_do_menu(c, y, snap),
            );
        }
        None => ctx
            .host
            .store_mut()
            .publish_own_list(ids::TIMELINE_CLIP_MENU, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ Sem duração autorada a Length lê ∞; com ela, o número. *Mutação: devolver sempre `None` ⇒ vermelho.*
    #[test]
    fn a_dur_le_infinito_so_sem_duracao_autorada() {
        let mut snap = TimelineViewSnapshot {
            view_length_explicit: false,
            ..TimelineViewSnapshot::default()
        };
        assert_eq!(valor_lido_da_dur(&snap), Some(f64::INFINITY));
        snap.view_length_explicit = true;
        assert_eq!(valor_lido_da_dur(&snap), None);
    }

    /// ⭐⭐ Cada célula pede o que a caixa de antes pedia, e a escolhida não pede nada (re-armar o Loop
    /// repunha a faixa dele). *Mutação: tirar o `agora == quer` ⇒ o Loop aceso pede outra vez.*
    #[test]
    fn cada_celula_do_fim_pede_o_toggle_de_antes() {
        use Fim::*;
        assert_eq!(toggle_que_pede(Laco, Laco), None);
        assert_eq!(
            toggle_que_pede(Para, Laco),
            Some((ids::TIMELINE_LOOP, true))
        );
        assert_eq!(
            toggle_que_pede(Laco, PingPong),
            Some((ids::TIMELINE_PINGPONG, true))
        );
        assert_eq!(
            toggle_que_pede(Laco, Para),
            Some((ids::TIMELINE_LOOP, false))
        );
        assert_eq!(
            toggle_que_pede(PingPong, Para),
            Some((ids::TIMELINE_PINGPONG, false))
        );
    }
}
