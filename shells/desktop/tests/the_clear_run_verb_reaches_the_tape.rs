//! **Arch-gate: os verbos de FITA da §14 (`ClearRun`/`RestoreRun`) são
//! honrados na shell, e não passam pelo fan-out** (W17, W24).
//!
//! ## Por que um gate de TEXTO
//!
//! O `ClearRun` e o seu irmão `RestoreRun` (W24) são os únicos verbos da §14 que
//! não são escritas de componente: a fita de entrada mora na shell
//! (`App.player_tape`), não no `PlatformPlayer`.
//! Os dois são honrados no laço de ações — o lugar onde `self` é mutável, o
//! mesmo do `Join` da §11 e do eyedropper da §12 —, e esse laço exige `hero`/`sim`/
//! `renderer` mais uma janela: **nenhum gate de unidade o alcança**.
//!
//! ## O modo de falha, e é silencioso
//!
//! Empurrá-lo para o `player_edits` compila e passa o seam (que só prova que o
//! clique levanta o verbo): o `apply_player_edit` o nomeia num braço INERTE, o
//! botão fica pintado, clicável, e não descarta corrida nenhuma.
//!
//! ⚠️ E a segunda asserção existe porque *descartar é idempotente*: espalhado
//! pela seleção ele não corromperia nada HOJE. É precisamente essa forma que
//! apodrece — o Ctrl+V do editor de nós colava duas vezes porque um dispatch
//! duplicado *"nunca tinha importado enquanto todos os verbos eram idempotentes"*.

const SRC: &str = include_str!("../src/render_loop/mod.rs");

/// O braço do laço de ações que trata uma edição da §14.
fn player_edit_arm() -> &'static str {
    let at = SRC
        .find("EditorAction::InspectorPlayerEdit { entity_bits, edit } =>")
        .expect("o laco de acoes tem de tratar a edicao da §14");
    let rest = &SRC[at..];
    let end = rest
        .find("EditorAction::InspectorWheelEdit")
        .expect("o braco seguinte fecha este");
    &rest[..end]
}

/// **E ele NÃO cai no fan-out por entidade** — o controle positivo do irmão
/// abaixo.
///
/// Sem esta metade, o `mem::take` escrito AO LADO do `player_edits.push` (as
/// duas coisas, no mesmo braço) passaria: a asserção de presença só diz que o
/// texto certo está lá, não que o errado não está.
#[test]
fn the_clear_run_arm_does_not_fan_out() {
    let arm = player_edit_arm();
    let at = arm
        .find("ClearRun")
        .expect("o braco tem de NOMEAR o verbo que intercepta");
    let after = &arm[at..];
    let else_at = after
        .find("} else {")
        .expect("a intercepcao tem de ter um ramo alternativo para o resto da secao");
    assert!(
        !after[..else_at].contains("player_edits.push"),
        "o `ClearRun` tambem foi empurrado para o fan-out por entidade. Braco:\n{arm}"
    );
    assert!(
        after[else_at..].contains("player_edits.push"),
        "o RESTO da §14 deixou de chegar ao fan-out -- a intercepcao engoliu a secao \
         inteira. Braco:\n{arm}"
    );
}

/// **DESCARTAR ESVAZIA A FITA — GUARDANDO-A** (W17, reescrito na W24).
///
/// ⚠️ Este gate **substitui** o `the_clear_run_arm_clears_the_tape`, que
/// afirmava o literal `self.player_tape.clear()`. O verbo não mudou de sentido:
/// o `mem::take` esvazia a fita viva **e** entrega a corrida ao guardado, o que
/// é estritamente mais que limpar — manter os dois seria pinar o mesmo fato por
/// duas ancoragens, e a que cita `clear()` descreveria um mecanismo que já não
/// existe.
///
/// ⚠️ **Ele é a metade que torna o descarte reversível**, e sem ele o
/// `RestoreRun` seria um verbo pintado, clicável e inerte: a corrida ficaria
/// guardada na sessão e **inalcançável**, que é o mesmo que perdida.
///
/// A troca é `mem::take` nos DOIS sentidos — descartar move a fita viva para o
/// guardado, devolver move de volta — e é isso que faz o ciclo de vida ser
/// **derivado** em vez de mantido: nunca há duas corridas ao mesmo tempo.
#[test]
fn the_discard_stashes_and_the_restore_brings_it_back() {
    let arm = player_edit_arm();
    assert!(
        arm.contains("self.discarded_run = std::mem::take(&mut self.player_tape)"),
        "descartar tem de GUARDAR a corrida, nao apaga-la. Braco:\n{arm}"
    );
    assert!(
        arm.contains("self.player_tape = std::mem::take(&mut self.discarded_run)"),
        "o `RestoreRun` nao devolve a corrida guardada. Braco:\n{arm}"
    );
}

/// **E o `RestoreRun` também NÃO cai no fan-out** — o irmão exato do gate acima.
///
/// Ele vive no mesmo `if/else if/else`, então a asserção é a mesma propriedade:
/// tudo o que a §14 emite e **não** é uma escrita de componente é interceptado
/// aqui, e só o resto desce para o laço por entidade.
#[test]
fn the_restore_run_arm_does_not_fan_out() {
    let arm = player_edit_arm();
    let at = arm
        .find("RestoreRun")
        .expect("o braco tem de NOMEAR o verbo que devolve a corrida");
    let after = &arm[at..];
    let else_at = after
        .find("} else {")
        .expect("a intercepcao tem de ter um ramo alternativo para o resto da secao");
    assert!(
        !after[..else_at].contains("player_edits.push"),
        "o `RestoreRun` tambem foi empurrado para o fan-out por entidade. Braco:\n{arm}"
    );
}

/// **E o scanner acha alguma coisa** — o controle que impede os dois gates acima
/// de ficarem verdes por não terem lido nada.
#[test]
fn the_scanner_finds_the_arm() {
    let arm = player_edit_arm();
    assert!(
        arm.len() > 200,
        "o scanner leu {} bytes: o braco mudou de forma e os gates acima deixaram de \
         olhar para o produto",
        arm.len()
    );
}

/// O trecho do `snapshots.rs` que deriva os dois números de segundos.
fn run_seconds_block() -> &'static str {
    const SNAP: &str = include_str!("../src/render_loop/snapshots.rs");
    let at = SNAP
        .find("let recorded_run_seconds")
        .expect("o snapshot tem de derivar os segundos de corrida GRAVADA");
    let rest = &SNAP[at..];
    let end = rest
        .find("let inspector_player")
        .expect("o bloco termina onde a §14 e' construida");
    &rest[..end]
}

/// **OS DOIS NÚMEROS DE CORRIDA SAEM DAS DUAS FITAS** (W24).
///
/// ⚠️ **Este gate nasceu de uma mutação SOBREVIVENTE**, e o buraco é de classe:
/// trocar a derivação por `0.0` deixa **toda** a suíte verde — os gates de seam
/// constroem o `InspectorPlayerInfo` à mão, então nenhum deles observa de onde o
/// número vem. O painel decidiria para sempre *"não há corrida descartada"* e o
/// botão de devolver **nunca** apareceria, com o produto compilando (e só um
/// `warning: unused variable` como testemunha, que não é gate).
///
/// A propriedade é **de onde o número vem**, não o seu valor: cada segundo é o
/// comprimento da SUA fita vezes o passo fixo — o `recorded` da viva, o
/// `discarded` da guardada. Trocá-los é o erro que ninguém pega lendo, então os
/// dois são afirmados por nome.
#[test]
fn both_run_readouts_are_derived_from_their_own_tape() {
    let block = run_seconds_block();
    assert!(
        block.contains("(player_tape_ticks as f64 * fixed_dt)"),
        "o readout de corrida GRAVADA nao sai da fita viva. Bloco:\n{block}"
    );
    assert!(
        block.contains("(discarded_run_ticks as f64 * fixed_dt)"),
        "o readout de corrida DESCARTADA nao sai da fita guardada -- o botao de \
         devolver nunca aparece. Bloco:\n{block}"
    );
}

/// **E os dois comprimentos saem das DUAS fitas da shell** — a metade de cima do
/// mesmo fio, no `render_loop`.
///
/// Sem ela, passar `0` no sítio de chamada reproduz o mesmo defeito um nível
/// acima: a derivação continua correta e alimentada por nada.
#[test]
fn the_shell_hands_both_tape_lengths_to_the_snapshot() {
    assert!(
        SRC.contains("self.player_tape.len()"),
        "a shell nao entrega o comprimento da fita viva ao snapshot"
    );
    assert!(
        SRC.contains("self.discarded_run.len()"),
        "a shell nao entrega o comprimento da fita GUARDADA ao snapshot"
    );
}

/// **E o scanner do bloco acha alguma coisa** — o controle positivo dos dois
/// gates acima, sem o qual eles ficariam verdes por não terem lido nada.
#[test]
fn the_run_seconds_scanner_finds_the_block() {
    let block = run_seconds_block();
    assert!(
        block.len() > 60,
        "o scanner leu {} bytes: o bloco mudou de forma",
        block.len()
    );
}
