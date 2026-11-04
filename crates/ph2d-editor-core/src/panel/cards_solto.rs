//! ⭐ **C5 — o cartão solto no canvas** (`docs/CartoesUI/02_especificacao.md` §8): a linha-fantasma
//! na lista, o cartão a pairar (sombra de pairar, «Return to panel», piscar), o gesto de o mover e
//! de o reencaixar, e os cliques dos dois ids próprios. O estado mora em `card_loose_ops`.

use super::*;
use crate::interaction::{
    CardDrag, LooseEdge, loose_edge_of, loose_face_of, loose_ghost_of, loose_pin_of, loose_rect_of,
    loose_return_of,
};

/// O canto do cartão solto preso ao canvas: pelo menos `LOOSE_KEEP_VISIBLE` dele fica à vista, e o
/// cabeçalho nunca sobe acima do canvas (a pega não se perde).
pub(super) fn preso(at: (f32, f32), w: f32, canvas: Rect) -> (f32, f32) {
    let k = m::LOOSE_KEEP_VISIBLE;
    let x =
        at.0.clamp(canvas.x + k - w, (canvas.x + canvas.w - k).max(canvas.x));
    let y = at.1.clamp(
        canvas.y,
        (canvas.y + canvas.h - m::CARD_HEAD_H).max(canvas.y),
    );
    (x, y)
}

/// `(desvio da mão, tamanho)` de um solto que volta à lista.
type SoltoVoo = ((f32, f32), (f32, f32));

/// A mão sobre um cartão SOLTO. Fora da lista ele anda com ela; ao entrar na coluna da lista volta
/// a ser dela: devolve `(desvio da mão, tamanho)` para a lista o levantar como voo.
pub(super) fn arrasta(
    store: &mut WidgetStore,
    d: CardDrag,
    na_lista: bool,
    canvas: Rect,
) -> Option<(CardDrag, SoltoVoo)> {
    if let Some(e) = d.edge {
        largura(store, d, e, canvas);
        return None;
    }
    let c = store.loose_card(d.section)?;
    let h = c.altura();
    let grab = c.grab.unwrap_or((d.down.0 - c.at.0, d.down.1 - c.at.1));
    if na_lista {
        store.return_loose_card(d.section);
        store.take_card_arrival(c.section); // a mão traz o cartão: não há voo de regresso
        // T54: o arrasto passou a ser da SECÇÃO (a instância saiu do mapa).
        return Some((store.card_drag()?, (grab, (c.w, h))));
    }
    store.raise_loose_card(d.section, false);
    if let Some(c) = store.loose_card_mut(d.section) {
        c.grab = Some(grab);
        c.at = (d.at.0 - grab.0, d.at.1 - grab.1);
    }
    None
}

/// ⭐ T39: a mão numa borda muda a largura; a borda oposta fica onde estava. Entre
/// [`m::LOOSE_MIN_W`] e a largura do canvas.
fn largura(store: &mut WidgetStore, d: CardDrag, e: LooseEdge, canvas: Rect) {
    let Some(c) = store.loose_card_mut(d.section) else {
        return;
    };
    let (x0, w0) = c.resize.unwrap_or((c.at.0, c.w));
    let dx = d.at.0 - d.down.0;
    let max = canvas.w.max(m::LOOSE_MIN_W);
    let w = match e {
        LooseEdge::Right => w0 + dx,
        LooseEdge::Left => w0 - dx,
    }
    .clamp(m::LOOSE_MIN_W, max);
    c.resize = Some((x0, w0));
    c.w = w;
    if e == LooseEdge::Left {
        c.at.0 = x0 + w0 - w;
    }
}

/// T40: as secções soltas que deixam a linha-fantasma na lista — as que têm uma cópia que segue a
/// seleção ou presa ao sujeito dela (as presas a OUTRO deixam a lista mostrar a secção da seleção).
/// T54: uma secção pode ter várias instâncias soltas.
pub(super) fn fantasmas(store: &WidgetStore, panel: NodeId, soltos: &[NodeId]) -> Vec<NodeId> {
    let agora = store.card_subject(panel);
    let todos = store.loose_cards_of(panel);
    soltos
        .iter()
        .copied()
        .filter(|k| {
            let mut copias = todos.iter().filter(|(_, c)| c.section == *k).peekable();
            copias.peek().is_none() || copias.any(|(_, c)| c.pin.is_none_or(|p| Some(p) == agora))
        })
        .collect()
}

/// T42: a opacidade do nome em destaque — 1 parado; com o rato por cima, pisca devagar
/// ([`m::LOOSE_PIN_BLINK_S`]).
fn piscar(clock_ns: u128, sobre: bool) -> f32 {
    if !sobre {
        return 1.0;
    }
    #[allow(clippy::cast_precision_loss)]
    let fase = (clock_ns as f64 / 1e9 / m::LOOSE_PIN_BLINK_S).fract();
    #[allow(clippy::cast_possible_truncation)]
    let onda = (0.5 + 0.5 * (fase * std::f64::consts::TAU).cos()) as f32;
    m::LOOSE_PIN_BLINK_MIN + (1.0 - m::LOOSE_PIN_BLINK_MIN) * onda
}

/// Os botões próprios dos soltos (linha-fantasma, alfinete, «Return to panel»), com as dicas, e o
/// título de cada um (o «Not available» precisa dele) — antes da pintura, que só lê o store.
/// T54: `soltos` são secções; cada instância solta de uma delas prepara-se.
pub(super) fn prepara(store: &mut WidgetStore, soltos: &[NodeId], cards: &[CardDef<'_>]) {
    let instancias: Vec<NodeId> = store
        .loose_section_ids()
        .into_iter()
        .filter(|k| soltos.contains(&store.loose_section(*k)))
        .collect();
    for k in &instancias {
        if let Some(c) = cards.iter().find(|c| c.section == store.loose_section(*k)) {
            store.set_loose_card_title(*k, c.title);
        }
        let estado = store.loose_card(*k).map(|c| (c.pin.is_some(), c.rope));
        let proximo = match estado {
            Some((true, false)) => "card.loose.rope",
            Some((true, true)) => "card.loose.unpin",
            _ => "card.loose.pin",
        };
        store.set_tooltip(loose_pin_of(*k), ph2d_i18n::tr(proximo));
        for (id, dica) in [
            (loose_ghost_of(*k), "card.loose.ghost_tip"),
            (loose_pin_of(*k), "card.loose.pin"),
            (loose_return_of(*k), "card.loose.return"),
        ] {
            store.register_if_absent(
                id,
                crate::interaction::InteractiveState::Button {
                    state: crate::widget::ButtonState::Normal,
                },
            );
            if store.tooltip_for(id).is_none() {
                store.set_tooltip(id, ph2d_i18n::tr(dica));
            }
        }
    }
}

/// A linha-fantasma «<Título> · on canvas» no lugar do cartão (tracejada; clique = trazer à vista).
pub(super) fn fantasma(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    theme: Theme,
    r: Rect,
    card: &CardDef<'_>,
) {
    let id = loose_ghost_of(card.section);
    hit_index.register(id, r);
    if store.hover_live(id) > 0.0 {
        fill_rounded_rect(
            scene,
            r,
            m::CARD_RADIUS,
            resolve(ColorToken::CardFieldHover, theme),
        );
    }
    let tinta = resolve(ColorToken::KanbanListFg, theme);
    crate::paint::stroke_rounded_rect_dashed(
        scene,
        r,
        m::CARD_RADIUS,
        StrokeToken::Default.px(),
        m::LOOSE_DASH,
        tinta,
    );
    let g = m::ICON_GLYPH;
    let ix = r.x + m::CARD_PAD_X;
    paint_icon(
        scene,
        IconId::Link,
        Rect::new(ix, r.y + (r.h - g) * 0.5, g, g),
        tinta,
        StrokeToken::Default.px(),
    );
    let texto = format!("{} · {}", card.title, ph2d_i18n::tr("card.loose.on_canvas"));
    let tx = ix + g + m::SEG_ICON_GAP;
    crate::paint::paint_text(
        text_system,
        scene,
        &texto,
        tx,
        r.y + (r.h - m::LABEL_PX) * 0.5,
        m::LABEL_PX,
        (r.x + r.w - m::CARD_PAD_X - tx).max(0.0),
        tinta,
    );
}

/// ⭐ Pinta os cartões soltos de `panel` (do fundo para cima) sobre o canvas. Devolve o rect de cada
/// um (`None` = não pintado) para [`publica`] — é o rect que dá o clique ao cartão e não ao canvas.
#[allow(clippy::too_many_arguments)]
pub(super) fn pinta_soltos(
    tela: &mut VectorScene,
    text_system: &mut TextSystem,
    loja: &mut WidgetStore,
    hit_index: &mut HitIndex,
    panel: NodeId,
    theme: Theme,
    canvas: Rect,
    cards: &[CardDef<'_>],
    pinned: &[CardDef<'_>],
) -> Vec<(NodeId, Option<Rect>)> {
    let mut out = Vec::new();
    let d = super::solto_some::passo(loja);
    for (k, c) in loja.loose_cards_of(panel) {
        // T42: o preso mostra o dele com qualquer seleção; o não-preso só aparece quando a secção
        // se aplica à seleção (a mesma classe) — senão espera invisível.
        // T54: o preso pelo id da instância (o painel dá-lho); o que segue a seleção pela secção.
        let def = c
            .pin
            .and_then(|_| pinned.iter().find(|d| d.section == k))
            .or_else(|| cards.iter().find(|d| d.section == c.section));
        let copia;
        let def = match def {
            Some(d) if d.section != k => {
                copia = instancia(d, k);
                Some(&copia)
            }
            outro => outro,
        };
        let Some(card) = def else {
            super::solto_some::apaga(tela, k, d);
            out.push((k, None));
            continue;
        };
        // T61: o corpo do preso pinta-se com os estados do objecto dele.
        let trocados = loja.troca_estados_presos(k);
        let store: &WidgetStore = loja;
        let mut foto = super::solto_some::toma(k);
        let scene = &mut foto;
        let at = if c.grab.is_some() {
            c.at
        } else {
            preso(c.at, c.w, canvas)
        };
        let r = Rect::new(at.0, at.1, c.w, c.altura());
        let volta = loose_return_of(k);
        let ink = CardInk::of_card(store.section_theme(loose_face_of(k)), theme);
        let sombra = ink.color(ColorToken::KanbanShadowFlight);
        super::shadow(scene, r, m::CARD_RADIUS, sombra, m::LOOSE_SHADOW_FAR);
        super::shadow(
            scene,
            r,
            m::CARD_RADIUS,
            ink.color(ColorToken::KanbanFlightNear),
            m::LOOSE_SHADOW_NEAR,
        );
        let pinavel = store.is_card_pinnable(k);
        let mut acoes = Vec::with_capacity(2);
        if pinavel {
            acoes.push(HeadAction {
                id: loose_pin_of(k),
                icon: if c.rope {
                    IconId::CardPinRope
                } else {
                    IconId::CardPin
                },
                on: c.pin.is_some(),
            });
        }
        acoes.push(HeadAction {
            id: volta,
            icon: IconId::Close,
            on: false,
        });
        let sub = pinavel
            .then(|| c.pin.or_else(|| store.card_subject(panel)))
            .flatten()
            .and_then(|b| store.card_subject_name(b));
        let destaque = c
            .pin
            .filter(|p| Some(*p) != store.card_subject(panel))
            .map(|_| {
                let sobre = store
                    .last_pointer()
                    .is_some_and(|[px, py]| r.contains(px, py));
                piscar(store.clock_ns(), sobre)
            });
        let valores = if c.pin.is_some() {
            store.pinned_values(k)
        } else {
            &[]
        };
        let h = crate::widget::card_field::com_valores(valores, || {
            super::paint_card_with(
                scene,
                text_system,
                store,
                hit_index,
                panel,
                theme,
                r,
                card,
                false,
                super::Extra {
                    acoes,
                    sub,
                    destaque,
                    rola: Some(rola::Rola::de(k, canvas)),
                },
            )
        });
        // T39: as bordas por cima de tudo (o `hit` anda do fim para o princípio).
        let g = m::LOOSE_EDGE_GRIP;
        hit_index.register(
            loose_edge_of(k, LooseEdge::Left),
            Rect::new(at.0 - g * 0.5, at.1, g, h),
        );
        hit_index.register(
            loose_edge_of(k, LooseEdge::Right),
            Rect::new(at.0 + c.w - g * 0.5, at.1, g, h),
        );
        if let Some(t) = c.flash {
            let s = t.elapsed().as_secs_f64();
            if s < m::LOOSE_FLASH_S {
                #[allow(clippy::cast_possible_truncation)]
                let a = (1.0 - s / m::LOOSE_FLASH_S) as f32;
                let o = m::CARD_OUTLINE_GAP;
                crate::paint::stroke_rounded_rect(
                    scene,
                    Rect::new(r.x - o, r.y - o, r.w + o * 2.0, h + o * 2.0),
                    m::CARD_RADIUS + o,
                    StrokeToken::Thick.px(),
                    resolve(ColorToken::Accent, theme).multiply_alpha(a),
                );
            }
        }
        loja.repoe_estados(trocados);
        super::solto_some::cola(tela, k, foto, d);
        out.push((k, Some(Rect::new(at.0, at.1, c.w, h))));
    }
    out
}

/// T54: o cartão de `d` com o id da instância `k` (o cabeçalho, o arrasto e a dobra são dela; os
/// campos do corpo continuam os da secção — a pressão escolhe o solto de cima, T40).
fn instancia<'b>(d: &'b CardDef<'_>, k: NodeId) -> CardDef<'b> {
    CardDef {
        section: k,
        title: d.title,
        actions: d.actions.clone(),
        add: d.add,
        summary: d.summary.clone(),
        body: d
            .body
            .as_ref()
            .map(|f| -> CardBody<'b> { Box::new(move |c, y| f(c, y)) }),
    }
}

/// A ponta da corda no cartão `r` deste quadro: a borda mais perto do objecto (a outra ponta).
pub(super) fn na_borda(pontos: &[[f32; 2]], r: Rect) -> Option<(f32, f32)> {
    let &[x, y] = pontos.last()?;
    Some((x.clamp(r.x, r.x + r.w), y.clamp(r.y, r.y + r.h)))
}

/// Publica (ou larga) o rect de cada solto pintado por [`pinta_soltos`].
pub(super) fn publica(store: &mut WidgetStore, rects: Vec<(NodeId, Option<Rect>)>) {
    rola::publica(store);
    for (k, r) in rects {
        match r {
            Some(r) => {
                store.set_loose_card_rect(k, r);
                if let Some(c) = store.loose_card_mut(k) {
                    c.h = r.h; // T51: a altura é do cartão
                }
            }
            None => store.clear_panel_rect(loose_rect_of(k)),
        }
    }
}

/// Os dois cliques próprios: a linha-fantasma (traz o cartão à vista) e «Return to panel».
pub(super) fn clique(store: &mut WidgetStore, id: NodeId) -> bool {
    let soltos: Vec<NodeId> = store.loose_section_ids();
    for k in soltos {
        if id == loose_ghost_of(k) {
            store.raise_loose_card(k, true);
            return true;
        }
        if id == loose_return_of(k) {
            store.return_loose_card(k);
            return true;
        }
        if id == loose_pin_of(k) {
            store.toggle_loose_pin(k);
            return true;
        }
    }
    false
}

/// Os cartões soltos de `panel` sobre o canvas (e a lista suspensa aberta de um deles).
pub(super) fn pinta_os_soltos(
    ctx: &mut PaintCtx,
    panel: NodeId,
    theme: Theme,
    canvas: Rect,
    cards: &[CardDef<'_>],
    pinned: &[CardDef<'_>],
) {
    let canvas = rola::acima_do_hud(canvas, ctx.layout.bottom_hud);
    super::limita_janelinhas(canvas);
    // T61: a pintura troca estados no store — o índice de cliques sai e volta (a mesma ordem).
    let mut hits = std::mem::take(ctx.host.hit_index_mut());
    let rects = pinta_soltos(
        ctx.scene,
        ctx.text_system,
        ctx.host.store_mut(),
        &mut hits,
        panel,
        theme,
        canvas,
        cards,
        pinned,
    );
    *ctx.host.hit_index_mut() = hits;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    super::estilos::stash_aberta(store, hit_index, theme, cards);
    super::estilos::paint_pending(ctx.scene, ctx.text_system, hit_index, store);
    super::dropdown::paint_pending(ctx.scene, ctx.text_system, hit_index, store);
    publica(ctx.host.store_mut(), rects);
    super::solto_some::cordas(ctx.scene, ctx.host.store_mut(), panel, theme);
}

/// T53: a queda na lista — largou fora da coluna ⇒ solta (o suspenso volta com alfinete e corda);
/// encaixou ou cancelou ⇒ o suspenso acaba (a corda recolhe-se).
pub(super) fn larga(
    store: &mut WidgetStore,
    panel: NodeId,
    drop: Option<crate::interaction::CardDrop>,
    solta: Option<ph2d_card_stack::Flight<NodeId>>,
) {
    if let Some(f) = solta {
        store.detach_card(f.key, panel, f.ghost(), f.size.0);
    } else if let Some(d) = drop {
        store.settle_suspended_card(d.section);
    }
}

#[path = "cards_solto_rola.rs"]
pub(super) mod rola;

#[cfg(test)]
#[path = "cards_solto_tests.rs"]
mod tests;
