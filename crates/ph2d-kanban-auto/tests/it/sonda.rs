//! Sonda dos tectos (MEDICOES.md §W8): quanto custa uma automação corrida num Kanban grande.
//! `cargo test -p ph2d-kanban-auto --profile smoke --test it sonda -- --ignored --nocapture`

use std::time::Instant;

use ph2d_kanban_auto::{Action, Added, By, Kind, Seen, Spec, Trigger, react};
use ph2d_kanban_model::{CardRef, Place, Plan, move_card};

use crate::{NOW, Scene, enio, utc};

#[test]
#[ignore = "sonda: mede, não julga"]
fn sonda_custo_por_automacao() {
    for cards in [100_usize, 1000] {
        for rules in [1_usize, 10, 100, 1000] {
            let mut s = Scene::new();
            for i in 0..cards {
                s.card(s.todo, &format!("C{i}"), |_| {});
            }
            let spec = Spec {
                name: String::new(),
                enabled: true,
                owner: enio(),
                kind: Kind::Rule {
                    trigger: Trigger::CardAdded {
                        list: Some(s.done),
                        how: Added::Any,
                    },
                    by: By::Anyone,
                    conditions: Vec::new(),
                },
                actions: vec![Action::Comment("{cardname}".into()), Action::JoinMe],
            };
            for _ in 0..rules {
                s.add(&spec);
            }
            let c = s
                .set
                .get(s.k)
                .unwrap()
                .cards
                .keys()
                .next()
                .copied()
                .unwrap();
            let to = Place {
                kanban: s.k,
                list: s.done,
                pos: s.set.get(s.k).unwrap().pos_after_cards(s.done),
            };
            let src = CardRef {
                kanban: s.k,
                card: c,
            };
            let plan: Plan = move_card(&s.set, src, &to).unwrap();
            let seen = Seen::plan(&s.set, &plan);
            crate::apply(&mut s.set, plan, &enio());
            let events = seen.events(&s.set, &enio());
            let mut best = f64::MAX;
            for _ in 0..5 {
                let t = Instant::now();
                let o = react(&s.set, events.clone(), NOW, &utc());
                best = best.min(t.elapsed().as_secs_f64() * 1e3);
                assert_eq!(o.fired.len(), rules);
            }
            println!(
                "cartões {cards:>5} · regras {rules:>5} · {best:>9.3} ms · {:>8.1} µs/automação",
                best * 1e3 / rules as f64
            );
        }
    }
}
