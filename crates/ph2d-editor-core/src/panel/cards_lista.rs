//! ⭐⭐ **A lista de cartões** — a pintura do painel inteiro: cabeçalho da lista, a pilha que
//! se arrasta, os soltos presos, o pé e a rolagem.

use super::*;

/// [`paint_card_list_with`] e os cartões dos soltos PRESOS a outro sujeito (T40): a lista mostra
/// os da seleção e o solto o dele — a mesma secção nos dois sítios.
pub fn paint_card_list_pinned(
    ctx: &mut PaintCtx,
    panel: NodeId,
    title: &str,
    cards: &[CardDef<'_>],
    footer: Option<Footer<'_>>,
    pinned: &[CardDef<'_>],
) -> Rect {
    paint_card_list_top(ctx, panel, title, cards, footer, pinned, None)
}

/// O que uma lista prende debaixo do título e não rola com os cartões (a busca da Hierarquia).
#[derive(Copy, Clone)]
pub struct ListTop<'a> {
    /// A altura da faixa, sem o vão até aos cartões.
    pub h: f32,
    /// Pinta a faixa em `Rect` (a coluna dos cartões), na tinta da lista.
    pub body: &'a dyn Fn(&mut CardCtx<'_>, Rect),
    /// O «+» do cabeçalho da lista, sem pé: `(id, dica)`.
    pub add: Option<(NodeId, &'a str)>,
}

/// A mão que arrasta a `y`, perto de uma borda da banda visível da lista, rola-a (05 §6).
pub fn scroll_at_edge(store: &mut WidgetStore, panel: NodeId, y: f32, top: f32) {
    rolagem::na_borda(store, panel, y, top);
}

/// [`paint_card_list_pinned`] com a faixa presa.
pub fn paint_card_list_top(
    ctx: &mut PaintCtx,
    panel: NodeId,
    title: &str,
    cards: &[CardDef<'_>],
    footer: Option<Footer<'_>>,
    pinned: &[CardDef<'_>],
    preso: Option<ListTop<'_>>,
) -> Rect {
    let theme = ctx.host.theme();
    dicas::comeca(panel);
    // A lista curta do convidado (`card_guest_ops`): sem título nem rolagem, por cima do anfitrião.
    let curta = ctx.host.store().in_card_guest_pass(panel);
    let footer = footer.filter(|_| !curta);
    let slot = ctx.slot;
    let area = ctx.host.content_area().filter(|c| c.h > 0.0);
    // D31, espelho: na coluna da esquerda a lista começa depois da régua vertical, que passa a morar
    // na borda da janela. Fora da metade do encaixe não é a régua da coluna (a lista ainda não flutua).
    let x0 = area.map_or(slot.x, |c| {
        if c.x > slot.x && c.x < slot.x + slot.w * 0.5 {
            c.x
        } else {
            slot.x
        }
    });
    let x = x0 + m::LIST_MARGIN;
    let w = (slot.x + slot.w - x - m::LIST_MARGIN).max(0.0);
    // A barra de opções da ferramenta atravessa a janela por cima dos painéis: a lista começa abaixo.
    let barra = ctx.layout.fila_que_cobre_o_topo();
    let topo = if barra.h > 0.0 {
        slot.y.max(barra.y + barra.h)
    } else {
        slot.y
    };
    // D31: e abaixo da régua de cima (a lista flutua sobre a área de desenho, D24).
    let topo = area.map_or(topo, |c| topo.max(c.y));
    let convidado_h = ctx.host.store().card_guest_h_over(panel);
    let topo = if convidado_h > 0.0 {
        topo + convidado_h + m::LIST_MARGIN
    } else {
        topo
    };
    let top_list = topo + m::LIST_MARGIN;
    let preso_h = preso.map_or(0.0, |p| p.h + m::LIST_TOP_GAP);
    let cabeca = if curta {
        m::LIST_PAD_BOTTOM
    } else {
        m::LIST_HEAD_H
    };
    let body_top = top_list + cabeca + preso_h;
    let keys: Vec<NodeId> = cards.iter().map(|c| c.section).collect();
    let store = ctx.host.store_mut();
    // Na lista curta, só o que o convidado declarou; os soltos dos outros cartões continuam dele.
    let da_lista: Vec<NodeId> = keys
        .iter()
        .copied()
        .filter(|k| !curta || store.card_guest_only().contains(k))
        .collect();
    let cw = if curta {
        w - m::LIST_PAD_X * 2.0
    } else {
        rolagem::largura_dos_cartoes(store, panel, x, w)
    };
    // A rolagem (a lista que passa da janela): a origem dos cartões sobe o que rolou.
    let origem = if curta {
        body_top
    } else {
        body_top - store.panel_scroll(panel).max(0.0)
    };
    for k in &keys {
        store.mark_card_section(*k);
    }
    // A ordem: a das secções (`sections.txt`), sobre a ordem natural do painel.
    let mut ordem = crate::widget::section_plan::ordem(&da_lista, store);
    let drop = store.take_card_drop(panel, &keys);
    let drag = store
        .card_drag()
        .filter(|d| d.active && store.card_drag_is_of(d, panel, &keys));
    // ⭐ C5: a mão fora da COLUNA leva o cartão ao canvas; o solto que volta a uma lista que o mostra
    // é levantado como voo dela. Aba atrás: a lista não se vê, só os soltos continuam no canvas.
    let escondida = store.loose_only_pass() == Some(panel);
    let na_lista = |p: (f32, f32)| !escondida && p.0 >= x && p.0 <= x + w;
    let canvas = ctx.layout.canvas;
    let era_solto = |s: &WidgetStore, k: NodeId| s.is_loose_in(k, panel);
    let mut reentra = None;
    if let Some(d) = drag
        && d.loose
        && era_solto(store, d.section)
    {
        let cabe = na_lista(d.at) && da_lista.contains(&store.loose_section(d.section));
        reentra = solto::arrasta(store, d, cabe, canvas);
    }
    let drop = match drop {
        Some(d) if d.loose && era_solto(store, d.section) => {
            store.settle_loose_card(d.section);
            None
        }
        outra => outra,
    };
    let drag = drag.filter(|d| !d.loose);
    if let Some(d) = drag.filter(|d| !curta && na_lista(d.at)) {
        rolagem::na_borda(store, panel, d.at.1, body_top);
    }
    let todos: Vec<NodeId> = keys
        .iter()
        .copied()
        .filter(|k| era_solto(store, *k))
        .collect();
    solto::prepara(store, &todos, cards);
    estilos::prepara(store);
    if escondida {
        solto::pinta_os_soltos(ctx, panel, theme, canvas, cards, pinned);
        return Rect::new(x, top_list, 0.0, 0.0);
    }
    // T40: o solto preso a OUTRO sujeito não deixa fantasma — a lista mostra a secção da seleção.
    let soltos = solto::fantasmas(store, panel, &todos);
    let cx = x + m::LIST_PAD_X;
    let chegadas: Vec<(NodeId, (f32, f32))> = keys
        .iter()
        .filter_map(|k| {
            store
                .take_card_arrival(*k)
                .map(|a| (*k, (a.0 - cx, a.1 - origem)))
        })
        .collect();
    let law = crate::motion::UiMotion::default().law(crate::motion::Role::Travel);
    let list = store.card_list_mut(panel);
    let now = std::time::Instant::now();
    let dt = crate::aquecimento::passo_do_quadro(
        list.last
            .map_or(0.0, |t| now.duration_since(t).as_secs_f64().min(0.1)),
    );
    list.last = Some(now);
    let soltas_h = list.loose_notes_h;
    // A queda ANTES da disposição: o cartão assenta já no lugar novo, sem passar pelo velho.
    let mut nova_ordem = None;
    let mut solta = None;
    if let Some(d) = drop {
        if d.cancelled || list.stack.flight().is_some_and(|f| soltos.contains(&f.key)) {
            list.stack.cancel(origem);
        } else if list.stack.flight().is_some_and(|f| f.to.is_none()) {
            solta = list.stack.detach();
        } else if let Some((k, to)) = list.stack.land(origem) {
            reorder(&mut ordem, &k, to);
            nova_ordem = Some(ordem.clone());
        }
    }
    // As alturas com mola (T41: dobrar, desdobrar, entrar e sair não saltam).
    let mut items = entra_sai::itens(panel, &mut list.stack, &list.items, &ordem, &soltos);
    let crescem = list.stack.smooth_heights(&mut items, dt, law);
    if let Some((d, (grab, size))) = reentra {
        let i = items.iter().position(|i| i.key == d.section).unwrap_or(0);
        list.stack.lift(d.section, d.at, grab, size, i);
    }
    let drag = drag.or(reentra.map(|(d, _)| d));
    if let Some(d) = drag {
        if list.stack.flight().is_none() {
            let mut y = origem;
            for it in &items {
                if it.key == d.section {
                    let i = items.iter().position(|i| i.key == d.section).unwrap_or(0);
                    list.stack.lift(
                        d.section,
                        d.down,
                        (d.down.0 - x, d.down.1 - y),
                        (cw, it.h),
                        i,
                    );
                    break;
                }
                y += it.h + m::CARD_GAP;
            }
        }
        list.stack
            .drag(&items, m::CARD_GAP, d.at, na_lista(d.at).then_some(origem));
    }
    for (k, de) in chegadas {
        list.stack.arrive(k, de);
    }
    let placed = list.stack.place(&items, m::CARD_GAP, dt, law);
    let regressam: Vec<(NodeId, f32)> = items
        .iter()
        .filter_map(|i| list.stack.arriving_dx(i.key).map(|d| (i.key, d)))
        .collect();
    let voo = list.stack.flight().copied();
    let tilt = list.stack.tilt_deg();
    solto::larga(store, panel, drop, solta);
    if let Some(o) = nova_ordem {
        let atual = store.section_order().to_vec();
        let mut todas: Vec<NodeId> = atual
            .into_iter()
            .filter(|k| !da_lista.contains(k))
            .collect();
        todas.extend(o);
        store.set_section_order(todas);
    }

    // ── A lista: flutua, abraça o conteúdo até à altura do encaixe (D16) ──
    let pe = if footer.is_some() {
        m::LIST_FOOTER_H
    } else {
        0.0
    };
    let content_h = placed.content_h + soltas_h + pe;
    let natural = cabeca + preso_h + content_h + m::LIST_PAD_BOTTOM;
    let tecto = slot.y + slot.h - top_list - m::LIST_MARGIN;
    let list_h = natural.min(tecto);
    let lista = Rect::new(x, top_list, w, list_h.max(cabeca));
    // ⭐ D24: a lista FLUTUA — publica o rect DELA (o clique abaixo vai ao canvas) sem reservar a
    // coluna; a costura de largura passa para a beirada dela (`DockSides::with_floating`).
    ctx.host.store_mut().set_floating_panel_rect(panel, lista);
    if curta {
        // Sem cartão na lista, ela não ocupa nada: nem fundo, nem vão, nem clique.
        let vazia = placed.ys.is_empty();
        let h = if vazia { 0.0 } else { lista.h };
        ctx.host.store_mut().set_card_guest_h(h);
        if vazia {
            ctx.host.store_mut().drop_card_guest_rect(panel);
        }
    }
    let vazia_curta = curta && placed.ys.is_empty();
    if !vazia_curta {
        shadow(
            ctx.scene,
            lista,
            m::LIST_RADIUS,
            resolve(ColorToken::KanbanListShadow, theme),
            m::LIST_SHADOW_FAR,
        );
        shadow(
            ctx.scene,
            lista,
            m::LIST_RADIUS,
            resolve(ColorToken::KanbanShadowNear, theme),
            m::LIST_SHADOW_NEAR,
        );
        fill_rounded_rect(
            ctx.scene,
            lista,
            m::LIST_RADIUS,
            resolve(ColorToken::KanbanList, theme),
        );
    }
    if !curta {
        let nome = title.to_uppercase();
        let vao = w
            - m::LIST_PAD_X * 2.0
            - m::HEAD_ICON_BOX
                * (1.0
                    + f32::from(u8::from(
                        footer.is_some_and(|f| f.at_top) || preso.is_some_and(|p| p.add.is_some()),
                    ))
                    + f32::from(u8::from(ctx.host.store().card_list_close(panel).is_some())));
        let (corpo, tracking) = super::banda::titulo_que_cabe(ctx.text_system, &nome, vao);
        crate::paint::paint_text_title_elided_tracked(
            ctx.text_system,
            ctx.scene,
            &nome,
            [
                x + m::LIST_PAD_X + m::LIST_HEAD_PAD_X,
                top_list + crate::paint::folga_do_texto(m::LIST_HEAD_H, corpo),
            ],
            corpo,
            vao,
            resolve(ColorToken::KanbanListFg, theme),
            tracking,
        );
        // ⭐ D21: o ícone de menu do painel (Reset Positions · Reset Colors · Reset All).
        let do_cabecalho = footer.or(preso.and_then(|p| p.add).map(|(id, text)| Footer {
            id,
            text,
            at_top: true,
        }));
        menu::pinta(
            ctx,
            panel,
            theme,
            Rect::new(x, top_list, w, m::LIST_HEAD_H),
            do_cabecalho,
        );
    }
    if let Some(p) = preso {
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        let ink = CardInk::of_list(theme);
        let mut c = CardCtx::flutuante(ctx.scene, ctx.text_system, store, hit_index, ink, cx, cw);
        (p.body)(&mut c, Rect::new(cx, top_list + m::LIST_HEAD_H, cw, p.h));
    }

    // ── Os cartões ── (T43: o corpo, e com ele a pista da barra, pára acima da margem do fundo)
    let clip = Rect::new(
        x,
        body_top,
        w,
        (lista.y + lista.h - m::LIST_PAD_BOTTOM - body_top).max(0.0),
    );
    let mut voltam = Vec::new();
    let mut alturas: Vec<Item<NodeId>> = Vec::with_capacity(cards.len());
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let area = (!curta).then(|| {
        crate::widget::scroll_area::open_with(
            ctx.scene,
            hit_index,
            store,
            panel,
            scroll_bar_of(panel),
            clip,
        )
    });
    if let Some((sy, sh)) = placed.slot {
        let vao = Rect::new(cx, origem + sy, cw, sh);
        fill_rounded_rect(
            ctx.scene,
            vao,
            m::CARD_RADIUS,
            resolve(ColorToken::KanbanSlot, theme),
        );
    }
    for (k, y) in &placed.ys {
        let h_antes = items.iter().find(|i| i.key == *k).map_or(0.0, |i| i.h);
        let r = Rect::new(cx, origem + y, cw, h_antes.max(0.0));
        let Some(card) = cards.iter().find(|c| c.section == *k) else {
            entra_sai::pinta_saida(ctx.scene, panel, *k, Rect { h: h_antes, ..r });
            continue;
        };
        if let Some(dx) = regressam.iter().find(|(c, _)| c == k).map(|(_, d)| *d) {
            voltam.push((Rect::new(r.x + dx, r.y, r.w, r.h), card));
            continue;
        }
        if soltos.contains(k) {
            solto::fantasma(ctx.scene, ctx.text_system, store, hit_index, theme, r, card);
            alturas.push(Item {
                key: *k,
                h: m::LOOSE_GHOST_H,
            });
            continue;
        }
        // O cartão cabe na altura VISTA: a crescer ou a encolher, o corpo recorta-se nela.
        let o = if crescem.contains(k) {
            0.0
        } else {
            m::CARD_OUTLINE_GAP + StrokeToken::Thick.px()
        };
        let corte = Rect::new(r.x - o, r.y - o, r.w + o * 2.0, r.h + o * 2.0);
        ctx.scene.push_clip(&crate::paint::rect_to_vello(corte));
        hit_index.push_clip(corte);
        let h = entra_sai::fotografa(ctx.scene, panel, *k, r, |s| {
            paint_card(
                s,
                ctx.text_system,
                store,
                hit_index,
                panel,
                theme,
                r,
                card,
                true,
            )
        });
        hit_index.pop_clip();
        ctx.scene.pop_layer();
        alturas.push(Item { key: *k, h });
    }
    // As notas sem cartão (criadas sobre a lista, fora de um cartão) e o fantasma da nota
    // arrastada: sem isto a lista não se declara anfitriã e o botão direito não oferece «Create Note».
    let topo_soltas = origem + placed.content_h;
    let fim_soltas = if curta {
        topo_soltas
    } else {
        crate::widget::showcase::notes_chrome::pinta_as_que_sobram(
            ctx.scene,
            ctx.text_system,
            hit_index,
            store,
            panel,
            theme,
            cx,
            cw,
            topo_soltas,
        )
    };
    let notas_h = (fim_soltas - topo_soltas).max(0.0);
    if let Some(f) = footer {
        let fr = Rect::new(cx, topo_soltas + soltas_h, cw, m::LIST_FOOTER_H);
        hit_index.register(f.id, fr);
        if store.hover_live(f.id) > 0.0 {
            fill_rounded_rect(
                ctx.scene,
                fr,
                m::FIELD_RADIUS,
                resolve(ColorToken::CardFieldHover, theme),
            );
        }
        let g = m::ICON_GLYPH;
        let tinta = resolve(ColorToken::KanbanListFg, theme);
        paint_icon(
            ctx.scene,
            IconId::Add,
            Rect::new(fr.x + m::CARD_PAD_X, fr.y + (fr.h - g) * 0.5, g, g),
            tinta,
            StrokeToken::Default.px(),
        );
        crate::paint::paint_text(
            ctx.text_system,
            ctx.scene,
            f.text,
            fr.x + m::CARD_PAD_X + g + m::SEG_ICON_GAP,
            fr.y + crate::paint::folga_do_texto(fr.h, m::VALUE_PX),
            m::VALUE_PX,
            fr.w,
            tinta,
        );
    }
    let rolagem = area.map(|area| {
        crate::widget::scroll_area::close_parts(
            area,
            ctx.scene,
            hit_index,
            store,
            rolagem::conteudo(placed.content_h + notas_h + pe, natural <= tecto),
            theme,
        )
    });

    // ── O que voa: por cima de tudo, inclinado, sem registar cliques ──
    if let Some(f) = voo
        && let Some(card) = cards.iter().find(|c| c.section == f.key)
    {
        let (gx, gy) = f.ghost();
        let local = Rect::new(0.0, 0.0, f.size.0, f.size.1);
        let mut s = VectorScene::new();
        let mut nada = HitIndex::default();
        flight_shadow(&mut s, theme, local);
        let h = paint_card(
            &mut s,
            ctx.text_system,
            store,
            &mut nada,
            panel,
            theme,
            local,
            card,
            false,
        );
        alturas.push(Item { key: f.key, h });
        let pivot = ph2d_card_stack::flight_pivot(local.w, local.h);
        let t = Affine::translate((f64::from(gx), f64::from(gy)))
            * Affine::translate(pivot)
            * Affine::rotate(tilt.to_radians())
            * Affine::translate((-pivot.0, -pivot.1));
        ctx.scene.append_transformed(&s, t);
    }
    for (r, card) in voltam {
        flight_shadow(ctx.scene, theme, r);
        let mut nada = HitIndex::default();
        let h = paint_card(
            ctx.scene,
            ctx.text_system,
            ctx.host.store_mut(),
            &mut nada,
            panel,
            theme,
            r,
            card,
            false,
        );
        alturas.push(Item {
            key: card.section,
            h,
        });
    }
    solto::pinta_os_soltos(ctx, panel, theme, canvas, cards, pinned);
    super::linhas_extra::voo::pinta_os_voos(ctx.scene);
    if let Some(rolagem) = rolagem {
        rolagem.publish(ctx.host.store_mut());
    }
    let list = ctx.host.store_mut().card_list_mut(panel);
    list.items = alturas;
    list.loose_notes_h = notas_h;
    lista
}
