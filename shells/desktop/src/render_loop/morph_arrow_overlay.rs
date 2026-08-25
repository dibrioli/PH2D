//! **AS SETAS DA MÁQUINA DE MORPH, desenhadas no canvas** (plano 32, W3a).
//!
//! Enio, 2026-08-25: *"as setas devem ser desenhadas no canvas onde as formas foram desenhadas"*.
//!
//! # ⚠️ Elas são CHROME, e não desenho
//!
//! Uma seta não é um objecto do documento: não se selecciona como forma, não exporta, não imprime.
//! É por isso que ela nasce aqui — no overlay — e não como um `VecPath` derivado, que é o que o
//! **conector** faz (`connector_live`). *O conector é uma linha que o artista quer ver no produto
//! final; a seta é a explicação de uma regra.*
//!
//! # A geometria é PURA, e é isso que a torna gateável
//!
//! [`arrows`] recebe o grafo, os retângulos das formas **em mundo**, a câmera e a janela — e
//! devolve caminhos em **coordenadas de ecrã**. Nada de `World`, nada de GPU. É o mesmo desenho do
//! `physics_overlay_contacts`, e pelo mesmo motivo: uma sonda que precise de uma janela real não é
//! uma sonda.
//!
//! # ⭐ Duas leis de LEGIBILIDADE que a forma do desenho tem de honrar
//!
//! 1. **`A→B` e `B→A` não se sobrepõem.** Toda máquina útil tem pares de ida e volta, e duas rectas
//!    entre os mesmos dois centros são **uma** recta na tela. As setas **curvam**, e o lado da
//!    curva sai da ORDEM DOS IDS — ⚠️ nunca da ordem em que o artista as desenhou, que muda quando
//!    ele apaga uma.
//! 2. **A ponta encosta na BORDA da forma, não no centro dela.** Uma seta que morre no meio de um
//!    rectângulo grande fica escondida por baixo dele.

use ph2d_morph_machine::{MorphGraph, ShapeId};
use ph2d_render::Camera2d;
use ph2d_vector::{BezPath, Point};

use ph2d_host::WindowSize;

/// **A caixa de uma forma, em MUNDO** — o que o chamador resolve do `VecScene`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ShapeBox {
    pub id: ShapeId,
    /// Centro, em mundo.
    pub center: [f32; 2],
    /// Meias-extensões, em mundo. Podem ser `0` (uma forma degenerada) sem partir nada.
    pub half: [f32; 2],
}

/// **Quanto a seta se afasta da recta**, em px de ECRÃ.
///
/// ⚠️ **Ecrã e não mundo**: uma curvatura em mundo desapareceria ao afastar o zoom, e é exactamente
/// aí — com a máquina inteira à vista — que a ida e a volta precisam de se distinguir. Mesma
/// escolha do `MARK_MIN_PX` do overlay de contactos.
const BEND_PX: f64 = 22.0; // LITERAL-PX-OK: chrome de overlay

/// O comprimento de cada aba da ponta, em px de ecrã.
const HEAD_PX: f64 = 9.0; // LITERAL-PX-OK: chrome de overlay

/// Quanto a ponta recua da borda da forma, para não a tocar.
const GAP_PX: f64 = 4.0; // LITERAL-PX-OK: chrome de overlay

/// Uma seta desenhada.
pub(crate) struct Arrow {
    /// O arco, do bordo de `from` até ao bordo de `to`.
    pub path: BezPath,
    /// As duas abas da ponta.
    pub head: BezPath,
    /// **É esta a transição em VOO?** O chamador pinta-a com o realce.
    pub live: bool,
}

/// **AS SETAS, em coordenadas de ecrã.**
///
/// Uma aresta cuja forma não está na lista é **saltada** — e não desenhada até ao ponto `(0,0)`:
/// o artista pode ter apagado uma das formas, e uma seta a apontar para o canto do mundo seria
/// pior que a ausência dela. (É a mesma escolha do `morph_live`, que **congela** a forma quando
/// uma fonte some em vez de a fazer sumir.)
pub(crate) fn arrows(
    graph: &MorphGraph,
    boxes: &[ShapeBox],
    live: Option<(ShapeId, ShapeId)>,
    camera: &Camera2d,
    window: WindowSize,
) -> Vec<Arrow> {
    let find = |id: ShapeId| boxes.iter().find(|b| b.id == id).copied();
    graph
        .edges
        .iter()
        .filter_map(|e| {
            let (a, b) = (find(e.from)?, find(e.to)?);
            Some(one(a, b, live == Some((e.from, e.to)), camera, window))
        })
        .collect()
}

/// Uma seta, do bordo de `a` ao bordo de `b`.
fn one(a: ShapeBox, b: ShapeBox, live: bool, camera: &Camera2d, window: WindowSize) -> Arrow {
    let (ca, ha) = screen_box(a, camera, window);
    let (cb, hb) = screen_box(b, camera, window);

    let (dx, dy) = (cb.x - ca.x, cb.y - ca.y);
    let len = dx.hypot(dy);
    // Duas formas no mesmo sítio: sem direcção não há seta que se leia. Devolve caminhos vazios em
    // vez de dividir por zero — o `filter_map` de cima já provou que as formas existem, e uma
    // sobreposição exacta é um facto do documento, não um erro.
    if len < f64::EPSILON {
        return Arrow {
            path: BezPath::new(),
            head: BezPath::new(),
            live,
        };
    }
    let (ux, uy) = (dx / len, dy / len);

    // ⭐ **O LADO da curva sai da DIRECÇÃO DE VIAGEM, e de mais nada.** A normal de um versor
    // (`perp(u)`) já inverte quando ele inverte ⇒ `A→B` e `B→A` bombeiam para lados opostos **por
    // construção**, sem ninguém escolher um lado.
    //
    // ⛔⛔ **Duas tentativas anteriores morreram aqui, e a segunda passou no gate pelo motivo
    // errado.** A 1ª fazia `perp(u) * sign(ids)`: os dois fatores **cancelavam-se** (em `B→A` o
    // versor inverte *e* o sinal inverte), e as duas setas dobravam para o mesmo lado. A 2ª
    // construiu um "versor canónico" `u * sign` e tirou-lhe a normal com o mesmo `sign` — que é
    // **algebricamente `perp(u)`**: o gate ficou verde, mas a mutação `sign = 1.0` não o matava,
    // porque o `sign` não decidia nada. *Um gate verde sobre código morto continua verde.*
    //
    // ⚠️ E a ordem em que o artista desenhou as setas **não entra aqui** — nem pode: esta função
    // não recebe o índice da aresta, e é isso que impede o lado de saltar no dia em que ele apagar
    // uma anterior.
    let (nx, ny) = (-uy, ux);
    let mid = Point::new(
        (ca.x + cb.x) * 0.5 + nx * BEND_PX,
        (ca.y + cb.y) * 0.5 + ny * BEND_PX,
    );

    // Os extremos: onde a recta CENTRO→CENTRO sai de cada caixa, mais a folga.
    let start = exit(ca, ha, (ux, uy), GAP_PX);
    let end = exit(cb, hb, (-ux, -uy), GAP_PX);

    let mut path = BezPath::new();
    path.move_to(start);
    path.quad_to(mid, end);

    // A ponta aponta na direcção com que a curva CHEGA — a tangente do fim de uma quadrática é
    // `end - mid`, e usar a recta centro-a-centro faria a ponta olhar para o lado numa seta curva.
    let (tx, ty) = (end.x - mid.x, end.y - mid.y);
    let tl = tx.hypot(ty).max(f64::EPSILON);
    let (tx, ty) = (tx / tl, ty / tl);
    let mut head = BezPath::new();
    for s in [1.0_f64, -1.0] {
        // ±30°, escrito como a rotação do versor de chegada.
        let (c, si) = (
            (30.0_f64).to_radians().cos(),
            (30.0_f64).to_radians().sin() * s,
        );
        let (bx, by) = (-(tx * c - ty * si), -(tx * si + ty * c));
        head.move_to(end);
        head.line_to(Point::new(end.x + bx * HEAD_PX, end.y + by * HEAD_PX));
    }
    Arrow { path, head, live }
}

/// Centro e meias-extensões de uma forma, em px de ecrã.
fn screen_box(b: ShapeBox, camera: &Camera2d, window: WindowSize) -> (Point, (f64, f64)) {
    let (cx, cy) = camera.world_to_screen(b.center, window);
    // As meias-extensões vêm da DIFERENÇA de duas projecções, e não de um factor de escala lido à
    // parte: assim elas continuam certas se a câmera ganhar um dia rotação ou uma projecção que
    // não seja um simples afim de eixos.
    let (ex, _) = camera.world_to_screen([b.center[0] + b.half[0], b.center[1]], window);
    let (_, ey) = camera.world_to_screen([b.center[0], b.center[1] + b.half[1]], window);
    (
        Point::new(f64::from(cx), f64::from(cy)),
        (f64::from(ex - cx).abs(), f64::from(ey - cy).abs()),
    )
}

/// **Onde a semi-recta `centro + t·u` sai da caixa**, mais `gap` px.
///
/// ⚠️ Uma caixa degenerada (meia-extensão zero nos dois eixos) devolve o próprio centro deslocado
/// pela folga — em vez de `∞`, que é o que a divisão daria.
fn exit(c: Point, half: (f64, f64), u: (f64, f64), gap: f64) -> Point {
    let mut t = f64::INFINITY;
    for (h, d) in [(half.0, u.0), (half.1, u.1)] {
        if d.abs() > f64::EPSILON {
            t = t.min(h / d.abs());
        }
    }
    let t = if t.is_finite() { t } else { 0.0 } + gap;
    Point::new(c.x + u.0 * t, c.y + u.1 * t)
}

#[cfg(test)]
#[path = "morph_arrow_overlay_tests.rs"]
mod tests;

/// **A cor das setas** — o âmbar dos joints da física é o vocabulário de *"isto é uma relação
/// autorada entre dois objectos"*, e uma seta de máquina é exactamente isso.
///
/// ⚠️ **Um valor de chrome, não um token do documento** — mesma família do `CONTACT_RGBA`.
const ARROW_RGBA: [f32; 4] = [1.0, 0.72, 0.25, 0.75]; // LITERAL-COLOR-OK: overlay de maquina

/// A transição em VOO, no mesmo âmbar a opacidade cheia — é o mesmo vocabulário a dizer
/// *"esta, agora"*, tal como o flash do contacto faz.
const ARROW_LIVE_RGBA: [f32; 4] = [1.0, 0.72, 0.25, 1.0]; // LITERAL-COLOR-OK: overlay de maquina

const STROKE_PX: f64 = 1.5; // LITERAL-PX-OK: chrome de overlay
const STROKE_LIVE_PX: f64 = 2.5; // LITERAL-PX-OK: chrome de overlay

/// **O que uma máquina precisa para ser desenhada** — colhido onde o `VecScene` e os afins deste
/// frame estão vivos, e consumido onde a câmera está.
///
/// ⚠️ **Dois tempos, um frame** — o mesmo desenho do `vec_blend_overlay`: colher no sítio onde os
/// afins acabaram de ser construídos e pintar no sítio onde a câmera existe. Guardar isto entre
/// frames faria a seta descrever o mundo do frame anterior.
pub(crate) struct MachineView {
    pub graph: MorphGraph,
    pub boxes: Vec<ShapeBox>,
    /// O par que o `VecMorph` desta entidade mostra AGORA — é ele que marca a seta viva.
    ///
    /// ⚠️ **Vem do componente, e não de uma máquina em memória.** O estado vivo da máquina só
    /// nasce na W5; até lá, *"que par a cena mostra"* é um facto que já está no mundo, e lê-lo
    /// aqui não inventa uma segunda fonte.
    pub live: Option<(ShapeId, ShapeId)>,
}

/// **Colhe as máquinas do mundo**, com as caixas das formas em MUNDO.
pub(crate) fn gather(
    sim: &ph2d_ecs::SimWorld,
    scene: &ph2d_vec_scene::VecScene,
    xforms: &ph2d_vec_scene::VecXforms,
    map: &crate::vec_entities::VecEntityMap,
) -> Vec<MachineView> {
    map.iter()
        .filter_map(|(_, &bits)| {
            let e = ph2d_ecs::Entity::from_bits(bits);
            let m = sim.world().get::<ph2d_ecs::VecMorphMachine>(e)?;
            let live = sim
                .world()
                .get::<ph2d_ecs::VecMorph>(e)
                .map(|v| (v.sources[0], v.sources[1]));
            let boxes = m
                .graph
                .shapes()
                .iter()
                .filter_map(|&id| world_box(scene, xforms, id))
                .collect();
            Some(MachineView {
                graph: m.graph.clone(),
                boxes,
                live,
            })
        })
        .collect()
}

/// A caixa de uma forma, em mundo — `None` se ela já não existe na cena.
fn world_box(
    scene: &ph2d_vec_scene::VecScene,
    xforms: &ph2d_vec_scene::VecXforms,
    id: ShapeId,
) -> Option<ShapeBox> {
    let mut p = scene.paths().iter().find(|p| p.id == id)?.clone();
    ph2d_vec_scene::bake_xform(&mut p, &ph2d_vec_scene::xform_of(xforms, id));
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for v in &p.verts {
        for k in 0..2 {
            lo[k] = lo[k].min(v.anchor[k]);
            hi[k] = hi[k].max(v.anchor[k]);
        }
    }
    if lo[0] > hi[0] {
        return None; // um path sem vértices — não tem centro que se aponte
    }
    #[allow(clippy::cast_possible_truncation)] // LITERAL-PX-OK: mundo em f32, como a camera
    Some(ShapeBox {
        id,
        center: [
            ((lo[0] + hi[0]) * 0.5) as f32,
            ((lo[1] + hi[1]) * 0.5) as f32,
        ],
        half: [
            ((hi[0] - lo[0]) * 0.5) as f32,
            ((hi[1] - lo[1]) * 0.5) as f32,
        ],
    })
}

/// **Desenha as setas de todas as máquinas.**
pub(crate) fn draw(
    views: &[MachineView],
    camera: &Camera2d,
    window: WindowSize,
    scene: &mut ph2d_vector::VectorScene,
) {
    use ph2d_vector::{Affine, Brush, Color, Stroke};
    for v in views {
        for a in arrows(&v.graph, &v.boxes, v.live, camera, window) {
            let (rgba, w) = if a.live {
                (ARROW_LIVE_RGBA, STROKE_LIVE_PX)
            } else {
                (ARROW_RGBA, STROKE_PX)
            };
            for p in [&a.path, &a.head] {
                scene.inner_mut().stroke(
                    &Stroke::new(w),
                    Affine::IDENTITY,
                    &Brush::Solid(Color::new(rgba)),
                    None,
                    p,
                );
            }
        }
    }
}
