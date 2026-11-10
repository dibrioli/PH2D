//! Smoke do dono 09/11: numa lista plana (o balão, o Problems) a 1.ª coluna não guarda o lugar do ▸,
//! e os fios das colunas acabam na última linha.

use super::{cols, item, lay};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_ui_table::{CellText, Ink, Metrics, Row, natural_widths, rules_bottom};
use ph2d_vector::VectorScene;

/// Uma letra = 7 px; não pinta.
struct Sete;

impl CellText for Sete {
    fn width(&mut self, _: &mut TextSystem, text: &str, _: bool) -> f32 {
        text.chars().count() as f32 * 7.0
    }

    fn paint(
        &mut self,
        _: &mut TextSystem,
        _: &mut VectorScene,
        _: &str,
        _: Ink,
        _: bool,
        _: f32,
        _: (f32, f32),
        _: f32,
    ) {
    }
}

const NOME: &str = "a_long_name_wider_than_two_rows";

fn largura_do_nome(rows: &[Row], m: &Metrics) -> f32 {
    let mut ts = TextSystem::without_system_fonts();
    natural_widths(&cols(), rows, m, &mut ts, &mut Sete)[0]
}

#[test]
fn a_lista_plana_nao_reserva_o_chevron_na_primeira_coluna() {
    let m = Metrics::standard();
    let texto = NOME.len() as f32 * 7.0;
    let plana = [item(NOME, 0, false), item("b", 0, false)];
    assert_eq!(m.twist(&plana), 0.0, "nenhuma linha mostra ▸");
    assert!((largura_do_nome(&plana, &m) - (texto + m.pad * 2.0)).abs() < 1e-3);
    let arvore = [item(NOME, 0, false), item("b", 0, true)];
    assert_eq!(
        m.twist(&arvore),
        m.chevron(),
        "uma linha com ▸ reserva-o em todas"
    );
    let com = largura_do_nome(&arvore, &m);
    assert!((com - (texto + m.chevron() + m.pad * 2.0)).abs() < 1e-3);
}

#[test]
fn os_fios_das_colunas_acabam_na_ultima_linha() {
    let m = Metrics::standard();
    let mut st = ph2d_ui_table::TableState::default();
    let alta = Rect::new(0.0, 0.0, 600.0, m.header_h + m.row_h * 40.0);
    let (l, _) = lay(&mut st, alta);
    let ultima = l.rows.last().expect("linhas à vista").rect;
    assert!(
        ultima.y + ultima.h < l.body.y + l.body.h - m.row_h,
        "sobra vazio"
    );
    assert_eq!(
        rules_bottom(&l),
        ultima.y + ultima.h,
        "o fio pára na última linha"
    );
    // A transbordar o corpo só mostra linhas inteiras: a meia linha do pé fica vazia, e sem fio.
    let baixa = Rect::new(0.0, 0.0, 600.0, m.header_h + m.row_h * 2.5);
    let (l, _) = lay(&mut st, baixa);
    let ultima = l.rows.last().expect("linhas à vista").rect;
    assert!(ultima.y + ultima.h < l.body.y + l.body.h);
    assert_eq!(rules_bottom(&l), ultima.y + ultima.h);
}
