//! **A JANELA DO INPUT MAP** (plano 30 §0.2) — a janela flutuante que abre sobre o canvas,
//! onde as acções NOMEADAS à la Godot se ligam a uma tecla ou a um comando.
//!
//! ⚠️ **Um corte por ASSUNTO**, como os irmãos `tags.rs`/`factory.rs`/`topdown.rs`: estas
//! chaves viveram no `match` do pai até 2026-09-17, quando ele passou o tecto de 700 LOC. Uma
//! JANELA com plano próprio não é assunto do encaminhador — e um tecto cura-se por CORTE, nunca
//! por uma entrada nova de dívida.
//!
//! ⚠️ **UI em INGLÊS** (decisão do dono), e pela tabela mesmo com uma língua só: uma string
//! literal no pintor é a que ninguém encontra no dia em que a segunda língua entrar.

/// A tradução de uma chave da janela do Input Map, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ⭐ **A JANELA DO INPUT MAP** (plano 30 §0.2) — a janela flutuante que abre sobre o canvas.
        // ⚠️ UI em INGLÊS (feedback do Enio), e via i18n mesmo sendo uma língua só: uma string
        // literal no pintor é a que ninguém encontra no dia em que a segunda língua entrar.
        "input_map.title" => "Input Map",
        "input_map.add" => "Add",
        "input_map.new_name.placeholder" => "New action name",
        // ⛔ As duas frases-guia abaixo NOMEAVAM controlos que nao existem — auditoria 2026-08-24.
        // «above» quando o campo esta em baixo, e «Bind» quando o botao virou um `+` na W6. Um
        // indicador ERRADO e pior que a ausencia dele: o artista procura, nao encontra, e conclui
        // que a feature esta partida.
        // ⚠️ A chave `input_map.listen` (o rótulo «Bind…») MORREU com o botão dela na W6, e sai
        // daqui: uma string órfã é onde alguém escreve, um dia, uma frase sobre um controlo que já
        // não existe — que é exactamente o defeito que esta linha acabou de pagar.
        // ⛔ E a de 24/08 (2ª volta) foi a MESMA doença, na 3ª frase: «at the bottom» deixou de ser
        // verdade quando o campo subiu para o topo, que é onde a referência (Godot) o tem.
        // ⚠️ A `listening` é uma FRASE COMPLETA porque é lida em DOIS sítios — a face vazia da
        // acção e a faixa do título, onde o nome dela vai à frente. Um fragmento («· press a
        // key…») só lê bem num deles.
        "input_map.listening" => "Press a key or a gamepad button. Esc cancels.",
        "input_map.listening.title" => "Listening for",
        "input_map.empty" => "No actions yet. Type a name at the top and press Add.",
        "input_map.binding.key" => "Key",
        "input_map.binding.pad" => "Pad",
        "input_map.binding.axis" => "Axis",
        // ⭐ Os DOIS números que substituem a `deadzone` de duplo propósito do Godot.
        "input_map.no_binding" => "No key yet. Press + on this row, then press a key.",
        "input_map.dead_zone" => "Dead",
        "input_map.press_point" => "Press",
        _ => return None,
    })
}
