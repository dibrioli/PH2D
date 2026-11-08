//! A busca no popover do Preset: o filtro (nome, sem caixa nem acento, só os títulos com achados, a
//! linha «No brushes found») e as setas (só entre pincéis, presas às pontas).

use super::*;
use crate::paint_wetpaint_presets::opcoes_filtradas;
use crate::state_wet_presets::{VistaPreset, VistaPresets};

fn e(nome: &str, categoria: &str, favorito: bool) -> VistaPreset {
    VistaPreset {
        relativo: format!("{categoria}/{nome}.ph2d-brush.png"),
        nome: nome.to_owned(),
        categoria: categoria.to_owned(),
        favorito,
        apagavel: false,
    }
}

fn vista() -> VistaPresets {
    VistaPresets {
        entradas: vec![
            e("Gouache Flat", "Gouache", true),
            e("Round Gouache", "Gouache", false),
            e("Pencil HB", "Pencil", false),
            e("Pinceau Léger", "Watercolor", false),
            e("Wet Round", "Watercolor", false),
        ],
        ativo: None,
        politica: 0,
        ilegiveis: 1,
        ordem_manual: false,
    }
}

fn rotulos(o: &[DropdownOption<usize>]) -> Vec<String> {
    o.iter().map(|o| o.label.clone()).collect()
}

#[test]
fn a_busca_filtra_pelo_nome_sem_caixa_nem_acento_e_guarda_so_os_titulos_com_achados() {
    let fav = tr("panel.painter_layers.wetpaint.preset_favorites").to_owned();
    let (o, _) = opcoes_filtradas(&vista(), "GOU");
    assert_eq!(
        rotulos(&o),
        [fav.as_str(), "Gouache Flat", "Gouache", "Round Gouache"]
    );
    let (o, _) = opcoes_filtradas(&vista(), "leger");
    assert_eq!(rotulos(&o), ["Watercolor", "Pinceau Léger"]);
    let (o, _) = opcoes_filtradas(&vista(), "round");
    assert_eq!(
        rotulos(&o),
        ["Gouache", "Round Gouache", "Watercolor", "Wet Round"]
    );
    // Vazia: a lista inteira, com o aviso dos ilegíveis.
    let (o, _) = opcoes_filtradas(&vista(), "  ");
    assert_eq!(o.len(), 1 + 1 + 4 + 2 + 2, "aviso + 4 títulos + 5 pincéis");
}

#[test]
fn sem_achados_a_lista_diz_no_brushes_found_e_nada_se_escolhe() {
    let (o, _) = opcoes_filtradas(&vista(), "zzz");
    assert_eq!(
        rotulos(&o),
        [tr("panel.painter_layers.wetpaint.preset_no_match")]
    );
    assert!(o[0].disabled && !o[0].header);
    assert_eq!(passo(&o, None, 1), None);
}

#[test]
fn as_setas_andam_so_entre_pinceis_e_param_nas_pontas() {
    let (o, _) = opcoes_filtradas(&vista(), "round");
    let valor = |nome: &str| o.iter().find(|x| x.label == nome).map(|x| x.value);
    let (rg, wr) = (valor("Round Gouache"), valor("Wet Round"));
    // Sem partida: ↓ vai ao primeiro pincel (salta o título), ↑ ao último.
    assert_eq!(passo(&o, None, 1), rg);
    assert_eq!(passo(&o, None, -1), wr);
    // Salta o título do meio, e pára nas pontas.
    assert_eq!(passo(&o, rg, 1), wr);
    assert_eq!(passo(&o, wr, 5), wr);
    assert_eq!(passo(&o, wr, -9), rg);
}

#[test]
fn a_linha_do_campo_e_um_titulo_vazio_no_topo() {
    let (o, _) = opcoes_filtradas(&vista(), "pencil");
    let c = com_campo(o);
    assert!(c[0].header && c[0].label.is_empty());
    assert_eq!(c.len(), 3);
    assert_eq!(
        passo(&c, None, 1),
        Some(c[2].value),
        "o campo não se realça"
    );
}

#[test]
fn a_dobra_tira_caixa_e_acento() {
    assert_eq!(dobrar("ÁguA Çéu Ñ"), "agua ceu n");
    assert!(casa("Pinceau Léger", "LEG"));
    assert!(!casa("Pencil", "gou"));
    assert!(casa("Pencil", ""));
}
