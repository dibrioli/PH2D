//! ⭐ O cursor tingido (dono 10/11: «o cursor da cor da alça que está sobre»): a alça quente ou
//! apertada publica a cor de REPOUSO dela pelo id do clique; o alvo neutro e o clássico, nenhuma.

use super::*;

fn rgba(c: Color) -> [u8; 4] {
    let u = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    let k = c.components;
    [u(k[0]), u(k[1]), u(k[2]), 255]
}

/// Na faixa REAL: o canto de esticar publica verde, o de cortar vermelho, o fade o azul dele — no
/// hover e no aperto; o corpo da strip e a faixa vazia não publicam; `PH2D_CARDS=0` também não.
#[test]
fn a_alca_sob_o_ponteiro_publica_a_cor_dela() {
    let theme = Theme::Forge;
    let snap = snap_duas_pecas();
    let tinta_de = |cartoes, id: NodeId, aperta: bool| {
        let prepara = |s: &mut WidgetStore| {
            if aperta {
                s.set_active(Some(id));
            } else {
                s.set_hot(Some(id));
            }
        };
        pinta_faixa_lendo(theme, cartoes, prepara, |s| s.timeline_cursor_tint(id)).3
    };
    let verde = rgba(resolve(ColorToken::Success, theme));
    let vermelho = rgba(resolve(ColorToken::Danger, theme));
    let fade = rgba(resolve(ColorToken::TimelinePlayhead, theme));
    assert_ne!(verde, vermelho);
    for (edge, cor) in [(5u8, verde), (1, vermelho), (EASE_IN_CODE, fade)] {
        let id = id_de(&snap, edge);
        assert_eq!(tinta_de(true, id, false), Some(cor), "edge {edge}: hover");
        assert_eq!(tinta_de(true, id, true), Some(cor), "edge {edge}: aperto");
        assert_eq!(
            tinta_de(false, id, false),
            None,
            "edge {edge}: PH2D_CARDS=0"
        );
    }
    assert_eq!(
        tinta_de(true, id_de(&snap, 2), false),
        None,
        "o corpo é neutro"
    );
    assert_eq!(tinta_de(true, NodeId(0xFA1A), false), None, "a faixa vazia");
}

/// A faixa do laço (teal, `LacoDaRegua`) e a chave da folha: a porta publica a cor de repouso.
#[test]
fn a_faixa_do_laco_e_a_chave_publicam_a_cor_de_repouso() {
    let theme = Theme::Forge;
    let laco = crate::ids::timeline_loop_brace_id(2);
    let chave = NodeId(0xC4A);
    let (_, teal) = crate::ruler::faixa_do_laco(theme);
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(Some(true));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::TimelinePanel>();
    let layout = HeroLayout::for_viewport(Rect::new(0.0, 0.0, 400.0, 200.0));
    let mut publicado = |id: NodeId, pinta_alca: &dyn Fn(&mut PaintCtx, EstadoAlca)| {
        host.store_mut().set_hot(Some(id));
        host.paint_fn(&layout, |ctx| {
            let e = EstadoAlca::nos_cartoes(ctx, id);
            pinta_alca(ctx, e);
        });
        host.store().timeline_cursor_tint(id)
    };
    let barra = [Rect::new(100.0, 0.0, 200.0, 5.0)];
    let l = publicado(laco, &|ctx, e| pinta(ctx, theme, e, teal, &barra, &[]));
    let k = publicado(chave, &|ctx, e| {
        pinta_chave(ctx, theme, 50.0, 50.0, false, e)
    });
    ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread(None);
    assert_eq!(l, Some(rgba(teal)), "o laço é teal");
    assert_eq!(k, Some(rgba(tinta(TimelineTinta::Chave, theme))), "a chave");
    assert_eq!(
        TimelineTinta::LacoDaRegua.resolve(theme).g,
        rgba(teal)[1],
        "a faixa é a tinta do laço"
    );
}
