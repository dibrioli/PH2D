//! ⚠️ SONDA (`#[ignore]`): o tecto de cartões a 60 fps — o CPU que a família põe num quadro
//! (`frame` + a pintura da aba numa `VectorScene`), para N cartões em 10 listas e numa só, parado e
//! com um arrasto em curso. Régua de `docs/DevOps/MEDIR_VELOCIDADE.md`: variantes lado a lado no
//! mesmo processo, aquecidas fora da régua, blocos intercalados por ordem RODADA, resumo = o MÍNIMO
//! das rodadas (mediana ao lado). Corre em `--profile smoke`.

use crate::Mesa;
use ph2d_kanban_model::{
    Card, CardId, FracKey, Hue, KanbanId, KanbanOp, Label, LabelColor, LabelId, List, ListId,
    Member, Meta, Moment, Record, Shade,
};
use std::time::Instant;

const NS: [usize; 5] = [200, 1_000, 5_000, 20_000, 100_000];
const RODADAS: usize = 5;
const BLOCO: usize = 30;

/// Um quadro com `n` cartões repartidos por `listas` listas, aberto numa aba.
fn monta(n: usize, listas: usize) -> (Mesa, KanbanId) {
    let mut m = Mesa::new();
    let shared = m.hero.documents.kanbans_shared();
    let id = {
        let mut set = shared.borrow_mut();
        let id = set.create("Teto");
        let k = set.doc_mut(id).expect("kanban");
        let mut next = k.mint() - 1;
        let mut nid = || {
            next += 1;
            next
        };
        let mut ops = Vec::with_capacity(n + listas + 1);
        let tag = LabelId(nid());
        ops.push(KanbanOp::put(Record::Label(Label {
            id: tag,
            name: "Shot".into(),
            color: Some(LabelColor {
                hue: Hue::Purple,
                shade: Shade::Normal,
            }),
            meta: Meta::default(),
        })));
        let mut pos: Option<FracKey> = None;
        let mut key = || {
            let p = FracKey::between(pos.as_ref(), None);
            pos = Some(p.clone());
            p
        };
        let ids: Vec<ListId> = (0..listas)
            .map(|i| {
                let lid = ListId(nid());
                ops.push(KanbanOp::put(Record::List(List {
                    id: lid,
                    title: format!("Sequence {}", i + 1),
                    pos: key(),
                    color: None,
                    card_limit: None,
                    archived: false,
                    meta: Meta::default(),
                    watchers: Vec::new(),
                })));
                lid
            })
            .collect();
        for c in 0..n {
            ops.push(KanbanOp::put(Record::card(
                Card {
                    id: CardId(nid()),
                    list: ids[c % listas],
                    pos: key(),
                    title: format!("Shot {:05} \u{2014} layout, anim, light", c + 1),
                    description: String::new(),
                    labels: if c % 3 == 0 { vec![tag] } else { Vec::new() },
                    members: if c % 5 == 0 {
                        vec![Member("Enio".into())]
                    } else {
                        Vec::new()
                    },
                    start: None,
                    due: None,
                    cover: None,
                    fields: std::collections::BTreeMap::new(),
                    location: String::new(),
                    votes: Vec::new(),
                    created: Moment(0),
                    archived: false,
                    meta: Meta::default(),
                    watchers: Vec::new(),
                    origin: None,
                    template: false,
                }
                .into(),
            )));
        }
        ph2d_kanban_model::apply_batch(k, ops);
        id
    };
    m.docs().follow_project(shared);
    m.docs().open(id);
    (m, id)
}

/// Um quadro medido: o relógio e o `follow_surfaces` ficam fora; dentro, o `frame` e a pintura.
fn quadro_medido(m: &mut Mesa) -> f64 {
    m.relogio();
    let t = Instant::now();
    ph2d_app_kanban::frame(&mut m.hero);
    let a = t.elapsed();
    ph2d_editor_core::screens::hero::document_tabs::follow_surfaces(&mut m.hero);
    let mut scene = ph2d_vector::VectorScene::new();
    let t = Instant::now();
    m.pinta(&mut scene);
    (a + t.elapsed()).as_secs_f64() * 1e3
}

struct Variante {
    nome: String,
    mesa: Mesa,
    arrasto: Option<(f32, f32)>,
    blocos: Vec<f64>,
    primeiro: f64,
}

/// Só o que a disposição pede ao modelo por quadro: `cards_in` de cada lista (varre TODOS os
/// cartões e ordena os dela).
fn so_cards_in(m: &Mesa, id: KanbanId) -> f64 {
    let set = m.hero.documents.kanbans();
    let k = set.get(id).expect("kanban");
    let t = Instant::now();
    let mut tot = 0usize;
    for l in k.lists_in_order() {
        tot += k.cards_in(l.id).len();
    }
    std::hint::black_box(tot);
    t.elapsed().as_secs_f64() * 1e3
}

fn loadavg() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .unwrap_or_default()
        .trim()
        .to_owned()
}

#[test]
#[ignore = "sonda de medição: corre à mão em --profile smoke"]
fn teto_de_cartoes_a_60_fps() {
    eprintln!("loadavg no início: {}", loadavg());
    let mut vs: Vec<Variante> = Vec::new();
    for n in NS {
        for listas in [10usize, 1] {
            for (arrasto, reduzido) in [(false, false), (true, false), (false, true)] {
                if reduzido && listas == 1 {
                    continue;
                }
                let (mut mesa, id) = monta(n, listas);
                // Movimento reduzido: as molas chegam já (a parte delas no quadro).
                mesa.hero.motion.set_reduced_motion(reduzido);
                let primeiro = quadro_medido(&mut mesa);
                quadro_medido(&mut mesa);
                let mut pos = None;
                if arrasto {
                    let l = mesa.lista(id, "Sequence 1");
                    let c = mesa.cartoes(id, l)[0];
                    let r = mesa.rect_cartao(id, c);
                    let p = crate::pega(r);
                    let alvo = (600.0, 300.0);
                    mesa.arrasta(p, alvo, 4);
                    assert!(
                        mesa.docs().probe_drop(id.0).is_some(),
                        "o arrasto está em curso"
                    );
                    pos = Some(alvo);
                }
                quadro_medido(&mut mesa);
                vs.push(Variante {
                    nome: format!(
                        "{n:>7} cartões · {listas:>2} lista(s){}",
                        if arrasto {
                            " · arrasto"
                        } else if reduzido {
                            " · sem molas"
                        } else {
                            ""
                        }
                    ),
                    mesa,
                    arrasto: pos,
                    blocos: Vec::new(),
                    primeiro,
                });
            }
        }
    }
    let ids: Vec<KanbanId> = vs.iter().map(|_| KanbanId(1)).collect();
    let mut so: Vec<Vec<f64>> = vec![Vec::new(); vs.len()];
    for r in 0..RODADAS {
        let len = vs.len();
        for j in 0..len {
            let i = (j + r * 3) % len;
            let v = &mut vs[i];
            let mut soma = 0.0;
            for f in 0..BLOCO {
                if let Some((x, y)) = v.arrasto {
                    let dx = if f % 2 == 0 { 1.0 } else { -1.0 };
                    v.mesa.move_(x + dx, y);
                }
                soma += quadro_medido(&mut v.mesa);
            }
            #[allow(clippy::cast_precision_loss)]
            v.blocos.push(soma / BLOCO as f64);
            so[i].push(so_cards_in(&v.mesa, ids[i]));
        }
    }
    eprintln!("loadavg no fim: {}", loadavg());
    eprintln!(
        "{:<40} {:>10} {:>10} {:>12} {:>12}",
        "variante", "mín ms", "mediana", "1.º quadro", "cards_in ms"
    );
    for (i, v) in vs.iter_mut().enumerate() {
        v.blocos.sort_by(f64::total_cmp);
        so[i].sort_by(f64::total_cmp);
        eprintln!(
            "{:<40} {:>10.3} {:>10.3} {:>12.1} {:>12.3}",
            v.nome,
            v.blocos[0],
            v.blocos[v.blocos.len() / 2],
            v.primeiro,
            so[i][0]
        );
    }
}
