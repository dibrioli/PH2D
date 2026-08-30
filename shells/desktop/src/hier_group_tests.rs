//! Os gates da lei do sujeito e das frases (plano de Grupos, 2026-08-30).
//!
//! ⚠️ Estes gates são **puros de propósito**: a mutação vai por portas que já têm gates próprios
//! (`vec_entities::group_entities_nests_and_accepts_mixed_types`), e o que esta wave acrescentou
//! foi a **decisão** — sobre quem o verbo age, e o que ele diz. É essa que não tinha dono.

use super::{Outcome, Subject, subject};

/// ⭐ **O caso normal: o clique DENTRO da selecção age sobre a selecção inteira.**
#[test]
fn clicking_inside_the_selection_takes_the_whole_selection() {
    assert_eq!(subject(7, &[3, 7, 9]), Subject::These(vec![3, 7, 9]));
}

/// ⭐⭐ **A ORDEM é a da selecção, não a do clique** — ela vira a ordem de z dentro do grupo.
#[test]
fn the_order_is_the_selections_not_the_clicks() {
    let Subject::These(v) = subject(9, &[3, 7, 9]) else {
        panic!("devia agir");
    };
    assert_eq!(
        v,
        vec![3, 7, 9],
        "a linha clicada saltou para a frente - o `Children` preserva a ordem de insercao, entao \
         isto trocaria a pilha de z do artista por causa de ONDE ele carregou"
    );
}

/// ⛔⛔ **O clique FORA de uma selecção múltipla não age — ORIENTA.**
///
/// Agir sobre a união traria para dentro do grupo um objecto que o artista não escolheu; agir só
/// sobre a linha faria o verbo falhar por ter um sujeito só. ⚠️ É a lei que o *Merge Sprites* já
/// tinha, e segui-la é o que impede duas respostas para a mesma pergunta.
#[test]
fn clicking_outside_a_multi_selection_steers_instead_of_guessing() {
    assert_eq!(subject(1, &[3, 7, 9]), Subject::ClickedOutsideSelection);
}

/// ⚠️ **Com UM só seleccionado, o clique noutra linha vale a linha clicada** — e não a união.
///
/// Se o artista tinha um objecto escolhido e carregou com o direito noutro, o que ele apontou foi
/// o segundo. Unir os dois agruparia por acidente algo que ele nunca pôs junto.
#[test]
fn a_single_selection_does_not_absorb_the_clicked_row() {
    assert_eq!(subject(1, &[3]), Subject::These(vec![1]));
    assert_eq!(subject(1, &[]), Subject::These(vec![1]));
}

/// ⚠️ **Cada desfecho diz uma coisa DIFERENTE**, e nenhum é mudo.
///
/// *Um verbo que come o clique em silêncio é pior que um ausente* — é a lei que a própria tabela
/// deste menu declara. Este gate obriga as cinco frases a existir e a serem distintas.
#[test]
fn every_outcome_says_something_and_they_all_differ() {
    let todos = [
        Outcome::Grouped {
            group: 1,
            members: 3,
        },
        Outcome::NeedsTwo,
        Outcome::Ungrouped { groups: 1 },
        Outcome::NotGrouped,
        Outcome::ClickedOutsideSelection,
    ];
    let frases: Vec<String> = todos.iter().map(|o| o.toast().message.clone()).collect();
    for (o, f) in todos.iter().zip(&frases) {
        assert!(!f.trim().is_empty(), "{o:?} nao diz nada ao artista");
    }
    for i in 0..frases.len() {
        for j in (i + 1)..frases.len() {
            assert_ne!(
                frases[i], frases[j],
                "{:?} e {:?} dizem a MESMA coisa - o artista nao consegue distinguir os dois \
                 estados, e um deles fica sem explicacao",
                todos[i], todos[j]
            );
        }
    }
}

/// ⭐ **A frase de sucesso diz a CONTAGEM**, porque é ela que confirma que a conta bateu.
#[test]
fn the_success_line_names_the_count() {
    let m = Outcome::Grouped {
        group: 1,
        members: 3,
    }
    .toast()
    .message;
    assert!(
        m.contains('3'),
        "a frase nao diz quantos objectos entraram: {m}"
    );
    let u = Outcome::Ungrouped { groups: 2 }.toast().message;
    assert!(
        u.contains('2'),
        "a frase nao diz quantos grupos sumiram: {u}"
    );
}
