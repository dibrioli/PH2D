//! ⭐ **Um cartão** — o cabeçalho (pega, título, ações), o corpo e as sombras (em repouso e em
//! voo).

use super::*;

/// ⭐ **Um cartão** em `r` (a altura de `r` é a do quadro anterior). Devolve a altura medida.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_card(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    panel: NodeId,
    painel: Theme,
    r: Rect,
    card: &CardDef<'_>,
    rest_shadow: bool,
) -> f32 {
    paint_card_with(
        scene,
        text_system,
        store,
        hit_index,
        panel,
        painel,
        r,
        card,
        rest_shadow,
        Extra::default(),
    )
}

/// [`paint_card`] com o [`Extra`] do solto.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_card_with(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    panel: NodeId,
    painel: Theme,
    r: Rect,
    card: &CardDef<'_>,
    rest_shadow: bool,
    extra: Extra<'_>,
) -> f32 {
    let face = extra.rola.map_or(card.section, |k| k.face);
    let ink = CardInk::of_card(store.section_theme(face), painel);
    if rest_shadow {
        shadow(
            scene,
            r,
            m::CARD_RADIUS,
            ink.color(ColorToken::KanbanShadowNear),
            ph2d_card_stack::shadow::SHADOW_NEAR,
        );
        shadow(
            scene,
            r,
            m::CARD_RADIUS,
            ink.color(ColorToken::KanbanShadowRim),
            ph2d_card_stack::shadow::SHADOW_RIM,
        );
    }
    fill_rounded_rect(scene, r, m::CARD_RADIUS, ink.color(ColorToken::KanbanCard));

    // ── O cabeçalho = a pega (e o menu do botão direito da secção) ──
    // O cartão Object não tem título na lista; solto, tem cabeçalho para o alfinete e o «×».
    let sem_titulo = card.title.is_empty() && extra.acoes.is_empty();
    let head_h = if sem_titulo {
        m::CARD_PAD_TOP
    } else {
        m::CARD_PAD_TOP + m::CARD_HEAD_H
    };
    let head = Rect::new(r.x, r.y, r.w, head_h);
    let ix = r.x + m::CARD_PAD_X;
    let iw = r.w - m::CARD_PAD_X * 2.0;
    let hy = r.y + m::CARD_PAD_TOP;
    let dobrado = store.is_collapsed(card.section);
    let ausente = card.add.is_some();
    let mut acoes: Vec<HeadAction> = card.actions.clone();
    if let Some(add) = card.add {
        acoes.push(HeadAction {
            id: add,
            icon: IconId::Add,
            on: false,
        });
    }
    acoes.extend(extra.acoes);
    let mut ax = ix + iw;
    // ⚠️ Os ícones registam-se DEPOIS do cabeçalho (o `hit` anda do fim para o princípio: o
    // cabeçalho registado a seguir cobria-os — `HitIndex::register`, «Pitfall»).
    let mut icones: Vec<(NodeId, Rect)> = Vec::with_capacity(acoes.len());
    for a in acoes.iter().rev() {
        ax -= m::HEAD_ICON_BOX;
        let b = Rect::new(
            ax,
            hy + (m::CARD_HEAD_H - m::HEAD_ICON_BOX) * 0.5,
            m::HEAD_ICON_BOX,
            m::HEAD_ICON_BOX,
        );
        icones.push((a.id, b));
        dicas::icone(a.id);
        if a.on || store.hover_live(a.id) > 0.0 {
            fill_rounded_rect(
                scene,
                b,
                m::HEAD_ICON_RADIUS,
                ink.color(ColorToken::CardFieldHover),
            );
        }
        let g = m::HEAD_ICON_GLYPH;
        let gr = Rect::new(b.x + (b.w - g) * 0.5, b.y + (b.h - g) * 0.5, g, g);
        let tinta = if a.on {
            ColorToken::Text1
        } else {
            ColorToken::Text2
        };
        paint_icon(
            scene,
            a.icon,
            gr,
            ink.color(tinta),
            StrokeToken::Default.px(),
        );
    }
    let grip = Rect::new(head.x, head.y, (ax - head.x).max(0.0), head.h);
    hit_index.register_section(card.section, head, grip_of(card.section), grip);
    for (id, b) in icones {
        hit_index.register(id, b);
    }
    // Dobrado: «›» antes do título, e o título um tom abaixo (maquete C).
    let seta = dobrado && !sem_titulo;
    let tx = if seta {
        let g = m::CARD_CHEVRON;
        paint_icon(
            scene,
            IconId::ChevronRight,
            Rect::new(ix, hy + (m::CARD_HEAD_H - g) * 0.5, g, g),
            ink.color(ColorToken::Text3),
            StrokeToken::Default.px(),
        );
        ix + g + m::CARD_CHEVRON_GAP
    } else {
        ix
    };
    let mut titulo_w = (ax - tx).max(0.0);
    if let Some(sub) = extra.sub.filter(|s| !s.is_empty() && !sem_titulo) {
        let sw = text_system
            .prefix_width(sub, m::LABEL_PX)
            .min(titulo_w * 0.5);
        crate::paint::paint_text(
            text_system,
            scene,
            sub,
            ax - sw - m::FIELD_GAP,
            hy + crate::paint::folga_do_texto(m::CARD_HEAD_H, m::LABEL_PX),
            m::LABEL_PX,
            sw,
            extra.destaque.map_or(ink.color(ColorToken::Text3), |a| {
                ink.color(ColorToken::Accent).multiply_alpha(a)
            }),
        );
        titulo_w = (titulo_w - sw - m::FIELD_GAP * 2.0).max(0.0);
    }
    let tinta = if ausente {
        ColorToken::Text3
    } else if seta {
        ColorToken::Text2
    } else {
        ColorToken::Text1
    };
    crate::paint::paint_text_title_elided_tracked(
        text_system,
        scene,
        card.title,
        [
            tx,
            hy + crate::paint::folga_do_texto(m::CARD_HEAD_H, m::CARD_TITLE_PX),
        ],
        m::CARD_TITLE_PX,
        titulo_w,
        ink.color(tinta),
        0.0,
    );
    if dobrado && let Some(s) = card.summary.as_deref() {
        let sw = text_system.prefix_width(s, m::LABEL_PX);
        crate::paint::paint_text(
            text_system,
            scene,
            s,
            ax - sw - m::FIELD_GAP,
            hy + crate::paint::folga_do_texto(m::CARD_HEAD_H, m::LABEL_PX),
            m::LABEL_PX,
            sw,
            ink.color(ColorToken::Text3),
        );
    }
    let mut fundo = if sem_titulo { hy } else { hy + m::CARD_HEAD_H };

    // ── O corpo (os campos e as notas, D6) ──
    if !dobrado && !ausente && card.body.is_some() {
        let topo = fundo + if sem_titulo { 0.0 } else { m::CARD_HEAD_BOTTOM };
        let (s, t) = (&mut *scene, &mut *text_system);
        fundo = solto::rola::corpo(
            s, t, store, hit_index, panel, painel, ink, r, card, topo, extra.rola,
        );
        fundo += m::CARD_PAD_BOTTOM;
    } else {
        fundo += m::CARD_PAD_TOP;
    }
    let h = fundo - r.y;
    if let Some(cor) = store.section_outline_color(face) {
        let o = m::CARD_OUTLINE_GAP;
        let caixa = Rect::new(r.x - o, r.y - o, r.w + o * 2.0, h + o * 2.0);
        crate::widget::showcase::notes_chrome::contorno_do_cartao(
            scene,
            cor,
            caixa,
            m::CARD_RADIUS + o,
        );
    }
    h
}

pub(super) fn shadow(
    scene: &mut VectorScene,
    r: Rect,
    radius: f32,
    color: ph2d_vector::Color,
    (dy, blur): (f64, f64),
) {
    let rr = ph2d_vector::Rect::new(
        f64::from(r.x),
        f64::from(r.y) + dy,
        f64::from(r.x + r.w),
        f64::from(r.y + r.h) + dy,
    );
    scene.blurred_rounded_rect(Affine::IDENTITY, rr, color, f64::from(radius), blur / 2.0);
}

pub(super) fn flight_shadow(scene: &mut VectorScene, theme: Theme, r: Rect) {
    use ph2d_card_stack::shadow as s;
    let far = if theme.is_dark() {
        s::SHADOW_FLIGHT_DARK
    } else {
        s::SHADOW_FLIGHT_LIGHT
    };
    shadow(
        scene,
        r,
        m::CARD_RADIUS,
        resolve(ColorToken::KanbanShadowFlight, theme),
        far,
    );
    shadow(
        scene,
        r,
        m::CARD_RADIUS,
        resolve(ColorToken::KanbanFlightNear, theme),
        s::SHADOW_FLIGHT_NEAR,
    );
}
