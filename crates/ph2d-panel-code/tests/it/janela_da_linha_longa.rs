//! ⛔ **Uma linha maior que o tecto pinta a janela À VISTA, não os primeiros bytes** (o tecto
//! `LINE_RENDER_BYTES_MAX` é do quadro): com a vista no fim de uma linha colada de 64 KB, o
//! ecrã ficava em branco — o pedido de vista levava a rolagem ao troço que não se pintava.

use crate::{Deco, TAB, area_para, pinta, ts};
use ph2d_code_buffer::{Buffer, Range, Selection};
use ph2d_panel_code::{CodeView, Hit};

/// Duas linhas curtas e uma de ~64 KB com tabs, `é` e espaços no meio.
fn longa() -> (Buffer, usize) {
    let l = format!("local x = 1 --{}", "a\tbc é ".repeat(64 << 10 >> 3));
    let buf = Buffer::new(&format!("local a = 1\n{l}\nlocal b = 2"));
    (buf, 1)
}

/// A coluna de ecrã que a conta da mono põe no x `x` (a origem e a rolagem da vista).
fn coluna_em(v: &CodeView, x: f32) -> usize {
    let m = v.metrics();
    ((x - v.text_rect().x - m.text_pad + v.scroll().0) / m.advance).floor() as usize
}

#[test]
fn a_vista_no_fim_da_linha_longa_pinta_o_fim_dela() {
    let (mut buf, line) = longa();
    let (mut v, mut ts) = (CodeView::new(), ts());
    let area = area_para(&mut v, &buf, &mut ts, 10);
    let fim = buf.text().line_end(line);
    buf.set_selection(Selection::single(Range::cursor(fim)));
    v.ensure_visible(&buf, fim);
    pinta(&mut v, &buf, &mut ts, area, &Deco::default());
    let tr = v.text_rect();
    let cols: Vec<_> = v
        .probe_columns()
        .into_iter()
        .filter(|(c, _)| buf.text().line_of_byte(c.byte) == line)
        .collect();
    let ultimo = buf.text().prev_grapheme(fim);
    let c = cols
        .iter()
        .find(|(c, _)| c.byte == ultimo)
        .map(|p| p.0)
        .unwrap_or_else(|| {
            panic!(
                "o último grafema não se pintou: bytes {:?}",
                cols.first()
                    .zip(cols.last())
                    .map(|(a, b)| (a.0.byte, b.0.byte))
            )
        });
    assert!(
        c.x0 >= tr.x && c.x1 <= tr.x + tr.w,
        "o fim pinta-se fora da vista: {c:?} em {tr:?}"
    );
    assert_eq!(
        coluna_em(&v, (c.x0 + c.x1) * 0.5),
        buf.text().display_col(ultimo, TAB),
        "o x do fim é o da coluna dele na linha"
    );
    let caret = v.caret_rect(&buf, fim).expect("o cursor do fim à vista");
    assert!(caret.x >= tr.x && caret.x <= tr.x + tr.w, "{caret:?}");
    assert!(
        (caret.x - c.x1).abs() <= 1.0,
        "cursor {caret:?} × fim {c:?}"
    );
    // Controlo: a janela é a do tecto, não a linha inteira.
    assert!(cols.len() <= ph2d_panel_code::LINE_RENDER_BYTES_MAX);
}

#[test]
fn o_clique_no_meio_da_janela_deslocada_cai_no_byte_certo() {
    let (mut buf, line) = longa();
    let (mut v, mut ts) = (CodeView::new(), ts());
    let area = area_para(&mut v, &buf, &mut ts, 10);
    let t = buf.text();
    let meio = t.floor_char((t.byte_of_line(line) + t.line_end(line)) / 2);
    buf.set_selection(Selection::single(Range::cursor(meio)));
    v.ensure_visible(&buf, meio);
    pinta(&mut v, &buf, &mut ts, area, &Deco::default());
    let tr = v.text_rect();
    let t = buf.text();
    let vistas: Vec<_> = v
        .probe_columns()
        .into_iter()
        .filter(|(c, _)| t.line_of_byte(c.byte) == line && c.x0 >= tr.x && c.x1 <= tr.x + tr.w)
        .collect();
    assert!(
        vistas.len() > 20,
        "a vista mostra a linha: {}",
        vistas.len()
    );
    assert!(
        vistas.iter().any(|(c, _)| c.byte == meio),
        "o cursor do meio pinta-se"
    );
    for (c, mid) in vistas.iter().step_by(7) {
        let x = c.x0 + (c.x1 - c.x0) * 0.25;
        assert_eq!(v.hit(&buf, x, *mid), Hit::Text(c.byte), "{c:?}");
        assert_eq!(
            coluna_em(&v, x),
            t.display_col(c.byte, TAB),
            "o byte {} pinta-se na coluna dele",
            c.byte
        );
    }
}

#[test]
fn uma_linha_curta_pinta_se_inteira_a_partir_da_coluna_0() {
    let buf = Buffer::new("local a = 1\n\tlocal é = 2");
    let (mut v, mut ts) = (CodeView::new(), ts());
    let area = area_para(&mut v, &buf, &mut ts, 10);
    pinta(&mut v, &buf, &mut ts, area, &Deco::default());
    let t = buf.text();
    let cols = v.probe_columns();
    let x0 = v.text_rect().x + v.metrics().text_pad;
    for line in 0..2 {
        let (ls, le) = (t.byte_of_line(line), t.line_end(line));
        let mine: Vec<_> = cols
            .iter()
            .filter(|(c, _)| t.line_of_byte(c.byte) == line)
            .collect();
        assert_eq!(mine.first().map(|p| p.0.byte), Some(ls));
        assert!((mine[0].0.x0 - x0).abs() <= 1.0, "{:?}", mine[0]);
        assert_eq!(mine.last().map(|p| t.next_grapheme(p.0.byte)), Some(le));
        for (c, mid) in mine {
            assert_eq!(
                v.hit(&buf, c.x0 + (c.x1 - c.x0) * 0.25, *mid),
                Hit::Text(c.byte)
            );
        }
    }
}

/// As colunas pintadas da linha `line`, por ordem de byte.
fn colunas_da(v: &CodeView, buf: &Buffer, line: usize) -> Vec<(ph2d_panel_code::Column, f32)> {
    let mut c: Vec<_> = v
        .probe_columns()
        .into_iter()
        .filter(|(c, _)| buf.text().line_of_byte(c.byte) == line)
        .collect();
    c.sort_by_key(|p| p.0.byte);
    c
}

/// ⛔ Shift+End do meio de uma linha de 64 KB com a vista no início: a selecção pára na borda da
/// janela pintada (o «…»), sem o meio-bloco do fim de linha — o fim REAL está a 60 KB dali.
#[test]
fn a_selecao_ate_ao_fim_da_linha_cortada_para_no_corte_sem_meia_coluna() {
    let (mut buf, line) = longa();
    let (mut v, mut ts) = (CodeView::new(), ts());
    // Um tecto menor que a vista: a janela leva a vista e o degrau, e o corte cai logo depois dela.
    v.set_render_cap(60);
    let area = area_para(&mut v, &buf, &mut ts, 10);
    let t = buf.text();
    let (ls, fim) = (t.byte_of_line(line), t.line_end(line));
    let (fim0, b1) = (t.line_end(0), t.byte_of_line(1));
    // Do fim ao meio: a vista fica no início (o fim REAL fora dela).
    buf.set_selection(Selection::single(Range::new(fim, ls + 4)));
    pinta(&mut v, &buf, &mut ts, area, &Deco::default());
    let cols = colunas_da(&v, &buf, line);
    let ultima = cols.last().expect("a janela pinta-se").0;
    assert!(ultima.byte + 64 < fim, "a vista está no início: {ultima:?}");
    let segs = v.probe_selection_segments(&buf, ls + 4, fim);
    let seg = segs.first().expect("a selecção pinta-se na fila");
    assert!(
        (seg.x + seg.w - ultima.x1).abs() <= 0.5,
        "a selecção acaba no corte {} e não em {}",
        ultima.x1,
        seg.x + seg.w
    );
    // Controlo: numa linha curta a selecção que leva o fim de linha ganha a meia coluna.
    buf.set_selection(Selection::single(Range::new(0, b1)));
    pinta(&mut v, &buf, &mut ts, area, &Deco::default());
    let c0 = colunas_da(&v, &buf, 0).last().expect("linha 0").0;
    assert_eq!(buf.text().next_grapheme(c0.byte), fim0);
    let s0 = v.probe_selection_segments(&buf, 0, b1);
    let s0 = s0.first().expect("a selecção da linha 0");
    let meia = v.metrics().advance * 0.5;
    assert!(
        (s0.x + s0.w - (c0.x1 + meia)).abs() <= 0.5,
        "a linha curta estica meia coluna: {s0:?} × {c0:?}"
    );
}

/// ⛔ Uma linha dobrada E cortada pintava dois «…» no mesmo x (a dobra e o corte).
#[test]
fn a_linha_dobrada_e_cortada_mostra_um_so_fecho() {
    let (buf, line) = longa();
    let (mut v, mut ts) = (CodeView::new(), ts());
    // Um tecto menor que a vista: a janela leva a vista e o degrau, e o corte cai logo depois dela.
    v.set_render_cap(60);
    let area = area_para(&mut v, &buf, &mut ts, 10);
    let d = Deco {
        folds: vec![ph2d_panel_code::FoldMark {
            start: line,
            end: line + 1,
            folded: true,
        }],
        ..Deco::default()
    };
    let scene = pinta(&mut v, &buf, &mut ts, area, &d);
    let cols = colunas_da(&v, &buf, line);
    let (ultima, mid) = *cols.last().expect("a janela pinta-se");
    let lh = v.metrics().line_h;
    let depois: Vec<_> = crate::linhas_de_base(&scene)
        .into_iter()
        .filter(|&(_, x, y)| x >= ultima.x1 && (y - mid).abs() < lh * 0.5)
        .collect();
    assert_eq!(depois.len(), 1, "um só «…» depois do corte: {depois:?}");
}

/// ⛔ Com caracteres largos (CJK, emoji, cuja largura moldada não é a da conta da mono) e um tecto
/// menor que a vista, o corte nunca fica À VISTA (a janela cobre as colunas à vista): o clique na
/// borda direita cai num byte pintado, e logo depois do corte já é fora do texto. (O corte esquerdo
/// também não: a janela começa na coluna esquerda ou antes dela.)
#[test]
fn o_corte_com_caracteres_largos_fica_fora_da_vista_e_a_borda_cai_dentro() {
    let l = format!("local s = \"{}\"", "中文字😀".repeat(2048));
    let buf = Buffer::new(&format!("local a = 1\n{l}\nlocal b = 2"));
    let line = 1;
    let (mut v, mut ts) = (CodeView::new(), ts());
    v.set_render_cap(60);
    let area = area_para(&mut v, &buf, &mut ts, 10);
    pinta(&mut v, &buf, &mut ts, area, &Deco::default());
    let (adv, tr) = (v.metrics().advance, v.text_rect());
    let cols = colunas_da(&v, &buf, line);
    let (ultima, mid) = *cols.last().expect("a janela pinta-se");
    assert!(
        ultima.byte + 64 < buf.text().line_end(line),
        "cortada à direita"
    );
    assert!(
        ultima.x1 >= tr.x + tr.w,
        "o corte {ultima:?} fora da vista {tr:?}"
    );
    match v.hit(&buf, tr.x + tr.w - adv * 0.5, mid) {
        Hit::Text(b) => assert!(b <= ultima.byte, "{b} dentro da janela até {}", ultima.byte),
        h => panic!("{h:?}"),
    }
    assert!(matches!(
        v.hit(&buf, ultima.x1 + adv * 0.75, mid),
        Hit::Outside
    ));
}

/// ⛔ **Uma linha DENSA (emoji 4 B, CJK 3 B por coluna) numa vista LARGA**: o tecto de 2 000 B são
/// 500–666 colunas e o degrau da janela comia metade — no meio ficava um troço à vista por pintar e
/// no fim o fim não se pintava. A janela cobre as colunas à vista, no meio e no fim.
#[test]
fn a_linha_densa_numa_vista_larga_pinta_a_vista_inteira_no_meio_e_no_fim() {
    for unidade in ["😀", "中"] {
        let l = unidade.repeat((64 << 10) / unidade.len());
        let mut buf = Buffer::new(&format!("local a = 1\n{l}\nlocal b = 2"));
        let line = 1;
        let (mut v, mut ts) = (CodeView::new(), ts());
        let mut area = area_para(&mut v, &buf, &mut ts, 10);
        area.w = 8000.0;
        let t = buf.text();
        let (ls, le) = (t.byte_of_line(line), t.line_end(line));
        let meio = t.floor_char((ls + le) / 2);
        for alvo in [meio, le] {
            buf.set_selection(Selection::single(Range::cursor(alvo)));
            pinta(&mut v, &buf, &mut ts, area, &Deco::default());
            v.ensure_visible(&buf, alvo);
            pinta(&mut v, &buf, &mut ts, area, &Deco::default());
            let t = buf.text();
            let tr = v.text_rect();
            let m = v.metrics();
            let vistas = (tr.w - m.text_pad) / m.advance;
            assert!(vistas > 700.0, "a vista é larga: {vistas} colunas");
            let esq = coluna_em(&v, tr.x + m.text_pad);
            let ultima_col = t.display_col(t.prev_grapheme(le), TAB);
            let dir = coluna_em(&v, tr.x + tr.w - 1.0).min(ultima_col);
            let cols = colunas_da(&v, &buf, line);
            let (a, b) = (
                cols.first().expect("pinta-se").0,
                cols.last().expect("pinta-se").0,
            );
            let (ca, cb) = (t.display_col(a.byte, TAB), t.display_col(b.byte, TAB));
            assert!(
                ca <= esq && cb >= dir,
                "{unidade} em {alvo}: a vista {esq}..={dir} e a janela pinta {ca}..={cb}"
            );
            // A janela começa antes (ou na) coluna esquerda: o «…» do corte esquerdo fica fora.
            assert!(ca <= esq);
            if alvo == le {
                assert_eq!(t.next_grapheme(b.byte), le, "{unidade}: o fim pinta-se");
            }
        }
    }
}
