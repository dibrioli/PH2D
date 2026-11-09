//! A geometria da tabela da casa (plano 14 Q1): a MESMA caixa pinta e recebe o clique.

use ph2d_editor_core::zones::Rect;
use ph2d_ui_table::{
    Cell, Column, Hit, Ink, Layout, Metrics, Row, TableState, layout, natural_height, spark_points,
};

fn cols() -> Vec<Column> {
    vec![
        Column::new("name", "Name").flex(),
        Column::new("value", "Value").end().mono(),
        Column::new("type", "Type").hideable(),
    ]
}

fn item(key: &str, depth: u8, expandable: bool) -> Row {
    Row {
        key: key.into(),
        depth,
        cells: vec![
            Cell::new(key, Ink::Name),
            Cell::new("1", Ink::Number),
            Cell::new("number", Ink::Muted),
        ],
        expandable,
        ..Row::default()
    }
}

fn group(key: &str) -> Row {
    Row {
        key: key.into(),
        group: Some("2".into()),
        cells: vec![Cell::new(key, Ink::Plain)],
        expandable: true,
        default_open: true,
        ..Row::default()
    }
}

/// Locals (dt, pos ▸ (x, y)), self (id).
fn rows() -> Vec<Row> {
    vec![
        group("Locals"),
        item("dt", 1, false),
        item("pos", 1, true),
        item("pos.x", 2, false),
        item("pos.y", 2, false),
        group("self"),
        item("id", 1, false),
    ]
}

fn natural(m: &Metrics) -> Vec<f32> {
    vec![m.min_col * 2.0, m.min_col * 1.5, m.min_col]
}

fn lay(st: &mut TableState, area: Rect) -> (Layout, Metrics) {
    let m = Metrics::standard();
    let l = layout(&cols(), &rows(), &natural(&m), st, area, &m);
    (l, m)
}

fn area(m: &Metrics) -> Rect {
    Rect::new(10.0, 20.0, m.min_col * 8.0, m.header_h + m.row_h * 20.0)
}

#[test]
fn a_tabela_aninhada_comeca_dobrada_e_os_grupos_abertos() {
    let st = TableState::default();
    let r = rows();
    let shown: Vec<&str> = st
        .visible_rows(&r)
        .into_iter()
        .map(|i| r[i].key.as_str())
        .collect();
    assert_eq!(shown, ["Locals", "dt", "pos", "self", "id"]);
}

#[test]
fn abrir_a_tabela_mostra_os_campos_e_dobrar_o_grupo_esconde_tudo_abaixo() {
    let mut st = TableState::default();
    let r = rows();
    st.toggle_row("pos");
    assert_eq!(st.visible_rows(&r).len(), 7);
    st.toggle_row("Locals");
    let shown: Vec<&str> = st
        .visible_rows(&r)
        .into_iter()
        .map(|i| r[i].key.as_str())
        .collect();
    assert_eq!(
        shown,
        ["Locals", "self", "id"],
        "o grupo dobrado leva as netas"
    );
}

#[test]
fn a_coluna_flex_toma_o_que_sobra_e_as_outras_ficam_na_natural() {
    let m = Metrics::standard();
    let mut st = TableState::default();
    let a = area(&m);
    let (l, _) = lay(&mut st, a);
    let n = natural(&m);
    assert_eq!(l.cols.len(), 3);
    assert!((l.cols[1].w - n[1]).abs() < 1e-3);
    assert!((l.cols[2].w - n[2]).abs() < 1e-3);
    let total: f32 = l.cols.iter().map(|c| c.w).sum();
    assert!((total - a.w).abs() < 1e-3, "as colunas enchem a largura");
    assert!((l.cols[1].x - (l.cols[0].x + l.cols[0].w)).abs() < 1e-3);
}

#[test]
fn arrastar_o_puxador_muda_a_largura_e_nunca_desce_do_minimo() {
    let m = Metrics::standard();
    let mut st = TableState::default();
    let r = rows();
    let (l, _) = lay(&mut st, area(&m));
    let edge = l.cols[1].x + l.cols[1].w;
    let y = l.header.y + l.header.h * 0.5;
    let hit = l.hit(&r, edge, y).expect("puxador");
    assert_eq!(hit, Hit::Resize("value"));
    assert!(st.press(&hit, (edge, y), &l, &r));
    assert!(st.dragging());
    assert!(st.drag_to((edge + m.min_col, y), &m));
    let (l2, _) = lay(&mut st, area(&m));
    assert!((l2.col("value").unwrap().w - (l.cols[1].w + m.min_col)).abs() < 1e-3);
    st.drag_to((edge - m.min_col * 100.0, y), &m);
    let (l3, _) = lay(&mut st, area(&m));
    assert!((l3.col("value").unwrap().w - m.min_col).abs() < 1e-3);
    assert!(st.release());
    assert!(!st.dragging());
}

#[test]
fn esconder_a_coluna_type_tira_a_do_cabecalho_e_do_corpo() {
    let m = Metrics::standard();
    let mut st = TableState::default();
    let c = cols();
    assert_eq!(
        st.column_menu(&c),
        vec![("type", "Type", true)],
        "só Type se esconde"
    );
    st.set_hidden("type", true);
    let (l, _) = lay(&mut st, area(&m));
    assert!(l.col("type").is_none());
    assert_eq!(st.column_menu(&c), vec![("type", "Type", false)]);
    let a = area(&m);
    let total: f32 = l.cols.iter().map(|c| c.w).sum();
    assert!(
        (total - a.w).abs() < 1e-3,
        "a flex cresce para o lugar dela"
    );
}

#[test]
fn o_clique_no_chevron_vira_a_dobra_e_no_valor_e_uma_celula() {
    let m = Metrics::standard();
    let mut st = TableState::default();
    let r = rows();
    let (l, _) = lay(&mut st, area(&m));
    let pos = l.rows.iter().find(|s| r[s.index].key == "pos").unwrap();
    let cy = pos.rect.y + pos.rect.h * 0.5;
    let cx = l.cols[0].x + m.pad + m.indent + m.chevron() * 0.5;
    let hit = l.hit(&r, cx, cy).expect("chevron");
    assert_eq!(hit, Hit::Toggle(pos.index));
    assert!(st.press(&hit, (cx, cy), &l, &r));
    assert!(st.is_open(&r[pos.index]));
    let vx = l.cols[1].x + l.cols[1].w * 0.5;
    assert_eq!(
        l.hit(&r, vx, cy),
        Some(Hit::Cell {
            row: pos.index,
            col: "value"
        })
    );
    // O cabeçalho de grupo dobra em qualquer ponto da linha.
    let g = &l.rows[0];
    assert_eq!(l.hit(&r, vx, g.rect.y + 1.0), Some(Hit::Toggle(g.index)));
}

#[test]
fn com_pouca_altura_o_corpo_rola_e_so_pinta_as_linhas_que_toca() {
    let m = Metrics::standard();
    let mut st = TableState::default();
    let r = rows();
    let small = Rect::new(0.0, 0.0, m.min_col * 8.0, m.header_h + m.row_h * 2.0);
    let (l, _) = lay(&mut st, small);
    assert!((l.content_h - m.row_h * 5.0).abs() < 1e-3);
    assert!((l.max_scroll() - m.row_h * 3.0).abs() < 1e-3);
    assert_eq!(l.rows.len(), 2);
    let inside = (l.body.x + 1.0, l.body.y + 1.0);
    assert!(st.wheel(&l, inside.0, inside.1, -m.row_h * 100.0));
    assert!((st.scroll - l.max_scroll()).abs() < 1e-3, "clampado ao fim");
    let (l2, _) = lay(&mut st, small);
    assert_eq!(r[l2.rows.last().unwrap().index].key, "id");
    assert!(
        !st.wheel(&l2, inside.0, l2.area.y - 1.0, m.row_h),
        "fora do corpo não rola"
    );
    assert!((natural_height(&r, &st, &m) - (m.header_h + m.row_h * 5.0)).abs() < 1e-3);
}

#[test]
fn o_traco_poe_o_minimo_em_baixo_o_maximo_em_cima_e_a_serie_plana_a_meio() {
    let m = Metrics::standard();
    let r = Rect::new(0.0, 0.0, 100.0, 20.0);
    let p = spark_points(&[1.0, 3.0, 2.0], r, &m);
    assert_eq!(p.len(), 3);
    assert!((p[0].0 - 0.0).abs() < 1e-3 && (p[2].0 - 100.0).abs() < 1e-3);
    assert!(p[1].1 < p[2].1 && p[2].1 < p[0].1, "o máximo é o mais alto");
    let flat = spark_points(&[5.0, 5.0], r, &m);
    assert!((flat[0].1 - 10.0).abs() < 1e-3);
    assert!(spark_points(&[f32::NAN], r, &m).is_empty());
}

/// ⭐ A barra do corpo arrasta-se (smoke do dono 09/11: «não consigo arrastar o novo scroll»): no
/// polegar o conteúdo segue o dedo na proporção da trilha, preso aos dois fins; na trilha o polegar
/// salta para debaixo do dedo; durante o arrasto nenhuma coluna muda de largura.
#[test]
fn a_barra_do_corpo_arrasta_pelo_polegar_e_salta_pela_trilha() {
    use ph2d_editor_core::widget::{scrollbar_thumb_rect, scrollbar_track_rect};
    let m = Metrics::standard();
    let mut st = TableState::default();
    st.toggle_row("pos");
    let r = rows();
    let small = Rect::new(0.0, 0.0, m.min_col * 8.0, m.header_h + m.row_h * 3.0);
    let (l, _) = lay(&mut st, small);
    let max = l.max_scroll();
    assert!(max > 0.0);
    let track = scrollbar_track_rect(l.body);
    let thumb = scrollbar_thumb_rect(track, 0.0, l.content_h, l.body.h);
    let range = track.h - thumb.h;
    assert!(range > 1.0, "o polegar tem por onde andar");
    let bx = l.body.x + l.body.w - 1.0;
    let y0 = thumb.y + thumb.h * 0.5;
    assert_eq!(l.hit(&r, bx, y0), Some(Hit::Bar));
    assert_ne!(l.hit(&r, l.cols[1].x + 1.0, y0), Some(Hit::Bar));
    // O polegar: carregar não salta; meio caminho = meia rolagem; preso aos dois fins.
    assert!(st.press(&Hit::Bar, (bx, y0), &l, &r));
    assert!(st.bar_dragging() && st.dragging());
    assert!(st.scroll.abs() < 1e-3, "carregar no polegar não rola");
    assert!(st.drag_to((bx, y0 + range * 0.5), &m));
    assert!(
        (st.scroll - max * 0.5).abs() < 1e-2,
        "{} vs {}",
        st.scroll,
        max * 0.5
    );
    st.drag_to((bx, y0 + range * 10.0), &m);
    assert!((st.scroll - max).abs() < 1e-3, "preso ao fim");
    st.drag_to((bx - m.min_col * 3.0, y0 - range * 10.0), &m);
    assert!(st.scroll.abs() < 1e-3, "preso ao início");
    assert!(
        st.widths.is_empty(),
        "o arrasto da barra não mexe em colunas"
    );
    assert!(st.release());
    assert!(!st.dragging());
    assert!(
        !st.drag_to((bx, y0 + range), &m),
        "largado, o rato não rola"
    );
    // A trilha: o polegar salta para debaixo do dedo (o fundo da trilha = o fim).
    let (l, _) = lay(&mut st, small);
    assert!(st.press(&Hit::Bar, (bx, track.y + track.h - 1.0), &l, &r));
    assert!((st.scroll - max).abs() < 1e-3, "a trilha salta");
    assert!(st.release());
}

/// A rolagem anda em linhas inteiras (roda e um valor solto, como o revelar), e nenhuma linha
/// pintada começa acima do corpo (nada a meio por baixo do cabeçalho).
#[test]
fn a_rolagem_anda_em_linhas_inteiras_e_nada_fica_por_cima_do_cabecalho() {
    let m = Metrics::standard();
    let area = Rect::new(0.0, 0.0, m.min_col * 6.0, m.header_h + m.row_h * 3.5);
    let mut st = TableState::default();
    let (l, _) = lay(&mut st, area);
    let whole = |s: f32| (s / m.row_h - (s / m.row_h).round()).abs() < 1e-3;
    for dy in [-1.0, -m.row_h * 0.4, -m.row_h * 1.3, m.row_h * 0.2] {
        st.wheel(&l, l.body.x + 1.0, l.body.y + 1.0, dy);
        let (l2, _) = lay(&mut st, area);
        assert!(whole(st.scroll), "roda {dy}: {}", st.scroll);
        assert!(l2.rows.iter().all(|r| r.rect.y >= l2.body.y - 1e-3));
    }
    st.scroll = m.row_h * 1.6;
    let (l3, _) = lay(&mut st, area);
    assert!(whole(st.scroll), "revelar: {}", st.scroll);
    assert!(l3.rows.iter().all(|r| r.rect.y >= l3.body.y - 1e-3));
    st.scroll = f32::MAX;
    let (l4, _) = lay(&mut st, area);
    assert!(whole(st.scroll) && st.scroll > 0.0);
    let last = l4.rows.last().map(|r| r.rect.y + r.rect.h).unwrap_or(0.0);
    assert!(
        last <= l4.body.y + l4.body.h + 1e-3,
        "a última fica inteira"
    );
}

/// Corpo que transborda: as colunas + a calha da barra enchem a área, a barra não pisa coluna
/// nenhuma, e a Type fica com a largura natural (é o Name que encolhe).
#[test]
fn o_corpo_que_transborda_da_a_barra_uma_calha_so_dela() {
    let m = Metrics::standard();
    let area = Rect::new(0.0, 0.0, m.min_col * 6.0, m.header_h + m.row_h * 3.0);
    let mut st = TableState::default();
    let (l, _) = lay(&mut st, area);
    assert!(l.max_scroll() > 0.0);
    let sum: f32 = l.cols.iter().map(|c| c.w).sum();
    let gutter = ph2d_ui_table::bar_gutter();
    assert!(
        (sum + gutter - area.w).abs() < 1e-3,
        "{sum} + {gutter} != {}",
        area.w
    );
    let track = ph2d_editor_core::widget::scrollbar_track_rect(l.body);
    assert!(l.cols.iter().all(|c| c.x + c.w <= track.x + 1e-3));
    let ty = l.col("type").expect("Type");
    assert!(ty.w >= natural(&m)[2] - 1e-3);
    let mut big = TableState::default();
    let tall = Rect::new(0.0, 0.0, area.w, m.header_h + m.row_h * 30.0);
    let (l2, _) = lay(&mut big, tall);
    let sum2: f32 = l2.cols.iter().map(|c| c.w).sum();
    assert!((sum2 - area.w).abs() < 1e-3, "sem transbordo, sem calha");
}

/// Smoke do dono 09/11 (cena 14): no app cada movimento do rato vem com um quadro pintado no meio
/// (o `layout` arredonda a rolagem à linha inteira). O polegar agarrado segue o RATO, não a sua
/// própria rolagem arredondada: a cada passo a rolagem é a proporcional à distância desde o Down,
/// à linha inteira, e o arredondamento nunca se acumula.
#[test]
fn o_polegar_agarrado_segue_o_rato_atraves_dos_quadros() {
    use ph2d_editor_core::widget::{scrollbar_thumb_rect, scrollbar_track_rect};
    let mut st = TableState::default();
    st.toggle_row("pos");
    let r = rows();
    let m = Metrics::standard();
    let small = Rect::new(0.0, 0.0, m.min_col * 8.0, m.header_h + m.row_h * 3.0);
    let (l, _) = lay(&mut st, small);
    let (b, max) = (l.bar(), l.max_scroll());
    let thumb = scrollbar_thumb_rect(scrollbar_track_rect(b.body), 0.0, b.content_h, b.visible_h);
    let range = b.body.h - thumb.h;
    let (x, y0) = (l.body.x + l.body.w - 1.0, thumb.y + thumb.h * 0.5);
    assert!(st.press(&Hit::Bar, (x, y0), &l, &r));
    let mut last = 0.0;
    for k in 1..=8 {
        let f = k as f32 / 8.0;
        st.drag_to((x, y0 + range * f), &m);
        let (l, _) = lay(&mut st, small);
        // À linha mais próxima da proporcional (num empate, qualquer das duas).
        let want = max * f;
        assert!(
            (l.scroll - want).abs() <= m.row_h * 0.5 + 1e-3,
            "passo {k}: {} vs {want}",
            l.scroll
        );
        assert!(l.scroll >= last, "nunca volta para trás a descer");
        last = l.scroll;
    }
    assert!((last - max).abs() < 1e-3, "o fim do curso é o fim");
}
