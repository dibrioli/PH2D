//! O cartão da chave: o que cada clique pede, e a geometria cabe e não se sobrepõe.

use super::*;
use crate::interaction::ContextMenuKind as K;

const KIND: K = K::TimelineSegment {
    scope: TimelineInterpScope::Key { target: 9, key: 2 },
};

/// Sem nada aceso, Curve pede Cubic In-Out; aceso Quad Out, Curve mantém Quad Out e o Ease in
/// troca só o modo; a forma abre a lista no modo aceso; o fade NÃO é cartão.
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
    assert_eq!(
        acao(c::CTX_MENU_TL_SHAPE, KIND, &store),
        Some(Acao::AbreFormas {
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

/// Todo alvo cabe no cartão, e nenhum alvo cobre outro.
#[test]
fn every_card_target_fits_and_none_overlaps() {
    let g = geo(10.0, 20.0);
    let mut alvos: Vec<Rect> = Vec::new();
    alvos.extend(g.blocos);
    alvos.extend(g.eases);
    alvos.push(g.forma);
    alvos.extend(g.linhas);
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
}
