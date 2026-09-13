//! ⛔⛔ **O GESTO DA PEÇA ACRESCENTADA TEM BRAÇO NO DRENO** (ADR-0164 / F5.11).
//!
//! # Porque este gate é textual, e porque ele é preciso
//!
//! O `match` que consome o `EditorAction` na `render_loop` termina num `_ => {}`: uma acção **nova
//! sem braço compila, corre e não faz nada**. É a primeira das duas espécies de controlo morto que
//! a caça de 2026-08-30 nomeou, e **nenhum gate de registo a apanha** — um seam de painel prova que
//! o clique chega ao *barramento*, nunca que alguém do outro lado o lê. Os irmãos deste ficheiro
//! são o `the_unused_override_gestures_reach_the_verb` e o `the_apply_ladder_has_one_door`.
//!
//! ⚠️ **Ele descasca comentários antes de varrer** — um censo textual que não separa prosa de
//! código mente nos dois sentidos, e esta linha já o pagou.

/// O QUADRO pela ordem em que corre (`frame_text::render_frame`), só com as linhas de CÓDIGO.
///
/// ⚠️ **Desde a OBRA 2 da `line/render-loop` (2026-09-13) o gesto mora em DUAS casas:** o braço do
/// barramento continua no dreno do `render_loop/mod.rs`, e a aplicação da peça mudou-se para a fase
/// `fase_recipe_and_asset_verbs`. Lido só no `mod.rs`, este censo reprovava sobre produto correcto — e é
/// a espécie que avisa: a que fica muda é um censo de AUSÊNCIA a ler um ficheiro de onde o código saiu.
fn frame_code() -> &'static str {
    static CODE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    CODE.get_or_init(|| {
        crate::frame_text::render_frame()
            .lines()
            .map(|l| match l.find("//") {
                Some(i) => &l[..i],
                None => l,
            })
            .collect::<Vec<_>>()
            .join("\n")
    })
}

/// O braço que APLICA a peça: do `if let Some(piece) = apply_added {` à chaveta que o FECHA.
///
/// ⛔⛔ **Era uma janela de 1600 BYTES, e um número de bytes é uma agulha cujo sentido depende do
/// COMPRIMENTO DOS NOMES.** Ao mudar `crate::instance_added::` para `ph2d_app_components::instance_added::`
/// (+17 caracteres por ocorrência) os dois toasts do braço passaram de dentro para fora da janela, e o gate
/// reprovou sobre produto **correcto**. Alargar o número seria pior: medido, havia outro par de toasts a
/// `3573`/`3816` bytes, de um braço VIZINHO — uma janela de `4000` ficaria verde a contar os toasts de outra
/// pessoa.
///
/// ⛔ **E a cura seguinte — a chaveta À INDENTAÇÃO do braço (`\n            }`) — também era um proxy:** ao
/// mudar-se para uma fase o braço desceu de 12 para 8 espaços, e a 1.ª chaveta a 12 passou a ser o `};` do
/// `OwnedDocs` na 3.ª linha do braço — a janela encolheria para antes dos toasts. ⇒ a fatia é **o bloco
/// pelas chavetas**, que não depende nem dos nomes nem da coluna. As chavetas dentro de um literal (os
/// `{name}` e `\u{201c}` dos toasts) não contam.
fn apply_added_arm() -> &'static str {
    const HEAD: &str = "if let Some(piece) = apply_added {";
    let code = frame_code();
    let at = code.find(HEAD).expect("o braco adiado");
    let open = at + HEAD.len() - 1;
    let mut depth = 0usize;
    let mut in_str = false;
    let mut escaped = false;
    for (i, c) in code[open..].char_indices() {
        if in_str {
            match c {
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                '"' => in_str = false,
                _ => {}
            }
            continue;
        }
        match c {
            '"' => in_str = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &code[at..open + i];
                }
            }
            _ => {}
        }
    }
    panic!("o braco da peca acrescentada nunca fecha");
}

/// ⭐⭐⭐ **O botão do cartão chega à porta que põe a peça na receita.**
///
/// **Mutação que deve sangrar:** apagar o braço do `InspectorApplyAddedPiece`, ou fazê-lo chamar
/// outra porta.
#[test]
fn the_apply_added_gesture_has_an_arm_and_its_own_door() {
    let body = frame_code();
    assert!(
        body.contains("EditorAction::InspectorApplyAddedPiece"),
        "a accao nao tem braco no dreno — o `_ => {{}}` do fim do match come-a em silencio"
    );
    assert!(
        body.contains("instance_added::promote("),
        "o braco nao chama a porta que promove a peca — o clique morre a um passo do efeito"
    );
}

/// ⛔⛔ **A CHAVE atravessa: o dreno resolve o `StableId`, e nunca os bits que o cartão viu.**
///
/// Entre o clique e o ponto de aplicação pode ter corrido um Ctrl+Z, que **respawna tudo com bits
/// novos**. Um braço que fizesse `Entity::from_bits` sobre o que o painel mandou apontaria para uma
/// entidade morta — ou, pior, para outra que nasceu no lugar dela.
///
/// **Mutação que deve sangrar:** trocar o `entity_for_stable_id` por um `Entity::from_bits` do
/// campo.
#[test]
fn the_arm_resolves_the_piece_by_identity_not_by_bits() {
    assert!(
        apply_added_arm().contains("entity_for_stable_id("),
        "o braco resolve a peca por outra via que nao a identidade — um Ctrl+Z entre o clique e \
         este ponto troca todos os bits"
    );
}

/// ⛔ **Todo caminho negativo FALA.** Um botão que come o clique em silêncio é pior que um ausente
/// — a lei que o menu dos verbos já paga, e que o report de 2026-09-05 cobrou.
#[test]
fn every_refusal_of_the_apply_added_gesture_has_a_voice() {
    assert!(
        apply_added_arm().matches("Toast::warning").count() >= 2,
        "as recusas do gesto nao falam — `NotAdded` e o resto tem de dizer coisas diferentes"
    );
}
