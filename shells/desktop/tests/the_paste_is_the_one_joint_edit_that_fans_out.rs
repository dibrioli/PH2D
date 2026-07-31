//! **Arch-gate — o Paste da §12 se ESPALHA sobre a seleção, e é o único que se
//! espalha** (W-JointCopy, 2026-07-31).
//!
//! ## O que isto protege
//!
//! A §12 descreve UM joint e edita UM: o comentário do laço de ações diz isso
//! desde o W3 (*"No fan-out either, and for a simpler reason: the section only
//! ever describes one joint object"*). O Paste é a exceção, e a exceção É a
//! feature — sem ela o gesto é *digitar quinze campos, dez vezes*, e o botão que
//! promete `Paste to 10 Joints` no rótulo pousaria num só.
//!
//! E as duas irmãs estruturais continuam recusando o fan-out por motivos que
//! **não valem para o paste**: um `Join` espalhado criaria N joints entre o
//! mesmo par, e um `Bake` espalhado re-simularia a cena inteira N vezes pelos
//! MESMOS números, deixando N passos de undo. Um paste espalhado faz exatamente
//! o que o artista pediu.
//!
//! ## Por que um gate de FONTE
//!
//! O fan-out mora no laço de ações do `render_loop`, que precisa de janela e de
//! um frame inteiro — nenhum teste de unidade o alcança. A metade
//! COMPORTAMENTAL (a porta escreve pela fila, atravessa o que não é joint, é
//! idempotente) está em `render_loop::inspector_joint_paste_tests`; este gate
//! prova que o laço de fato ESPALHA, e que o Copy arma a área de transferência
//! em vez de escrever componente.
//!
//! Mutação: troque o braço do fan-out por um `push` único e a 1ª asserção fica
//! vermelha; faça o Copy escrever um componente em vez de `self.joint_clipboard`
//! e a 3ª fica.

const SRC: &str = include_str!("../src/render_loop/mod.rs");

/// A extensão do `match edit { … }` que roteia as edições da §12 no laço de
/// ações, por CASAMENTO DE CHAVES.
///
/// ⚠️ Não por distância em bytes até um landmark seguinte: o gate irmão do flush
/// já ficou vermelho uma vez por um rename que ele não tinha opinião nenhuma
/// sobre, e a lição está escrita lá — *um landmark de bytes é um proxy que
/// expira; a extensão do próprio bloco é a propriedade*.
fn joint_action_arm() -> &'static str {
    // ⚠️ A âncora TERMINA no `=> {` de propósito: o padrão do braço traz o
    // próprio `{ entity_bits, edit }`, e procurar "a primeira chave depois do
    // head" abriria o DESTRUCTURING em vez do corpo — o casamento fecharia duas
    // palavras adiante e o gate mediria uma janela vazia. (Foi o que ele fez na
    // primeira corrida: `left: 0`.)
    const HEAD: &str = "EditorAction::InspectorJointEdit { entity_bits, edit } => {";
    let head = SRC.find(HEAD).unwrap_or_else(|| {
        panic!(
            "o braço de ação da §12 sumiu do render loop — se ele foi \
                 reestruturado, atualize este gate (e confirme que o Paste ainda \
                 se espalha sobre a seleção)"
        )
    });
    let open = head + HEAD.len() - 1;
    let mut depth = 0usize;
    for (i, c) in SRC[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &SRC[open..open + i];
                }
            }
            _ => {}
        }
    }
    panic!("o braço de ação da §12 nunca fecha");
}

/// **O Paste se espalha sobre `inspector_selection`.**
#[test]
fn the_paste_arm_fans_out_over_the_selection() {
    let arm = joint_action_arm();
    let paste = arm
        .find("JointFieldEdit::PasteProperties")
        .expect("o braço da §12 não menciona o Paste — ele deixou de ter rota");
    let tail = &arm[paste..];
    assert!(
        tail.contains("inspector_selection"),
        "o braço do Paste não lê a seleção: ele pousaria num joint só, e o \
         rótulo `Paste to N Joints` estaria mentindo sobre o que o clique faz"
    );
    assert!(
        tail.contains("for &t in &inspector_selection"),
        "o braço do Paste não ITERA a seleção — ler a lista e usar só o primeiro \
         seria o mesmo defeito com um nome melhor"
    );
}

/// **E o fan-out é EXCLUSIVO dele** — o `Join` e o `Bake` continuam recusando.
///
/// ⚠️ A asserção é sobre o braço da §12: o único `inspector_selection` que ele
/// pode conter é o do Paste. Um segundo fan-out aqui seria uma edição de campo
/// espalhada — `Kind` sobre dez joints, digamos — que é uma feature legítima e
/// **outra decisão**, com o rótulo e a contagem que ela exigiria.
#[test]
fn no_other_joint_edit_fans_out() {
    let arm = joint_action_arm();
    assert_eq!(
        arm.matches("inspector_selection").count(),
        2,
        "o braço da §12 lê a seleção um número de vezes diferente das 2 do \
         Paste (a guarda e o `for`) — se outra edição passou a se espalhar, ela \
         precisa do próprio rótulo dizendo quantos objetos o clique muda"
    );
}

/// **O Copy ARMA a área de transferência da shell** — não escreve componente
/// nenhum, e por isso não passa pela fila nem pelo clamp.
#[test]
fn the_copy_arm_only_arms_the_clipboard() {
    let head = SRC
        .find("JointFieldEdit::CopyProperties")
        .expect("o Copy sumiu do laço de aplicação da §12");
    // A janela é o braço `else if` dele, até o `} else if` seguinte.
    let tail = &SRC[head..];
    let end = tail.find("} else if").unwrap_or(tail.len());
    let body = &tail[..end];
    assert!(
        body.contains("self.joint_clipboard = sim"),
        "o Copy não escreve a área de transferência da shell a partir do mundo \
         — ele é a metade que ARMA, e sem esta linha o Paste seguinte cola o que \
         alguém copiou antes"
    );
    assert!(
        !body.contains("queue_set") && !body.contains("apply_editor_commands"),
        "o Copy está mexendo na fila de comandos: copiar não muda componente \
         nenhum, e uma escrita aqui viraria um passo de undo por CÓPIA"
    );
}
