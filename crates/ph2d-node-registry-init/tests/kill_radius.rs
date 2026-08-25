//! **O `Kill Radius` do `force.attractor`, MEDIDO** (doc 89, folha 02) — e a ressalva que
//! a célula deixou por conferir.
//!
//! A célula dava a cura como exprimível:
//!
//! > *"**SIM.** `motion.falloff(Circle, center=target, radius=kill_r) → motion.cull(mode =
//! > Falloff, amount≈1, invert=1)` … ⚠️ **Ressalva não medida:** isto muda a CONTAGEM
//! > (ADR-0136) dentro do laço `pre` do `motion.integrate`, que pareia por `id` — se é
//! > estável ali, não conferi"*
//!
//! ## As duas metades, medidas
//!
//! ⭐ **A ressalva está RESPONDIDA e é boa notícia:** cortar dentro do laço `pre` **não**
//! quebra o pareamento por `id`. Medido em 180 tiques, **zero** saltos de estado — o
//! integrador emparelha por identidade e uma linha que falta num tique é simplesmente uma
//! que não recebeu força, não uma que trocou de dono.
//!
//! ⛔ **E a cura está REFUTADA, por uma razão que a célula não considerou: naquela
//! arquitetura nada pode MATAR.** O `motion.emitter` é uma janela em FORMA FECHADA — ele
//! recalcula a população inteira a cada tique a partir do playhead. Um `motion.cull`:
//!
//! - **dentro do laço `pre`** tira a linha do ramo das FORÇAS, e mais nada. A contagem no
//!   integrador **nunca cai** (medido: `1 → 16 → 31 → 46 → 61 → 76` com o cull ligado);
//! - **depois do integrador** tira a linha do que o SINK vê — mas o emissor continua a
//!   emiti-la e o integrador continua a integrá-la. Ela fica **escondida, não morta**, e a
//!   pergunta é re-feita a cada tique sobre uma posição viva.
//!
//! ⇒ **Um `cull` é um FILTRO, e um filtro não é uma morte.** A diferença é observável: uma
//! partícula que atravesse o raio some e **volta** do outro lado, porque nada guardou que
//! ela tinha morrido.
//!
//! ## ⭐⭐ E é por isso que o item não é um param do `force.attractor`
//!
//! Um *Kill Radius* pertence a quem é DONO da população. Nas duas arquiteturas de partícula
//! desta casa a resposta é diferente, e nenhuma delas é a força:
//!
//! | arquitetura | quem é dono da população | um `cull` mata? |
//! |---|---|---|
//! | `sim.zone` + `sim.spawn` + `sim.lifetime` | a **zona** (a população é estado) | **sim** — é a morte-no-contato que a folha 13 já documenta |
//! | `motion.emitter` + `motion.integrate` | ninguém: a janela é forma fechada | **não** |
//!
//! ⇒ na zona o item **já é exprimível** e é ergonomia; no emissor ele é **estrutural**, e o
//! que o resolveria seria o emissor saber onde a partícula está DEPOIS das forças — que é
//! precisamente o que uma fonte sem estado não pode saber.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16, delayed: bool) {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed,
    })
    .expect("liga");
}

/// ⭐ **CORTAR DENTRO DO LAÇO `pre` NÃO QUEBRA O PAREAMENTO POR `id`** — a ressalva da
/// célula, respondida.
///
/// ⚠️ **E a mesma corrida mede a outra metade:** a contagem no integrador nunca cai, o que
/// prova que aquele corte não mata ninguém.
#[test]
fn culling_inside_the_pre_loop_never_breaks_the_state_pairing() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registram");
    let mut g = Graph::new();
    let em = g.add_node("motion.emitter");
    g.set_param(em, "rate", 30.0);
    g.set_param(em, "life", 3.0);
    g.set_param(em, "max", 128.0);
    g.set_param(em, "speed", 1.2);
    g.set_param(em, "spread", 160.0);

    let integ = g.add_node("motion.integrate");
    let fall = g.add_node("motion.falloff");
    g.set_param(fall, "shape", 0.0);
    g.set_param(fall, "radius", 0.6);
    let cull = g.add_node("motion.cull");
    g.set_param(cull, "mode", 1.0); // Falloff
    g.set_param(cull, "amount", 0.5);
    let att = g.add_node("force.attractor");
    g.set_param(att, "strength", 3.0);
    g.set_param(att, "radius", 6.0);

    wire(&mut g, em, 0, integ, 0, false);
    wire(&mut g, integ, 0, fall, 0, true);
    wire(&mut g, fall, 0, cull, 0, false);
    wire(&mut g, cull, 0, att, 0, false);
    wire(&mut g, att, 0, integ, 1, false);
    g.graph_validate(&reg);

    let mut cook = Cook::new();
    let mut jumps = 0_u32;
    let mut counts: Vec<usize> = Vec::new();
    let mut prev: Vec<(u32, [f32; 2])> = Vec::new();
    for k in 0..180 {
        let t = f64::from(k) / 60.0;
        cook.advance_tick(&g, &reg, t).expect("avanca");
        let out = cook.cook(&g, &reg, integ, t).expect("coze");
        let s = out[0].as_stream();
        let ids: Vec<u32> = match s.get("id") {
            Some(Column::Scalar(v)) => v.iter().map(|x| *x as u32).collect(),
            _ => Vec::new(),
        };
        let ps: Vec<[f32; 2]> = match s.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => Vec::new(),
        };
        let now: Vec<(u32, [f32; 2])> = ids.into_iter().zip(ps).collect();
        for (id, p) in &now {
            if let Some((_, q)) = prev.iter().find(|(j, _)| j == id) {
                // Um SALTO: a mesma identidade andou muito mais que um passo num tique.
                if (p[0] - q[0]).hypot(p[1] - q[1]) > 0.5 {
                    jumps += 1;
                }
            }
        }
        if k % 30 == 0 {
            counts.push(now.len());
        }
        prev = now;
    }
    println!("populacao no integrador a cada 30 tiques: {counts:?} · saltos: {jumps}");
    assert!(
        counts.last().copied().unwrap_or(0) > 40,
        "CONTROLE: a cadeia tem de estar a produzir populacao ({counts:?})"
    );
    assert_eq!(
        jumps, 0,
        "cortar dentro do laco `pre` quebrou o pareamento por `id`"
    );
    // ⛔ E a outra metade: a contagem no integrador SOBE — o corte nao matou ninguem.
    assert!(
        counts.windows(2).all(|w| w[1] >= w[0]),
        "a populacao no integrador tinha de nao cair: {counts:?}"
    );
}

/// Um atalho de leitura — a validação com a mensagem no sítio da chamada.
trait ValidateExt {
    fn graph_validate(&self, reg: &NodeRegistry);
}
impl ValidateExt for Graph {
    fn graph_validate(&self, reg: &NodeRegistry) {
        self.validate(reg).expect("a cadeia e' bem-tipada");
    }
}
