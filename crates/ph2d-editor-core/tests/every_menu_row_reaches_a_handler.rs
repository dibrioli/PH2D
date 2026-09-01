//! ⭐⭐⭐ **A SEGUNDA SONDA DO «O VALOR CHEGA A ALGUÉM?»** — agora sobre as **110 linhas de menu**.
//!
//! A irmã dela é a `the_rail_names_a_consumer_for_every_chip`, que nasceu de um controlo morto que
//! o Enio apanhou numa foto. Esta faz a mesma pergunta à outra superfície — e a resposta, medida em
//! 2026-09-01, é **boa**: das `110` linhas que as tabelas do
//! [`crate::screens::hero::menu_rows`] servem, **todas** têm sítio de despacho.
//!
//! ⚠️ **`110` linhas, não `118` símbolos:** o fonte menciona `118` nomes `ids::*`, e **oito** deles
//! são as TABELAS de menu da timeline (ver [`ROW_TABLES`]) — uma tabela não tem handler, os itens
//! dentro dela é que têm. *Contar os dois juntos daria uma população que a sonda não sabe julgar.*
//!
//! ⚠️ **Medir e confirmar É o resultado.** O trilho tinha quatro mortos; os menus têm zero — e sem
//! esta sonda as duas frases teriam o mesmo peso, que é nenhum.
//!
//! # ⭐ Por que ela não tem tabela escrita à mão, ao contrário da irmã
//!
//! No trilho o veredito é **por chip**: cada um escreve uma coisa diferente e o consumidor tem de
//! ser nomeado. Aqui a pergunta é mais simples e por isso mecanizável: uma linha de menu é uma
//! **acção**, e o que ela precisa é de **um sítio que reconheça o id fora da tabela que a pinta**.
//!
//! ⇒ **os dois lados são derivados**: a população sai do fonte do `menu_rows.rs`, e o veredito sai
//! de uma varredura da árvore. ⛔ Não há lista a envelhecer — que é o que uma tabela de 118 linhas
//! seria de qualquer maneira.
//!
//! # ⛔⛔ E a varredura corre SEM `head`
//!
//! *Um `grep` truncado devolve «zero consumidores» com a mesma cara de um `grep` completo.* Foi um
//! `head -20` que fez a entrega 36 declarar morto um chip com dois leitores.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Os OITO nomes das TABELAS de menu da timeline — eles aparecem como `ids::X` no `menu_rows.rs`
/// tal como uma linha aparece, e **não são linhas**.
///
/// ⚠️ Sem esta exclusão a sonda acusa-os de mortos: uma tabela não tem handler, os itens **dentro**
/// dela é que têm. ⛔ E não é uma folga — cada uma tem gate próprio de anti-item-morto na
/// `ph2d-panel-timeline` (`seam.rs`, `marker_menu_seam.rs`, `extrapolation_seam.rs`), que percorre
/// a tabela e exige um braço de `event.rs` por linha.
const ROW_TABLES: &[&str] = &[
    "TIMELINE_TRACK_MENU",
    "TIMELINE_AXIS_TRACK_MENU",
    "TIMELINE_PATH_TRACK_MENU",
    "TIMELINE_TIMEREMAP_TRACK_MENU",
    "TIMELINE_EXTRAP_MENU",
    "TIMELINE_MARKER_MENU",
    "TIMELINE_LANE_MENU",
    "TIMELINE_STRIP_MENU",
];

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

const MENU_ROWS: &str = "crates/ph2d-editor-core/src/screens/hero/menu_rows.rs";

/// **A população** — todo `ids::NOME` que a tabela de linhas menciona, menos as tabelas.
fn menu_row_ids(root: &Path) -> BTreeSet<String> {
    let src = std::fs::read_to_string(root.join(MENU_ROWS)).expect("menu_rows.rs");
    let mut out = BTreeSet::new();
    let mut rest = src.as_str();
    while let Some(i) = rest.find("ids::") {
        rest = &rest[i + 5..];
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '_')
            .collect();
        if name.len() > 3 && !ROW_TABLES.contains(&name.as_str()) {
            out.insert(name);
        }
    }
    out
}

/// Todo `.rs` da árvore que não é declaração de id, nem a tabela que pinta, nem **registo**, nem
/// um teste.
///
/// ⚠️ **As três exclusões foram todas cobradas por MUTAÇÃO, e a do REGISTO custou a segunda
/// tentativa deste gate.** Apagar o braço inteiro do *Save As…* do `chrome::io_menu` deixou-o
/// **verde** — porque o id continuava a aparecer no `pre_populate.rs`, que **regista** o widget e
/// não o despacha. *Um id registado é um id que existe, não um id que faz.*
///
/// | excluído | porque um sítio ali não é um handler |
/// |---|---|
/// | `/ids/` | é a declaração do nome |
/// | `menu_rows.rs` | é a tabela que o PINTA |
/// | `pre_populate*.rs` · `populate.rs` | é o REGISTO — cunha o widget no store |
/// | `tests/` · `*_tests.rs` | prova que alguém o TESTA, não que alguém o despacha |
fn dispatch_sources(root: &Path) -> Vec<(PathBuf, String)> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for e in rd.flatten() {
            let p = e.path();
            let name = p
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            if p.is_dir() {
                if name != "target" && !name.starts_with('.') {
                    walk(&p, out);
                }
            } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
                out.push(p);
            }
        }
    }
    let mut files = Vec::new();
    for top in ["crates", "shells"] {
        walk(&root.join(top), &mut files);
    }
    files
        .into_iter()
        .filter(|p| {
            let s = p.to_string_lossy();
            !s.contains("/ids/")
                && !s.ends_with("menu_rows.rs")
                && !s.contains("pre_populate")
                && !s.ends_with("populate.rs")
                && !s.contains("/tests/")
                && !s.contains("_tests.rs")
                && !s.contains("/test")
        })
        .filter_map(|p| std::fs::read_to_string(&p).ok().map(|s| (p, s)))
        .collect()
}

/// ⭐⭐⭐ **Toda linha de menu é reconhecida em algum lado que não é a tabela que a pinta.**
#[test]
fn every_menu_row_reaches_a_handler() {
    let root = workspace_root();
    let ids = menu_row_ids(&root);
    let sources = dispatch_sources(&root);

    let mut where_: BTreeMap<&String, usize> = BTreeMap::new();
    for id in &ids {
        let needle = format!("ids::{id}");
        // ⚠️ A fronteira à direita importa: `CTX_MENU_SAVE` casaria dentro de `CTX_MENU_SAVE_AS`, e
        // a linha *Save As…* passaria a cobrir a *Save* — um falso verde exactamente na família de
        // ids que mais partilha prefixo.
        let n = sources
            .iter()
            .filter(|(_, src)| {
                src.match_indices(&needle).any(|(i, _)| {
                    !src[i + needle.len()..].starts_with(|c: char| {
                        c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'
                    })
                })
            })
            .count();
        where_.insert(id, n);
    }

    let orphans: Vec<&&String> = where_
        .iter()
        .filter(|(_, n)| **n == 0)
        .map(|(k, _)| k)
        .collect();
    println!(
        "linhas de menu: {} ; sem sitio de despacho: {}",
        ids.len(),
        orphans.len()
    );
    assert!(
        orphans.is_empty(),
        "estas linhas de menu sao PINTADAS e nenhum sitio fora da tabela reconhece o id:\n  {orphans:?}\n\n\
         ⚠️ Uma linha de menu que ninguem despacha e' um alvo que consome o clique e nao faz nada — \
         a especie mais cara de controlo morto (`CLAUDE.md` §5.0).\n\
         ⛔ Se a ausencia for deliberada (uma TABELA de linhas, nao uma linha), acrescente o nome a \
         `ROW_TABLES` com o gate que a cobre nomeado ao lado."
    );
}

/// ⚠️ **O controle positivo da população** — sem ele um `menu_rows.rs` que mudasse de nome ou de
/// forma deixaria o gate acima **verde por vácuo**.
///
/// ⭐ O piso é a medição de 2026-09-01 (`110` linhas). ⛔ Ele **desce** quando um menu encolhe de
/// propósito: baixe-o com o número novo ao lado, nunca apague a asserção.
#[test]
fn the_population_is_the_whole_menu_surface() {
    let ids = menu_row_ids(&workspace_root());
    assert!(
        ids.len() >= 100,
        "o `menu_rows.rs` deu {} linhas — em 2026-09-01 eram 110, entao o parser deixou de ler a \
         tabela e o gate irmao ficaria verde sobre nada",
        ids.len()
    );
}

/// ⚠️ **E o controle positivo da VARREDURA** — a metade que prova que ela sabe dizer «não».
///
/// ⛔ Sem ele, um `dispatch_sources` que devolvesse a lista errada (ou vazia) faria o gate principal
/// acusar toda a gente **ou** ninguém, conforme o sentido do engano. Um id inventado tem de dar
/// zero, e um que existe tem de dar mais do que zero.
#[test]
fn the_scan_can_still_say_no() {
    let root = workspace_root();
    let sources = dispatch_sources(&root);
    let hits = |needle: &str| sources.iter().filter(|(_, s)| s.contains(needle)).count();
    assert_eq!(
        hits("ids::CTX_MENU_UM_ID_QUE_NAO_EXISTE"),
        0,
        "a varredura acha um id inventado — ela deixou de medir o que diz medir"
    );
    assert!(
        hits("ids::CTX_MENU_SAVE") > 0,
        "a varredura nao acha um id que TEM despacho — a lista de fontes esta' vazia ou errada"
    );
}
