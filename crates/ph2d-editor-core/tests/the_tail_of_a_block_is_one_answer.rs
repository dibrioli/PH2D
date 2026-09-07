//! ⭐⭐⭐ **O que fica DEPOIS de um bloco é UMA resposta — o degrau do meio de três.**
//!
//! Esta casa tem três ritmos verticais, e até à wave 19 só os das pontas tinham nome:
//!
//! | pergunta | porta | valor | derivação (Godot Modern, MIT) |
//! |---|---|---|---|
//! | de uma LINHA para a seguinte | `control_gap_px` / `list_row_gap_px` | 4 / 1 | `separation_margin` · `Tree.v_separation` |
//! | de um BLOCO para o seguinte | `control_gap_px` | **6** | `base_margin · 1,5` (13 usos no tema) |
//! | de um CARTÃO DE SECÇÃO para o seguinte | `section_gap_px` | 8 | `Separator separation = base_margin · 2` |
//!
//! ⛔ **Censo de 2026-09-07: 78 sítios respondiam à cauda de um bloco, com QUATRO respostas** —
//! `Sm` (6) em 40 · `Xs` (4) em 29 · `Md` (8) em 8 · `Lg` (12) em 1.
//!
//! ⭐ **A resposta maioritária era a certa, e não por ser maioria:** **6** é o único valor que a
//! escada admite — um bloco separa-se mais que duas linhas do mesmo bloco (4) e menos que duas
//! secções (8). Um bloco que se separasse `8` leria como uma secção; a `4`, como mais uma linha.
//!
//! ⚠️ **A conversão foi UNIFORME e sem julgamento por sítio:** o degrau que cada sítio escreveu é
//! a evidência da intenção dele — `Xs` disse *«sou uma linha»*, `Sm`/`Md`/`Lg` disseram *«sou um
//! bloco»*. ⇒ **só os 9 que discordavam da escada mudam de valor** (8 e 12 → 6, a direcção que o
//! dono pediu cinco vezes); os outros 69 mudam de **dono**, que é o que impede a quinta resposta.
//!
//! ⚠️⚠️ **E o `section_gap_px` não é número novo: ele já era shipado sem nome**, como `pad * 2.0`
//! dentro do `SectionCards::close_at`. *Enquanto uma grandeza não tem nome, ela não é COMPARÁVEL
//! com a vizinha* — e foi isso que deixou 8 sítios responderem à cauda de um bloco com o número da
//! secção, sem que nada pudesse dizer que estavam a responder à pergunta errada.

use std::fs;
use std::path::{Path, PathBuf};

/// Os degraus que alguém poderia somar na cauda de um bloco.
const RUNGS: &[&str] = &["Xxs", "Xs", "Sm", "Md", "Lg"];

/// Os nomes de cursor vertical que um pintor de painel usa.
const CURSORS: &[&str] = &["y", "cur_y", "yy", "new_y"];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().and_then(|n| n.to_str()) != Some("target") {
                walk(&p, out);
            }
        } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

/// As raízes de UI: `ph2d-editor-core`, as crates de painel e a shell.
fn ui_sources() -> Vec<PathBuf> {
    let root = repo_root();
    let mut out = Vec::new();
    walk(&root.join("crates/ph2d-editor-core/src"), &mut out);
    if let Ok(entries) = fs::read_dir(root.join("crates")) {
        for e in entries.flatten() {
            let p = e.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            if name.starts_with("ph2d-panel-") {
                walk(&p.join("src"), &mut out);
            }
        }
    }
    walk(&root.join("shells/desktop/src"), &mut out);
    out.sort();
    out
}

/// A linha é a **cauda** de um pintor — a expressão final `y + … + <degrau>` que devolve onde o
/// próximo bloco começa?
///
/// ⚠️ **A régua é a AUSÊNCIA de `;`**, e ela é o que separa esta pergunta da do passo de linha: um
/// `y += altura + vão;` anda **dentro** de um bloco (a pergunta da wave 8/17), e um `y + … + vão`
/// sem ponto e vírgula é o valor que **sai** dele.
fn is_a_tail(line: &str) -> bool {
    let t = line.trim();
    if t.ends_with(';') || t.starts_with("//") {
        return false;
    }
    let Some(head) = t.split_whitespace().next() else {
        return false;
    };
    CURSORS.contains(&head) && t[head.len()..].trim_start().starts_with('+')
}

fn hand_written_tails() -> Vec<String> {
    let root = repo_root();
    let mut out = Vec::new();
    for p in ui_sources() {
        let Ok(src) = fs::read_to_string(&p) else {
            continue;
        };
        let rel = p
            .strip_prefix(&root)
            .unwrap_or(&p)
            .to_string_lossy()
            .replace('\\', "/");
        for (n, line) in src.lines().enumerate() {
            if !is_a_tail(line) {
                continue;
            }
            for r in RUNGS {
                if line.contains(&format!("Spacing::{r}.px()")) {
                    out.push(format!("{rel}:{}: {}", n + 1, line.trim()));
                    break;
                }
            }
        }
    }
    out
}

/// ⭐ **Ninguém escreve a cauda de um bloco à mão — ela sai da porta.**
///
/// **Mutação que deve sangrar:** repor `y + row_h + Spacing::Md.px()` em qualquer um dos oito
/// sítios do editor de áudio — é exactamente o estado em que esta wave encontrou o app.
#[test]
fn the_tail_of_a_block_is_never_written_at_the_painting_site() {
    let found = hand_written_tails();
    assert!(
        found.is_empty(),
        "{} sitio(s) escrevem a cauda de um bloco em vez de chamar \
         `ph2d_tokens::control_gap_px()` (ou `control_gap_px()`, se o que acaba ali e' uma LINHA). \
         Cada um e' a segunda resposta a uma pergunta que ja' tem uma:\n  {}",
        found.len(),
        found.join("\n  ")
    );
}

/// ⭐⭐ **A escada tem TRÊS degraus, e a ordem deles é a lei.**
///
/// ⚠️ **Deliberadamente não re-escrevo nenhuma das expressões aqui** — comparar cada porta com a
/// própria conta seria o gate vácuo que esta jornada já pagou três vezes. O que se afirma é a
/// **relação** entre elas, que não é derivável de nenhuma sozinha.
///
/// ⛔⛔ **A wave 20 tirou um degrau desta escada, e a razão está no doc do
/// [`ph2d_tokens::control_gap_px`]:** o `block_gap` (6) da wave 19 justificava-se por a fronteira
/// de um bloco ter de se ler mais que a fronteira entre duas linhas DELE — e isso pressupõe que as
/// linhas de um bloco distam o vão de linha. A wave 20 **junta os botões em grupos**, onde as
/// peças distam um **fio de 1 px**, e a premissa dissolve-se. *Uma recusa medida responde uma
/// pergunta; quem muda o substrato tem de a reconferir.*
#[test]
fn a_control_breathes_more_than_a_list_row_and_less_than_a_section() {
    let list = ph2d_tokens::list_row_gap_px();
    let control = ph2d_tokens::control_gap_px();
    let section = ph2d_tokens::section_gap_px();
    assert!(
        list < control,
        "uma LISTA ({list}) nao esta' mais apertada que dois controlos ({control}) — as linhas de \
         uma lista deixaram de encostar"
    );
    assert!(
        control < section,
        "dois CONTROLOS ({control}) separam-se tanto ou mais que duas SECCOES ({section}) — um \
         controlo interior passa a ler-se como uma seccao, que e' a confusao que a escada existe \
         para impedir"
    );
}

/// ⭐⭐⭐ **A porta do CARTÃO e a porta do VÃO DE SECÇÃO são a mesma resposta.**
///
/// ⚠️ **É a metade que impede a lei de se partir em duas outra vez.** O `SectionCards::close_at`
/// avança `y + section_gap_px()`; se alguém lá escrever de novo um `pad * 2.0` — ou qualquer outro
/// número —, o vão que o app DESENHA entre dois cartões deixa de ser o que o `control_gap_px` diz
/// que ele é, e a escada acima passa a comparar-se com uma constante que ninguém pinta.
///
/// **Mutação que deve sangrar:** trocar o `section_gap_px()` do `close_at` por `pad * 2.0`.
#[test]
fn the_card_door_asks_the_section_gap_instead_of_computing_one() {
    let root = repo_root();
    let src =
        fs::read_to_string(root.join("crates/ph2d-editor-core/src/widget/section_cards/mod.rs"))
            .expect("o modulo dos cartoes de seccao mudou de sitio");
    let body: Vec<&str> = src
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect();
    let body = body.join("\n");
    assert!(
        body.contains("section_gap_px()"),
        "o `close_at` dos cartoes deixou de chamar `ph2d_tokens::section_gap_px()` — o vao que o \
         app DESENHA entre dois cartoes voltou a ser um numero local, e a escada de tres degraus \
         passa a comparar-se com uma constante que ninguem pinta"
    );
}
