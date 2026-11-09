//! Smoke do dono 09/11 (cena 14): o Stopped toma no máximo um terço da aba sozinho, o puxador de
//! cima muda-lhe a altura entre o mínimo e dois terços, e os Cards rolam por dentro.

use ph2d_a11y::NodeId;
use ph2d_code_debugcard::{CardState, CardStyle, DebugCard, StackEntry, VarRow};
use ph2d_editor_core::doc_surface::PointerLook;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::CodeView;
use ph2d_panel_code::debug_card::DebugCardView;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

const ABA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 900.0,
    h: 900.0,
};

fn parado(n: usize) -> DebugCard {
    let var = |i: usize| VarRow {
        key: format!("v{i}"),
        name: format!("v{i}"),
        depth: 0,
        value: i.to_string(),
        kind: Default::default(),
        field_count: 0,
        expandable: false,
        old: None,
    };
    DebugCard {
        state: CardState::Paused,
        line: 3,
        locals: (0..n).map(var).collect(),
        stack: vec![StackEntry {
            function: "update".into(),
            line: 3,
        }],
        can_step: true,
        ..DebugCard::default()
    }
}

/// Pinta `c` com a altura que o quadro pede na aba, encostado ao fundo dela.
fn pinta(v: &mut DebugCardView, c: &DebugCard) -> Rect {
    let h = v.height(c, ABA);
    let r = Rect::new(ABA.x, ABA.y + ABA.h - h, ABA.w, h);
    let (mut scene, mut ts) = (VectorScene::new(), TextSystem::without_system_fonts());
    let ids = [NodeId(1), NodeId(2)];
    v.paint(
        c,
        ids,
        r,
        &mut CodeView::new(),
        &mut scene,
        &mut ts,
        Theme::default(),
    );
    r
}

#[test]
fn o_stopped_tem_tecto_de_um_terco_e_o_puxador_vai_do_minimo_a_dois_tercos() {
    let (c, mut v) = (parado(40), DebugCardView::default());
    let r = pinta(&mut v, &c);
    assert!((r.h - ABA.h / 3.0).abs() < 1.0, "um terço sozinho: {}", r.h);
    let poucas = DebugCardView::default().height(&parado(1), ABA);
    assert!(poucas < r.h, "o natural abaixo do tecto: {poucas}");
    // Carregar no puxador, subir 200 px: cresce 200; o ponteiro é a mão.
    let g = v.painted.grip.expect("o puxador");
    let (gx, gy) = (g.x + g.w * 0.5, g.y + g.h * 0.5);
    assert_eq!(v.look(gx, gy), Some(PointerLook::Grab));
    assert!(v.press(gx, gy).is_some());
    assert!(v.drag_to(gx, gy - 200.0));
    assert_eq!(v.look(0.0, 0.0), Some(PointerLook::Grabbing), "a arrastar");
    assert!(v.release());
    assert!((pinta(&mut v, &c).h - (r.h + 200.0)).abs() < 1.0);
    // Muito para cima: dois terços; muito para baixo: o mínimo (a faixa + 4 linhas).
    for (dy, h) in [(-2000.0, ABA.h * 2.0 / 3.0), (2000.0, poucas.min(r.h))] {
        let g = v.painted.grip.expect("o puxador");
        let gy = g.y + g.h * 0.5;
        assert!(v.press(gx, gy).is_some() && v.drag_to(gx, gy + dy) && v.release());
        let now = pinta(&mut v, &c).h;
        assert!(now <= h + 1.0 && now > 0.0, "{dy}: {now} (até {h})");
    }
}

#[test]
fn os_cards_rolam_por_dentro_e_a_roda_fora_do_quadro_nao_e_dele() {
    let (c, mut v) = (parado(40), DebugCardView::default());
    v.style = CardStyle::Cards;
    let r = pinta(&mut v, &c);
    assert!(v.painted.cards_max > 0.0, "40 locais não cabem num terço");
    let shown = v.painted.values.len();
    assert!(shown > 0 && shown < 40, "só as que cabem: {shown}");
    assert!(
        !v.wheel(r.x + 10.0, r.y - 50.0, -100.0),
        "acima do quadro é do código"
    );
    assert!(v.wheel(r.x + r.w * 0.5, r.y + r.h * 0.5, -100.0));
    assert!((v.cards_scroll - 100.0).abs() < 1e-3);
    assert!(v.wheel(r.x + r.w * 0.5, r.y + r.h * 0.5, -1e6));
    assert!(
        (v.cards_scroll - v.painted.cards_max).abs() < 1e-3,
        "preso ao fim"
    );
    pinta(&mut v, &c);
    let first = v.painted.values.first().map(|(s, _)| s.clone());
    assert_ne!(first.as_deref(), Some("0"), "a primeira saiu de cima");
}
