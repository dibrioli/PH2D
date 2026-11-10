//! **Os pontos do «Outline» no canvas** (W5, plano `f2_contorno_e_formas_plano.md`): clicar seleciona (Shift
//! alterna), arrastar move, soltar sobre outro ponto do anel funde (o arrastado sai), clicar numa aresta acrescenta,
//! Alt+clique e Delete tiram. Só com a FORMA em edição (a entidade do `ColliderOutline`; o corpo não tem pontos).
//! A verdade são os anéis Q4 da imagem; o gesto escreve-os inteiros e marca «Custom».
//!
//! **Anéis válidos:** nenhuma aresta cruza outra (de qualquer anel), cada anel guarda o sentido e área não nula,
//! e nunca menos de 3 pontos. O arrasto que cruzaria é RECUSADO: o ponto fica na última posição válida do gesto;
//! tirar abaixo de 3 ou criando um cruzamento é recusado (nada muda).

use crate::{AGARRE_PX, Medida, Pose, caixa};
use bevy_ecs::prelude::Resource;
use bevy_ecs::world::World;
use ph2d_ecs::Entity;
use ph2d_host::WindowSize;
use ph2d_physics_ecs::{Collider, ColliderOutline, ColliderShape, OutlineRing as Ring};
use ph2d_render::Camera2d;
use std::sync::Arc;

/// Um ponto: o anel (0 = o de fora) e o índice nele.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ponto {
    pub anel: usize,
    pub i: usize,
}

/// O arrasto de um ponto: os anéis do `Down` (o `Esc` devolve-os) e a folga da pega.
#[derive(Clone, Debug, PartialEq)]
pub struct ArrastoDoPonto {
    pub ponto: Ponto,
    pub inicio: Arc<[Ring]>,
    pub custom: bool,
    pub(crate) folga: [f32; 2],
    pub(crate) ecra0: (f32, f32),
    /// O ponteiro do último `Move`: soltar sobre outro ponto funde.
    pub ecra: (f32, f32),
    pub armado: bool,
}

/// O gesto dos pontos e os pontos selecionados (ordenados, sem repetição).
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct PontosNoCanvas {
    pub arrasto: Option<ArrastoDoPonto>,
    pub selecionados: Vec<Ponto>,
}

/// A forma em edição com os seus pontos: só a entidade do `ColliderOutline` com anéis, numa forma `Outline`.
#[must_use]
pub fn do_alvo(world: &World, alvo: u64) -> Option<(Pose, Medida, ColliderOutline)> {
    let e = Entity::from_bits(alvo);
    let col = world.get::<Collider>(e)?;
    let o = world.get::<ColliderOutline>(e)?;
    (matches!(col.shape, ColliderShape::Outline { .. }) && !o.rings.is_empty())
        .then(|| Some((Pose::de(world, e)?, Medida::da_forma(world, e)?, o.clone())))?
}

/// Os anéis no referencial escalado: o centro `offset × escala` mais a lei de [`ColliderOutline::to_metres`].
#[must_use]
pub fn locais(pose: &Pose, m: &Medida, o: &ColliderOutline) -> Vec<Vec<[f32; 2]>> {
    let ColliderShape::Outline { half_x, half_y } = m.shape else {
        return Vec::new();
    };
    let (c, _) = caixa(pose, m);
    let s = ph2d_core::Vec2::new(pose.s[0], pose.s[1]);
    o.rings_m([half_x, half_y], s)
        .into_iter()
        .map(|r| r.into_iter().map(|p| [c[0] + p[0], c[1] + p[1]]).collect())
        .collect()
}

/// O inverso: um ponto do referencial escalado em Q4 da imagem.
#[must_use]
pub fn para_q4(pose: &Pose, m: &Medida, o: &ColliderOutline, e: [f32; 2]) -> [i32; 2] {
    let ColliderShape::Outline { half_x, half_y } = m.shape else {
        return [0, 0];
    };
    let (c, _) = caixa(pose, m);
    let pw = 16.0 * o.picture_px[0].max(1) as f32;
    let ph = 16.0 * o.picture_px[1].max(1) as f32;
    let u = (e[0] - c[0]) / (2.0 * half_x * pose.s[0]) + 0.5;
    let v = 0.5 - (e[1] - c[1]) / (2.0 * half_y * pose.s[1]);
    [(u * pw).round() as i32, (v * ph).round() as i32]
}

/// Os pontos em ecrã.
#[must_use]
pub fn no_ecra(
    pose: &Pose,
    m: &Medida,
    o: &ColliderOutline,
    cam: &Camera2d,
    win: WindowSize,
) -> Vec<Vec<(f32, f32)>> {
    locais(pose, m, o)
        .into_iter()
        .map(|r| {
            r.into_iter()
                .map(|q| cam.world_to_screen(pose.para_mundo(q), win))
                .collect()
        })
        .collect()
}

/// O ponto mais próximo do ponteiro dentro do dedo.
#[must_use]
pub fn ponto_em(ecra: &[Vec<(f32, f32)>], s: (f32, f32)) -> Option<Ponto> {
    let mut melhor: Option<(Ponto, f32)> = None;
    for (anel, r) in ecra.iter().enumerate() {
        for (i, p) in r.iter().enumerate() {
            let d = (p.0 - s.0).hypot(p.1 - s.1);
            if d <= AGARRE_PX && melhor.is_none_or(|(_, b)| d < b) {
                melhor = Some((Ponto { anel, i }, d));
            }
        }
    }
    melhor.map(|(p, _)| p)
}

/// A aresta mais próxima dentro do dedo: `(anel, i)` = a aresta de `i` a `i + 1`, e o parâmetro `t` nela.
#[must_use]
pub fn aresta_em(ecra: &[Vec<(f32, f32)>], s: (f32, f32)) -> Option<(Ponto, f32)> {
    let mut melhor: Option<(Ponto, f32, f32)> = None;
    for (anel, r) in ecra.iter().enumerate() {
        for i in 0..r.len() {
            let (a, b) = (r[i], r[(i + 1) % r.len()]);
            let (dx, dy) = (b.0 - a.0, b.1 - a.1);
            let l2 = dx * dx + dy * dy;
            if l2 <= f32::EPSILON {
                continue;
            }
            let t = (((s.0 - a.0) * dx + (s.1 - a.1) * dy) / l2).clamp(0.0, 1.0);
            let d = (a.0 + t * dx - s.0).hypot(a.1 + t * dy - s.1);
            if d <= AGARRE_PX && melhor.is_none_or(|(_, _, b)| d < b) {
                melhor = Some((Ponto { anel, i }, t, d));
            }
        }
    }
    melhor.map(|(p, t, _)| (p, t))
}

fn area2(r: &Ring) -> i64 {
    (0..r.len())
        .map(|i| {
            let (a, b) = (r[i], r[(i + 1) % r.len()]);
            i64::from(a[0]) * i64::from(b[1]) - i64::from(b[0]) * i64::from(a[1])
        })
        .sum()
}

fn orienta(a: [i32; 2], b: [i32; 2], c: [i32; 2]) -> i64 {
    let d = |p: [i32; 2], q: [i32; 2]| [i64::from(q[0] - p[0]), i64::from(q[1] - p[1])];
    let (u, v) = (d(a, b), d(a, c));
    (u[0] * v[1] - u[1] * v[0]).signum()
}

/// Os segmentos `ab` e `cd` tocam-se (inclusive colineares sobrepostos)?
fn tocam(a: [i32; 2], b: [i32; 2], c: [i32; 2], d: [i32; 2]) -> bool {
    let (o1, o2, o3, o4) = (
        orienta(a, b, c),
        orienta(a, b, d),
        orienta(c, d, a),
        orienta(c, d, b),
    );
    let entre = |p: [i32; 2], q: [i32; 2], r: [i32; 2]| {
        r[0] >= p[0].min(q[0])
            && r[0] <= p[0].max(q[0])
            && r[1] >= p[1].min(q[1])
            && r[1] <= p[1].max(q[1])
    };
    (o1 != o2 && o3 != o4)
        || (o1 == 0 && entre(a, b, c))
        || (o2 == 0 && entre(a, b, d))
        || (o3 == 0 && entre(c, d, a))
        || (o4 == 0 && entre(c, d, b))
}

/// **Os anéis são válidos?** ≥ 3 pontos, o sentido de `antes` (área não nula) e nenhuma aresta toca outra
/// que não seja a vizinha no ponto partilhado.
#[must_use]
pub fn validos(antes: &[Ring], rings: &[Ring]) -> bool {
    if antes.len() != rings.len() {
        return false;
    }
    let sentido = antes
        .iter()
        .zip(rings)
        .all(|(a, r)| r.len() >= 3 && area2(r) != 0 && area2(r).signum() == area2(a).signum());
    if !sentido {
        return false;
    }
    let seg: Vec<(usize, usize, [i32; 2], [i32; 2])> = rings
        .iter()
        .enumerate()
        .flat_map(|(k, r)| (0..r.len()).map(move |i| (k, i, r[i], r[(i + 1) % r.len()])))
        .collect();
    for (x, &(k1, i1, a, b)) in seg.iter().enumerate() {
        for &(k2, i2, c, d) in &seg[x + 1..] {
            if k1 == k2 {
                let n = rings[k1].len();
                if (i1 + 1) % n == i2 || (i2 + 1) % n == i1 {
                    continue;
                }
            }
            if tocam(a, b, c, d) {
                return false;
            }
        }
    }
    true
}

/// Leva o ponto `p` a `q`; `None` = cruzaria (recusado).
#[must_use]
pub fn movido(rings: &[Ring], p: Ponto, q: [i32; 2]) -> Option<Vec<Ring>> {
    let mut v = rings.to_vec();
    *v.get_mut(p.anel)?.get_mut(p.i)? = q;
    validos(rings, &v).then_some(v)
}

/// Acrescenta `q` depois do ponto `p` (na aresta `p → p + 1`); `None` = inválido.
#[must_use]
pub fn acrescentado(rings: &[Ring], p: Ponto, q: [i32; 2]) -> Option<Vec<Ring>> {
    let mut v = rings.to_vec();
    v.get_mut(p.anel)?.insert(p.i + 1, q);
    validos(rings, &v).then_some(v)
}

/// Tira o ponto `p`; `None` = ficaria com menos de 3 ou cruzaria.
#[must_use]
pub fn tirado(rings: &[Ring], p: Ponto) -> Option<Vec<Ring>> {
    let mut v = rings.to_vec();
    let r = v.get_mut(p.anel)?;
    if r.len() <= 3 || p.i >= r.len() {
        return None;
    }
    r.remove(p.i);
    validos(rings, &v).then_some(v)
}

/// Tira vários pontos, do fim para o começo de cada anel; o que deixaria um anel abaixo de 3 ou cruzaria é recusado
/// e os outros saem. `None` = nada saiu.
#[must_use]
pub fn tirados(rings: &[Ring], pontos: &[Ponto]) -> Option<Vec<Ring>> {
    let mut v = pontos.to_vec();
    v.sort_unstable_by(|a, b| b.cmp(a));
    v.dedup();
    let mut cur = rings.to_vec();
    let mut tirou = false;
    for p in v {
        if let Some(n) = tirado(&cur, p) {
            cur = n;
            tirou = true;
        }
    }
    tirou.then_some(cur)
}

/// O alvo da fusão: outro ponto do MESMO anel sob o ponteiro (o arrastado não conta).
#[must_use]
pub fn alvo_da_fusao(ecra: &[Vec<(f32, f32)>], arrastado: Ponto, s: (f32, f32)) -> Option<Ponto> {
    let r = ecra.get(arrastado.anel)?;
    r.iter()
        .enumerate()
        .filter(|(i, _)| *i != arrastado.i)
        .map(|(i, p)| (i, (p.0 - s.0).hypot(p.1 - s.1)))
        .filter(|(_, d)| *d <= AGARRE_PX)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| Ponto {
            anel: arrastado.anel,
            i,
        })
}

/// Grava os anéis na forma (a pré-visualização é o documento; o undo pós-quadro fecha o gesto num passo).
pub fn escreve(world: &mut World, alvo: u64, rings: Vec<Ring>, custom: bool) {
    if let Some(mut o) = world.get_mut::<ColliderOutline>(Entity::from_bits(alvo)) {
        o.rings = Arc::from(rings);
        o.custom = custom;
    }
}
