//! ⭐ **A lista de problemas na tabela da casa** (plano 14 Q4 C): colunas fixas (o ponto da
//! gravidade, o texto que estica, «On» e a linha encostada à direita), o «On» do erro de corrida e o
//! «—» do assistente, a linha do F8 seleccionada e rolada à vista em linhas inteiras, e o leitor de
//! ecrã sobre os MESMOS rectângulos que a tabela pintou.

use crate::ts;
use ph2d_a11y::{NodeId, Role};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::{
    Band, BandTab, ConsoleRow, ConsoleSource, ConsoleView, ProblemRow, Severity,
};
use ph2d_tokens::{ROW_H_PX, Theme};
use ph2d_vector::VectorScene;

struct Nada;

impl ConsoleSource for Nada {
    fn len(&self) -> usize {
        0
    }
    fn row(&self, _: usize) -> ConsoleRow<'_> {
        unreachable!("o console está vazio")
    }
}

/// A faixa (cabeçalho + quatro linhas da tabela por baixo do cabeçalho dela).
const FAIXA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 800.0,
    h: ROW_H_PX * 6.0,
};

/// Pinta a faixa; devolve quantas linhas a mono do código moldou (os números da coluna Line).
fn pinta(v: &mut ConsoleView, rows: &[ProblemRow<'_>], current: Option<usize>) -> u64 {
    let band = Band {
        open: true,
        tab: BandTab::Problems,
        console: &Nada,
        problem_count: rows.len(),
        problems: rows,
        current,
        tab_width: 4,
    };
    let mut scene = VectorScene::new();
    let view = &mut ph2d_panel_code::CodeView::new();
    v.paint(band, FAIXA, view, &mut scene, &mut ts(), &Theme::default());
    view.cache().shapes()
}

fn linha<'a>(severity: Severity, text: &'a str, line: usize, on: &'a str) -> ProblemRow<'a> {
    ProblemRow {
        severity,
        text,
        line,
        on,
    }
}

#[test]
fn as_colunas_sao_fixas_e_a_linha_encosta_a_direita() {
    let rows = [
        linha(
            Severity::Error,
            "Unknown global «math.sinn»",
            8,
            "Circle, Star +1",
        ),
        linha(Severity::Warning, "«speed» is never used", 2, ""),
    ];
    let mut v = ConsoleView::default();
    assert!(pinta(&mut v, &rows, None) > 0, "a linha na mono do código");
    let (lay, trows) = v.problems_table().expect("a tabela pintada");
    let keys: Vec<&str> = lay.cols.iter().map(|c| &*c.key).collect();
    assert_eq!(keys, ["sev", "problem", "on", "line"]);
    let last = lay.cols.last().unwrap();
    assert_eq!(
        last.x + last.w,
        lay.area.x + lay.area.w,
        "a linha encosta à direita"
    );
    assert!(lay.header.h > 0.0, "com cabeçalho");
    assert_eq!(
        trows[0].cells[2].text, "Circle, Star +1",
        "o «On» do erro de corrida"
    );
    assert_eq!(
        trows[1].cells[2].text, "\u{2014}",
        "o assistente não sabe em quem"
    );
    assert_eq!(trows[0].cells[3].text, "8");
    assert!(trows[0].cells[0].dot.is_some(), "a gravidade é um ponto");
    let p = &v.probe().problems;
    assert_eq!(p[1].on, "\u{2014}");
    // O leitor de ecrã diz em quem (e nada quando ninguém).
    assert_eq!(
        p[0].label,
        "Error: Unknown global «math.sinn», line 8, on Circle, Star +1"
    );
    assert_eq!(p[1].label, "Warning: «speed» is never used, line 2");
}

#[test]
fn a_linha_do_f8_rola_a_vista_em_linhas_inteiras_e_o_leitor_le_os_rects_da_tabela() {
    let textos: Vec<String> = (0..20).map(|i| format!("problem {i}")).collect();
    let rows: Vec<ProblemRow<'_>> = textos
        .iter()
        .enumerate()
        .map(|(i, t)| linha(Severity::Warning, t, i + 1, ""))
        .collect();
    let mut v = ConsoleView::default();
    pinta(&mut v, &rows, None);
    pinta(&mut v, &rows, Some(15));
    let (lay, _) = v.problems_table().unwrap();
    assert_eq!(
        lay.scroll % ROW_H_PX,
        0.0,
        "linhas inteiras: {}",
        lay.scroll
    );
    let body = lay.body;
    assert!(
        lay.rows
            .iter()
            .all(|s| s.rect.y >= body.y && s.rect.y + s.rect.h <= body.y + body.h)
    );
    let probe = v.probe();
    let sel: Vec<usize> = probe
        .problems
        .iter()
        .filter(|p| p.selected)
        .map(|p| p.index)
        .collect();
    assert_eq!(sel, [15], "a linha do F8 está à vista e seleccionada");
    let spans: Vec<Rect> = lay.rows.iter().map(|s| s.rect).collect();
    let (_, nos) = v.a11y(NodeId(0));
    let itens: Vec<Rect> = nos
        .iter()
        .filter(|(_, n)| n.role() == Role::ListItem)
        .map(|(_, n)| {
            let b = n.bounds().expect("com limites");
            Rect::new(
                b.x0 as f32,
                b.y0 as f32,
                (b.x1 - b.x0) as f32,
                (b.y1 - b.y0) as f32,
            )
        })
        .collect();
    assert_eq!(itens, spans, "o leitor de ecrã lê as linhas da tabela");
}
