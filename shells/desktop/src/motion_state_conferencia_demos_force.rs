//! **A FAMÍLIA `force.*`** — a cena `=71` (doc 89, folha 02: três células, três nós).
//!
//! Três pares. O mesmo grafo dos dois lados; só o número novo muda.
//!
//! | par | esquerda | direita |
//! |---|---|---|
//! | `force.drag` | o arrasto isotrópico de sempre | **`Drag X`/`Drag Y`** — freia mais num eixo |
//! | `force.vortex` | a rampa `Linear` cravada | **`Curve = Smoother`** — o mesmo aro, outra queda |
//! | `force.buoyancy` | uma densidade para todos | **a coluna `density`** — a rolha e a pedra |
//!
//! ⚠️ **ESTA CENA SÓ SE JULGA COM O PLAY.** Uma força não move nada sozinha: ela
//! acumula em `accel`, e é o `motion.integrate` que a aplica. Parada, as seis bandas
//! são seis nuvens idênticas — a leitura é o CAMINHO que cada uma faz.
//!
//! ⚠️ **O laço é `integrate =pre=> força =fwd=> integrate.forces`** (o molde da cena
//! `=61`): a força vive no cone do integrador, então cada sub-passada volta a
//! perguntar quanta força há.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O vão entre as duas colunas e entre as três linhas.
const GAP_X: f32 = 6.0;
const GAP_Y: f32 = 5.2;
/// A anisotropia que o par 1 autora: o eixo Y freia, o X não.
const DRAG_Y: f32 = 6.0;
/// A densidade que o par 3 dá à ROLHA (a pedra fica no neutro).
const CORK: f32 = 3.0;

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .ok()
}

/// O fio de RETORNO do integrador para a força — o laço que o artista não desenha.
fn wire_pre(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) -> Option<()> {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: true,
    })
    .ok()
}

fn push(g: &mut Graph, head: NodeId, kind: &str, ps: &[(&str, f32)], ey: f32, x: f32) -> NodeId {
    let n = g.add_node(kind);
    g.set_pos(n, Pos { x, y: ey });
    for (k, v) in ps {
        g.set_param(n, *k, *v);
    }
    let _ = wire(g, head, 0, n, 0);
    n
}

/// Uma banda: a semente, o integrador, a força no laço, a cor e a saída.
///
/// `seed` monta o que a banda começa por ser; `force` devolve **(cabeça, ponta)** da
/// cadeia de forças — a cabeça recebe o `pre` do integrador, a ponta alimenta o
/// `forces` dele.
fn band(
    g: &mut Graph,
    ey: f32,
    at: [f32; 2],
    rgb: [f32; 3],
    seed: impl FnOnce(&mut Graph, f32) -> NodeId,
    force: impl FnOnce(&mut Graph, f32) -> (NodeId, NodeId),
) -> Option<NodeId> {
    let start = seed(g, ey);
    let placed = push(
        g,
        start,
        "motion.move",
        &[("dx", at[0]), ("dy", at[1])],
        ey,
        300.0,
    );
    let integ = g.add_node("motion.integrate");
    g.set_pos(integ, Pos { x: 460.0, y: ey });
    wire(g, placed, 0, integ, 0)?;

    // ⚠️ **O `pre` vai à CABEÇA da cadeia de forças e a saída vem da PONTA.** Com uma
    // força só as duas são o mesmo nó; com duas (o par 1: vento + arrasto) não são, e
    // ligar o `pre` à ponta faria o vento ler o estado de dois quadros atrás.
    let (head, tail) = force(g, ey);
    wire_pre(g, integ, 0, head, 0)?;
    wire(g, tail, 0, integ, 1)?;

    let t = push(
        g,
        integ,
        "motion.tint",
        &[("r", rgb[0]), ("g", rgb[1]), ("b", rgb[2])],
        ey,
        760.0,
    );
    let out = g.add_node("motion.output");
    g.set_pos(out, Pos { x: 900.0, y: ey });
    wire(g, t, 0, out, 0)?;
    Some(out)
}

/// Uma grelha quadrada de peças pequenas.
fn grid(g: &mut Graph, side: f32, gap: f32, piece: f32, ey: f32) -> NodeId {
    let n = g.add_node("motion.grid");
    g.set_pos(n, Pos { x: 0.0, y: ey });
    g.set_param(n, "rows", side);
    g.set_param(n, "cols", side);
    g.set_param(n, "gap_x", gap);
    g.set_param(n, "gap_y", gap);
    push(g, n, "motion.scale", &[("amount", piece)], ey, 160.0)
}

/// Monta a cena. Devolve os seis sinks, em pares.
pub(crate) fn build_force_demo_document(
    doc: &mut MotionDoc,
    _registry: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::with_capacity(6);

    for (row, right) in (0..3).flat_map(|r| [(r, false), (r, true)]) {
        let ey = (row * 2 + usize::from(right)) as f32 * 240.0;
        let at = [
            if right { GAP_X } else { -GAP_X },
            GAP_Y - row as f32 * GAP_Y,
        ];
        let on = f32::from(u8::from(right));
        let sink = match row {
            // **PAR 1** — o arrasto anisotrópico. As peças começam a cair na diagonal
            // (uma velocidade inicial igual nos dois eixos); à direita o eixo Y é
            // freado e o X não, então a nuvem CURVA em vez de descer reta.
            0 => band(
                g,
                ey,
                at,
                [0.46, 0.72, 1.0],
                |g, ey| grid(g, 4.0, 0.55, 0.16, ey),
                |g, ey| {
                    // ⚠️ **DUAS forças no laço**, e a ordem é a do fluxo: o vento
                    // empurra na diagonal, o arrasto freia o que ele acelerou. O
                    // `pre` do integrador entra na PRIMEIRA (o `band` liga-o ao nó
                    // devolvido, então o vento é quem o recebe).
                    let w = g.add_node("force.wind");
                    g.set_pos(
                        w,
                        Pos {
                            x: 460.0,
                            y: ey + 140.0,
                        },
                    );
                    g.set_param(w, "angle", 315.0);
                    g.set_param(w, "strength", 7.0);
                    g.set_param(w, "gust", 0.0);
                    let d = g.add_node("force.drag");
                    g.set_pos(
                        d,
                        Pos {
                            x: 620.0,
                            y: ey + 140.0,
                        },
                    );
                    g.set_param(d, "coefficient", 1.0);
                    // ⚠️ O X fica em `1` nos DOIS lados — o que muda é só o Y. É
                    // ESCRITO e não deixado no default: o artista que abre a cena tem
                    // de ver no painel que o par difere num eixo só.
                    g.set_param(d, "scale_x", 1.0);
                    g.set_param(d, "scale_y", 1.0 + on * (DRAG_Y - 1.0));
                    let _ = wire(g, w, 0, d, 0);
                    (w, d)
                },
            )?,
            // **PAR 2** — o perfil do vórtice. O mesmo aro e a mesma força; à direita
            // a queda até a borda é `Smoother`, então o miolo gira mais e a borda
            // solta antes.
            1 => band(
                g,
                ey,
                at,
                [1.0, 0.74, 0.3],
                |g, ey| grid(g, 7.0, 0.42, 0.13, ey),
                |g, ey| {
                    let v = g.add_node("force.vortex");
                    g.set_pos(
                        v,
                        Pos {
                            x: 460.0,
                            y: ey + 140.0,
                        },
                    );
                    g.set_param(v, "strength", 9.0);
                    g.set_param(v, "radius", 2.4);
                    g.set_param(v, "curve", on * 3.0);
                    (v, v)
                },
            )?,
            // **PAR 3** — a densidade por-instância. Uma fileira submersa; à direita
            // um `motion.drive` no canal CUSTOM escreve a coluna `density` a partir de
            // uma rampa, então cada peça flutua com a sua própria densidade.
            _ => band(
                g,
                ey,
                at,
                [0.62, 1.0, 0.66],
                |g, ey| {
                    let row = g.add_node("motion.grid");
                    g.set_pos(row, Pos { x: 0.0, y: ey });
                    g.set_param(row, "rows", 1.0);
                    g.set_param(row, "cols", 8.0);
                    g.set_param(row, "gap_x", 0.62);
                    let fit = push(g, row, "motion.scale", &[("amount", 0.2)], ey, 160.0);
                    if on < 0.5 {
                        return fit;
                    }
                    // A rampa que vira densidade: o índice normalizado.
                    let ramp = g.add_node("value.instance_field");
                    g.set_pos(
                        ramp,
                        Pos {
                            x: 0.0,
                            y: ey + 140.0,
                        },
                    );
                    let drv = g.add_node("motion.drive");
                    g.set_pos(
                        drv,
                        Pos {
                            x: 220.0,
                            y: ey + 140.0,
                        },
                    );
                    // ⚠️ **O canal CUSTOM** é o que torna a coluna alcançável — a
                    // célula pedia um canal por-instância e o escritor já existia.
                    g.set_param(drv, "channel", 9.0);
                    g.set_param(drv, "scale", 4.0);
                    // ⚠️ A chave é `"column"` (`ph2d_node_motion_drive::DRIVE_COL_KEY`),
                    // escrita literal porque a shell não depende daquela drop-crate —
                    // e o gate abaixo confere que a coluna que sai é `density`.
                    g.set_text_param(drv, "column", "density");
                    let _ = wire(g, fit, 0, drv, 0);
                    let _ = wire(g, ramp, 0, drv, 1);
                    drv
                },
                |g, ey| {
                    let b = g.add_node("force.buoyancy");
                    g.set_pos(
                        b,
                        Pos {
                            x: 460.0,
                            y: ey + 140.0,
                        },
                    );
                    g.set_param(b, "level", 1.2);
                    g.set_param(b, "wave_amplitude", 0.0);
                    g.set_param(b, "density", CORK);
                    (b, b)
                },
            )?,
        };
        sinks.push(sink);
    }
    Some(sinks)
}

/// Os rótulos das seis bandas, na ordem em que a cena as monta.
pub(crate) fn band_labels() -> impl Iterator<Item = (usize, &'static str)> {
    [
        "ARRASTO isotropico -- a nuvem desce na diagonal, reta",
        "ARRASTO com Drag Y -- so' o eixo vertical e' freado, e a queda CURVA",
        "VORTICE Linear -- a rampa cravada que sempre shipou",
        "VORTICE Smoother -- o mesmo aro e a mesma forca, outra queda ate' a borda",
        "EMPUXO uma densidade para todos -- a fileira sobe junta",
        "EMPUXO com a coluna `density` -- cada peca flutua na sua propria densidade",
    ]
    .into_iter()
    .enumerate()
}

/// Os números que a mensagem do smoke cita, para ela não os repetir à mão.
pub(crate) fn authored() -> (f32, f32) {
    (DRAG_Y, CORK)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_force_tests.rs"]
mod tests;
