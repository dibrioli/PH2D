//! Testes de [`super`] — a coluna de nomes da faixa nos cartões.

use super::*;
use ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread;

/// O meio das maiúsculas que o pintor de texto desenha quando recebe `y` a `font` (a conta de
/// `ph2d_text`, a mesma de `linhas_tests::o_texto_da_coluna_fica_no_meio_da_linha`).
pub(crate) fn meio_das_maiusculas(y: f32, font: f32) -> f32 {
    let topo = y - (ph2d_text::displayed_font_px(font) - font) * 0.5;
    topo + ph2d_text::cap_centre_px(font)
}

fn centro(r: Rect) -> f32 {
    r.y + r.h * 0.5
}

/// ⭐ Doc 16 onda 3 (o centro vertical da onda 2h, agora na faixa de 30): o nome, o ícone de camadas,
/// o peso (alvo, pílula e número), o olho e o `+` ficam a ≤ 1 px do meio da faixa. *Mutação: o nome
/// pela conta `(linha − letra)/2` ⇒ ~2,5 px ⇒ vermelho.*
#[test]
fn o_texto_e_os_icones_da_faixa_ficam_no_meio_da_linha() {
    force_cards_on_this_thread(Some(true));
    let row = Rect::new(10.0, 300.0, 1200.0, geom::faixa_h());
    let meio = centro(row);
    let c = coluna_da_faixa(row, crate::tracks::LABEL_COL_W);
    let p = pilula(c.peso, TypeToken::Xs.px() * 2.0);
    for (y, o_que) in [
        (meio_das_maiusculas(c.texto_y, TypeToken::Sm.px()), "nome"),
        (centro(c.icone), "ícone de camadas"),
        (centro(c.peso), "alvo do peso"),
        (centro(p), "pílula do peso"),
        (
            meio_das_maiusculas(texto_y(p, TypeToken::Xs.px()), TypeToken::Xs.px()),
            "número do peso",
        ),
        (centro(glifo_em(c.mudo)), "olho"),
        (centro(glifo_em(c.mais)), "+"),
    ] {
        assert!(
            (y - meio).abs() <= 1.0,
            "{o_que} a {:.2} px do meio da faixa",
            y - meio
        );
    }
    force_cards_on_this_thread(None);
}

/// Os alvos ficam dentro da coluna, lado a lado sem se tocar, e a etiqueta acaba onde o peso começa
/// (o clique dela nunca rouba o de um controlo).
#[test]
fn os_controlos_da_faixa_cabem_na_coluna_e_nao_se_sobrepoem() {
    force_cards_on_this_thread(Some(true));
    let row = Rect::new(10.0, 300.0, 1200.0, geom::faixa_h());
    let w = crate::geom::MIN_LANE_LABEL_W;
    let c = coluna_da_faixa(row, w);
    assert!(c.mais.x + c.mais.w <= row.x + w, "o + sai da coluna");
    assert!(c.mudo.x + c.mudo.w < c.mais.x, "olho sobre o +");
    assert!(c.peso.x + c.peso.w < c.mudo.x, "peso sobre o olho");
    assert_eq!(
        c.etiqueta.x + c.etiqueta.w,
        c.peso.x,
        "a etiqueta acaba no peso"
    );
    assert!(
        c.nome.0 + c.nome.1 <= c.peso.x,
        "o nome passa por baixo do peso"
    );
    let p = pilula(c.peso, TypeToken::Xs.px() * 3.0);
    assert!(
        p.x >= c.peso.x && p.x + p.w <= c.peso.x + c.peso.w,
        "a pílula sai do alvo"
    );
    force_cards_on_this_thread(None);
}
