//! Os gates do [`super`] — a resolução da tabela **nome → acção**, medida caso a caso.
//!
//! ⚠️ **A APLICAÇÃO não se testa aqui**, e é a fronteira do módulo: quem escreve na cena é a
//! shell, porque escrever uma `Visibility` é escrever um componente registado e isso passa pelo
//! ledger de pré-visualização. Aqui prova-se o que a lei DECIDE.

use super::*;
use crate::{StableId, Transform, Visibility, assign_missing_stable_ids};

fn linha(on: &str, target: &str, verb: SignalVerb, arg: &str) -> SignalAction {
    SignalAction {
        on: on.into(),
        target: target.into(),
        verb,
        arg: arg.into(),
    }
}

/// Um mundo com um reactor chamado `nome` e a tabela dada.
fn mundo(nome: &str, tabela: Vec<SignalAction>) -> (World, Entity) {
    let mut w = World::new();
    let e = w
        .spawn((Transform::default(), Name::new(nome), SignalActions(tabela)))
        .id();
    assign_missing_stable_ids(&mut w);
    (w, e)
}

/// **O sinal chega, a acção sai** — o circuito inteiro da resolução, num caso só.
#[test]
fn a_named_signal_produces_the_action_it_names() {
    let (mut w, e) = mundo("Porta", vec![linha("botao", "", SignalVerb::Show, "")]);
    let out = resolve(&mut w, &["botao"]);
    assert_eq!(out.len(), 1, "a accao nao saiu");
    assert_eq!(out[0].target, e, "o alvo vazio nao caiu em quem reagiu");
    assert_eq!(out[0].source, e);
    assert_eq!(out[0].verb, SignalVerb::Show);
}

/// ⛔ **Um sinal que ninguém nomeia não faz nada** — e uma linha com o nome VAZIO nunca dispara.
///
/// ⚠️ As duas metades no mesmo gate porque são a mesma lei vista dos dois lados: *um consumidor sem
/// nome não escuta, em vez de escutar tudo* — o espelho exacto da lei do produtor.
///
/// **Mutação que deve sangrar:** apagar o `action.on.is_empty()` da guarda.
#[test]
fn an_unnamed_row_never_fires_and_neither_does_an_unheard_signal() {
    let (mut w, _) = mundo(
        "Porta",
        vec![
            linha("", "", SignalVerb::Show, ""),
            linha("botao", "", SignalVerb::Hide, ""),
        ],
    );
    assert!(
        resolve(&mut w, &["outro"]).is_empty(),
        "um sinal que a tabela nao nomeia produziu accao"
    );
    let out = resolve(&mut w, &["botao"]);
    assert_eq!(out.len(), 1, "a linha SEM NOME disparou junto com a outra");
    assert_eq!(out[0].verb, SignalVerb::Hide);
}

/// ⭐⭐ **O alvo é o NOME, e ele encontra OUTRO objecto.**
///
/// ⚠️ É a lei do CLAUDE.md §5 — *referência durável entre objetos é o NOME, nunca os bits* —, e o
/// que este gate prende é que ela chega ao produto: o undo respawna tudo com bits novos, e uma
/// tabela que guardasse bits ficaria a apontar para o vazio no primeiro `Ctrl+Z`.
#[test]
fn a_named_target_reaches_another_object() {
    let mut w = World::new();
    let parede = w
        .spawn((
            Transform::default(),
            Name::new("Parede"),
            Visibility::visible(),
        ))
        .id();
    let porta = w
        .spawn((
            Transform::default(),
            Name::new("Porta"),
            SignalActions(vec![linha("botao", "Parede", SignalVerb::Hide, "")]),
        ))
        .id();
    assign_missing_stable_ids(&mut w);

    let out = resolve(&mut w, &["botao"]);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].target, parede, "a accao nao alcancou o alvo nomeado");
    assert_eq!(out[0].source, porta, "quem REAGIU perdeu-se");
}

/// ⛔ **Um alvo que não existe é SILÊNCIO — nunca «cai em quem reagiu».**
///
/// ⚠️ É a decisão que o doc do [`resolve`] nomeia: um *fallback* para o próprio objecto faria uma
/// porta esconder-se a si mesma no dia em que alguém renomeasse a parede, e o artista leria isso
/// como um defeito do motor.
///
/// **Mutação que deve sangrar:** trocar o `let Some(target) = … else { continue }` por
/// `unwrap_or(source)`.
#[test]
fn a_target_that_does_not_exist_produces_nothing_at_all() {
    let (mut w, _) = mundo(
        "Porta",
        vec![linha("botao", "NaoExiste", SignalVerb::Hide, "")],
    );
    assert!(
        resolve(&mut w, &["botao"]).is_empty(),
        "um alvo inexistente caiu em quem reagiu — a porta esconder-se-ia a si mesma"
    );
}

/// **A ordem entre reactores é a da IDENTIDADE, nunca a da query.**
///
/// ⚠️ Duas entidades que reajam ao mesmo sinal e escrevam no mesmo alvo têm de o fazer sempre na
/// mesma ordem, senão o replay diverge. A ordem da query é a do arquétipo, que **muda quando um
/// componente é inserido** — e é por isso que este gate insere um componente no meio.
///
/// **Mutação que deve sangrar:** apagar o `reactors.sort_unstable_by_key`.
#[test]
fn the_order_between_reactors_is_the_identity_not_the_query() {
    let mut w = World::new();
    let mut ids = Vec::new();
    for n in ["A", "B", "C"] {
        ids.push(
            w.spawn((
                Transform::default(),
                Name::new(n),
                SignalActions(vec![linha("s", "", SignalVerb::Show, "")]),
            ))
            .id(),
        );
    }
    assign_missing_stable_ids(&mut w);
    let esperado: Vec<Entity> = {
        let mut v: Vec<(u64, Entity)> = ids
            .iter()
            .map(|&e| (w.get::<StableId>(e).expect("id").0, e))
            .collect();
        v.sort_unstable_by_key(|(id, _)| *id);
        v.into_iter().map(|(_, e)| e).collect()
    };

    let antes: Vec<Entity> = resolve(&mut w, &["s"])
        .into_iter()
        .map(|f| f.source)
        .collect();
    assert_eq!(antes, esperado, "a ordem nao e' a da identidade");

    // Mover a entidade do MEIO para outro arquétipo — a ordem da query muda, a da lei não.
    w.entity_mut(ids[1]).insert(Visibility::hidden());
    let depois: Vec<Entity> = resolve(&mut w, &["s"])
        .into_iter()
        .map(|f| f.source)
        .collect();
    assert_eq!(
        depois, esperado,
        "inserir um componente mudou a ordem das accoes — o replay divergiria"
    );
}

/// **Dentro de uma entidade, a ordem é a que o ARTISTA escreveu.**
///
/// ⚠️ Ela é visível na lista do painel, e reordená-la seria o painel a mentir sobre o que acontece.
#[test]
fn within_one_object_the_order_is_the_authored_one() {
    let (mut w, _) = mundo(
        "Porta",
        vec![
            linha("s", "", SignalVerb::Hide, ""),
            linha("s", "", SignalVerb::Show, ""),
        ],
    );
    let verbos: Vec<SignalVerb> = resolve(&mut w, &["s"])
        .into_iter()
        .map(|f| f.verb)
        .collect();
    assert_eq!(verbos, vec![SignalVerb::Hide, SignalVerb::Show]);
}

/// **Vários sinais no mesmo quadro disparam as várias linhas** — e uma tabela vazia é ignorada
/// sem custo.
#[test]
fn several_signals_in_one_frame_fire_their_own_rows() {
    let mut w = World::new();
    w.spawn((
        Transform::default(),
        Name::new("Vazia"),
        SignalActions::default(),
    ));
    w.spawn((
        Transform::default(),
        Name::new("Porta"),
        SignalActions(vec![
            linha("abre", "", SignalVerb::Show, ""),
            linha("fecha", "", SignalVerb::Hide, ""),
            linha("nunca", "", SignalVerb::ToggleVisibility, ""),
        ]),
    ));
    assign_missing_stable_ids(&mut w);
    let verbos: Vec<SignalVerb> = resolve(&mut w, &["abre", "fecha"])
        .into_iter()
        .map(|f| f.verb)
        .collect();
    assert_eq!(verbos, vec![SignalVerb::Show, SignalVerb::Hide]);
}

/// **Sem sinais neste quadro, a resolução não toca no mundo** — é o caso de quase todo quadro.
#[test]
fn a_frame_with_no_signals_resolves_to_nothing() {
    let (mut w, _) = mundo("Porta", vec![linha("s", "", SignalVerb::Show, "")]);
    assert!(resolve(&mut w, &[]).is_empty());
}

/// ⭐ **O `arg` só é lido pelos verbos que o usam**, e a lista é DERIVADA do verbo.
///
/// ⚠️ Um painel que mostra um campo que o verbo não lê é um controlo morto; um que o esconde onde o
/// verbo o lê é uma feature inalcançável. Este gate é o que impede as duas metades de divergirem.
#[test]
fn only_the_timer_verbs_read_the_argument() {
    for v in SignalVerb::ALL {
        let esperado = matches!(v, SignalVerb::StartTimer | SignalVerb::StopTimer);
        assert_eq!(
            v.uses_arg(),
            esperado,
            "{}: uses_arg discorda da familia do verbo",
            v.label()
        );
    }
}

/// **A POSIÇÃO no array É a tag** — a ida-e-volta que o painel usa nos segmentados.
///
/// ⚠️ Reordenar [`SignalVerb::ALL`] faria um clique escrever outro verbo, **e compila**. Este gate
/// é o que o transforma num vermelho.
#[test]
fn the_verb_tag_is_its_position_and_it_round_trips() {
    for (i, v) in SignalVerb::ALL.iter().enumerate() {
        assert_eq!(
            usize::from(v.tag()),
            i,
            "{}: a tag nao e' a posicao",
            v.label()
        );
        assert_eq!(SignalVerb::from_tag(v.tag()), *v, "a ida-e-volta partiu-se");
    }
    // Uma tag fora da faixa cai no primeiro, nunca estoura: ela pode vir de um ficheiro velho.
    assert_eq!(SignalVerb::from_tag(200), SignalVerb::default());
}

/// **Todo verbo tem rótulo, e nenhum se repete** — a lista é o que o artista vê.
#[test]
fn every_verb_has_a_distinct_label() {
    let mut vistos = std::collections::BTreeSet::new();
    for v in SignalVerb::ALL {
        assert!(!v.label().is_empty(), "um verbo sem rotulo");
        assert!(
            vistos.insert(v.label()),
            "dois verbos com o rotulo '{}'",
            v.label()
        );
    }
}

/// ⭐⭐ **ARRANCAR é do princípio; PARAR guarda o progresso.**
///
/// ⚠️ As duas metades no mesmo gate porque a diferença entre elas é a lei: o `start` é o
/// `Timer.start()` do Godot (reinicia) e o `stop` é o *Pause* dele (guarda). Confundi-las tornaria
/// *parar e voltar a arrancar* indistinguível de *parar*.
#[test]
fn starting_rewinds_and_stopping_keeps_the_progress() {
    let mut s = crate::TimerState {
        elapsed_us: 700_000,
        running: true,
    };
    crate::timer::stop(&mut s);
    assert!(!s.running, "parar nao parou");
    assert_eq!(s.elapsed_us, 700_000, "parar ZEROU o progresso");
    crate::timer::start(&mut s);
    assert!(s.running, "arrancar nao arrancou");
    assert_eq!(s.elapsed_us, 0, "arrancar nao voltou ao principio");
}
