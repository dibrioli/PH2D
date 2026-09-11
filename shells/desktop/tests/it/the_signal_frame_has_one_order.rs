//! **A ORDEM do quadro é load-bearing, e nenhum teste de unidade a alcança.**
//!
//! Os cinco passos vivem em linha reta dentro de UMA função do `render_loop`, então a ordem em
//! que aparecem no fonte É a ordem em que rodam:
//!
//! 1. `advance_frame` — o quadro da saída vira, ANTES de qualquer produtor;
//! 2. a TIMELINE publica o que o play cruzou;
//! 3. `physics_bridge::dispatch` — o mundo anda (é ele que PRODUZ os contatos);
//! 4. a FÍSICA publica o que se tocou;
//! 5. o DRENO — os consumidores leem, depois dos dois.
//!
//! ⚠️ **Cada troca de par é um defeito distinto, e nenhum deles falha um teste de unidade:**
//! virar o quadro depois de (2) aposenta o que a timeline acabou de publicar · publicar a
//! física antes de (3) entrega os contatos do quadro ANTERIOR (o comentário do produtor já diz
//! isso: *"um atraso de um quadro é invisível num toast e deixa de ser invisível no dia em que
//! o consumidor for som"*) · drenar antes de (4) faz o sinal da física chegar sempre atrasado.
//!
//! O gate lê o FONTE porque a função exige uma janela e um device — a mesma razão dos arch-gates
//! irmãos desta shell.

use std::path::Path;

/// Os cinco marcos, na ordem em que TÊM de aparecer.
const ORDER: &[(&str, &str)] = &[
    ("self.signals.advance_frame()", "o quadro da saída vira"),
    (
        "ph2d_runtime::Signal::from_timeline(",
        "a TIMELINE publica o que o play cruzou",
    ),
    (
        "physics_bridge::dispatch(",
        "o mundo anda -- é ele que produz os contatos",
    ),
    (
        "ph2d_runtime::Signal::from_contact(",
        "a FÍSICA publica o que se tocou",
    ),
    (
        "self.signals.read(&mut self.signal_toast_reader)",
        "o DRENO: os consumidores leem, depois dos dois",
    ),
];

#[test]
fn the_shell_turns_the_signal_frame_before_it_publishes_and_drains_after_both() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/render_loop/mod.rs");
    let text = std::fs::read_to_string(&src).expect("read render_loop/mod.rs");

    let mut at = Vec::new();
    for (needle, what) in ORDER {
        let hits = text.matches(needle).count();
        // Um marco que aparece duas vezes não tem POSIÇÃO — e o `find` abaixo pegaria a
        // primeira em silêncio, que é como um gate de ordem começa a medir outra função.
        assert_eq!(
            hits, 1,
            "o marco `{needle}` ({what}) aparece {hits}x no render_loop. Um gate de ORDEM só \
             pode falar de um marco que existe uma vez; se o passo virou dois sítios, esta \
             ordem precisa ser re-declarada antes de voltar a valer."
        );
        at.push(text.find(needle).unwrap_or(usize::MAX));
    }

    for w in ORDER.windows(2).zip(at.windows(2)) {
        let ((_, before), (_, after)) = (w.0[0], w.0[1]);
        let (a, b) = (w.1[0], w.1[1]);
        assert!(
            a < b,
            "a ordem do quadro de sinais quebrou: `{before}` tem de vir ANTES de `{after}`, e \
             no fonte vem depois.\n\n\
             Os dois produtores publicam numa saída só e os consumidores leem UMA vez, no fim — \
             é essa ordem que faz a entrega ser no MESMO quadro. Trocar um par não quebra \
             nenhum teste de unidade: quebra a latência, em silêncio."
        );
    }
}
