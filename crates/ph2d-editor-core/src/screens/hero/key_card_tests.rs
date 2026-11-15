//! O cartão da chave: o que cada clique pede, e a geometria cabe e não se sobrepõe.

use super::*;
use crate::interaction::ContextMenuKind as K;
use crate::interaction::TimelineInterpScope;
use crate::interaction::key_card_alvo::e_alvo_do_cartao;

const KIND: K = K::TimelineSegment {
    scope: TimelineInterpScope::Key { target: 9, key: 2 },
};

/// Sem nada aceso, Curve pede Cubic In-Out; aceso Quad Out, Curve mantém Quad Out e o Ease in
/// troca só o modo; um botão de forma pede a forma no modo aceso; o fade NÃO é cartão.
#[test]
fn each_card_click_asks_the_same_pick_the_list_menu_would() {
    let mut store = WidgetStore::default();
    assert_eq!(
        acao(c::CTX_MENU_TL_CURVE, KIND, &store),
        Some(Acao::Escolhe {
            item: c::CTX_MENU_TL_FAM_CUBIC,
            mode: c::TL_EASE_MODE_INOUT
        })
    );
    store.set_menu_marks(
        KIND,
        &[c::CTX_MENU_TL_EASE_OUT, c::CTX_MENU_TL_FAM_QUAD],
        "",
    );
    assert_eq!(
        acao(c::CTX_MENU_TL_CURVE, KIND, &store),
        Some(Acao::Escolhe {
            item: c::CTX_MENU_TL_FAM_QUAD,
            mode: c::TL_EASE_MODE_OUT
        })
    );
    assert_eq!(
        acao(c::CTX_MENU_TL_CURVE_IN, KIND, &store),
        Some(Acao::Escolhe {
            item: c::CTX_MENU_TL_FAM_QUAD,
            mode: c::TL_EASE_MODE_IN
        })
    );
    // Um botão de forma pede ELA no modo aceso — sem lista no meio.
    assert_eq!(
        acao(c::CTX_MENU_TL_FAM_BOUNCE, KIND, &store),
        Some(Acao::Escolhe {
            item: c::CTX_MENU_TL_FAM_BOUNCE,
            mode: c::TL_EASE_MODE_OUT
        })
    );
    assert_eq!(
        acao(c::CTX_MENU_TL_KEY_CARD, KIND, &store),
        Some(Acao::Nada)
    );
    // Straight, Hold e as linhas de baixo seguem pelo menu em lista.
    assert_eq!(acao(c::CTX_MENU_TL_LINEAR, KIND, &store), None);
    let fade = K::TimelineSegment {
        scope: TimelineInterpScope::StripFade {
            lane: 0,
            strip: 0,
            edge: 0,
        },
    };
    assert!(!e_cartao(fade));
    assert_eq!(acao(c::CTX_MENU_TL_CURVE, fade, &store), None);
}

/// Todo alvo cabe no cartão, nenhum cobre outro, e o título (a pega do arrasto) é o topo.
#[test]
fn every_card_target_fits_and_none_overlaps() {
    let g = geo(10.0, 20.0);
    assert_eq!(g.interp.len(), INTERP.len());
    assert_eq!(g.ease.len(), EASES.len());
    assert_eq!(g.formas.len(), c::TIMELINE_EASE_MENU.len());
    let mut alvos = vec![g.cabeca, g.rove, g.reset, g.done];
    alvos.extend(g.interp.iter().chain(&g.ease).chain(&g.formas));
    let dentro = |r: &Rect| {
        r.x >= g.cartao.x
            && r.y >= g.cartao.y
            && r.x + r.w <= g.cartao.x + g.cartao.w + 1e-3
            && r.y + r.h <= g.cartao.y + g.cartao.h + 1e-3
    };
    for (i, a) in alvos.iter().enumerate() {
        assert!(dentro(a), "alvo {i} sai do cartao");
        for b in &alvos[i + 1..] {
            let cruza = a.x < b.x + b.w - 1e-3
                && b.x < a.x + a.w - 1e-3
                && a.y < b.y + b.h - 1e-3
                && b.y < a.y + a.h - 1e-3;
            assert!(!cruza, "dois alvos sobrepostos");
        }
    }
    assert_eq!((g.cabeca.x, g.cabeca.y), (g.cartao.x, g.cartao.y));
}

/// Done fecha; o título e o fundo não fazem nada (e por isso o cartão fica aberto).
#[test]
fn done_closes_and_the_title_does_nothing() {
    let store = WidgetStore::default();
    assert_eq!(
        acao(c::CTX_MENU_TL_KEY_DONE, KIND, &store),
        Some(Acao::Fecha)
    );
    assert_eq!(
        acao(c::CTX_MENU_TL_KEY_CARD_HEAD, KIND, &store),
        Some(Acao::Nada)
    );
    // Todo alvo do cartão é «dele»: apertá-lo não fecha o cartão (o aperto fecha o que não é).
    for id in [
        c::CTX_MENU_TL_CURVE_IN,
        c::CTX_MENU_TL_LINEAR,
        c::CTX_MENU_TL_ROVE,
        c::CTX_MENU_TL_FAM_ELASTIC,
    ] {
        assert!(e_alvo_do_cartao(id));
    }
    assert!(!e_alvo_do_cartao(c::CTX_MENU_TL_DELETE_TRACK));
}

/// Régua de teste: só Quad tem curva (`u²` no In), o resto não é forma.
fn quad(item: NodeId, mode: u8, u: f32) -> Option<f32> {
    (item == c::CTX_MENU_TL_FAM_QUAD && mode == c::TL_EASE_MODE_IN).then_some(u * u)
}

/// Cada botão desenha o que a escolha FAZ: Straight a diagonal, Hold o salto no fim, Nearest o
/// salto a meio, a forma a régua publicada (e nada sem régua — nunca uma curva inventada).
#[test]
fn each_button_draws_what_its_choice_does() {
    assert_eq!(pontos(Desenho::Reta, None), vec![(0.0, 0.0), (1.0, 1.0)]);
    assert_eq!(
        pontos(Desenho::Segura, None),
        vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0)]
    );
    let prox = pontos(Desenho::Proxima, None);
    assert!(prox.contains(&(0.5, 0.0)) && prox.contains(&(0.5, 1.0)));
    let bez = pontos(Desenho::Bezier, None);
    assert_eq!((bez[0], *bez.last().unwrap()), ((0.0, 0.0), (1.0, 1.0)));
    let d = Desenho::Ease {
        item: c::CTX_MENU_TL_FAM_QUAD,
        mode: c::TL_EASE_MODE_IN,
    };
    assert!(pontos(d, None).is_empty(), "sem régua não há curva");
    let q = pontos(d, Some(quad));
    assert_eq!(q.len(), SAMPLES + 1);
    assert!(q.iter().all(|&(u, y)| (y - u * u).abs() < 1e-6));
    let outra = Desenho::Ease {
        item: c::CTX_MENU_TL_FAM_BACK,
        mode: c::TL_EASE_MODE_IN,
    };
    assert!(pontos(outra, Some(quad)).is_empty());
}

/// O editor da Custom mora no lugar de Ease e Curve shape (cabe no cartão, não toca Interpolation
/// nem Rove/Done); as alças e a área são do cartão; o cursor e a curva convertem-se sem perda.
#[test]
fn the_custom_editor_takes_the_place_of_ease_and_shape() {
    let g = geo(10.0, 20.0);
    let e = g.editor;
    let interp_base = g.interp.iter().map(|r| r.y + r.h).fold(0.0, f32::max);
    assert!(e.y >= interp_base && e.y + e.h <= g.rove.y && e.y + e.h <= g.done.y);
    assert!(e.x >= g.cartao.x && e.x + e.w <= g.cartao.x + g.cartao.w);
    assert!(e_alvo_do_cartao(c::CTX_MENU_TL_KEY_BEZ_AREA));
    let area = crate::interaction::state::BezArea {
        rect: Rect::new(100.0, 50.0, 200.0, 160.0),
        y_min: -0.5,
        y_max: 1.5,
        desce: false,
    };
    let virada = crate::interaction::state::BezArea {
        desce: true,
        ..area
    };
    for a in [area, virada] {
        for (u, y) in [(0.0, 0.0), (0.3, 1.2), (1.0, -0.5)] {
            let (px, py) = a.tela(u, y);
            let (u2, y2) = a.unidade(px, py);
            assert!((u - u2).abs() < 1e-5 && (y - y2).abs() < 1e-5, "{a:?}");
        }
    }
    // Dono 15/11: com o valor a DESCER a curva desce no cartão como na timeline — o início (0,0)
    // fica ACIMA do fim (1,1). À mão: y = 0 é 1/4 da faixa −0,5..1,5 ⇒ 40 px de 160 a partir de cima.
    assert_eq!(virada.tela(0.0, 0.0), (100.0, 90.0));
    assert_eq!(virada.tela(1.0, 1.0), (300.0, 170.0));
    assert_eq!(
        area.tela(0.0, 0.0),
        (100.0, 170.0),
        "a subir fica como sempre"
    );
    // Fora da área prende-se à borda (uma alça nunca sai do gráfico).
    assert_eq!(area.unidade(0.0, 1000.0), (0.0, -0.5));
    // Um clique em cima da curva cria um ponto ali; perto de um ponto/alça agarra-o; longe, nada.
    let mut curva = crate::interaction::key_card_alvo::linear();
    let (x, y) = area.tela(0.5, 0.5);
    let u = crate::interaction::key_card_alvo::na_curva(&curva, &area, x, y + 3.0)
        .expect("em cima da curva");
    assert!((u - 0.5).abs() < 1e-3);
    assert_eq!(
        crate::interaction::key_card_alvo::na_curva(&curva, &area, x, y - 40.0),
        None
    );
    let i = curva.insert(u).unwrap();
    use crate::interaction::state::BezAlvo;
    assert_eq!(
        crate::interaction::key_card_alvo::alvo_em(&curva, &area, x + 2.0, y),
        Some(BezAlvo::Ponto(i))
    );
    let (hx, hy) = {
        let q = curva.node(0).h_out;
        area.tela(q[0], q[1])
    };
    assert_eq!(
        crate::interaction::key_card_alvo::alvo_em(&curva, &area, hx, hy),
        Some(BezAlvo::Alca(0, ph2d_anim::SplineSide::Out))
    );
    assert_eq!(
        crate::interaction::key_card_alvo::alvo_em(&curva, &area, 290.0, 60.0),
        None
    );
}

/// Pinta o cartão inteiro e devolve os alvos registados e os pontos que a cena emite.
fn pinta_cartao(store: &WidgetStore) -> (Vec<(NodeId, Rect)>, Vec<[f32; 2]>, Geo) {
    let mut scene = VectorScene::new();
    let mut ts = TextSystem::without_system_fonts();
    let mut hit = HitIndex::new();
    let req = crate::interaction::ContextMenuRequest {
        x: 40.0,
        y: 30.0,
        kind: KIND,
    };
    let viewport = Rect::new(0.0, 0.0, 1600.0, 900.0);
    paint(
        &mut scene,
        &mut ts,
        Theme::Dark,
        &mut hit,
        store,
        req,
        viewport,
    );
    let regs = hit.iter_registrations().collect();
    (regs, scene.probe_strokes().1, geo(40.0, 30.0))
}

/// ⭐ «Reset timing» (dono 15/11): só existe — pintado e clicável — quando a timeline publica que
/// o trecho tem ritmo; o clique pelo despacho do cartão pede o reset do alcance e o cartão FICA.
#[test]
fn reset_timing_so_existe_com_ritmo_e_o_clique_pede_o_reset() {
    crate::test_support::ensure_panel_registry();
    let mut store = WidgetStore::default();
    let (regs, _, g) = pinta_cartao(&store);
    let id = c::CTX_MENU_TL_KEY_RESET_TIMING;
    assert!(
        !regs.iter().any(|r| r.0 == id),
        "sem ritmo: nada de botão morto"
    );
    assert_eq!(acao(id, KIND, &store), Some(Acao::Nada));
    store.set_key_card_retimado(true);
    let (regs, _, _) = pinta_cartao(&store);
    assert_eq!(regs.iter().find(|r| r.0 == id).map(|r| r.1), Some(g.reset));
    assert!(e_alvo_do_cartao(id), "apertá-lo não fecha o cartão");
    let mut hero = crate::screens::hero::HeroScreen::new(NodeId(1));
    hero.store.set_key_card_retimado(true);
    hero.store
        .open_context_menu(crate::interaction::ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: KIND,
        });
    hero.store.close_context_menu();
    assert!(hero.apply_event(crate::interaction::WidgetEvent::Click(id)));
    let K::TimelineSegment { scope } = KIND else {
        unreachable!()
    };
    assert_eq!(hero.store.take_key_card_reset_timing(), Some(scope));
    assert!(hero.pending_timeline_interp.is_none(), "não troca a curva");
}

/// ⭐ As miniaturas viram quando o valor DESCE (dono 15/11), como o editor e a timeline: o botão de
/// Quad (`u²` no In) tem o meio `(½, ¼)` a ¼ da altura a partir de BAIXO a subir e a ¼ a partir
/// de CIMA a descer. Lido do que a cena emite, com o cartão inteiro pintado.
#[test]
fn as_miniaturas_viram_quando_o_valor_desce() {
    let i = c::TIMELINE_EASE_MENU
        .iter()
        .position(|r| r.0 == c::CTX_MENU_TL_FAM_QUAD)
        .expect("Quad é uma forma");
    for desce in [false, true] {
        let mut store = WidgetStore::default();
        store.set_ease_sampler(quad);
        store.set_menu_marks(KIND, &[c::CTX_MENU_TL_EASE_IN, c::CTX_MENU_TL_FAM_QUAD], "");
        store.set_key_card_desce(desce);
        let (_, pontos, g) = pinta_cartao(&store);
        let a = area_do_botao(g.formas[i]);
        let f = if desce { 0.75 } else { 0.25 };
        let alvo = [a.x + a.w * 0.5, a.y + a.h - f * a.h];
        let errado = [a.x + a.w * 0.5, a.y + a.h - (1.0 - f) * a.h];
        let perto = |p: [f32; 2]| {
            pontos
                .iter()
                .any(|q| (q[0] - p[0]).abs() < 0.05 && (q[1] - p[1]).abs() < 0.05)
        };
        assert!(
            perto(alvo),
            "desce = {desce}: o meio da curva não está em {alvo:?}"
        );
        assert!(
            !perto(errado),
            "desce = {desce}: a curva saiu do lado de antes"
        );
    }
}
