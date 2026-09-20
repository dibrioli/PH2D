//! **COMO SE DESENHA UM CARTÃO** — a moldura, o cabeçalho, a faixa de params, o véu e o selo.
//!
//! Irmão de [`super`] (`paint`) por RESPONSABILIDADE: o pai decide **o que o painel pinta**
//! (o que está no ecrã, em que ordem, com que fundo); este responde **como UM nó se desenha**.
//!
//! ⚠️⚠️ **O corte foi imposto pela ÁRVORE COMBINADA, e nenhuma das duas linhas o via sozinha.**
//! A `line/UIUX` levou o `paint.rs` a `~570` (a porta da moldura do tema) e a
//! `line/motion-value` a `~600` (os params dentro do cartão); fundidas, o ficheiro ficou em
//! **617** sobre um teto de **600**, e o portão de cada linha estava verde.
//! *Um teto de LOC é a única espécie de vermelho que só a SOMA produz* — e a cura é o corte,
//! nunca a lista de tolerância.

use super::*;
use crate::state::PreviewPos;

/// Draw one node card; returns its screen-space body rect (for hit registration).
pub(super) fn draw_card(
    ctx: &mut PaintCtx,
    state: &MotionGraphPanelState,
    n: &GraphNodeView,
    view: &View,
    theme: Theme,
    veiled: bool,
    // ⚠️ **O lado do retrato chega DECIDIDO** e não é perguntado aqui: a lei
    // (`crate::geom::retratos_em_cima`) responde sobre a TELA inteira e este pintor vê um cartão
    // de cada vez — perguntá-la por cartão seria `O(cartões²)` por CARTÃO.
    lado_do_retrato: PreviewPos,
) -> Rect {
    // ⚠️ **O rect vem da GEOMETRIA** (`card_rect`), e não de `CARD_W`: desde 2026-09-20 uma
    // cápsula é tão larga quanto o nome dela, e o pintor tem de desenhar exactamente o rect que o
    // hit-test regista — *duas contas para a mesma caixa divergem no dia em que uma delas mudar*.
    let body = geom::card_rect(n, view);
    let (sx, sy) = (body.x, body.y);
    let w = body.w;
    let h = body.h;
    let r = CARD_RADIUS * view.zoom;

    // ⭐⭐⭐ **A CÁPSULA** (ordem do dono, 2026-09-19) — abaixo do limiar do texto o nó deixa de ser
    // um cartão e passa a ser uma pastilha da cor do grupo, com o nome a enchê-la. Ver
    // [`crate::paint_capsula`] para o desenho e para o que ele deixa de fora.
    //
    // ⚠️ **A saída é AQUI, antes de tudo o resto**, e não um `if` por peça lá dentro: o que a
    // cápsula esconde são os params, o readout, o selo de papel, os nomes das portas, o badge e a
    // moldura da pré-visualização — *nove ramos que teriam de concordar, e o primeiro que alguém
    // esquecesse desenharia fora da pastilha.*
    if geom::detalhe(view) == geom::Detalhe::Capsula
        && n.kind != crate::snapshot::NodeViewKind::Subgraph
    {
        return super::paint_capsula::draw_capsula(ctx, state, n, view, theme, body);
    }

    // **A collapsed subgraph draws as a STACK** (doc 57): two cards peeking out from
    // behind the front one. It is the universal idiom for "there is more than one
    // thing here", it needs no icon and no legend, and it is the only difference
    // between a card that opens and a card that does not — which is exactly the
    // difference a double-click depends on the artist knowing.
    if n.kind == crate::snapshot::NodeViewKind::Subgraph {
        for step in [2.0, 1.0] {
            let off = STACK_OFFSET * step * view.zoom;
            let back = Rect::new(sx + off, sy - off, w, h);
            fill_rounded_rect(ctx.scene, back, r, resolve(ColorToken::Bg3, theme));
            ph2d_editor_core::paint::stroke_frame(
                ctx.scene,
                back,
                r,
                theme,
                ph2d_tokens::visuals::Feel::Rest,
                1.0,
                resolve(ColorToken::Border, theme),
            );
        }
    }

    fill_rounded_rect(ctx.scene, body, r, resolve(ColorToken::Bg2, theme));
    let header = Rect::new(sx, sy, w, geom::HEADER_H * view.zoom);
    fill_rounded_rect(ctx.scene, header, r, resolve(cat_token(n.category), theme));
    // ⭐ Pela porta do TEMA: o nó em repouso não tem borda num tema moderno (o `GraphNode` do
    //    Godot Modern tem `border_width 0`), e a SELECÇÃO é a moldura que sobrevive — 2 px em
    //    `mono`, como o `gn_panel_selected_style` dele. O raio `r` fica: escala com o zoom.
    ph2d_editor_core::paint::stroke_frame(
        ctx.scene,
        body,
        r,
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        1.0,
        resolve(ColorToken::Border, theme),
    );
    if state.selected.contains(&n.id) {
        ph2d_editor_core::paint::stroke_frame(
            ctx.scene,
            body,
            r,
            theme,
            ph2d_tokens::visuals::Feel::Selected,
            2.0,
            resolve(ColorToken::Accent, theme),
        );
    }
    // ⭐⭐⭐ **O REALCE DE UMA LARGADA** — o alvo que vai trocar de lugar (enquanto a mão paira) e
    // o eco de depois (a desvanecer), pela MESMA porta e com a mesma cor: a decisão vive em
    // [`crate::realce`], porque um `if` aqui só se deixa gatear por texto (medido).
    if let Some(forca) = crate::realce::realce_do_cartao(state, n.id) {
        ph2d_editor_core::paint::stroke_frame(
            ctx.scene,
            body,
            r,
            theme,
            ph2d_tokens::visuals::Feel::Selected,
            // ⚠️ Mais grosso do que o anel de selecção (`Thick`, 2 px): enquanto o gesto está
            // vivo, o que o artista precisa de ler é o ALVO.
            ph2d_tokens::StrokeToken::Heavy.px(),
            resolve(ColorToken::Success, theme).multiply_alpha(forca),
        );
    }

    // ⭐⭐⭐ **O SELO DE PAPEL** — o que este nó É no grafo (fonte · decisão · junção ·
    // terminal · I/O), desenhado no cabeçalho. Ver [`role_glyph`].
    let role = role_inset_px(n.silhouette);
    role_glyph(ctx, n, view, theme);

    // A stamped node's title clips short of its header toggle (doc 86).
    let toggle_w = geom::PREVIEW_TOGGLE_W * f32::from(u8::from(n.preview.is_some()));
    paint_text_title_elided(
        ctx.text_system,
        ctx.scene,
        &n.display_name,
        sx + (TITLE_PAD_X + role) * view.zoom,
        sy + TITLE_PAD_Y * view.zoom,
        TITLE_SIZE * view.zoom,
        w - (TITLE_INSET_R + toggle_w + role) * view.zoom,
        resolve(ColorToken::Text1, theme),
    );

    for (i, p) in n.inputs.iter().enumerate() {
        let (cx, cy) = socket_center(n, view, false, i);
        paint_socket_glyph(ctx, cx, cy, SOCKET_R * view.zoom, p, theme);
    }
    for (i, p) in n.outputs.iter().enumerate() {
        let (cx, cy) = socket_center(n, view, true, i);
        paint_socket_glyph(ctx, cx, cy, SOCKET_R * view.zoom, p, theme);
    }

    // **O NOME de cada porta**, na faixa que o cartão já reservava e deixava em branco (report
    // do Enio, 2026-08-27) — ver `paint_port_label`. Depois dos glifos, porque o texto é o que
    // tem de ganhar quando os dois disputam o mesmo pixel.
    draw_port_labels(ctx, n, view, theme);

    // ⭐ **A faixa de params** (ciclo 1, doc 103): o que o cartão CONTROLA, sob os sockets.
    draw_card_params(ctx, n, view, theme);

    // The inline readout: what this card produced on this frame's cook, under its sockets.
    // Text2 (the muted tone), not Text1 — it is a live instrument reading, not a label the
    // artist authored, and it must not compete with the node's own name.
    if let Some(text) = &n.readout {
        let row_y = sy + geom::readout_top(n) * view.zoom;
        paint_text_title_elided(
            ctx.text_system,
            ctx.scene,
            text,
            sx + TITLE_PAD_X * view.zoom,
            row_y + READOUT_PAD_Y * view.zoom,
            READOUT_SIZE * view.zoom,
            w - TITLE_INSET_R * view.zoom,
            resolve(ColorToken::Text2, theme),
        );
    }

    let pos = lado_do_retrato;
    draw_preview(ctx, n, view, theme, pos);
    draw_preview_toggle(ctx, n, view, theme, pos);

    // **Veiled** — the card is not part of what the artist is looking at. Two reasons, ONE
    // veil (a card veiled twice is just darker, and the artist cannot read "why" out of a
    // shade):
    //
    // - **Inert**: no sink reaches it, so the cook never pulls it and nothing it does reaches
    //   the canvas. The reading is REACHABILITY (F3), not "has a readout" (F2): a node cooked
    //   inside a scoped lane (`motion.time_remap`) has no root-lane memo and therefore no
    //   number — it is consumed all the same, and veiling it would be a lie it could not argue
    //   with.
    // - **Out of the influence**: something is selected, and this card neither feeds it nor is
    //   fed by it. Esc drops the selection and the whole canvas comes back.
    //
    // A veil, not a repaint: the card keeps its category colour, its title and its sockets, and
    // stays perfectly grabbable. It recedes; it does not become a different kind of thing.
    // (The selection ring is drawn ABOVE it — a selected node must still look selected.)
    if veiled {
        fill_rounded_rect(ctx.scene, body, r, resolve(ColorToken::GraphInert, theme));
        if state.selected.contains(&n.id) {
            ph2d_editor_core::paint::stroke_frame(
                ctx.scene,
                body,
                r,
                theme,
                ph2d_tokens::visuals::Feel::Selected,
                2.0,
                resolve(ColorToken::Accent, theme),
            );
        }
    }

    // **Switched OFF** (bypass/mute — H). A bypassed node IS inert — nothing flows through it — so
    // it takes the same veil as an unwired card; the STRIKE, corner to corner, is what says the
    // inertness is DELIBERATE (you muted it) rather than an accident (you forgot to wire it). It
    // sits above the veil and the selection ring so a muted node reads as muted even when selected.
    if n.bypassed {
        fill_rounded_rect(ctx.scene, body, r, resolve(ColorToken::GraphInert, theme));
        let strike = bypass_strike(body);
        stroke_polyline(
            ctx.scene,
            &strike,
            BYPASS_STRIKE_W * view.zoom,
            resolve(ColorToken::Text2, theme),
        );
    }
    // The ⚠ inert badge sits ON TOP of the card corner (ADR-0155); a healthy node has
    // `n.inert` false, so it draws nothing. A ghost is never inert (`veiled` is its whole
    // message), so it grows no badge either.
    draw_inert_badge(ctx, n, view, theme);
    body
}
