//! ⚡ **A ESCADA DOS TECTOS** (`PH2D_MOTION_OBJ_SMOKE=17`, ciclo 12, [doc 120]) — a cena que
//! MEDE quantos objectos o sistema aguenta, com o USO REAL do dono dentro:
//!
//! - **uma FORMA** — o objecto de imagem `Particle` (`source.object`), ou, com
//!   `PH2D_TECTO_FORMA=1`, uma ESTRELA vectorial viva (`source.shape`). ⚠️ As duas vão pela MESMA
//!   rota híbrida (o `motion.duplicator` é a fronteira do planeador); o que a estrela muda é o
//!   **desenho** — geometria vectorial por instância contra um quad do atlas;
//! - **SIMULAÇÃO com campos** — um emissor que enche a população e um integrador com redemoinho,
//!   ruído e arrasto (o laço de sempre: `integrate --pre--> forças → integrate`);
//! - **a população é um NÚMERO na linha de comando** (`PH2D_TECTO_N=<n>`, por omissão `16 384`),
//!   porque a escada é o que a tabela do ciclo precisa e o zoom de um slider não se alcança de
//!   uma corrida sem interface.
//!
//! ⚠️ **A população CHEIA leva `VIDA` segundos a chegar** (o emissor nasce `n / VIDA` por
//! segundo e cada partícula vive `VIDA`) — uma medição antes disso mede uma cena a meio de encher.
//!
//! ⛔ **O tecto por nó (`MAX_INSTANCIAS_POR_NO`) corta o carimbo:** acima de `32 768` a cena
//! desenha `32 768`, e o arranque DIZ quantas pediu e quantas vai ter. Medir acima do tecto é
//! uma compilação LOCAL com a constante levantada, e nunca um commit (doc 120 §3).
//!
//! [doc 120]: ../../../docs/Motion%20Nodes/120_ciclo_12_os_tectos_confortaveis.md

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};
use ph2d_render::Sprite;

/// O nome do objecto que a escada carimba.
pub(super) const PARTICULA: &str = "Particle";

/// O lado de uma partícula, em metros — o mesmo do enxame (`=16`).
const LADO: f32 = 0.08;
/// Quanto vive uma partícula, em segundos — e portanto quanto a população leva a encher.
pub(super) const VIDA: f32 = 4.0;
/// Os MEIOS-lados do rectângulo onde as partículas nascem, em metros — do tamanho da tela de
/// arranque, para a população inteira ficar À VISTA (a escada mede o desenho também).
const AREA: [f32; 2] = [9.0, 3.0];
/// A população de omissão — o tecto de fábrica de hoje dividido por dois, dentro dele.
pub(super) const N_OMISSAO: u32 = 16_384;

/// **A população pedida** — `PH2D_TECTO_N=<n>`; ausente, ilegível ou zero ⇒ [`N_OMISSAO`].
pub(super) fn populacao() -> u32 {
    populacao_por(std::env::var("PH2D_TECTO_N").ok().as_deref())
}

/// A LEI da porta acima, **pura** (*um gate que lê o ambiente mede a máquina*).
pub(super) fn populacao_por(valor: Option<&str>) -> u32 {
    valor
        .and_then(|v| v.trim().parse::<u32>().ok())
        .filter(|&n| n > 0)
        .unwrap_or(N_OMISSAO)
}

/// **A forma é a estrela vectorial?** — `PH2D_TECTO_FORMA=1`.
fn forma_vectorial() -> bool {
    forma_por(std::env::var("PH2D_TECTO_FORMA").ok().as_deref())
}

/// A LEI da porta acima, pura.
pub(super) fn forma_por(valor: Option<&str>) -> bool {
    matches!(valor.map(str::trim), Some("1"))
}

pub(super) fn run(cx: &mut crate::motion_scene_ctx::MotionSceneCtx<'_>) {
    cx.sim.world_mut().spawn((
        Transform::from_translation(Vec2::new(0.0, -4.5)),
        Sprite::atlas(super::DEMO_TILE_KEY, [LADO, LADO], [1.0, 1.0, 1.0, 1.0]),
        Name::new(PARTICULA),
    ));
    let n = populacao();
    let estrela = forma_vectorial();
    let sink = build(&mut cx.motion.doc.graph, PARTICULA, n, estrela);
    cx.motion.sinks.push(sink);
    let _ = cx
        .tools
        .set_active(&ph2d_editor_core::ToolId::new("motion"));
    let tecto = ph2d_nodegraph::node::MAX_INSTANCIAS_POR_NO;
    let vai_ter = (n as usize).min(tecto);
    eprintln!(
        "[motion.obj smoke =17] A ESCADA DOS TECTOS: {pediu} objectos pedidos, {vai_ter} desenhados \
         (o tecto por no' e' {tecto}). A forma e' {forma}. Cada objecto nasce, gira num redemoinho \
         com ruido e morre ao fim de {VIDA} s -- a populacao ENCHE em {VIDA} s e depois fica.\n  \
         Olhe a BARRA DE BAIXO: o `raw` e' a folga do quadro. Mude o numero com PH2D_TECTO_N=<n>.",
        pediu = n,
        forma = if estrela {
            "uma ESTRELA vectorial (cada uma desenhada como curva)"
        } else {
            "o quadradinho 'Particle' (cada um desenhado como imagem)"
        }
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

/// `emitter → integrate ← (vortex → curl → drag) ← pre(integrate)`, e o `duplicator` veste cada
/// partícula com a forma; o `move` é o estágio que a placa corre (o molde do enxame, `=16`).
pub(super) fn build(g: &mut Graph, nome: &str, n: u32, estrela: bool) -> NodeId {
    #[expect(clippy::cast_precision_loss, reason = "uma populacao, muito abaixo de 2^24")]
    let nf = n as f32;
    let forma = if estrela {
        let f = g.add_node("source.shape");
        #[expect(clippy::cast_precision_loss, reason = "um indice de enum")]
        let kind = ph2d_node_motion_shape::ALL_KINDS
            .iter()
            .position(|k| *k == ph2d_node_motion_shape::ShapeKind::Star)
            .expect("ha' estrela") as f32;
        g.set_param(f, ph2d_node_motion_shape::param::KIND, kind);
        g.set_param(f, ph2d_node_motion_shape::param::SIZE, LADO / 2.0);
        f
    } else {
        let f = g.add_node("source.object");
        g.set_text_param(f, "object", nome);
        f
    };
    let em = g.add_node("motion.emitter");
    g.set_param(em, "rate", nf / VIDA);
    g.set_param(em, "life", VIDA);
    g.set_param(em, "max", nf);
    g.set_param(em, "shape_mode", 3.0); // Rect (meios-lados)
    g.set_param(em, "shape_w", AREA[0]);
    g.set_param(em, "shape_h", AREA[1]);
    g.set_param(em, "speed", 0.3);
    g.set_param(em, "spread", 360.0);
    g.set_param(em, "seed", 12.0);
    let ig = g.add_node("motion.integrate");
    let vortex = g.add_node("force.vortex");
    g.set_param(vortex, "strength", 1.2);
    g.set_param(vortex, "radius", 30.0);
    let curl = g.add_node("force.curl");
    g.set_param(curl, "strength", 1.5);
    g.set_param(curl, "scale", 0.5);
    g.set_param(curl, "speed", 0.4);
    let drag = g.add_node("force.drag");
    g.set_param(drag, "coefficient", 1.0);
    let dup = g.add_node("motion.duplicator");
    let mv = g.add_node("motion.move");
    let out = g.add_node("motion.output");
    wire(g, em, 0, ig, 0, false);
    wire(g, ig, 0, vortex, 0, true);
    wire(g, vortex, 0, curl, 0, false);
    wire(g, curl, 0, drag, 0, false);
    wire(g, drag, 0, ig, 1, false);
    wire(g, forma, 0, dup, 0, false);
    wire(g, ig, 0, dup, 1, false);
    wire(g, dup, 0, mv, 0, false);
    wire(g, mv, 0, out, 0, false);
    for (i, no) in [forma, em, ig, dup, mv, out].into_iter().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "seis nos")]
        let x = i as f32 * 210.0;
        g.set_pos(no, Pos { x, y: 0.0 });
    }
    for (i, no) in [vortex, curl, drag].into_iter().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "tres nos")]
        let x = 420.0 + i as f32 * 180.0;
        g.set_pos(no, Pos { x, y: 150.0 });
    }
    g.set_label(dup, "Dress Every Particle");
    g.set_label(out, "The Ladder");
    out
}

#[cfg(test)]
#[path = "motion_object_smoke_escada_tests.rs"]
mod tests;
