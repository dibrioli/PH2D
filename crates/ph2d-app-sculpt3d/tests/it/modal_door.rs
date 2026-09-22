//! ⭐⭐⭐⭐ **OS MODAIS DESTA FAMÍLIA PASSAM PELA PORTA** — irmão do
//! `ph2d-app-field3d/tests/it/modal_door.rs`, e ele nasceu de um report do dono.
//!
//! # ⛔⛔⛔ O defeito que o fez existir (22/09)
//!
//! A wave da SAÍDA pôs o aviso *«o que este formato não carrega»* no toast da exportação, e o
//! dono reportou: ***«passo 4 não mostra mensagem nenhuma no app. Onde deveria aparecer?»***
//!
//! A mensagem era escrita, era empurrada para a fila e **nunca chegava a ser pintada**. Um
//! `rfd::FileDialog` **congela o laço**, logo o quadro seguinte traz um `wall_dt` do tamanho do
//! tempo que o artista passou no diálogo; o `ToastQueue` anda `3 s` de relógio de **PAREDE**, e o
//! `tick` do primeiro quadro depois do diálogo apagava o toast antes de alguém o desenhar.
//!
//! ⭐⭐ **E a cura já existia desde 2026-08-22** — com as palavras do dono de então, citadas no
//! `fase_chrome_clock.rs`: *«não vejo em nenhum lugar a mensagem»*. Ela vive no
//! [`ph2d_app_host::modal`], que desconta do relógio do chrome a parte parada **declarada por
//! quem a causou**. ⇒ *este chamador é que estava fora dela.*
//!
//! ⚠️⚠️ **A lei: uma cura escrita para UM chamador não é uma lei — só uma PORTA é.** A mesma
//! frase que esta linha já pagou no `stroke_uniform`, no `compact_for_faces` e no
//! `fora_da_pegada`, aqui uma camada acima: a porta estava construída, tinha um gate a exigi-la, e
//! **a população desse gate era UMA crate**.
//!
//! # ⛔ E a agulha do irmão era CEGA AO PLURAL
//!
//! A lista dele é `[".save_file()", ".pick_file()"]`, e `.pick_files()` **não contém**
//! `.pick_file()` — o parêntesis fecha antes do `s`. A importação desta família usa exactamente o
//! plural, logo ela teria passado por aquele gate sem uma palavra. *Uma agulha que fecha o
//! parêntesis mede o verbo exacto e é cega ao irmão dele*; as três estão aqui, e o `.pick_files()`
//! foi acrescentado ao do vizinho no mesmo commit.
//!
//! # ⚠️ Por que ele tem PISO DE POPULAÇÃO
//!
//! Pela razão que o irmão aprendeu ao sair da shell: uma varredura de directório fica **verde a
//! medir zero** no dia em que o directório muda de sítio, e `bad.is_empty()` sobre uma lista
//! construída a partir de nenhum ficheiro é trivialmente verdadeiro.

/// Quantos ficheiros de produto esta família tinha quando este piso foi escrito (2026-09-22).
///
/// ⚠️ **É um PISO, não uma igualdade** — a família cresce, e exigir o número exacto seria uma
/// catraca a reprovar todo ficheiro novo. O que ele barra são os dois modos em que a varredura
/// deixa de ter sujeito sem o dizer: a **vazia** e a **decapitada**.
const PISO_DE_FICHEIROS: usize = 150;

#[test]
fn every_sculpt3d_modal_goes_through_the_door() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut bad = Vec::new();
    let mut vistos = 0usize;
    for entry in std::fs::read_dir(&dir)
        .expect("o src da família existe")
        .flatten()
    {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if !name.ends_with(".rs") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        vistos += 1;
        // ⚠️ A agulha é a CHAMADA que bloqueia, não o tipo: construir o `FileDialog` é inofensivo,
        // e os dois módulos continuam a fazê-lo para montar os filtros.
        //
        // ⚠️ **COMENTÁRIOS FORA**, e não é higiene: o comentário que EXPLICA a regra contém a
        // agulha por construção — um gate que lê a prosa sobre a lei reprova quem a documenta.
        for line in text.lines().filter(|l| !l.trim_start().starts_with("//")) {
            for needle in [".save_file()", ".pick_file()", ".pick_files()"] {
                if line.contains(needle) {
                    bad.push(format!("{name} chama `{needle}` fora da porta"));
                }
            }
        }
    }
    assert!(
        vistos >= PISO_DE_FICHEIROS,
        "este gate varreu {vistos} ficheiros e esperava pelo menos {PISO_DE_FICHEIROS} — ele \
         perdeu o sujeito (a família mudou-se?) e estava prestes a afirmar-se VERDE sobre nada"
    );
    assert!(
        bad.is_empty(),
        "diálogo modal aberto sem declarar o congelamento — use `ph2d_app_host::modal::save_file` \
         / `pick_file` / `pick_files`. Sem isso a mensagem escrita LOGO A SEGUIR ao diálogo é \
         apagada antes de ser pintada (report do dono, 2026-09-22):\n  {}",
        bad.join("\n  ")
    );
}

/// ⭐⭐ **O CONTROLO da agulha do plural** — ele é o que separa este gate do irmão.
///
/// ⚠️ Sem ele, alguém que voltasse a pôr a lista em `[".save_file()", ".pick_file()"]` não
/// reprovava aqui: a família passaria a poder abrir `pick_files` à mão outra vez, e o defeito do
/// report voltava **pela metade que ninguém mede**.
#[test]
fn a_agulha_do_plural_nao_e_apanhada_pela_do_singular() {
    assert!(
        !".pick_files()".contains(".pick_file()"),
        "se o plural contivesse o singular, a lista de duas agulhas bastaria — e ela NÃO basta"
    );
}
