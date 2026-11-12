//! **O Look da forma de colisão no Inspector e no canvas** (GEF02 onda 8): o arrasto das alças escreve pelo centro
//! real da forma, e o `offset` cru volta pelo pivô do [`ph2d_physics_ecs::ShapeLook`] (a porta `ph2d_physics_ecs::shape_look`).

use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::{Entity, SimWorld, Transform, World};
use ph2d_editor_core::{PhysicsFieldEdit, ShapeLookFacts};
use ph2d_inspector_ordering::queue_set;
use ph2d_physics_ecs::shape_look::meia_extensao;
use ph2d_physics_ecs::{Collider, ColliderShape, ShapeLook};

/// Abaixo disto uma meia-extensão é degenerada: o pivô nesse eixo fica no centro.
const MEIA_MINIMA: f32 = 1e-6;
/// Dois pivôs a menos disto (em fração) são o mesmo ponto da grade.
const MESMO_PONTO: f32 = 1e-4;

/// O pivô (fração) do ponto `[coluna, linha]` da grade, a linha 0 em cima.
#[must_use]
pub fn fracao_do_ponto(p: [u8; 2]) -> [f32; 2] {
    [f32::from(p[0]) - 1.0, 1.0 - f32::from(p[1])]
}

/// ⭐ O que o cartão Look lê da forma `e`; `on` = `e` é um objeto «Collision Shape» (um `Collider` sem corpo).
#[must_use]
pub fn fatos(world: &World, e: Entity, col: &Collider, on: bool) -> ShapeLookFacts {
    if !on {
        return ShapeLookFacts::NONE;
    }
    let l = world.get::<ShapeLook>(e).copied().unwrap_or_default();
    let h = meia_extensao(col.shape);
    let ponto = (0..3_u8)
        .flat_map(|r| (0..3_u8).map(move |c| [c, r]))
        .find(|q| {
            let f = fracao_do_ponto(*q);
            (f[0] - l.pivot[0]).abs() < MESMO_PONTO && (f[1] - l.pivot[1]).abs() < MESMO_PONTO
        });
    let corpo = ph2d_physics_ecs::governing_kind(world, e);
    let traco = crate::contorno::Tom::from(&l)
        .traco
        .unwrap_or_else(|| crate::contorno::traco_do_corpo(col, corpo));
    ShapeLookFacts {
        on,
        ponto,
        metros: [l.pivot[0] * h[0], l.pivot[1] * h[1]],
        tint: l.tint,
        opacity: l.opacity,
        cor_propria: l.cor_propria,
        camada: col.layer,
        largura: traco.largura as f32,
        tracejado: traco.padrao.is_some(),
        traco: traco.padrao.map_or(l.traco, |p| p.map(|v| v as f32)),
        traco_proprio: l.traco_proprio,
    }
}

/// ⭐ As edições do Look. `false` = não é uma delas. O pivô novo deixa a forma onde está no mundo: a origem do
/// objeto-forma anda para o pivô, o `offset` dobra-se nela (fica zero) e os filhos ficam no lugar. Tudo vai pela fila
/// de comandos, que é o passo de undo (e o Auto Key lê os canais do `ShapeLook` e da posição).
pub fn aplica(
    sim: &SimWorld,
    e: Entity,
    edit: PhysicsFieldEdit,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) -> bool {
    const LOOK: &str = "ph2d::physics::ShapeLook";
    let world = sim.world();
    let bits = e.to_bits();
    let mut l = world.get::<ShapeLook>(e).copied().unwrap_or_default();
    let antes = l;
    let Some(col) = world.get::<Collider>(e).copied() else {
        return matches!(
            edit,
            PhysicsFieldEdit::LookPivotPoint(_)
                | PhysicsFieldEdit::LookPivotX(_)
                | PhysicsFieldEdit::LookPivotY(_)
                | PhysicsFieldEdit::LookOpacity(_)
                | PhysicsFieldEdit::LookTint(_)
                | PhysicsFieldEdit::LookStroke { .. }
        );
    };
    let h = meia_extensao(col.shape);
    let de_metros = |m: f32, i: usize| if h[i] > MEIA_MINIMA { m / h[i] } else { 0.0 };
    let pivo = match edit {
        PhysicsFieldEdit::LookPivotPoint(p) => Some(fracao_do_ponto(p)),
        PhysicsFieldEdit::LookPivotX(m) => Some([de_metros(m, 0), l.pivot[1]]),
        PhysicsFieldEdit::LookPivotY(m) => Some([l.pivot[0], de_metros(m, 1)]),
        PhysicsFieldEdit::LookOpacity(v) => {
            l.opacity = v;
            None
        }
        PhysicsFieldEdit::LookTint(c) => {
            l.tint = c;
            l.cor_propria = true;
            None
        }
        PhysicsFieldEdit::LookStroke {
            width,
            dashed,
            dash,
        } => {
            (l.largura, l.tracejado, l.traco) = (width, dashed, dash);
            l.traco_proprio = true;
            None
        }
        _ => return false,
    };
    if let Some(p) = pivo {
        l.pivot = p;
        poe_o_pivo(sim, e, &col, antes.pivot, p, queue, registry);
    }
    l.sanitize();
    if l != antes {
        queue_set(queue, registry, bits, LOOK, &l);
    }
    true
}

/// `e` é um objeto «Collision Shape» (um `Collider` sem corpo nem desenho): o alvo do Look e da ferramenta Pivot.
#[must_use]
pub fn e_forma(world: &World, e: Entity) -> bool {
    world.get::<Collider>(e).is_some()
        && world.get::<ph2d_physics_ecs::RigidBody>(e).is_none()
        && world.get::<ph2d_render::Sprite>(e).is_none()
}

/// ⭐ A ferramenta Pivot do canvas: a âncora (o centro da forma a partir da origem, local, antes da escala) e a
/// meia-extensão de uma forma de colisão — o par que a Sprite dá do quad. `None` fora de uma forma.
#[must_use]
pub fn anchor_half(world: &World, e: Entity) -> Option<([f32; 2], [f32; 2])> {
    if !e_forma(world, e) {
        return None;
    }
    let col = world.get::<Collider>(e)?;
    let r = ph2d_physics_ecs::forma_resolvida(world, e, col);
    Some((r.offset, meia_extensao(r.shape)))
}

/// O ponto de mundo `p` cai dentro da forma `e`? (o clique da ferramenta Pivot agarra a forma, não só o ponto).
#[must_use]
pub fn contem(world: &World, e: Entity, p: [f32; 2]) -> bool {
    let (Some((c, h)), Some(col), Some(t)) = (
        anchor_half(world, e),
        world.get::<Collider>(e),
        ph2d_ecs::world_transform(world, e),
    ) else {
        return false;
    };
    if t.scale.x.abs() < MEIA_MINIMA || t.scale.y.abs() < MEIA_MINIMA {
        return false;
    }
    let (s, cs) = libm::sincosf(-t.rotation);
    let (dx, dy) = (p[0] - t.translation.x, p[1] - t.translation.y);
    let q = [
        (cs * dx - s * dy) / t.scale.x - c[0],
        (s * dx + cs * dy) / t.scale.y - c[1],
    ];
    match col.shape {
        ColliderShape::Ball { radius } => q[0] * q[0] + q[1] * q[1] <= radius * radius,
        ColliderShape::Capsule {
            half_height,
            radius,
        } => {
            let y = q[1].clamp(-half_height, half_height);
            q[0] * q[0] + (q[1] - y) * (q[1] - y) <= radius * radius
        }
        ColliderShape::Cuboid { .. } | ColliderShape::Outline { .. } => {
            q[0].abs() <= h[0] && q[1].abs() <= h[1]
        }
    }
}

/// ⭐ O arrasto da ferramenta Pivot sobre uma forma: a `origem` (no pai) e a `anchor` nova (o centro da forma a
/// partir dela, local) que o gesto calculou. O pivô é `−anchor / meia-extensão`, o Offset dobra-se, os filhos ficam
/// no mundo. `false` se `e` não é uma forma (o chamador segue com a Sprite).
pub fn arrasta_pivo(world: &mut World, e: Entity, origem: [f32; 2], anchor: [f32; 2]) -> bool {
    let Some((antes, h)) = anchor_half(world, e) else {
        return false;
    };
    let mut l = world.get::<ShapeLook>(e).copied().unwrap_or_default();
    let mut offset = [0.0; 2];
    for i in 0..2 {
        if h[i] > MEIA_MINIMA {
            l.pivot[i] = -anchor[i] / h[i];
        } else {
            l.pivot[i] = 0.0;
            offset[i] = anchor[i];
        }
    }
    let d = [antes[0] - anchor[0], antes[1] - anchor[1]];
    let filhos: Vec<Entity> = world
        .get::<ph2d_ecs::Children>(e)
        .map(|f| f.iter().copied().collect())
        .unwrap_or_default();
    for f in filhos {
        if let Some(mut tf) = world.get_mut::<Transform>(f) {
            tf.translation.x -= d[0];
            tf.translation.y -= d[1];
        }
    }
    if let Some(mut t) = world.get_mut::<Transform>(e) {
        t.translation = ph2d_core::Vec2::new(origem[0], origem[1]);
    }
    if let Some(mut c) = world.get_mut::<Collider>(e) {
        c.offset = offset;
    }
    world.entity_mut(e).insert(l);
    true
}

/// A origem de `e` vai para o pivô `novo` com a forma parada: `d` = centro de antes − centro de agora (local, antes
/// da escala) é o passo da origem; a translação anda `R·(S ⊙ d)` no pai e os filhos `−d`.
fn poe_o_pivo(
    sim: &SimWorld,
    e: Entity,
    col: &Collider,
    velho: [f32; 2],
    novo: [f32; 2],
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) {
    let world = sim.world();
    let Some(t) = world.get::<Transform>(e).copied() else {
        return;
    };
    let h = meia_extensao(col.shape);
    let c0 = ph2d_physics_ecs::shape_look::centro_local(col, velho);
    let c1 = [-novo[0] * h[0], -novo[1] * h[1]];
    let d = [c0[0] - c1[0], c0[1] - c1[1]];
    if d == [0.0, 0.0] && col.offset == [0.0, 0.0] {
        return;
    }
    let (s, c) = libm::sincosf(t.rotation);
    let (sx, sy) = (d[0] * t.scale.x, d[1] * t.scale.y);
    let mut t2 = t;
    t2.translation.x += c * sx - s * sy;
    t2.translation.y += s * sx + c * sy;
    queue_set(queue, registry, e.to_bits(), "ph2d::ecs::Transform", &t2);
    if col.offset != [0.0, 0.0] {
        let sem = Collider {
            offset: [0.0, 0.0],
            ..*col
        };
        queue_set(
            queue,
            registry,
            e.to_bits(),
            "ph2d::physics::Collider",
            &sem,
        );
    }
    let filhos: Vec<Entity> = world
        .get::<ph2d_ecs::Children>(e)
        .map(|f| f.iter().copied().collect())
        .unwrap_or_default();
    for f in filhos {
        if let Some(mut tf) = world.get::<Transform>(f).copied() {
            tf.translation.x -= d[0];
            tf.translation.y -= d[1];
            queue_set(queue, registry, f.to_bits(), "ph2d::ecs::Transform", &tf);
        }
    }
}

/// A forma que as alças deixaram: `medidas` = meias-extensões (bola: `[raio, 0]`; cápsula: `[raio, meia-altura]`)
/// e `centro` = o centro local que elas mostram. Com pivô, o `offset` cru é `centro + pivô ⊙ meia-extensão nova`.
#[must_use]
pub fn forma_arrastada(
    world: &World,
    e: Entity,
    cur: &Collider,
    medidas: [f32; 2],
    centro: [f32; 2],
) -> Collider {
    let shape = match cur.shape {
        ColliderShape::Cuboid { .. } => ColliderShape::Cuboid {
            half_x: medidas[0],
            half_y: medidas[1],
        },
        ColliderShape::Outline { .. } => ColliderShape::Outline {
            half_x: medidas[0],
            half_y: medidas[1],
        },
        ColliderShape::Ball { .. } => ColliderShape::Ball { radius: medidas[0] },
        ColliderShape::Capsule { .. } => ColliderShape::Capsule {
            half_height: medidas[1],
            radius: medidas[0],
        },
    };
    Collider {
        shape,
        offset: ph2d_physics_ecs::shape_look::offset_para_centro(world, e, shape, centro),
        ..*cur
    }
}

#[cfg(test)]
#[path = "look_tests.rs"]
mod tests;
