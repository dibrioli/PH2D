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
const MUTE_OK: &[&str] = &[
    // ⚠️ Estes quatro do `ph2d-editor-core` são chrome com geometria própria (o picker, a barra de
    //    rolagem, duas páginas da galeria) — eles empilham AMOSTRAS, não linhas de formulário. A
    //    régua não os separa hoje, e nomeá-los é mais honesto do que alargar o sujeito até eles
    //    saírem por acidente.
    "crates/ph2d-editor-core/src/widget/blender_color_picker/paint.rs",
    "crates/ph2d-editor-core/src/widget/scrollbar.rs",
    "crates/ph2d-editor-core/src/widget/showcase/actions.rs",
    "crates/ph2d-editor-core/src/widget/showcase/switches.rs",
    // Painéis anteriores à porta, cada um por medir e curar.
    "crates/ph2d-panel-color-equalization/src/paint.rs",
    "crates/ph2d-panel-painter-layers/src/paint.rs",
    "crates/ph2d-panel-painter-layers/src/paint_taper.rs",
    "crates/ph2d-panel-timeline/src/geom.rs",
    "crates/ph2d-panel-timeline/src/tracks.rs",
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
    src.match_indices("y +=").any(|(i, _)| {
        i == 0
            || !src[..i]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_alphanumeric() || c == '_')
    }) || src.contains("cursor_y")
}

/// Empilha linhas? — avança um cursor vertical **e** mede a altura de uma linha.
fn stacks_rows(src: &str) -> bool {
    advances_y(src) && src.contains("ROW_H_PX")
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
            if !stacks_rows(&s) || doors.iter().any(|d| s.contains(d.as_str())) {
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
    let fresh: Vec<&String> = found
        .iter()
        .filter(|f| !MUTE_OK.contains(&f.as_str()))
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
    let stale: Vec<&str> = MUTE_OK
        .iter()
        .copied()
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
