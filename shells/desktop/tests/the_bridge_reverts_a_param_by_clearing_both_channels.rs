//! **Arch-gate: a ponte consome o intent de reverter, e limpa os DOIS canais.**
//!
//! O braço vive dentro de `apply_param_edits`, que exige um `WidgetStore` vivo e a seleção do
//! frame — nenhum teste headless o alcança. Sem este gate, um botão que emite um intent que
//! ninguém consome passa nas outras três condições de UI e some em silêncio.
//!
//! ⚠️ E ele afirma **as duas chamadas**, não uma: um param viaja por UM canal, mas qual é
//! conhecimento que apodrece (a §5 registra dois params migrando de canal). Meia limpeza
//! deixaria a seta a reverter curvas e gradientes e a não reverter sliders — ou o inverso —,
//! e o modo de falha é uma seta que às vezes não faz nada.

const SRC: &str = include_str!("../src/render_loop/motion_bridge_params_edit.rs");

/// O corpo do braço `MotionParamIntent::ResetParam` do `match` de intents.
fn reset_arm() -> &'static str {
    let start = SRC
        .find("MotionParamIntent::ResetParam")
        .expect("o braço do intent de reverter tem de existir no `match` da ponte");
    // Até o começo do braço seguinte, ou o fim do arquivo — a janela é o braço.
    let rest = &SRC[start..];
    let end = rest
        .find("\n            }\n            }")
        .unwrap_or(rest.len().min(1200));
    &rest[..end]
}

#[test]
fn the_reset_arm_clears_both_the_number_and_the_text_channel() {
    let arm = reset_arm();
    assert!(
        arm.contains("clear_param("),
        "o braço tem de limpar o override de f32 — sem isto a seta é decoração:\n{arm}"
    );
    assert!(
        arm.contains("clear_text_param("),
        "e o de TEXTO (curva / gradiente / paleta / fórmula), pelo mesmo nome:\n{arm}"
    );
}

/// **E não devolve o default ESCREVENDO-o.**
///
/// A tentação mecânica é `set_param(nid, param, default)`. Ela parece igual na tela do dia e
/// deixa um override que por acaso vale o default — um número fossilizado no dia em que o nó
/// mudar de default. O gate afirma a ausência da forma errada dentro do braço.
#[test]
fn the_reset_arm_does_not_write_the_default_back() {
    let arm = reset_arm();
    assert!(
        !arm.contains("set_param("),
        "reverter é REMOVER a chave, nunca reescrevê-la com o default:\n{arm}"
    );
    assert!(!arm.contains("set_text_param("), "idem no canal de texto");
}

/// **Controle positivo:** a janela do braço é de fato o braço, e não o arquivo inteiro.
///
/// Sem isto os dois gates acima passariam por acidente — `clear_param` aparece no arquivo de
/// qualquer jeito, e uma janela larga demais engoliria os braços vizinhos (que CHAMAM
/// `set_param`, e fariam o gate de ausência falhar por outro motivo, ou passar por outro).
#[test]
fn the_window_is_the_arm_and_not_the_whole_file() {
    let arm = reset_arm();
    assert!(
        arm.len() < SRC.len() / 2,
        "a janela ({}) não pode ser metade do arquivo ({})",
        arm.len(),
        SRC.len()
    );
    assert!(
        SRC.contains("set_param("),
        "o ARQUIVO chama set_param noutros braços — é isso que torna a ausência dentro do \
         braço uma afirmação, e não um vácuo"
    );
}
