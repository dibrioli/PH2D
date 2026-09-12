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

fn info(sim: &SimWorld, e: Entity) -> InspectorActionInfo {
    build_action_info(sim.world(), e.to_bits(), 1).expect("o objecto tem a tabela")
}

/// ⛔ **Um objecto SEM a tabela não tem secção** — ADR-0166.
#[test]
fn an_object_without_the_component_has_no_section() {
    let mut sim = SimWorld::default();
    let e = sim.world_mut().spawn((Transform::default(),)).id();
    assert!(build_action_info(sim.world(), e.to_bits(), 1).is_none());
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
        ph2d_editor_core::ids::INSP_ACTION_VERB.len(),
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
        ph2d_editor_core::ids::INSP_ACTION_ROW.len(),
        SIGNAL_ACTIONS_MAX,
        "o painel desenha um numero de linhas diferente do cap do modelo — as accoes a mais \
         seriam inalcancaveis"
    );
}
