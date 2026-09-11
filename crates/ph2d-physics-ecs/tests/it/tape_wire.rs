//! **A FITA VIAJA NO ARQUIVO** (W17) — a forma de arquivo e a de memória dizem a
//! mesma corrida.
//!
//! O item estava no §4 do plano 06 desde o começo (*"Persistir a fita (W7)"*), e
//! o que o tornou útil foi o **bake da W16**: a fita é a entrada que o bake
//! replaya, então uma corrida que morre com a sessão é uma corrida que só pode
//! ser assada no dia em que foi jogada.

use ph2d_physics_ecs::{InputTape, PlayerInput, PlayerInputAtTick, TapeWire};

/// Uma corrida com os quatro campos MEXENDO — e é isso que a torna um oráculo.
///
/// ⚠️ Uma fita de `PlayerInput::default()` passaria por qualquer tradução, certa
/// ou errada: com tudo em zero, trocar dois bits de lugar dá o mesmo resultado.
fn scripted() -> InputTape {
    let mut t = InputTape::new();
    for k in 7..=97u64 {
        t.record(
            k,
            PlayerInput {
                // Um eixo que varre negativo, zero e positivo.
                drive: (k as f32 - 50.0) / 50.0,
                // Os três botões em períodos DIFERENTES, senão dois bits trocados
                // entre si seriam indistinguíveis.
                jump: k % 2 == 0,
                down: k % 3 == 0,
                dash: k % 5 == 0,
                grab: false,
            },
        );
    }
    t
}

/// **A ida e a volta devolvem a MESMA corrida** — a wave inteira num gate.
///
/// ⚠️ **Tique a tique, e não só o comprimento:** uma tradução que perdesse o
/// `first` daria uma fita do mesmo tamanho descrevendo uma corrida deslocada no
/// tempo, e o bake a replayaria como se tivesse acontecido noutro instante.
///
/// ⚠️ **Mutação medida:** trocar `BIT_DOWN` por `BIT_DASH` no `from_wire` faz 30
/// tiques divergirem; perder o `first` faz o replay começar no tique 0.
#[test]
fn the_wire_form_round_trips_the_run_exactly() {
    let mut before = scripted();
    let wire = before.to_wire();
    let mut after = InputTape::from_wire(&wire);

    assert_eq!(after.len(), before.len(), "a fita mudou de tamanho");
    // ⚠️ O intervalo começa ANTES do primeiro tique gravado e acaba DEPOIS do
    // último: fora do alcance a fita responde `None`, e uma tradução que movesse
    // o `first` seria vista aqui e em lugar nenhum se o laço fosse só 7..=97.
    for k in 0..=110u64 {
        assert_eq!(
            after.input(k),
            before.input(k),
            "o tique {k} volta do arquivo diferente do que entrou"
        );
    }
}

/// **Uma fita VAZIA volta vazia** — o caso que todo projeto sem player carrega.
///
/// ⚠️ Ele não é redundante com o de cima: o `Default` do `TapeWire` é o que o
/// `postcard` produz para o campo de um projeto onde ninguém correu, e uma
/// tradução que inventasse um quadro nele poria uma corrida de um tique em todo
/// arquivo do app.
#[test]
fn an_empty_run_survives_the_round_trip_as_empty() {
    let empty = InputTape::new();
    assert_eq!(empty.to_wire(), TapeWire::default());
    assert!(InputTape::from_wire(&TapeWire::default()).is_empty());
}

/// **Um BIT que este build não conhece é ignorado, não lido como outro botão.**
///
/// ⚠️ É a propriedade pela qual o bitmask existe, e ela tem história: o
/// `PlayerInput` ganhou o `down` na W12 e o `dash` na W14 — dois botões em duas
/// waves. Com quatro `bool`s, o quinto seria um byte novo POR TIQUE, ou seja um
/// bump de schema por botão; num `u8` ele é um bit novo no mesmo byte, e o
/// layout do arquivo não se move.
#[test]
fn an_unknown_button_bit_is_ignored_rather_than_misread() {
    // O que um build FUTURO, com mais um botão, gravaria neste tique.
    //
    // ⚠️ **O bit é o ÚLTIMO do byte, e a escolha é uma cicatriz:** a primeira
    // versão desta fixture usava o bit **3** para dizer *"desconhecido"*, e a
    // W23 reclamou-o para o botão de AGARRAR — o gate ficou vermelho no dia
    // exacto em que a sua premissa deixou de valer, o que é o comportamento
    // certo, mas a lição fica: um oráculo que codifica *"ninguém usa isto"* num
    // número literal expira quando alguém usa. No bit 7 ele só pode expirar
    // quando o byte encher, e nesse dia a conversa é outra (um segundo byte é
    // mudança de FORMATO, não um bit livre).
    let from_the_future = TapeWire {
        first: 1,
        frames: vec![(0.5, 0b1000_0001)],
    };
    let mut tape = InputTape::from_wire(&from_the_future);
    let got = tape.input(1).expect("o tique 1 esta' na fita");
    assert_eq!(
        got,
        PlayerInput {
            drive: 0.5,
            jump: true,
            down: false,
            dash: false,
            grab: false,
        },
        "um bit desconhecido foi lido como um botao que este build tem"
    );
}
