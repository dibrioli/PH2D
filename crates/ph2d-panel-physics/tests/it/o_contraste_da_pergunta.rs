//! ⭐ Onda 9f: na caixa avermelhada do Remove os nomes dos destinos (e, fecho 07/11, o rótulo «Move them to») leem-se a ≥ 4,5:1 (WCAG 2.2 AA 1.4.3) sobre o
//! fundo dela, em todos os temas, com e sem cartões — a tinta e o fundo que a pintura usa.

use ph2d_panel_physics::{aba, cards};
use ph2d_tokens::Theme;

fn luminancia(c: ph2d_vector::Color) -> f64 {
    let [r, g, b, _] = c.components;
    let canal = |v: f32| {
        let v = f64::from(v);
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * canal(r) + 0.7152 * canal(g) + 0.0722 * canal(b)
}

fn razao(a: ph2d_vector::Color, b: ph2d_vector::Color) -> f64 {
    let (la, lb) = (luminancia(a), luminancia(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

#[test]
fn os_nomes_da_pergunta_do_remove_leem_se_a_4_5() {
    let mut menor = f64::MAX;
    for com_cartoes in [true, false] {
        cards::force_cards_on_this_thread(Some(com_cartoes));
        for t in Theme::ALL {
            let r = razao(aba::tinta_do_destino(t), aba::fundo_da_pergunta(t));
            eprintln!("contraste {t:?} cartões={com_cartoes}: {r:.2}");
            assert!(r >= 4.5, "{t:?} (cartões {com_cartoes}): {r:.2}:1 < 4,5:1");
            menor = menor.min(r);
            // Fecho 07/11: o rótulo «Move them to» também (era `Text3`, nunca medido).
            let rotulo = razao(aba::tinta_do_rotulo(t), aba::fundo_da_pergunta(t));
            eprintln!("rótulo {t:?} cartões={com_cartoes}: {rotulo:.2}");
            assert!(
                rotulo >= 4.5,
                "rótulo {t:?} (cartões {com_cartoes}): {rotulo:.2}:1 < 4,5:1"
            );
            menor = menor.min(rotulo);
        }
    }
    cards::force_cards_on_this_thread(None);
    eprintln!("contraste mínimo: {menor:.2}");
}
