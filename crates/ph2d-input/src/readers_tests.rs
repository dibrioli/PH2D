use super::*;
use crate::action::Binding;
use crate::keyboard::Key;

const UP: Key = Key(0xF700);
const DOWN: Key = Key(0xF701);
const S: Key = Key(0x53);
const J: Key = Key(0x4A);

fn names_of(map: &InputMap, ids: &[ActionId]) -> Vec<String> {
    ids.iter()
        .map(|i| map.get(*i).expect("existe").name.clone())
        .collect()
}

#[test]
fn the_factory_map_shares_up_down_and_s_on_purpose() {
    let m = InputMap::with_player_defaults();
    let groups = shared_bindings(&m, &readers(&ReaderFacts::default()));
    let got: Vec<(Binding, Vec<String>, bool)> = groups
        .iter()
        .map(|g| (g.binding, names_of(&m, &g.actions), g.on_purpose))
        .collect();
    let pair = |a: &str, b: &str| vec![a.to_owned(), b.to_owned()];
    assert_eq!(
        got,
        vec![
            (Binding::Key(S), pair("down", "move_down"), true),
            (Binding::Key(UP), pair("jump", "move_up"), true),
            (Binding::Key(DOWN), pair("down", "move_down"), true),
        ]
    );
}

#[test]
fn two_actions_nobody_reads_on_one_key_is_a_mistake() {
    let mut m = InputMap::with_player_defaults();
    for n in ["fire", "laser"] {
        let id = m.create(n);
        m.get_mut(id).expect("existe").bindings.push(Binding::Key(J));
    }
    let groups = shared_bindings(&m, &readers(&ReaderFacts::default()));
    let wrong: Vec<_> = groups.iter().filter(|g| !g.on_purpose).collect();
    assert_eq!(wrong.len(), 1);
    assert_eq!(wrong[0].binding, Binding::Key(J));
    assert_eq!(names_of(&m, &wrong[0].actions), ["fire", "laser"]);
}

#[test]
fn a_trigger_reading_jump_makes_the_up_arrow_a_mistake() {
    let m = InputMap::with_player_defaults();
    let facts = ReaderFacts {
        triggers: vec![("Door".to_owned(), "jump".to_owned())],
        scripts: Vec::new(),
    };
    let groups = shared_bindings(&m, &readers(&facts));
    let up = groups
        .iter()
        .find(|g| g.binding == Binding::Key(UP))
        .expect("Up é partilhada");
    assert!(!up.on_purpose);
    assert!(
        groups
            .iter()
            .filter(|g| g.binding != Binding::Key(UP))
            .all(|g| g.on_purpose)
    );
}

#[test]
fn a_shared_key_between_a_mover_name_and_move_left_is_a_mistake() {
    // move_left é lido pelos DOIS movers: não é disjunto de jump.
    let mut m = InputMap::with_player_defaults();
    let left = m.id("move_left").expect("fábrica");
    m.get_mut(left).expect("existe").bindings.push(Binding::Key(UP));
    let groups = shared_bindings(&m, &readers(&ReaderFacts::default()));
    let up = groups
        .iter()
        .find(|g| g.binding == Binding::Key(UP))
        .expect("Up");
    assert_eq!(up.actions.len(), 3);
    assert!(!up.on_purpose);
}

#[test]
fn the_static_readers_are_always_there_and_scripts_and_triggers_join_them() {
    let facts = ReaderFacts {
        triggers: vec![
            ("Door".to_owned(), "open".to_owned()),
            ("Door".to_owned(), "open".to_owned()),
        ],
        scripts: vec![(
            "hero".to_owned(),
            "if ph2d.action_pressed(\"jump\") then end".to_owned(),
        )],
    };
    let r = readers(&facts);
    assert_eq!(
        r["move_left"],
        [Reader::PlatformPlayer, Reader::TopDownPlayer]
    );
    assert_eq!(r["move_up"], [Reader::TopDownPlayer]);
    assert_eq!(
        r["open"],
        [Reader::Trigger {
            object: "Door".to_owned()
        }]
    );
    assert_eq!(
        r["jump"],
        [
            Reader::PlatformPlayer,
            Reader::Script {
                name: "hero".to_owned()
            }
        ]
    );
    assert_eq!(
        Reader::PlatformPlayer.label_key(),
        Some("input_map_tab.reader.platform_player")
    );
    assert_eq!(Reader::Script { name: "x".into() }.label_key(), None);
}

#[test]
fn a_script_is_read_for_the_four_calls_both_quotes_and_never_inside_a_comment() {
    let src = r#"
local s = ph2d.action_strength('move_left')
if ph2d.action_down("fire") and ph2d.action_released ( "reload" ) then end
-- ph2d.action_pressed("commented")
local t = "-- not a comment" ; ph2d.action_pressed("after_string") -- ph2d.action_down("tail")
--[[ ph2d.action_down("block")
]] ph2d.action_down("after_block")
myph2d.action_down("not_ours")
ph2d.action_down(name)
ph2d.action_down("")
"#;
    let got: Vec<String> = action_names_in_script(src).into_iter().collect();
    assert_eq!(
        got,
        [
            "after_block",
            "after_string",
            "fire",
            "move_left",
            "reload"
        ]
    );
}
