//! **SONDA — reproduzir uma MACRO do Motion nó a nó** (linha `Macros-Automação`).
//!
//! Uma macro grava os `GraphIntent`s que o painel emite e reproduz-os pela MESMA porta
//! (`apply_graph_intents` + o dreno dos params), cozinhando o quadro entre cada passo para o
//! artista ver o efeito de cada nó. Esta sonda mede (a) o preço de cada passo, (b) o que cada
//! passo muda no que se vê, (c) o determinismo e (d) a dependência do RELÓGIO da cena.
//!
//! A cena é a regra do dono: FORMAS (`source.shape` → `motion.duplicator`) e SIMULAÇÃO com campo
//! (`motion.integrate` ⇄ `field.box` → `force.wind` → `force.drag`).
//!
//! O cozimento é a metade CPU do `quadro::coze_o_quadro` (escopos, leques, preguiça, `ticks_owed`,
//! `advance_or_scrub_scoped`); a placa, a lente/filtro, a origem do objecto e o LOD ficam de fora.
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo test -p ph2d-app-motion --profile smoke --lib -- \
//!   --ignored --nocapture --test-threads=1 macro_replay
//! ```

use super::{apply_graph_intents, output_nodes, params};
use crate::motion_state::MotionState;
use ph2d_nodegraph::graph::{Edge, NodeId};
use ph2d_panel_motion_graph::push_intent;
// A macro e o retrato são os do gate 3 (`macro_track::gate_tests`): a sonda mede o que o gate prova.
use crate::macro_track::gate_tests::{Gesto as Passo, MACRO, estado, intent_de, retrato};

const DT: f64 = 1.0 / 60.0;
const RODADAS: usize = 15;

/// Um QUADRO do produto: drena as intenções do grafo e depois as dos params (a ordem do
/// `dispatch`). Devolve os nós que nasceram.
fn aplica(m: &mut MotionState, toasts: &mut ph2d_editor_core::ToastQueue) -> Vec<NodeId> {
    let antes: Vec<NodeId> = m.doc.graph.nodes().iter().map(|n| n.id).collect();
    apply_graph_intents(
        m,
        &mut ph2d_core::Playhead::default(),
        toasts,
        &mut ph2d_editor_core::screens::layout::CenterSplit::None,
    );
    let store = ph2d_editor_core::interaction::WidgetStore::default();
    params::apply_param_edits_for_tests(m, &store, toasts);
    let mut novos: Vec<NodeId> = m
        .doc
        .graph
        .nodes()
        .iter()
        .map(|n| n.id)
        .filter(|n| !antes.contains(n))
        .collect();
    novos.sort_by_key(|n| n.0);
    novos
}

/// A metade CPU do `coze_o_quadro`, no `tick`. Devolve se cozinhou.
fn coze(m: &mut MotionState, tick: u64) -> bool {
    coze_com(m, tick, false)
}

/// `resim = true`: em vez da porta do quadro (que num tique REPETIDO re-coze sobre o estado que
/// está no ar), re-simula da SEMENTE até `tick` (`scrub_to_scoped` com o anel que o `mark_dirty`
/// esvaziou) — o que uma macro faria para mostrar o efeito "verdadeiro" do passo.
fn coze_com(m: &mut MotionState, tick: u64, resim: bool) -> bool {
    // As MEMBRANAS (forma viva, texto, áudio, tabela…) — sem elas a `source.shape` não desenha.
    crate::motion_externals::publish_all(m, tick as f64 * DT);
    m.sinks = crate::motion_objeto::sinks_visiveis(m, output_nodes(&m.doc.graph));
    let scopes = ph2d_node_motion_time_remap::time_scopes(&m.doc.graph, &m.registry);
    let mut fans = ph2d_node_motion_trail::time_fans(&m.doc.graph, &m.registry, DT);
    fans.extend(ph2d_node_motion_emitter::time_fans(
        &m.doc.graph,
        &m.registry,
        DT,
    ));
    fans.extend(ph2d_node_motion_clone::fan::time_fans(
        &m.doc.graph,
        &m.registry,
        DT,
    ));
    m.pump.set_time_fans(fans);
    m.pump
        .cook
        .set_lazy_branches(ph2d_node_value_switch::lazy::plan(
            &m.doc.graph,
            &m.registry,
        ));
    if resim {
        m.pump.set_separa_o_desenho(true);
        return m.pump.scrub_to_scoped(
            &m.doc.graph,
            &m.registry,
            &m.sinks,
            tick,
            |t| t as f64 * DT,
            m.default_uv_rect,
            m.default_size,
            &scopes,
        );
    }
    let tiques = super::ticks_owed(m.pump.last_cooked_tick(), tick);
    let ultimo = *tiques.end();
    let mut cozeu = false;
    for t in tiques {
        m.pump.set_separa_o_desenho(t == ultimo);
        cozeu |= m.pump.advance_or_scrub_scoped(
            &m.doc.graph,
            &m.registry,
            &m.sinks,
            t,
            |t| t as f64 * DT,
            m.default_uv_rect,
            m.default_size,
            &scopes,
        );
    }
    cozeu
}

fn fnv(v: &[u32]) -> u64 {
    v.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, x| {
        (h ^ u64::from(*x)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// O grafo como conjunto comparável (tipos, params e fios).
fn grafo(m: &MotionState) -> String {
    let g = &m.doc.graph;
    let mut s = String::new();
    for n in g.nodes() {
        let mut ps: Vec<(String, u32)> = g
            .node_param_overrides(n.id)
            .map(|p| {
                p.iter()
                    .map(|(k, v)| (k.to_string(), v.to_bits()))
                    .collect()
            })
            .unwrap_or_default();
        ps.sort();
        s.push_str(&format!("{}:{}:{ps:?}\n", n.id.0, n.type_name));
    }
    let mut es: Vec<Edge> = g.edges().to_vec();
    es.sort_by_key(|e| (e.to.0.0, e.to.1, e.from.0.0, e.from.1, e.delayed));
    for e in es {
        s.push_str(&format!("{e:?}\n"));
    }
    s
}

/// Uma linha da tabela.
#[derive(Clone, Default)]
struct Medida {
    apply_ms: f64,
    cook_ms: f64,
    cozeu: bool,
    quads: usize,
    vecs: usize,
    marca: u64,
}

/// Como a reprodução coze entre passos.
#[derive(Clone, Copy)]
enum Modo<'a> {
    /// O passo `i` coze no tique `f(i)`; `bool` = re-simula da semente ([`coze_com`]).
    Cada(&'a dyn Fn(usize) -> u64, bool),
    /// Não coze entre passos; um cozimento no fim, neste tique.
    NoFim(u64),
}

/// Reproduz a macro passo a passo.
fn reproduz(modo: Modo<'_>) -> (MotionState, Vec<Medida>, usize) {
    let mut m = estado();
    let mut toasts = ph2d_editor_core::ToastQueue::default();
    let mut slots: Vec<NodeId> = Vec::new();
    let mut linhas = Vec::with_capacity(MACRO.len());
    for (i, p) in MACRO.iter().enumerate() {
        push_intent(intent_de(*p, &slots, i));
        let t0 = std::time::Instant::now();
        let novos = aplica(&mut m, &mut toasts);
        let apply_ms = t0.elapsed().as_secs_f64() * 1e3;
        if let Passo::Add(_) = p {
            assert_eq!(novos.len(), 1, "passo {i}: um AddNode cria UM nó");
            slots.push(novos[0]);
        }
        let mut l = Medida {
            apply_ms,
            ..Medida::default()
        };
        if let Modo::Cada(f, resim) = modo {
            let t1 = std::time::Instant::now();
            l.cozeu = coze_com(&mut m, f(i), resim);
            l.cook_ms = t1.elapsed().as_secs_f64() * 1e3;
            l.quads = m.pump.instances.len();
            l.vecs = m.pump.vector_instances.len();
            l.marca = fnv(&retrato(&m));
        }
        linhas.push(l);
    }
    if let Modo::NoFim(t) = modo {
        coze(&mut m, t);
    }
    (m, linhas, toasts.len())
}

/// Tudo de uma vez: os MESMOS intents num lote só, um apply, um cozimento. Os ids vêm de uma
/// reprodução de referência (a macro precisa deles determinísticos — confere-se abaixo).
fn de_uma_vez(slots: &[NodeId], tick: u64) -> MotionState {
    let mut m = estado();
    let mut toasts = ph2d_editor_core::ToastQueue::default();
    for (i, p) in MACRO.iter().enumerate() {
        push_intent(intent_de(*p, slots, i));
    }
    let novos = aplica(&mut m, &mut toasts);
    assert_eq!(
        novos, slots,
        "num lote só os ids nascem iguais aos do passo a passo"
    );
    coze(&mut m, tick);
    m
}

fn slots_de(m: &MotionState) -> Vec<NodeId> {
    let mut v: Vec<NodeId> = m.doc.graph.nodes().iter().map(|n| n.id).collect();
    v.sort_by_key(|n| n.0);
    v
}

/// Marcha contígua até `ate`.
fn marcha(m: &mut MotionState, ate: u64) {
    let de = m.pump.last_cooked_tick().map_or(0, |t| t + 1);
    for t in de..=ate {
        coze(m, t);
    }
}

fn carga() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .map(|s| s.split_whitespace().take(3).collect::<Vec<_>>().join(" "))
        .unwrap_or_else(|_| "?".into())
}

#[test]
#[ignore = "sonda: medição da reprodução de uma macro — corra à mão"]
fn macro_replay_por_passo() {
    eprintln!(
        "\n  ═══ MACRO: reprodução nó a nó (CPU) — mínimo de {RODADAS} rodadas intercaladas ═══"
    );
    eprintln!("  load antes: {}", carga());
    let zero: &dyn Fn(usize) -> u64 = &|_| 0;
    let sessenta: &dyn Fn(usize) -> u64 = &|_| 60;
    // As três variantes: parado no 0 · parado no 60 (porta do quadro) · 60 re-simulado da semente.
    let modos = [
        Modo::Cada(zero, false),
        Modo::Cada(sessenta, false),
        Modo::Cada(sessenta, true),
    ];
    let mut min: Vec<Vec<Medida>> = vec![Vec::new(); modos.len()];
    for r in 0..RODADAS {
        for j in 0..modos.len() {
            let v = (r + j) % modos.len();
            let (_, linhas, _) = reproduz(modos[v]);
            if min[v].is_empty() {
                min[v] = linhas;
                continue;
            }
            for (a, b) in min[v].iter_mut().zip(linhas) {
                assert_eq!(a.marca, b.marca, "rodadas diferem no que se vê");
                a.apply_ms = a.apply_ms.min(b.apply_ms);
                a.cook_ms = a.cook_ms.min(b.cook_ms);
            }
        }
    }
    eprintln!(
        "  {:>2} │ {:<28} │ {:>6} │ {:>6} │ {:>6} │ {:>6} │ {:>4} │ muda? 0 · 60 · 60r",
        "#", "passo", "apply", "c@0", "c@60", "c@60r", "vecs"
    );
    let mut anterior = [0u64; 3];
    let mut soma = [0.0f64; 4];
    for (i, p) in MACRO.iter().enumerate() {
        let rotulo = match p {
            Passo::Add(t) => format!("Add {t}"),
            Passo::Liga(a, ap, b, bp) => format!("Connect s{a}.{ap}->s{b}.{bp}"),
            Passo::Param(n, k, v) => format!("Set s{n}.{k}={v}"),
        };
        let l: Vec<&Medida> = min.iter().map(|v| &v[i]).collect();
        let apply = l.iter().map(|x| x.apply_ms).fold(f64::INFINITY, f64::min);
        let muda: Vec<&str> = (0..3)
            .map(|v| {
                if l[v].marca != anterior[v] {
                    "SIM"
                } else {
                    "-"
                }
            })
            .collect();
        eprintln!(
            "  {i:>2} │ {rotulo:<28} │ {apply:>6.3} │ {:>6.3} │ {:>6.3} │ {:>6.3} │ {:>4} │ {:>3} · {:>3} · {:>3}",
            l[0].cook_ms,
            l[1].cook_ms,
            l[2].cook_ms,
            l[2].vecs + l[2].quads,
            muda[0],
            muda[1],
            muda[2]
        );
        for v in 0..3 {
            anterior[v] = l[v].marca;
            soma[v + 1] += l[v].cook_ms;
        }
        soma[0] += apply;
    }
    assert!(
        min.iter().all(|v| v.last().is_some_and(|l| l.vecs > 0)),
        "o último passo cozeu VAZIO — a sonda estaria a medir o nada"
    );
    eprintln!(
        "  soma (ms): apply {:.3} · c@0 {:.3} · c@60 {:.3} · c@60r {:.3}",
        soma[0], soma[1], soma[2], soma[3]
    );
    eprintln!("  load depois: {}\n", carga());
}

#[test]
#[ignore = "sonda: determinismo e relógio da reprodução de uma macro — corra à mão"]
fn macro_replay_determinismo_e_relogio() {
    const T: u64 = 120;
    let zero: &dyn Fn(usize) -> u64 = &|_| 0;
    // A — passo a passo, parado no tique 0, cozendo entre passos.
    let (mut a, _, toasts_a) = reproduz(Modo::Cada(zero, false));
    let (mut a2, _, _) = reproduz(Modo::Cada(zero, false));
    // C — passo a passo SEM cozer entre passos; um cozimento no fim.
    let (mut c, _, _) = reproduz(Modo::NoFim(0));
    let slots = slots_de(&a);
    // B — tudo num lote só.
    let mut b = de_uma_vez(&slots, 0);
    eprintln!(
        "\n  ═══ MACRO: determinismo (c) e relógio (d) ═══  load {}",
        carga()
    );
    eprintln!("  toasts no passo a passo: {toasts_a}");
    eprintln!(
        "  sinks {:?} · erro {:?} · a lei {} · quads {} · vecs {}",
        a.sinks,
        a.pump.last_error(),
        a.pump.a_lei(),
        a.pump.instances.len(),
        a.pump.vector_instances.len()
    );
    assert!(
        !a.pump.vector_instances.is_empty(),
        "a cena cozeu VAZIA — a comparação ao bit seria sobre nada"
    );
    let g = |x: &MotionState| grafo(x);
    eprintln!(
        "  grafo  A==B {}  A==C {}  A==A' {}",
        g(&a) == g(&b),
        g(&a) == g(&c),
        g(&a) == g(&a2)
    );
    let r = |x: &MotionState| retrato(x);
    eprintln!(
        "  tique 0   A==A' {}  A==B {}  C==B {}   (n = {} palavras)",
        r(&a) == r(&a2),
        r(&a) == r(&b),
        r(&c) == r(&b),
        r(&b).len()
    );
    let difere = |x: &MotionState, y: &MotionState| {
        x.pump
            .vector_instances
            .iter()
            .zip(&y.pump.vector_instances)
            .map(|(p, q)| {
                ((p.world_pos[0] - q.world_pos[0]).abs())
                    .max((p.world_pos[1] - q.world_pos[1]).abs())
            })
            .fold(0.0f32, f32::max)
    };
    eprintln!("  tique 0   max |Δpos| A×B = {:.6}", difere(&a, &b));
    // E — o A depois de um SALTO (scrub) para T: re-simula da semente sob o grafo final.
    let (mut e, _, _) = reproduz(Modo::Cada(zero, false));
    coze(&mut e, T);
    for x in [&mut a, &mut a2, &mut b, &mut c] {
        marcha(x, T);
    }
    eprintln!(
        "  tique {T} (marcha contígua)  A==A' {}  A==B {}  C==B {}  max|Δpos| A×B = {:.6}",
        r(&a) == r(&a2),
        r(&a) == r(&b),
        r(&c) == r(&b),
        difere(&a, &b)
    );
    eprintln!("  tique {T} E (scrub da semente) == B {}", r(&e) == r(&b));
    // D — a TOCAR: o passo i cozido no tique i·k. Mesmo k → iguais? k diferente → iguais?
    let k4: &dyn Fn(usize) -> u64 = &|i| i as u64 * 4;
    let k6: &dyn Fn(usize) -> u64 = &|i| i as u64 * 6;
    let (mut d4, _, _) = reproduz(Modo::Cada(k4, false));
    let (mut d4b, _, _) = reproduz(Modo::Cada(k4, false));
    let (mut d6, _, _) = reproduz(Modo::Cada(k6, false));
    const TD: u64 = 200;
    for x in [&mut d4, &mut d4b, &mut d6, &mut b] {
        marcha(x, TD);
    }
    eprintln!(
        "  a tocar, tique {TD}: k=4 vs k=4 {}  k=4 vs k=6 {} (max|Δpos| {:.6})  k=4 vs B {} (max|Δpos| {:.6})",
        r(&d4) == r(&d4b),
        r(&d4) == r(&d6),
        difere(&d4, &d6),
        r(&d4) == r(&b),
        difere(&d4, &b)
    );
    let mut d4s = reproduz(Modo::Cada(k4, false)).0;
    coze(&mut d4s, 0);
    coze(&mut d4s, TD);
    eprintln!(
        "  a tocar k=4, depois scrub 0 → {TD}: == B {}",
        r(&d4s) == r(&b)
    );
    // P — PARADO a meio da linha do tempo (tique 60), como o artista que edita em pausa.
    let sessenta: &dyn Fn(usize) -> u64 = &|_| 60;
    let (p, _, _) = reproduz(Modo::Cada(sessenta, false));
    let (p2, _, _) = reproduz(Modo::Cada(sessenta, false));
    let (pr, _, _) = reproduz(Modo::Cada(sessenta, true));
    let b60 = de_uma_vez(&slots, 60);
    eprintln!(
        "  parado no 60: porta do quadro P==P' {}  P==B60 {} (max|Δpos| {:.6}) · re-sim R==B60 {}\n",
        r(&p) == r(&p2),
        r(&p) == r(&b60),
        difere(&p, &b60),
        r(&pr) == r(&b60)
    );
}
