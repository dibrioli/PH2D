//! ⭐⭐ **Os cartões Advanced e Render no vocabulário** (dono 03/11, doc `CartoesUI/08` §1.2): gestos
//! reais pelo mesmo [`Galeria`] do irmão, e o que a pintura seguinte mostra.

use super::cartoes::Galeria;
use bumpalo::Bump;
use ph2d_editor_core::interaction::dispatch_text_input;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::widget::card_field::painted_letter;
use ph2d_editor_core::widget::{BitmaskGrid32, TagTone};
use ph2d_panel_widget_gallery::cards_render::{Fase, PASSOS_POR_QUADRO, QUADROS, pintado};
use ph2d_panel_widget_gallery::ids as g;
use ph2d_panel_widget_gallery::state::{Flag, Pick};

/// O dono (03/11, foto 2): campos à Figma, com a letra e sem setas. *Mutação: a linha do W·H com
/// `Glyph::Text(x)`; `ADV_X` fora da tabela `FIELDS` (o campo nasce morto).*
#[test]
fn bounds_e_rotation_sao_campos_figma_vivos() {
    let mut gal = Galeria::aberta();
    for (id, letra) in [
        (g::ADV_X, "X"),
        (g::ADV_Y, "Y"),
        (g::ADV_W, "W"),
        (g::ADV_H, "H"),
    ] {
        assert!(gal.rect(id).is_some(), "{letra} não foi pintado");
        assert_eq!(painted_letter(id).as_deref(), Some(letra));
    }
    assert_eq!(
        painted_letter(g::ADV_ROTATION),
        None,
        "o ângulo tem ícone que o quer dizer"
    );
    let antes = gal.host.store().number_value(g::ADV_W).expect("W");
    assert_eq!(antes, 100.0, "fixtura: W de fábrica");
    gal.arrasta_o_icone(g::ADV_W, 12.0);
    let depois = gal.host.store().number_value(g::ADV_W).expect("W");
    assert!(depois > antes, "arrastar o W não mudou o valor: {depois}");
}

/// Toda camada acende e apaga pelo clique, e a célula é a da grelha. *Mutação: `Kind::Layer` só
/// consumido; `BitmaskGrid32::cell_rect` com a coluna trocada.*
#[test]
fn a_camada_de_colisao_alterna_pelo_clique() {
    let mut gal = Galeria::aberta();
    let fabrica = gal.st.choices.layers;
    let origem = gal.rect(g::LAYERS[0]).expect("camada 1");
    for bit in [1usize, 8, 31] {
        let r = gal.rect(g::LAYERS[bit]).expect("camada");
        let esperado = BitmaskGrid32::cell_rect(origem.x, origem.y, origem.w * 8.0, origem.h, bit);
        assert!(
            (r.x - esperado.x).abs() < 0.5 && (r.y - esperado.y).abs() < 0.5,
            "a camada {bit} não está na sua célula"
        );
        let era = gal.st.choices.layers >> bit & 1;
        gal.clica(g::LAYERS[bit]);
        assert_ne!(gal.st.choices.layers >> bit & 1, era, "camada {bit}");
        gal.clica(g::LAYERS[bit]);
    }
    assert_eq!(gal.st.choices.layers, fabrica, "dois cliques não voltaram");
}

/// A pílula do Slice: um clique, um aceso. *Mutação: `Pick::SliceMode` em `opt` trocado.*
#[test]
fn o_slice_mode_escolhe_na_pilula() {
    let mut gal = Galeria::aberta();
    assert_eq!(gal.st.choices.pick(Pick::SliceMode), 1, "fixtura: Sliced");
    gal.clica(g::SLICE[3]);
    assert_eq!(gal.st.choices.pick(Pick::SliceMode), 3);
    let (a, b) = (
        gal.rect(g::SLICE[0]).expect("Simple"),
        gal.rect(g::SLICE[3]).expect("Tiled Fit"),
    );
    assert_eq!(a.y, b.y, "os quatro cabem numa linha nesta largura");
}

/// O tipo e o valor na MESMA linha, um valor por tipo; o tipo troca o widget. *Mutação: `meta_row`
/// com o valor numa segunda `c.row`; o `match` sempre no 0.*
#[test]
fn metadata_o_tipo_e_o_valor_na_mesma_linha_e_o_tipo_troca_o_valor() {
    let mut gal = Galeria::aberta();
    let linha = |gal: &Galeria, i: usize, valor| {
        let (t, v) = (
            gal.rect(g::META[i].kind).expect("tipo"),
            gal.rect(valor).expect("valor"),
        );
        assert_eq!(
            t.y, v.y,
            "entrada {i}: o tipo e o valor em linhas diferentes"
        );
        assert!(t.x + t.w <= v.x, "entrada {i}: o tipo sobrepõe o valor");
    };
    linha(&gal, 0, g::META[0].number);
    linha(&gal, 1, g::META[1].text);
    linha(&gal, 2, g::META[2].swatch);
    assert_eq!(
        gal.host.store().text(g::META[1].text),
        Some("muzzle"),
        "fixtura: o texto de fábrica"
    );
    assert!(
        gal.rect(g::META[0].text).is_none(),
        "Float com caixa de texto"
    );
    // O tipo troca o valor, pela lista suspensa de verdade.
    gal.clica(g::META[0].kind);
    gal.clica(g::META[0].options[1]);
    assert_eq!(gal.st.choices.pick(Pick::Meta0), 1);
    assert!(
        gal.rect(g::META[0].number).is_none(),
        "Text manteve o número"
    );
    linha(&gal, 0, g::META[0].text);
}

/// O «+» abre os tipos e a entrada nasce com o escolhido; o «−» tira-a. Cheia, o «+» sai (um «+» que
/// não faz nada é controlo morto). *Mutação: `Kind::AddMeta` só consumido; o `add` sempre `Some`.*
#[test]
fn metadata_o_mais_acrescenta_com_o_tipo_e_o_menos_tira() {
    let mut gal = Galeria::aberta();
    assert!(!gal.st.choices.on(Flag::Meta3), "fixtura: três entradas");
    gal.clica(g::META_ADD);
    gal.clica(g::META_ADD_COLOR);
    assert!(gal.st.choices.on(Flag::Meta3), "o «+» não acrescentou");
    assert_eq!(gal.st.choices.pick(Pick::Meta3), 2, "nasceu sem o tipo");
    assert!(gal.rect(g::META[3].swatch).is_some(), "a cor não se pintou");
    gal.clica(g::META[3].remove);
    assert!(gal.rect(g::META[3].kind).is_none(), "o «−» não tirou");
    for _ in 0..3 {
        gal.clica(g::META_ADD);
        gal.clica(g::META_ADD_TEXT);
    }
    assert!(gal.rect(g::META_ADD).is_none(), "cheia e o «+» ficou");
}

/// A amostra da entrada Color abre o seletor do app. *Mutação: `Kind::Swatch(META_SWATCH + i)` errado.*
#[test]
fn metadata_a_amostra_abre_o_seletor() {
    let mut gal = Galeria::aberta();
    gal.clica(g::META[2].swatch);
    assert_eq!(gal.host.store().picker_target(), Some(g::META[2].swatch));
}

/// Chave + valor + lixo numa linha; o texto escreve-se; «+» e lixo agem. *Mutação: `PROPS` fora de
/// `TEXTS` (o texto nasce morto); `Kind::Remove` com a linha trocada.*
#[test]
fn custom_properties_chave_valor_e_lixo_numa_linha() {
    let mut gal = Galeria::aberta();
    let r = g::PROPS[0];
    let (k, v, t) = (
        gal.rect(r.key).expect("chave"),
        gal.rect(r.value).expect("valor"),
        gal.rect(r.remove).expect("lixo"),
    );
    assert!(k.y == v.y && v.y == t.y, "não estão numa linha só");
    assert!(k.x < v.x && v.x < t.x);
    gal.clica(r.value);
    let arena = Bump::new();
    let _ = dispatch_text_input(gal.host.store_mut(), '9', &arena);
    gal.repinta();
    let valor = gal.host.store().text(r.value).unwrap_or_default();
    assert!(valor.contains('9'), "a tecla não chegou ao valor: {valor}");
    gal.clica(g::PROP_ADD);
    assert!(gal.rect(g::PROPS[1].key).is_some(), "o «+» não acrescentou");
    gal.clica(g::PROPS[1].remove);
    assert!(gal.rect(g::PROPS[1].key).is_none(), "o lixo não tirou");
}

/// Sem correr: «Ready», sem spinner, sem Cancel (não há o que cancelar); as etiquetas dizem o que são.
/// *Mutação: `spinner: true`; o Cancel sempre pintado; «PRF» de volta.*
#[test]
fn o_render_parado_diz_ready_sem_spinner() {
    let mut gal = Galeria::aberta();
    gal.pinta(2);
    let l = pintado().expect("o cartão pintou");
    assert_eq!(l.status, "Ready");
    assert_eq!(l.percent, 0);
    assert!(!l.spinner, "spinner parado à vista");
    assert_eq!(
        l.tags,
        vec![
            ("Preview".to_owned(), TagTone::Accent),
            ("Ready".to_owned(), TagTone::Success),
            ("Hot reload".to_owned(), TagTone::Warn),
        ]
    );
    assert!(gal.rect(g::RN_RUN).is_some());
    assert!(
        gal.rect(g::RN_CANCEL).is_none(),
        "Cancel sem o que cancelar"
    );
}

/// ⭐ Render corre: o quadro anda um passo por pintura, determinista, o estado e as etiquetas mudam, os
/// medidores mexem; Cancel volta a «Ready». *Mutação: `avanca` não chamado; `cancel` só `fase`.*
#[test]
fn o_render_corre_por_quadro_e_o_cancel_repoe() {
    let mut gal = Galeria::aberta();
    gal.clica(g::RN_RUN);
    assert_eq!(gal.st.render.fase, Fase::Correndo);
    let passos = gal.st.render.passos;
    gal.pinta(10);
    assert_eq!(
        gal.st.render.passos,
        passos + 10,
        "um passo por quadro pintado"
    );
    let total = QUADROS[1];
    let q = gal.st.render.passos / PASSOS_POR_QUADRO;
    let l = pintado().expect("o cartão pintou");
    assert_eq!(l.status, format!("Rendering frame {q} / {total}"));
    assert!(l.spinner, "a correr sem spinner");
    assert_eq!(l.tags[1], ("Rendering".to_owned(), TagTone::Warn));
    assert_eq!(l.tags[2].1, TagTone::Neutral, "o hot reload pausa a correr");
    let antes = l.niveis;
    gal.pinta(5);
    assert_ne!(pintado().expect("pintou").niveis, antes, "medidor parado");
    assert!(gal.rect(g::RN_CANCEL).is_some(), "a correr sem Cancel");
    gal.clica(g::RN_CANCEL);
    assert_eq!(gal.st.render.fase, Fase::Pronta);
    gal.pinta(1);
    let l = pintado().expect("pintou");
    assert_eq!(
        (l.status.as_str(), l.percent, l.spinner),
        ("Ready", 0, false)
    );
}

/// A corrida acaba a 100 %, sem spinner, com o nome do formato; a qualidade muda o total e o formato
/// muda o nome. *Mutação: `Fase::Feita` nunca; a qualidade fora de `QUADROS`.*
#[test]
fn o_render_acaba_e_a_saida_decide_quadros_e_nome() {
    let mut gal = Galeria::aberta();
    gal.clica(g::RN_QUALITY[0]);
    gal.clica(g::RN_FORMAT);
    gal.clica(g::RN_FORMATS[1]);
    gal.clica(g::RN_RUN);
    gal.pinta(2);
    let l = pintado().expect("pintou");
    assert!(
        l.status.ends_with(&format!("/ {}", QUADROS[0])),
        "{}",
        l.status
    );
    gal.pinta(QUADROS[0] * PASSOS_POR_QUADRO);
    assert_eq!(gal.st.render.fase, Fase::Feita);
    gal.pinta(1);
    let l = pintado().expect("pintou");
    assert_eq!(l.status, "Saved render.mp4");
    assert_eq!((l.percent, l.spinner), (100, false));
    assert_eq!(l.tags[1], ("Done".to_owned(), TagTone::Success));
    // Mudar a saída invalida o resultado.
    gal.clica(g::RN_QUALITY[2]);
    assert_eq!(gal.st.render.fase, Fase::Pronta);
}
