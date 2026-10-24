//! **«Simular» recozido** (`docs/Macros/17_W19_plano.md` §2.3–2.4): dentro de um `Simula` o palco corre um
//! rapier PRIVADO — os corpos da macro em `S(ts)` com a pose de mundo, e os corpos `Static` da cena de
//! destino fora da macro como obstáculos (decisão 9 do dono) — dirigido pela fita do evento (decisão 10).
//! O tique é `de + (t − ts)/dt`, preso a `ate`; depois do evento a pose fica no `ate` até um passo da
//! trilha voltar a escrever o `Transform`. A ponte guarda-se: tique a avançar ⇒ só os tiques novos;
//! tique para trás ⇒ retoma do ponto ≤ tique (um a cada [`RETOMA`]; medição no plano §3, C').
//! No `ate` a pose confere-se, bit a bit, com a que o artista viu ([`Simula::conferencia`]).

use super::formato::{Quem, RAIZ, Simula};
use super::palco::{Contexto, Trilha};
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, StableId, Transform};
use ph2d_editor_core::macro_edits::ErroDaMacroView;
use ph2d_editor_core::macro_etapas::erros::{MUNDO_DIFERE, erro_com};
use ph2d_macro::MacroTime;
use ph2d_physics_ecs::{BodyKind, FrozenScene, InputTape, PhysicsBridge, RigidBody};
use std::collections::BTreeMap;

/// O passo fixo do relógio da física (o `fixed_dt` da shell, que converte a cabeça em tiques).
pub const DT: f64 = 1.0 / ph2d_core::time::DEFAULT_HZ;
/// Um ponto de retoma a cada tantos tiques: o pior scrub para trás replaya `RETOMA − 1` passos
/// (`docs/Macros/17_W19_plano.md` §3, a tabela C').
pub const RETOMA: u64 = 60;
/// O tecto de RAM dos pontos de um `Simula` (medido: 1,34 kB por corpo e ponto): 100 corpos guardam
/// ~8 min de corrida; acima disso o anel larga os mais antigos e esse scrub replaya do repouso.
pub const TECTO_DOS_PONTOS: usize = 64 * 1024 * 1024;

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
    /// Quantas vezes o tique andou para trás e se retomou de um ponto.
    pub retomas: u32,
}

impl Recozido {
    /// O tique em que o rapier privado está.
    #[must_use]
    pub fn tique(&self) -> u64 {
        self.ponte.last_stepped()
    }

    /// Os passos do rapier privado desde que nasceu (o pior scrub mede-se por eles).
    #[must_use]
    pub fn passos(&self) -> u64 {
        self.ponte.steps_taken()
    }

    /// Os pontos de retoma guardados e o que custam em bytes.
    #[must_use]
    pub fn pontos(&self) -> (usize, usize) {
        self.ponte.ring_stats()
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

/// O que nasce no mundo privado.
enum Nasce {
    Obstaculo(Entity),
    Raiz,
    Objeto(u32),
}

/// **Monta o mundo privado** do `Simula` que começa em `ts`. Nasce tudo pela ordem do `Entity` vivo
/// (a da ponte: `bodies` é um `BTreeMap<Entity, _>` e o spawn ordena por `to_bits`), para os handles do
/// rapier serem os da corrida gravada; quem não está vivo vem depois, pelo `StableId`.
fn monta(
    tr: &Trilha,
    vivo: &mut SimWorld,
    cx: &Contexto<'_>,
    dono: Entity,
    ts: MacroTime,
    entidade: &dyn Fn(u32) -> Option<Entity>,
) -> (SimWorld, BTreeMap<u32, Entity>) {
    let mut p = SimWorld::new();
    let em = tr.lida.em(ts);
    let da_raiz = |id: u32, o: &super::estado::ObjetoEm| id == RAIZ || o.quem == Quem::Macro;
    let mut ordem: Vec<((u8, u64), Nasce)> = vivo
        .world_mut()
        .query::<(Entity, &RigidBody)>()
        .iter(vivo.world())
        .filter(|(e, c)| c.kind == BodyKind::Static && !dentro_de(vivo.world(), *e, dono))
        .map(|(e, _)| ((0, e.to_bits()), Nasce::Obstaculo(e)))
        .collect();
    ordem.push(((0, dono.to_bits()), Nasce::Raiz));
    for (&id, o) in em.iter().filter(|(id, o)| o.vivo && !da_raiz(**id, o)) {
        let k = entidade(id).map_or((1, o.original), |e| (0, e.to_bits()));
        ordem.push((k, Nasce::Objeto(id)));
    }
    ordem.sort_by_key(|(k, _)| *k);
    // O k-ésimo `Entity` privado (por `to_bits`, a ordem da ponte) é o k-ésimo vivo: o sentido em que o
    // ECS numera não entra.
    let mut pes: Vec<Entity> = (0..ordem.len())
        .map(|_| p.world_mut().spawn_empty().id())
        .collect();
    pes.sort_by_key(|e| e.to_bits());
    let mut locais = BTreeMap::new();
    for ((_, n), pe) in ordem.into_iter().zip(pes) {
        match n {
            Nasce::Obstaculo(e) => {
                for r in cx.registo.iter() {
                    if let Ok(Some(b)) = (r.serialize)(vivo.world(), e) {
                        let _ = (r.insert_from_bytes)(p.world_mut(), pe, &b);
                    }
                }
                if let Some(t) = ph2d_ecs::world_transform(vivo.world(), e) {
                    p.world_mut().entity_mut(pe).insert(t);
                }
            }
            Nasce::Raiz => {
                locais.insert(RAIZ, pe);
                for (&id, o) in em.iter().filter(|(id, o)| o.vivo && da_raiz(**id, o)) {
                    p.world_mut().entity_mut(pe).insert(StableId(o.original));
                    poe_tudo(cx, p.world_mut(), pe, &o.componentes);
                    locais.insert(id, pe);
                }
                let mundo_do_dono =
                    ph2d_ecs::world_transform(vivo.world(), dono).unwrap_or_default();
                p.world_mut().entity_mut(pe).insert(mundo_do_dono);
            }
            Nasce::Objeto(id) => {
                let o = &em[&id];
                p.world_mut().entity_mut(pe).insert(StableId(o.original));
                poe_tudo(cx, p.world_mut(), pe, &o.componentes);
                locais.insert(id, pe);
            }
        }
    }
    let raiz = locais[&RAIZ];
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

/// A pose local do corpo `id` no mundo privado, no referencial do documento (a raiz fala no do pai
/// do objecto Macro).
fn local_de(r: &Recozido, id: u32, pai_do_dono: Transform) -> Option<Transform> {
    let local = *r.sim.world().get::<Transform>(*r.locais.get(&id)?)?;
    if id == RAIZ {
        Transform::inverse_compose(pai_do_dono, local)
    } else {
        Some(local)
    }
}

/// **A impressão dos obstáculos** de `dono`: os corpos `Static` de fora dele (o `RigidBody`, o
/// `Collider` e a pose de mundo), por ordem dos próprios bytes — a mesma cena dá a mesma; outro chão,
/// outra. O `StableId` não entra: a cena de destino pode ter o mesmo chão com outra identidade.
#[must_use]
pub fn impressao(w: &ph2d_ecs::World, dono: Entity) -> [u8; 32] {
    let mut linhas: Vec<Vec<u8>> = w
        .iter_entities()
        .filter(|e| {
            e.get::<RigidBody>()
                .is_some_and(|c| c.kind == BodyKind::Static)
        })
        .filter(|e| !dentro_de(w, e.id(), dono))
        .map(|e| {
            let mut b = postcard::to_allocvec(&e.get::<RigidBody>()).unwrap_or_default();
            let c = e.get::<ph2d_physics_ecs::Collider>();
            b.extend(postcard::to_allocvec(&c).unwrap_or_default());
            b.extend(
                postcard::to_allocvec(&ph2d_ecs::world_transform(w, e.id())).unwrap_or_default(),
            );
            b
        })
        .collect();
    linhas.sort();
    let mut h = blake3::Hasher::new();
    for l in &linhas {
        h.update(&(l.len() as u64).to_le_bytes());
        h.update(l);
    }
    *h.finalize().as_bytes()
}

/// **A conferência** no `ate`: a pose recozida de cada corpo contra a que o artista viu, bit a bit —
/// só sobre os obstáculos com que correu ([`Simula::obstaculos`]); noutra cena cair noutro sítio é a lei.
fn confere(
    r: &Recozido,
    s: &Simula,
    pai_do_dono: Transform,
    vivo: &SimWorld,
    entidade: &dyn Fn(u32) -> Option<Entity>,
) -> Option<ErroDaMacroView> {
    let dono = entidade(RAIZ)?;
    if impressao(vivo.world(), dono) != s.obstaculos {
        return None;
    }
    s.conferencia.iter().find_map(|&(id, viu)| {
        let t = local_de(r, id, pai_do_dono)?;
        let deu = [t.translation.x, t.translation.y, t.rotation];
        if deu.map(f32::to_bits) == viu.map(f32::to_bits) {
            return None;
        }
        let nome = entidade(id)
            .and_then(|e| vivo.world().get::<Name>(e))
            .map_or_else(
                || ph2d_i18n::tr("panel.macros.step.world.unnamed").to_owned(),
                |n| n.0.clone(),
            );
        let d = (deu[0] - viu[0]).hypot(deu[1] - viu[1]);
        let graus = (deu[2] - viu[2]).to_degrees().abs();
        Some(erro_com(
            MUNDO_DIFERE,
            format!("{nome} \u{b7} {d:.4} m \u{b7} {graus:.2}\u{b0}"),
        ))
    })
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
        .is_none_or(|r| r.simula != i || (cx.retoma.is_none() && tique < r.ponte.last_stepped()));
    if refaz {
        let do_zero = tr.recozido.as_ref().map_or(0, |r| r.do_zero) + 1;
        let (sim, locais) = monta(tr, vivo, cx, dono, ts, entidade);
        let mut ponte = PhysicsBridge::new();
        ponte.set_settings(cx.definicoes);
        if let Some(n) = cx.retoma {
            ponte.set_checkpoint_policy(n, TECTO_DOS_PONTOS);
        }
        tr.recozido = Some(Recozido {
            simula: i,
            sim,
            ponte,
            fita: InputTape::from_wire(&s.fita),
            locais,
            do_zero,
            retomas: 0,
        });
    }
    if let Some(r) = tr.recozido.as_mut()
        && (tique != r.ponte.last_stepped() || refaz)
    {
        r.retomas += u32::from(tique < r.ponte.last_stepped());
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
    if tique == u64::from(s.ate) && !tr.conferidos.contains_key(&i) {
        tr.conferidos
            .insert(i, confere(r, &s, pai_do_dono, vivo, entidade));
    }
    for (&id, &pe) in &r.locais {
        let dinamico = r
            .sim
            .world()
            .get::<RigidBody>(pe)
            .is_some_and(|c| c.kind.solver_owns_pose());
        if !dinamico || tr.ultima_pose(id, t).is_some_and(|p| p > ts) || entidade(id).is_none() {
            continue;
        }
        let Some(local) = local_de(r, id, pai_do_dono) else {
            continue;
        };
        if let Ok(b) = postcard::to_allocvec(&local) {
            out.insert(id, b);
        }
    }
    out
}

/// **O FIM no destino** (o aplicar): cada corpo que acaba num `Simula` fica no documento com a pose que
/// a MESMA reprodução dá nos obstáculos DESTE mundo — ao acabar o clipe, nada salta. Um passo posterior
/// que reescreve a pose ganha (já está no FIM).
pub(crate) fn fim_no_destino(
    t: &ph2d_macro::Track,
    sim: &mut SimWorld,
    registo: &ph2d_ecs::scene::ComponentRegistry,
    dono: Entity,
) {
    let Ok(lida) = super::estado::Lida::de(t) else {
        return;
    };
    if lida.simulas().next().is_none() {
        return;
    }
    let quem: BTreeMap<u32, (Quem, u64)> = lida
        .objetos()
        .map(|o| (o.id, (o.quem, o.original)))
        .collect();
    let sids: BTreeMap<u64, Entity> = sim
        .world_mut()
        .query::<(Entity, &StableId)>()
        .iter(sim.world())
        .map(|(e, s)| (s.0, e))
        .collect();
    let entidade = |id: u32| match quem.get(&id) {
        _ if id == RAIZ => Some(dono),
        Some((Quem::Macro, _)) => Some(dono),
        Some((_, sid)) => sids.get(sid).copied(),
        None => None,
    };
    let definicoes = super::palco::com(|p| p.definicoes).unwrap_or_default();
    let cx = Contexto::novo(registo, definicoes);
    let mut tr = Trilha::de(lida);
    let poses = poses(&mut tr, sim, &cx, dono, MacroTime(u64::MAX), &entidade);
    for (id, b) in poses {
        if let (Some(e), Ok(pose)) = (entidade(id), postcard::from_bytes::<Transform>(&b)) {
            sim.world_mut().entity_mut(e).insert(pose);
        }
    }
}
