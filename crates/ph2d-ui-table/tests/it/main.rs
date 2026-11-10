//! A geometria da tabela da casa (plano 14 Q1): a MESMA caixa pinta e recebe o clique.

use ph2d_editor_core::zones::Rect;
use ph2d_ui_table::{
    Cell, Column, Hit, Ink, Layout, Metrics, Row, TableState, layout, natural_height, spark_points,
};

mod reserva_e_fios;

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
    assert_eq!(hit, Hit::Resize("value".into()));
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
        vec![("type".into(), "Type", true)],
        "só Type se esconde"
    );
    st.set_hidden("type", true);
    let (l, _) = lay(&mut st, area(&m));
    assert!(l.col("type").is_none());
    assert_eq!(st.column_menu(&c), vec![("type".into(), "Type", false)]);
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
            col: "value".into()
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

/// Plano 14 Q3: o anel de cada linha guarda as últimas `cap` amostras, sabe o texto da anterior
/// (o «mudou»), larga o traço de quem deixa de ser número e esquece quem não veio.
#[test]
fn o_anel_guarda_as_ultimas_amostras_e_o_que_mudou() {
    use ph2d_ui_table::{Span, Trails};
    let mut t = Trails::new(3);
    let um = |k: &str, v: f32| (k.to_owned(), v.to_string(), Some(v));
    for i in 1..=5 {
        let i = i as f32;
        t.sample([um("a", i), um("b", 7.0)]);
    }
    assert_eq!(t.samples("a"), Some(vec![3.0, 4.0, 5.0]), "só as 3 últimas");
    assert_eq!(t.old("a"), Some("4"), "mudou desde a amostra anterior");
    assert_eq!(t.old("b"), None, "igual: não mudou");
    let s = Span::of(&t.samples("a").unwrap()).unwrap();
    assert_eq!((s.min, s.max, s.now), (3.0, 5.0, 5.0));
    t.sample([("a".to_owned(), "\"x\"".to_owned(), None)]);
    assert_eq!(t.samples("a"), None, "um texto não tem traço");
    assert_eq!(t.old("a"), Some("5"));
    assert_eq!(t.samples("b"), None, "quem não veio sai");
    t.sample([um("b", 1.0)]);
    assert_eq!(t.old("b"), None, "quem volta recomeça");
    assert_eq!(Span::of(&[f32::NAN]), None);
}

// ── Plano 14 Q4: as marcas que a tabela pinta ─────────────────────────────────────────────────

use ph2d_editor_core::widget::{RowHighlight, paint_row_highlight};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_ui_table::{StableWidths, UiText, paint, paint_cell_frames};
use ph2d_vector::VectorScene;

const TEMA: Theme = Theme::Dark;

/// O `draw_data` de um preenchimento na cor `c`: a «tinta» que se procura numa cena.
fn tinta(c: ph2d_vector::Color) -> u32 {
    let mut s = VectorScene::new();
    ph2d_editor_core::paint::fill_rounded_rect(&mut s, Rect::new(0.0, 0.0, 1.0, 1.0), 0.0, c);
    s.inner().encoding().draw_data[0]
}

fn tom(t: ColorToken) -> u32 {
    tinta(ph2d_editor_core::paint::resolve(t, TEMA))
}

fn realce(h: RowHighlight) -> u32 {
    let mut s = VectorScene::new();
    paint_row_highlight(&mut s, Rect::new(0.0, 0.0, 1.0, 1.0), TEMA, h, 0.0);
    s.inner().encoding().draw_data[0]
}

/// Pinta `rows` e devolve `(draw_data, path_data, segmentos)` e a geometria.
fn pinta(st: &mut TableState, rows: &[Row], m: &Metrics) -> (Vec<u32>, Vec<u32>, u32, Layout) {
    let (c, a) = (cols(), area(m));
    let l = layout(&c, rows, &natural(m), st, a, m);
    let (mut scene, mut ts) = (VectorScene::new(), TextSystem::without_system_fonts());
    let tones = (ColorToken::PanelBg, ColorToken::Bg3);
    let mut text = UiText { theme: TEMA };
    paint(
        &l, &c, rows, st, tones, &mut scene, &mut ts, &mut text, TEMA,
    );
    let e = scene.inner().encoding();
    (
        e.draw_data.clone(),
        e.path_data.clone(),
        e.n_path_segments,
        l,
    )
}

#[test]
fn a_coluna_acesa_pinta_a_banda_e_acende_o_titulo_e_a_leve_o_tom_do_hover() {
    let m = Metrics::standard();
    let r = rows();
    let (antes, ..) = pinta(&mut TableState::default(), &r, &m);
    let (soft, title) = (realce(RowHighlight::Hovered), tom(ColorToken::Text1));
    assert!(!antes.contains(&tom(ColorToken::AccentSoft)) && !antes.contains(&soft));
    assert!(
        !antes.contains(&title),
        "controlo: nada em Text1 sem coluna acesa"
    );
    let mut st = TableState::default();
    st.lit_col = Some("value".into());
    let (acesa, ..) = pinta(&mut st, &r, &m);
    assert!(
        acesa.contains(&tom(ColorToken::AccentSoft)),
        "a banda da coluna acesa"
    );
    assert!(acesa.contains(&title), "o título da coluna acesa em Text1");
    let mut st = TableState::default();
    st.soft_col = Some("type".into());
    let (leve, ..) = pinta(&mut st, &r, &m);
    assert!(
        leve.contains(&soft),
        "a coluna do cursor no tom do hover das listas"
    );
    assert!(!leve.contains(&tom(ColorToken::AccentSoft)) && !leve.contains(&title));
}

#[test]
fn a_celula_mudada_ganha_a_banda_ambar_recuada_e_o_texto_ambar() {
    let m = Metrics::standard();
    let mut r = rows();
    let band = tinta(ph2d_ui_table::changed_tint(TEMA));
    let (antes, ..) = pinta(&mut TableState::default(), &r, &m);
    assert!(!antes.contains(&band));
    let i = r.iter().position(|x| x.key == "dt").unwrap();
    r[i].cells[1].changed = true;
    let (depois, path, _, l) = pinta(&mut TableState::default(), &r, &m);
    assert!(depois.contains(&band), "a banda âmbar da célula");
    let ambar = tinta(ph2d_ui_table::ink_color(Ink::Changed, TEMA));
    assert!(
        !antes.contains(&ambar) && depois.contains(&ambar),
        "o texto da célula em âmbar"
    );
    let cell = l.cell_rect(i, "value").unwrap();
    let frame = l.cell_frame(&r, "dt", "value").unwrap();
    let d = ph2d_tokens::Spacing::Xxs.px();
    assert!((frame.x - (cell.x + d)).abs() < 1e-3 && (frame.w - (cell.w - d * 2.0)).abs() < 1e-3);
    assert!(
        path.contains(&frame.x.to_bits()),
        "a banda assenta na célula recuada"
    );
    assert!(!r[i].changed, "a linha não mudou: só a célula");
}

#[test]
fn a_linha_escolhida_vence_o_hover() {
    let m = Metrics::standard();
    let r = rows();
    let hover = tom(ColorToken::AccentSoft);
    let mut st = TableState::default();
    st.hover = Some("dt".into());
    let (so_hover, ..) = pinta(&mut st, &r, &m);
    assert!(so_hover.contains(&hover), "controlo: o hover pinta-se");
    st.selected = Some("dt".into());
    let (escolhida, ..) = pinta(&mut st, &r, &m);
    assert!(escolhida.contains(&realce(RowHighlight::Selected)));
    assert!(!escolhida.contains(&hover), "a selecção tapa o hover");
}

#[test]
fn o_quadrado_de_cor_toma_o_lugar_do_texto_e_a_largura_e_a_dele() {
    let m = Metrics::standard();
    let mut r = rows();
    let cor = [0.9, 0.2, 0.1, 1.0];
    for row in r.iter_mut().filter(|x| x.group.is_none()) {
        row.cells[1] = Cell {
            text: "a very long value that would be wide".into(),
            swatch: Some(cor),
            ..Cell::default()
        };
    }
    let (data, ..) = pinta(&mut TableState::default(), &r, &m);
    assert!(data.contains(&tinta(ph2d_vector::Color::new(cor))));
    let mut ts = TextSystem::without_system_fonts();
    let mut text = UiText { theme: TEMA };
    let n = ph2d_ui_table::natural_widths(&cols(), &r, &m, &mut ts, &mut text);
    let want = (ph2d_ui_table::swatch_side() + m.pad * 2.0).max(m.min_col);
    assert!((n[1] - want).abs() < 1e-3, "{} vs {want}", n[1]);
}

#[test]
fn sem_cabecalho_o_corpo_comeca_em_cima_e_nada_do_cabecalho_se_pinta() {
    let m = Metrics {
        header_h: 0.0,
        ..Metrics::standard()
    };
    let r = rows();
    let (data, .., l) = pinta(&mut TableState::default(), &r, &m);
    assert!((l.body.y - l.area.y).abs() < 1e-3 && l.header.h == 0.0);
    let top = l.rows.first().unwrap();
    assert_eq!(
        l.hit(&r, l.cols[1].x + 1.0, l.area.y),
        Some(Hit::Toggle(top.index))
    );
    let vx = l.cols[1].x + 1.0;
    let first_item = l.rows.iter().find(|s| r[s.index].group.is_none()).unwrap();
    assert!(matches!(
        l.hit(&r, vx, first_item.rect.y + 1.0),
        Some(Hit::Cell { .. })
    ));
    // O `base` (PanelBg) só se pinta no fundo opaco do cabeçalho (a listra é um tom derivado).
    let fundo = tom(ColorToken::PanelBg);
    assert!(!data.contains(&fundo), "sem cabeçalho, nada do fundo dele");
    let (com, ..) = pinta(&mut TableState::default(), &r, &Metrics::standard());
    assert!(
        com.contains(&fundo),
        "controlo: com cabeçalho o fundo dele pinta-se"
    );
}

#[test]
fn a_moldura_da_celula_premida_vence_a_do_rato() {
    let m = Metrics::standard();
    let r = rows();
    let mut st = TableState::default();
    let l = layout(&cols(), &r, &natural(&m), &mut st, area(&m), &m);
    let frames = |hover, pressed| {
        let mut s = VectorScene::new();
        paint_cell_frames(&l, &r, hover, pressed, &mut s, TEMA);
        s.inner().encoding().draw_data.clone()
    };
    assert!(frames(None, None).is_empty());
    let rato = frames(Some(("dt", "value")), None);
    assert!(!rato.is_empty() && !rato.contains(&tom(ColorToken::Accent)));
    let premida = frames(Some(("dt", "value")), Some(("dt", "value")));
    assert_ne!(premida, rato, "a premida é outra moldura");
    assert!(
        frames(Some(("nobody", "value")), None).is_empty(),
        "linha que não existe"
    );
}

#[test]
fn as_larguras_estaveis_so_crescem() {
    let c = cols();
    let mut s = StableWidths::default();
    assert_eq!(s.widths(&c, &[50.0, 30.0, 20.0]), [50.0, 30.0, 20.0]);
    assert_eq!(
        s.widths(&c, &[40.0, 35.0, 20.0]),
        [50.0, 35.0, 20.0],
        "só cresce"
    );
    let menos = &c[..2];
    assert_eq!(
        s.widths(menos, &[10.0, 10.0]),
        [10.0, 10.0],
        "outras colunas: recomeça"
    );
    let mut trocada = cols();
    trocada[0] = Column::new(String::from("speed"), "speed");
    s.widths(&c, &[50.0, 30.0, 20.0]);
    assert_eq!(
        s.widths(&trocada, &[5.0, 30.0, 20.0])[0],
        5.0,
        "outra chave: recomeça"
    );
    s.reset();
    assert_eq!(s.widths(&trocada, &[1.0, 2.0, 3.0]), [1.0, 2.0, 3.0]);
}
