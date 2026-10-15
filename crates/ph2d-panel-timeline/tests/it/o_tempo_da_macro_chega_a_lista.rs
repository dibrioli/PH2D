//! ⭐ **O RELÓGIO DA MACRO É ALCANÇÁVEL PELO ARTISTA** — re-cronometrar uma gravação, ou recriar a
//! faixa que o Gravar ▸ Parar fez. Sem a linha no «+ Track» só um teste lhe chega (a lição do Morph).

use ph2d_timeline::PropKind;

#[test]
fn o_tempo_da_macro_esta_na_lista_do_mais_track() {
    let tabela = ph2d_panel_timeline::ids::ADDPROP_BUTTONS;
    assert!(
        tabela.iter().any(|(_, p)| *p == PropKind::MacroTime),
        "o relogio da macro nao esta' na lista do «+ Track»"
    );
    let ids: std::collections::BTreeSet<_> = tabela.iter().map(|(id, _)| *id).collect();
    assert_eq!(
        ids.len(),
        tabela.len(),
        "dois botoes do «+ Track» partilham o id"
    );
}
