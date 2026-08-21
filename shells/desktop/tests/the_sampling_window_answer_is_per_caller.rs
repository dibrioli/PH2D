//! **CADA CHAMADOR DA CAUDA PARTILHADA TEM DE ESCOLHER A SUA RESPOSTA — E ESTE GATE MEDE A
//! ESCOLHA, NÃO A FUNÇÃO.**
//!
//! # Por que este ficheiro existe ao lado dos testes de unidade
//!
//! `rebind_to_individual` ganhou um parâmetro [`SamplingWindow`] em 2026-08-21, e três testes de
//! unidade provam que os dois ramos fazem coisas diferentes
//! (`texture_edit::sampling_window_tests`). ⚠️ **Isso não chega**: o defeito que shipou não estava
//! no corpo da função, estava na **escolha do argumento**. Um gate que exercita os dois ramos fica
//! verde enquanto alguém passa o ramo errado — e foi exatamente essa a forma do bug.
//!
//! O caminho de precisão não é testável de ponta a ponta sem GPU (`apply` precisa de um
//! `SpriteRenderer`), por isso a escolha é medida onde ela está escrita: no texto do chamador.
//!
//! ⚠️ **A varredura ignora comentários.** É a lição que esta mesma linha pagou em 2026-08-20: o
//! gate irmão `a_verb_that_costs_precision_says_so` procurava no ficheiro inteiro, e um
//! doc-comment a *mencionar* a chamada satisfazia-o — a mutação sobreviveu. Aqui só o corpo conta.
//!
//! # O que cada chamador tem de responder, e porquê
//!
//! | chamador | resposta | razão |
//! |---|---|---|
//! | `precision_convert.rs` | `Survives` | sobe **a mesma imagem** noutra precisão — mesmo conteúdo, mesmas dimensões, e o `region_rect` é em pixels da fonte |
//! | `texture_edit.rs` (as duas caudas de commit) | `Dies` | os pixels novos são **outra imagem**: `read_sprite_source` já recortou a região, então a janela antiga recortaria um pedaço arbitrário dela |
//!
//! [`SamplingWindow`]: 'shells/desktop/src/hero_intents/texture_edit.rs'

use std::path::{Path, PathBuf};

fn shell_src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// O corpo sem comentários de linha — a mesma lei do gate irmão, e pela mesma razão medida.
fn code_without_comments(src: &str) -> String {
    src.lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

/// O corpo de PRODUÇÃO: sem comentários e **sem os módulos `#[cfg(test)]`**.
///
/// ⚠️ A primeira versão desta varredura contou os próprios testes de unidade como chamadores — eles
/// exercitam os dois ramos de propósito, por isso «5 mortes» e «8 respostas para 4 chamadas». *Um
/// gate que conta a si mesmo mede o teste, não o produto.*
fn body_of(rel: &str) -> String {
    let path = shell_src().join(rel);
    let src = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("ler {path:?}: {e}"));
    let production = src.split("#[cfg(test)]").next().unwrap_or("");
    code_without_comments(production)
}

/// **A conversão de precisão preserva a janela.** O contrato que `precision_convert.rs` declara
/// por escrito — *«`8 → 16 → 8` … tem de devolver a mesma sprite»* — depende deste argumento.
#[test]
fn the_precision_swap_asks_for_the_window_to_survive() {
    let body = body_of("precision_convert.rs");
    assert!(
        body.contains("SamplingWindow::Survives,"),
        "`precision_convert.rs` nao pede `SamplingWindow::Survives` na chamada a \
         `rebind_to_individual`.\n\
         A troca de precisao sobe A MESMA imagem noutra precisao: apagar `region_enabled` ali \
         destro'i o recorte do artista sem aviso, contra o contrato escrito no topo do proprio \
         ficheiro. Foi o defeito de 2026-08-21 (`docs/Sprite_projeto/20` §4.1)."
    );
    assert!(
        !body.contains("SamplingWindow::Dies,"),
        "`precision_convert.rs` pede `SamplingWindow::Dies` algures — nenhum caminho de precisao \
         deve matar a janela"
    );
}

/// **As caudas de commit de ferramenta matam a janela** — o controlo positivo.
///
/// ⚠️ Sem ele, a cura degenera em «nunca apagar» e revive o bug das «múltiplas repetições» de
/// 2026-08-19: uma peça de folha a amostrar a imagem nova por uma janela que já não a descreve.
#[test]
fn the_tool_commits_ask_for_the_window_to_die() {
    let body = body_of("hero_intents/texture_edit.rs");
    // ⚠️ **Com vírgula final: posição de ARGUMENTO.** Sem ela contaríamos também o
    // `if window == SamplingWindow::Dies {` de dentro da própria função — que é a leitura da
    // resposta, não uma resposta.
    let dies = body.matches("SamplingWindow::Dies,").count();
    assert_eq!(
        dies, 2,
        "esperava as DUAS caudas de commit de ferramenta (8 bits e 16 bits) a pedir \
         `SamplingWindow::Dies`, e contei {dies}.\n\
         As duas escrevem pixels NOVOS — `read_sprite_source` ja' recortou a regiao, entao a \
         janela antiga recortaria um pedaco arbitrario da imagem nova."
    );
}

/// **Nenhum chamador esquece o argumento.** Um terceiro chamador que nasça sem escolher é o
/// próximo defeito desta família, e ele passa MUDO: o compilador exige o parâmetro, mas não exige
/// que quem o passa tenha pensado.
///
/// ⚠️ A conta é *chamadas de `rebind_to_individual` no corpo* contra *variantes de
/// `SamplingWindow` no corpo*, somada sobre os dois ficheiros que a chamam. Uma chamada nova sem
/// resposta desequilibra a conta e este gate nomeia-a.
#[test]
fn every_call_site_states_an_answer() {
    let mut calls = 0usize;
    let mut answers = 0usize;
    for rel in [
        "precision_convert.rs",
        "hero_intents/texture_edit.rs",
        "hero_intents/texture_rebind.rs",
    ] {
        let body = body_of(rel);
        // A definição da função também casa `rebind_to_individual(` — desconta-se onde ela vive.
        let defs = body.matches("fn rebind_to_individual(").count();
        calls += body.matches("rebind_to_individual(").count() - defs;
        answers += body.matches("SamplingWindow::Dies,").count()
            + body.matches("SamplingWindow::Survives,").count();
    }
    assert!(
        calls >= 3,
        "so' {calls} chamadas encontradas — a varredura partiu-se e este gate mede o vazio"
    );
    assert_eq!(
        calls, answers,
        "ha' {calls} chamadas a `rebind_to_individual` e {answers} respostas de `SamplingWindow` \
         nos mesmos ficheiros.\n\
         Toda chamada tem de dizer o que acontece a` janela de amostragem — e a resposta certa \
         depende de os bytes novos serem OUTRA imagem (`Dies`) ou A MESMA noutra precisao \
         (`Survives`)."
    );
}

/// **TODO CAMINHO QUE TIRA A SPRITE DA FOLHA TEM DE LARGAR A AUTORIA.**
///
/// `SpriteSheetRef` diz *"os meus pixels são a região R da folha F"*. Quando os pixels ganham
/// outro dono — carimbo próprio ou célula do atlas partilhado — a afirmação fica falsa, e deixá-la
/// faz o `restore_sprite_sheets` re-ligar a sprite no load seguinte e **apagar a conversão**.
///
/// ⚠️ **O defeito só aparece depois de fechar e reabrir o projeto.** Em 2026-08-21 o
/// `demote_to_atlas` (Individual → Atlas) estava sem ele enquanto o caminho oposto o tinha desde o
/// primeiro dia — e o comentário do caminho oposto **descrevia o perigo por extenso**
/// (`docs/Sprite_projeto/20` §4.3). *Uma invariante que dois sítios têm de lembrar é uma que um
/// deles vai esquecer;* daí a porta única `drop_sheet_authorship`.
#[test]
fn every_path_that_re_homes_the_pixels_drops_the_sheet_authorship() {
    let strategy = body_of("render_loop/inspector_strategy.rs");
    assert!(
        strategy.contains("drop_sheet_authorship("),
        "`inspector_strategy.rs` re-aloja os pixels numa celula de atlas e nao chama \
         `drop_sheet_authorship`.\n\
         Sem isso o `restore_sprite_sheets` re-liga a sprite a` folha no load seguinte e apaga a \
         conversao — e o artista so' descobre depois de fechar e reabrir o projeto."
    );

    // ⛔ A remoção crua fora da porta é o que reabre a classe: dois sítios a lembrar-se de uma
    // invariante é um deles a esquecê-la.
    for rel in [
        "render_loop/inspector_strategy.rs",
        "hero_intents/texture_rebind.rs",
    ] {
        let body = body_of(rel);
        let raw = body.matches("remove::<ph2d_ecs::SpriteSheetRef>()").count()
            + body.matches("remove::<SpriteSheetRef>()").count();
        let allowed = usize::from(rel == "hero_intents/texture_rebind.rs");
        assert_eq!(
            raw, allowed,
            "{rel} faz {raw} remocao(oes) crua(s) de `SpriteSheetRef` (esperado {allowed}: so' a \
             porta `drop_sheet_authorship` a faz). Chame a porta."
        );
    }
}
