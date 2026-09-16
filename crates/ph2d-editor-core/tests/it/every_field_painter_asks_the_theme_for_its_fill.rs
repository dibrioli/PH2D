//! ⭐⭐ **A metade do PINTOR: quem enche um campo pergunta à porta, nunca ao token.**
//!
//! ⛔⛔ **Report do dono, 2026-09-14:** *«caixas de input numérico sem cor de fundo»*. A causa não
//! era a paleta — era **quem a escolhia**: o `text_input` perguntava ao tema
//! (`visuals::Chrome::field_fill`) e os dois pintores irmãos, `number_input` e `text_area`,
//! enchiam com `ColorToken::Bg1`, que é o token de um **cartão de secção**. Sobre um cartão isso é
//! `0/255`, e num tema moderno a moldura de repouso é zero ⇒ nenhuma caixa.
//!
//! ⚠️ ***Uma lei com uma porta e dois consumidores fora dela não é uma lei; é uma coincidência que
//! ainda não divergiu.*** O gate irmão (`a_field_is_never_the_colour_of_what_it_sits_on`, em
//! `ph2d-tokens`) defende a COR; este defende o **caminho** até ela — os dois são precisos, porque
//! uma paleta certa lida por um pintor que não a lê não pinta nada.

use std::fs;
use std::path::{Path, PathBuf};

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
            walk(&p, out);
        } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

/// O sítio ÚNICO onde a escada clássica do preenchimento pode ser lida — a porta.
const A_PORTA: &str = "crates/ph2d-editor-core/src/widget/text_input/mod.rs";

#[test]
fn every_field_painter_asks_the_theme_for_its_fill() {
    let root = repo_root();
    let mut ficheiros = Vec::new();
    walk(
        &root.join("crates/ph2d-editor-core/src/widget"),
        &mut ficheiros,
    );
    ficheiros.sort();
    // ⚠️ **Piso de população:** um caminho errado varre zero ficheiros e o gate fica verde a medir
    // nada (a armadilha §2.7 do HOWTO, que já mordeu quatro vezes neste repo).
    assert!(
        ficheiros.len() >= 60,
        "a varredura achou so' {} ficheiros de widget — o caminho mudou?",
        ficheiros.len()
    );

    let mut fora = Vec::new();
    let mut na_porta = 0;
    for p in &ficheiros {
        let Ok(src) = fs::read_to_string(p) else {
            continue;
        };
        let rel = p
            .strip_prefix(&root)
            .unwrap_or(p)
            .to_string_lossy()
            .replace('\\', "/");
        for (n, line) in src.lines().enumerate() {
            let t = line.trim();
            // ⚠️ Prosa não conta: este ficheiro e os doc-comments dos pintores NOMEIAM o defeito,
            // e um censo que não separa prosa de código mente nos dois sentidos.
            if t.starts_with("//") || !t.contains("fill_token(") {
                continue;
            }
            if rel.ends_with(A_PORTA.trim_start_matches("crates/ph2d-editor-core/")) {
                na_porta += 1;
            } else {
                fora.push(format!("{rel}:{}: {t}", n + 1));
            }
        }
    }
    assert!(
        fora.is_empty(),
        "{} pintor(es) escolhem o fundo de um campo por TOKEN em vez de perguntar a\n\
         `text_input::field_fill(state, theme)`:\n  {}\n\nNum tema moderno nao ha moldura de \
         repouso — um token que calhe ser o do cartao por baixo deixa o campo INVISIVEL, que foi \
         o report do dono de 2026-09-14.",
        fora.len(),
        fora.join("\n  ")
    );
    // ⚠️ E a porta tem de continuar a existir: `0` aqui significa que a escada clássica se perdeu
    // (o clássico deixaria de ser byte-idêntico) — não que o app ficou limpo.
    assert!(
        na_porta >= 1,
        "a escada classica do preenchimento desapareceu da porta"
    );
}

/// ⏳ **Dívida MEDIDA, e só ENCOLHE** — quem ainda resolve o token do CARTÃO para pintar um corpo.
const AINDA_O_CARTAO: &[&str] = &[
    // ⭐ O cartão É o cartão: aqui o token não é a tinta de um controlo, é a superfície.
    "crates/ph2d-editor-core/src/widget/section_cards/mod.rs",
    // ⛔ A escada CLÁSSICA vive dentro da porta — é o valor que ela devolve quando o tema traça
    //    moldura de repouso, e ali `0/255` lê-se pela borda.
    "crates/ph2d-editor-core/src/widget/text_input/mod.rs",
    // ⏳ **NOMEADO, e não é o mesmo caso:** o miolo do menu radial flutua sobre o CANVAS, e o
    //    `Bg1` responde *também* a «de que cor é o canvas» (o doc do `derive::Roles::panel`
    //    escreve-o). A porta afunda um degrau abaixo da pilha PAINEL/CARTÃO, que não é a
    //    superfície debaixo deste. ⛔ Converter às cegas trocaria um defeito por outro — quem lhe
    //    tocar mede primeiro contra o que está por baixo DELE.
    "crates/ph2d-editor-core/src/widget/radial_menu.rs",
];

/// ⭐⭐⭐ **Nenhum pintor de controlo enche com o token da superfície em que o controlo assenta.**
///
/// ⛔⛔ **Segundo report do dono no mesmo dia, com foto: *«Checkbox invisível»*.** A caixa
/// *Centered*, MARCADA, lê-se (é `Accent`); as de *Flip H* e *Flip V*, desmarcadas, não existem —
/// elas enchiam `ColorToken::Bg1`, que é o token do cartão por baixo, e num tema moderno a moldura
/// de repouso é ZERO.
///
/// ⚠️ **Eram SEIS pintores, não um:** `number_input` · `text_area` · `checkbox` · `combobox` ·
/// `dropdown` · `radio_group`. *Seis respostas à mesma pergunta divergem no dia em que uma
/// superfície se mexe — e uma mexeu-se em 2026-09-05, quando o painel desceu para o cartão se ler.*
#[test]
fn no_widget_paints_a_control_body_with_the_card_token() {
    let root = repo_root();
    let mut ficheiros = Vec::new();
    walk(
        &root.join("crates/ph2d-editor-core/src/widget"),
        &mut ficheiros,
    );
    ficheiros.sort();
    assert!(
        ficheiros.len() >= 60,
        "a varredura achou so' {} ficheiros de widget — o caminho mudou?",
        ficheiros.len()
    );

    let mut fora = Vec::new();
    for p in &ficheiros {
        let Ok(src) = fs::read_to_string(p) else {
            continue;
        };
        let rel = p
            .strip_prefix(&root)
            .unwrap_or(p)
            .to_string_lossy()
            .replace('\\', "/");
        // ⛔⛔ **O código de TESTE tem DUAS formas nesta casa, e a cerca só conhecia uma.** O
        //    `#[cfg(test)]` abaixo apanha o `mod tests` no fim do ficheiro; o repo também usa
        //    o ficheiro IRMÃO (`number_input/tests.rs`, `text_input/tests.rs`, e desde
        //    2026-09-15 o `checkbox/mark_tests.rs`, que nasceu do tecto de LOC). Este gate
        //    ficou vermelho na hora em que um bloco de teste **mudou de forma sem mudar de
        //    natureza**.
        // ⇒ *«o que é código de teste» responde-se UMA vez, e cobre as duas formas.*
        let base = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if base == "tests.rs" || base.ends_with("_tests.rs") {
            continue;
        }
        for (n, line) in src.lines().enumerate() {
            let t = line.trim();
            // ⚠️ **O módulo de teste termina a varredura.** Um gate que afirma a lei CITA o token
            // dos dois lados, e contá-lo acusaria o próprio gate — o defeito que o
            // `a_census_gate_that_scans_its_own_tree_counts_itself` já registou.
            // ⛔ A cerca presume o que o `rustfmt` desta casa faz: o `mod tests` é o FIM do
            //    ficheiro. Um bloco de teste a meio esconderia o código depois dele.
            if t.starts_with("#[cfg(test)]") {
                break;
            }
            if t.starts_with("//") {
                continue;
            }
            if !t.contains("resolve(ColorToken::Bg1") && !t.contains("ColorToken::Bg1.resolve") {
                continue;
            }
            if AINDA_O_CARTAO.iter().any(|d| rel.starts_with(d)) {
                continue;
            }
            fora.push(format!("{rel}:{}: {t}", n + 1));
        }
    }
    assert!(
        fora.is_empty(),
        "{} pintor(es) enchem um corpo com o token do CARTAO em que ele assenta:\n  {}\n\n\
         Num tema moderno nao ha moldura de repouso — um corpo da cor do que esta' por baixo dele \
         NAO EXISTE. A porta e' `crate::paint::body_fill(theme, feel, classico)`.",
        fora.len(),
        fora.join("\n  ")
    );
}

/// ⚠️ **A metade de OBSOLESCÊNCIA** — `CLAUDE.md` §5.0: uma catraca sem censo não desce, vira
/// licença.
#[test]
fn the_card_token_tolerance_still_describes_something() {
    let root = repo_root();
    let mortas: Vec<&&str> = AINDA_O_CARTAO
        .iter()
        .filter(|d| {
            let p = root.join(d);
            let Ok(src) = fs::read_to_string(&p) else {
                return true;
            };
            !src.contains("ColorToken::Bg1")
        })
        .collect();
    assert!(
        mortas.is_empty(),
        "entrada(s) STALE na tolerancia — o ficheiro sumiu ou ja' nao cita o token:\n  {mortas:?}"
    );
}
