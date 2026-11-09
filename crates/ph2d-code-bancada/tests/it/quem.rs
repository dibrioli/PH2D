//! «Who has what» (itens 6–8): os objectos são os do script pela `StableId`, as colunas são as do
//! TEXTO com o tipo das declarações, e a célula diz o valor próprio ou o do script.

use crate::{OUTRO, SCRIPT, decls, factos, objecto};
use ph2d_code_bancada::names::object_name;
use ph2d_code_bancada::read;
use ph2d_code_viewmodel::{Cell, CellKind, Entry, Holder};
use ph2d_ecs::{Name, StableId, World};
use ph2d_script::ScriptValue;

/// ⭐ Postos fora de ordem (3, 1, 2), com um objecto de OUTRO script e um sem script: a grelha tem
/// só os três do script, pela `StableId`, a cor de cada um é o índice nessa ordem, e o sem nome
/// chama-se como na Hierarchy («Object N»).
#[test]
fn os_objectos_sao_os_do_script_pela_stable_id_e_a_cor_e_o_indice() {
    let mut w = World::new();
    let caixa = objecto(&mut w, 3, Some("Caixa"), SCRIPT, &[]);
    let _outro = objecto(&mut w, 0, Some("De outro script"), OUTRO, &[]);
    let aranha = objecto(&mut w, 1, Some("Aranha"), SCRIPT, &[]);
    w.spawn((StableId(4), Name("Sem script".into())));
    let sem_nome = objecto(&mut w, 2, None, SCRIPT, &[]);

    let who = read(&mut w, SCRIPT, Some(&decls()), &factos());
    let fallback = object_name(&w, sem_nome);
    assert_eq!(fallback, format!("Object {}", sem_nome.index()));
    assert_eq!(
        who.holders,
        [
            Holder {
                bits: aranha.to_bits(),
                name: "Aranha".into(),
                color: 0,
            },
            Holder {
                bits: sem_nome.to_bits(),
                name: fallback,
                color: 1,
            },
            Holder {
                bits: caixa.to_bits(),
                name: "Caixa".into(),
                color: 2,
            },
        ]
    );
    let outro = read(&mut w, OUTRO, None, &factos());
    assert_eq!(outro.holders.len(), 1, "o outro script tem só o seu");
}

/// As colunas são as `ph2d.property` do TEXTO, pela ordem e linha dele; o tipo, o valor do script e
/// as opções vêm das declarações. A que só o texto tem não tem tipo (`kind: None`).
#[test]
fn as_colunas_sao_as_do_texto_com_o_tipo_das_declaracoes() {
    let mut w = World::new();
    objecto(&mut w, 1, Some("A"), SCRIPT, &[]);
    let who = read(&mut w, SCRIPT, Some(&decls()), &factos());
    let resumo: Vec<(&str, usize, Option<CellKind>, &str)> = who
        .columns
        .iter()
        .map(|c| (c.name.as_str(), c.line, c.kind, c.default.as_str()))
        .collect();
    assert_eq!(
        resumo,
        [
            ("speed", 0, Some(CellKind::Number), "220"),
            ("active", 1, Some(CellKind::Bool), "true"),
            ("mode", 2, Some(CellKind::Choice), "walk"),
            ("aim", 3, Some(CellKind::Vec2), "0, 1"),
            ("tint", 4, Some(CellKind::Color), ""),
            ("label", 5, Some(CellKind::Text), "bob"),
            ("novo", 6, None, ""),
        ]
    );
    assert_eq!(who.columns[2].options, ["walk", "run"]);
    assert!(who.columns[5].options.is_empty(), "texto livre: sem opções");
    assert_eq!(who.functions, [Entry("on_update".into(), 7)]);

    let sem_decls = read(&mut w, SCRIPT, None, &factos());
    assert_eq!(
        sem_decls.columns.len(),
        7,
        "sem declarações, as colunas são as do texto"
    );
    assert!(sem_decls.columns.iter().all(|c| c.kind.is_none()));
}

/// O valor próprio diferente do do script está «mudado»; o próprio IGUAL ao do script não; sem
/// valor próprio, a célula mostra o do script. A coluna só do texto fica vazia.
#[test]
fn a_celula_diz_o_valor_e_so_o_proprio_diferente_do_default_esta_mudado() {
    let mut w = World::new();
    objecto(
        &mut w,
        1,
        Some("A"),
        SCRIPT,
        &[
            ("speed", ScriptValue::Number(300.0)),
            ("active", ScriptValue::Bool(true)),
            ("aim", ScriptValue::Vec2([2.5, -1.0])),
        ],
    );
    objecto(&mut w, 2, Some("B"), SCRIPT, &[]);
    let who = read(&mut w, SCRIPT, Some(&decls()), &factos());
    let cel = |h: usize, c: usize| who.cell(h, c).cloned().expect("célula");
    let escrita = |text: &str, changed: bool| Cell {
        text: text.into(),
        changed,
        swatch: None,
    };
    assert_eq!(who.cells.len(), 2 * 7);
    assert_eq!(cel(0, 0), escrita("300", true));
    assert_eq!(
        cel(0, 1),
        escrita("true", false),
        "igual ao do script: não mudado"
    );
    assert_eq!(cel(0, 3), escrita("2.5, -1", true));
    assert_eq!(
        cel(1, 0),
        escrita("220", false),
        "sem valor próprio: o do script"
    );
    assert_eq!(cel(1, 2), escrita("walk", false));
    assert_eq!(
        cel(1, 4),
        Cell {
            text: String::new(),
            changed: false,
            swatch: Some([1.0, 0.0, 0.0, 1.0]),
        },
        "a cor é a amostra, sem texto"
    );
    assert_eq!(
        cel(0, 6),
        escrita("", false),
        "a coluna só do texto fica vazia"
    );
    assert!(who.cell(0, 7).is_none() && who.cell(2, 0).is_none());
}
