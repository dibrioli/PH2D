//! ⭐ **Cada controlo das três janelas, pelo clique real, muda o que sai no verbo.** O canal é o que a
//! família lê: nada aqui fala com o estado do painel por dentro.

use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::cards::dropdown;
use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_pixel_export::canal::{self, Contexto, Imagem, Pedido, Tela};
use ph2d_panel_pixel_export::{PixelExportPanel, PixelExportPanelState, ids};
use ph2d_pixel_core::LayerId;
use ph2d_pixel_io::exporta::Export;
use ph2d_pixel_io::render::{Camadas, Direcao, Quadros};
use ph2d_pixel_io::saida::Formato;
use ph2d_pixel_io::sheet::{Limite, Tipo};
use ph2d_ui_testkit::mesa::Mesa;
use std::sync::Arc;

const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1930.0,
    h: 1800.0,
};

fn mesa(t: Tela) -> Mesa<PixelExportPanel> {
    canal::publish_contexto(Contexto {
        lados: (64, 32),
        quadros: 16,
        camadas: vec![
            (LayerId(7), "Top".to_owned()),
            (LayerId(3), "Base".to_owned()),
        ],
        tags: vec!["walk".to_owned(), "jump".to_owned()],
    });
    canal::publish_imagem(Some(Imagem {
        rgba: Arc::new(vec![255; 64 * 32 * 4]),
        w: 64,
        h: 32,
        seq: 1,
        lados_reais: (64, 32),
        entradas: 0,
        sugestao: None,
    }));
    let _ = canal::take_pedido();
    canal::abrir(t);
    let mut m0 = Mesa::nova(PixelExportPanelState::default());
    m0.host.set_panel_visible(PixelExportPanel::ID, true);
    m0.assenta_em(JANELA);
    m0
}

fn clica(m0: &mut Mesa<PixelExportPanel>, id: NodeId) {
    let _ = m0.clica(id);
    m0.assenta_em(JANELA);
}

/// Escreve o número pelo teclado real; o painel lê-o do store.
fn escreve(m0: &mut Mesa<PixelExportPanel>, id: NodeId, texto: &str) {
    let ev = m0.host.type_into_number(id, texto);
    let _ = m0.aplica(ev);
}

/// Abre a lista pelo clique na caixa e escolhe a linha.
fn escolhe(m0: &mut Mesa<PixelExportPanel>, caixa: NodeId, linha: NodeId) {
    assert!(m0.rect(caixa).is_some(), "a caixa da lista não se pintou");
    assert!(dropdown::on_click(caixa), "a caixa não foi declarada");
    m0.assenta_em(JANELA);
    clica(m0, linha);
    dropdown::on_click(caixa);
}

fn verbo(m0: &mut Mesa<PixelExportPanel>) -> Pedido {
    clica(m0, ids::OK);
    canal::take_pedido().expect("o verbo não pediu nada")
}

fn export_as(m0: &mut Mesa<PixelExportPanel>) -> ph2d_pixel_io::exporta::ExportAs {
    match verbo(m0) {
        Pedido::Exportar(Export::Ficheiro(x)) => x,
        o => panic!("{o:?}"),
    }
}

fn folha(m0: &mut Mesa<PixelExportPanel>) -> ph2d_pixel_io::exporta::ExportFolha {
    match verbo(m0) {
        Pedido::Exportar(Export::Folha(x)) => x,
        o => panic!("{o:?}"),
    }
}

/// *Mutação: um arm a menos em `clique` ⇒ vermelho aqui.*
#[test]
fn export_as_cada_controlo_chega_ao_verbo() {
    let mut m0 = mesa(Tela::ExportAs);
    assert_eq!(m0.st.tela(), Tela::ExportAs);
    for (id, f) in [
        (ids::FORMATS[1], Formato::Gif),
        (ids::FORMATS[2], Formato::Apng),
        (ids::FORMATS[3], Formato::Webp),
        (ids::FORMATS[4], Formato::Ase),
        (ids::FORMATS[0], Formato::Png),
    ] {
        clica(&mut m0, id);
        assert_eq!(export_as(&mut m0).formato, f);
    }
    escolhe(&mut m0, ids::LAYERS, ids::LAYERS_ALL);
    assert_eq!(export_as(&mut m0).pedido.camadas, Camadas::Todas);
    escolhe(&mut m0, ids::LAYERS, ids::layer_option(1));
    assert_eq!(export_as(&mut m0).pedido.camadas, Camadas::Uma(LayerId(3)));
    escolhe(&mut m0, ids::FRAMES, ids::tag_option(1));
    assert_eq!(export_as(&mut m0).pedido.quadros, Quadros::Tag(1));
    for (i, d) in [
        Direcao::Forward,
        Direcao::Reverse,
        Direcao::PingPong,
        Direcao::PingPongReverse,
    ]
    .into_iter()
    .enumerate()
    {
        escolhe(&mut m0, ids::DIRECTION, ids::DIRECTIONS[i]);
        assert_eq!(export_as(&mut m0).pedido.direcao, d);
    }
    escreve(&mut m0, ids::SCALE, "250");
    m0.assenta_em(JANELA);
    assert!((export_as(&mut m0).pedido.escala - 2.5).abs() < 1e-9);
}

/// O intervalo mostra os dois campos só quando escolhido, e eles contam de 1 na tela.
#[test]
fn o_intervalo_conta_de_um_na_tela_e_de_zero_no_documento() {
    let mut m0 = mesa(Tela::ExportAs);
    assert!(m0.rect(ids::FROM).is_none(), "o campo sem intervalo");
    escolhe(&mut m0, ids::FRAMES, ids::FRAMES_RANGE);
    assert_eq!(
        export_as(&mut m0).pedido.quadros,
        Quadros::Intervalo { de: 0, ate: 15 }
    );
    escreve(&mut m0, ids::FROM, "3");
    escreve(&mut m0, ids::TO, "5");
    m0.assenta_em(JANELA);
    assert_eq!(
        export_as(&mut m0).pedido.quadros,
        Quadros::Intervalo { de: 2, ate: 4 }
    );
}

/// Sem laço em PNG; laço em GIF. Nada de conteúdo no `.ase`, que leva o sprite inteiro.
#[test]
fn cada_formato_mostra_so_o_que_vale_para_ele() {
    let mut m0 = mesa(Tela::ExportAs);
    assert!(m0.rect(ids::LOOPS).is_none());
    assert!(m0.rect(ids::SCALE).is_some());
    clica(&mut m0, ids::FORMATS[1]);
    assert!(m0.rect(ids::LOOPS).is_some());
    escreve(&mut m0, ids::LOOPS, "4");
    m0.assenta_em(JANELA);
    assert_eq!(export_as(&mut m0).repeat, 4);
    clica(&mut m0, ids::FORMATS[4]);
    assert!(m0.rect(ids::SCALE).is_none());
    assert!(m0.rect(ids::LAYERS).is_none());
}

#[test]
fn a_folha_cada_caixa_e_cada_numero_chegam_ao_verbo() {
    let mut m0 = mesa(Tela::Folha);
    for (id, ler) in [
        (
            ids::MERGE,
            (|x| x.opcoes.merge_duplicates) as fn(&ph2d_pixel_io::exporta::ExportFolha) -> bool,
        ),
        (ids::IGNORE_EMPTY, |x| x.opcoes.ignore_empty),
        (ids::POW2, |x| x.opcoes.potencia_de_2),
        (ids::SPLIT_LAYERS, |x| x.opcoes.split_layers),
        (ids::SPLIT_TAGS, |x| x.opcoes.split_tags),
        (ids::TRIM_CELS, |x| x.opcoes.trim_cels),
        (ids::TRIM_SPRITE, |x| x.opcoes.trim_sprite),
        (ids::JSON_LAYERS, |x| x.json.is_some_and(|j| j.layers)),
        (ids::JSON_SLICES, |x| x.json.is_some_and(|j| j.slices)),
    ] {
        assert!(!ler(&folha(&mut m0)), "nasce desligada");
        clica(&mut m0, id);
        assert!(ler(&folha(&mut m0)), "o clique não ligou");
    }
    assert!(
        folha(&mut m0).json.is_some_and(|j| j.tags),
        "tags nasce ligada"
    );
    clica(&mut m0, ids::JSON_TAGS);
    assert!(folha(&mut m0).json.is_some_and(|j| !j.tags));
    clica(&mut m0, ids::JSON_ARRAY);
    assert!(folha(&mut m0).json.is_some_and(|j| j.array));
    clica(&mut m0, ids::JSON_HASH);
    assert!(folha(&mut m0).json.is_some_and(|j| !j.array));
    for (id, ler) in [
        (
            ids::BORDER,
            (|x| x.opcoes.border) as fn(&ph2d_pixel_io::exporta::ExportFolha) -> u32,
        ),
        (ids::SHAPE, |x| x.opcoes.shape),
        (ids::INNER, |x| x.opcoes.inner),
        (ids::EXTRUDE, |x| x.opcoes.extrude),
    ] {
        escreve(&mut m0, id, "5");
        m0.assenta_em(JANELA);
        assert_eq!(ler(&folha(&mut m0)), 5);
    }
    clica(&mut m0, ids::JSON);
    assert_eq!(folha(&mut m0).json, None);
    assert!(
        m0.rect(ids::JSON_ARRAY).is_none(),
        "o JSON desligado esconde as opções"
    );
}

/// Cada tipo oferece só os limites do diálogo do Aseprite, e trocar de tipo larga o que ele não aceita.
#[test]
fn o_limite_segue_o_tipo_da_folha() {
    let mut m0 = mesa(Tela::Folha);
    assert_eq!(folha(&mut m0).opcoes.tipo, Tipo::Rows);
    assert!(m0.rect(ids::LIMIT_N).is_none(), "sem limite, sem número");
    escolhe(&mut m0, ids::LIMIT, ids::LIMIT_COLUMNS);
    assert_eq!(folha(&mut m0).opcoes.limite, Limite::Colunas(8));
    escreve(&mut m0, ids::LIMIT_N, "3");
    m0.assenta_em(JANELA);
    assert_eq!(folha(&mut m0).opcoes.limite, Limite::Colunas(3));
    escolhe(&mut m0, ids::SHEET_TYPE, ids::SHEET_TYPES[3]);
    let x = folha(&mut m0);
    assert_eq!(
        (x.opcoes.tipo, x.opcoes.limite),
        (Tipo::Columns, Limite::Nenhum)
    );
    escolhe(&mut m0, ids::SHEET_TYPE, ids::SHEET_TYPES[0]);
    assert!(m0.rect(ids::LIMIT).is_none(), "a tira não tem limite");
}

/// A janela da folha pede as opções a cada quadro: é assim que a prévia fica ao vivo.
#[test]
fn a_folha_aberta_pede_a_previa_e_fechada_nao() {
    let mut m0 = mesa(Tela::Folha);
    assert_eq!(canal::folha_pedida().map(|o| o.border), Some(0));
    escreve(&mut m0, ids::BORDER, "2");
    m0.assenta_em(JANELA);
    assert_eq!(canal::folha_pedida().map(|o| o.border), Some(2));
    let _ = m0.clica(ids::CANCEL);
    assert!(
        !m0.host.panel_visible(PixelExportPanel::ID),
        "o Cancel não fechou"
    );
}

#[test]
fn o_import_devolve_a_grelha_e_recusa_a_que_nao_cabe() {
    let mut m0 = mesa(Tela::Import);
    for (id, v) in [
        (ids::GRID_X, "1"),
        (ids::GRID_Y, "2"),
        (ids::GRID_W, "4"),
        (ids::GRID_H, "5"),
        (ids::GAP_X, "1"),
        (ids::GAP_Y, "1"),
    ] {
        escreve(&mut m0, id, v);
    }
    m0.assenta_em(JANELA);
    clica(&mut m0, ids::BY_COLUMNS);
    let Pedido::Importar(g) = verbo(&mut m0) else {
        panic!("não é o Import");
    };
    assert_eq!(
        (g.x, g.y, g.w, g.h, g.gap_x, g.gap_y, g.por_colunas),
        (1, 2, 4, 5, 1, 1, true)
    );
    escreve(&mut m0, ids::GRID_W, "100");
    m0.assenta_em(JANELA);
    clica(&mut m0, ids::OK);
    assert_eq!(
        canal::take_pedido(),
        None,
        "não cabe célula nenhuma: nada a importar"
    );
}

/// A imagem nova traz a grelha sugerida, uma vez: mexer nos campos depois não a perde.
#[test]
fn a_sugestao_entra_uma_vez() {
    let mut m0 = mesa(Tela::Import);
    let g = ph2d_pixel_io::abrir::Grelha {
        x: 0,
        y: 0,
        w: 8,
        h: 8,
        gap_x: 0,
        gap_y: 0,
        por_colunas: false,
    };
    canal::publish_imagem(Some(Imagem {
        rgba: Arc::new(vec![255; 64 * 32 * 4]),
        w: 64,
        h: 32,
        seq: 2,
        lados_reais: (64, 32),
        entradas: 0,
        sugestao: Some(g),
    }));
    m0.assenta_em(JANELA);
    assert!(matches!(verbo(&mut m0), Pedido::Importar(x) if x == g));
    escreve(&mut m0, ids::GRID_W, "4");
    m0.assenta_em(JANELA);
    assert!(matches!(verbo(&mut m0), Pedido::Importar(x) if x.w == 4));
}

/// O Reset volta à fábrica de cada janela.
#[test]
fn o_reset_volta_a_fabrica() {
    let mut m0 = mesa(Tela::ExportAs);
    clica(&mut m0, ids::FORMATS[1]);
    clica(&mut m0, ids::EXPORT_RESET);
    assert_eq!(export_as(&mut m0).formato, Formato::Png);
    let mut m1 = mesa(Tela::Folha);
    clica(&mut m1, ids::POW2);
    clica(&mut m1, ids::EXPORT_RESET);
    assert!(!folha(&mut m1).opcoes.potencia_de_2);
}

/// O título arrasta-se (a alça é registada) e a faixa dos verbos fica fora dos cartões.
#[test]
fn os_verbos_ficam_na_faixa_e_o_titulo_arrasta() {
    let m0 = mesa(Tela::Folha);
    m0.prova_a_faixa(
        PixelExportPanel::NODE_ID,
        &[ids::SEC_PREVIEW, ids::SEC_LAYOUT],
        &[ids::EXPORT_RESET, ids::CANCEL, ids::OK],
    );
    assert!(m0.rect(ids::EXPORT_DRAG).is_some(), "sem alça");
}
