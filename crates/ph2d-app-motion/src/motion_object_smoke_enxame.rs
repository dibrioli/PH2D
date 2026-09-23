//! ⭐⭐⭐ **O ENXAME** (`PH2D_MOTION_OBJ_SMOKE=16`) — a cena do smoke dos ciclos 10 e 11, construída
//! pela regra do dono de 2026-09-23: *toda cena de smoke do Motion tem FORMAS e SIMULAÇÃO com
//! campos* (doc 103 §1). As cenas que eu tinha dado antes (`=93`, `=108`, `=126`) mostravam
//! gizmos ou ficavam paradas.
//!
//! **O que ela tem, e porquê cada peça:**
//! - **uma FORMA que a placa desenha** — um objecto de imagem (`source.object` sobre um sprite)
//!   carimbado numa grelha. ⚠️ Uma forma vectorial viva (`source.shape`) RECUSA a placa hoje, e a
//!   cena existe para mostrar a placa.
//! - **DUAS saídas que PARTILHAM o objecto** — o que o ciclo 11 destravou: as duas vão à placa
//!   no MESMO buffer, cada uma com o seu estágio.
//! - **SIMULAÇÃO com CAMPOS DE FORÇA** — cada enxame é um `motion.integrate` com o laço de estado
//!   (o `pre` do tique anterior) a entrar numa cadeia de forças: à esquerda um REDEMOINHO
//!   (`force.vortex` + `force.drag` + `force.curl`), à direita uma NUVEM que respira à volta de um
//!   ÍMAN (`force.attractor` + `force.curl` + `force.drag`).
//!
//! ⚠️ **A rota é a HÍBRIDA** (a simulação e o carimbo na CPU — o duplicador não despacha —, e o
//! posicionamento e o desenho das duas saídas na placa) — e o gate da cena exige-o pelo nome.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};
use ph2d_render::Sprite;

/// O nome do objecto que os dois enxames carimbam.
pub(super) const PARTICULA: &str = "Particle";

/// O lado de uma partícula — pequeno, para um enxame de `LADO_DA_GRELHA²` se ler como enxame e não
/// como um tapete.
const LADO: f32 = 0.08;
/// A grelha de partida de cada enxame: `LADO_DA_GRELHA²` cópias.
pub(super) const LADO_DA_GRELHA: f32 = 40.0;
/// Onde cada enxame vive — a distância do centro dele ao meio da tela.
pub(super) const CENTRO: f32 = 3.2;
/// O passo da grelha de partida (`LADO_DA_GRELHA · PASSO` de lado).
const PASSO: f32 = 0.06;
/// O raio dos campos — LARGO de propósito: um íman cujo raio o enxame ultrapassa deixa as peças
/// de fora sem puxão nenhum, e o ruído leva-as (medido com raio `6`: divergia).
const ALCANCE: f32 = 20.0;

/// As intensidades dos campos, num sítio só — afinadas pela régua que virou o gate
/// `os_enxames_ficam_cada_um_no_seu_lado`.
struct Forcas {
    vortex: f32,
    iman_fraco: f32,
    iman: f32,
    ruido: f32,
    ruido_leve: f32,
    arrasto: f32,
}
const F: Forcas = Forcas {
    vortex: 3.0,
    iman_fraco: 5.0,
    iman: 3.5,
    ruido: 2.0,
    ruido_leve: 1.5,
    arrasto: 1.0,
};

pub(super) fn run(cx: &mut crate::motion_scene_ctx::MotionSceneCtx<'_>) {
    cx.sim.world_mut().spawn((
        Transform::from_translation(Vec2::new(0.0, -4.0)),
        Sprite::atlas(super::DEMO_TILE_KEY, [LADO, LADO], [1.0, 1.0, 1.0, 1.0]),
        Name::new(PARTICULA),
    ));
    let sinks = build(&mut cx.motion.doc.graph, PARTICULA);
    cx.motion.sinks.extend(sinks);
    let _ = cx
        .tools
        .set_active(&ph2d_editor_core::ToolId::new("motion"));
    eprintln!(
        "[motion.obj smoke =16] O ENXAME. Dois enxames da MESMA imagem ('Particle', o quadradinho \
         laranja -- o original esta' sozinho em baixo, ao centro), cada um com {n} copias, movidos \
         por CAMPOS DE FORCA. Tudo se MEXE sozinho, sem carregar em Play:\n  \
         ESQUERDA = um REDEMOINHO (Vortex + um iman fraco + ruido): as copias giram em espiral.\n  \
         DIREITA  = um IMAN (Attractor + ruido): a nuvem encolhe e RESPIRA a volta do centro.\n  \
         Os dois enxames partem do MESMO objecto -- e' o que esta fase ensina: um desenho com \
         DUAS saidas passou a ser feito pela PLACA GRAFICA.",
        n = (LADO_DA_GRELHA * LADO_DA_GRELHA) as u32
    );
}

fn wire(g: &mut Graph, a: NodeId, ap: u16, b: NodeId, bp: u16, delayed: bool) {
    g.connect(Edge {
        from: (a, ap),
        to: (b, bp),
        delayed,
    })
    .expect("connect");
}

/// Um enxame: `grid → integrate ← forças ← pre(integrate)` simula os PONTOS, o `duplicator` veste
/// cada ponto com o objecto partilhado, e o `move` (o estágio da PLACA) põe o enxame no seu lado.
///
/// ⚠️ **A simulação vem ANTES do carimbo, e não é só gosto:** é a ordem de quem faz isto noutros
/// programas (simular pontos, copiar a forma para os pontos) **e** a única que o planeador aceita
/// hoje — um `motion.integrate` DEPOIS do carimbo recua da placa, porque o carimbo corre na CPU e
/// o planeador não consegue PROVAR que colunas ele entrega (a chave `id` do integrador). Medido ao
/// construir esta cena; dívida nomeada no doc 119 §10.
fn enxame(g: &mut Graph, objecto: NodeId, dx: f32, redemoinho: bool, y: f32) -> NodeId {
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", LADO_DA_GRELHA);
    g.set_param(grid, "cols", LADO_DA_GRELHA);
    g.set_param(grid, "gap_x", PASSO);
    g.set_param(grid, "gap_y", PASSO);
    let ig = g.add_node("motion.integrate");
    let dup = g.add_node("motion.duplicator");
    let mv = g.add_node("motion.move");
    g.set_param(mv, "dx", dx);
    g.set_param(mv, "dy", 0.0);
    let out = g.add_node("motion.output");
    let forcas: Vec<NodeId> = if redemoinho {
        let vortex = g.add_node("force.vortex");
        g.set_param(vortex, "strength", F.vortex);
        g.set_param(vortex, "radius", ALCANCE);
        g.set_label(vortex, "Whirl (Vortex)");
        // Um íman FRACO dentro do redemoinho: sem ele a força tangencial atira as peças para fora
        // (medido: sem ele o p95 do enxame passa de 2 m a 17 m em 10 s).
        let im = g.add_node("force.attractor");
        g.set_param(im, "strength", F.iman_fraco);
        g.set_param(im, "radius", ALCANCE);
        // E um RUÍDO leve: sem ele o equilíbrio vórtice × íman é um anel FINO (medido na foto),
        // que se lê como um círculo parado e não como um redemoinho.
        let curl = g.add_node("force.curl");
        g.set_param(curl, "strength", F.ruido_leve);
        g.set_param(curl, "scale", 0.8);
        g.set_param(curl, "speed", 0.6);
        let drag = g.add_node("force.drag");
        g.set_param(drag, "coefficient", F.arrasto);
        vec![vortex, im, curl, drag]
    } else {
        let im = g.add_node("force.attractor");
        g.set_param(im, "strength", F.iman);
        g.set_param(im, "radius", ALCANCE);
        g.set_label(im, "Magnet (Attractor)");
        let curl = g.add_node("force.curl");
        g.set_param(curl, "strength", F.ruido);
        g.set_param(curl, "scale", 0.8);
        g.set_param(curl, "speed", 0.6);
        let drag = g.add_node("force.drag");
        g.set_param(drag, "coefficient", F.arrasto);
        vec![im, curl, drag]
    };
    wire(g, grid, 0, ig, 0, false);
    // O laço que o artista nunca desenha: o estado do tique anterior entra na cadeia de forças.
    wire(g, ig, 0, forcas[0], 0, true);
    for par in forcas.windows(2) {
        wire(g, par[0], 0, par[1], 0, false);
    }
    wire(g, *forcas.last().expect("forcas"), 0, ig, 1, false);
    wire(g, objecto, 0, dup, 0, false);
    wire(g, ig, 0, dup, 1, false);
    wire(g, dup, 0, mv, 0, false);
    wire(g, mv, 0, out, 0, false);
    for (i, n) in [grid, ig, dup, mv, out].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 210.0 + i as f32 * 210.0,
                y,
            },
        );
    }
    for (i, n) in forcas.iter().enumerate() {
        g.set_pos(
            *n,
            Pos {
                x: 210.0 + i as f32 * 180.0,
                y: y + 150.0,
            },
        );
    }
    g.set_label(dup, "Dress With The Particle");
    g.set_label(
        out,
        if redemoinho {
            "Whirl Swarm"
        } else {
            "Magnet Swarm"
        },
    );
    out
}

/// O objecto (PARTILHADO pelas duas saídas) e os dois enxames. Devolve as duas saídas.
pub(super) fn build(g: &mut Graph, nome: &str) -> Vec<NodeId> {
    let src = g.add_node("source.object");
    g.set_text_param(src, "object", nome);
    g.set_pos(src, Pos { x: 0.0, y: 0.0 });
    g.set_label(src, "The Particle");
    let esquerda = enxame(g, src, -CENTRO, true, -300.0);
    let direita = enxame(g, src, CENTRO, false, 300.0);
    vec![esquerda, direita]
}

#[cfg(test)]
#[path = "motion_object_smoke_enxame_tests.rs"]
mod tests;
