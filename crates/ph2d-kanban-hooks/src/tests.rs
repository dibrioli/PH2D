//! A porta: o gesto, o Butler, os scripts, o desfazer em camadas e a agenda fora dele.

use super::*;
use ph2d_kanban_auto::{Action, Added, By, Spec, Trigger};
use ph2d_kanban_model::{Card, CardId, CardRef, List, ListId, Meta, Place, Record, move_card};

const NOW: Moment = Moment(1_793_188_800_000);

struct S {
    set: KanbanSet,
    h: BTreeMap<KanbanId, History>,
    k: KanbanId,
    todo: ListId,
    done: ListId,
    clock: u64,
}

fn list(set: &mut KanbanSet, k: KanbanId, title: &str) -> ListId {
    let kd = set.get(k).unwrap();
    let id = ListId(kd.mint());
    let l = List {
        id,
        title: title.to_owned(),
        pos: kd.pos_after_lists(),
        color: None,
        card_limit: None,
        archived: false,
        meta: Meta::default(),
        watchers: Vec::new(),
    };
    set.apply(k, vec![KanbanOp::put(Record::List(l))]);
    id
}

impl S {
    fn new() -> Self {
        let mut set = KanbanSet::default();
        let k = set.create("P");
        let todo = list(&mut set, k, "To Do");
        let done = list(&mut set, k, "Done");
        Self {
            set,
            h: BTreeMap::new(),
            k,
            todo,
            done,
            clock: 0,
        }
    }

    fn card(&mut self, title: &str) -> CardId {
        let kd = self.set.get(self.k).unwrap();
        let id = CardId(kd.mint());
        let c = Card::new(id, self.todo, kd.pos_after_cards(self.todo), title, NOW);
        self.set
            .apply(self.k, vec![KanbanOp::put(Record::card(c))]);
        id
    }

    fn rule_archive_in_done(&mut self) {
        let spec = Spec {
            name: String::new(),
            enabled: true,
            owner: Member("Enio".into()),
            kind: Kind::Rule {
                trigger: Trigger::CardAdded {
                    list: Some(self.done),
                    how: Added::Any,
                },
                by: By::Anyone,
                conditions: Vec::new(),
            },
            actions: vec![Action::Archive],
        };
        let op = spec.create(self.set.get(self.k).unwrap(), NOW).expect("automação válida");
        self.set.apply(self.k, vec![op]);
    }

    fn move_to_done(&mut self, c: CardId) -> Reaction {
        let kd = self.set.get(self.k).unwrap();
        let to = Place {
            kanban: self.k,
            list: self.done,
            pos: kd.pos_after_cards(self.done),
        };
        let src = CardRef {
            kanban: self.k,
            card: c,
        };
        let plan = move_card(&self.set, src, &to).unwrap();
        let zone = Zone::utc();
        let who = Who {
            me: Member("Enio".into()),
            now: NOW,
            zone: &zone,
        };
        let mut t = self.clock;
        let r = gesture(&mut self.set, &mut self.h, plan, &who, &mut || {
            t += 1;
            t
        });
        self.clock = t;
        r
    }

    fn undo(&mut self) -> bool {
        let actor = Actor {
            by: Member("Enio".into()),
            at: NOW,
        };
        self.clock += 1;
        self.set.undo_gesture(&mut self.h, self.clock, &actor)
    }

    fn get(&self, c: CardId) -> Card {
        self.set.get(self.k).unwrap().cards[&c].clone()
    }
}

#[test]
fn the_butler_reacts_to_a_gesture_in_its_own_undo_step() {
    let mut s = S::new();
    s.rule_archive_in_done();
    let c = s.card("Shot");
    let r = s.move_to_done(c);
    assert!(r.wrote);
    assert_eq!(r.butler.len(), 1);
    assert!(s.get(c).archived, "a regra arquivou");
    assert!(s.undo());
    assert!(!s.get(c).archived, "o 1.º desfazer tira o que o Butler fez");
    assert_eq!(s.get(c).list, s.done, "e deixa o gesto da pessoa");
    assert!(s.undo());
    assert_eq!(s.get(c).list, s.todo);
    // Refazer não reage: o Butler não corre de novo (o refazer devolve o passo dele, só uma vez).
    let actor = Actor {
        by: Member("Enio".into()),
        at: NOW,
    };
    assert!(s.set.redo_gesture(&mut s.h, 99, &actor));
    assert!(!s.get(c).archived);
}

#[test]
fn a_loaded_kanban_script_reacts_after_the_butler() {
    let mut s = S::new();
    let c = s.card("Shot");
    let zone = Zone::utc();
    let who = Who {
        me: Member("Enio".into()),
        now: NOW,
        zone: &zone,
    };
    let req = RunRequest {
        key: 7,
        name: "notes.luau".into(),
        source: "ph2d.kanban.on(\"card_moved\", function(e) e.card:comment(\"moved\") end)".into(),
    };
    let rep = scripts::run(&req, &mut s.set, &mut s.h, &who, 1);
    assert!(rep.errors.is_empty(), "{:?}", rep.errors);
    let r = s.move_to_done(c);
    assert_eq!(r.handlers, 1);
    let k = s.set.get(s.k).unwrap();
    assert!(k.comments.values().any(|x| x.card == c && x.text == "moved"));
    unload(7);
    assert!(with_scripts(|x| x.loaded().is_empty()));
}

#[test]
fn script_errors_and_prints_reach_the_console_of_their_script() {
    let mut s = S::new();
    let zone = Zone::utc();
    let who = Who {
        me: Member("Enio".into()),
        now: NOW,
        zone: &zone,
    };
    let _ = take_lines();
    let req = RunRequest {
        key: 3,
        name: "a.luau".into(),
        source: "print(#ph2d.kanban.list())\nerror(\"boom\")".into(),
    };
    let _ = scripts::run(&req, &mut s.set, &mut s.h, &who, 1);
    let lines = take_lines();
    assert!(lines.iter().any(|l| l.key == 3 && !l.error && l.text == "1"));
    assert!(
        lines
            .iter()
            .any(|l| l.key == 3 && l.error && l.text.contains("boom") && l.line == Some(2)),
        "{lines:?}"
    );
}

#[test]
fn a_kanban_script_is_one_whose_code_uses_ph2d_kanban() {
    assert!(is_kanban_script("local c = ph2d.kanban.get(\"x\")"));
    assert!(!is_kanban_script("-- ph2d.kanban.get(\"x\")\nprint(1)"));
    assert!(!is_kanban_script("print(\"ph2d.kanban\")"));
    assert!(!is_kanban_script("--[[ ph2d.kanban ]] print(1)"));
    assert!(!is_kanban_script("local s = [==[ ph2d.kanban ]==]"));
    assert!(is_kanban_script("--[[ x ]] ph2d.kanban.on('card_moved', f)"));
}

#[test]
fn the_agenda_skips_a_project_without_scheduled_commands_and_stays_out_of_undo() {
    let mut s = S::new();
    s.rule_archive_in_done();
    assert!(!has_scheduled(&s.set), "uma regra não é um comando agendado");
    let zone = Zone::utc();
    let who = Who {
        me: Member("Enio".into()),
        now: NOW,
        zone: &zone,
    };
    let mut a = Agenda::default();
    assert!(a.tick(&mut s.set, &who).is_none());
    assert!(s.h.is_empty(), "a agenda nunca escreve no desfazer");
}

/// Régua (ignorada) da agenda no quadro do app: `has_scheduled` (o custo de CADA verificação) e
/// `run_scheduled` (o de uma corrida sem nada a disparar), na cena 1 × `n` Kanbans com 20 regras e
/// um comando de calendário cada; o MÍNIMO de 200 amostras. Resultado em `MEDICOES.md` §J3.
#[test]
#[ignore = "régua: corre com --ignored --nocapture"]
fn medir_agenda() {
    use ph2d_kanban_auto::{Every, Scope, TimeOfDay};
    for n in [1usize, 10, 50] {
        let mut set = KanbanSet::default();
        for _ in 0..n {
            let k = ph2d_kanban_scenes::short_film(&mut set, NOW);
            for i in 0..21 {
                let kind = if i == 0 {
                    Kind::Calendar {
                        every: Every::Day,
                        at: TimeOfDay { hour: 9, minute: 0 },
                        scope: Scope::default(),
                    }
                } else {
                    Kind::Rule {
                        trigger: Trigger::CardAdded {
                            list: None,
                            how: Added::Any,
                        },
                        by: By::Anyone,
                        conditions: Vec::new(),
                    }
                };
                let spec = Spec {
                    name: String::new(),
                    enabled: true,
                    owner: Member("Enio".into()),
                    kind,
                    actions: vec![Action::Archive],
                };
                let op = spec.create(set.get(k).unwrap(), NOW).expect("automação válida");
                set.apply(k, vec![op]);
            }
        }
        let zone = Zone::utc();
        let who = Who {
            me: Member("Enio".into()),
            now: NOW,
            zone: &zone,
        };
        // A 1.ª corrida arma (carimba `last_run`); as seguintes são as do dia-a-dia.
        let _ = run_scheduled(&mut set, &who);
        let (mut has, mut run) = (f64::MAX, f64::MAX);
        for _ in 0..200 {
            let t = std::time::Instant::now();
            assert!(std::hint::black_box(has_scheduled(&set)));
            has = has.min(t.elapsed().as_secs_f64() * 1e6);
            let t = std::time::Instant::now();
            let r = run_scheduled(&mut set, &who);
            run = run.min(t.elapsed().as_secs_f64() * 1e6);
            assert!(!r.wrote);
        }
        println!("agenda n={n}: has_scheduled {has:.1} µs · run_scheduled {run:.1} µs");
    }
}
