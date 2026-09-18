//! ⭐⭐⭐ **AS CEGUEIRAS DECLARADAS DESTA RÉGUA TÊM NOME, E A LISTA DELAS É CONSULTÁVEL.**
//!
//! ⛔⛔ **Este ficheiro nasce de duas medições, e as duas doeram:** as LETRAS SOZINHAS estiveram no
//! doc-comment do [`ph2d_label_census::is_language`] como exclusão declarada enquanto três painéis
//! pintavam `S`/`R`/`M`/`X`/`Y`/`W`/`H` cruas com o censo VERDE (2026-09-18), e o TOKEN NU o mesmo
//! com o `repeats`/`once` do Inspector (2026-09-13). *Uma regra sem instrumento é uma nota que
//! envelhece* — e enquanto a razão de cada recusa vivia dentro de um `bool`, ninguém conseguia
//! LISTAR a população onde um rótulo se esconde.
//!
//! ⛔ **Alargar a régua NÃO é a cura**, e é por isso que estes casos continuam a ser recusados: este
//! repo tem centenas de params chamados `"x"`, `"n"`, `"b"` e de ids em `snake_case`. A cura é a
//! CERCA mudar de sítio (um `TextKey` no pintor) e esta lista existir.

use ph2d_label_census::{Cegueira, blind_literals_in, cegueira, is_language};

/// **Cada recusa diz QUAL foi**, e o `is_language` é o acessório derivado dela.
#[test]
fn cada_recusa_da_regua_diz_qual_cerca_a_produziu() {
    let casos = [
        // (texto, a cerca que o recusa)
        ("X", Some(Cegueira::SemDuasLetras)),
        ("W", Some(Cegueira::SemDuasLetras)),
        ("%", Some(Cegueira::SemDuasLetras)),
        ("m/s", Some(Cegueira::SemDuasLetras)),
        ("repeats", Some(Cegueira::TokenNu)),
        ("snake_case", Some(Cegueira::TokenNu)),
        ("9-Slice", Some(Cegueira::TokenNu)),
        ("EQ", Some(Cegueira::TokenNu)),
        ("chrome.fill.title", Some(Cegueira::ParecemChave)),
        // ⚠️ **A ORDEM das cercas é observável, e esta linha existe por eu a ter errado:**
        // `assets/brush.png` é recusado pelo CAMINHO (a barra vem primeiro) e nunca pela
        // extensão — para ver a última é preciso um nome sem barra e com maiúscula (senão o
        // `key_like` apanha-o antes). *Uma lista de razões só é honesta se a ordem delas estiver
        // medida.*
        ("assets/brush.png", Some(Cegueira::ParecemCaminho)),
        ("Brush.png", Some(Cegueira::NomeDeFicheiro)),
        ("src/main", Some(Cegueira::ParecemCaminho)),
        // … e o que ela ACEITA como língua.
        ("Hello", None),
        ("Black & White", None),
        ("WALK", None),
        ("{n} entities", None),
        ("Speed (m/s)", None),
    ];
    for (texto, esperado) in casos {
        assert_eq!(
            cegueira(texto),
            esperado,
            "a régua classificou {texto:?} de outra maneira"
        );
        // ⭐ O acessório NUNCA discorda da lei — é o que impede uma segunda cópia do critério.
        assert_eq!(
            is_language(texto),
            esperado.is_none(),
            "o `is_language` e a `cegueira` discordam sobre {texto:?}"
        );
    }
}

/// ⭐⭐ **A porta do complemento devolve o que a régua deita fora — e SÓ as duas cegueiras que já
/// esconderam um rótulo.**
///
/// ⚠️ A metade NEGATIVA é metade do valor: uma chave e um caminho ficam de FORA, senão a lista de
/// triagem enche-se de coisas que nunca podem ser um rótulo e ninguém a lê.
#[test]
fn o_complemento_lista_a_letra_e_o_token_e_deixa_a_chave_de_fora() {
    let fonte = r#"
        fn pinta(c: &mut C) {
            c.label("W");
            c.label("repeats");
            c.label("Stretch");
            c.label("chrome.fill.title");
            c.label("assets/brush.png");
        }
    "#;
    let achados = blind_literals_in("pinta.rs", fonte);
    let textos: Vec<(&str, Cegueira)> =
        achados.iter().map(|(l, c)| (l.text.as_str(), *c)).collect();
    assert_eq!(
        textos,
        vec![
            ("W", Cegueira::SemDuasLetras),
            ("repeats", Cegueira::TokenNu),
        ],
        "o complemento tem de trazer a letra e o token nu, e mais nada"
    );
    // ⛔⛔ **E o literal VAZIO não entra**: `Dropdown::new(id, "", …)` é o dropdown SEM rótulo, e
    //    ele era **um terço** da lista do Inspector (200 → 126). *Uma palavra não se esconde no
    //    vazio, e uma lista de triagem com um terço de ruído é uma lista que ninguém lê.*
    let vazios = blind_literals_in("v.rs", r#"fn p(c: &mut C) { c.dd("", 1); c.dd("  ", 2); }"#);
    assert!(
        vazios.is_empty(),
        "o complemento trouxe literais vazios: {vazios:?}"
    );

    // ⛔ CONTROLO: o que a régua ACEITA continua a sair pela porta de sempre, e nunca por esta.
    let lingua = ph2d_label_census::language_literals_in("pinta.rs", fonte);
    assert!(
        lingua.iter().any(|l| l.text == "Stretch"),
        "a régua de língua continua a ver a palavra — as duas portas partilham o encadeamento"
    );
    assert!(
        lingua.iter().all(|l| l.text != "W"),
        "e a letra continua fora dela: o complemento não alargou a régua"
    );
}
