//! **O INPUT MAP ATRAVESSA O ARQUIVO** (v96) — os gates da costura.
//!
//! ⚠️ **Por que estes gates são de TEXTO e não de comportamento:** a ida-e-volta *serializada* já é
//! provada em [`super::tests`], que monta um `ProjectFile` **à mão** — e é precisamente isso que
//! deixa um buraco: uma mutação no `project_save.rs` que gravasse um mapa **vazio** em vez do vivo
//! passaria por aquele gate sem o acordar, porque ele nunca chama o save. Um teste de comportamento
//! aqui exigiria uma `App` inteira (GPU, janela, atlas), que é o que o irmão da **fita** também não
//! pôde fazer.
//!
//! ⇒ a forma é a dele, controle positivo incluído: ler a **CONSTRUÇÃO**, e falhar alto se ela mudar
//! de forma. *Um scanner que deixa de encontrar o que mede tem de gritar, não passar.*

/// **O SAVE grava o mapa VIVO da sessão**, e não um mapa vazio.
///
/// ⚠️ **Ancorado na CONSTRUÇÃO, não no nome do campo.** A primeira ocorrência de `input_map:` na
/// família é a **declaração** do struct, e um gate que a lesse ficaria verde afirmando uma coisa
/// sobre a outra — o defeito que o irmão da fita nomeia palavra por palavra.
///
/// ⚠️ **A FAMÍLIA, não um endereço:** o `project.rs` já foi partido **duas** vezes por teto de LOC.
/// O `include_str!` é o controle — se um membro sumir, isto vira **erro de compilação** em vez de um
/// `.expect()` que ninguém sabe ler.
#[test]
fn the_save_writes_the_live_input_map() {
    let src = concat!(include_str!("project.rs"), include_str!("project_save.rs"));
    let at = src
        .find("let file = ProjectFile {")
        .expect("o save constroi um `ProjectFile`");
    let rest = &src[at..];
    let body = &rest[..rest.find("\n        };").expect("a construcao fecha")];
    // O CONTROLE POSITIVO: se a construcao mudar de forma, o scanner le' pouco e este gate tem de
    // gritar em vez de passar a afirmar sobre uma janela vazia.
    assert!(
        body.len() > 200,
        "o scanner leu {} bytes: a construcao mudou de forma e este gate parou de olhar para o \
         produto",
        body.len()
    );
    assert!(
        body.contains("input_map: self") && body.contains("h.input_map.clone()"),
        "o save nao grava o Input Map VIVO da sessao -- o artista autora as accoes e elas nao \
         chegam ao ficheiro. Construcao:\n{body}"
    );
}

/// **O LOAD instala o mapa do arquivo, e INSTALA — nunca funde.**
///
/// ⚠️ Um load é uma **troca de documento**. Fundir o mapa do projecto novo com o da sessão anterior
/// deixaria acções de um jogo a viver dentro de outro — é o mesmo argumento que faz o
/// `project_forget` deitar fora o relógio, a fila de undo e a timeline, e que o irmão da fita
/// escreve como *"uma fita costurada com a da sessão anterior descreveria uma corrida que ninguém
/// deu"*.
#[test]
fn the_load_installs_the_map_and_never_merges_it() {
    let src = include_str!("project_load.rs");
    assert!(
        src.contains("hero.input_map = file.input_map.clone();"),
        "o load nao instala o Input Map do arquivo: abrir um projecto deixaria as accoes da sessao \
         anterior no lugar das dele"
    );
    // ⛔ A forma que NAO pode aparecer: qualquer coisa que ACRESCENTE ao mapa vivo em vez de o
    // substituir. Um `extend`/`insert` sobre o mapa da sessao seria a fusao que este gate proibe.
    assert!(
        !src.contains("hero.input_map.insert(") && !src.contains("hero.input_map.create("),
        "o load esta' a FUNDIR o mapa do arquivo com o da sessao anterior, em vez de o instalar"
    );
    // ⚠️ E o estado RESOLVIDO tem de ser zerado junto: ele guarda um tique atras, e o tique atras
    // de um documento que acabou de fechar e' de outro jogo -- uma borda `just_pressed` fantasma
    // no primeiro quadro depois do load.
    assert!(
        src.contains("self.input_actions = ph2d_input::ActionState::new();"),
        "o load nao zera o estado resolvido: a borda do documento ANTERIOR sobreviveria ao load"
    );
}

/// **O mapa é do PROJECTO, não do `ProjectState`** — e portanto não entra no undo.
///
/// ⚠️ Se ele estivesse dentro do `ProjectState`, cada Ctrl+Z do canvas rebobinaria o mapa de
/// controlos junto — o motivo, palavra por palavra, que mantém `motion`, `timeline`, `physics` e a
/// fita fora dele.
#[test]
fn the_map_is_not_part_of_the_undo_unit() {
    let src = include_str!("undo.rs");
    assert!(
        !src.contains("input_map"),
        "o Input Map entrou no `ProjectState`: um Ctrl+Z do canvas passa a rebobinar os controlos"
    );
}

/// ⛔⛔ **O MAPA TEM UM DONO SÓ.**
///
/// ⚠️ Ele nasceu na `App` (W2) e mudou-se para o `HeroScreen` na W3, quando a janela flutuante
/// mostrou que o pintor só recebe o hero. Guardá-lo nos **dois** seria duas memórias do mesmo
/// facto — e a segunda divergiria no primeiro `load`, com o artista a ver as acções antigas numa
/// janela e o jogo a obedecer às novas.
#[test]
fn the_authored_map_has_exactly_one_holder() {
    let src = include_str!("app_state.rs");
    // ⚠️ **A agulha tem sintaxe que so' o CODIGO tem.** Procurar `input_map` cru acusaria o
    // doc-comment ao lado do `input_actions`, que EXPLICA porque o mapa nao mora aqui -- e' o
    // segundo gate desta linha a quase reprovar sobre PROSA (o primeiro foi removido em 24/08 por
    // acusar um comentario do `keyboard.rs`). *Um gate de texto cuja agulha e' um identificador nu
    // le' documentacao.*
    let declares_field = src
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .any(|l| l.contains("input_map:"));
    assert!(
        !declares_field,
        "a `App` voltou a DECLARAR o Input Map. O dono e' o `HeroScreen` (o pintor da janela so' \
         recebe o hero); o que fica na App e' o RESOLVIDO (`input_actions`), que e' derivado."
    );
    // O controle POSITIVO: se o campo derivado tambem sumir, este gate deixou de olhar para o
    // ficheiro certo e passaria verde a afirmar sobre nada.
    assert!(
        src.contains("input_actions:"),
        "o `input_actions` sumiu da App: ou o desenho mudou, ou este gate esta' a ler o ficheiro \
         errado -- em nenhum dos casos ele esta' a provar que ha' um dono so'"
    );
}
