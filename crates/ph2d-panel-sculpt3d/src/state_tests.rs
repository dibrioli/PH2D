//! **OS GATES DO ESTADO DO PAINEL** — irmão (`#[path]`, `cfg(test)`) do
//! [`super`], no molde do `preview_tests`.

use super::*;

/// **O PAINEL ABRE CADA VERBO NUM MODO QUE O DECLARA** — e pela MESMA porta que
/// a shell.
///
/// ⚠️ **Este `Default` já estava certo e não era o que shipava.** Ele derivava a
/// resposta num laço inline enquanto o `sculpt3d_birth.rs` da shell escrevia um
/// `S` chapado; duas respostas para uma pergunta, e a que ganhava era a que
/// ninguém tinha escrito de propósito. O gate afirma as duas metades: que todo
/// verbo abre num modo **oferecido**, e que o número é o da porta
/// ([`RefMode::birth_for`]) e não uma segunda derivação que possa divergir dela.
#[test]
fn the_panel_opens_every_verb_in_a_mode_that_declares_it() {
    let ui = Sculpt3dUi::default();
    for v in Verb::ALL {
        let m = ui.mode_of(v);
        assert!(
            m.declares(v),
            "{v:?} abre em {m:?}, que não o declara — a faixa de modo nasce com \
             NENHUM chip aceso, porque o painel só pinta os oferecidos"
        );
        assert_eq!(
            m,
            RefMode::birth_for(v),
            "{v:?}: o painel e a porta têm de dar o mesmo número"
        );
    }
    // CONTROLE POSITIVO: a lista não pode ser vazia nem uniforme por acidente —
    // se todo verbo abrisse no mesmo modo, as duas asserções acima passariam
    // sobre um `Default` que não deriva coisa nenhuma.
    assert_eq!(ui.slots.len(), Verb::ALL.len());
    assert!(
        Verb::ALL
            .iter()
            .any(|&v| ui.mode_of(v) != ui.mode_of(Verb::ALL[0])),
        "sete verbos são do Blender e o resto do SculptGL: a lista TEM de ter \
         dois modos diferentes, senão ela não está derivando nada"
    );
}
