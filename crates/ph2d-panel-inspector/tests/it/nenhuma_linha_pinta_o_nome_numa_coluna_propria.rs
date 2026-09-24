//! ⭐⭐⭐ **NENHUMA LINHA DE PROPRIEDADE PINTA O NOME À MÃO NUMA COLUNA PRÓPRIA.**
//!
//! ⛔⛔ **Report do dono, 2026-09-23, foto do Grid Snap com duas setas:** *«painel grid fora do
//! padrão»* — o nome À ESQUERDA, pintado por `paint_text`, com o valor a arrancar numa coluna que a
//! secção calculava sozinha. A mesma forma vivia em TRÊS secções deste Inspector (a leitura ao
//! vivo do jogador, os dois corpos de uma junta e as três linhas da roldana), cada uma com uma
//! coluna `N alturas de letra` e um tecto em fracção da linha, e o comentário de uma delas dizia
//! *«a mesma coluna de rótulo das rows»* — não era.
//!
//! ⚠️ **Porque é TEXTUAL e não geométrico:** a régua do alinhamento (`onde_comeca_o_valor`, no
//! `ph2d-panel-registry-init`) só vê as linhas que PASSAM pela porta — é a porta que as regista.
//! Uma linha pintada à mão é invisível a ela por construção, que é exactamente o defeito. ⇒ este
//! censo procura o IDIOMA: um `paint_text` cuja largura ou `x` é uma coluna de nome da secção.
//!
//! ⏳ **O limite, nomeado:** uma coluna com OUTRO nome de variável escapa-lhe — é a lição que o
//! `no_row_paints_its_name_above_its_control` já pagou. A agulha cobre os nomes que a varredura de
//! 2026-09-23 achou nas `70` secções.

use std::fs;
use std::path::PathBuf;

/// Os nomes de variável com que uma secção desta casa escreveu a coluna do nome à mão.
const AGULHAS: &[&str] = &[
    "label_w",
    "label_col",
    "label_col_w",
    "label_x",
    "nome_w",
    "name_col",
    "coluna_dos_nomes",
];

/// ⏳ **Os que FICAM, com a razão — e só ENCOLHE.**
///
/// ⛔ Não acrescente uma linha de propriedade aqui: ela nasce por `rows::property_label_row`, e o
/// valor pinta-se na `linha.control`.
const NAO_SAO_LINHAS_DE_PROPRIEDADE: &[(&str, &str)] = &[(
    "timers.rs",
    "uma LISTA de temporizadores — cada linha é o NOME do temporizador e o resumo dele, não um \
     par nome/valor de um campo; o `nome_w` é a largura do nome dele",
)];

fn sections_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/sections")
}

/// Os blocos `paint_text( … );` de um ficheiro — do nome da função ao fecho do parêntese dela.
fn chamadas_de_paint_text(src: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut resto = src;
    while let Some(i) = resto.find("paint_text(") {
        // ⚠️ `paint_text(` também casa dentro de `ph2d_editor_core::paint::paint_text(`, que é a
        //    mesma chamada; o que NÃO pode casar é a definição de uma função com esse sufixo.
        let depois = &resto[i + "paint_text(".len()..];
        let mut profundidade = 1usize;
        let mut fim = depois.len();
        for (k, c) in depois.char_indices() {
            match c {
                '(' => profundidade += 1,
                ')' => {
                    profundidade -= 1;
                    if profundidade == 0 {
                        fim = k;
                        break;
                    }
                }
                _ => {}
            }
        }
        out.push(&depois[..fim]);
        resto = &depois[fim..];
    }
    out
}

fn menciona_uma_coluna_de_nome(bloco: &str) -> bool {
    AGULHAS.iter().any(|a| {
        bloco.match_indices(a).any(|(i, _)| {
            let antes = bloco[..i].chars().next_back();
            let depois = bloco[i + a.len()..].chars().next();
            let fronteira = |c: Option<char>| c.is_none_or(|c| !(c.is_alphanumeric() || c == '_'));
            fronteira(antes) && fronteira(depois)
        })
    })
}

fn censo() -> (usize, usize, Vec<String>) {
    let (mut ficheiros, mut chamadas) = (0, 0);
    let mut acusados = Vec::new();
    for entry in fs::read_dir(sections_dir()).expect("sections/ existe") {
        let path = entry.expect("entrada legível").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let nome = path
            .file_name()
            .and_then(|n| n.to_str())
            .expect("nome utf-8")
            .to_string();
        if nome.ends_with("_tests.rs") {
            continue;
        }
        ficheiros += 1;
        let src = fs::read_to_string(&path).expect("ficheiro legível");
        for bloco in chamadas_de_paint_text(&src) {
            chamadas += 1;
            if menciona_uma_coluna_de_nome(bloco) {
                acusados.push(nome.clone());
            }
        }
    }
    acusados.sort();
    acusados.dedup();
    (ficheiros, chamadas, acusados)
}

/// **Nenhuma secção pinta um nome numa coluna que ela própria calculou.**
///
/// **Mutação que deve sangrar:** repor o `paint_text(…, x, …, label_w, …)` numa das três secções
/// convertidas em 2026-09-23 — é a foto do dono.
#[test]
fn nenhuma_linha_pinta_o_nome_numa_coluna_propria() {
    let (ficheiros, chamadas, acusados) = censo();
    // ⚠️ **Pisos de população, MEDIDOS em 2026-09-23** (`sections/` tinha `70` ficheiros de produto
    //    e `68` chamadas a `paint_text`). Uma varredura que deixasse de achar o directório ou o
    //    idioma devolveria zero acusados e leria-se como aprovada.
    assert!(
        ficheiros >= 60,
        "a varredura leu {ficheiros} ficheiro(s) em sections/ — deixou de alcançar o directório"
    );
    assert!(
        chamadas >= 50,
        "a varredura achou {chamadas} chamada(s) a `paint_text` — o extractor deixou de as achar"
    );
    let tolerados: Vec<&str> = NAO_SAO_LINHAS_DE_PROPRIEDADE
        .iter()
        .map(|(f, _)| *f)
        .collect();
    let maus: Vec<&String> = acusados
        .iter()
        .filter(|f| !tolerados.contains(&f.as_str()))
        .collect();
    assert!(
        maus.is_empty(),
        "estas secções pintam um nome por `paint_text` numa coluna calculada por elas (o report do \
         dono de 2026-09-23, «fora do padrão»): {maus:?}\n\
         cura: `rows::property_label_row` com uma `Seccao::medida` sobre os nomes da secção, e o \
         valor pintado na `linha.control` — a coluna é a do PAINEL."
    );
}

/// ⚠️ **A metade de OBSOLESCÊNCIA** — uma excepção que já não descreve nada sai da lista.
#[test]
fn cada_excepcao_ainda_pinta_por_uma_coluna_propria() {
    let (_, _, acusados) = censo();
    for (f, porque) in NAO_SAO_LINHAS_DE_PROPRIEDADE {
        assert!(
            acusados.iter().any(|a| a == f),
            "`{f}` está em NAO_SAO_LINHAS_DE_PROPRIEDADE ({porque}) e já não pinta por uma coluna \
             própria — apague a entrada"
        );
    }
}

/// O extractor corta o bloco no parêntese CERTO — com argumentos que têm parênteses dentro.
#[test]
fn o_extractor_fecha_no_parentese_da_chamada() {
    let src = "paint_text(a, b(c), (d - e) * 0.5, label_w, cor);\nlet z = label_w;";
    let blocos = chamadas_de_paint_text(src);
    assert_eq!(blocos, ["a, b(c), (d - e) * 0.5, label_w, cor"]);
    assert!(menciona_uma_coluna_de_nome(blocos[0]));
    // Fronteira de palavra: `label_width` não é `label_w`.
    assert!(!menciona_uma_coluna_de_nome("x, label_width, y"));
}
