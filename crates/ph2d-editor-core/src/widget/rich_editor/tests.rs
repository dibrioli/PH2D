//! A régua do documento dentro de um cartão (dono 03/11) e o controlo de fora dele.

use super::layout::{BlockBox, Marker, Regua};
use crate::widget::card_ink::{CardInk, com};
use crate::widget::showcase::card_back::demo_rich_editor;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_richdoc::BlockKind;
use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, Theme, TypeToken};

const W: f32 = 240.0; // LITERAL-PX-OK: teste, a coluna de um cartão

fn blocos(ink: Option<CardInk>) -> Vec<(BlockKind, f32, f32, Marker, Rect)> {
    let mut ed = demo_rich_editor(NodeId(1));
    ed.editing = false;
    let mut ts = TextSystem::new();
    let mut medir = || {
        ed.preferred_height(&mut ts, W);
        let l = ed.layout(&mut ts, Rect::new(0.0, 0.0, W, 0.0));
        let linha = |b: &BlockBox| (b.kind.clone(), b.text_x, b.font, b.marker, b.marker_rect);
        l.blocks.iter().map(linha).collect()
    };
    match ink {
        Some(ink) => com(ink, medir),
        None => medir(),
    }
}

fn texto_de(bs: &[(BlockKind, f32, f32, Marker, Rect)], k: &BlockKind) -> f32 {
    bs.iter()
        .find(|b| b.0 == *k)
        .map(|b| b.1)
        .expect("o documento de amostra tem este bloco")
}

/// ⭐⭐ A foto do dono (03/11): cada marcador com o seu recuo. No cartão, lista, numerada, checklist e
/// citação têm o texto na MESMA coluna, a seguir à `CHECK_BOX`. *Mutação: a citação sem o recuo.*
#[test]
fn in_a_card_every_marker_shares_one_text_column() {
    let bs = blocos(Some(CardInk::of_list(Theme::Dark)));
    let coluna = m::CHECK_BOX + m::CHECK_GAP;
    for k in [
        BlockKind::Numbered,
        BlockKind::Check(true),
        BlockKind::Check(false),
        BlockKind::Quote,
    ] {
        assert_eq!(texto_de(&bs, &k), coluna, "{k:?}");
    }
    assert_eq!(
        bs.iter().find(|b| b.0 == BlockKind::Bullet).map(|b| b.1),
        Some(coluna)
    );
}

/// ⭐ As duas caixas da checklist são a do cartão, e os títulos não gritam sobre a letra de valor:
/// degraus da escala, do corpo ao `Md`. *Mutação: voltar ao `Xl`.*
#[test]
fn in_a_card_the_headings_step_up_from_the_value_type() {
    let bs = blocos(Some(CardInk::of_list(Theme::Dark)));
    let corpo = bs.iter().find(|b| b.0 == BlockKind::Paragraph).map(|b| b.2);
    assert_eq!(corpo, Some(m::VALUE_PX));
    let h1 = bs
        .iter()
        .find(|b| b.0 == BlockKind::Heading(1))
        .map(|b| b.2);
    assert_eq!(h1, Some(TypeToken::Md.px()));
    let h2 = Regua::Cartao.font_px(&BlockKind::Heading(2));
    assert!(m::VALUE_PX < h2 && h2 < TypeToken::Md.px(), "{h2}");
    for b in bs.iter().filter(|b| matches!(b.3, Marker::Check(_))) {
        assert!(
            b.4.h >= m::CHECK_BOX,
            "a caixa cabe na primeira linha: {}",
            b.4.h
        );
    }
}

/// O CONTROLO: fora de um cartão a régua aprovada do Kanban não mudou.
#[test]
fn outside_a_card_the_rich_editor_keeps_the_kanban_ruler() {
    let bs = blocos(None);
    let h1 = bs
        .iter()
        .find(|b| b.0 == BlockKind::Heading(1))
        .map(|b| b.2);
    assert_eq!(h1, Some(TypeToken::Xl.px()));
    let pad = Spacing::Md.px();
    assert_eq!(
        texto_de(&bs, &BlockKind::Check(true)),
        pad + Spacing::Xl.px()
    );
    assert_eq!(texto_de(&bs, &BlockKind::Quote), pad + Spacing::Md.px());
}

/// ⭐ Numa coluna de cartão a barra mostra os doze botões (a do Kanban corta os que não cabem), e o
/// clique responde na mesma régua. *Mutação: o botão com a largura da régua do painel.*
#[test]
fn in_a_card_the_toolbar_fits_all_its_tools() {
    use super::toolbar::{ToolbarHit, button_de, hit};
    let bar = Rect::new(0.0, 0.0, W, m::HEAD_ICON_BOX);
    for i in 0..super::RICH_TOOLS.len() {
        let r = button_de(Regua::Cartao, bar, i);
        assert!(r.x + r.w <= bar.w + 0.01, "o botão {i} sai da barra: {r:?}");
        let centro = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert_eq!(
            hit(bar, centro.0, centro.1, Regua::Cartao),
            Some(ToolbarHit(i))
        );
    }
    let ultimo = button_de(Regua::Painel, bar, super::RICH_TOOLS.len() - 1);
    assert!(
        ultimo.x + ultimo.w > bar.w,
        "o controlo: o painel não cabe nesta coluna"
    );
}

/// ⭐ A foto g6: o fundo do «inline code» esticava até ao espaço que se pendura no fim da linha. Num
/// cartão cada linha visual tem o seu fundo, e ele acaba na última letra dela. *Mutação: voltar ao
/// `range_rects` no `hugging_rects`.*
#[test]
fn in_a_card_a_wrapped_inline_code_background_hugs_each_visual_line() {
    use super::layout::DocLayout;
    use ph2d_richdoc::{Block, Caret, Inline, Mark, RichDoc};
    let (antes, codigo) = ("Ship the text, ", "inline code that wraps at the column");
    let mut linha = Inline::default();
    linha.push(antes, &[]);
    linha.push(codigo, &[Mark::Code]);
    let doc = RichDoc::from_blocks(vec![Block::new(BlockKind::Paragraph, linha)]);
    let mut ts = TextSystem::new();
    let l = DocLayout::build_with(Regua::Cartao, &mut ts, &doc, (0.0, 0.0), W);
    let b = &l.blocks[0];
    let texto = doc.blocks()[0].inline.text();
    let (s, e) = (antes.len(), antes.len() + codigo.len());

    let justos = l.hugging_rects(b, texto, s, e);
    let crus = l.range_rects(b, s, e);
    assert!(justos.len() >= 2, "o troço quebra: {justos:?}");
    assert_eq!(justos.len(), crus.len(), "um fundo por linha visual");
    let mut pendurado = false;
    for (line, (j, c)) in b.layout.lines().zip(justos.iter().zip(&crus)) {
        let r = line.text_range();
        let fim = r.start + texto[r.start..r.end.min(e)].trim_end().len();
        let letra = l.caret_rect(Caret::new(0, fim)).x;
        assert!(
            (j.x + j.w - letra).abs() < 0.01,
            "acaba na última letra da linha: {} × {letra}",
            j.x + j.w
        );
        pendurado |= c.w > j.w;
    }
    assert!(pendurado, "o controlo: o rectângulo cru leva o espaço");
}
