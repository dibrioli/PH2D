//! **Empuxo — a área sabe QUANTO do corpo está dentro dela** (ADR-0131 W-Buoyancy).
//!
//! Fecha a lacuna que o W-AreaDrag deixou nomeada: até aqui, "flutuar" era uma
//! `Force Y` constante para cima vencendo o peso, o que tem três defeitos que um
//! artista sente imediatamente —
//!
//! 1. **não se auto-nivela**: a força não sabe onde está a superfície, então o corpo
//!    leve é arremessado para fora da piscina em vez de parar na linha d'água;
//! 2. **é por MASSA, não por densidade**: o número certo tem de ser re-descoberto para
//!    cada objeto (uma caixa 4× mais pesada precisa de 4× a força), quando a intuição
//!    real é *madeira boia, pedra afunda*, que é uma propriedade do MATERIAL;
//! 3. **não endireita nada**: um barco tombado fica tombado.
//!
//! Arquimedes resolve os três com um número só: a força é `ρ_fluido · |g| · A_submersa`,
//! **para cima**, aplicada no **centroide da parte submersa**. O corpo sobe até a área
//! submersa gerar exatamente o próprio peso — a linha d'água cai de graça. Um corpo
//! mais denso que o fluido nunca chega lá e afunda. E como o centroide se desloca
//! quando o corpo inclina, o braço de alavanca **endireita** o barco sozinho: nada disso
//! é código extra, é a mesma fórmula.
//!
//! ## A superfície é perpendicular à GRAVIDADE, não ao eixo Y
//!
//! Água tem superfície horizontal mesmo numa piscina torta — então a superfície não é a
//! borda de cima do collider, é o plano perpendicular à gravidade que passa pelo ponto
//! mais alto dele. Cai de graça e apaga dois casos especiais: uma zona **rotacionada**
//! se comporta certo, e gravidade lateral (ou zero) também. ⚠️ Com gravidade **zero** não
//! existe empuxo — o que é fisicamente certo, não um caso degenerado a tratar.
//!
//! ## O recorte
//!
//! Sutherland–Hodgman contra um semi-plano, e a área + centroide pela fórmula do
//! shoelace.
//!
//! ⚠️ O polígono vem do **collider VIVO do rapier** ([`local_polygon`]), nunca do
//! `BodyDesc`: é o collider que a escala do `Transform` já alcançou (W6), então uma
//! piscina ou um barco escalados boiam com o tamanho que estão DESENHADOS. Re-derivar a
//! silhueta do descritor seria uma segunda resposta para *"que forma é esta?"*, e as duas
//! divergiriam exatamente onde ninguém olha — num corpo escalado.
//!
//! Para bola e cápsula o rapier não tem vértices (ele as representa exatamente), então
//! elas são tesseladas pelas MESMAS portas que o collider de elipse e o overlay usam
//! (`ellipse_vertices`/`capsule_vertices`). Um polígono regular de N lados inscrito num
//! círculo tem `(N/2π)·sin(2π/N)` da área dele — **0,64% a menos** em `ELLIPSE_SEGS = 32`.
//! Nomeado aqui e pinado num gate, porque é um viés que o empuxo herda.

//! ⚠️ **PONTO e VETOR são o MESMO tipo aqui** ([`crate::rmath`]): o polígono, o centroide e a
//! superfície são LUGARES; a gravidade, o `up` e a força são DESLOCAMENTOS. O compilador já não
//! separa os dois, então cada expressão abaixo diz por escrito qual dos dois ela manipula — este
//! módulo é o sítio da crate onde os dois mais se cruzam (um centro de pressão é um ponto, um
//! empuxo é um vetor, e o que os liga é uma diferença de pontos).

use crate::rmath::{Pose, Vector};
use rapier2d::dynamics::{RigidBodyHandle, RigidBodySet};
use rapier2d::geometry::ColliderSet;

use rapier2d::geometry::Shape;

use super::shape::{capsule_vertices, ellipse_vertices};

/// O polígono do collider **que o solver de fato tem**, em espaço local.
///
/// A porta única de *"que forma é esta?"* para o empuxo. `Cuboid` são seus quatro
/// cantos; `ConvexPolygon` (elipse e stadium, W6) devolve os próprios pontos do casco;
/// `Ball` e `Capsule` — que o rapier representa exatamente — são tesseladas pelas mesmas
/// funções que constroem o collider de elipse e que o overlay desenha, com o viés de
/// 0,64% que o cabeçalho nomeia. `None` para uma forma que este módulo não conhece:
/// nenhum empuxo é melhor que um empuxo sobre uma silhueta inventada.
///
/// ⚠️ O `Vec<Vector>` é uma lista de **PONTOS** (vértices em espaço local), não de
/// deslocamentos — ver o aviso do topo do módulo.
#[must_use]
pub(super) fn local_polygon(shape: &dyn Shape) -> Option<Vec<Vector>> {
    let pt = |[x, y]: [f32; 2]| Vector::new(x, y);
    if let Some(c) = shape.as_cuboid() {
        let (hx, hy) = (c.half_extents.x, c.half_extents.y);
        return Some(vec![
            Vector::new(hx, hy),
            Vector::new(-hx, hy),
            Vector::new(-hx, -hy),
            Vector::new(hx, -hy),
        ]);
    }
    if let Some(b) = shape.as_ball() {
        return Some(
            ellipse_vertices(b.radius, b.radius)
                .into_iter()
                .map(pt)
                .collect(),
        );
    }
    if let Some(c) = shape.as_capsule() {
        // O segmento do rapier é centrado na origem e alinhado em Y (a convenção que
        // `ShapeDesc::Capsule` impõe), então a meia-altura é a do ponto `b`.
        let half_height = c.segment.b.y;
        return Some(
            capsule_vertices(half_height, c.radius, c.radius)
                .into_iter()
                .map(pt)
                .collect(),
        );
    }
    shape.as_convex_polygon().map(|p| p.points().to_vec())
}

/// Área (com sinal, CCW positivo) e centroide de um polígono — fórmula do shoelace.
/// `None` para um polígono degenerado (área ~zero), onde o centroide não existe.
///
/// ⚠️ Entra uma lista de **PONTOS** e sai um **PONTO** (o centroide) — a soma
/// `(p.x + q.x)` é de coordenadas escalares, que é o que a fórmula do shoelace pede, e não
/// uma soma de dois lugares.
#[must_use]
fn area_centroid(poly: &[Vector]) -> Option<(f32, Vector)> {
    if poly.len() < 3 {
        return None;
    }
    let (mut a2, mut cx, mut cy) = (0.0f32, 0.0f32, 0.0f32);
    for i in 0..poly.len() {
        let p = poly[i];
        let q = poly[(i + 1) % poly.len()];
        let cross = p.x * q.y - q.x * p.y;
        a2 += cross;
        cx += (p.x + q.x) * cross;
        cy += (p.y + q.y) * cross;
    }
    if a2.abs() < 1e-12 {
        return None;
    }
    let area = a2 * 0.5;
    Some((area.abs(), Vector::new(cx / (3.0 * a2), cy / (3.0 * a2))))
}

/// Recorta `poly` ao semi-plano `dot(p, up) <= level` — a parte SUBMERSA
/// (Sutherland–Hodgman). O polígono de entrada é convexo, então a saída também é.
///
/// ⚠️ `poly` são **PONTOS** e `up` é uma **DIREÇÃO** — o `dot` de um com o outro é a ALTURA
/// com sinal daquele lugar ao longo do eixo do fluido, que é a única grandeza que este
/// recorte compara com `level`.
#[must_use]
fn clip_below(poly: &[Vector], up: Vector, level: f32) -> Vec<Vector> {
    let mut out: Vec<Vector> = Vec::with_capacity(poly.len() + 1);
    for i in 0..poly.len() {
        let p = poly[i];
        let q = poly[(i + 1) % poly.len()];
        let (dp, dq) = (p.dot(up) - level, q.dot(up) - level);
        if dp <= 0.0 {
            out.push(p);
        }
        // Cruzou a superfície: o ponto exato entra, para que a área varie
        // CONTINUAMENTE com a profundidade. Sem isto o empuxo saltaria de vértice
        // em vértice e o corpo tremeria na linha d'água.
        if (dp < 0.0 && dq > 0.0) || (dp > 0.0 && dq < 0.0) {
            let t = dp / (dp - dq);
            // PONTO + (PONTO − PONTO)·t: `q - p` é a ARESTA (um deslocamento), e somá-la ao
            // lugar `p` devolve um lugar. Nada aqui soma dois pontos.
            out.push(p + (q - p) * t);
        }
    }
    out
}

/// A profundidade da superfície da zona ao longo de `up`: o extremo do collider dela
/// nessa direção. Um fluido tem superfície onde a poça acaba, e a poça é o collider.
///
/// ⚠️ `pose * p` é **transformação de PONTO** (rotação **e** translação) — em `glamx` o
/// `Mul<Vector>` de uma [`Pose`] é o `transform_point`. Um vértice do polígono é um lugar, e
/// levá-lo ao mundo sem a translação daria a altura do collider como se ele estivesse na
/// origem.
#[must_use]
fn surface_level(poly: &[Vector], pose: &Pose, up: Vector) -> f32 {
    poly.iter()
        .map(|p| (pose * *p).dot(up))
        .fold(f32::NEG_INFINITY, f32::max)
}

/// A força de empuxo sobre um corpo, e onde ela é aplicada. `None` quando não há parte
/// submersa nenhuma (o corpo está todo acima da superfície) — o caso comum.
///
/// ⚠️ **Os dois membros da tupla são o MESMO tipo e NÃO são a mesma coisa:** o primeiro é a
/// **FORÇA** (um deslocamento, em newtons) e o segundo é o **CENTRO DE PRESSÃO** (um lugar, em
/// unidades de mundo). Trocá-los compila em silêncio desde que a matemática da rapier deixou de
/// ser `nalgebra` — ver [`crate::rmath`]. Quem chama aplica-os como `(impulso, ponto)`, e é
/// justamente o ponto que produz o momento restaurador que endireita um barco.
#[must_use]
pub(crate) fn buoyant_force(
    body: &dyn Shape,
    body_pose: &Pose,
    zone: &dyn Shape,
    zone_pose: &Pose,
    gravity: Vector,
    fluid_density: f32,
) -> Option<(Vector, Vector)> {
    let g = gravity.length();
    // Sem gravidade não há empuxo — Arquimedes é consequência do peso do fluido, não uma
    // força independente. O caso degenerado se resolve pela própria física.
    if g <= 0.0 || fluid_density <= 0.0 {
        return None;
    }
    // `-gravity / g` é a normalização feita à mão, e o `g > 0` acima é a guarda dela — por isso
    // NÃO se troca por `normalize()`, que no glam é UB-adjacente com vetor nulo (NaN, e um
    // `debug_assert` sob `glam-assert`).
    let up = -gravity / g;
    let level = surface_level(&local_polygon(zone)?, zone_pose, up);
    // Os vértices são PONTOS: `body_pose * p` é `transform_point` (rotação + translação).
    let poly: Vec<Vector> = local_polygon(body)?
        .into_iter()
        .map(|p| body_pose * p)
        .collect();
    let (area, centroid) = area_centroid(&clip_below(&poly, up, level))?;
    Some((up * (fluid_density * g * area), centroid))
}

/// Aplica o empuxo de uma zona a um corpo, por sub-passo.
///
/// ⚠️ **`apply_impulse_at_point`, não `apply_impulse`**: aplicado no centroide da parte
/// submersa, o empuxo gera torque quando esse centroide não está sobre o centro de massa
/// — que é exatamente o momento restaurador que endireita um barco tombado. Uma força no
/// centro de massa faria o corpo boiar igual e nunca endireitaria nada.
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply(
    bodies: &mut RigidBodySet,
    colliders: &ColliderSet,
    body: RigidBodyHandle,
    zone_collider: rapier2d::geometry::ColliderHandle,
    gravity: Vector,
    fluid_density: f32,
    dt: f32,
    scratch: &mut Vec<rapier2d::geometry::ColliderHandle>,
) {
    let Some(zone) = colliders.get(zone_collider) else {
        return;
    };
    {
        let Some(rb) = bodies.get(body) else {
            return;
        };
        super::shapes::sorted_shapes(rb, scratch);
    }
    let Some(b) = bodies.get_mut(body) else {
        return;
    };
    for &h in scratch.iter() {
        // ⚠️ **Um SENSOR não desloca fluido**, e esta é uma pergunta que o
        // `.first()` nunca deixou aparecer. Um sensor é um marcador, não matéria:
        // ele atravessa tudo por definição, e o pé-sensor de um personagem
        // (W-PartSensor) daria empuxo a um pedaço de nada. `shapes::displaces`
        // é a porta única — o shape drag faz a MESMA pergunta uma linha adiante.
        let Some(shape) = colliders.get(h).filter(|c| super::shapes::displaces(c)) else {
            continue;
        };
        // ⚠️ **A pose é a do COLLIDER, não a do corpo.** Para a forma própria de
        // um corpo sem offset as duas coincidem — e é por isso que ler
        // `rb.position()` sobreviveu tanto tempo. Com o **offset** do W-Offset
        // elas já divergiam, e com uma PEÇA divergem sempre.
        let Some((force, at)) = buoyant_force(
            shape.shape(),
            shape.position(),
            zone.shape(),
            zone.position(),
            gravity,
            fluid_density,
        ) else {
            continue;
        };
        // `force * dt` é o IMPULSO (vetor) e `at` é o PONTO de aplicação — a `rapier` 0.35 pede
        // os dois como `Vector`, e a ordem dos argumentos é a única coisa que ainda os separa.
        b.apply_impulse_at_point(force * dt, at, true);
    }
}

/// **A LINHA D'ÁGUA de uma zona** — os dois extremos do segmento onde a superfície
/// corta o collider dela, em unidades de mundo.
///
/// Existe porque a superfície é o único número que este módulo calcula e que nada na
/// tela mostrava: o artista posiciona o tronco *no olho*, sem saber onde ele vai parar.
///
/// ⚠️ **Perpendicular à GRAVIDADE**, pela MESMA [`surface_level`] que o empuxo usa — não
/// por uma re-derivação. Duas respostas para *"onde está a água?"* divergiriam
/// exatamente onde ninguém confere: numa poça rotacionada, ou sob gravidade lateral. E o
/// segmento é recortado à LARGURA do collider, projetando os vértices na tangente, para
/// que a linha acabe onde a poça acaba, em vez de atravessar a cena.
///
/// `None` sem gravidade — sem peso de fluido não há superfície que signifique alguma
/// coisa, a mesma resposta que [`buoyant_force`] dá.
#[must_use]
pub(crate) fn waterline(
    zone: &dyn Shape,
    zone_pose: &Pose,
    gravity: Vector,
) -> Option<([f32; 2], [f32; 2])> {
    let g = gravity.length();
    if g <= 0.0 {
        return None;
    }
    let up = -gravity / g;
    let poly = local_polygon(zone)?;
    let level = surface_level(&poly, zone_pose, up);
    // A tangente da superfície: perpendicular a `up`, no plano.
    let tangent = Vector::new(-up.y, up.x);
    let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
    for p in &poly {
        // Vértice (PONTO) levado ao mundo e projetado na tangente — a coordenada dele ao
        // longo da superfície.
        let t = (zone_pose * *p).dot(tangent);
        lo = lo.min(t);
        hi = hi.max(t);
    }
    if !(lo.is_finite() && hi.is_finite()) {
        return None;
    }
    let at = |t: f32| {
        let v = tangent * t + up * level;
        [v.x, v.y]
    };
    Some((at(lo), at(hi)))
}
