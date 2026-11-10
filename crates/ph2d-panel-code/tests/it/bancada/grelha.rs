//! ⭐ A grelha «Who has what» sobre a tabela da casa (plano 14 Q4 B): os alvos saem da MESMA
//! geometria que pinta, a linha/coluna acesas e a do cursor, o âmbar da célula mudada, a cor e o
//! ponto, larguras que não pulam, o tecto em linhas inteiras, a mão, e os dois arrastos (a barra ↕
//! em linhas inteiras e o separador ↔ que dá a largura ao mapa).

use super::{GRADE, LIMITES, LIVES, MARCA, SPEED, TINT, alvo, celula, meio, pinta_grade, quem};
use crate::ts;
use ph2d_code_viewmodel::{Holder, Who};
use ph2d_editor_core::doc_surface::PointerLook;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::CodeView;
use ph2d_panel_code::bancada::{Aim, BancadaView};
use ph2d_tokens::{ROW_H_PX, Spacing, Theme};
use ph2d_vector::VectorScene;

/// `quem()` com `n` objectos (os dois de `quem()` repetidos): a grelha transborda e rola.
fn muitos(n: usize) -> Who {
    let mut who = quem();
    let per = who.columns.len();
    let (holders, cells) = (who.holders.clone(), who.cells.clone());
    who.holders = (0..n)
        .map(|i| Holder {
            bits: i as u64 + 1,
            name: format!("{} {i}", holders[i % 2].name),
            color: i % 2,
        })
        .collect();
    who.cells = (0..n)
        .flat_map(|i| cells[(i % 2) * per..(i % 2 + 1) * per].to_vec())
        .collect();
    who
}

#[test]
fn a_grelha_e_a_tabela_da_casa_e_os_alvos_saem_da_mesma_geometria() {
    let (who, mut ts) = (quem(), ts());
    let mut v = BancadaView::default();
    v.paint_bench(
        &who,
        GRADE,
        Some(LIVES),
        &mut CodeView::new(),
        &mut VectorScene::new(),
        &mut ts,
        &Theme::default(),
    );
    let (lay, rows, st) = v.grid_table().expect("a tabela pintada");
    assert_eq!(v.target(Aim::Cell(1, SPEED)), lay.cell_rect(1, "speed"));
    assert_eq!(v.target(Aim::Holder(0)), lay.cell_rect(0, "@name"));
    let head = lay.col("lives").expect("a coluna lives");
    assert_eq!(
        v.target(Aim::Column(LIVES)).map(|r| (r.x, r.y, r.w)),
        Some((head.x, lay.header.y, head.w))
    );
    assert_eq!(st.soft_col.as_deref(), Some("lives"), "a coluna do cursor");
    assert!(rows[1].cells[1].changed, "speed de Hero 2 mudou: âmbar");
    assert!(!rows[0].cells[1].changed, "o de Hero não");
    assert!(rows[0].cells[0].dot.is_some(), "o ponto da cor do objecto");
    assert!(
        rows[0].cells[1 + TINT].swatch.is_some(),
        "a cor é um quadrado"
    );
    // Pairar a célula acende a linha e a coluna dela; a mão no que se clica.
    let (x, y) = alvo(&v, Aim::Cell(1, SPEED));
    assert!(v.hover(x, y, 0));
    assert_eq!(v.look(x, y), Some(PointerLook::Hand));
    let (nx, ny) = alvo(&v, Aim::Holder(0));
    assert_eq!(v.look(nx, ny), Some(PointerLook::Hand), "o nome");
    let (hx, hy) = alvo(&v, Aim::Column(SPEED));
    assert_eq!(v.look(hx, hy), Some(PointerLook::Hand), "o cabeçalho");
    v.paint_bench(
        &who,
        GRADE,
        None,
        &mut CodeView::new(),
        &mut VectorScene::new(),
        &mut ts,
        &Theme::default(),
    );
    let (_, _, st) = v.grid_table().expect("a tabela pintada");
    assert_eq!(st.hover.as_deref(), Some("1"), "a linha de Hero 2 acesa");
    assert_eq!(st.lit_col.as_deref(), Some("speed"), "a coluna acesa");
    assert_eq!(st.soft_col, None);
}

#[test]
fn as_larguras_da_grelha_nao_encolhem_quando_o_valor_encolhe() {
    let (mut who, mut ts) = (quem(), ts());
    let mut v = BancadaView::default();
    let w = |v: &BancadaView| v.grid_table().unwrap().0.col("speed").unwrap().w;
    who.cells[6] = celula("1800000000", true);
    pinta_grade(&mut v, &who, &mut ts);
    let larga = w(&v);
    who.cells[6] = celula("1", true);
    pinta_grade(&mut v, &who, &mut ts);
    assert!((w(&v) - larga).abs() < 1e-3, "{} ≠ {larga}", w(&v));
}

#[test]
fn o_tecto_da_grelha_e_um_terco_em_linhas_inteiras() {
    let v = BancadaView::default();
    let (s, title) = (Spacing::Sm.px(), Spacing::Sm.px() * 3.0 + ROW_H_PX);
    for avail in [500.0, 517.0, 640.0, 901.0] {
        let body = v.bench_height(&muitos(40), avail) - title;
        let rows = body / ROW_H_PX;
        assert!((rows - rows.round()).abs() < 1e-3, "{avail}: {rows} linhas");
        assert!(body + title - s <= avail / 3.0 + 1e-3, "{avail}: {body}");
    }
}

#[test]
fn a_barra_da_grelha_arrastada_rola_em_linhas_inteiras() {
    let (who, mut ts) = (muitos(20), ts());
    let mut v = BancadaView::default();
    pinta_grade(&mut v, &who, &mut ts);
    let bar = v
        .grid_table()
        .unwrap()
        .0
        .bar()
        .hit()
        .expect("transborda: há barra");
    let (x, y) = (bar.x + bar.w * 0.5, bar.y + Spacing::Xs.px());
    assert_eq!(v.look(x, y), Some(PointerLook::ResizeNs));
    assert!(v.press(&who, x, y), "a barra é da Bancada");
    assert!(v.hover(x, y + ROW_H_PX * 1.3, 1), "o arrasto é tomado");
    assert!(v.release());
    pinta_grade(&mut v, &who, &mut ts);
    let (lay, ..) = v.grid_table().unwrap();
    assert!(lay.scroll > 0.0, "rolou");
    let n = lay.scroll / ROW_H_PX;
    assert!((n - n.round()).abs() < 1e-3, "em linhas inteiras: {n}");
    let first = lay.rows.first().expect("linhas à vista");
    assert!(first.index > 0 && (first.rect.y - lay.body.y).abs() < 1e-3);
    assert!(lay.cell_rect(0, "@name").is_none(), "a 1.ª saiu de vista");
    assert!(
        v.target(Aim::Cell(0, SPEED)).is_none(),
        "e o alvo dela também"
    );
    // Largado, o rato volta a só pairar: nada rola.
    let antes = lay.scroll;
    let _ = v.hover(x, y + ROW_H_PX * 5.0, 2);
    pinta_grade(&mut v, &who, &mut ts);
    assert!((v.grid_table().unwrap().0.scroll - antes).abs() < 1e-3);
}

#[test]
fn o_separador_da_grelha_muda_a_largura_do_mapa() {
    let (who, mut ts) = (quem(), ts());
    let mut v = BancadaView::default();
    pinta_grade(&mut v, &who, &mut ts);
    let s = v.split().expect("com o mapa há separador");
    let grid_w = v.grid_table().unwrap().0.area.w;
    let (x, y) = meio(s.hit);
    assert_eq!(v.look(x, y), Some(PointerLook::ResizeEw));
    assert!(v.press(&who, x, y));
    assert!(v.hover(x - 60.0, y, 1));
    assert!(v.release());
    pinta_grade(&mut v, &who, &mut ts);
    let s2 = v.split().unwrap();
    assert!(
        (s2.side_w - (s.side_w + 60.0)).abs() < 1e-3,
        "{} → {}",
        s.side_w,
        s2.side_w
    );
    let grid_w2 = v.grid_table().unwrap().0.area.w;
    assert!(
        (grid_w - grid_w2 - 60.0).abs() < 1e-3,
        "a grelha cede {grid_w} → {grid_w2}"
    );
    v.map_on = false;
    pinta_grade(&mut v, &who, &mut ts);
    assert!(v.split().is_none(), "sem mapa, sem separador");
}

/// Seis propriedades num cartão estreito com o mapa ao lado: as colunas que transbordam são
/// cortadas, e o rato sobre o mapa nunca acerta numa célula ou cabeçalho da grelha.
#[test]
fn as_celulas_cortadas_da_grelha_nao_tomam_o_rato_sobre_o_mapa() {
    let (who, mut ts) = (quem(), ts());
    let mut v = BancadaView::default();
    let area = Rect::new(0.0, 0.0, 420.0, GRADE.h);
    let mut scene = VectorScene::new();
    let view = &mut CodeView::new();
    v.paint_bench(
        &who,
        area,
        None,
        view,
        &mut scene,
        &mut ts,
        &Theme::default(),
    );
    let s = v.split().expect("com o mapa há separador");
    let (lay, ..) = v.grid_table().unwrap();
    let last = lay.cols.last().unwrap();
    assert!(
        last.x + last.w > s.hit.x + s.hit.w,
        "controlo: a grelha transborda para debaixo do mapa"
    );
    let x0 = s.hit.x + s.hit.w + 1.0;
    let mut tocou = 0;
    for i in 0..40 {
        for j in 0..26 {
            let (x, y) = (x0 + i as f32 * 4.0, j as f32 * 10.0);
            if x >= area.w {
                continue;
            }
            let _ = v.hover(x, y, 0);
            let a = v.aim();
            assert!(
                !matches!(a, Aim::Cell(..) | Aim::Column(_)),
                "({x}, {y}) → {a:?}"
            );
            tocou += usize::from(matches!(a, Aim::Holder(_)));
        }
    }
    assert!(tocou > 0, "controlo: o mapa tem objectos sob o rato");
}

/// Smoke 09/11: os números da grelha e os valores do balão na mono do CÓDIGO (a da planilha), não
/// na letra da interface — cada tabela molda-os na cache da vista que recebe.
#[test]
fn os_numeros_da_grelha_e_do_balao_pintam_na_mono_do_codigo() {
    let (who, mut ts) = (quem(), ts());
    let (grade, balao) = (&mut CodeView::new(), &mut CodeView::new());
    let mut v = BancadaView::default();
    let mut scene = VectorScene::new();
    let theme = Theme::default();
    v.paint_bench(&who, GRADE, None, grade, &mut scene, &mut ts, &theme);
    assert!(grade.cache().shapes() > 0, "a grelha moldou os números");
    v.set_marks(&[(0, MARCA)]);
    let (x, y) = meio(MARCA);
    assert!(v.hover(x, y, 0), "a marca abre o balão");
    v.paint_popover(&who, LIMITES, balao, &mut scene, &mut ts, &theme);
    assert!(v.popover_table().is_some(), "o balão pintou a tabela");
    assert!(balao.cache().shapes() > 0, "o balão moldou os valores");
}

/// Fecho 10/11: a primeira coluna da grelha tem título, o mesmo «Name» da folha do Stopped (a tabela
/// irmã na mesma aba); antes o cabeçalho dela estava vazio.
#[test]
fn a_primeira_coluna_da_grelha_chama_se_name_como_na_folha_do_stopped() {
    let (who, mut ts) = (quem(), ts());
    let mut v = BancadaView::default();
    let mut scene = VectorScene::new();
    v.paint_bench(
        &who,
        GRADE,
        None,
        &mut CodeView::new(),
        &mut scene,
        &mut ts,
        &Theme::default(),
    );
    let (lay, ..) = v.grid_table().expect("a tabela pintada");
    let (col, head) = (lay.col("@name").expect("a coluna dos nomes"), lay.header);
    assert_eq!(col.x, lay.area.x, "é a primeira");
    let titulo = crate::linhas_de_base(&scene)
        .into_iter()
        .find(|&(_, x, b)| x >= col.x && x < col.x + col.w && b > head.y && b < head.y + head.h);
    assert!(
        titulo.is_some(),
        "o cabeçalho da coluna dos nomes pintou texto"
    );
    assert_eq!(ph2d_i18n::tr("code.debug.col.name"), "Name");
}
