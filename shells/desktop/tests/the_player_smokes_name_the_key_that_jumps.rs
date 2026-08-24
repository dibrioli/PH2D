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

// ⚠️ **A constante `KEYMAP` foi REMOVIDA em 2026-08-24.** Ela lia o `player_input.rs` à procura
// das teclas cravadas do jogador; a W5 do plano 30 tirou-as de lá (quem as declara é o mapa de
// fábrica) e o gate que a usava passou a **medir comportamento** em vez de texto. Deixá-la aqui
// seria um `include_str!` de um ficheiro que ninguém lê -- e um dia alguém escreveria um gate
// novo sobre ela, a afirmar sobre um sítio onde a lei já não mora.
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
/// ⚠️⚠️ **RE-ANCORADA em 2026-08-24: a lei mudou de ENDEREÇO, não de valor.** Até aqui ela varria
/// o `player_input.rs` à procura de `KeyCode::ArrowUp | KeyCode::KeyZ`, porque as teclas do jogador
/// viviam cravadas lá. A W5 do plano 30 tirou-as de lá: quem as declara agora é o **mapa de
/// fábrica** (`ph2d_input::InputMap::with_player_defaults`), e o artista pode remapeá-las na janela
/// flutuante.
///
/// ⇒ o gate passa a **medir o comportamento** em vez de ler texto, o que é mais forte: um scanner
/// verifica que uma frase existe; isto verifica que a ligação existe. *Um gate ancorado na
/// implementação expira no dia em que a lei se muda de sítio — e o que se faz com ele é
/// RE-ANCORAR, nunca apagar.*
///
/// **Mutação que deve sangrar:** acrescentar a ligação do Espaço ao `jump` do mapa de fábrica.
#[test]
fn the_space_bar_is_not_a_jump_key() {
    use ph2d_input::{Binding, InputMap, Key};
    /// O Espaço no espaço de keycode normalizado da shell.
    const SPACE: Key = Key(0x20);
    /// A seta para cima e o `Z` — as duas que de facto pulam.
    const UP: Key = Key(0xF700);
    const Z: Key = Key(0x5A);

    let m = InputMap::with_player_defaults();
    let jump = m.id("jump").expect("o mapa de fabrica declara `jump`");
    let b = &m.get(jump).expect("existe").bindings;

    assert!(
        !b.contains(&Binding::Key(SPACE)),
        "o pulo passou a reconhecer o Espaco -- ele e' o Play/Pause do transporte, e um platformer \
         cujo botao de pulo tambem pausa a cena e' uma tecla com dois donos. Se isso e' \
         deliberado, o roteiro dos smokes tem de ser revisto junto (e este gate com ele)"
    );
    // Controle positivo: o pulo EXISTE e nomeia as teclas que de facto pulam. Sem ele a asserção
    // acima passaria sobre uma acção que alguém esvaziou.
    assert!(
        b.contains(&Binding::Key(UP)) && b.contains(&Binding::Key(Z)),
        "o pulo de fabrica tem de ser a seta para cima E o Z -- ficou {b:?}"
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
