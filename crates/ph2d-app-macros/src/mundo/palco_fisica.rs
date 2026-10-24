//! **«Simular» recozido** (`docs/Macros/17_W19_plano.md` §2.3–2.4): dentro de um `Simula` o palco corre um
//! rapier PRIVADO — os corpos da macro em `S(ts)` com a pose de mundo, e os corpos `Static` da cena de
//! destino fora da macro como obstáculos (decisão 9 do dono) — dirigido pela fita do evento (decisão 10).
//! O tique é `de + (t − ts)/dt`, preso a `ate`; depois do evento a pose fica no `ate` até um passo da
//! trilha voltar a escrever o `Transform`. A ponte guarda-se: tique a avançar ⇒ só os tiques novos;
//! tique para trás ⇒ recoze do zero.

use super::formato::{Quem, RAIZ};
use super::palco::{Contexto, Trilha};
use ph2d_ecs::{ChildOf, Entity, SimWorld, StableId, Transform};
use ph2d_macro::MacroTime;
use ph2d_physics_ecs::{BodyKind, FrozenScene, InputTape, PhysicsBridge, RigidBody};
use std::collections::BTreeMap;

/// O passo fixo do relógio da física (o `fixed_dt` da shell, que converte a cabeça em tiques).
pub const DT: f64 = 1.0 / ph2d_core::time::DEFAULT_HZ;

/// O rapier privado de um `Simula`.
pub struct Recozido {
    /// O índice do `Simula` entre os da trilha.
    simula: usize,
    sim: SimWorld,
    ponte: PhysicsBridge,
    fita: InputTape,
    /// Os corpos da macro: id local → entidade privada.
    locais: BTreeMap<u32, Entity>,
    /// Quantas vezes se recozeu do zero (os gates do scrub).
    pub do_zero: u32,
}

impl Recozido {
    /// O tique em que o rapier privado está.
    #[must_use]
    pub fn tique(&self) -> u64 {
        self.ponte.last_stepped()
    }
}

fn dentro_de(w: &ph2d_ecs::World, mut e: Entity, dono: Entity) -> bool {
    loop {
        if e == dono {
            return true;
        }
        match w.get::<ChildOf>(e) {
            Some(c) => e = c.0,
            None => return false,
        }
    }
}

fn poe_tudo(cx: &Contexto<'_>, w: &mut ph2d_ecs::World, e: Entity, cs: &BTreeMap<String, Vec<u8>>) {
    for (n, b) in cs {
        if let Some(r) = cx.registo.get_by_name(n) {
            let _ = (r.insert_from_bytes)(w, e, b);
        }
    }
}

/// **Monta o mundo privado** do `Simula` que começa em `ts`.
fn monta(
    tr: &Trilha,
    vivo: &mut SimWorld,
    cx: &Contexto<'_>,
    dono: Entity,
    ts: MacroTime,
) -> (SimWorld, BTreeMap<u32, Entity>) {
    let mut p = SimWorld::new();
    // Os obstáculos primeiro (a ordem de nascimento é a da ponte: a da gravação).
    let estaticos: Vec<Entity> = vivo
        .world_mut()
        .query::<(Entity, &RigidBody)>()
        .iter(vivo.world())
        .filter(|(e, c)| c.kind == BodyKind::Static && !dentro_de(vivo.world(), *e, dono))
        .map(|(e, _)| e)
        .collect();
    for e in estaticos {
        let pe = p.world_mut().spawn_empty().id();
        for r in cx.registo.iter() {
            if let Ok(Some(b)) = (r.serialize)(vivo.world(), e) {
                let _ = (r.insert_from_bytes)(p.world_mut(), pe, &b);
            }
        }
        if let Some(t) = ph2d_ecs::world_transform(vivo.world(), e) {
            p.world_mut().entity_mut(pe).insert(t);
        }
    }
    let em = tr.lida.em(ts);
    let mut locais = BTreeMap::new();
    let raiz = p.world_mut().spawn_empty().id();
    locais.insert(RAIZ, raiz);
    for (&id, o) in em.iter().filter(|(_, o)| o.vivo) {
        let pe = if id == RAIZ || o.quem == Quem::Macro {
            raiz
        } else {
            p.world_mut().spawn_empty().id()
        };
        p.world_mut().entity_mut(pe).insert(StableId(o.original));
        poe_tudo(cx, p.world_mut(), pe, &o.componentes);
        locais.insert(id, pe);
    }
    let mundo_do_dono = ph2d_ecs::world_transform(vivo.world(), dono).unwrap_or_default();
    p.world_mut().entity_mut(raiz).insert(mundo_do_dono);
    for (&id, o) in em.iter().filter(|(_, o)| o.vivo) {
        let pe = locais[&id];
        if pe != raiz {
            let pai = o.pai.and_then(|q| locais.get(&q).copied()).unwrap_or(raiz);
            p.world_mut().entity_mut(pe).insert(ChildOf(pai));
        }
    }
    ph2d_ecs::assign_master_pieces(p.world_mut());
    (p, locais)
}

/// **As poses do rapier privado em `t`** (blobs do `Transform` LOCAL), por id local; vazio fora de um
/// `Simula` ou para um corpo cuja pose um passo posterior reescreveu.
pub(super) fn poses(
    tr: &mut Trilha,
    vivo: &mut SimWorld,
    cx: &Contexto<'_>,
    dono: Entity,
    t: MacroTime,
    entidade: &dyn Fn(u32) -> Option<Entity>,
) -> BTreeMap<u32, Vec<u8>> {
    let mut out = BTreeMap::new();
    let Some((i, ts, s)) = tr
        .lida
        .simulas()
        .enumerate()
        .filter(|(_, (ts, _))| *ts <= t)
        .last()
        .map(|(i, (ts, s))| (i, ts, s.clone()))
    else {
        tr.recozido = None;
        return out;
    };
    let decorrido = (t.0 - ts.0) as f64 / 1e6;
    let tique = (u64::from(s.de) + (decorrido / DT).round() as u64).min(u64::from(s.ate));
    let refaz = tr
        .recozido
        .as_ref()
        .is_none_or(|r| r.simula != i || tique < r.ponte.last_stepped());
    if refaz {
        let do_zero = tr.recozido.as_ref().map_or(0, |r| r.do_zero) + 1;
        let (sim, locais) = monta(tr, vivo, cx, dono, ts);
        let mut ponte = PhysicsBridge::new();
        ponte.set_settings(cx.definicoes);
        tr.recozido = Some(Recozido {
            simula: i,
            sim,
            ponte,
            fita: InputTape::from_wire(&s.fita),
            locais,
            do_zero,
        });
    }
    if let Some(r) = tr.recozido.as_mut()
        && (tique > r.ponte.last_stepped() || refaz)
    {
        r.ponte.dispatch_with_scene_and_tape(
            &mut r.sim,
            true,
            tique,
            &mut FrozenScene,
            &mut r.fita,
        );
    }
    let Some(r) = tr.recozido.as_ref() else {
        return out;
    };
    let pai_do_dono = ph2d_ecs::parent_world_transform(vivo.world(), dono);
    for (&id, &pe) in &r.locais {
        let w = r.sim.world();
        let dinamico = w
            .get::<RigidBody>(pe)
            .is_some_and(|c| c.kind.solver_owns_pose());
        if !dinamico || tr.ultima_pose(id, t).is_some_and(|p| p > ts) || entidade(id).is_none() {
            continue;
        }
        let Some(&local) = w.get::<Transform>(pe) else {
            continue;
        };
        let local = if id == RAIZ {
            match Transform::inverse_compose(pai_do_dono, local) {
                Some(l) => l,
                None => continue,
            }
        } else {
            local
        };
        if let Ok(b) = postcard::to_allocvec(&local) {
            out.insert(id, b);
        }
    }
    out
}
