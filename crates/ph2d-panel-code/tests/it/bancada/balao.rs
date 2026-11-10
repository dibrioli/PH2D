//! ⭐ **O balão «em quem» na tabela da casa** (plano 14 Q4 C): uma tabela SEM cabeçalho logo abaixo da
//! faixa do título, uma linha por objecto; os alvos do pairar e do clique são as linhas dela (a MESMA
//! geometria), e a linha do objecto que mudou o valor é âmbar e diz «changed».

use super::{balao_aberto, quem};
use crate::ts;
use ph2d_panel_code::bancada::Aim;
use ph2d_tokens::{ROW_H_PX, Spacing};

#[test]
fn o_balao_e_uma_tabela_sem_cabecalho_e_os_alvos_sao_as_linhas_dela() {
    let (who, mut ts) = (quem(), ts());
    let v = balao_aberto(&who, &mut ts);
    let (lay, rows) = v.popover_table().expect("o balão pintou a tabela");
    assert_eq!(lay.header.h, 0.0, "sem cabeçalho");
    assert_eq!(lay.body, lay.area, "o corpo é a tabela inteira");
    let pop = v.target(Aim::Mark(0)).expect("o balão inteiro");
    let strip = pop.y + Spacing::Sm.px() + ROW_H_PX;
    assert_eq!(
        lay.area.y, strip,
        "a 1.ª linha logo abaixo da faixa do título"
    );
    assert_eq!(rows.len(), who.holders.len());
    assert_eq!(lay.rows.len(), who.holders.len(), "todas à vista");
    for span in &lay.rows {
        let hit = v.target(Aim::Holder(span.index)).expect("o alvo da linha");
        assert_eq!((hit.y, hit.h), (span.rect.y, span.rect.h), "{span:?}");
        assert_eq!(rows[span.index].cells[0].text, who.holders[span.index].name);
    }
    let select = v.target(Aim::Select(1)).expect("o «Select»");
    let last = lay.rows.last().unwrap().rect;
    assert!(select.y >= last.y + last.h, "o «Select» abaixo da tabela");
}

#[test]
fn a_linha_mudada_do_balao_e_ambar_e_diz_changed() {
    let (who, mut ts) = (quem(), ts());
    let v = balao_aberto(&who, &mut ts);
    let (_, rows) = v.popover_table().unwrap();
    let texto = |h: usize| -> Vec<&str> { rows[h].cells.iter().map(|c| c.text.as_str()).collect() };
    assert_eq!(texto(0), ["Hero", "220", "default"]);
    assert_eq!(texto(1), ["Hero 2", "180", "changed"]);
    assert!(!rows[0].changed, "o Hero tem o valor do script");
    assert!(rows[1].changed, "o Hero 2 mudou: a linha inteira em âmbar");
    assert!(
        rows.iter().all(|r| r.cells[0].dot.is_some()),
        "o ponto da cor"
    );
}
