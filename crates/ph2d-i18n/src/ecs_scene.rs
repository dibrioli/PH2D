//! ⭐⭐ **AS PALAVRAS DO MOTOR DA CENA** (`ph2d-ecs`) — a 7.ª fatia da fronteira dos motores, e a
//! primeira achada por um INSTRUMENTO em vez de por uma fotografia do dono.
//!
//! # O que estava cru, e porque nenhuma régua o via
//!
//! O `ph2d-ecs` publicava **21** palavras por `fn label()`. Nenhuma das 30 réguas lexicais o
//! varre (elas varrem os painéis, as `ph2d-app-*`, a `ph2d-editor-core` e a shell) e a régua de
//! porta segue o texto até um pintor **dentro da mesma crate** — e aqui o pintor é a shell. Quem
//! as enumerou foi a régua nova (`ph2d_label_census::fronteira`), com o gate
//! `a_fronteira_dos_motores` a guardá-las.
//!
//! # ⛔⛔ E das 21, só ONZE eram dívida — as outras dez eram ÓRFÃS
//!
//! `BlendMode::label` (6), `AnimDirection::label` (4) e `BoundProp::label` (5) **não tinham um
//! único consumidor de produto**: a sonda renomeou as três e a workspace INTEIRA compilou. Quem
//! pinta esses três vocabulários é o Inspector (e o painel de Vector), cada um com a tabela de
//! CHAVES dele, **de propósito**, porque nenhum deles depende desta crate — o snapshot leva só o
//! `tag()`. ⇒ traduzi-las criaria uma **segunda** chave para a mesma palavra.
//!
//! ⚠️ *A terceira espécie do `CLAUDE.md` §5.0: um órfão (cura: apagar) lê-se exactamente como um
//! morto (cura: ligar), e a régua não os separa — quem julga é quem lê a lista.* As três foram
//! apagadas, com a prova escrita ao lado de cada uma.
//!
//! # As onze que ficam
//!
//! Os **4 barramentos de áudio** e os **7 verbos de sinal**, os dois pintados pela shell ao montar
//! o snapshot do Inspector (`render_loop/inspector_audio.rs` · `ph2d-app-components::signal_actions_inspector`).
//!
//! ⚠️ **A chave deriva da VARIANTE, nunca da palavra inglesa** — a forma forte. Uma ponte que casa
//! por palavra daria ao `Master` do barramento a palavra de qualquer outro *Master* do app, que é
//! exactamente a colisão que o painel de Vector teve de partir em cinco famílias.

/// A tradução de uma chave `ecs.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ph2d-migrar-texto:begin
        "ecs.audio_bus.master" => "Master",
        "ecs.audio_bus.music" => "Music",
        // ⚠️ **`SFX` é uma sigla e fica assim em toda língua** — como `RGB` ou `UV`. Uma tradução
        // que a expanda parte a coluna do selector, que foi medida para quatro entradas curtas.
        "ecs.audio_bus.sfx" => "SFX",
        "ecs.audio_bus.voice" => "Voice",
        "ecs.signal_from.anyone" => "Anyone",
        "ecs.signal_from.myself" => "Myself",
        "ecs.signal_verb.add_to_counter" => "Add to Counter",
        "ecs.signal_verb.damage" => "Damage",
        "ecs.signal_verb.destroy" => "Destroy",
        "ecs.signal_verb.heal" => "Heal",
        "ecs.signal_verb.hide" => "Hide",
        "ecs.signal_verb.play_sound" => "Play Sound",
        "ecs.signal_verb.restart_run" => "Restart Run",
        "ecs.signal_verb.show" => "Show",
        "ecs.signal_verb.start_timer" => "Start Timer",
        "ecs.signal_verb.stop_sound" => "Stop Sound",
        "ecs.signal_verb.stop_timer" => "Stop Timer",
        "ecs.signal_verb.toggle_visibility" => "Toggle Visibility",
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
