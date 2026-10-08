//! **As figuras dos tutoriais: PRODUZIR × ESCREVER, e o disco comparado com o produtor.**
//!
//! Cada gerador `figuras()` devolve `(ficheiro, conteúdo)` com as asserções de história lá
//! dentro; o teste `#[ignore]` com o nome antigo só chama [`escreve`]. Os gates abaixo (não
//! ignorados) apanham o disco que envelheceu em silêncio — doc 123 §0 item 3.

use std::collections::BTreeMap;
use std::path::Path;

/// O que um gerador produz: `(nome do ficheiro em [`DIR`], conteúdo)`.
pub(super) type Figuras = Vec<(String, String)>;
/// Um produtor de figuras.
#[cfg(feature = "panel-motion-graph")]
type Produtor = fn() -> Figuras;

/// A pasta das figuras — relativa ao MANIFESTO, nunca à cwd do processo.
pub(super) const DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/Motion Nodes/tutoriais/fig"
);

/// A pasta dos fontes HTML que incluem as tabelas geradas.
const SRC_HTML: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/Motion Nodes/tutoriais/src"
);

/// ⭐ O registo: o NOME do teste que escreve (o que se corre à mão) e o produtor.
///
/// ⚠️ Cinco dos nove só existem com `panel-motion-graph` (ligada por omissão), e os gates com eles.
#[cfg(feature = "panel-motion-graph")]
pub(super) const GERADORES: &[(&str, Produtor)] = &[
    ("dump_arranjo_figures", super::arranjo_figures::figuras),
    (
        "dump_animadores_figures",
        super::animadores_figures::figuras,
    ),
    (
        "dump_deformadores_figures",
        super::deformadores_figures::figuras,
    ),
    ("write_the_field_figures", super::campos_figures::figuras),
    ("write_the_sim_figures", super::sim_figures::figuras),
    ("write_the_value_figures", super::valor_figures::figuras),
    (
        "write_the_appearance_figures",
        super::aparencia_figures::figuras,
    ),
    ("write_the_source_figures", super::fontes_figures::figuras),
    ("write_the_rig_figures", super::rig_figures::figuras),
];

/// Escreve no disco o que um produtor devolveu — só depois de as asserções dele passarem.
pub(super) fn escreve(figs: &[(String, String)]) {
    let dir = Path::new(DIR);
    std::fs::create_dir_all(dir).expect("a pasta das figuras");
    for (nome, conteudo) in figs {
        std::fs::write(dir.join(nome), conteudo).expect("escrever a figura");
        eprintln!("  escrito │ {nome}");
    }
}

#[cfg(feature = "panel-motion-graph")]
/// A 1.ª linha em que dois textos diferem, para a mensagem do gate.
fn primeira_diferenca(disco: &str, gerado: &str) -> String {
    let corta = |s: &str| s.chars().take(100).collect::<String>();
    let (mut a, mut b) = (disco.lines(), gerado.lines());
    for n in 1.. {
        match (a.next(), b.next()) {
            (Some(x), Some(y)) if x == y => {}
            (None, None) => return "só no fim de linha".to_string(),
            (x, y) => {
                return format!(
                    "linha {n}: disco «{}» · gerador «{}»",
                    corta(x.unwrap_or("<fim>")),
                    corta(y.unwrap_or("<fim>"))
                );
            }
        }
    }
    unreachable!()
}

/// ⛔⛔ **O disco é o que o gerador produz HOJE**, byte a byte. Vermelho ⇒ corra o escritor nomeado.
#[cfg(feature = "panel-motion-graph")]
#[test]
fn as_figuras_dos_tutoriais_sao_as_que_os_geradores_produzem() {
    let mut maus: Vec<String> = Vec::new();
    let mut dono: BTreeMap<String, &str> = BTreeMap::new();
    for (escritor, gera) in GERADORES {
        for (nome, gerado) in gera() {
            if let Some(outro) = dono.insert(nome.clone(), escritor) {
                maus.push(format!(
                    "`{nome}` sai de DOIS geradores: `{outro}` e `{escritor}`"
                ));
            }
            match std::fs::read_to_string(Path::new(DIR).join(&nome)) {
                Err(_) => maus.push(format!("`{nome}` ausente do disco — corra `{escritor}`")),
                Ok(disco) if disco != gerado => maus.push(format!(
                    "`{nome}` envelheceu ({}) — corra `{escritor}`",
                    primeira_diferenca(&disco, &gerado)
                )),
                Ok(_) => {}
            }
        }
    }
    assert!(
        maus.is_empty(),
        "{} figura(s) do tutorial no disco nao sao o que o gerador produz hoje:\n  {}\n\
         corra: cargo test -p ph2d-app-motion --lib --release <escritor> -- --ignored",
        maus.len(),
        maus.join("\n  ")
    );
}

#[cfg(feature = "panel-motion-graph")]
/// O nome definido por uma linha `[pub(..)] fn <nome>`, se for uma definição.
fn definicao(linha: &str) -> Option<&str> {
    let l = linha.trim_start();
    let l = l
        .strip_prefix("pub(super) ")
        .or_else(|| l.strip_prefix("pub(crate) "))
        .or_else(|| l.strip_prefix("pub "))
        .unwrap_or(l);
    let resto = l.strip_prefix("fn ")?;
    let fim = resto
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .unwrap_or(resto.len());
    Some(&resto[..fim])
}

/// ⛔ Um gerador fora do registo escapa ao gate do disco — censo TEXTUAL sobre `src/`.
#[cfg(feature = "panel-motion-graph")]
#[test]
fn todo_gerador_de_figuras_esta_no_registo() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut achados: BTreeMap<String, String> = BTreeMap::new();
    for entrada in std::fs::read_dir(&src).expect("src") {
        let p = entrada.expect("entrada").path();
        if p.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let texto = std::fs::read_to_string(&p).expect("ler o .rs");
        for nome in texto.lines().filter_map(definicao) {
            let e_gerador = nome.starts_with("write_the_")
                || (nome.starts_with("dump_") && nome.ends_with("_figures"));
            if e_gerador {
                let ficheiro = p.file_name().map(|f| f.to_string_lossy().into_owned());
                achados.insert(nome.to_string(), ficheiro.unwrap_or_default());
            }
        }
    }
    // Controlo: um censo que não acha nada passa sempre.
    assert!(
        achados.len() >= 9,
        "o censo achou so' {} gerador(es) — a regua parou de ler",
        achados.len()
    );
    let registados: Vec<&str> = GERADORES.iter().map(|(n, _)| *n).collect();
    let fora: Vec<String> = achados
        .iter()
        .filter(|(n, _)| !registados.contains(&n.as_str()))
        .map(|(n, f)| format!("{n} ({f})"))
        .collect();
    let orfaos: Vec<&&str> = registados
        .iter()
        .filter(|n| !achados.contains_key(**n))
        .collect();
    assert!(
        fora.is_empty() && orfaos.is_empty(),
        "geradores fora do registo GERADORES: {fora:?}; nomes no registo sem `fn`: {orfaos:?}"
    );
}

/// ⛔ Tabela gerada que nenhum tutorial inclui não chega ao PDF; cópia COLADA envelhece.
#[test]
fn toda_tabela_gerada_e_incluida_e_nenhuma_e_colada() {
    let mut incluidas: Vec<String> = Vec::new();
    let mut maus: Vec<String> = Vec::new();
    for entrada in std::fs::read_dir(SRC_HTML).expect("src dos tutoriais") {
        let p = entrada.expect("entrada").path();
        if p.extension().and_then(|e| e.to_str()) != Some("html") {
            continue;
        }
        let nome = p.file_name().map(|f| f.to_string_lossy().into_owned());
        let nome = nome.unwrap_or_default();
        let texto = std::fs::read_to_string(&p).expect("ler o html");
        if texto.contains("GERADO pelo gerador") {
            maus.push(format!(
                "{nome} tem uma tabela COLADA — troque por um include"
            ));
        }
        for pedaco in texto.split("<!--#include ../fig/").skip(1) {
            let Some(fim) = pedaco.find("-->") else {
                continue;
            };
            let alvo = pedaco[..fim].trim().to_string();
            if !Path::new(DIR).join(&alvo).is_file() {
                maus.push(format!("{nome} inclui `{alvo}`, que nao existe"));
            }
            incluidas.push(alvo);
        }
    }
    assert!(
        incluidas.len() >= 3,
        "a varredura achou so' {} include(s) — a regua parou de ler",
        incluidas.len()
    );
    // ⚠️ Lê as tabelas do DISCO: o gate do disco garante que todo `params*.html` gerado lá está.
    for entrada in std::fs::read_dir(DIR).expect("a pasta das figuras") {
        let f = entrada
            .expect("entrada")
            .file_name()
            .to_string_lossy()
            .into_owned();
        if f.starts_with("params") && f.ends_with(".html") && !incluidas.contains(&f) {
            maus.push(format!("`{f}` nao e' incluida por nenhum tutorial"));
        }
    }
    assert!(
        maus.is_empty(),
        "tabelas dos tutoriais:\n  {}",
        maus.join("\n  ")
    );
}
