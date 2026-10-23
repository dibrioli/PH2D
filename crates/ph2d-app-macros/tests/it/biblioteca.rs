//! W18 (`docs/Macros/16_W18_plano.md` §2.6): a biblioteca do utilizador é uma pasta — guardar, listar,
//! renomear, apagar, importar e exportar sem nada fora do disco.

use ph2d_app_macros::biblioteca::{self, Biblioteca, Erro};
use ph2d_macro::ficheiro::{EXTENSAO, Ficheiro};
use ph2d_macro::{MacroDoc, RefRect};
use std::path::PathBuf;

fn pasta(nome: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!(
        "ph2d-macros-{nome}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&p);
    p
}

fn macro_(modo: u8) -> Ficheiro {
    Ficheiro {
        doc: MacroDoc::new(RefRect { w: 96.0, h: 64.0 }),
        modo,
        nome: String::new(),
    }
}

#[test]
fn a_biblioteca_guarda_lista_renomeia_apaga() {
    let raiz = pasta("crud");
    assert!(Biblioteca::carregar(&raiz).entradas.is_empty(), "pasta ausente = vazia");
    biblioteca::grava(&raiz, "Céu", &macro_(2)).expect("grava");
    assert_eq!(
        biblioteca::grava(&raiz, "Céu", &macro_(0)),
        Err(Erro::JaExiste("Céu".into()))
    );
    let b = Biblioteca::carregar(&raiz);
    assert_eq!(b.entradas.len(), 1);
    let e = b.procura("Céu").expect("está");
    assert_eq!((e.ficheiro.modo, e.ficheiro.nome.as_str()), (2, "Céu"));
    assert_eq!(b.nome_livre("Céu"), "Céu 2");

    biblioteca::renomeia(&raiz, "Céu", "Mar/azul").expect("renomeia");
    let b = Biblioteca::carregar(&raiz);
    assert_eq!(
        b.entradas.iter().map(|e| e.nome.as_str()).collect::<Vec<_>>(),
        ["Mar_azul"]
    );
    assert_eq!(b.entradas[0].ficheiro.nome, "Mar_azul", "o nome de dentro acompanha");

    biblioteca::apaga(&raiz, "Mar_azul").expect("apaga");
    biblioteca::apaga(&raiz, "Mar_azul").expect("apagar o ausente não é erro");
    assert!(Biblioteca::carregar(&raiz).entradas.is_empty());
    let _ = std::fs::remove_dir_all(&raiz);
}

#[test]
fn exportar_e_importar_noutra_biblioteca_da_a_mesma_macro() {
    let a = pasta("export-a");
    let b = pasta("export-b");
    biblioteca::grava(&a, "Tutorial", &macro_(2)).expect("grava");
    let original = Biblioteca::carregar(&a).entradas.remove(0).ficheiro;
    let fora = a.join("enviada");
    let escrito = biblioteca::exporta(&fora, &original).expect("exporta");
    assert!(escrito.to_str().unwrap().ends_with(EXTENSAO), "a extensão acrescenta-se");

    biblioteca::grava(&b, "Tutorial", &macro_(0)).expect("já há uma com o nome");
    let nome = biblioteca::importa(&b, &escrito).expect("importa");
    assert_eq!(nome, "Tutorial 2", "o nome livre na biblioteca de destino");
    let lida = Biblioteca::carregar(&b).procura(&nome).cloned().expect("está");
    assert_eq!((lida.ficheiro.doc, lida.ficheiro.modo), (original.doc, 2));
    let _ = std::fs::remove_dir_all(&a);
    let _ = std::fs::remove_dir_all(&b);
}

#[test]
fn um_ficheiro_estragado_fica_nos_erros_e_a_importacao_recusa() {
    let raiz = pasta("estragado");
    std::fs::create_dir_all(&raiz).unwrap();
    let mau = raiz.join(format!("Mau{EXTENSAO}"));
    std::fs::write(&mau, b"PH2DMACRO\x09\x00\x00\x00").unwrap();
    let b = Biblioteca::carregar(&raiz);
    assert!(b.entradas.is_empty());
    assert_eq!(b.erros.len(), 1, "nunca em silêncio");
    let outra = pasta("estragado-destino");
    assert!(matches!(
        biblioteca::importa(&outra, &mau),
        Err(Erro::Ficheiro(_))
    ));
    let _ = std::fs::remove_dir_all(&raiz);
}

/// ⛔ Um teste nunca lê a biblioteca do DONO: o `Macros` por omissão não tem pasta (só o do processo
/// do app, [`ph2d_app_macros::com`], resolve a do utilizador); a injectada manda.
#[test]
fn o_macros_por_omissao_nao_le_a_pasta_do_utilizador() {
    let m = ph2d_app_macros::Macros::default();
    assert_eq!(m.pasta_da_biblioteca(), None, "o Macros::default() leu o disco do dono");
    assert_eq!(
        ph2d_app_macros::Macros::do_utilizador().pasta_da_biblioteca(),
        biblioteca::pasta_do_utilizador(),
        "controlo: o do app resolve a do utilizador"
    );
    let mut m = m;
    let p = pasta("injectada");
    m.com_pasta(p.clone());
    assert_eq!(m.pasta_da_biblioteca(), Some(p));
}
