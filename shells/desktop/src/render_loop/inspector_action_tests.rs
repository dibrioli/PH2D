//! **Os gates da costura da secção SIGNAL ACTIONS** — irmão de [`crate::render_loop::inspector_action`].

use super::*;
use ph2d_ecs::Transform;
use ph2d_ecs::scene::{EditorCommandQueue, apply_editor_commands, register_ecs_components};

fn registry() -> ComponentRegistry {
    let mut r = ComponentRegistry::new();
    register_ecs_components(&mut r);
    ph2d_render::register_render_components(&mut r);
    r
}

fn objecto(sim: &mut SimWorld, rows: Vec<SignalAction>) -> Entity {
    sim.world_mut()
        .spawn((Transform::default(), SignalActions(rows)))
        .id()
}

fn edit(
    sim: &mut SimWorld,
    e: Entity,
    reg: &ComponentRegistry,
    ed: ActionFieldEdit,
) -> Option<Toast> {
    let queue = EditorCommandQueue::new();
    let t = apply_action_edit(sim, e.to_bits(), &ed, &queue, reg);
    apply_editor_commands(sim.world_mut(), &queue, reg).expect("o commit aplica");
    t
}

/// A árvore das fixturas — vazia, para os gates que não falam de tags. ⚠️ Quem fala delas usa a
/// [`arvore_com_tags`].
fn sem_tags() -> ph2d_tags::TagTree {
    ph2d_tags::TagTree::new()
}

fn info(sim: &SimWorld, e: Entity) -> InspectorActionInfo {
    build_action_info(sim.world(), &sem_tags(), e.to_bits(), 1).expect("o objecto tem a tabela")
}

/// ⛔ **Um objecto SEM a tabela não tem secção** — ADR-0166.
#[test]
fn an_object_without_the_component_has_no_section() {
    let mut sim = SimWorld::default();
    let e = sim.world_mut().spawn((Transform::default(),)).id();
    assert!(build_action_info(sim.world(), &sem_tags(), e.to_bits(), 1).is_none());
}

/// ⭐⭐ **Os RÓTULOS do painel saem de `SignalVerb::ALL`, que é a fonte.**
///
/// ⚠️ Copiá-los para o painel envelheceria no primeiro verbo novo — e o artista leria o nome
/// errado sobre a entrada certa do seletor. Este gate é o que liga as duas pontas.
///
/// **Mutação que deve sangrar:** truncar a lista de `verb_labels` no `build_action_info`.
#[test]
fn the_verb_labels_come_from_the_engines_own_list() {
    let mut sim = SimWorld::default();
    let e = objecto(&mut sim, vec![SignalAction::default()]);
    let i = info(&sim, e);
    let esperado: Vec<String> = SignalVerb::ALL
        .iter()
        .map(|v| v.label().to_string())
        .collect();
    assert_eq!(i.verb_labels, esperado, "os rotulos divergiram da fonte");
    assert_eq!(
        i.verb_labels.len(),
        ph2d_panel_inspector::ids::INSP_ACTION_VERB.len(),
        "o seletor oferece um numero de entradas diferente do de verbos — uma delas seria muda, \
         ou um verbo seria inalcancavel"
    );
}

/// ⭐ **O `uses_arg` chega DERIVADO do verbo** — e muda quando o verbo muda.
///
/// ⚠️ É o que decide se o campo do parâmetro é pintado. Um campo mostrado onde o verbo não o lê é
/// um controlo morto; escondido onde ele o lê, uma feature inalcançável.
#[test]
fn the_argument_field_follows_the_verb() {
    let mut sim = SimWorld::default();
    let reg = registry();
    let e = objecto(&mut sim, vec![SignalAction::default()]);
    for v in SignalVerb::ALL {
        edit(&mut sim, e, &reg, ActionFieldEdit::Verb(0, v.tag()));
        assert_eq!(
            info(&sim, e).rows[0].uses_arg,
            v.uses_arg(),
            "{}: o campo do parametro discorda do verbo",
            v.label()
        );
    }
}

/// ⭐⭐ **E a DICA do campo segue o que o parâmetro É** (plano 28, W2b) — a tradução
/// `SignalVerb::arg_kind` → `ActionArgHint` mora aqui, e sem esta régua ela é invisível.
///
/// ⛔ Nasceu de uma FOTO: a linha `Damage` pintava *«timer name»* no campo da quantidade.
#[test]
fn the_argument_hint_says_what_the_argument_is() {
    use ph2d_editor_core::screens::hero::ActionArgHint;
    let mut sim = SimWorld::default();
    let reg = registry();
    let e = objecto(&mut sim, vec![SignalAction::default()]);
    for (v, esperado) in [
        (SignalVerb::StartTimer, ActionArgHint::TimerName),
        (SignalVerb::StopTimer, ActionArgHint::TimerName),
        (SignalVerb::AddToCounter, ActionArgHint::Count),
        (SignalVerb::Damage, ActionArgHint::Amount),
        (SignalVerb::Heal, ActionArgHint::Amount),
    ] {
        edit(&mut sim, e, &reg, ActionFieldEdit::Verb(0, v.tag()));
        assert_eq!(info(&sim, e).rows[0].arg_hint, esperado, "{}", v.label());
    }
}

/// **O `+` cria uma linha que ainda não dispara — e o painel tem de o dizer.**
///
/// ⚠️ Um default que disparasse em alguma coisa faria um `+` mudar a cena sem ninguém pedir.
#[test]
fn a_new_action_never_fires_until_it_is_named() {
    let mut sim = SimWorld::default();
    let reg = registry();
    let e = objecto(&mut sim, vec![]);
    edit(&mut sim, e, &reg, ActionFieldEdit::Add);
    let i = info(&sim, e);
    assert_eq!(i.rows.len(), 1);
    assert!(i.rows[0].never_fires(), "a linha nova ja' dispara");
    assert!(
        i.rows[0].target_is_self(),
        "o alvo nao nasceu como ESTE objecto"
    );

    edit(&mut sim, e, &reg, ActionFieldEdit::On(0, "botao".into()));
    assert!(!info(&sim, e).rows[0].never_fires());
}

/// ⛔ **O `+` no tecto RECUSA COM VOZ.**
#[test]
fn adding_past_the_cap_refuses_out_loud() {
    let mut sim = SimWorld::default();
    let reg = registry();
    let cheio: Vec<SignalAction> = (0..SIGNAL_ACTIONS_MAX)
        .map(|n| SignalAction {
            on: format!("s{n}"),
            ..SignalAction::default()
        })
        .collect();
    let e = objecto(&mut sim, cheio);
    assert!(edit(&mut sim, e, &reg, ActionFieldEdit::Add).is_some());
    assert_eq!(info(&sim, e).rows.len(), SIGNAL_ACTIONS_MAX);
}

/// **Mexer num campo não repõe os outros.**
#[test]
fn editing_one_field_leaves_the_others_alone() {
    let mut sim = SimWorld::default();
    let reg = registry();
    let e = objecto(
        &mut sim,
        vec![SignalAction {
            on: "botao".into(),
            target: "Parede".into(),
            verb: SignalVerb::Hide,
            arg: "x".into(),
            target_by: ph2d_ecs::SignalTarget::Named,
            from: ph2d_ecs::SignalFrom::Anyone,
        }],
    );
    edit(
        &mut sim,
        e,
        &reg,
        ActionFieldEdit::Target(0, "Porta".into()),
    );
    let r = &info(&sim, e).rows[0];
    assert_eq!(r.target, "Porta");
    assert_eq!(r.on, "botao", "o sinal foi reposto");
    assert_eq!(r.verb_tag, SignalVerb::Hide.tag(), "o verbo foi reposto");
    assert_eq!(r.arg, "x", "o parametro foi reposto");
}

/// **Uma edição num índice que já não existe não faz nada, e não estoura.**
#[test]
fn an_edit_on_a_vanished_index_is_a_no_op() {
    let mut sim = SimWorld::default();
    let reg = registry();
    let e = objecto(&mut sim, vec![SignalAction::default()]);
    assert!(edit(&mut sim, e, &reg, ActionFieldEdit::Remove(7)).is_none());
    assert!(edit(&mut sim, e, &reg, ActionFieldEdit::On(7, "x".into())).is_none());
    assert_eq!(info(&sim, e).rows.len(), 1);
}

/// ⭐⭐ **Os ids do painel cobrem o cap do MODELO.**
#[test]
fn the_action_row_ids_cover_the_model_cap() {
    assert_eq!(
        ph2d_panel_inspector::ids::INSP_ACTION_ROW.len(),
        SIGNAL_ACTIONS_MAX,
        "o painel desenha um numero de linhas diferente do cap do modelo — as accoes a mais \
         seriam inalcancaveis"
    );
}

/// Uma linha de acção comum, para os gates do alvo por TAG.
fn linha_de_accao() -> SignalAction {
    SignalAction {
        on: "alarme".into(),
        target: "Parede".into(),
        verb: SignalVerb::Hide,
        ..SignalAction::default()
    }
}

/// Uma árvore com `Inimigo` e `Inimigo/Voador`, para os gates do alvo por TAG.
fn arvore_com_tags() -> (ph2d_tags::TagTree, u64, u64) {
    let mut t = ph2d_tags::TagTree::new();
    let inimigo = t.create("Inimigo").expect("cria").0;
    let voador = t.create("Inimigo/Voador").expect("cria").0;
    (t, inimigo, voador)
}

/// ⭐⭐⭐ **Virar o alvo para TAG não escolhe tag nenhuma** — a linha fica *por acabar*, e não
/// apontada a alguém que o artista nunca nomeou.
///
/// ⛔ **Escolher a primeira tag da árvore por ele** faria um clique num segmentado mudar a QUEM a
/// acção acerta — que é o mesmo defeito do `+` que nasce com um sinal a disparar.
///
/// **Mutação que deve sangrar:** o `Tagged(0)` trocado pela primeira tag da árvore.
#[test]
fn switching_the_target_to_tag_picks_no_tag_at_all() {
    let (tree, _inimigo, _voador) = arvore_com_tags();
    let reg = registry();
    let mut sim = SimWorld::default();
    let e = objecto(&mut sim, vec![linha_de_accao()]);
    edit(
        &mut sim,
        e,
        &reg,
        ActionFieldEdit::TargetMode(0, ActionTargetMode::Tag.tag()),
    );
    let i = build_action_info(sim.world(), &tree, e.to_bits(), 1).expect("tem a tabela");
    let r = &i.rows[0];
    assert!(r.target_is_tag(), "a linha nao ficou no modo TAG");
    assert!(
        r.target_tag_unset(),
        "virar para TAG escolheu uma tag sozinho — o alvo mudou sem ninguem o pedir"
    );
    assert!(
        !r.target_tag_missing(),
        "uma linha POR ACABAR foi lida como uma linha PARTIDA — sao duas historias diferentes"
    );
}

/// ⭐⭐ **Escolhida a tag, a linha mostra o CAMINHO dela** — e não o número.
///
/// ⚠️ O painel não conhece a árvore: sem o caminho no snapshot ele só teria um id para desenhar.
///
/// **Mutação que deve sangrar:** o `target_tag_path` a vir sempre vazio.
#[test]
fn a_tag_target_shows_its_path_not_its_number() {
    let (tree, _inimigo, voador) = arvore_com_tags();
    let reg = registry();
    let mut sim = SimWorld::default();
    let e = objecto(&mut sim, vec![linha_de_accao()]);
    edit(&mut sim, e, &reg, ActionFieldEdit::TargetTag(0, voador));
    let i = build_action_info(sim.world(), &tree, e.to_bits(), 1).expect("tem a tabela");
    let r = &i.rows[0];
    assert_eq!(r.target_tag, Some(voador));
    assert_eq!(r.target_tag_path, "Inimigo/Voador");
    assert!(!r.target_tag_missing() && !r.target_tag_unset());
    // ⚠️ E o alvo deixa de ser «este objecto», mesmo com o campo do NOME vazio: ali o vazio já não
    // significa nada, porque ninguém lê aquele campo.
    assert!(
        !r.target_is_self(),
        "uma linha com alvo por TAG foi lida como apontando a si mesma"
    );
}

/// ⛔⛔ **Uma tag APAGADA deixa a linha a alcançar ninguém, e a secção tem de o poder dizer.**
///
/// ⚠️ *Uma acção que deixou de acertar por causa de um gesto noutro painel é a forma mais
/// silenciosa de «não acontece nada»* — e ela distingue-se de uma linha por acabar.
///
/// **Mutação que deve sangrar:** o `target_tag_missing` a devolver `false` sempre · o
/// `unwrap_or_default` do caminho trocado por um `format!("{id}")` (a linha passaria a mostrar um
/// número e a parecer viva).
#[test]
fn a_deleted_tag_leaves_the_action_pointing_at_nobody_and_says_so() {
    let (mut tree, _inimigo, voador) = arvore_com_tags();
    let reg = registry();
    let mut sim = SimWorld::default();
    let e = objecto(&mut sim, vec![linha_de_accao()]);
    edit(&mut sim, e, &reg, ActionFieldEdit::TargetTag(0, voador));
    // O painel *Tags* (W4) apaga a tag — a acção não é tocada.
    let _ = tree.delete(ph2d_tags::TagId(voador));
    let i = build_action_info(sim.world(), &tree, e.to_bits(), 1).expect("tem a tabela");
    let r = &i.rows[0];
    assert!(
        r.target_tag_missing(),
        "a tag foi apagada e a linha nao acusa nada — ela alcanca ninguem em silencio"
    );
    assert!(
        !r.target_tag_unset(),
        "uma linha PARTIDA foi lida como uma linha POR ACABAR"
    );
}

/// ⚠️ **Voltar ao modo NOME larga a tag, e isso é a decisão do modelo, não um esquecimento** — o
/// `SignalTarget` é um enum, e o ramo `Named` não tem onde guardar uma tag. ⛔ Guardar as duas
/// respostas ao mesmo tempo é o que o doc dele recusa por escrito.
///
/// **Mutação que deve sangrar:** o braço `false` do `TargetMode` a não repor o `Named`.
#[test]
fn going_back_to_name_drops_the_tag_by_construction() {
    let (tree, _i, voador) = arvore_com_tags();
    let reg = registry();
    let mut sim = SimWorld::default();
    let e = objecto(&mut sim, vec![linha_de_accao()]);
    edit(&mut sim, e, &reg, ActionFieldEdit::TargetTag(0, voador));
    edit(
        &mut sim,
        e,
        &reg,
        ActionFieldEdit::TargetMode(0, ActionTargetMode::Name.tag()),
    );
    let i = build_action_info(sim.world(), &tree, e.to_bits(), 1).expect("tem a tabela");
    assert!(
        !i.rows[0].target_is_tag(),
        "a linha continua no modo TAG depois de voltar ao NOME"
    );
    assert_eq!(i.rows[0].target_tag, None);
}

/// ⭐⭐⭐ **A CERCA vai ao documento e VOLTA ao painel** (suplente #24) — a pergunta que o
/// `CLAUDE.md` §5.0 diz que nenhum instrumento deste repo faz: *o valor CHEGA a um consumidor?*
///
/// ⚠️ **As três metades:** a ida (o clique escreve o motor) · a volta (o snapshot lê o que foi
/// escrito) · e o **CONTROLO** (voltar a `Anyone` desfaz). Sem a terceira, um dreno que escrevesse
/// `Myself` sempre ficaria verde nas duas primeiras.
///
/// **Mutação que deve sangrar:** o braço `From` do dreno a cravar `SignalFrom::Anyone`.
#[test]
fn a_cerca_vai_ao_documento_e_volta_ao_painel() {
    let mut sim = SimWorld::default();
    let reg = registry();
    let e = objecto(&mut sim, vec![linha_de_accao()]);
    assert!(
        !info(&sim, e).rows[0].from_is_myself(),
        "a linha nao nasceu em `Anyone` — o default do modelo mudou"
    );

    edit(&mut sim, e, &reg, ActionFieldEdit::From(0, 1));
    assert!(
        info(&sim, e).rows[0].from_is_myself(),
        "a cerca nao chegou ao documento, ou nao voltou ao painel"
    );
    // ⛔ O CONTROLO: e ela volta atrás.
    edit(&mut sim, e, &reg, ActionFieldEdit::From(0, 0));
    assert!(
        !info(&sim, e).rows[0].from_is_myself(),
        "a cerca ficou presa em `Myself` — o dreno crava o valor"
    );
}

/// ⭐⭐ **Virar o alvo para QUEM BATEU LARGA a tag**, e o painel di-lo.
///
/// ⚠️ **É a lei do enum vista na fronteira:** o `SignalTarget` guarda a carga de UM modo, logo
/// mudar de modo deita a anterior fora. *Guardar as duas ao mesmo tempo é o que o doc dele recusa.*
///
/// **Mutação que deve sangrar:** o braço `Other` do dreno a escrever `Tagged(0)`.
#[test]
fn virar_o_alvo_para_quem_bateu_larga_a_tag() {
    let (tree, _i, voador) = arvore_com_tags();
    let reg = registry();
    let mut sim = SimWorld::default();
    let e = objecto(&mut sim, vec![linha_de_accao()]);
    edit(&mut sim, e, &reg, ActionFieldEdit::TargetTag(0, voador));
    assert!(
        build_action_info(sim.world(), &tree, e.to_bits(), 1)
            .expect("tem a tabela")
            .rows[0]
            .target_is_tag(),
        "o controlo falhou: a linha nao ficou no modo TAG"
    );

    edit(
        &mut sim,
        e,
        &reg,
        ActionFieldEdit::TargetMode(0, ActionTargetMode::Other.tag()),
    );
    let i = build_action_info(sim.world(), &tree, e.to_bits(), 1).expect("tem a tabela");
    let r = &i.rows[0];
    assert!(r.target_is_other(), "a linha nao ficou no modo QUEM BATEU");
    assert!(
        !r.target_is_tag(),
        "ela ficou nos DOIS modos ao mesmo tempo"
    );
    assert_eq!(r.target_tag, None, "a tag anterior sobreviveu ao modo novo");
}

/// ⭐⭐ **Os ids dos segmentos do alvo cobrem os modos do vocabulário.**
///
/// ⚠️ **Um modo sem segmento é inalcançável e um segmento a mais é mudo** — as duas metades da
/// mesma cerca, e é a forma exacta do defeito que o `Density` pagou (*«o verbo existe, tem lei, tem
/// gates, e o artista nao lhe chega»*).
#[test]
fn cada_modo_do_alvo_tem_um_segmento_no_painel() {
    assert_eq!(
        ActionTargetMode::ALL.len(),
        3,
        "o vocabulario do alvo mudou de tamanho — reveja os segmentos do painel"
    );
    for m in ActionTargetMode::ALL {
        assert_eq!(
            ActionTargetMode::from_tag(m.tag()),
            m,
            "{m:?} nao volta da tag — a POSICAO no array e' o que atravessa a fronteira"
        );
    }
}
