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
        target_by: SignalTarget::Named,
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
    let out = resolve(&mut w, &TagTree::new(), &["botao"]);
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
        resolve(&mut w, &TagTree::new(), &["outro"]).is_empty(),
        "um sinal que a tabela nao nomeia produziu accao"
    );
    let out = resolve(&mut w, &TagTree::new(), &["botao"]);
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

    let out = resolve(&mut w, &TagTree::new(), &["botao"]);
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
/// **Mutação que deve sangrar:** o braço `Named` do [`targets_of`] a cair em `source` quando o nome
/// não casa.
#[test]
fn a_target_that_does_not_exist_produces_nothing_at_all() {
    let (mut w, _) = mundo(
        "Porta",
        vec![linha("botao", "NaoExiste", SignalVerb::Hide, "")],
    );
    assert!(
        resolve(&mut w, &TagTree::new(), &["botao"]).is_empty(),
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

    let antes: Vec<Entity> = resolve(&mut w, &TagTree::new(), &["s"])
        .into_iter()
        .map(|f| f.source)
        .collect();
    assert_eq!(antes, esperado, "a ordem nao e' a da identidade");

    // Mover a entidade do MEIO para outro arquétipo — a ordem da query muda, a da lei não.
    w.entity_mut(ids[1]).insert(Visibility::hidden());
    let depois: Vec<Entity> = resolve(&mut w, &TagTree::new(), &["s"])
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
    let verbos: Vec<SignalVerb> = resolve(&mut w, &TagTree::new(), &["s"])
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
    let verbos: Vec<SignalVerb> = resolve(&mut w, &TagTree::new(), &["abre", "fecha"])
        .into_iter()
        .map(|f| f.verb)
        .collect();
    assert_eq!(verbos, vec![SignalVerb::Show, SignalVerb::Hide]);
}

/// **Sem sinais neste quadro, a resolução não toca no mundo** — é o caso de quase todo quadro.
#[test]
fn a_frame_with_no_signals_resolves_to_nothing() {
    let (mut w, _) = mundo("Porta", vec![linha("s", "", SignalVerb::Show, "")]);
    assert!(resolve(&mut w, &TagTree::new(), &[]).is_empty());
}

/// ⭐ **O `arg` só é lido pelos verbos que o usam**, e a lista é DERIVADA do verbo.
///
/// ⚠️ Um painel que mostra um campo que o verbo não lê é um controlo morto; um que o esconde onde o
/// verbo o lê é uma feature inalcançável. Este gate é o que impede as duas metades de divergirem.
///
/// ⚠️⚠️ **A PREMISSA deste gate MORREU em 2026-09-17, e o nome dele era ela:** ele chamava-se
/// `only_the_timer_verbs_read_the_argument`, e o `AddToCounter` do HUD (TOP-20 #20) lê o `arg`
/// **sem ser um relógio** — ali ele é *quanto somar*. A lista continua a ser escrita à mão de
/// propósito: ela é o CONTROLO do `uses_arg`, e derivá-la dele tornaria o gate uma tautologia.
#[test]
fn so_estes_verbos_leem_o_argumento() {
    for v in SignalVerb::ALL {
        let esperado = matches!(
            v,
            // o `arg` é o NOME do relógio…
            SignalVerb::StartTimer | SignalVerb::StopTimer
            // …e aqui é QUANTO somar (TOP-20 #20).
            | SignalVerb::AddToCounter
        );
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

// ── ⭐⭐⭐ O ALVO POR TAG (TOP-20 #9, `docs/Components/08_plano_tags.md` §5.2, W2) ─────────────────

/// A árvore do plano §5.1: `Enemy` › `Flying` › `Boss`, e a raiz irmã `Statue`.
fn arvore() -> (TagTree, TagId, TagId, TagId, TagId) {
    let mut t = TagTree::new();
    let boss = t.create("Enemy/Flying/Boss").expect("cria");
    let enemy = t.find("Enemy").expect("ancestral");
    let flying = t.find("Enemy/Flying").expect("ancestral");
    let statue = t.create("Statue").expect("cria");
    (t, enemy, flying, boss, statue)
}

fn marcado(w: &mut World, nome: &str, tags: &[TagId]) -> Entity {
    w.spawn((
        Transform::default(),
        Name::new(nome),
        crate::Tags::from_ids(tags.iter().copied()),
    ))
    .id()
}

fn por_tag(on: &str, tag: TagId, verb: SignalVerb) -> SignalAction {
    SignalAction {
        on: on.into(),
        target: String::new(),
        verb,
        arg: String::new(),
        target_by: SignalTarget::Tagged(tag.0),
    }
}

/// ⭐⭐⭐ **Um sinal para `Enemy` atinge a subárvore inteira, e a raiz irmã fica** (gate 14).
///
/// **Mutações que devem sangrar:** o braço `Tagged` a ler só a pertença directa (o `Bat` e o `Dragon`
/// ficam) · a ordem dos alvos ser a da query.
#[test]
fn a_signal_to_a_tag_reaches_the_whole_subtree_and_the_sibling_root_stays() {
    let (tree, enemy, flying, boss, statue) = arvore();
    let mut w = World::new();
    let goblin = marcado(&mut w, "Goblin", &[enemy]);
    let bat = marcado(&mut w, "Bat", &[flying]);
    let dragon = marcado(&mut w, "Dragon", &[boss]);
    let estatua = marcado(&mut w, "Statue", &[statue]);
    let cerebro = w
        .spawn((
            Transform::default(),
            Name::new("Scene Brain"),
            SignalActions(vec![por_tag("alarm", enemy, SignalVerb::Hide)]),
        ))
        .id();
    assign_missing_stable_ids(&mut w);
    // ⚠️ O controlo da ordem: o Goblin muda de arquétipo, e a query passa a listá-lo depois.
    w.entity_mut(goblin).insert(Visibility::hidden());

    let out = resolve(&mut w, &tree, &["alarm"]);
    let alvos: Vec<Entity> = out.iter().map(|f| f.target).collect();
    assert_eq!(
        alvos,
        vec![goblin, bat, dragon],
        "a subarvore inteira, pela ordem da identidade"
    );
    assert!(!alvos.contains(&estatua), "a raiz irma casou");
    assert!(
        out.iter()
            .all(|f| f.source == cerebro && f.verb == SignalVerb::Hide),
        "um efeito perdeu quem reagiu ou o verbo"
    );
}

/// ⭐ **Quem reage É atingido, se pertencer à tag** — o `call_group` do Godot (medido na sonda do
/// plano §1.1: o chamador está no grupo e recebe a chamada).
#[test]
fn the_reactor_is_reached_when_it_belongs_to_the_tag() {
    let (tree, enemy, ..) = arvore();
    let mut w = World::new();
    let chefe = w
        .spawn((
            Transform::default(),
            Name::new("Chefe"),
            crate::Tags::from_ids([enemy]),
            SignalActions(vec![por_tag("grito", enemy, SignalVerb::Show)]),
        ))
        .id();
    let lacaio = marcado(&mut w, "Lacaio", &[enemy]);
    assign_missing_stable_ids(&mut w);
    let alvos: Vec<Entity> = resolve(&mut w, &tree, &["grito"])
        .into_iter()
        .map(|f| f.target)
        .collect();
    assert_eq!(alvos, vec![chefe, lacaio]);
}

/// ⛔ **Uma tag que já não existe não alcança ninguém** — nem o id reservado `0`, nem um id de outro
/// documento (gate 15).
///
/// **Mutação que deve sangrar:** o braço `Tagged` a cair em `source` (ou no alvo por nome) quando a
/// tag falta.
#[test]
fn a_deleted_tag_target_reaches_nobody() {
    let (mut tree, _, _, boss, _) = arvore();
    let mut w = World::new();
    marcado(&mut w, "Dragon", &[boss]);
    w.spawn((
        Transform::default(),
        Name::new("Brain"),
        SignalActions(vec![
            por_tag("x", boss, SignalVerb::Hide),
            por_tag("y", TagId(999), SignalVerb::Hide),
            por_tag("z", TagId(0), SignalVerb::Hide),
        ]),
    ));
    assign_missing_stable_ids(&mut w);
    assert_eq!(
        resolve(&mut w, &tree, &["x"]).len(),
        1,
        "controlo: com a tag viva o Dragon e' atingido"
    );
    assert!(
        resolve(&mut w, &tree, &["y", "z"]).is_empty(),
        "um id que a arvore nao tem atingiu alguem"
    );
    let _ = tree.delete(boss);
    assert!(
        resolve(&mut w, &tree, &["x"]).is_empty(),
        "a tag apagada continuou a atingir o Dragon"
    );
}

/// ⚠️ **Um alvo por NOME resolve igual com qualquer árvore** — o mundo de antes da W2, byte a byte no
/// comportamento (gate 15). E o default de uma linha é o alvo por nome.
///
/// **Mutação que deve sangrar:** o braço `Named` a consultar a árvore.
#[test]
fn a_named_target_resolves_the_same_whatever_the_tree() {
    let construir = |tags: &[TagId]| {
        let mut w = World::new();
        marcado(&mut w, "Parede", tags);
        w.spawn((
            Transform::default(),
            Name::new("Porta"),
            crate::Tags::from_ids(tags.iter().copied()),
            SignalActions(vec![
                linha("botao", "Parede", SignalVerb::Hide, ""),
                linha("botao", "", SignalVerb::Show, ""),
            ]),
        ));
        assign_missing_stable_ids(&mut w);
        w
    };
    let resumo =
        |w: &mut World, t: &TagTree| -> Vec<(Option<String>, SignalVerb, Option<String>)> {
            resolve(w, t, &["botao"])
                .into_iter()
                .map(|f| (name_of(w, f.target), f.verb, name_of(w, f.source)))
                .collect()
        };
    let (cheia, enemy, ..) = arvore();
    let sem = resumo(&mut construir(&[]), &TagTree::new());
    let com = resumo(&mut construir(&[enemy]), &cheia);
    assert_eq!(sem.len(), 2, "controlo: as duas linhas por nome resolvem");
    assert_eq!(sem, com, "a arvore mudou o que um alvo por NOME atinge");
    assert_eq!(SignalAction::default().target_by, SignalTarget::Named);
}

/// ⭐⭐⭐ **Os bytes CONGELADOS de um `SignalActions` v128 carregam como alvo por nome** (gate 16).
///
/// ⚠️ **Os bytes são literais, e não saem do tipo congelado**: se o tipo congelado derivasse com o
/// vivo, os dois lados mudariam juntos e nada ficaria vermelho — a lei do `the_frozen_v95_bytes_still_load`.
/// Eles são o postcard de `[("botao", "Parede", Hide, "")]`: comprimento `1`, as duas strings com o
/// comprimento à frente, o verbo pela POSIÇÃO (`Hide` = `3`) e o `arg` vazio.
///
/// **Mutações que devem sangrar:** a migração a escrever `Tagged` · a perder um campo · ler com o tipo
/// vivo (que falha — e é o controlo de que a migração é precisa).
#[test]
fn a_v128_signal_action_loads_as_a_named_target() {
    const V128: &[u8] = &[
        0x01, 0x05, b'b', b'o', b't', b'a', b'o', 0x06, b'P', b'a', b'r', b'e', b'd', b'e', 0x03,
        0x00,
    ];
    assert!(
        postcard::from_bytes::<SignalActions>(V128).is_err(),
        "controlo: o tipo VIVO leu um v128 -- entao a migracao nao seria precisa"
    );
    let vivo = migrate_v1_blob(V128).expect("um v128 le-se");
    let (lido, resto): (SignalActions, &[u8]) =
        postcard::take_from_bytes(&vivo).expect("o blob reescrito le-se com o tipo vivo");
    assert!(resto.is_empty(), "sobraram bytes no blob reescrito");
    assert_eq!(
        lido.0,
        vec![SignalAction {
            on: "botao".into(),
            target: "Parede".into(),
            verb: SignalVerb::Hide,
            arg: String::new(),
            target_by: SignalTarget::Named,
        }]
    );
    assert_eq!(migrate_v1_blob(&[0x00]), Some(vec![0x00]), "a tabela vazia");
}

/// ⚠️ **Um blob que não se lê como v128 fica como estava** — e um blob VIVO de uma linha é recusado
/// pela sobra do `target_by`.
///
/// **Mutação que deve sangrar:** trocar o `take_from_bytes` + resto vazio por um `from_bytes`, que
/// aceita um prefixo válido e ignora o que sobra.
#[test]
fn a_blob_that_is_not_v128_is_left_alone() {
    let vivo = postcard::to_allocvec(&SignalActions(vec![por_tag(
        "a",
        TagId(5),
        SignalVerb::Show,
    )]))
    .expect("serializa");
    assert_eq!(migrate_v1_blob(&vivo), None);
    assert_eq!(migrate_v1_blob(&[0xff, 0xff]), None);
}
