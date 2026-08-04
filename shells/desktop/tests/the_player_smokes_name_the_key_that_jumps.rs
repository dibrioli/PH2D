//! **Arch-gate: o roteiro de um smoke de player nomeia a tecla que de fato PULA.**
//!
//! ## O defeito (Enio, 2026-08-04)
//!
//! *"O smoke do terminal erra: pular é com a seta e não com espaço."*
//!
//! Três linhas de roteiro mandavam apertar **Espaço** para pular. O `PlayerKeys::key` recusa o
//! Espaço **de propósito**, e o doc-comment dele já dizia por quê: *"o Espaço é o Play/Pause do
//! transporte, e um platformer cujo botão de pulo também pausa a cena é uma tecla com dois donos"*.
//!
//! ⚠️ **O custo não era só ler errado.** Quem seguia a instrução ao pé da letra **PAUSAVA a
//! simulação** no instante em que devia pular — e nas duas cenas onde isso acontecia (`=86` a fita,
//! `=87` o perdão) o que se está julgando é justamente *o que acontece naquele instante*. O roteiro
//! não falhava em silêncio: ele instruía o gesto que destrói a própria medição.
//!
//! ## Por que o gate ata o TEXTO ao KEYMAP em vez de proibir uma palavra
//!
//! Proibir a string `"Espaco"` sozinha seria uma regra sobre prosa, e ela envelheceria no dia em que
//! alguém tornasse o Espaço uma tecla de pulo. O gate afirma as DUAS metades: o keymap **recusa** o
//! Espaço (é ele que torna a proibição correta) e os roteiros **não** o nomeiam. Se o keymap mudar, a
//! primeira asserção cai primeiro e diz que a segunda tem de ser revista.
//!
//! ⚠️ **Lê a FONTE, não chama o produto** — o precedente é o irmão
//! `the_players_finger_reaches_the_bridge`: `PlayerKeys` é `pub(crate)` e um `winit::KeyEvent` não
//! pode ser construído fora do winit, então nada em `tests/` alcança a política por chamada.

const KEYMAP: &str = include_str!("../src/player_input.rs");
const WALK: &str = include_str!("../src/physics_smoke_player.rs");
const TAPE: &str = include_str!("../src/physics_smoke_player_tape.rs");
const FORGIVE: &str = include_str!("../src/physics_smoke_player_forgive.rs");

/// Os arquivos que carregam roteiro de player, com o nome que o erro deve citar.
const SCRIPTS: [(&str, &str); 3] = [
    ("physics_smoke_player.rs", WALK),
    ("physics_smoke_player_tape.rs", TAPE),
    ("physics_smoke_player_forgive.rs", FORGIVE),
];

/// **A metade do KEYMAP** — e ela vem primeiro porque é ela que dá sentido à outra.
///
/// **Mutação que deve sangrar:** acrescentar `KeyCode::Space` ao braço do pulo em
/// `player_input.rs`.
#[test]
fn the_space_bar_is_not_a_jump_key() {
    // ⚠️ Só a metade de PRODUÇÃO. O `mod tests` do próprio `player_input.rs` cita `KeyCode::Space`
    // — para afirmar que ele é RECUSADO —, e varrer o arquivo inteiro faria este gate falhar sobre
    // exatamente o teste que prova o fato que ele quer.
    let keymap = KEYMAP
        .split_once("#[cfg(test)]")
        .map_or(KEYMAP, |(prod, _)| prod);
    assert!(
        !keymap.contains("KeyCode::Space"),
        "a entrada do player passou a reconhecer o Espaco — se isso e' deliberado, o roteiro dos \
         smokes tem de ser revisto junto (e este gate com ele)"
    );
    // Controle positivo: o braço do pulo existe e nomeia as teclas que de fato pulam. Sem ele a
    // asserção acima passaria num arquivo que alguém esvaziou.
    assert!(
        keymap.contains("KeyCode::ArrowUp | KeyCode::KeyZ"),
        "o braco do pulo tem de existir e ser a seta para cima (ou Z)"
    );
}

/// **A metade do TEXTO** — nenhum roteiro manda apertar a tecla que o keymap recusa.
///
/// **Mutação que deve sangrar:** trocar qualquer `SETA PARA CIMA` de volta por `Espaco` num dos três
/// roteiros.
#[test]
fn no_player_script_tells_the_artist_to_press_space() {
    for (name, src) in SCRIPTS {
        // A menção NEGADA é legítima e é a que ensina — o roteiro pode (e deve) dizer que o Espaço
        // NÃO pula. O que se proíbe é mandar apertá-lo.
        for line in src.lines() {
            let l = line.trim();
            if !l.contains("Espaco") {
                continue;
            }
            assert!(
                l.contains("NAO"),
                "o roteiro `{name}` cita o Espaco sem negar que ele pula:\n  {l}\n\
                 O Espaco e' o Play/Pause do transporte; quem segue essa instrucao PAUSA a cena no \
                 instante em que devia pular."
            );
        }
    }
}

/// Controle positivo do scanner: toda cena que manda PULAR nomeia a tecla certa.
#[test]
fn every_script_that_asks_for_a_jump_names_the_arrow() {
    for (name, src) in SCRIPTS {
        if !src.contains("pul") {
            continue; // este roteiro não fala de pulo — nada a exigir
        }
        assert!(
            src.contains("SETA PARA CIMA"),
            "o roteiro `{name}` fala de pulo e nao nomeia a tecla que pula"
        );
    }
}
