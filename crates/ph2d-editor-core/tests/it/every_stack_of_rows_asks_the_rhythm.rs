//! ⭐⭐⭐ **QUEM EMPILHA LINHAS PERGUNTA O RITMO** — o censo que a integração de 2026-09-07 pediu.
//!
//! > *«as telas novas das outras cinco frentes … foram escritas antes de a interface nova existir,
//! > e ainda não usam as medidas dela. Não é defeito nem trabalho da junção — é uma tarefa para
//! > quem cuida da interface.»* — o agente integrador, 2026-09-07.
//!
//! # ⛔ Por que o `no_magic_numeric` não vê isto
//!
//! Ele pergunta *«alguém escreveu um número CRU?»*, e a resposta destas superfícies é **não**: elas
//! escrevem `Spacing::Xs.px()`, que é um token. O que elas não fazem é perguntar **a porta da
//! grandeza** — e as duas coisas divergiram no dia em que o dono fixou o vão entre controlos em
//! `3 px` (`control_gap_px`), que **não** é o `Spacing::Xs` (`4`). ⇒ *um token certo na pergunta
//! errada passa em todo gate desta casa e continua fora do ritmo.*
//!
//! # ⚠️ As DUAS afirmações que este censo NÃO faz
//!
//! 1. **Não acusa quem empilha qualquer coisa.** A 1.ª redacção varria todo `y +=` e devolvia
//!    **48 de 81** ficheiros — uma barra de rolagem e a galeria de widgets incluídas, que avançam
//!    um cursor e não empilham linha nenhuma. *Uma acusação sobre a população errada fabrica
//!    dívida.* O sujeito é quem menciona o [`ph2d_tokens::ROW_H_PX`]: quem mede a altura de uma
//!    LINHA está a empilhar linhas.
//! 2. **Não diz que o número está errado** — diz que a **pergunta** não foi feita. Um ficheiro que
//!    chame qualquer uma das portas está fora do censo, mesmo que use `Spacing` noutro sítio para
//!    outra coisa (recuar um rect, medir uma caixa). *A porta é a prova de que alguém sabia que a
//!    grandeza tinha dono.*
//!
//! # ⚠️ A catraca traz o censo de obsolescência ao lado
//!
//! `CLAUDE.md` §5.0: *uma catraca sem censo de obsolescência não desce — ela vira LICENÇA.* A
//! segunda metade deste ficheiro pergunta, por entrada, se o ficheiro ainda existe, se ainda
//! empilha linhas e se ainda não pergunta. Uma entrada que já não descreve nada **reprova**.

use std::fs;
use std::path::{Path, PathBuf};

/// As portas do ritmo — as perguntas que uma superfície que empilha tem de fazer a alguém.
///
/// ⚠️ **Derivadas do fonte, não escritas à mão:** a lista sai dos `pub fn` do
/// `ph2d-tokens/src/spacing.rs` cujo nome acaba em `_px`, para que uma porta nova entre no censo
/// no dia em que nasce. *Uma lista de portas escrita aqui envelheceria na primeira wave.*
fn doors(repo: &Path) -> Vec<String> {
    let src = fs::read_to_string(repo.join("crates/ph2d-tokens/src/spacing.rs"))
        .expect("o ficheiro das portas do ritmo tem de existir");
    let mut v: Vec<String> = src
        .lines()
        .filter_map(|l| l.trim().strip_prefix("pub fn "))
        .filter_map(|l| l.split('(').next())
        .filter(|n| n.ends_with("_px"))
        .map(str::to_string)
        .collect();
    v.sort();
    v.dedup();
    v
}

/// ⛔ **A DÍVIDA TOLERADA — ela só ENCOLHE.**
///
/// Cada entrada é uma superfície que empilha linhas e nunca pergunta o ritmo. Curar uma é trocar o
/// `let row_gap = Spacing::…` pela porta da grandeza; ⛔ acrescentar uma exige o veredito de quem
/// cuida da interface, e a direcção é sempre para baixo.
///
/// Medido em 2026-09-08: **16**, e cinco painéis foram curados na mesma wave (equalize-sizes ·
/// inspector/color_tint · padding · physics/body · upscale), o que a deixa em **11**.
///
/// ⚠️ **E o censo de obsolescência apanhou DUAS na primeira corrida** — o `grid-snap/
/// paint_helpers.rs` e o `painter-layers/paint_inpaint.rs` avançam um cursor com outro NOME
/// (`sy`, `body_y`), e a medição a shell que produziu esta lista não tinha fronteira de
/// palavra. *Duas acusações fabricadas, mortas pela metade que pergunta se a entrada ainda
/// descreve alguma coisa.* Ficam **9**.
///
/// ⭐ **E desceu para 8 em 2026-09-09, sem ninguém a mirar a lista:** a wave dos modos do
/// Painter pôs o vão entre a fileira segmentada e o corpo na porta `control_gap_px()`, e o
/// censo de obsolescência cobrou a entrada no mesmo fecho. *Uma catraca com censo desce por
/// efeito colateral de quem faz a coisa certa noutro sítio.*
const MUTE_OK: &[&str] = &[];

/// ⛔⛔ **A OUTRA lista — e ela não é dívida: é outra PERGUNTA.**
///
/// `(caminho, por que o passo ali não é «uma linha de formulário»)`.
///
/// ⚠️ **Sem esta separação a catraca mentia nos dois sentidos:** ela chamava «por medir e curar» a
/// duas superfícies cujo passo está CERTO, e quem viesse curá-las mudaria a geometria de um
/// dope-sheet para satisfazer uma régua. *Uma lista de dívida que contém não-dívida é uma licença
/// com cara de catraca* — a mesma doença que o §5.0 do `CLAUDE.md` descreve, um nível acima.
const NOT_THIS_QUESTION: &[(&str, &str)] = &[
    (
        "crates/ph2d-panel-timeline/src/geom.rs",
        "GRELHA DENSA, nao formulario: o passo de uma track E' a altura dela (`ROW_H_PX`), sem \
         vao nenhum, porque a fileira tem de alinhar com a regua e com as chaves. Um vao entre \
         linhas desalinhava o dope-sheet do editor de curvas",
    ),
    (
        "crates/ph2d-panel-timeline/src/tracks.rs",
        "A MESMA grelha, no ficheiro que a pinta: `y += ROW_H_PX` e' o passo da grade, e as \
         posicoes das chaves derivam dele (`ROW_H_PX * i`)",
    ),
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/<crate>/ tem dois pais")
        .to_path_buf()
}

fn scan_roots(repo: &Path) -> Vec<PathBuf> {
    let mut v = vec![
        repo.join("crates/ph2d-editor-core/src/widget"),
        repo.join("crates/ph2d-editor-core/src/screens"),
    ];
    if let Ok(entries) = fs::read_dir(repo.join("crates")) {
        let mut panels: Vec<PathBuf> = entries
            .flatten()
            .filter_map(|e| {
                let p = e.path();
                let n = p.file_name()?.to_str()?.to_string();
                (p.is_dir() && n.starts_with("ph2d-panel-") && n != "ph2d-panel-registry-init")
                    .then(|| p.join("src"))
                    .filter(|s| s.is_dir())
            })
            .collect();
        panels.sort();
        v.extend(panels);
    }
    v
}

fn rs_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n == "tests") {
                continue;
            }
            rs_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs")
            && !p
                .file_name()
                .is_some_and(|n| n.to_str().is_some_and(|s| s.ends_with("_tests.rs")))
        {
            out.push(p);
        }
    }
}

/// O cursor chama-se mesmo `y`? — `sy += …` e `body_y += …` são outras grandezas.
///
/// ⚠️ **Sem esta fronteira o censo saltou de 16 para 28 acusados** na primeira corrida: um
/// `contains("y +=")` apanha o fim de qualquer nome. *Uma régua sem fronteira de palavra mede um
/// sufixo, não uma variável.*
fn advances_y(src: &str) -> bool {
    let code = code_only(src);
    code.match_indices("y +=").any(|(i, _)| {
        i == 0
            || !code[..i]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_alphanumeric() || c == '_')
    }) || code.contains("cursor_y")
}

/// O ficheiro **sem comentários de linha** — a prosa não empilha nada.
///
/// ⛔⛔ **A 1.ª redacção lia o fonte CRU, e fabricou uma acusação:** o `widget/scrollbar.rs` não
/// avança `y` em lado nenhum; ele diz *«`cursor_y - cursor_y_at_down`»* num **doc-comment** sobre
/// o arrasto. ⇒ *um censo textual que não separa prosa de código mente nos dois sentidos* — é a
/// 5.ª vez que esta linha paga a mesma lição, e a primeira em que o acusado é um ficheiro que
/// nunca teve o defeito.
fn code_only(src: &str) -> String {
    src.lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Empilha linhas? — avança um cursor vertical **e** mede a altura de uma linha.
fn stacks_rows(src: &str) -> bool {
    advances_y(src) && src.contains("ROW_H_PX")
}

/// O `mod.rs` do módulo a que este ficheiro pertence — vazio quando ele próprio é o `mod.rs`.
fn sibling_mod(p: &Path) -> String {
    if p.file_name().is_some_and(|n| n == "mod.rs") {
        return String::new();
    }
    p.parent()
        .map(|d| d.join("mod.rs"))
        .and_then(|m| fs::read_to_string(m).ok())
        .unwrap_or_default()
}

/// As superfícies que empilham linhas e nunca perguntam o ritmo, por caminho relativo ao repo.
fn mute(repo: &Path) -> Vec<String> {
    let doors = doors(repo);
    let mut out = Vec::new();
    for root in scan_roots(repo) {
        let mut files = Vec::new();
        rs_files(&root, &mut files);
        for p in files {
            let Ok(s) = fs::read_to_string(&p) else {
                continue;
            };
            // ⚠️ **A porta pode viver no `mod.rs` IRMÃO, e ler um ficheiro de cada vez não a
            //    vê.** O `widget/showcase/actions.rs` avança por `row_gap()`, que é
            //    `ph2d_tokens::control_gap_px()` no `showcase/mod.rs` ao lado — duas acusações
            //    fabricadas por uma régua que não segue uma chamada de um passo. ⇒ o alcance é o
            //    ficheiro **mais o `mod.rs` do módulo dele**, que é o que ele alcança sem `use`.
            let scope = format!("{s}{}", sibling_mod(&p));
            if !stacks_rows(&s) || doors.iter().any(|d| scope.contains(d.as_str())) {
                continue;
            }
            let rel = p.strip_prefix(repo).unwrap_or(&p);
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
    out.sort();
    out
}

/// ⭐⭐⭐ **Nenhuma superfície NOVA empilha linhas sem perguntar o ritmo.**
#[test]
fn every_stack_of_rows_asks_the_rhythm() {
    let repo = repo_root();
    let doors = doors(&repo);
    // ⚠️ **O controlo do INSTRUMENTO:** sem portas encontradas, o censo acusaria a casa inteira e
    //    ninguém saberia porquê. Um ficheiro renomeado tem de reprovar aqui, não lá em baixo.
    assert!(
        doors.len() >= 5,
        "só {} portas do ritmo encontradas em `ph2d-tokens/src/spacing.rs` — o censo perdeu a \
         régua e acusaria toda a gente: {doors:?}",
        doors.len()
    );

    let found = mute(&repo);
    let exempt: Vec<&str> = NOT_THIS_QUESTION.iter().map(|(f, _)| *f).collect();
    let fresh: Vec<&String> = found
        .iter()
        .filter(|f| !MUTE_OK.contains(&f.as_str()) && !exempt.contains(&f.as_str()))
        .collect();
    assert!(
        fresh.is_empty(),
        "estas superfícies empilham LINHAS e nunca perguntam nenhuma das portas do ritmo \
         ({doors:?}) — é o achado do agente integrador de 2026-09-07:\n  {}\n\ncura: a grandeza \
         tem dono. O vão entre dois controlos é `control_gap_px()` (3 px, ordem do dono), o passo \
         de uma linha é `row_pitch_px()`, o de duas secções é `section_gap_px()`. ⛔ Um \
         `Spacing::Xs` escrito à mão passa em todo gate desta casa e continua fora do ritmo.",
        fresh
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

/// ⛔⛔ **E a lista tolerada não pode descrever o que já não existe** — `CLAUDE.md` §5.0.
#[test]
fn the_tolerated_list_has_no_stale_entries() {
    let repo = repo_root();
    let found = mute(&repo);
    let declared: Vec<&str> = MUTE_OK
        .iter()
        .copied()
        .chain(NOT_THIS_QUESTION.iter().map(|(f, _)| *f))
        .collect();
    let stale: Vec<&str> = declared
        .into_iter()
        .filter(|e| !found.contains(&(*e).to_string()))
        .collect();
    assert!(
        stale.is_empty(),
        "entradas toleradas que já não descrevem nada — o ficheiro sumiu, deixou de empilhar \
         linhas, ou passou a perguntar a porta:\n  {}\ncura: apague a linha. As tolerâncias \
         encolhem, nunca crescem.",
        stale.join("\n  ")
    );
}
