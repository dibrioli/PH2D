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

/// **O fonte sem os COMENTÁRIOS** — o que uma asserção NEGATIVA tem de ler.
///
/// ⚠️ **Duas asserções deste arquivo nasceram vermelhas sobre código correto**,
/// porque a prosa que explica *por que a troca mora numa porta* cita o
/// `mem::take` que ela proíbe. É a mesma doença que o `{stamp_avg:` do Painter
/// mediu por outro lado: *um oráculo que casa com a documentação de si mesmo não
/// está a olhar para o produto*. Uma asserção de PRESENÇA sobrevive a isso por
/// acidente; uma de AUSÊNCIA não pode.
fn code_only(src: &str) -> String {
    src.lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

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

/// **DESCARTAR ESVAZIA A FITA — GUARDANDO-A, E PELA PORTA** (W17 · W24 · W25).
///
/// ⚠️ Este gate **substituiu** o `the_clear_run_arm_clears_the_tape`, que
/// afirmava o literal `self.player_tape.clear()`. O verbo não mudou de sentido:
/// a troca esvazia a fita viva **e** entrega a corrida ao guardado, o que é
/// estritamente mais que limpar — manter os dois seria pinar o mesmo fato por
/// duas ancoragens, e a que cita `clear()` descreveria um mecanismo que já não
/// existe.
///
/// ⚠️ **E desde a W25 ele afirma a PORTA, não o `mem::take`.** A corrida ganhou
/// uma segunda VISTA (o painel de MUNDO), e duas cópias da troca fariam a mesma
/// coisa hoje: é essa forma que apodrece no dia em que o descarte ganhar um caso
/// especial. O gate irmão abaixo afirma a outra ponta do mesmo fio.
///
/// ⚠️ **Ele é a metade que torna o descarte reversível**, e sem ele o
/// `RestoreRun` seria um verbo pintado, clicável e inerte: a corrida ficaria
/// guardada na sessão e **inalcançável**, que é o mesmo que perdida.
#[test]
fn the_discard_stashes_and_the_restore_brings_it_back() {
    let arm = player_edit_arm();
    assert!(
        arm.contains("run_stash::apply(") && arm.contains("RunVerb::Discard"),
        "descartar tem de atravessar a porta unica (`run_stash`). Braco:\n{arm}"
    );
    assert!(
        arm.contains("RunVerb::Restore"),
        "o `RestoreRun` nao devolve a corrida guardada. Braco:\n{arm}"
    );
    assert!(
        !code_only(arm).contains("mem::take"),
        "a §14 voltou a fazer a troca a mao -- a porta existe porque o painel de \
         MUNDO e' a segunda vista dela. Braco:\n{arm}"
    );
}

/// **A SEGUNDA VISTA ATRAVESSA A MESMA PORTA** (W25).
///
/// ⚠️ **A corrida gravada tem duas vistas e um só dono.** A §14 do Inspector é
/// por-ENTIDADE — o `build_player_info` devolve `None` para tudo o que não é um
/// corpo Dynamic —, mas a fita é do DOCUMENTO e sobrevive ao player que a
/// gravou: apagar o personagem prendia a corrida no arquivo, ainda a ser o que o
/// Bake replaya, sem gesto nenhum que a alcançasse.
///
/// O que este gate impede é a cura barata e errada: duas cópias do `mem::take`,
/// uma por vista. Elas fariam a mesma coisa hoje.
#[test]
fn the_world_panel_route_goes_through_the_same_door() {
    const BRIDGE: &str = include_str!("../src/render_loop/physics_panel_bridge.rs");
    for (intent, verb) in [
        ("PhysicsIntent::ClearRun", "RunVerb::Discard"),
        ("PhysicsIntent::RestoreRun", "RunVerb::Restore"),
    ] {
        let at = BRIDGE
            .find(intent)
            .unwrap_or_else(|| panic!("a ponte do painel de mundo tem de honrar {intent}"));
        let arm = &BRIDGE[at..(at + 200).min(BRIDGE.len())];
        assert!(
            arm.contains("run_stash::apply(") && arm.contains(verb),
            "{intent} nao atravessa a porta unica. Braco:\n{arm}"
        );
    }
    assert!(
        !code_only(BRIDGE).contains("mem::take"),
        "a ponte do painel de mundo faz a troca a mao -- a porta e' o que impede as \
         duas vistas de divergirem"
    );
}

/// **E os dois números que ela publica saem das DUAS fitas** — a metade de cima
/// do fio, no painel de mundo.
///
/// O irmão exato do `both_run_readouts_are_derived_from_their_own_tape` da §14,
/// e existe pela mesma razão medida: trocar uma derivação por `0.0` deixa toda a
/// suíte verde, porque os gates de seam constroem o snapshot à mão.
#[test]
fn the_world_panel_publishes_both_run_lengths() {
    const BRIDGE: &str = include_str!("../src/render_loop/physics_panel_bridge.rs");
    assert!(
        BRIDGE.contains("recorded_run_seconds: (run.live.len()"),
        "o readout de corrida GRAVADA do painel de mundo nao sai da fita viva"
    );
    assert!(
        BRIDGE.contains("discarded_run_seconds: (run.stash.len()"),
        "o readout de corrida DESCARTADA do painel de mundo nao sai do guardado"
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
